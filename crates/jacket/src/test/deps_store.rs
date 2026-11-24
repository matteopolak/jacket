//! The store of dependency lists: for each program `jacket test` compiles, the source files that
//! compile read, kept between runs so `--changed` need not compile a program again to learn them.
//!
//! The compiles jacket does anyway pass `-deps_file <entry>`, and `jaic` writes the list there
//! once the compile has finished, whole or not at all. An entry is a text file of absolute paths,
//! one per line, named by a hash of what decides the list ([`Store::entry`]). A compile that fails
//! leaves no entry: [`Store::prepare`] removes the old one first, and a program with no entry is
//! always kept by `--changed`.
use crate::upgrade::checksum;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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
    jaic: PathBuf,
    /// The compiler's identity, hashed on first use.
    compiler: OnceLock<String>,
}

impl Store {
    /// The store `jacket test` uses: `JACKET_TEST_DEPS_DIR`, else `jacket/test-deps` in the user
    /// cache folder. `None` when neither is known: nothing is recorded then.
    pub fn open(jaic: &Path) -> Option<Self> {
        let dir = match jacket_config::env::TEST_DEPS_DIR
            .os()
            .filter(|v| !v.is_empty())
        {
            Some(dir) => PathBuf::from(dir),
            None => jacket_config::user_cache_dir()
                .ok()?
                .join("jacket")
                .join("test-deps"),
        };
        Some(Self::at(dir, jaic))
    }

    /// The store in `dir`, for the lists `jaic` writes.
    pub fn at(dir: PathBuf, jaic: &Path) -> Self {
        Self {
            dir: std::path::absolute(&dir).unwrap_or(dir),
            jaic: jaic.to_path_buf(),
            compiler: OnceLock::new(),
        }
    }

    /// The compiler's bytes, hashed: a compiler that changes may read other files, and a rebuild
    /// that changes nothing keeps its entries (a cached folder stays useful in CI).
    fn compiler(&self) -> &str {
        self.compiler.get_or_init(|| {
            checksum::sha256_file(&self.jaic)
                .unwrap_or_else(|_| self.jaic.to_string_lossy().into_owned())
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
        hash.part(self.compiler());
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
    pub fn prepare(&self, entry: &Path) {
        let _ = std::fs::create_dir_all(&self.dir);
        let _ = std::fs::remove_file(entry);
    }
}

/// The files listed in `entry`, or `None` when there is no such list.
pub fn read(entry: &Path) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(entry).ok()?;
    Some(text.lines().map(str::to_string).collect())
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
    use super::*;

    fn store(dir: &Path) -> Store {
        let jaic = dir.join("jaic");
        std::fs::write(&jaic, "compiler one").unwrap();
        Store::at(dir.join("store"), &jaic)
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
        let changed = Store::at(dir.join("store"), &dir.join("jaic"));
        assert_ne!(
            base,
            entry(&changed, file, &["-os", "wasm"], false),
            "compiler"
        );
    }

    #[test]
    fn preparing_removes_the_old_list() {
        let dir = temp("prepare");
        let store = store(&dir);
        let entry = entry(&store, Program::Text("x"), &[], false);
        store.prepare(&entry);
        assert_eq!(read(&entry), None);
        std::fs::write(&entry, "/a.jai\n/b.jai\n").unwrap();
        assert_eq!(read(&entry), Some(vec!["/a.jai".into(), "/b.jai".into()]));
        store.prepare(&entry);
        assert_eq!(read(&entry), None, "a failed compile leaves no list");
    }
}
