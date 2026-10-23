//! The file system the compiler reads sources through, and finding a module on it.
use crate::paths::normalize;
use crate::sema::{HashMap, HashSet, PathBuf, Rc, ast};
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
}

#[derive(Default)]
pub struct VirtualFs {
    files: HashMap<PathBuf, Rc<[u8]>>,
    /// Every directory holding a file, however deep: `is_dir` is asked for each path the
    /// compiler canonicalizes, and a scan of `files` for each is quadratic in the library size.
    dirs: HashSet<PathBuf>,
}

impl VirtualFs {
    pub fn insert(&mut self, path: impl Into<PathBuf>, bytes: impl Into<Rc<[u8]>>) {
        let path = normalize(&path.into());
        for dir in path.ancestors().skip(1) {
            if !self.dirs.insert(dir.to_path_buf()) {
                break;
            }
        }
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
        self.dirs.contains(&normalize(path))
    }

    fn list_dir(&self, path: &Path) -> Vec<(String, bool)> {
        let p = normalize(path);
        let mut out: std::collections::BTreeMap<String, bool> =
            std::collections::BTreeMap::default();
        for key in self.files.keys() {
            let Ok(rest) = key.strip_prefix(&p) else {
                continue;
            };
            let mut parts = rest.components();
            let Some(first) = parts.next() else {
                continue;
            };
            let nested = parts.next().is_some();
            let name = first.as_os_str().to_string_lossy().into_owned();
            let entry = out.entry(name).or_insert(false);
            *entry |= nested;
        }
        out.into_iter().collect()
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
            BootstrapModule::Preload => "Preload",
            BootstrapModule::RuntimeSupport => "Runtime_Support",
        }
    }

    /// The bootstrap module an `#import` of `name` means.
    pub fn named(name: &str) -> Option<Self> {
        [BootstrapModule::Preload, BootstrapModule::RuntimeSupport]
            .into_iter()
            .find(|module| module.name() == name)
    }
}

/// Find `Name.jai` or `Name/module.jai` in `from_dir/modules`, then on `import_paths`.
/// The public `Preload` protocol name also aliases the single bootstrap entry.
pub fn find_module_in(
    fs: &dyn FileSystem,
    import_paths: &[PathBuf],
    name: &str,
    from_dir: &Path,
) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![from_dir.join("modules")];
    dirs.extend(import_paths.iter().cloned());
    for dir in &dirs {
        let file = dir.join(format!("{name}.jai"));
        if fs.is_file(&file) {
            return Some(file);
        }
        let entry = dir.join(name).join("module.jai");
        if fs.is_file(&entry) {
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
        ast::ImportSource::Dir(path) => Some(dir.join(&**path).join("module.jai")),
        ast::ImportSource::String(_) => None,
    }
}

/// The module names a directory offers: `Name.jai` files and `Name/module.jai` folders.
pub(super) fn modules_in(fs: &dyn FileSystem, dir: &Path) -> Vec<String> {
    fs.list_dir(dir)
        .into_iter()
        .filter_map(|(name, is_dir)| {
            if is_dir {
                fs.is_file(&dir.join(&name).join("module.jai"))
                    .then_some(name)
            } else {
                name.strip_suffix(".jai").map(str::to_string)
            }
        })
        .collect()
}
