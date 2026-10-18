//! A file system that overlays open buffers on disk.
use jaic::sema::FileSystem;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Open documents over the environment's file system.
pub(super) struct OverlayFs {
    pub(super) base: Rc<dyn FileSystem>,
    /// Where relative paths start (the root file's folder): the server's own working directory
    /// is wherever the editor started it, and a build metaprogram names its files and module
    /// folders relative to itself.
    pub(super) dir: PathBuf,
    pub(super) files: BTreeMap<PathBuf, Rc<[u8]>>,
    /// `canonical` answers so far: a compile asks about the same paths over and over.
    pub(super) canonical: jaic::paths::CanonCache,
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
            crate::document::file_key(&*self.base, &self.at(path))
        })
    }
}

impl FileSystem for OverlayFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        match self.files.get(&self.canonical_of(path)) {
            Some(bytes) => Some(bytes.to_vec()),
            None => self.base.read(&self.at(path)),
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&self.canonical_of(path)) || self.base.is_file(&self.at(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let p = self.canonical_of(path);
        self.files.keys().any(|k| k.starts_with(&p) && *k != p) || self.base.is_dir(&self.at(path))
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_of(path)
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.base.list_dir(&self.at(path))
    }
}
