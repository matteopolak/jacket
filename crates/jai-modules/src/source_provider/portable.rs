//! Compatibility native entrypoints fail explicitly; supplied virtual overlays are closed.
use jai_source::{SourceProvider, normalize_virtual_path};
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, Default)]
pub struct Filesystem;
impl SourceProvider for Filesystem {
    fn canonicalize(&self, _: &Path) -> io::Result<PathBuf> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native source filesystem is unavailable on wasm",
        ))
    }
    fn read(&self, _: &Path) -> io::Result<Vec<u8>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "native source filesystem is unavailable on wasm",
        ))
    }
    fn is_file(&self, _: &Path) -> bool {
        false
    }
}
#[derive(Debug, Default)]
pub struct SourceOverlay {
    files: BTreeMap<PathBuf, Vec<u8>>,
}
impl SourceOverlay {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, path: &Path, bytes: Vec<u8>) -> io::Result<PathBuf> {
        let path = normalize_virtual_path("/", path)?;
        self.files.insert(path.clone(), bytes);
        Ok(path)
    }
}
impl SourceProvider for SourceOverlay {
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        normalize_virtual_path("/", path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let name = self.normalize(path)?;
        if self.files.contains_key(&name) {
            Ok(name)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "virtual source was not supplied",
            ))
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files
            .get(&self.normalize(path)?)
            .cloned()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "virtual source was not supplied")
            })
    }
    fn is_file(&self, path: &Path) -> bool {
        self.normalize(path)
            .is_ok_and(|path| self.files.contains_key(&path))
    }
}
