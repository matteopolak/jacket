//! The file system a root is compiled over.
use jaic::paths::CanonCache;
use jaic::sema::{FileSystem, NativeFs};
use std::path::{Path, PathBuf};

/// The native file system with relative paths taken from `base` (the root's directory), as if
/// the compiler ran there: metaprograms name files relative to where they are built from.
pub(super) struct RootedFs {
    pub(super) base: PathBuf,
    pub(super) virtual_source: Option<(PathBuf, Vec<u8>)>,
    /// Paths that exist, resolved: the compiler asks about the same few hundred files for
    /// every access check and import.
    pub(super) resolved: CanonCache,
}

impl RootedFs {
    fn at(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.base.join(path)
        } else {
            path.to_path_buf()
        }
    }
}

impl FileSystem for RootedFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        if let Some((source, text)) = &self.virtual_source
            && self.at(path) == *source
        {
            return Some(text.clone());
        }
        NativeFs.read(&self.at(path))
    }

    fn is_file(&self, path: &Path) -> bool {
        if self
            .virtual_source
            .as_ref()
            .is_some_and(|(source, _)| self.at(path) == *source)
        {
            return true;
        }
        NativeFs.is_file(&self.at(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        NativeFs.is_dir(&self.at(path))
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        let path = self.at(path);
        // Not there: the compiler's lexical form, asked for again next time.
        self.resolved
            .canonicalize(&path)
            .unwrap_or_else(|_| NativeFs.canonical(&path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        NativeFs.list_dir(&self.at(path))
    }
}
