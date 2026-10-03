//! Generated inputs overlay only an explicitly selected source snapshot.
use jai_source::SourceProvider;
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct SourceOverlay<'a> {
    base: &'a dyn SourceProvider,
    files: BTreeMap<PathBuf, Arc<[u8]>>,
}
impl<'a> SourceOverlay<'a> {
    pub fn new(base: &'a dyn SourceProvider) -> Self {
        Self {
            base,
            files: BTreeMap::new(),
        }
    }
    pub fn insert(&mut self, path: &Path, bytes: Vec<u8>) -> io::Result<PathBuf> {
        let path = self.base.normalize(path)?;
        self.files.insert(path.clone(), Arc::from(bytes));
        Ok(path)
    }
}
impl SourceProvider for SourceOverlay<'_> {
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.base.normalize(path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let name = self.normalize(path)?;
        if self.files.contains_key(&name) {
            Ok(name)
        } else {
            self.base.canonicalize(path)
        }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let name = self.normalize(path)?;
        match self.files.get(&name) {
            Some(bytes) => Ok(bytes.to_vec()),
            None => self.base.read(path),
        }
    }
    fn is_file(&self, path: &Path) -> bool {
        self.normalize(path)
            .is_ok_and(|name| self.files.contains_key(&name))
            || self.base.is_file(path)
    }
}
