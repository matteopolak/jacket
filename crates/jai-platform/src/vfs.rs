use jai_source::{SourceProvider, normalize_virtual_path};
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Debug)]
pub struct VfsLimits {
    pub bytes: usize,
    pub files: usize,
}
impl Default for VfsLimits {
    fn default() -> Self {
        Self {
            bytes: 4 * 1024 * 1024,
            files: 4096,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Store {
    pub(crate) revision: u64,
    pub(crate) files: BTreeMap<PathBuf, Arc<[u8]>>,
    pub(crate) bytes: usize,
    pub(crate) limits: VfsLimits,
}
#[derive(Clone, Debug)]
pub struct SharedVfs {
    pub(crate) root: Arc<str>,
    pub(crate) store: Arc<Mutex<Store>>,
}
#[derive(Clone, Debug)]
pub struct VfsSnapshot {
    pub(crate) root: Arc<str>,
    pub(crate) revision: u64,
    pub(crate) files: BTreeMap<PathBuf, Arc<[u8]>>,
    pub(crate) bytes: usize,
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
impl SharedVfs {
    pub fn new(root: &str, limits: VfsLimits) -> io::Result<Self> {
        let validated = normalize_virtual_path(root, Path::new("__root_validation__"))?;
        let root = validated
            .to_str()
            .unwrap()
            .strip_suffix("/__root_validation__")
            .unwrap();
        let root = if root.is_empty() {
            "/"
        } else {
            root
        };
        Ok(Self {
            root: Arc::from(root),
            store: Arc::new(Mutex::new(Store {
                revision: 0,
                files: BTreeMap::new(),
                bytes: 0,
                limits,
            })),
        })
    }
    /// Publish an embedding-supplied file for the next snapshot; old jobs retain old bytes.
    pub fn insert(&self, name: impl AsRef<Path>, bytes: Vec<u8>) -> io::Result<PathBuf> {
        let path = normalize_virtual_path(&self.root, name.as_ref())?;
        let mut store = self
            .store
            .lock()
            .map_err(|_| invalid("VFS store is poisoned"))?;
        let previous = store.files.get(&path).map_or(0, |bytes| bytes.len());
        let name_bytes = if store.files.contains_key(&path) {
            0
        } else {
            path.as_os_str().as_encoded_bytes().len()
        };
        let total = store
            .bytes
            .checked_sub(previous)
            .and_then(|n| n.checked_add(bytes.len()))
            .and_then(|n| n.checked_add(name_bytes))
            .filter(|n| *n <= store.limits.bytes)
            .ok_or_else(|| invalid("VFS byte limit exceeded"))?;
        if !store.files.contains_key(&path) && store.files.len() >= store.limits.files {
            return Err(invalid("VFS file limit exceeded"));
        }
        let revision = store
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("VFS revision space exhausted"))?;
        store.files.insert(path.clone(), Arc::from(bytes));
        store.bytes = total;
        store.revision = revision;
        Ok(path)
    }
    pub fn snapshot(&self) -> io::Result<VfsSnapshot> {
        let store = self
            .store
            .lock()
            .map_err(|_| invalid("VFS store is poisoned"))?;
        Ok(VfsSnapshot {
            root: self.root.clone(),
            revision: store.revision,
            files: store.files.clone(),
            bytes: store.bytes,
        })
    }
    pub fn root(&self) -> &str {
        &self.root
    }
    pub fn byte_count(&self) -> io::Result<usize> {
        Ok(self
            .store
            .lock()
            .map_err(|_| invalid("VFS store is poisoned"))?
            .bytes)
    }
}
impl VfsSnapshot {
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn byte_count(&self) -> usize {
        self.bytes
    }
    pub fn root(&self) -> &str {
        &self.root
    }
}
impl SourceProvider for VfsSnapshot {
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        normalize_virtual_path(&self.root, path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let path = self.normalize(path)?;
        if self.files.contains_key(&path) {
            Ok(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "virtual source file was not supplied",
            ))
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files
            .get(&self.normalize(path)?)
            .map(|bytes| bytes.to_vec())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "virtual source file was not supplied",
                )
            })
    }
    fn is_file(&self, path: &Path) -> bool {
        self.normalize(path)
            .is_ok_and(|path| self.files.contains_key(&path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshots_pin_observed_source_bytes_and_have_no_native_fallback() {
        let files = SharedVfs::new("/jai-script//", VfsLimits::default()).unwrap();
        assert_eq!(files.root(), "/jai-script");
        for root in ["/sandbox/__root_validation__", "/__root_validation__"] {
            assert_eq!(
                SharedVfs::new(root, VfsLimits::default()).unwrap().root(),
                root
            );
        }
        files
            .insert("main.jai", b"main :: () -> int { return 42; }".to_vec())
            .unwrap();
        let first = files.snapshot().unwrap();
        files
            .insert("main.jai", b"main :: () -> int { return 7; }".to_vec())
            .unwrap();
        let second = files.snapshot().unwrap();
        assert_eq!(
            first.read(Path::new("main.jai")).unwrap(),
            b"main :: () -> int { return 42; }"
        );
        assert_ne!(
            first.read(Path::new("main.jai")).unwrap(),
            second.read(Path::new("main.jai")).unwrap()
        );
        assert!(second.revision() > first.revision());
        assert!(first.read(Path::new("/etc/passwd")).is_err());
    }
    #[test]
    fn replacement_budgets_are_atomic_and_include_file_names() {
        let files = SharedVfs::new(
            "/v",
            VfsLimits {
                bytes: 8,
                files: 1,
            },
        )
        .unwrap();
        files.insert("a", vec![1; 4]).unwrap(); // /v/a name + four payload bytes
        let old = files.snapshot().unwrap();
        assert!(files.insert("a", vec![2; 5]).is_err());
        assert!(files.insert("b", vec![]).is_err());
        let current = files.snapshot().unwrap();
        assert_eq!(old.revision(), current.revision());
        assert_eq!(current.read(Path::new("a")).unwrap(), vec![1; 4]);
    }
}
