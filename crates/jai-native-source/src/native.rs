use super::*;
use std::{
    path::Component,
    sync::{Arc, Mutex},
};

pub(super) fn normalize(path: &Path) -> io::Result<PathBuf> {
    if let Ok(existing) = path.canonicalize() {
        return Ok(existing);
    }
    if let (Some(parent), Some(name)) = (path.parent(), path.file_name())
        && let Ok(parent) = parent.canonicalize()
    {
        return Ok(parent.join(name));
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

/// The first observation of a native source is retained for this source round.
/// Create a new snapshot to observe editor/filesystem changes on a later round.
#[derive(Debug, Default)]
pub struct NativeSourceSnapshot {
    observations: Mutex<Observations>,
}
#[derive(Debug, Default)]
struct Observations {
    canonical: HashMap<PathBuf, Result<PathBuf, ObservedError>>,
    bytes: HashMap<PathBuf, Result<Arc<[u8]>, ObservedError>>,
    text: HashMap<PathBuf, jai_source::SourceTextSnapshot>,
    files: HashMap<PathBuf, bool>,
}
#[derive(Clone, Debug)]
struct ObservedError {
    kind: io::ErrorKind,
    message: String,
}
impl From<io::Error> for ObservedError {
    fn from(error: io::Error) -> Self {
        Self {
            kind: error.kind(),
            message: error.to_string(),
        }
    }
}
impl ObservedError {
    fn error(&self) -> io::Error {
        io::Error::new(self.kind, self.message.clone())
    }
}
impl SourceProvider for NativeSourceSnapshot {
    fn retain_decoded_text(
        &self,
        path: &Path,
        text: &str,
    ) -> io::Result<jai_source::SourceTextSnapshot> {
        let path = self.canonicalize(path)?;
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| io::Error::other("native source snapshot is poisoned"))?;
        let bytes = observations
            .bytes
            .get(&path)
            .and_then(|observed| observed.as_ref().ok())
            .ok_or_else(|| {
                io::Error::other("retained source text has no successful actual read observation")
            })?;
        let decoded = jai_lexer::decode_source(bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        if decoded.as_ref() != text {
            return Err(io::Error::other(
                "retained text differs from actual decoded read observation",
            ));
        }
        let snapshot = observations
            .text
            .entry(path)
            .or_insert_with(|| jai_source::SourceTextSnapshot::new(text.to_owned()));
        if snapshot.text() != text {
            return Err(io::Error::other(
                "decoded source changed within immutable native snapshot",
            ));
        }
        Ok(snapshot.clone())
    }

    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        normalize(path)
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| io::Error::other("native source snapshot is poisoned"))?;
        let result = observations
            .canonical
            .entry(path.to_owned())
            .or_insert_with(|| path.canonicalize().map_err(ObservedError::from));
        result.clone().map_err(|error| error.error())
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let path = self.canonicalize(path)?;
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| io::Error::other("native source snapshot is poisoned"))?;
        let result = observations.bytes.entry(path).or_insert_with_key(|path| {
            std::fs::read(path)
                .map(Arc::from)
                .map_err(ObservedError::from)
        });
        result
            .as_ref()
            .map(|bytes| bytes.to_vec())
            .map_err(|error| error.error())
    }
    fn is_file(&self, path: &Path) -> bool {
        let Ok(mut observations) = self.observations.lock() else {
            return false;
        };
        *observations
            .files
            .entry(path.to_owned())
            .or_insert_with(|| path.is_file())
    }
}
