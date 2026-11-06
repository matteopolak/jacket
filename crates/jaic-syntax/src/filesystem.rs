//! The file system the compiler reads sources through.
use crate::fxhash::HashMap;
use crate::paths::normalize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

/// A small shared owner for read bytes. A weak owner retains only this indirection;
/// dropping its last strong reference releases its byte Arc independently.
#[derive(Debug, PartialEq, Eq)]
pub struct ReadBuffer(Arc<[u8]>);

impl ReadBuffer {
    /// Wrap existing immutable bytes without copying them; for example an open buffer.
    pub fn new(bytes: Arc<[u8]>) -> Self {
        Self(bytes)
    }

    /// The byte allocation captured by comparison proofs, shared without a copy.
    pub fn bytes(&self) -> &Arc<[u8]> {
        &self.0
    }

    /// The read response; for example compare it with a fresh native read.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for ReadBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Source access. Native builds read the filesystem; the browser supplies a
/// virtual file system containing the workspace, prelude and stdlib.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Option<Vec<u8>>;

    /// The same read response with shared immutable ownership. Wrappers record this exact Arc
    /// before returning it, so a proof can own bytes without a second retained allocation.
    /// For example, an observed provider clones the Arc into its Read observation.
    fn read_shared(&self, path: &Path) -> Option<Arc<ReadBuffer>> {
        self.read(path)
            .map(|bytes| Arc::new(ReadBuffer::new(Arc::from(bytes))))
    }

    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;

    fn canonical(&self, path: &Path) -> PathBuf {
        normalize(path)
    }

    /// The real path of `path` when this file system could work it out, which is the only
    /// answer worth remembering: a file that does not exist yet may exist later. A file system
    /// with nothing but lexical answers always has one.
    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        Some(self.canonical(path))
    }

    /// Certify resolving absence and its exact native fallback identity, or return no authority.
    /// `Some(key)` promises that `canonical_exact(path)` is unavailable and
    /// `paths::file_identity(self, path)` would equal `key`. Files, directories, permission
    /// errors and uncertain providers must return `None`; `!is_file(path)` is insufficient.
    /// Callers must replay this operation to reuse an absence fact. For example, an absent
    /// `/project/Mod.jai` can return that normalized identity until the file appears.
    fn missing_file_key(&self, _path: &Path) -> Option<PathBuf> {
        None
    }

    /// Whether this is the real filesystem, which worker threads may read directly.
    fn is_native(&self) -> bool {
        false
    }

    /// Whether observations from this provider require explicit input-authority checks.
    /// A sealed native-derived provider returns `true` while denying direct native I/O.
    /// This flag is not validation: callers must also check `input_epoch_valid` and proof.
    fn tracks_native_inputs(&self) -> bool {
        self.is_native()
    }

    /// Current sealed-epoch readiness, or `None` for a provider without a sealed epoch.
    /// `Some(false)` rejects publication even if placeholder responses match a proof.
    /// For example, an unknown sealed `read` marks its request unavailable without I/O.
    fn input_epoch_valid(&self) -> Option<bool> {
        None
    }

    /// Names (and whether each is a directory) directly inside `path`; used by the sandbox host.
    fn list_dir(&self, _path: &Path) -> Vec<(String, bool)> {
        Vec::new()
    }

    /// A fingerprint of the file or folder at `path` that changes whenever what [`Self::read`] or
    /// [`Self::list_dir`] would answer for it changes, or `None` when this file system cannot say
    /// cheaply. What was worked out from a set of paths is reused while their stamps stay equal
    /// (see `sema::stdlib_index`); without stamps it is worked out again for every compile.
    fn stamp(&self, _path: &Path) -> Option<u64> {
        None
    }
}

/// A [`FileSystem::stamp`] of `bytes`.
pub fn content_stamp(bytes: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut hasher = crate::fxhash::FxHasher::default();
    hasher.write(bytes);
    hasher.write_usize(bytes.len());
    hasher.finish()
}

pub struct NativeFs;

impl FileSystem for NativeFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_exact(path)
            .unwrap_or_else(|| normalize(path))
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        crate::paths::canonicalize(path).ok()
    }

    fn missing_file_key(&self, path: &Path) -> Option<PathBuf> {
        match std::fs::metadata(path) {
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                Some(crate::paths::identity_spelling(&normalize(path)))
            }
            _ => None,
        }
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let Ok(entries) = std::fs::read_dir(path) else {
            return Vec::new();
        };
        entries
            .filter_map(std::result::Result::ok)
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .collect()
    }

    /// Size and modification time (a path that does not exist has a stamp of its own).
    fn stamp(&self, path: &Path) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let mut hasher = crate::fxhash::FxHasher::default();
        match std::fs::metadata(path) {
            Ok(meta) => {
                meta.len().hash(&mut hasher);
                meta.is_dir().hash(&mut hasher);
                meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos())
                    .hash(&mut hasher);
            }
            Err(_) => u64::MAX.hash(&mut hasher),
        }
        Some(hasher.finish())
    }
}

#[derive(Default)]
pub struct VirtualFs {
    files: HashMap<PathBuf, Arc<[u8]>>,
    /// Every directory holding a file, however deep, with the names directly inside it (and
    /// whether each holds more): `is_dir` is asked for each path the compiler canonicalizes and
    /// `list_dir` for each module folder, and a scan of `files` for each is quadratic in the
    /// library size.
    dirs: HashMap<PathBuf, std::collections::BTreeMap<String, bool>>,
    /// `stamp`s of files, worked out when first asked for.
    stamps: std::cell::RefCell<HashMap<PathBuf, u64>>,
}

impl VirtualFs {
    pub fn insert(&mut self, path: impl Into<PathBuf>, bytes: impl Into<Arc<[u8]>>) {
        let path = normalize(&path.into());
        let parts: Vec<_> = path.components().collect();
        let mut dir = PathBuf::new();
        for (i, part) in parts.iter().enumerate() {
            if matches!(
                part,
                std::path::Component::RootDir | std::path::Component::Prefix(_)
            ) {
                dir.push(part);
                continue;
            }
            let name = part.as_os_str().to_string_lossy().into_owned();
            let nested = i + 1 < parts.len();
            let entry = self
                .dirs
                .entry(dir.clone())
                .or_default()
                .entry(name)
                .or_insert(false);
            *entry |= nested;
            dir.push(part);
        }
        self.stamps.get_mut().clear();
        self.files.insert(path, bytes.into());
    }

    /// The files by normalized path.
    pub fn files(&self) -> &HashMap<PathBuf, Arc<[u8]>> {
        &self.files
    }
}

impl FileSystem for VirtualFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.get(&normalize(path)).map(|b| b.to_vec())
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains_key(&normalize(path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.dirs
            .get(&normalize(path))
            .map(|names| names.iter().map(|(n, d)| (n.clone(), *d)).collect())
            .unwrap_or_default()
    }

    /// The contents of a file, the names inside a folder.
    fn stamp(&self, path: &Path) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let path = normalize(path);
        if let Some(stamp) = self.stamps.borrow().get(&path) {
            return Some(*stamp);
        }
        let stamp = if let Some(bytes) = self.files.get(&path) {
            content_stamp(bytes)
        } else if let Some(names) = self.dirs.get(&path) {
            let mut hasher = crate::fxhash::FxHasher::default();
            names.hash(&mut hasher);
            hasher.finish()
        } else {
            u64::MAX
        };
        self.stamps.borrow_mut().insert(path, stamp);
        Some(stamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_listing_names_files_and_folders_directly_inside() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/a.jai", b"".to_vec());
        fs.insert("/lib/b/module.jai", b"".to_vec());
        fs.insert("/lib/b/deep/c.jai", b"".to_vec());
        fs.insert("/lib/b.jai", b"".to_vec());
        fs.insert("/other/x.jai", b"".to_vec());
        assert_eq!(
            fs.list_dir(Path::new("/lib")),
            [
                ("a.jai".into(), false),
                ("b".into(), true),
                ("b.jai".into(), false)
            ]
        );
        assert_eq!(
            fs.list_dir(Path::new("/lib/b/../b/")),
            [("deep".into(), true), ("module.jai".into(), false)]
        );
        assert_eq!(
            fs.list_dir(Path::new("/")),
            [("lib".into(), true), ("other".into(), true)]
        );
        assert!(fs.list_dir(Path::new("/lib/a.jai")).is_empty());
        assert!(fs.is_dir(Path::new("/lib/b/deep")));
        assert!(!fs.is_dir(Path::new("/lib/a.jai")));
    }
}
