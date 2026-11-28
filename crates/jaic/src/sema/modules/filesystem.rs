//! Finding a module on a file system.
use std::path::Path;

pub use crate::filesystem::{FileSystem, NativeFs, VirtualFs, content_stamp};
use crate::sema::{PathBuf, ast};

/// The modules the compiler loads itself, which `#import` reaches by their protocol names
/// rather than by searching the import path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapModule {
    /// The selected bootstrap entry (`Options::preload`).
    Preload,
    /// The runtime, one instance per program.
    RuntimeSupport,
}

impl BootstrapModule {
    /// The name an `#import` writes.
    pub const fn name(self) -> &'static str {
        match self {
            BootstrapModule::Preload => crate::lang_items::modules::PRELOAD,
            BootstrapModule::RuntimeSupport => crate::lang_items::modules::RUNTIME_SUPPORT,
        }
    }

    /// The bootstrap module an `#import` of `name` means.
    pub fn named(name: &str) -> Option<Self> {
        [BootstrapModule::Preload, BootstrapModule::RuntimeSupport]
            .into_iter()
            .find(|module| module.name() == name)
    }
}

/// The entry file of the module `name` directly under `dir`: `Name.jai`, else `Name/module.jai`.
/// This is the one definition of what a module on disk looks like.
pub(super) fn module_file_in(fs: &dyn FileSystem, dir: &Path, name: &str) -> Option<PathBuf> {
    let file = dir.join(format!("{name}.jai"));

    if fs.is_file(&file) {
        return Some(file);
    }

    let entry = crate::paths::module_entry(&dir.join(name));
    fs.is_file(&entry).then_some(entry)
}

/// The module a directory listing entry is, if it is one: `Name.jai`, or a folder `Name` with a
/// `module.jai`.
pub(crate) fn module_listed(
    fs: &dyn FileSystem,
    dir: &Path,
    entry: &str,
    is_dir: bool,
) -> Option<String> {
    if is_dir {
        fs.is_file(&crate::paths::module_entry(&dir.join(entry)))
            .then(|| entry.to_string())
    } else {
        entry.strip_suffix(".jai").map(str::to_string)
    }
}

/// Folders that only the files of one package search: a dependency's own `import-path`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageImports {
    /// The package's folder; every file below it belongs to the package.
    pub dir: PathBuf,
    /// Searched for those files after their own `modules` folder and before the program's roots.
    pub import_paths: Vec<PathBuf>,
}

impl PackageImports {
    /// The `(package folder, folder)` pairs of `-package_import_dir` flags, one entry per
    /// package with its folders in the order given.
    pub fn grouped(pairs: &[(PathBuf, PathBuf)]) -> Vec<Self> {
        let mut out: Vec<Self> = Vec::new();

        for (package, dir) in pairs {
            match out.iter_mut().find(|p| &p.dir == package) {
                Some(found) => found.import_paths.push(dir.clone()),
                None => out.push(Self {
                    dir: package.clone(),
                    import_paths: vec![dir.clone()],
                }),
            }
        }

        out
    }
}

/// The folders an `#import "Name"` written in a file of `from_dir` searches, in order: the
/// `modules` folder next to that file, the `import_paths` of the innermost package holding the
/// file, then each of the program's `import_paths` (a project's own folders, then `-import_dir`
/// folders, then the standard library last; see `docs/compiler/project.md`).
/// The program's list serves every file: a file of the standard library resolves a name exactly
/// as the module that loads it does, so a project's replacement for a library module is seen by
/// all of the library or by none of it. A package's own folders reach only its own files, so a
/// dependency cannot replace a module for the rest of the program.
pub(super) fn search_roots(
    import_paths: &[PathBuf],
    packages: &[PackageImports],
    from_dir: &Path,
) -> Vec<PathBuf> {
    let own = packages
        .iter()
        .filter(|p| from_dir.starts_with(&p.dir))
        .max_by_key(|p| p.dir.components().count());
    let mut roots = Vec::with_capacity(import_paths.len() + 1);

    roots.push(from_dir.join("modules"));
    roots.extend(own.into_iter().flat_map(|p| p.import_paths.iter().cloned()));
    roots.extend(import_paths.iter().cloned());
    roots
}

/// Find `Name.jai` or `Name/module.jai` in the [`search_roots`], first match wins. The public
/// `Preload` protocol name also aliases the single bootstrap entry, found in the first root
/// that has one, when no module of that name exists.
pub fn find_module_in(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    packages: &[PackageImports],
    name: &str,
    from_dir: &Path,
) -> Option<PathBuf> {
    let roots = search_roots(import_paths, packages, from_dir);

    if let Some(entry) = roots.iter().find_map(|dir| module_file_in(fs, dir, name)) {
        return Some(entry);
    }

    if BootstrapModule::named(name) == Some(BootstrapModule::Preload) {
        roots.iter().find_map(|dir| crate::find_preload(fs, dir))
    } else {
        None
    }
}

/// The entry file an `#import` written in a file of `dir` loads: a module found on the import
/// path, `#import,file` relative to `dir`, or `#import,dir`'s `module.jai`. `None` for an
/// unknown module or `#import,string`. (The file itself may not exist for `,file`/`,dir`.)
pub fn import_entry(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    packages: &[PackageImports],
    source: &ast::ImportSource,
    dir: &Path,
) -> Option<PathBuf> {
    match source {
        ast::ImportSource::Module(name) => find_module_in(fs, import_paths, packages, name, dir),
        ast::ImportSource::File(path) => Some(dir.join(&**path)),
        ast::ImportSource::Dir(path) => Some(crate::paths::module_entry(&dir.join(&**path))),
        ast::ImportSource::String(_) => None,
    }
}

/// The module names a directory offers: `Name.jai` files and `Name/module.jai` folders.
pub(super) fn modules_in(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    fs.list_dir(dir)
        .into_iter()
        .filter_map(|(name, is_dir)| module_listed(fs, dir, &name, is_dir))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every row is one lookup: what is searched for, from which folder, and the file found.
    /// The files are `/project/modules` (the project's own `Windows`), `/project/src/modules`
    /// (a module private to that folder), `/libs` (an `-import_dir` folder) and `/std` (the
    /// library, last).
    #[test]
    fn search_roots_in_order() {
        let mut fs = VirtualFs::default();

        for file in [
            "/project/modules/Windows/module.jai",
            "/project/src/modules/Local.jai",
            "/libs/Shared.jai",
            "/libs/Windows.jai",
            "/std/Windows/module.jai",
            "/std/Basic/module.jai",
            "/std/Shared/module.jai",
            "/std/Objective_C/module.jai",
            "/std/prelude/module.jai",
        ] {
            fs.insert(file, b"".to_vec());
        }

        let paths = [
            PathBuf::from("/project/modules"),
            PathBuf::from("/libs"),
            PathBuf::from("/std"),
        ];
        let table: &[(&str, &str, Option<&str>)] = &[
            // The folder's own `modules` comes before the import paths.
            (
                "Local",
                "/project/src",
                Some("/project/src/modules/Local.jai"),
            ),
            ("Local", "/project/other", None),
            // Import paths in the order given: the project, `-import_dir`, the library.
            (
                "Windows",
                "/project/src",
                Some("/project/modules/Windows/module.jai"),
            ),
            ("Shared", "/project/src", Some("/libs/Shared.jai")),
            ("Basic", "/project/src", Some("/std/Basic/module.jai")),
            // A file of the library searches the same roots as any other file: a project's
            // replacement is seen by the whole library.
            (
                "Windows",
                "/std/internal/files",
                Some("/project/modules/Windows/module.jai"),
            ),
            (
                "Windows",
                "/std/Thread",
                Some("/project/modules/Windows/module.jai"),
            ),
            (
                "Objective_C",
                "/std/GL",
                Some("/std/Objective_C/module.jai"),
            ),
            // A name no root has.
            ("Missing", "/project/src", None),
            // The `Preload` protocol name aliases the prelude, wherever it is written.
            ("Preload", "/project/src", Some("/std/prelude/module.jai")),
            (
                "Preload",
                "/std/internal/files",
                Some("/std/prelude/module.jai"),
            ),
        ];

        for (name, from, expected) in table {
            assert_eq!(
                find_module_in(&fs, &paths, &[], name, Path::new(from)),
                expected.map(PathBuf::from),
                "`#import \"{name}\"` from {from}"
            );
        }

        assert_eq!(
            search_roots(&paths, &[], Path::new("/project/src")),
            [
                PathBuf::from("/project/src/modules"),
                PathBuf::from("/project/modules"),
                PathBuf::from("/libs"),
                PathBuf::from("/std"),
            ]
        );
    }

    /// A package's own folders come after its `modules` folder and before the program's, and
    /// only its own files see them.
    #[test]
    fn a_packages_folders_reach_only_its_own_files() {
        let mut fs = VirtualFs::default();

        for file in [
            "/p/modules/Basic/module.jai",
            "/p/modules/dep/module.jai",
            "/p/modules/dep/vendor/Basic/module.jai",
            "/p/modules/dep/vendor/Extra/module.jai",
            "/p/modules/dep/src/modules/Near.jai",
            "/p/modules/other/module.jai",
        ] {
            fs.insert(file, b"".to_vec());
        }

        let paths = [PathBuf::from("/p/modules")];
        let packages = [PackageImports {
            dir: "/p/modules/dep".into(),
            import_paths: vec!["/p/modules/dep/vendor".into()],
        }];
        let find =
            |name: &str, from: &str| find_module_in(&fs, &paths, &packages, name, Path::new(from));

        // The package's own `Basic` is seen from the package, and from nowhere else.
        assert_eq!(
            find("Basic", "/p/modules/dep/src"),
            Some("/p/modules/dep/vendor/Basic/module.jai".into())
        );
        assert_eq!(
            find("Basic", "/p/modules/other"),
            Some("/p/modules/Basic/module.jai".into())
        );
        assert_eq!(
            find("Basic", "/p"),
            Some("/p/modules/Basic/module.jai".into())
        );
        assert_eq!(find("Extra", "/p/modules/other"), None);
        assert_eq!(
            find("Extra", "/p/modules/dep"),
            Some("/p/modules/dep/vendor/Extra/module.jai".into())
        );
        // The file's own `modules` folder still goes first.
        assert_eq!(
            find("Near", "/p/modules/dep/src"),
            Some("/p/modules/dep/src/modules/Near.jai".into())
        );
        // A sibling folder with the package's prefix is not inside it.
        assert_eq!(find("Extra", "/p/modules/dependency"), None);
    }
}
