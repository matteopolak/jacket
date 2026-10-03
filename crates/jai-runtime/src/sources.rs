//! A closed source bundle: missing imports never fall back to an OS filesystem.
use jai_modules::SourceProvider;
use std::{collections::HashMap, io, path::{Component, Path, PathBuf}};

const ROOT: &str = "/jai-script";
pub const DEFAULT_SOURCE_LIMIT: usize = 4 * 1024 * 1024;

#[derive(Debug)]
pub struct SourceBundle {
    files: HashMap<PathBuf, Vec<u8>>,
    bytes: usize,
    limit: usize,
}
impl Default for SourceBundle {
    fn default() -> Self { Self::new(DEFAULT_SOURCE_LIMIT) }
}
impl SourceBundle {
    pub fn new(limit: usize) -> Self { Self { files: HashMap::new(), bytes: 0, limit } }

    /// Relative names identify explicitly supplied source files, including imports.
    pub fn insert(&mut self, name: impl AsRef<Path>, bytes: Vec<u8>) -> io::Result<PathBuf> {
        if name.as_ref().is_absolute() { return Err(invalid("source bundle names must be relative")); }
        let path = canonical(name.as_ref())?;
        let old = self.files.get(&path).map_or(0, Vec::len);
        let total = self.bytes.checked_sub(old).and_then(|n| n.checked_add(bytes.len()))
            .filter(|n| *n <= self.limit).ok_or_else(|| invalid("source bundle byte limit exceeded"))?;
        self.files.insert(path.clone(), bytes);
        self.bytes = total;
        Ok(path)
    }
    pub fn entry_path(&self, name: impl AsRef<Path>) -> io::Result<PathBuf> {
        self.canonicalize(name.as_ref())
    }
    pub fn byte_count(&self) -> usize { self.bytes }
}
fn invalid(message: &'static str) -> io::Error { io::Error::new(io::ErrorKind::InvalidInput, message) }
fn canonical(path: &Path) -> io::Result<PathBuf> {
    let root = Path::new(ROOT);
    let path = if path.is_absolute() {
        path.strip_prefix(root).map_err(|_| invalid("source path is outside the bundle"))?
    } else { path };
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) if !value.as_encoded_bytes().contains(&0) => relative.push(value),
            Component::CurDir => {},
            Component::ParentDir if relative.pop() => {},
            _ => return Err(invalid("source path escapes the bundle")),
        }
    }
    if relative.as_os_str().is_empty() { return Err(invalid("source path is empty")); }
    Ok(root.join(relative))
}
impl SourceProvider for SourceBundle {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let path = canonical(path)?;
        if self.files.contains_key(&path) { Ok(path) }
        else { Err(io::Error::new(io::ErrorKind::NotFound, "source file was not supplied in the bundle")) }
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files.get(&canonical(path)?).cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "source file was not supplied in the bundle"))
    }
    fn is_file(&self, path: &Path) -> bool {
        canonical(path).is_ok_and(|path| self.files.contains_key(&path))
    }
}
