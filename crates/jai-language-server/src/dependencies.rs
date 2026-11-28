//! The folders a dependency's own files search, worked out from the project's `jacket.toml` the
//! way `jacket build` passes them (`-package_import_dir`), by the code both share
//! (`jacket_config::packages::dependency_roots`). That includes the copy of a shared dependency
//! each package is routed to when the project holds several versions of it.
use jacket_config::{ConfigError, Files};
use jaic::sema::{FileSystem, PackageImports};
use std::path::Path;

/// A compiler file system read as `jacket_config` files, so the compile that reads a project's
/// manifests records them as inputs.
struct Through<'a>(&'a dyn FileSystem);

impl Files for Through<'_> {
    fn is_file(&self, path: &Path) -> bool {
        self.0.is_file(path)
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        self.0
            .read(path)
            .ok_or_else(|| std::io::ErrorKind::NotFound.into())
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })
    }
}

/// What each dependency of the project above `root` adds for its own files, spelled as the
/// compiler reaches them through `fs`. A project file or a dependency's `jacket.toml` that does
/// not read is an error naming it.
pub(crate) fn package_imports(
    fs: &dyn FileSystem,
    root: &Path,
) -> Result<Vec<PackageImports>, ConfigError> {
    let files = Through(fs);
    let Some(location) = root
        .parent()
        .and_then(|dir| jacket_config::locate(&files, dir))
    else {
        return Ok(Vec::new());
    };
    let loaded = jacket_config::load_at(&files, location)?;
    let identity = |path: &Path| jaic::paths::file_identity(fs, path);
    let roots = jacket_config::packages::dependency_roots(&files, loaded.dir(), &loaded.config)?;
    Ok(roots
        .packages
        .into_iter()
        .map(|(dir, paths)| PackageImports {
            dir: identity(&dir),
            import_paths: paths.iter().map(|p| identity(p)).collect(),
        })
        .collect())
}
