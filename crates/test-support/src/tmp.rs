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
    cleanup: PathBuf,
}

impl TempDir {
    /// A new directory whose name starts with `name`, under [`std::env::temp_dir`] (or under
    /// `JAIC_TEST_TMPDIR` when that names a directory). The process id and a counter make the
    /// name unique, so tests never share a directory and a stale one is never reused. The
    /// path is canonical, so it compares equal to what the compiler prints.
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
        let path = base.join(format!(
            "jaic-test-{clean}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", path.display()));
        let path = path.canonicalize().unwrap_or(path);
        TempDir {
            cleanup: path.clone(),
            path,
        }
    }

    /// View a private descendant while preserving ownership of the outer scratch tree.
    /// For example, LSP fixtures use `nested("workspace/project/source")` so nearby-name
    /// searches cannot observe another test's files. Dropping the view removes every parent.
    #[cfg(any(feature = "lsp", test))]
    pub(crate) fn nested(mut self, relative: &str) -> Self {
        assert!(
            Path::new(relative)
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_)))
        );
        let path = self.path.join(relative);
        std::fs::create_dir_all(&path).expect("private fixture ancestors can be created");
        self.path = path
            .canonicalize()
            .expect("created private fixture directory");
        self
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

    /// The directory's path, leaving its owned tree in place after the test: for a path that outlives the
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
        let _ = std::fs::remove_dir_all(&self.cleanup);
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

    #[test]
    fn nested_views_keep_outer_cleanup_ownership() {
        let temp = TempDir::new("nested-owner");
        let outer = temp.path().to_path_buf();
        let nested = temp.nested("workspace/project/source");
        let file = nested.write("main.jai", "main :: () {}");
        assert!(file.exists());
        assert!(nested.path().starts_with(&outer));
        drop(nested);
        assert!(
            !outer.exists(),
            "private ancestors belong to the same guard"
        );
    }
}
