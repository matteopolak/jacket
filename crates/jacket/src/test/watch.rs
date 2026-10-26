//! `jacket test --watch`: runs the tests, waits for a `.jai` file to change, and runs them again.
//!
//! Files are found by polling their modification times: the folders are few, and a watcher
//! crate would be a dependency (and a platform backend) for little more. A change is acted on
//! once the files have been still for [`SETTLE`]. If every changed file is a test file the next
//! run only runs that file's tests; any other change (code the tests import) runs all of them.
use super::discover::Root;
use super::run_once;
use crate::cli::TestArgs;
use std::collections::{BTreeMap, BTreeSet};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

/// How often the folders are looked at.
const POLL: Duration = Duration::from_millis(150);

/// How long the files must be unchanged before a run starts, so a save that writes several
/// files (or an editor's temporary one) causes one run.
const SETTLE: Duration = Duration::from_millis(200);

/// Folders never entered: build output, version control and the like.
const SKIPPED: [&str; 3] = ["target", "node_modules", ".git"];

/// What a run used, to know what to watch and which changes it can narrow itself to.
#[derive(Default)]
pub struct Watched {
    folders: BTreeSet<PathBuf>,
    tests: BTreeSet<PathBuf>,
}

impl Watched {
    pub fn clear(&mut self) {
        self.folders.clear();
        self.tests.clear();
    }

    /// The test roots and the folders the compiler is told to import from.
    pub fn set_folders(&mut self, roots: &[Root], flags: &[String], root_flags: &[Vec<String>]) {
        let imports = root_flags
            .iter()
            .flatten()
            .chain(flags)
            .map(String::as_str)
            .collect::<Vec<_>>();
        let named = imports
            .iter()
            .enumerate()
            .filter_map(|(i, flag)| match flag.split_once('=') {
                Some(("-import_dir" | "-I", value)) => Some(value),
                _ if matches!(*flag, "-import_dir" | "-I") => imports.get(i + 1).copied(),
                _ => None,
            });
        let folders = roots
            .iter()
            .map(|r| r.dir.as_path())
            .chain(named.map(Path::new));
        self.folders = folders
            .map(canonical)
            .filter(|folder| folder.is_dir())
            .collect();
        // A folder inside another is already walked.
        let all = self.folders.clone();
        self.folders
            .retain(|f| !all.iter().any(|other| other != f && f.starts_with(other)));
    }

    pub fn set_tests<'a>(&mut self, files: impl Iterator<Item = &'a Path>) {
        self.tests = files.map(canonical).collect();
    }
}

/// `path` with links resolved, so the files a scan finds are the files tests are in.
pub fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// What a scan knows of a file.
type Stamps = BTreeMap<PathBuf, (SystemTime, u64)>;

/// The `.jai` files below `folders`.
fn scan(folders: &BTreeSet<PathBuf>) -> Stamps {
    let mut stamps = Stamps::new();
    let mut pending: Vec<PathBuf> = folders.iter().cloned().collect();
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(&folder).into_iter().flatten().flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                let name = entry.file_name();
                if !SKIPPED.iter().any(|s| name == *s) {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "jai")
                && let Ok(meta) = entry.metadata()
            {
                let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                stamps.insert(path, (modified, meta.len()));
            }
        }
    }
    stamps
}

/// The files that are new, gone or different in `now`.
fn changed(before: &Stamps, now: &Stamps) -> BTreeSet<PathBuf> {
    before
        .keys()
        .chain(now.keys())
        .filter(|path| before.get(*path) != now.get(*path))
        .cloned()
        .collect()
}

/// Waits until a file differs from `stamps`, then until the files stop changing; `stamps`
/// becomes what was found, and the files that differ are returned.
fn wait_for_change(folders: &BTreeSet<PathBuf>, stamps: &mut Stamps) -> BTreeSet<PathBuf> {
    let mut now = scan(folders);
    while now == *stamps {
        std::thread::sleep(POLL);
        now = scan(folders);
    }
    loop {
        std::thread::sleep(SETTLE);
        let later = scan(folders);
        if later == now {
            break;
        }
        now = later;
    }
    let files = changed(stamps, &now);
    *stamps = now;
    files
}

/// Runs the tests, then again after each change, until interrupted (Ctrl-C). A run that fails
/// before it knows what to watch (a bad flag) ends it, with that run's status.
pub fn run(args: &TestArgs, cwd: &Path) -> ExitCode {
    let mut watched = Watched::default();
    let mut only: Option<BTreeSet<PathBuf>> = None;
    let mut stamps: Option<Stamps> = None;
    let human = args.format == crate::cli::OutputFormat::Human;
    loop {
        if human && std::io::stdout().is_terminal() {
            // Clear the screen and the scrollback, and go to the top.
            print!("\x1b[2J\x1b[3J\x1b[H");
        }
        let code = run_once(args, cwd, false, &mut watched, only.as_ref());
        if watched.folders.is_empty() {
            return code;
        }
        let stamps = stamps.get_or_insert_with(|| scan(&watched.folders));
        eprintln!(
            "jacket: watching {} for changes to .jai files; Ctrl-C quits",
            plural(watched.folders.len(), "folder"),
        );
        let files = wait_for_change(&watched.folders, stamps);
        only = narrowed(&files, &watched.tests);
        match &only {
            Some(files) => eprintln!("jacket: {} changed", plural(files.len(), "test file")),
            None => eprintln!("jacket: code changed; running all selected tests"),
        }
    }
}

/// The changed files when each is a test file that still exists.
fn narrowed(files: &BTreeSet<PathBuf>, tests: &BTreeSet<PathBuf>) -> Option<BTreeSet<PathBuf>> {
    (!files.is_empty() && files.iter().all(|f| tests.contains(f) && f.is_file()))
        .then(|| files.clone())
}

fn plural(count: usize, noun: &str) -> String {
    format!(
        "{count} {noun}{}",
        if count == 1 {
            ""
        } else {
            "s"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(paths: &[&str]) -> BTreeSet<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn a_change_to_test_files_narrows_the_run() {
        let tests = set(&["/t/a.jai", "/t/b.jai"]);
        assert_eq!(narrowed(&set(&["/t/a.jai"]), &tests), None, "not on disk");
        let dir = std::env::temp_dir().join(format!("jacket-watch-narrow-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (a, lib) = (dir.join("a.jai"), dir.join("lib.jai"));
        std::fs::write(&a, "").unwrap();
        std::fs::write(&lib, "").unwrap();
        let tests: BTreeSet<PathBuf> = [a.clone()].into();
        assert_eq!(
            narrowed(&[a.clone()].into(), &tests),
            Some([a.clone()].into())
        );
        assert_eq!(narrowed(&[a, lib].into(), &tests), None);
        assert_eq!(narrowed(&BTreeSet::new(), &tests), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scans_see_new_edited_and_removed_jai_files() {
        let dir = std::env::temp_dir().join(format!("jacket-watch-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();
        std::fs::write(dir.join("a.jai"), "one").unwrap();
        std::fs::write(dir.join("sub/b.jai"), "one").unwrap();
        std::fs::write(dir.join("notes.txt"), "one").unwrap();
        std::fs::write(dir.join("target/c.jai"), "one").unwrap();
        let folders: BTreeSet<PathBuf> = [dir.clone()].into();
        let first = scan(&folders);
        assert_eq!(first.len(), 2, "{first:?}");
        std::fs::write(dir.join("a.jai"), "longer").unwrap();
        std::fs::remove_file(dir.join("sub/b.jai")).unwrap();
        std::fs::write(dir.join("d.jai"), "new").unwrap();
        let now = scan(&folders);
        let names: Vec<_> = changed(&first, &now)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["a.jai", "d.jai", "b.jai"]);
        assert!(changed(&now, &now).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_folders_are_watched_with_the_roots() {
        let dir = std::env::temp_dir().join(format!("jacket-watch-dirs-{}", std::process::id()));
        let (root, vendor, inside) = (dir.join("root"), dir.join("vendor"), dir.join("root/src"));
        for folder in [&root, &vendor, &inside] {
            std::fs::create_dir_all(folder).unwrap();
        }
        let flags = [
            "-O0".to_string(),
            "-import_dir".into(),
            vendor.to_string_lossy().into_owned(),
            format!("-I={}", inside.display()),
            "-I".into(),
            root.to_string_lossy().into_owned(),
        ];
        let mut watched = Watched::default();
        watched.set_folders(&[], &flags, &[]);
        let folders: Vec<_> = watched.folders.iter().cloned().collect();
        assert_eq!(folders, [canonical(&root), canonical(&vendor)]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
