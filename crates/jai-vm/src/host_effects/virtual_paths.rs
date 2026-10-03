//! Portable POSIX virtual capabilities, independent of target Path component rules.
use super::*;
fn text(path: &Path) -> Result<&str, HostError> {
    let name = path.to_str().ok_or(HostError::InvalidPath)?;
    if name.contains(['\\', ':', '\0']) {
        return Err(HostError::InvalidPath);
    }
    Ok(name)
}
fn relative_parts(name: &str) -> Result<Vec<&str>, HostError> {
    if name.starts_with('/') {
        return Err(HostError::InvalidPath);
    }
    let mut parts = Vec::new();
    for part in name.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.pop().is_some() => {}
            ".." => return Err(HostError::Denied("filename escapes granted virtual root")),
            part => parts.push(part),
        }
    }
    Ok(parts)
}
impl HostPath {
    pub fn new_virtual(root: FileRootId, relative: impl AsRef<Path>) -> Result<Self, HostError> {
        let name = text(relative.as_ref())?;
        if name.is_empty()
            || name.starts_with('/')
            || name
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(HostError::InvalidPath);
        }
        Ok(Self {
            root,
            relative: PathBuf::from(name),
        })
    }
}
impl FilePathScope {
    pub fn new_virtual(
        root: FileRootId,
        canonical_root: impl AsRef<Path>,
        working_relative: impl AsRef<Path>,
    ) -> Result<Self, HostError> {
        let name = text(canonical_root.as_ref())?;
        if !name.starts_with('/') {
            return Err(HostError::InvalidPath);
        }
        let parts: Vec<_> = name.split('/').filter(|part| !part.is_empty()).collect();
        if parts.iter().any(|part| matches!(*part, "." | "..")) {
            return Err(HostError::InvalidPath);
        }
        let canonical_root = PathBuf::from(format!("/{}", parts.join("/")));
        let working_relative =
            PathBuf::from(relative_parts(text(working_relative.as_ref())?)?.join("/"));
        Ok(Self {
            root,
            canonical_root,
            working_relative,
            virtual_namespace: true,
        })
    }
    pub(super) fn resolve_virtual(&self, source: &Path) -> Result<HostPath, HostError> {
        let source = text(source)?;
        if source.is_empty() {
            return Err(HostError::InvalidPath);
        }
        let relative = if let Some(absolute) = source.strip_prefix('/') {
            let root = text(&self.canonical_root)?.trim_matches('/');
            if root.is_empty() {
                absolute
            } else {
                absolute
                    .strip_prefix(root)
                    .and_then(|name| name.strip_prefix('/'))
                    .ok_or(HostError::Denied(
                        "filename is outside its granted virtual root",
                    ))?
            }
        } else {
            let working = text(&self.working_relative)?;
            let joined = format!("{working}/{source}");
            return HostPath::new_virtual(
                self.root,
                relative_parts(joined.trim_start_matches('/'))?.join("/"),
            );
        };
        HostPath::new_virtual(self.root, relative_parts(relative)?.join("/"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn virtual_scope_is_portable_and_retains_exact_grant() {
        let root = FileRootId::allocate();
        let scope = FilePathScope::new_virtual(root, "/jai-script", "lib").unwrap();
        for name in ["../data.bin", "/jai-script/lib/../data.bin"] {
            let path = scope.resolve(Path::new(name)).unwrap();
            assert_eq!(path.root(), root);
            assert_eq!(path.relative(), Path::new("data.bin"));
        }
        for name in [
            "../../data.bin",
            "/jai-script-other/data.bin",
            "C:/data.bin",
            "a\\b",
            "",
        ] {
            assert!(scope.resolve(Path::new(name)).is_err(), "{name}");
        }
    }
}
