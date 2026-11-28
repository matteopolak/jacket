//! Scratch directories that belong to one test.

use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A fresh, empty directory that only its test uses. It is removed when dropped, unless the
/// test is panicking (the files are the evidence) or `JAIC_KEEP_TEST_TMP` is set.
///
/// It dereferences to its path, so `dir.join("main.jai")`, `&dir` where a `&Path` is expected
/// and `.arg(&dir)` all work.
#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    /// A new directory whose name starts with `name`, under [`std::env::temp_dir`] (or under
    /// `JAIC_TEST_TMPDIR` when that names a directory). The process id and a counter make the
    /// name unique, and a name already taken (left by an earlier run) is skipped, so tests never
    /// share a directory and a stale one is never reused. The path is canonical, so it compares
    /// equal to what the compiler prints.
    #[must_use]
    pub fn new(name: &str) -> Self {
        let base = jacket_config::env::TEST_TMPDIR
            .os()
            .map_or_else(std::env::temp_dir, PathBuf::from);

        std::fs::create_dir_all(&base).expect("the test temp directory can be created");
        let clean: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        // `create_dir` fails when the name exists, so a directory an earlier run left behind
        // (a panicked test keeps its files, and a later process can get the same id) is never
        // handed out: the next counter is tried instead.
        let path = loop {
            let path = base.join(format!(
                "jaic-test-{clean}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));

            match std::fs::create_dir(&path) {
                Ok(()) => break path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("cannot create {}: {error}", path.display()),
            }
        };
        let path = path.canonicalize().unwrap_or(path);

        TempDir {
            path,
        }
    }

    /// The directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes `text` to `relative` (creating parent directories) and returns the file's path.
    pub fn write(&self, relative: &str, text: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
        }

        std::fs::write(&path, text)
            .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        path
    }

    /// Writes every `(relative path, text)` pair of `files`.
    pub fn write_all(&self, files: &[(&str, &str)]) {
        for (relative, text) in files {
            self.write(relative, text);
        }
    }

    /// The directory's path, leaving it in place after the test: for a path that outlives the
    /// guard, such as a child process's working directory.
    #[must_use]
    pub fn keep(self) -> PathBuf {
        let path = self.path.clone();

        std::mem::forget(self);
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if std::thread::panicking() || jacket_config::env::KEEP_TEST_TMP.os().is_some() {
            eprintln!("test files kept in {}", self.path.display());
            return;
        }

        let _ = std::fs::remove_dir_all(&self.path);
    }
}

impl Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<OsStr> for TempDir {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_are_unique_and_empty() {
        let a = TempDir::new("same");
        let b = TempDir::new("same");

        assert_ne!(a.path(), b.path());
        assert!(a.read_dir().unwrap().next().is_none());
    }

    #[test]
    fn write_creates_parents() {
        let dir = TempDir::new("write");
        let file = dir.write("a/b/c.txt", "hi");

        assert_eq!(std::fs::read_to_string(file).unwrap(), "hi");
    }

    #[test]
    fn dropping_removes_the_directory() {
        let dir = TempDir::new("drop");
        let path = dir.path().to_path_buf();

        drop(dir);
        assert!(!path.exists());
    }

    #[test]
    fn names_are_made_safe() {
        let dir = TempDir::new("a/b c");

        assert!(dir.exists());
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();

        assert!(name.starts_with("jaic-test-a_b_c-"), "{name}");
    }

    /// A directory a killed or panicked test left behind has the name a later process with the
    /// same id would pick; the new directory must be a different, empty one.
    #[test]
    fn a_leftover_directory_is_never_reused() {
        let base = jacket_config::env::TEST_TMPDIR
            .os()
            .map_or_else(std::env::temp_dir, PathBuf::from);
        let from = NEXT.load(Ordering::Relaxed);
        let stale: Vec<PathBuf> = (from..from + 64)
            .map(|n| base.join(format!("jaic-test-leftover-{}-{n}", std::process::id())))
            .collect();

        for dir in &stale {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("old"), "from an earlier run").unwrap();
        }

        let fresh = TempDir::new("leftover");
        let canonical: Vec<PathBuf> = stale.iter().filter_map(|d| d.canonicalize().ok()).collect();
        let reused = canonical.contains(&fresh.path().to_path_buf());
        let empty = fresh.read_dir().unwrap().next().is_none();

        for dir in &stale {
            let _ = std::fs::remove_dir_all(dir);
        }

        assert!(!reused && empty, "reused a leftover directory");
    }
}
