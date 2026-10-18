//! Path helpers shared by the compiler, the language server and the linter: lexical
//! normalisation, canonicalisation without Windows' verbatim prefix, a cache for it, and the
//! standard library directory a native tool starts with.
use std::collections::HashMap;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

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
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
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
        .find(|dir| crate::find_preload(&crate::sema::NativeFs, dir).is_some())
        .unwrap_or(fallback)
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
            cache.get_or_insert_with(Path::new(&format!("/p/{i}")), |p| p.to_path_buf());
        }
        assert_eq!(cache.0.lock().unwrap().len(), 1);
    }

    #[test]
    fn the_stdlib_comes_from_the_environment_or_the_repository() {
        // `JAIC_STDLIB` is set by the tools' own tests; the fallback is what is checked here.
        if std::env::var_os("JAIC_STDLIB").is_none() {
            let fallback = PathBuf::from("/nowhere/stdlib");
            assert_eq!(stdlib_dir(fallback.clone()), fallback);
            assert!(crate::find_preload(&crate::sema::NativeFs, &default_stdlib_dir()).is_some());
        }
    }
}
