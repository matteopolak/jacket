//! A dependency's own `jacket.toml`, read as data.
//!
//! Only `[package]`, `[dependencies]` and the import folders of `[project]` are read, because
//! everything else (`[target]`, `[lint]`, `[fmt]`, `[experimental]`, runners) belongs to whoever
//! builds the program. A dependency can never change how the program that uses it builds.
use jacket_config::{Dependency, FILE_NAME, Package};
use std::collections::BTreeMap;
use std::path::Path;

/// What a dependency says about itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: BTreeMap<String, Dependency>,
    /// `[package] import_path`, else `[project] import_path`: the folders only this package's
    /// files search.
    pub import_path: Vec<String>,
}

/// The tables of `text` that a dependency may set. `None` when the text is not TOML.
pub fn read_text(text: &str) -> Option<Manifest> {
    let table: toml::Table = text.parse().ok()?;
    let package: Package = table
        .get("package")
        .and_then(|v| v.clone().try_into().ok())
        .unwrap_or_default();
    let dependencies: BTreeMap<String, Dependency> = table
        .get("dependencies")
        .and_then(|v| v.clone().try_into().ok())
        .unwrap_or_default();
    let project_paths: Vec<String> = table
        .get("project")
        .and_then(|p| p.get("import_path"))
        .map(|v| match v {
            toml::Value::String(one) => vec![one.clone()],
            toml::Value::Array(many) => many
                .iter()
                .filter_map(|m| m.as_str().map(str::to_string))
                .collect(),
            _ => Vec::new(),
        })
        .unwrap_or_default();
    let import_path = if package.import_path.0.is_empty() {
        project_paths
    } else {
        package.import_path.0.clone()
    };
    Some(Manifest {
        package,
        dependencies,
        import_path,
    })
}

/// The manifest of the checkout at `dir`; `None` without a `jacket.toml` or when it is invalid.
pub fn read(dir: &Path) -> Option<Manifest> {
    read_text(&std::fs::read_to_string(dir.join(FILE_NAME)).ok()?)
}

/// Where the module of a checkout is, relative to its root: `""` for the root itself.
pub fn find_module(
    checkout: &Path,
    manifest: &Manifest,
    asked: Option<&str>,
) -> Result<String, String> {
    let has = |dir: &str| checkout.join(dir).join("module.jai").is_file();
    let tidy = |dir: &str| dir.trim_matches('/').trim_start_matches("./").to_string();
    if let Some(dir) = asked.or(manifest.package.module.as_deref()) {
        let dir = tidy(dir);
        return if has(&dir) {
            Ok(dir)
        } else {
            Err(format!(
                "{} has no module.jai",
                if dir.is_empty() {
                    "the repository root".to_string()
                } else {
                    dir
                }
            ))
        };
    }
    if has("") {
        return Ok(String::new());
    }
    let mut found: Vec<String> = std::fs::read_dir(checkout.join("modules"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("module.jai").is_file())
        .map(|e| format!("modules/{}", e.file_name().to_string_lossy()))
        .collect();
    found.sort();
    match found.as_slice() {
        [one] => Ok(one.clone()),
        [] => Err("it has no module.jai at its root and no modules/<Name>/module.jai".into()),
        many => Err(format!(
            "it has several modules ({}) and none at its root",
            many.join(", ")
        )),
    }
}

/// The name a module is imported by: its folder's name, or `fallback` (the dependency's name)
/// for the repository root.
pub fn import_name(module: &str, fallback: &str) -> String {
    module
        .rsplit('/')
        .next()
        .filter(|n| !n.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

/// The folder that must be searched for the module to be found by name: the module folder's
/// parent. `None` for a module at the root, which sits where `modules/<name>` already is.
pub fn module_parent(module: &str) -> Option<&str> {
    module
        .rsplit_once('/')
        .map(|(parent, _)| parent)
        .or_else(|| (!module.is_empty()).then_some(""))
}

/// Why this jacket cannot use a package, if it cannot.
pub fn requires_jacket(package: &Package, jacket: &str) -> Result<(), String> {
    let Some(text) = &package.requires_jacket else {
        return Ok(());
    };
    let req = semver::VersionReq::parse(text)
        .map_err(|e| format!("requires_jacket `{text}` is not a version range: {e}"))?;
    let have = semver::Version::parse(jacket).map_err(|e| e.to_string())?;
    // A prerelease build of jacket counts as its release for this check.
    let release = semver::Version::new(have.major, have.minor, have.patch);
    if req.matches(&release) {
        Ok(())
    } else {
        Err(format!(
            "it requires jacket {text}, and this is jacket {jacket}"
        ))
    }
}

/// A note when the package does not list the operating system this runs on.
pub fn platform_note(package: &Package, host: &str) -> Option<String> {
    (!package.platforms.is_empty() && !package.platforms.iter().any(|p| p == host)).then(|| {
        format!(
            "it lists {} as its platforms, and this is {host}",
            package.platforms.join(", ")
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_package_dependencies_and_import_folders_are_read() {
        let manifest = read_text(
            "[package]\nname = \"my_lib\"\nversion = \"1.2.0\"\nmodule = \"src/My_Lib\"\nrequires_jacket = \">=0.40\"\nplatforms = [\"macos\"]\n\
             [dependencies]\nutf8 = { git = \"x/utf8\", version = \"^0.4\" }\n\
             [project]\nentry = [\"evil.jai\"]\nimport_path = [\"vendor\"]\n[target.pico]\ntriple = \"t\"\n[lint]\nmax_nesting = 1\n",
        )
        .unwrap();
        assert_eq!(manifest.package.name.as_deref(), Some("my_lib"));
        assert_eq!(manifest.package.module.as_deref(), Some("src/My_Lib"));
        assert_eq!(
            manifest.dependencies["utf8"].version.as_deref(),
            Some("^0.4")
        );
        assert_eq!(manifest.import_path, ["vendor"]);
        let own =
            read_text("[package]\nimport_path = \"mine\"\n[project]\nimport_path = [\"vendor\"]\n")
                .unwrap();
        assert_eq!(own.import_path, ["mine"], "[package] import_path wins");
        assert!(read_text("[package\n").is_none());
        assert_eq!(read_text("").unwrap(), Manifest::default());
    }

    #[test]
    fn requirements_and_platforms_are_checked_against_this_jacket() {
        let package = |req: &str| Package {
            requires_jacket: Some(req.into()),
            ..Package::default()
        };
        assert!(requires_jacket(&package(">=0.40"), "0.41.0").is_ok());
        assert!(requires_jacket(&package(">=0.40"), "0.41.0-rc.1").is_ok());
        let error = requires_jacket(&package(">=0.40"), "0.39.0").unwrap_err();
        assert!(
            error.contains(">=0.40") && error.contains("0.39.0"),
            "{error}"
        );
        assert!(requires_jacket(&package("nope"), "0.39.0").is_err());
        assert!(requires_jacket(&Package::default(), "0.39.0").is_ok());
        let linux = Package {
            platforms: vec!["linux".into()],
            ..Package::default()
        };
        assert!(platform_note(&linux, "macos").unwrap().contains("linux"));
        assert!(platform_note(&linux, "linux").is_none());
        assert!(platform_note(&Package::default(), "macos").is_none());
    }

    #[test]
    fn modules_are_found_at_the_root_in_a_single_folder_or_where_told() {
        let dir = test_support::TempDir::new("dep-package");
        let none = Manifest::default();
        assert!(find_module(&dir, &none, None).is_err());
        dir.write("modules/Only/module.jai", "");
        assert_eq!(find_module(&dir, &none, None).unwrap(), "modules/Only");
        dir.write("modules/Other/module.jai", "");
        let error = find_module(&dir, &none, None).unwrap_err();
        assert!(
            error.contains("modules/Only") && error.contains("modules/Other"),
            "{error}"
        );
        assert_eq!(
            find_module(&dir, &none, Some("modules/Other/")).unwrap(),
            "modules/Other"
        );
        assert!(find_module(&dir, &none, Some("nowhere")).is_err());
        dir.write("module.jai", "");
        assert_eq!(find_module(&dir, &none, None).unwrap(), "");
        let named = Manifest {
            package: Package {
                module: Some("modules/Only".into()),
                ..Package::default()
            },
            ..Manifest::default()
        };
        assert_eq!(find_module(&dir, &named, None).unwrap(), "modules/Only");
    }

    #[test]
    fn a_module_is_imported_by_its_folder_and_found_through_its_parent() {
        assert_eq!(import_name("src/My_Lib", "dep"), "My_Lib");
        assert_eq!(import_name("", "dep"), "dep");
        assert_eq!(module_parent("src/My_Lib"), Some("src"));
        assert_eq!(module_parent("My_Lib"), Some(""));
        assert_eq!(module_parent(""), None);
    }
}
