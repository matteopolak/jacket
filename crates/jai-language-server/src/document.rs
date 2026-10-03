use crate::{Error, Range};
use jai_source::{SourceProvider, SourceTextSnapshot};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentUri {
    uri: String,
    path: String,
}
impl DocumentUri {
    pub fn parse(uri: &str) -> Result<Self, Error> {
        if uri.len() > 4096 {
            return Err(Error::Limit("document URI exceeds 4096 bytes"));
        }
        let encoded = uri
            .strip_prefix("file://")
            .ok_or(Error::Uri("only absolute local file URIs are supported"))?;
        if !encoded.starts_with('/') || encoded.contains(['?', '#', '\0']) {
            return Err(Error::Uri(
                "file URI must have no authority, query or fragment",
            ));
        }
        let mut bytes = Vec::with_capacity(encoded.len());
        let input = encoded.as_bytes();
        let mut at = 0;
        while at < input.len() {
            if input[at] == b'%' {
                let value = input
                    .get(at + 1..at + 3)
                    .ok_or(Error::Uri("incomplete URI escape"))?;
                let high = hex(value[0]).ok_or(Error::Uri("invalid URI escape"))?;
                let low = hex(value[1]).ok_or(Error::Uri("invalid URI escape"))?;
                bytes.push(high * 16 + low);
                at += 3;
            } else {
                bytes.push(input[at]);
                at += 1;
            }
        }
        let decoded = String::from_utf8(bytes).map_err(|_| Error::Uri("URI path is not UTF-8"))?;
        let path = normalize(&decoded)?;
        let uri = format!("file://{}", encode(&path));
        if uri.len() > 4096 {
            return Err(Error::Limit("canonical document URI exceeds 4096 bytes"));
        }
        Ok(Self {
            uri,
            path,
        })
    }
    pub fn as_str(&self) -> &str {
        &self.uri
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub(crate) fn load(&self, target: &str) -> Result<Self, Error> {
        if target.contains(['\\', '\0', ':']) || target.len() > 4096 {
            return Err(Error::Uri("unsupported #load target"));
        }
        let joined = if target.starts_with('/') {
            target.to_owned()
        } else {
            format!(
                "{}/{}",
                self.path.rsplit_once('/').map_or("", |(parent, _)| parent),
                target
            )
        };
        let path = normalize(&joined)?;
        Ok(Self {
            uri: format!("file://{}", encode(&path)),
            path,
        })
    }
}
fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}
fn encode(path: &str) -> String {
    let mut out = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(&byte) {
            out.push(byte as char);
        } else {
            use std::fmt::Write;
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
}
fn normalize(path: &str) -> Result<String, Error> {
    if !path.starts_with('/') || path.contains(['\\', '\0']) {
        return Err(Error::Uri(
            "document path must be absolute and contain no backslash or NUL",
        ));
    }
    let mut parts = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    return Err(Error::Uri("document path escapes its root"));
                }
            }
            value => parts.push(value),
        }
    }
    if parts.is_empty() {
        return Err(Error::Uri("document URI names a root"));
    }
    Ok(format!("/{}", parts.join("/")))
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextChange {
    pub range: Option<Range>,
    pub range_length: Option<u32>,
    pub text: String,
}

/// A closed immutable source observation. It never consults an OS filesystem.
#[derive(Clone, Debug, Default)]
pub struct VirtualSources {
    pub(crate) files: BTreeMap<PathBuf, SourceTextSnapshot>,
}
impl SourceProvider for VirtualSources {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let normalized = self.normalize(path)?;
        if self.files.contains_key(&normalized) {
            Ok(normalized)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "document is not open",
            ))
        }
    }
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        let text = path.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "document path is not UTF-8")
        })?;
        normalize(text)
            .map(PathBuf::from)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        Ok(self
            .files
            .get(&self.canonicalize(path)?)
            .expect("checked open document")
            .text()
            .as_bytes()
            .to_vec())
    }
    fn is_file(&self, path: &Path) -> bool {
        self.canonicalize(path).is_ok()
    }
    fn retain_decoded_text(&self, path: &Path, text: &str) -> io::Result<SourceTextSnapshot> {
        let snapshot = &self.files[&self.canonicalize(path)?];
        if snapshot.text() != text {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "decoded document does not match the pinned source observation",
            ));
        }
        Ok(snapshot.clone())
    }
}
