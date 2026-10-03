//! Resolve native tools while excluding supplied reference and corpus executables.
use crate::Error;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

enum Tool {
    Compiler,
    Archiver,
}
impl Tool {
    fn configured_path(self) -> PathBuf {
        match self {
            Self::Compiler => {
                env::var_os("JAI_RS_CLANG").map_or_else(|| PathBuf::from("clang"), PathBuf::from)
            }
            Self::Archiver => env::var_os("JAI_RS_AR").map_or_else(
                || {
                    env::var_os("LLVM_SYS_221_PREFIX")
                        .or_else(|| option_env!("LLVM_SYS_221_PREFIX").map(Into::into))
                        .map_or_else(
                            || PathBuf::from("llvm-ar"),
                            |prefix| PathBuf::from(prefix).join("bin/llvm-ar"),
                        )
                },
                PathBuf::from,
            ),
        }
    }
}
pub fn compiler() -> Result<PathBuf, Error> {
    resolve(Tool::Compiler)
}
pub fn archiver() -> Result<PathBuf, Error> {
    resolve(Tool::Archiver)
}
pub(crate) fn protected_input(path: &Path) -> Result<bool, Error> {
    let logical = crate::native_paths::lexical(path)?;
    if protected_roots().any(|root| logical.starts_with(root)) {
        return Ok(true);
    }
    let resolved = crate::native_paths::resolve(path)?;
    Ok(protected_roots().any(|root| resolved.starts_with(root)))
}
fn resolve(tool: Tool) -> Result<PathBuf, Error> {
    resolve_named(tool.configured_path())
}
fn resolve_named(name: PathBuf) -> Result<PathBuf, Error> {
    let candidate = if name.components().count() > 1 || name.is_absolute() {
        Some(name.clone())
    } else {
        env::var_os("PATH").and_then(|paths| {
            env::split_paths(&paths)
                .map(|root| root.join(&name))
                .find(|path| path.is_file())
        })
    };
    let candidate = candidate.ok_or_else(|| Error::ToolNotFound(name.clone()))?;
    let logical = crate::native_paths::lexical(&candidate)?;
    if protected_roots().any(|root| logical.starts_with(root)) {
        return Err(Error::ReferenceTool(logical));
    }
    let resolved = crate::native_paths::resolve(&candidate)?;
    if protected_roots().any(|root| resolved.starts_with(root)) {
        return Err(Error::ReferenceTool(resolved));
    }
    let canonical = candidate.canonicalize().map_err(|cause| {
        if cause.kind() == std::io::ErrorKind::NotFound {
            Error::ToolNotFound(name)
        } else {
            Error::Io {
                path: candidate,
                cause,
            }
        }
    })?;
    if protected_roots().any(|root| canonical.starts_with(root)) {
        return Err(Error::ReferenceTool(canonical));
    }
    Ok(canonical)
}
pub fn validate_output(path: &Path) -> Result<(), Error> {
    let logical = crate::native_paths::lexical(path)?;
    let existing = crate::native_paths::resolve(path)?;
    if protected_roots().any(|root| logical.starts_with(&root) || existing.starts_with(&root)) {
        return Err(Error::Source(
            "artifact output is inside protected reference inputs; no artifact was emitted".into(),
        ));
    }
    for tool in [Tool::Compiler, Tool::Archiver] {
        if let Ok(tool) = resolve(tool) {
            if logical == tool || existing == tool {
                return Err(Error::Source("artifact output would overwrite an installed native tool; no artifact was emitted".into()));
            }
        }
    }
    Ok(())
}

fn protected_roots() -> impl Iterator<Item = PathBuf> {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace crate path");
    let project = project.canonicalize().unwrap_or_else(|_| project.into());
    ["reference", "vendor", "corpus/upstream"]
        .into_iter()
        .map(move |name| {
            let path = project.join(name);
            path.canonicalize().unwrap_or(path)
        })
}
pub fn scrub_environment(command: &mut Command) {
    for variable in [
        "LIBRARY_PATH",
        "LD_LIBRARY_PATH",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_ROOT_PATH",
        "DYLD_VERSIONED_LIBRARY_PATH",
        "DYLD_VERSIONED_FRAMEWORK_PATH",
        "LD_PRELOAD",
        "LD_AUDIT",
        "CCC_OVERRIDE_OPTIONS",
        "SDKROOT",
    ] {
        command.env_remove(variable);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn output_parent_aliases_cannot_enter_any_supplied_source_root() {
        use std::os::unix::fs::symlink;
        let fixture = env::temp_dir().join(format!("jai-own-native-output-{}", std::process::id()));
        std::fs::create_dir(&fixture).unwrap();
        for (index, root) in protected_roots().enumerate() {
            assert!(validate_output(&root.join("nonexistent-authored-output")).is_err());
            let alias = fixture.join(format!("alias-{index}"));
            symlink(&root, &alias).unwrap();
            assert!(validate_output(&alias.join("not-created/subdirectory/output")).is_err());
            let dangling = fixture.join(format!("dangling-{index}"));
            symlink(root.join("not-created/protected-subdirectory"), &dangling).unwrap();
            assert!(validate_output(&dangling.join("output")).is_err());
            assert!(protected_input(&dangling.join("library.a")).unwrap());
            assert!(matches!(
                resolve_named(dangling.join("unreviewed-tool")),
                Err(Error::ReferenceTool(_))
            ));
        }
        std::fs::remove_dir_all(fixture).unwrap();
    }
    #[test]
    fn absent_tools_in_protected_roots_are_rejected_before_lookup() {
        for root in protected_roots() {
            let path = root.join("not-created/unreviewed-tool");
            assert!(matches!(resolve_named(path), Err(Error::ReferenceTool(_))));
        }
    }
}
