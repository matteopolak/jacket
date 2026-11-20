//! The file system a root is compiled over.
use jaic::paths::CanonCache;
use jaic::sema::{FileSystem, NativeFs};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The native file system with relative paths taken from `base` (the root's directory), as if
/// the compiler ran there: metaprograms name files relative to where they are built from.
pub(super) struct RootedFs {
    pub(super) base: PathBuf,
    /// Files read from memory instead of disk, by full path: a module's generated stub, and the
    /// edited text `migrate` checks before anything is written.
    pub(super) overlay: BTreeMap<PathBuf, Arc<[u8]>>,
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

    /// The overlay's text for `path` (already made absolute), under any spelling of it.
    fn overlaid(&self, path: &Path) -> Option<&Arc<[u8]>> {
        if self.overlay.is_empty() {
            return None;
        }
        self.overlay.get(path).or_else(|| {
            let canonical = self.resolved.canonicalize(path).ok()?;
            self.overlay.get(&canonical)
        })
    }
}

impl FileSystem for RootedFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        let path = self.at(path);
        if let Some(text) = self.overlaid(&path) {
            return Some(text.to_vec());
        }
        NativeFs.read(&path)
    }

    fn is_file(&self, path: &Path) -> bool {
        let path = self.at(path);
        self.overlaid(&path).is_some() || NativeFs.is_file(&path)
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
