//! Path helpers shared by the compiler, the language server and the linter: lexical
//! normalisation, canonicalisation without Windows' verbatim prefix, a cache for it, and the
//! standard library directory a native tool starts with.
use crate::fxhash::HashMap;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

/// The file a folder module is entered through: `Name/module.jai`.
pub const MODULE_ENTRY: &str = "module.jai";

/// Whether `path` is the entry file of a folder module.
pub fn is_module_entry(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == MODULE_ENTRY)
}

/// The entry file of the folder module `dir`.
pub fn module_entry(dir: &Path) -> PathBuf {
    dir.join(MODULE_ENTRY)
}

pub use jacket_config::{slashed, to_slash};

/// The name a module is known by: its path below an import directory, `/`-separated
/// (`Extensions/SIMD`).
pub fn module_key(relative: &Path) -> String {
    slashed(relative)
}

/// A path as the program sees it (`#file`, `#filepath`): `/`-separated on Windows too, so a
/// metaprogram can put it in a string literal or splice it into `#load "..."` without escaping
/// backslashes.
pub fn program_path(path: &str) -> String {
    if cfg!(windows) {
        to_slash(path)
    } else {
        path.to_string()
    }
}

/// Lexically normalize `a/./b/../c` without touching the filesystem.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // `/..` is `/`, as on a real filesystem; a relative path keeps leading `..`.
                if out.has_root() && out.parent().is_none() {
                } else if out
                    .components()
                    .next_back()
                    .is_none_or(|c| c == Component::ParentDir)
                    || !out.pop()
                {
                    out.push("..");
                }
            }
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// `std::fs::canonicalize` without Windows' verbatim prefix: `\\?\C:\dir` becomes `C:\dir` and
/// `\\?\UNC\server\share` becomes `\\server\share`. A verbatim working directory or `#file` breaks
/// programs: `/tmp` relative to `\\?\C:\dir` is no folder on drive C, and the prefix shows up in
/// every path the program prints. Elsewhere this is `std::fs::canonicalize`.
pub fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let real = std::fs::canonicalize(path)?;
    Ok(strip_verbatim(&real.to_string_lossy()).map_or(real, Into::into))
}

/// The plain form of a verbatim Windows path (see [`canonicalize`]), or `None` when `path` is not
/// verbatim or has no plain form (a component that Win32 path parsing would change).
pub fn strip_verbatim(path: &str) -> Option<String> {
    let rest = path.strip_prefix(r"\\?\")?;
    let plain = if let Some(share) = rest.strip_prefix(r"UNC\") {
        format!(r"\\{share}")
    } else {
        let drive = rest.as_bytes();
        if drive.len() < 2 || !drive[0].is_ascii_alphabetic() || drive[1] != b':' {
            return None;
        }
        rest.to_string()
    };
    // Win32 parsing trims trailing dots and spaces and treats `/` as a separator; such names only
    // mean the same thing in their verbatim form, as does anything longer than MAX_PATH.
    let odd = plain.contains('/')
        || plain.len() >= 260
        || plain
            .split('\\')
            .any(|c| c.ends_with('.') || c.ends_with(' '));
    (!odd).then_some(plain)
}

/// A native path in the one spelling tools compare: on Windows without the verbatim prefix,
/// `/`-separated, with an upper-case drive letter; elsewhere as it is. `None` for a verbatim path
/// that has no plain form.
pub fn native_spelling(path: &str, windows: bool) -> Option<String> {
    if !windows {
        return Some(path.to_owned());
    }
    let mut result = path.to_owned();
    if result.starts_with(r"\\?\") {
        result = strip_verbatim(&result)?;
    }
    result = to_slash(&result);
    let bytes = result.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        result.replace_range(0..1, &result[..1].to_ascii_uppercase());
    }
    Some(result)
}

/// The identity of the file at `path` on `fs`: its canonical path (links resolved when the file
/// system can) in [`native_spelling`] form. File names keep their case, since canonicalization
/// already knows the real spelling and some Windows folders are case-sensitive. The language
/// server, the linter and the compiler's own file table all compare files through this, so one
/// file reached through a link, a verbatim prefix or a different drive-letter case is one file.
pub fn file_identity(fs: &dyn crate::sema::FileSystem, path: &Path) -> PathBuf {
    file_identity_on(fs, path, cfg!(windows))
}

/// [`file_identity`] with the Windows rules chosen by `windows` (for tests on any host).
pub fn file_identity_on(fs: &dyn crate::sema::FileSystem, path: &Path, windows: bool) -> PathBuf {
    identity_spelling_on(&fs.canonical(path), windows)
}

/// [`native_spelling`] of an already canonical path, left as it is when it has no plain form.
pub fn identity_spelling(path: &Path) -> PathBuf {
    identity_spelling_on(path, cfg!(windows))
}

fn identity_spelling_on(path: &Path, windows: bool) -> PathBuf {
    path.to_str()
        .and_then(|text| native_spelling(text, windows))
        .map_or_else(|| path.to_path_buf(), PathBuf::from)
}

/// Answers about paths, remembered: the file system does not change under one run, and a
/// program names hundreds of the same files. Holds at most [`CanonCache::LIMIT`] paths and starts
/// over beyond that.
#[derive(Default)]
pub struct CanonCache(Mutex<HashMap<PathBuf, PathBuf>>);

impl CanonCache {
    /// How many paths are kept before the cache is emptied.
    pub const LIMIT: usize = 20_000;

    pub fn new() -> Self {
        Self::default()
    }

    /// [`canonicalize`] of `path`, remembered. Only paths that exist are kept: a missing file
    /// may appear later.
    pub fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        if let Some(found) = self.lookup(path) {
            return Ok(found);
        }
        let real = canonicalize(path)?;
        self.store(path, real.clone());
        Ok(real)
    }

    /// The remembered answer for `path`, else what `resolve` gives (which is then remembered).
    pub fn get_or_insert_with(
        &self,
        path: &Path,
        resolve: impl FnOnce(&Path) -> PathBuf,
    ) -> PathBuf {
        if let Some(found) = self.lookup(path) {
            return found;
        }
        let found = resolve(path);
        self.store(path, found.clone());
        found
    }

    /// Forget everything (files appeared or went away).
    pub fn clear(&self) {
        self.0.lock().expect("path cache lock").clear();
    }

    /// The remembered answer for `path`, if there is one.
    pub fn get(&self, path: &Path) -> Option<PathBuf> {
        self.lookup(path)
    }

    /// Remember that `path` is `found`.
    pub fn remember(&self, path: &Path, found: PathBuf) {
        self.store(path, found);
    }

    fn lookup(&self, path: &Path) -> Option<PathBuf> {
        self.0.lock().expect("path cache lock").get(path).cloned()
    }

    fn store(&self, path: &Path, found: PathBuf) {
        let mut map = self.0.lock().expect("path cache lock");
        if map.len() >= Self::LIMIT {
            map.clear();
        }
        map.insert(path.to_path_buf(), found);
    }
}

/// The standard library directory for a native `jaic`, `jailsp` or `jailint`: `JAIC_STDLIB`,
/// else `stdlib/` next to the executable (release archives), else the repository's (for
/// development).
pub fn default_stdlib_dir() -> PathBuf {
    stdlib_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
}

/// [`default_stdlib_dir`] with `fallback` as the last resort. "Next to the executable" is checked
/// both where it was started from and, through symlinks, where it really is: an archive unpacked
/// into `/opt` and linked from `~/bin` finds `/opt/.../stdlib` (macOS reports the symlink's path as
/// the executable).
pub fn stdlib_dir(fallback: PathBuf) -> PathBuf {
    if let Some(dir) = jacket_config::env::STDLIB.os() {
        return dir.into();
    }
    let Ok(exe) = std::env::current_exe() else {
        return fallback;
    };
    let real = std::fs::canonicalize(&exe).ok();
    [Some(exe), real]
        .into_iter()
        .flatten()
        .filter_map(|exe| Some(exe.parent()?.join("stdlib")))
        .find(|dir| dir.is_dir() && crate::find_preload(&crate::sema::NativeFs, dir).is_some())
        .unwrap_or(fallback)
}

/// Whether the last component of `path` starts with a dot.
pub fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_none_or(|n| n.starts_with('.'))
}

/// The visible directories directly inside `dir`, sorted. An unreadable directory has none.
pub fn subdirs_of(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| !is_hidden(p) && p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// The visible `.jai` files directly inside `dir`, sorted. An unreadable directory has none.
pub fn jai_files_in(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "jai") && !is_hidden(p) && p.is_file())
        .collect();
    files.sort();
    files
}

/// The visible `.jai` files under `dir` at any depth, in path order. A directory or file
/// for which `prune` answers true is skipped, together with everything beneath it.
pub fn jai_files_under(dir: &Path, prune: &mut dyn FnMut(&Path) -> bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_jai_files(dir, prune, &mut out);
    out
}

fn collect_jai_files(dir: &Path, prune: &mut dyn FnMut(&Path) -> bool, out: &mut Vec<PathBuf>) {
    let mut children: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .collect();
    children.sort();
    for child in children {
        if is_hidden(&child) || prune(&child) {
            continue;
        }
        if child.is_dir() {
            collect_jai_files(&child, prune, out);
        } else if child.extension().is_some_and(|e| e == "jai") {
            out.push(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_lexically() {
        assert_eq!(
            normalize(Path::new("/p/src/../a/./b.jai")),
            Path::new("/p/a/b.jai")
        );
        assert_eq!(normalize(Path::new("/../a")), Path::new("/a"));
        assert_eq!(normalize(Path::new("../../a/../b")), Path::new("../../b"));
        assert_eq!(normalize(Path::new("a/../..")), Path::new(".."));
    }

    #[test]
    fn verbatim_prefixes_are_stripped_when_a_plain_form_exists() {
        assert_eq!(
            strip_verbatim(r"\\?\D:\a\jai\tests").as_deref(),
            Some(r"D:\a\jai\tests")
        );
        assert_eq!(
            strip_verbatim(r"\\?\UNC\host\share\x").as_deref(),
            Some(r"\\host\share\x")
        );
        assert_eq!(strip_verbatim(r"C:\plain"), None);
        assert_eq!(strip_verbatim("/usr/lib"), None);
        assert_eq!(strip_verbatim(r"\\?\Volume{1234}\x"), None);
        assert_eq!(strip_verbatim(r"\\?\C:\trailing dot."), None);
        assert_eq!(
            strip_verbatim(&format!(r"\\?\C:\{}", "x".repeat(300))),
            None
        );
    }

    #[test]
    fn canonicalize_resolves_existing_paths_and_rejects_missing_ones() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let real = canonicalize(manifest.join("src/../src/lib.rs")).unwrap();
        assert!(real.ends_with("src/lib.rs") || real.ends_with(r"src\lib.rs"));
        assert!(canonicalize(manifest.join("no/such/file.jai")).is_err());
    }

    #[test]
    fn the_cache_keeps_resolved_paths_only() {
        let cache = CanonCache::new();
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = manifest.join("src/../src/lib.rs");
        let first = cache.canonicalize(&path).unwrap();
        assert_eq!(first, canonicalize(&path).unwrap());
        assert_eq!(cache.canonicalize(&path).unwrap(), first);
        let missing = manifest.join("no/such/file.jai");
        assert!(cache.canonicalize(&missing).is_err());
        assert!(cache.lookup(&missing).is_none());
        let mut calls = 0;
        for _ in 0..2 {
            let got = cache.get_or_insert_with(&missing, |p| {
                calls += 1;
                normalize(p)
            });
            assert_eq!(got, normalize(&missing));
        }
        assert_eq!(calls, 1);
        cache.clear();
        assert!(cache.lookup(&path).is_none());
    }

    #[test]
    fn the_cache_starts_over_at_its_limit() {
        let cache = CanonCache::new();
        for i in 0..=CanonCache::LIMIT {
            cache.get_or_insert_with(Path::new(&format!("/p/{i}")), std::path::Path::to_path_buf);
        }
        assert_eq!(cache.0.lock().unwrap().len(), 1);
    }

    #[test]
    fn the_stdlib_comes_from_the_environment_or_the_repository() {
        // `JAIC_STDLIB` is set by the tools' own tests; the fallback is what is checked here.
        if jacket_config::env::STDLIB.os().is_none() {
            let fallback = PathBuf::from("/nowhere/stdlib");
            assert_eq!(stdlib_dir(fallback.clone()), fallback);
            assert!(crate::find_preload(&crate::sema::NativeFs, &default_stdlib_dir()).is_some());
        }
    }
}
