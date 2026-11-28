//! The folders dependencies add to the module search: their own `[package] import-path`, the
//! parent of a module that is not at their root, and the copy of a dependency that each package
//! needs when the program holds several (see [`Roots::packages`]).
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::version::{Scheme, Version, parse_range, parse_tag};
use crate::{Config, ConfigError, Dependency, FILE_NAME, Files, Package};

/// What the dependencies of a project add to the compiler's module search.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Roots {
    /// Each dependency's folder and the folders only its own files search: its `import-path`,
    /// then the folder of each copy of a package it needs that is not the shared one.
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

/// The tables of a dependency's `jacket.toml` that routing reads.
#[derive(Default)]
struct Tables {
    package: Package,
    dependencies: BTreeMap<String, Dependency>,
}

/// `[package]` and `[dependencies]` of a dependency's `jacket.toml`. The file must parse as a
/// whole, and `[package]` may not carry a key the schema does not know (an old spelling would
/// otherwise drop the dependency's import folders unnoticed); unknown keys elsewhere belong to
/// the dependency's own tools and are ignored.
fn tables(text: &str) -> Result<Tables, String> {
    let (config, unknown) = crate::parse_lenient(text)?;

    match unknown.iter().find(|key| key.starts_with("package.")) {
        Some(key) => Err(crate::unknown_note(key)),
        None => Ok(Tables {
            package: config.package,
            dependencies: config.dependencies,
        }),
    }
}

/// The name a `[dependencies]` key imports: `foo` for `foo` and for the `foo@1` entry of another
/// copy of it (written by `jacket dep` when the program needs two incompatible versions).
pub fn import_name(key: &str) -> &str {
    key.split_once('@').map_or(key, |(name, _)| name)
}

/// The folder that holds a dependency's module, so that its parent is searched: the entry's
/// `module`, else `[package] module`, else the dependency's own folder.
fn module_dir(entry: &Dependency, tables: &Tables, dir: &Path) -> PathBuf {
    entry
        .module
        .as_deref()
        .or(tables.package.module.as_deref())
        .map(|m| m.trim_matches('/'))
        .filter(|m| !m.is_empty())
        .map_or_else(|| dir.to_path_buf(), |m| dir.join(m))
}

/// Why a request could not be compared with a copy, and whose `jacket.toml` holds the value.
struct Refused {
    /// The requesting package's file, else the project's (the copy's entry).
    in_package: bool,
    message: String,
}

fn scheme_of(entry: &Dependency) -> Result<Scheme, String> {
    entry
        .scheme
        .as_deref()
        .map_or(Ok(Scheme::default()), Scheme::parse)
}

/// Whether the copy `copy`, the entry `key` of the project, is one that `wanted`, a package's
/// request, accepts. A scheme, range or tag that does not read is an error naming the value.
fn admits(key: &str, copy: &Dependency, wanted: &Dependency) -> Result<bool, Refused> {
    let package = |message: String| Refused {
        in_package: true,
        message,
    };
    let project = |message: String| Refused {
        in_package: false,
        message,
    };

    if let Some(range) = &wanted.version {
        let scheme = scheme_of(wanted).map_err(|e| package(format!("scheme: {e}")))?;
        let range = parse_range(scheme, range)
            .map_err(|e| package(format!("version `{range}` is not a range: {e}")))?;
        let Some(tag) = &copy.tag else {
            return Ok(false);
        };
        let scheme = scheme_of(copy).map_err(|e| project(format!("`{key}`: scheme: {e}")))?;
        let version: Version =
            parse_tag(scheme, copy.tag_prefix.as_deref(), tag).ok_or_else(|| {
                project(format!(
                    "`{key}`: tag `{tag}` is not a {} version",
                    scheme.name()
                ))
            })?;

        return Ok(range.matches(&version));
    }

    if let Some(tag) = &wanted.tag {
        return Ok(copy.tag.as_ref() == Some(tag));
    }

    Ok(match (&wanted.rev, &copy.rev) {
        (Some(wanted), Some(have)) => have.starts_with(wanted) || wanted.starts_with(have),
        (Some(_), None) => false,
        (None, _) => true,
    })
}

/// A dependency's tables; none for a dependency without a `jacket.toml`, an error naming the
/// file when it cannot be read or parsed.
fn read_tables(files: &dyn Files, dir: &Path) -> Result<Tables, ConfigError> {
    let file = dir.join(FILE_NAME);

    match files.read(&file) {
        Ok(text) => tables(&text).map_err(|message| ConfigError {
            path: file,
            message,
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Tables::default()),
        Err(error) => Err(ConfigError {
            path: file,
            message: format!("the file could not be read: {error}"),
        }),
    }
}

/// The search roots the dependencies of `config` add. For each one with a `jacket.toml`:
/// `[package] import-path` joined onto the dependency's folder, and the parent of its module
/// folder (the entry's `module`, else `[package] module`) when the module is not at the root.
/// Everything else in that file belongs to whoever builds the program and is never read.
///
/// When the program holds more than one copy of a name (`foo` and `foo@1`: one per group of
/// compatible versions), each package that needs `foo` is sent to the copy whose recorded tag
/// satisfies the range in the package's own `[dependencies]`, by adding that copy's parent
/// folder to the package's roots. A package whose range the shared `foo` satisfies is not
/// routed. This reads only `jacket.toml` files, so the build and the language server agree
/// without a lockfile.
///
/// A dependency without a `jacket.toml` adds nothing; one that cannot be read or parsed is an
/// error naming the file.
pub fn dependency_roots(
    files: &dyn Files,
    base: &Path,
    config: &Config,
) -> Result<Roots, ConfigError> {
    let mut roots = Roots::default();
    let mut copies: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for key in config.dependencies.keys() {
        copies.entry(import_name(key)).or_default().push(key);
    }

    let dirs = dependency_dirs(base, config);
    let mut read: BTreeMap<&str, Tables> = BTreeMap::new();

    for (name, dir) in &dirs {
        read.insert(name, read_tables(files, dir)?);
    }

    for (name, dir) in &dirs {
        let tables = &read[name.as_str()];
        let mut paths: Vec<PathBuf> = tables
            .package
            .import_path
            .0
            .iter()
            .map(|p| dir.join(p))
            .collect();
        // The shared copy (`foo`) is found through the program's roots; the others are reached
        // only by the packages routed to them.
        let module = module_dir(&config.dependencies[name], tables, dir);

        if !name.contains('@')
            && module != *dir
            && let Some(parent) = module.parent()
        {
            roots.modules.push(parent.to_path_buf());
        }

        for (wanted_name, wanted) in &tables.dependencies {
            let keys = copies
                .get(wanted_name.as_str())
                .map_or(&[][..], Vec::as_slice);

            if keys.len() < 2 || import_name(name) == wanted_name {
                continue;
            }

            // The shared copy is tried first, so a package it satisfies is not routed.
            let mut keys = keys.to_vec();

            keys.sort_by_key(|key| key.contains('@'));
            let mut chosen = None;

            for key in keys {
                let refused = |r: Refused| ConfigError {
                    path: if r.in_package {
                        dir.join(FILE_NAME)
                    } else {
                        base.join(FILE_NAME)
                    },
                    message: if r.in_package {
                        format!("dependency `{wanted_name}`, {}", r.message)
                    } else {
                        r.message
                    },
                };

                if admits(key, &config.dependencies[key], wanted).map_err(refused)? {
                    chosen = Some(key);
                    break;
                }
            }

            let Some(key) = chosen else {
                continue;
            };

            if key.contains('@') {
                let copy_dir = &dirs.iter().find(|(n, _)| n == key).expect("a listed key").1;
                let copy = module_dir(&config.dependencies[key], &read[key], copy_dir);

                if let Some(parent) = copy.parent() {
                    paths.push(parent.to_path_buf());
                }
            }
        }

        if !paths.is_empty() {
            roots.packages.push((dir.clone(), paths));
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

    /// Files by path suffix.
    struct Many(Vec<(&'static str, &'static str)>);

    impl Files for Many {
        fn is_file(&self, path: &Path) -> bool {
            self.0.iter().any(|(p, _)| path.ends_with(p))
        }

        fn read(&self, path: &Path) -> std::io::Result<String> {
            self.0
                .iter()
                .find(|(p, _)| path.ends_with(p))
                .map(|(_, text)| text.to_string())
                .ok_or_else(|| std::io::ErrorKind::NotFound.into())
        }
    }

    /// `a` and `b` need `foo 1.x`, `c` needs `foo 2.x`, `d` needs `foo 1.y`; `foo` is shared at
    /// 1.4.0 and 2.0.0 sits in its own group.
    fn grouped() -> (Many, Config) {
        let files = Many(vec![
            (
                "modules/a/jacket.toml",
                "[dependencies]\nfoo = { git = \"x/foo\", version = \"^1.2\" }\n",
            ),
            (
                "modules/c/jacket.toml",
                "[dependencies]\nfoo = { git = \"x/foo\", version = \"^2\" }\n",
            ),
            (
                "modules/d/jacket.toml",
                "[package]\nimport-path = [\"lib\"]\n[dependencies]\nfoo = { git = \"x/foo\", version = \">=1.0, <1.4\" }\n",
            ),
        ]);
        let config = toml::from_str(
            r#"[dependencies]
a = { git = "x/a", tag = "v1" }
c = { git = "x/c", tag = "v1" }
d = { git = "x/d", tag = "v1" }
foo = { git = "x/foo", tag = "v1.4.0", shared = true }
"foo@2" = { git = "x/foo", tag = "v2.0.0", shared = true, path = "modules/.versions/foo@2/foo" }
"#,
        )
        .unwrap();

        (files, config)
    }

    fn routed(roots: &Roots, package: &str) -> Vec<PathBuf> {
        roots
            .packages
            .iter()
            .find(|(dir, _)| dir.ends_with(package))
            .map(|(_, paths)| paths.clone())
            .unwrap_or_default()
    }

    #[test]
    fn a_package_is_routed_to_the_copy_its_range_admits() {
        let (files, config) = grouped();
        let roots = dependency_roots(&files, Path::new("/p"), &config).unwrap();
        let other = PathBuf::from("/p/modules/.versions/foo@2");

        assert_eq!(routed(&roots, "modules/c"), [other]);
        // The shared copy satisfies `a`; nothing is added.
        assert!(routed(&roots, "modules/a").is_empty());
        // `d`'s range excludes the shared 1.4.0 and the other copy is 2.x: not routed anywhere.
        assert_eq!(
            routed(&roots, "modules/d"),
            [PathBuf::from("/p/modules/d/lib")]
        );
    }

    #[test]
    fn one_copy_is_never_routed() {
        let (files, mut config) = grouped();

        config.dependencies.remove("foo@2");
        let roots = dependency_roots(&files, Path::new("/p"), &config).unwrap();

        assert!(routed(&roots, "modules/c").is_empty());
    }

    fn error_of(requirer: &'static str, config: &str) -> ConfigError {
        let (files, _) = grouped();
        let mut texts = files.0;

        texts.retain(|(p, _)| !p.contains("modules/c/"));
        texts.push(("modules/c/jacket.toml", requirer));
        let config: Config = toml::from_str(config).unwrap();
        dependency_roots(&Many(texts), Path::new("/p"), &config).unwrap_err()
    }

    const COPIES: &str = r#"[dependencies]
c = { git = "x/c", tag = "v1" }
foo = { git = "x/foo", tag = "v1.4.0", shared = true }
"foo@2" = { git = "x/foo", tag = "v2.0.0", shared = true, path = "modules/.versions/foo@2/foo" }
"#;

    #[test]
    fn a_scheme_or_range_that_does_not_read_is_an_error_naming_the_package_and_value() {
        let error = error_of(
            "[dependencies]\nfoo = { git = \"x/foo\", version = \"^2\", scheme = \"roman\" }\n",
            COPIES,
        );

        assert_eq!(error.path, PathBuf::from("/p/modules/c/jacket.toml"));
        assert!(error.message.contains("roman"), "{error}");
        let error = error_of(
            "[dependencies]\nfoo = { git = \"x/foo\", version = \"not a range\" }\n",
            COPIES,
        );

        assert_eq!(error.path, PathBuf::from("/p/modules/c/jacket.toml"));
        assert!(error.message.contains("not a range"), "{error}");
    }

    #[test]
    fn a_copy_whose_scheme_or_tag_does_not_read_is_an_error_in_the_projects_file() {
        let request = "[dependencies]\nfoo = { git = \"x/foo\", version = \"^2\" }\n";
        let error = error_of(
            request,
            &COPIES.replace("tag = \"v1.4.0\"", "tag = \"v1.4.0\", scheme = \"roman\""),
        );

        assert_eq!(error.path, PathBuf::from("/p/jacket.toml"));
        assert!(
            error.message.contains("`foo`") && error.message.contains("roman"),
            "{error}"
        );
        let error = error_of(request, &COPIES.replace("v1.4.0", "nightly"));

        assert_eq!(error.path, PathBuf::from("/p/jacket.toml"));
        assert!(error.message.contains("nightly"), "{error}");
    }
}
