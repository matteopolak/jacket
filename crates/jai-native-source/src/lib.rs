//! Native source adapters. Compiler discovery depends on the SourceProvider contract.
use jai_source::SourceProvider;
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::NativeSourceSnapshot;

#[derive(Clone, Copy, Debug, Default)]
pub struct Filesystem;
impl SourceProvider for Filesystem {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            path.canonicalize()
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native source filesystem is unavailable on wasm",
            ))
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read(path)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "native source filesystem is unavailable on wasm",
            ))
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        {
            path.is_file()
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            false
        }
    }
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::normalize(path)
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.canonicalize(path)
        }
    }
}

/// Compatibility native overlay; new portable callers use an explicitly selected base.
pub struct SourceOverlay {
    files: HashMap<PathBuf, Vec<u8>>,
    snapshots: HashMap<PathBuf, jai_source::SourceTextSnapshot>,
    base: std::sync::Arc<dyn SourceProvider>,
}
impl Default for SourceOverlay {
    fn default() -> Self {
        Self::with_base(std::sync::Arc::new(Filesystem))
    }
}
impl std::fmt::Debug for SourceOverlay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceOverlay")
            .field("files", &self.files.keys())
            .finish_non_exhaustive()
    }
}
impl SourceOverlay {
    pub fn with_base(base: std::sync::Arc<dyn SourceProvider>) -> Self {
        Self {
            files: HashMap::new(),
            snapshots: HashMap::new(),
            base,
        }
    }
    pub fn insert_snapshot(
        &mut self,
        path: &Path,
        snapshot: jai_source::SourceTextSnapshot,
    ) -> io::Result<PathBuf> {
        let path = self.base.normalize(path)?;
        self.files
            .insert(path.clone(), snapshot.text().as_bytes().to_vec());
        self.snapshots.insert(path.clone(), snapshot);
        Ok(path)
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, path: &Path, bytes: Vec<u8>) -> io::Result<PathBuf> {
        let path = self.base.normalize(path)?;
        self.snapshots.remove(&path);
        self.files.insert(path.clone(), bytes);
        Ok(path)
    }
}
impl SourceProvider for SourceOverlay {
    fn retain_decoded_text(
        &self,
        path: &Path,
        text: &str,
    ) -> io::Result<jai_source::SourceTextSnapshot> {
        let path = self.normalize(path)?;
        if let Some(snapshot) = self.snapshots.get(&path) {
            if snapshot.text() != text {
                return Err(io::Error::other(
                    "overlay source differs from actual retained snapshot",
                ));
            }
            Ok(snapshot.clone())
        } else if self.files.contains_key(&path) {
            Ok(jai_source::SourceTextSnapshot::new(text.to_owned()))
        } else {
            self.base.retain_decoded_text(&path, text)
        }
    }

    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.base.normalize(path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let normalized = self.normalize(path)?;
        if self.files.contains_key(&normalized) {
            Ok(normalized)
        } else {
            self.base.canonicalize(path)
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let normalized = self.normalize(path)?;
        match self.files.get(&normalized) {
            Some(bytes) => Ok(bytes.clone()),
            None => self.base.read(path),
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        self.normalize(path)
            .is_ok_and(|path| self.files.contains_key(&path))
            || self.base.is_file(path)
    }
}
