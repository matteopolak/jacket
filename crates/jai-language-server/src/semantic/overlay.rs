//! A file system that overlays open buffers on disk.
use jaic::sema::FileSystem;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// Every path a compile asked its file system about, by canonical key: files read or tested,
/// folders tested. Compile-time code reads through the same file system, so what `#run` read
/// is here too. A buffer that is none of these, and lies in none of these folders, cannot
/// change the compile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reads {
    files: BTreeMap<PathBuf, FileRead>,
    dirs: BTreeSet<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct FileRead {
    content: bool,
    exists: Option<bool>,
}

impl Reads {
    /// The compile checked only this file's existence, and every check found it present.
    pub fn only_checked_present(&self, path: &Path) -> bool {
        self.files
            .get(path)
            .is_some_and(|read| !read.content && read.exists == Some(true))
    }

    /// Whether a file appearing, going away or changing at `path` could change the compile.
    pub fn concerns(&self, path: &Path) -> bool {
        self.files.contains_key(path) || path.ancestors().any(|dir| self.dirs.contains(dir))
    }
}

/// Open documents over the environment's file system.
pub(super) struct OverlayFs {
    pub(super) base: Rc<dyn FileSystem>,
    /// Where relative paths start (the root file's folder): the server's own working directory
    /// is wherever the editor started it, and a build metaprogram names its files and module
    /// folders relative to itself.
    pub(super) dir: PathBuf,
    pub(super) files: BTreeMap<PathBuf, Arc<[u8]>>,
    /// `canonical` answers so far: a compile asks about the same paths over and over.
    pub(super) canonical: jaic::paths::CanonCache,
    /// What was asked so far; shared with the analysis, since queries after the compile can
    /// load more.
    pub(super) reads: Rc<RefCell<Reads>>,
}

impl OverlayFs {
    fn at<'a>(&self, path: &'a Path) -> std::borrow::Cow<'a, Path> {
        if path.is_relative() {
            self.dir.join(path).into()
        } else {
            path.into()
        }
    }

    fn canonical_of(&self, path: &Path) -> PathBuf {
        self.canonical.get_or_insert_with(path, |path| {
            jaic::paths::file_identity(&*self.base, &self.at(path))
        })
    }
}

impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        let key = self.canonical_of(path);
        let found = self.files.get(&key);
        self.reads
            .borrow_mut()
            .files
            .entry(key)
            .or_default()
            .content = true;
        match found {
            Some(bytes) => Some(bytes.to_vec()),
            None => self.base.read(&self.at(path)),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        let key = self.canonical_of(path);
        let open = self.files.contains_key(&key);
        let exists = open || self.base.is_file(&self.at(path));
        let mut reads = self.reads.borrow_mut();
        let read = reads.files.entry(key).or_default();
        read.exists = Some(read.exists.is_none_or(|old| old) && exists);
        exists
    }

    fn is_dir(&self, path: &Path) -> bool {
        let p = self.canonical_of(path);
        self.reads.borrow_mut().dirs.insert(p.clone());
        self.files.keys().any(|k| k.starts_with(&p) && *k != p) || self.base.is_dir(&self.at(path))
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_of(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let dir = self.canonical_of(path);
        self.reads.borrow_mut().dirs.insert(dir.clone());
        let mut entries: BTreeMap<_, _> = self.base.list_dir(&self.at(path)).into_iter().collect();
        for file in self.files.keys() {
            if let Ok(relative) = file.strip_prefix(&dir) {
                let mut parts = relative.components();
                if let Some(first) = parts.next() {
                    let directory = parts.next().is_some();
                    entries
                        .entry(first.as_os_str().to_string_lossy().into_owned())
                        .and_modify(|old| *old |= directory)
                        .or_insert(directory);
                }
            }
        }
        entries.into_iter().collect()
    }

    fn stamp(&self, path: &Path) -> Option<u64> {
        let key = self.canonical_of(path);
        let open = self
            .files
            .get(&key)
            .map(|bytes| jaic::sema::content_stamp(bytes));
        // What is stamped counts as read: the compile it belongs to depends on it.
        self.reads
            .borrow_mut()
            .files
            .entry(key)
            .or_default()
            .content = true;
        match open {
            Some(stamp) => Some(stamp),
            None => self.base.stamp(&self.at(path)),
        }
    }
}
