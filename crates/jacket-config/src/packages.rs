//! The folders dependencies add to the module search: their own `[package] import-path`, and the
//! parent of a module that is not at their root.
use crate::{Config, ConfigError, FILE_NAME, Files, Package};
use std::path::{Path, PathBuf};

/// What the dependencies of a project add to the compiler's module search.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Roots {
    /// Each dependency's folder and the folders only its own files search.
    pub packages: Vec<(PathBuf, Vec<PathBuf>)>,
    /// The folder holding each module that sits in a subfolder of its repository, so that
    /// `#import "<folder name>"` finds it from anywhere in the program.
    pub modules: Vec<PathBuf>,
}

/// Where each dependency of `config` is, relative to `base` (the folder holding the
/// `jacket.toml`): its `path`, else `modules/<name>`.
pub fn dependency_dirs(base: &Path, config: &Config) -> Vec<(String, PathBuf)> {
    config
        .dependencies
        .iter()
        .map(|(name, entry)| {
            let path = entry
                .path
                .clone()
                .unwrap_or_else(|| format!("modules/{name}"));
            (name.clone(), base.join(path))
        })
        .collect()
}

/// `[package]` of a dependency's `jacket.toml`. The file must parse as a whole, and `[package]`
/// may not carry a key the schema does not know (an old spelling would otherwise drop the
/// dependency's import folders unnoticed); unknown keys elsewhere belong to the dependency's own
/// tools and are ignored.
fn package_table(text: &str) -> Result<Package, String> {
    let (config, unknown) = crate::parse_lenient(text)?;
    match unknown.iter().find(|key| key.starts_with("package.")) {
        Some(key) => Err(crate::unknown_note(key)),
        None => Ok(config.package),
    }
}

/// The search roots the dependencies of `config` add. For each one with a `jacket.toml`:
/// `[package] import-path` joined onto the dependency's folder, and the parent of its module
/// folder (the entry's `module`, else `[package] module`) when the module is not at the root.
/// Everything else in that file belongs to whoever builds the program and is never read.
///
/// A dependency without a `jacket.toml` adds nothing; one that cannot be read or parsed is an
/// error naming the file.
pub fn dependency_roots(
    files: &dyn Files,
    base: &Path,
    config: &Config,
) -> Result<Roots, ConfigError> {
    let mut roots = Roots::default();
    for (name, dir) in dependency_dirs(base, config) {
        let file = dir.join(FILE_NAME);
        let package = match files.read(&file) {
            Ok(text) => package_table(&text).map_err(|message| ConfigError {
                path: file.clone(),
                message,
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Package::default(),
            Err(error) => {
                return Err(ConfigError {
                    path: file,
                    message: format!("the file could not be read: {error}"),
                });
            }
        };
        let paths: Vec<PathBuf> = package.import_path.0.iter().map(|p| dir.join(p)).collect();
        if !paths.is_empty() {
            roots.packages.push((dir.clone(), paths));
        }
        let module = config.dependencies[&name]
            .module
            .as_deref()
            .or(package.module.as_deref())
            .map(|m| m.trim_matches('/'))
            .filter(|m| !m.is_empty());
        if let Some(parent) = module.and_then(|m| dir.join(m).parent().map(Path::to_path_buf)) {
            roots.modules.push(parent);
        }
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct One(&'static str, &'static str);

    impl Files for One {
        fn is_file(&self, path: &Path) -> bool {
            path.ends_with(self.0)
        }

        fn read(&self, path: &Path) -> std::io::Result<String> {
            if path.ends_with(self.0) {
                Ok(self.1.to_string())
            } else {
                Err(std::io::ErrorKind::NotFound.into())
            }
        }
    }

    fn config() -> Config {
        toml::from_str("[dependencies]\ndep = { git = \"x/y\", tag = \"v1\" }\nmoved = { git = \"x/z\", tag = \"v1\", path = \"vendor/z\" }\n").unwrap()
    }

    #[test]
    fn a_dependency_adds_its_package_import_path() {
        let files = One(
            "modules/dep/jacket.toml",
            "[project]\nimport-path = [\"ignored\"]\n[package]\nimport-path = [\"lib\", \"more\"]\n",
        );
        let found = dependency_roots(&files, Path::new("/p"), &config())
            .unwrap()
            .packages;
        assert_eq!(
            found,
            [(
                PathBuf::from("/p/modules/dep"),
                vec![
                    PathBuf::from("/p/modules/dep/lib"),
                    PathBuf::from("/p/modules/dep/more")
                ]
            )]
        );
    }

    #[test]
    fn a_moved_folder_is_followed() {
        let files = One("vendor/z/jacket.toml", "[package]\nimport-path = \"v\"\n");
        let found = dependency_roots(&files, Path::new("/p"), &config())
            .unwrap()
            .packages;
        assert_eq!(
            found,
            [(
                PathBuf::from("/p/vendor/z"),
                vec![PathBuf::from("/p/vendor/z/v")]
            )]
        );
    }

    #[test]
    fn a_broken_manifest_is_an_error_naming_the_file() {
        let files = One("modules/dep/jacket.toml", "[package\n");
        let error = dependency_roots(&files, Path::new("/p"), &config()).unwrap_err();
        assert_eq!(error.path, PathBuf::from("/p/modules/dep/jacket.toml"));
    }

    #[test]
    fn an_unreadable_manifest_is_an_error_naming_the_file() {
        struct Denied;
        impl Files for Denied {
            fn is_file(&self, _: &Path) -> bool {
                true
            }
            fn read(&self, _: &Path) -> std::io::Result<String> {
                Err(std::io::ErrorKind::PermissionDenied.into())
            }
        }
        let error = dependency_roots(&Denied, Path::new("/p"), &config()).unwrap_err();
        assert!(error.to_string().contains("modules/dep/jacket.toml"));
        assert!(error.message.contains("could not be read"));
    }

    #[test]
    fn an_old_package_spelling_is_an_error_with_the_new_one() {
        let files = One(
            "modules/dep/jacket.toml",
            "[package]\nimport_path = [\"lib\"]\n",
        );
        let error = dependency_roots(&files, Path::new("/p"), &config()).unwrap_err();
        assert!(error.message.contains("did you mean `package.import-path`"));
    }

    #[test]
    fn a_dependency_without_folders_or_a_manifest_adds_nothing() {
        let files = One("modules/dep/jacket.toml", "[package]\nname = \"dep\"\n");
        let roots = dependency_roots(&files, Path::new("/p"), &config()).unwrap();
        assert_eq!(roots, Roots::default());
    }

    #[test]
    fn a_module_below_the_root_is_found_through_its_parent() {
        let files = One(
            "modules/dep/jacket.toml",
            "[package]\nmodule = \"modules/Inner\"\n",
        );
        let roots = dependency_roots(&files, Path::new("/p"), &config()).unwrap();
        assert_eq!(roots.modules, [PathBuf::from("/p/modules/dep/modules")]);
        // The entry's own `module` replaces the package's.
        let config: Config = toml::from_str(
            "[dependencies]\ndep = { git = \"x/y\", tag = \"v1\", module = \"Deep\" }\n",
        )
        .unwrap();
        let roots = dependency_roots(&files, Path::new("/p"), &config).unwrap();
        assert_eq!(roots.modules, [PathBuf::from("/p/modules/dep")]);
    }
}
