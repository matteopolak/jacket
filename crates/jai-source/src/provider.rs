//! Compiler source inputs are observations supplied by the selected platform.
use std::{
    io,
    path::{Path, PathBuf},
};

pub trait SourceProvider {
    /// Return a genuine retained decoded-text owner. The default creates a fresh observation.
    fn retain_decoded_text(
        &self,
        _path: &Path,
        text: &str,
    ) -> io::Result<crate::SourceTextSnapshot> {
        Ok(crate::SourceTextSnapshot::new(text.to_owned()))
    }
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn is_file(&self, path: &Path) -> bool;
    /// Normalize a prospective generated filename without requiring it to exist.
    fn normalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.canonicalize(path)
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

/// Virtual names use a POSIX namespace on every target, including wasm32.
/// Host filesystem normalization belongs exclusively to a native adapter.
pub fn normalize_virtual_path(root: &str, path: &Path) -> io::Result<PathBuf> {
    if !root.starts_with('/') || root.contains(['\\', ':', '\0']) {
        return Err(invalid("virtual root must be an absolute POSIX name"));
    }
    let root_parts: Vec<_> = root.split('/').filter(|part| !part.is_empty()).collect();
    if root_parts.iter().any(|part| matches!(*part, "." | "..")) {
        return Err(invalid("virtual root must be normalized"));
    }
    let name = path
        .to_str()
        .ok_or_else(|| invalid("virtual filename must be UTF-8"))?;
    if name.contains(['\\', ':', '\0']) || name.is_empty() {
        return Err(invalid("invalid virtual filename"));
    }
    let mut parts = if name.starts_with('/') {
        Vec::new()
    } else {
        root_parts.clone()
    };
    for part in name.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.len() > root_parts.len() => {
                parts.pop();
            }
            ".." => return Err(invalid("virtual filename escapes its root")),
            part => parts.push(part),
        }
    }
    if !parts.starts_with(&root_parts) || parts.len() <= root_parts.len() {
        return Err(invalid(
            "virtual filename is outside its root or names the root",
        ));
    }
    Ok(PathBuf::from(format!("/{}", parts.join("/"))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtual_names_normalize_identically_without_host_path_queries() {
        for name in ["main.jai", "./sub/../main.jai", "/jai-script/main.jai"] {
            assert_eq!(
                normalize_virtual_path("/jai-script", Path::new(name)).unwrap(),
                PathBuf::from("/jai-script/main.jai")
            );
        }
        for name in [
            "",
            "../main.jai",
            "/elsewhere/main.jai",
            "C:/main.jai",
            "a\\b.jai",
            "a\0b",
            "/jai-script",
        ] {
            assert!(
                normalize_virtual_path("/jai-script", Path::new(name)).is_err(),
                "{name:?}"
            );
        }
    }
}
