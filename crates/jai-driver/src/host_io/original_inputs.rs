//! Deny-only policy authenticated against compiled, out-of-source inventory receipts.
use super::{HostIo, original_input_receipts::*};
use jai_vm::host_effects::HostError;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

const MANIFEST_BYTES: usize = 1024 * 1024;

/// Trusted Rust inventory policy. It confers no file, process or linking grant.
#[derive(Clone, Debug)]
pub struct OriginalInputPolicy {
    roots: Vec<PathBuf>,
    native_fingerprints: Vec<[u8; 32]>,
}
impl OriginalInputPolicy {
    /// Verify fixed inventory files against reviewed receipts compiled into this
    /// driver. Changing a JSON file alone cannot change the active deny policy.
    pub fn from_trusted_inventory(workspace: &Path) -> Result<Self, HostError> {
        Self::load(workspace, MANIFEST_RECEIPTS, NATIVE_FINGERPRINTS)
    }
    fn load(
        workspace: &Path,
        manifests: &[(&str, [u8; 32])],
        fingerprints: &[[u8; 32]],
    ) -> Result<Self, HostError> {
        let workspace = workspace.canonicalize().map_err(io)?;
        if !workspace.is_dir() {
            return Err(HostError::InvalidPath);
        }
        for (relative, expected) in manifests {
            let path = workspace.join(relative);
            if !path.canonicalize().map_err(io)?.starts_with(&workspace)
                || fs::symlink_metadata(&path)
                    .map_err(io)?
                    .file_type()
                    .is_symlink()
            {
                return Err(HostError::Denied(
                    "inventory manifest escapes trusted workspace",
                ));
            }
            if manifest_fingerprint(&path)? != *expected {
                return Err(HostError::Denied(
                    "original input inventory receipt changed",
                ));
            }
        }
        let mut roots = Vec::new();
        for relative in ORIGINAL_ROOTS {
            let logical = workspace.join(relative);
            let resolved = prospective(&logical, 64)?;
            for root in [logical, resolved] {
                if !roots.contains(&root) {
                    roots.push(root);
                }
            }
        }
        Ok(Self {
            roots,
            native_fingerprints: fingerprints.to_vec(),
        })
    }
    pub fn protected_roots(&self) -> &[PathBuf] {
        &self.roots
    }
    /// Install the complete policy atomically before source observations.
    pub fn apply(&self, host: &mut HostIo) -> Result<(), HostError> {
        host.apply_original_protection(&self.roots, &self.native_fingerprints)
    }
}
fn io(error: std::io::Error) -> HostError {
    HostError::Io(error.to_string())
}
fn manifest_fingerprint(path: &Path) -> Result<[u8; 32], HostError> {
    let mut file = fs::File::open(path).map_err(io)?;
    let metadata = file.metadata().map_err(io)?;
    if !metadata.is_file() || metadata.len() > MANIFEST_BYTES as u64 {
        return Err(HostError::Budget("trusted inventory manifest bytes"));
    }
    let mut hash = Sha256::new();
    let mut bytes = 0usize;
    let mut buffer = [0u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(io)?;
        if count == 0 {
            break;
        }
        bytes = bytes
            .checked_add(count)
            .ok_or(HostError::Budget("trusted inventory manifest bytes"))?;
        if bytes > MANIFEST_BYTES {
            return Err(HostError::Budget("trusted inventory manifest bytes"));
        }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().into())
}
fn lexical(path: &Path) -> Result<PathBuf, HostError> {
    if !path.is_absolute() {
        return Err(HostError::InvalidPath);
    }
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err(HostError::InvalidPath);
                }
            }
            part => result.push(part),
        }
    }
    Ok(result)
}
// Resolve the existing prefix, including dangling symlink roots, while keeping
// not-yet-created original subtrees protected. This never creates any directory.
fn prospective(path: &Path, remaining: usize) -> Result<PathBuf, HostError> {
    if remaining == 0 {
        return Err(HostError::Budget("original root path resolution"));
    }
    let path = lexical(path)?;
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
                let target = fs::read_link(&path).map_err(io)?;
                let target = if target.is_absolute() {
                    target
                } else {
                    path.parent().ok_or(HostError::InvalidPath)?.join(target)
                };
                return prospective(&target, remaining - 1);
            }
            let parent = path.parent().ok_or(HostError::InvalidPath)?;
            let name = path.file_name().ok_or(HostError::InvalidPath)?;
            Ok(prospective(parent, remaining - 1)?.join(name))
        }
        Err(error) => Err(io(error)),
    }
}

#[cfg(test)]
#[path = "original_inputs/tests.rs"]
mod tests;
