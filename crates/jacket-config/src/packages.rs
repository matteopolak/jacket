//! The folders dependencies add to the module search: their own `[package] import_path`, and the
//! parent of a module that is not at their root.
use crate::{Config, FILE_NAME, Files, Package, Project};
use std::path::{Path, PathBuf};

/// The two tables of a dependency's `jacket.toml` that name import folders.
#[derive(Default, serde::Deserialize)]
struct Tables {
    #[serde(default)]
    package: Package,
    #[serde(default)]
    project: Project,
}

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

/// The search roots the dependencies of `config` add. For each one with a `jacket.toml`:
/// `[package] import_path`, else `[project] import_path`, joined onto the dependency's folder;
/// and the parent of its module folder (the entry's `module`, else `[package] module`) when the
/// module is not at the root. Everything else in that file belongs to whoever builds the
/// program and is never read.
pub fn dependency_roots(files: &dyn Files, base: &Path, config: &Config) -> Roots {
    let mut roots = Roots::default();
    for (name, dir) in dependency_dirs(base, config) {
        let text = files.read(&dir.join(FILE_NAME)).unwrap_or_default();
        let tables: Tables = toml::from_str(&text).unwrap_or_default();
        let own = if tables.package.import_path.0.is_empty() {
            &tables.project.import_path
        } else {
            &tables.package.import_path
        };
        let paths: Vec<PathBuf> = own.0.iter().map(|p| dir.join(p)).collect();
        if !paths.is_empty() {
            roots.packages.push((dir.clone(), paths));
        }
        let module = config.dependencies[&name]
            .module
            .as_deref()
            .or(tables.package.module.as_deref())
            .map(|m| m.trim_matches('/'))
            .filter(|m| !m.is_empty());
        if let Some(parent) = module.and_then(|m| dir.join(m).parent().map(Path::to_path_buf)) {
            roots.modules.push(parent);
        }
    }
    roots
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
    fn the_package_table_wins_over_project() {
        let files = One(
            "modules/dep/jacket.toml",
            "[project]\nimport_path = [\"old\"]\n[package]\nimport_path = [\"lib\", \"more\"]\n",
        );
        let found = dependency_roots(&files, Path::new("/p"), &config()).packages;
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
    fn project_import_path_stands_in_and_a_moved_folder_is_followed() {
        let files = One("vendor/z/jacket.toml", "[project]\nimport_path = \"v\"\n");
        let found = dependency_roots(&files, Path::new("/p"), &config()).packages;
        assert_eq!(
            found,
            [(
                PathBuf::from("/p/vendor/z"),
                vec![PathBuf::from("/p/vendor/z/v")]
            )]
        );
    }

    #[test]
    fn a_dependency_without_folders_or_a_manifest_adds_nothing() {
        let files = One("modules/dep/jacket.toml", "[package]\nname = \"dep\"\n");
        let roots = dependency_roots(&files, Path::new("/p"), &config());
        assert_eq!(roots, Roots::default());
    }

    #[test]
    fn a_module_below_the_root_is_found_through_its_parent() {
        let files = One(
            "modules/dep/jacket.toml",
            "[package]\nmodule = \"modules/Inner\"\n",
        );
        let roots = dependency_roots(&files, Path::new("/p"), &config());
        assert_eq!(roots.modules, [PathBuf::from("/p/modules/dep/modules")]);
        // The entry's own `module` replaces the package's.
        let config: Config = toml::from_str(
            "[dependencies]\ndep = { git = \"x/y\", tag = \"v1\", module = \"Deep\" }\n",
        )
        .unwrap();
        let roots = dependency_roots(&files, Path::new("/p"), &config);
        assert_eq!(roots.modules, [PathBuf::from("/p/modules/dep")]);
    }
}
