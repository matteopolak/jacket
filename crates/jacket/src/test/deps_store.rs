//! The store of dependency lists: for each program `jacket test` compiles, the source files that
//! compile read, kept between runs so `--changed` need not compile a program again to learn them.
//!
//! The compiles jacket does anyway pass `-deps_file <entry>`, and `jaic` writes the list there
//! once the compile has finished, whole or not at all. An entry is a text file of absolute paths,
//! one per line, named by a hash of what decides the list ([`Store::entry`]). A compile that fails
//! leaves no entry: [`Store::prepare`] removes the old one first, and a program with no entry is
//! always kept by `--changed`.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};

use crate::upgrade::checksum;

/// An entry nobody has read or written for this long is deleted when the store is opened.
const MAX_AGE: Duration = Duration::from_hours(30 * 24);

/// The program a list is for.
#[derive(Clone, Copy, Debug)]
pub enum Program<'a> {
    /// A file of the project, named by its absolute path.
    File(&'a Path),
    /// A program jacket generated into a folder that changes from run to run (a root's unit
    /// tests, the stand-in for a compile-fail test): named by its text.
    Text(&'a str),
}

/// Where the lists are kept.
#[derive(Debug)]
pub struct Store {
    dir: PathBuf,
    /// The compiler's identity: its bytes, hashed.
    compiler: String,
}

/// The error for an I/O failure on `path` of the store.
fn store_error(path: &Path, error: &std::io::Error) -> String {
    format!(
        "the folder of recorded test inputs: cannot use `{}`: {}",
        path.display(),
        jaic::io_reason(error)
    )
}

impl Store {
    /// The store `jacket test` uses: `JACKET_TEST_DEPS_DIR`, else `jacket/test-deps` in the user
    /// cache folder. With neither it is an error: every run records what its programs read, so
    /// a later `--changed` can use it.
    pub fn open(jaic: &Path) -> Result<Self, String> {
        let variable = jacket_config::env::TEST_DEPS_DIR.name;
        let dir = match jacket_config::env::TEST_DEPS_DIR
            .os()
            .filter(|v| !v.is_empty())
        {
            Some(dir) => PathBuf::from(dir),
            None => jacket_config::user_cache_dir()
                .map_err(|reason| {
                    format!(
                        "`jacket test` records the files each test program reads, and has no folder for them: {reason}\nnote: set {variable} to a writable folder"
                    )
                })?
                .join("jacket")
                .join("test-deps"),
        };

        Self::open_in(dir, jaic)
    }

    /// [`Store::open`], shared between the threads that run tests.
    pub fn open_shared(jaic: &Path) -> Result<std::sync::Arc<Self>, String> {
        Self::open(jaic).map(std::sync::Arc::new)
    }

    /// The store in `dir`, which must be a folder `jacket test` can write: the compiler writes
    /// its lists there, and a failure would otherwise surface as a failed compile of a test.
    pub fn open_in(dir: PathBuf, jaic: &Path) -> Result<Self, String> {
        crate::prerequisite::writable_dir(
            &dir,
            "the folder of recorded test inputs",
            "jacket test",
        )?;
        Self::opened(dir, jaic)
    }

    /// The store in `dir` with its old entries pruned.
    fn opened(dir: PathBuf, jaic: &Path) -> Result<Self, String> {
        let store = Self::at(dir, jaic)?;

        store.prune(MAX_AGE, SystemTime::now())?;
        Ok(store)
    }

    /// Deletes the entries last read or written more than `max_age` before `now`. A compiler that
    /// changes leaves its entries behind, since the compiler is part of the name; this is what
    /// removes them. An entry that cannot be opened for writing stays: reading cannot renew its
    /// time, so its age says nothing about use.
    fn prune(&self, max_age: Duration, now: SystemTime) -> Result<(), String> {
        let entries = match std::fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(store_error(&self.dir, &error)),
        };

        for entry in entries {
            let entry = entry.map_err(|error| store_error(&self.dir, &error))?;
            let path = entry.path();
            let old = path.extension().is_some_and(|ext| ext == "deps")
                && std::fs::File::options().write(true).open(&path).is_ok()
                && entry
                    .metadata()
                    .and_then(|meta| meta.modified())
                    .is_ok_and(|modified| {
                        now.duration_since(modified).is_ok_and(|age| age > max_age)
                    });

            if old {
                remove(&path)?;
            }
        }

        Ok(())
    }

    /// The store in `dir`, for the lists `jaic` writes. The compiler is read once, here: one that
    /// cannot be read has no identity to name entries by.
    pub fn at(dir: PathBuf, jaic: &Path) -> Result<Self, String> {
        // A compiler that changes may read other files, and a rebuild that changes nothing
        // keeps its entries (a cached folder stays useful in CI).
        let compiler = checksum::sha256_file(jaic).map_err(|error| {
            format!(
                "cannot read the compiler `{}` to name its recorded test inputs: {error}",
                jaic.display()
            )
        })?;

        Ok(Self {
            dir: std::path::absolute(&dir).unwrap_or(dir),
            compiler,
        })
    }

    /// The entry of a compile of `program` with these inputs: the compiler, the standard library,
    /// the flags that decide what is read, the root's flags, and whether the Discovery plugin runs.
    /// The mode (build, `-interp`, `-check`) is not an input: it does not change what is read, so
    /// a list recorded by any of them answers a query.
    pub fn entry(
        &self,
        program: Program,
        stdlib: &Path,
        flags: &[String],
        root_flags: &[String],
        plugin: bool,
    ) -> PathBuf {
        let mut hash = Hash::new();

        hash.part(&self.compiler);
        hash.part(&stdlib.to_string_lossy());

        match program {
            Program::File(path) => {
                hash.part("file");
                let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());

                hash.part(&path.to_string_lossy());
            }
            Program::Text(text) => {
                hash.part("text");
                hash.part(text);
            }
        }

        for list in [flags, root_flags] {
            hash.part(&list.len().to_string());

            for flag in list {
                hash.part(flag);
            }
        }

        hash.part(if plugin {
            "plugin"
        } else {
            "alone"
        });
        self.dir.join(format!("{}.deps", hash.finish()))
    }

    /// Makes `entry` ready for a compile that writes it: the folder exists and no list from an
    /// earlier compile is left, so a compile that fails leaves none.
    pub fn prepare(&self, entry: &Path) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir).map_err(|error| store_error(&self.dir, &error))?;
        remove(entry)
    }
}

#[cfg(test)]
impl Store {
    /// A store of its own in a scratch folder, for a test's toolchain.
    pub fn scratch() -> std::sync::Arc<Self> {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("jacket-scratch-store-{}-{n}", std::process::id()));

        std::fs::create_dir_all(&dir).unwrap();
        let jaic = dir.join("jaic");

        std::fs::write(&jaic, "compiler").unwrap();
        std::sync::Arc::new(Self::at(dir.join("store"), &jaic).unwrap())
    }
}

/// Removes `path`; one that is already gone is fine.
fn remove(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            Err(store_error(path, &error))
        }
        _ => Ok(()),
    }
}

/// The files listed in `entry`, or `None` when there is no such list. Reading counts as use: the
/// entry's time moves to now, so pruning keeps the lists a run still asks for. An entry that
/// cannot be opened for writing is read without that, and is never pruned ([`Store::prune`]).
pub fn read(entry: &Path) -> Result<Option<Vec<String>>, String> {
    let text = match std::fs::read_to_string(entry) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(store_error(entry, &error)),
    };

    if let Ok(file) = std::fs::File::options().write(true).open(entry) {
        file.set_modified(SystemTime::now())
            .map_err(|error| store_error(entry, &error))?;
    }

    Ok(Some(text.lines().map(str::to_string).collect()))
}

/// A SHA-256 over parts that each carry their length, so no two inputs hash the same bytes.
struct Hash(Sha256);

impl Hash {
    fn new() -> Self {
        Self(Sha256::new())
    }

    fn part(&mut self, text: &str) {
        self.0.update(format!("{}:", text.len()));
        self.0.update(text);
    }

    fn finish(self) -> String {
        checksum::hex(self.0.finalize().as_slice())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_store_folder_that_cannot_be_written_is_an_error_naming_it() {
        let dir = test_support::TempDir::new("deps-store-blocked");

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("file"), "").unwrap();
        let error = Store::open_in(dir.join("file").join("store"), Path::new("jaic")).unwrap_err();
        let missing = Store::at(dir.join("store"), Path::new("/nowhere/jaic")).unwrap_err();

        assert!(
            missing.starts_with("cannot read the compiler `/nowhere/jaic`"),
            "{missing}"
        );
        assert!(
            error.contains("cannot write to the folder of recorded test inputs"),
            "{error}"
        );
        assert!(error.contains("`jacket test` needs it\nnote: "), "{error}");
    }

    use super::*;

    fn store(dir: &Path) -> Store {
        let jaic = dir.join("jaic");

        std::fs::write(&jaic, "compiler one").unwrap();
        Store::at(dir.join("store"), &jaic).unwrap()
    }

    fn entry(store: &Store, program: Program, flags: &[&str], plugin: bool) -> PathBuf {
        let flags: Vec<String> = flags.iter().map(ToString::to_string).collect();
        store.entry(program, Path::new("/stdlib"), &flags, &[], plugin)
    }

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("jacket-deps-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn an_entry_is_named_by_what_decides_the_list() {
        let dir = temp("keys");
        let store = store(&dir);
        let file = Program::File(Path::new("/p/a.jai"));
        let base = entry(&store, file, &["-os", "wasm"], false);

        assert_eq!(base, entry(&store, file, &["-os", "wasm"], false));
        assert_ne!(base, entry(&store, file, &[], false), "flags");
        assert_ne!(base, entry(&store, file, &["-os", "wasm"], true), "plugin");
        assert_ne!(
            base,
            entry(
                &store,
                Program::File(Path::new("/p/b.jai")),
                &["-os", "wasm"],
                false
            ),
            "program"
        );
        assert_ne!(
            entry(&store, Program::Text("x"), &[], false),
            entry(&store, Program::File(Path::new("x")), &[], false),
            "text is not a path"
        );
        assert_ne!(
            store.entry(
                file,
                Path::new("/stdlib"),
                &["a".into()],
                &["b".into()],
                false
            ),
            store.entry(
                file,
                Path::new("/stdlib"),
                &["a".into(), "b".into()],
                &[],
                false
            ),
            "flags and root flags are two lists"
        );
        assert_ne!(
            store.entry(file, Path::new("/other"), &[], &[], false),
            store.entry(file, Path::new("/stdlib"), &[], &[], false),
            "standard library"
        );
        std::fs::write(dir.join("jaic"), "compiler two").unwrap();
        let changed = Store::at(dir.join("store"), &dir.join("jaic")).unwrap();

        assert_ne!(
            base,
            entry(&changed, file, &["-os", "wasm"], false),
            "compiler"
        );
    }

    #[test]
    fn pruning_removes_only_old_lists() {
        let dir = temp("prune");
        let store = store(&dir);

        store.prepare(&dir.join("store/x.deps")).unwrap();
        let old = dir.join("store/old.deps");
        let fresh = dir.join("store/fresh.deps");
        let other = dir.join("store/old.txt");

        for file in [&old, &fresh, &other] {
            std::fs::write(file, "/a.jai\n").unwrap();
        }

        let day = Duration::from_hours(24);
        let aged = |file: &Path, days: u32| {
            let handle = std::fs::File::options().write(true).open(file).unwrap();

            handle.set_modified(SystemTime::now() - day * days).unwrap();
        };

        aged(&old, 40);
        aged(&other, 40);
        aged(&fresh, 2);
        store.prune(MAX_AGE, SystemTime::now()).unwrap();
        assert!(!old.exists(), "a list unused for 40 days");
        assert!(fresh.exists());
        assert!(other.exists(), "only lists are the store's to delete");
        // Reading an old list keeps it.
        let kept = dir.join("store/kept.deps");

        std::fs::write(&kept, "/a.jai\n").unwrap();
        aged(&kept, 40);
        assert_eq!(read(&kept).unwrap(), Some(vec!["/a.jai".into()]));
        store.prune(MAX_AGE, SystemTime::now()).unwrap();
        assert!(kept.exists());
    }

    #[test]
    fn opening_prunes_and_a_pruned_list_is_a_miss() {
        let dir = temp("open");
        let jaic = dir.join("jaic");

        std::fs::write(&jaic, "compiler").unwrap();
        let old = dir.join("store/old.deps");

        std::fs::create_dir_all(dir.join("store")).unwrap();
        std::fs::write(&old, "/a.jai\n").unwrap();
        let handle = std::fs::File::options().write(true).open(&old).unwrap();

        handle
            .set_modified(SystemTime::now() - Duration::from_hours(24 * 40))
            .unwrap();
        let store = Store::opened(dir.join("store"), &jaic).unwrap();

        assert_eq!(
            read(&old).unwrap(),
            None,
            "pruned between a query and its read"
        );
        store.prepare(&old).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_list_that_cannot_be_renewed_is_not_pruned() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp("readonly");
        let store = store(&dir);

        store.prepare(&dir.join("store/x.deps")).unwrap();
        let old = dir.join("store/old.deps");

        std::fs::write(&old, "/a.jai\n").unwrap();
        let handle = std::fs::File::options().write(true).open(&old).unwrap();

        handle
            .set_modified(SystemTime::now() - Duration::from_hours(24 * 40))
            .unwrap();
        std::fs::set_permissions(&old, std::fs::Permissions::from_mode(0o444)).unwrap();

        // Root can still open it for writing, and then the entry may go.
        if std::fs::File::options().write(true).open(&old).is_err() {
            store.prune(MAX_AGE, SystemTime::now()).unwrap();
            assert!(old.exists());
        }
    }

    #[test]
    fn preparing_removes_the_old_list() {
        let dir = temp("prepare");
        let store = store(&dir);
        let entry = entry(&store, Program::Text("x"), &[], false);

        store.prepare(&entry).unwrap();
        assert_eq!(read(&entry).unwrap(), None);
        std::fs::write(&entry, "/a.jai\n/b.jai\n").unwrap();
        assert_eq!(
            read(&entry).unwrap(),
            Some(vec!["/a.jai".into(), "/b.jai".into()])
        );
        store.prepare(&entry).unwrap();
        assert_eq!(
            read(&entry).unwrap(),
            None,
            "a failed compile leaves no list"
        );
    }
}
