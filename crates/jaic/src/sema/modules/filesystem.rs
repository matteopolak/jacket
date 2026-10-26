//! The file system the compiler reads sources through, and finding a module on it.
use crate::paths::normalize;
use crate::sema::{HashMap, PathBuf, Rc, ast};
use std::path::Path;

/// Source access. Native builds read the filesystem; the browser supplies a
/// virtual file system containing the workspace, prelude and stdlib.
pub trait FileSystem {
    fn read(&self, path: &Path) -> Option<Vec<u8>>;
    fn is_file(&self, path: &Path) -> bool;
    fn is_dir(&self, path: &Path) -> bool;

    fn canonical(&self, path: &Path) -> PathBuf {
        normalize(path)
    }

    /// The real path of `path` when this file system could work it out, which is the only
    /// answer worth remembering: a file that does not exist yet may exist later. A file system
    /// with nothing but lexical answers always has one.
    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        Some(self.canonical(path))
    }

    /// Whether this is the real filesystem, which worker threads may read directly.
    fn is_native(&self) -> bool {
        false
    }

    /// Names (and whether each is a directory) directly inside `path`; used by the sandbox host.
    fn list_dir(&self, _path: &Path) -> Vec<(String, bool)> {
        Vec::new()
    }

    /// A fingerprint of the file or folder at `path` that changes whenever what [`Self::read`] or
    /// [`Self::list_dir`] would answer for it changes, or `None` when this file system cannot say
    /// cheaply. What was worked out from a set of paths is reused while their stamps stay equal
    /// (see `sema::stdlib_index`); without stamps it is worked out again for every compile.
    fn stamp(&self, _path: &Path) -> Option<u64> {
        None
    }
}

/// A [`FileSystem::stamp`] of `bytes`.
pub fn content_stamp(bytes: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut hasher = crate::fxhash::FxHasher::default();
    hasher.write(bytes);
    hasher.write_usize(bytes.len());
    hasher.finish()
}

pub struct NativeFs;

impl FileSystem for NativeFs {
    fn is_native(&self) -> bool {
        true
    }

    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn canonical(&self, path: &Path) -> PathBuf {
        self.canonical_exact(path)
            .unwrap_or_else(|| normalize(path))
    }

    fn canonical_exact(&self, path: &Path) -> Option<PathBuf> {
        crate::canonicalize(path).ok()
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let Ok(entries) = std::fs::read_dir(path) else {
            return Vec::new();
        };
        entries
            .filter_map(std::result::Result::ok)
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .collect()
    }

    /// Size and modification time (a path that does not exist has a stamp of its own).
    fn stamp(&self, path: &Path) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let mut hasher = crate::fxhash::FxHasher::default();
        match std::fs::metadata(path) {
            Ok(meta) => {
                meta.len().hash(&mut hasher);
                meta.is_dir().hash(&mut hasher);
                meta.modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos())
                    .hash(&mut hasher);
            }
            Err(_) => u64::MAX.hash(&mut hasher),
        }
        Some(hasher.finish())
    }
}

#[derive(Default)]
pub struct VirtualFs {
    files: HashMap<PathBuf, Rc<[u8]>>,
    /// Every directory holding a file, however deep, with the names directly inside it (and
    /// whether each holds more): `is_dir` is asked for each path the compiler canonicalizes and
    /// `list_dir` for each module folder, and a scan of `files` for each is quadratic in the
    /// library size.
    dirs: HashMap<PathBuf, std::collections::BTreeMap<String, bool>>,
    /// `stamp`s of files, worked out when first asked for.
    stamps: std::cell::RefCell<HashMap<PathBuf, u64>>,
}

impl VirtualFs {
    pub fn insert(&mut self, path: impl Into<PathBuf>, bytes: impl Into<Rc<[u8]>>) {
        let path = normalize(&path.into());
        let parts: Vec<_> = path.components().collect();
        let mut dir = PathBuf::new();
        for (i, part) in parts.iter().enumerate() {
            if matches!(
                part,
                std::path::Component::RootDir | std::path::Component::Prefix(_)
            ) {
                dir.push(part);
                continue;
            }
            let name = part.as_os_str().to_string_lossy().into_owned();
            let nested = i + 1 < parts.len();
            let entry = self
                .dirs
                .entry(dir.clone())
                .or_default()
                .entry(name)
                .or_insert(false);
            *entry |= nested;
            dir.push(part);
        }
        self.stamps.get_mut().clear();
        self.files.insert(path, bytes.into());
    }

    /// The files by normalized path.
    pub fn files(&self) -> &HashMap<PathBuf, Rc<[u8]>> {
        &self.files
    }
}

impl FileSystem for VirtualFs {
    fn read(&self, path: &Path) -> Option<Vec<u8>> {
        self.files.get(&normalize(path)).map(|b| b.to_vec())
    }

    fn is_file(&self, path: &Path) -> bool {
        self.files.contains_key(&normalize(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.dirs.contains_key(&normalize(path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        self.dirs
            .get(&normalize(path))
            .map(|names| names.iter().map(|(n, d)| (n.clone(), *d)).collect())
            .unwrap_or_default()
    }

    /// The contents of a file, the names inside a folder.
    fn stamp(&self, path: &Path) -> Option<u64> {
        use std::hash::{Hash, Hasher};
        let path = normalize(path);
        if let Some(stamp) = self.stamps.borrow().get(&path) {
            return Some(*stamp);
        }
        let stamp = if let Some(bytes) = self.files.get(&path) {
            content_stamp(bytes)
        } else if let Some(names) = self.dirs.get(&path) {
            let mut hasher = crate::fxhash::FxHasher::default();
            names.hash(&mut hasher);
            hasher.finish()
        } else {
            u64::MAX
        };
        self.stamps.borrow_mut().insert(path, stamp);
        Some(stamp)
    }
}

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
pub fn module_file_in(fs: &dyn FileSystem, dir: &Path, name: &str) -> Option<PathBuf> {
    let file = dir.join(format!("{name}.jai"));
    if fs.is_file(&file) {
        return Some(file);
    }
    let entry = crate::paths::module_entry(&dir.join(name));
    fs.is_file(&entry).then_some(entry)
}

/// The module a directory listing entry is, if it is one: `Name.jai`, or a folder `Name` with a
/// `module.jai`.
pub fn module_listed(fs: &dyn FileSystem, dir: &Path, entry: &str, is_dir: bool) -> Option<String> {
    if is_dir {
        fs.is_file(&crate::paths::module_entry(&dir.join(entry)))
            .then(|| entry.to_string())
    } else {
        entry.strip_suffix(".jai").map(str::to_string)
    }
}

/// Find `Name.jai` or `Name/module.jai` in `from_dir/modules`, then on `import_paths`.
/// A file under an import path's private `internal/` folder looks in that import path first,
/// so a project's replacement for a module (its own `Windows`) never reaches the library's
/// internals. Public modules see the replacement like any other import does, so a project's
/// `Objective_C` is the one the library's `GL` uses too and the two share its types.
/// The public `Preload` protocol name also aliases the single bootstrap entry.
pub fn find_module_in(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    name: &str,
    from_dir: &Path,
) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![from_dir.join("modules")];
    dirs.extend(
        import_paths
            .iter()
            .find(|root| from_dir.starts_with(root.join("internal")))
            .cloned(),
    );
    dirs.extend(import_paths.iter().cloned());
    for dir in &dirs {
        if let Some(entry) = module_file_in(fs, dir, name) {
            return Some(entry);
        }
    }
    if BootstrapModule::named(name) == Some(BootstrapModule::Preload) {
        dirs.iter().find_map(|dir| crate::find_preload(fs, dir))
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
    source: &ast::ImportSource,
    dir: &Path,
) -> Option<PathBuf> {
    match source {
        ast::ImportSource::Module(name) => find_module_in(fs, import_paths, name, dir),
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

    #[test]
    fn a_library_file_finds_its_siblings_before_other_import_paths() {
        let mut fs = VirtualFs::default();
        fs.insert("/std/Windows/module.jai", b"".to_vec());
        fs.insert("/std/Basic/module.jai", b"".to_vec());
        fs.insert("/project/libs/Windows.jai", b"".to_vec());
        let paths = [PathBuf::from("/project/libs"), PathBuf::from("/std")];
        let found = |name, dir| find_module_in(&fs, &paths, name, Path::new(dir));
        assert_eq!(
            found("Windows", "/std/internal/files"),
            Some(PathBuf::from("/std/Windows/module.jai"))
        );
        assert_eq!(
            found("Windows", "/project/src"),
            Some(PathBuf::from("/project/libs/Windows.jai"))
        );
        assert_eq!(
            found("Basic", "/project/libs"),
            Some(PathBuf::from("/std/Basic/module.jai"))
        );
    }

    #[test]
    fn a_public_library_module_sees_a_project_replacement() {
        let mut fs = VirtualFs::default();
        fs.insert("/std/Objective_C/module.jai", b"".to_vec());
        fs.insert("/project/modules/Objective_C/module.jai", b"".to_vec());
        let paths = [PathBuf::from("/project/modules"), PathBuf::from("/std")];
        assert_eq!(
            find_module_in(&fs, &paths, "Objective_C", Path::new("/std/GL")),
            Some(PathBuf::from("/project/modules/Objective_C/module.jai"))
        );
    }

    #[test]
    fn virtual_listing_names_files_and_folders_directly_inside() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/a.jai", b"".to_vec());
        fs.insert("/lib/b/module.jai", b"".to_vec());
        fs.insert("/lib/b/deep/c.jai", b"".to_vec());
        fs.insert("/lib/b.jai", b"".to_vec());
        fs.insert("/other/x.jai", b"".to_vec());
        assert_eq!(
            fs.list_dir(Path::new("/lib")),
            [
                ("a.jai".into(), false),
                ("b".into(), true),
                ("b.jai".into(), false)
            ]
        );
        assert_eq!(
            fs.list_dir(Path::new("/lib/b/../b/")),
            [("deep".into(), true), ("module.jai".into(), false)]
        );
        assert_eq!(
            fs.list_dir(Path::new("/")),
            [("lib".into(), true), ("other".into(), true)]
        );
        assert!(fs.list_dir(Path::new("/lib/a.jai")).is_empty());
        assert!(fs.is_dir(Path::new("/lib/b/deep")));
        assert!(!fs.is_dir(Path::new("/lib/a.jai")));
    }
}
