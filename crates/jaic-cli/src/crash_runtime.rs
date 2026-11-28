//! The private POSIX crash runtime, embedded by `build.rs` and unpacked on first use.
//!
//! `stdlib/internal/runtime/crash-handler.jai` binds it as `crash-native/portable_crash`; when
//! no such file sits beside the source, the interpreter and the linker look for it by file
//! name in the library directories, and this module supplies one.

use std::fs;
use std::path::{Path, PathBuf};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/crash_runtime.rs"));
}

/// The library name the interpreter and the linker know the runtime by.
pub(crate) const LIBRARY: &str = "portable_crash";

/// What to do when the runtime cannot be unpacked.
const FIX: &str = "note: set JAIC_CACHE_DIR to a writable folder";

/// Where unpacked helpers live: `JAIC_CACHE_DIR`, else `jaic` in the user cache.
fn cache_root() -> Result<PathBuf, String> {
    if let Some(dir) = jacket_config::env::CACHE_DIR.non_empty() {
        return Ok(PathBuf::from(dir));
    }

    jacket_config::user_cache_dir()
        .map(|root| root.join("jaic"))
        .map_err(|why| format!("cannot find a cache folder for the crash runtime: {why}\n{FIX}"))
}

/// The directory holding the embedded crash runtime, unpacking it if needed. `Ok(None)` when this
/// build embeds none; an error naming the folder when it cannot be written.
pub(crate) fn dir() -> Result<Option<PathBuf>, String> {
    if embedded::FILES.is_empty() {
        return Ok(None);
    }

    unpack_under(&cache_root()?).map(Some)
}

/// Unpacks the runtime into its folder under `root` and returns that folder.
fn unpack_under(root: &Path) -> Result<PathBuf, String> {
    let dir = root.join("crash-runtime").join(embedded::DIGEST);

    unpack(&dir).map_err(|e| {
        format!(
            "cannot unpack the crash runtime into {}: {}\n{FIX}",
            dir.display(),
            jaic::io_reason(&e)
        )
    })?;
    Ok(dir)
}

fn unpack(dir: &Path) -> std::io::Result<()> {
    let complete = embedded::FILES.iter().all(|(name, bytes)| {
        fs::metadata(dir.join(name)).is_ok_and(|m| m.len() == bytes.len() as u64)
    });

    if complete {
        return Ok(());
    }

    fs::create_dir_all(dir)?;

    for (name, bytes) in embedded::FILES {
        // Written beside the target and renamed, so concurrent jaic runs never see a partial file.
        let temporary = dir.join(format!(".{name}.{}", std::process::id()));

        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, dir.join(name))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unwritable_cache_names_the_variable_that_fixes_it() {
        if embedded::FILES.is_empty() {
            return;
        }

        let root = test_support::TempDir::new("unwritable-cache");
        // A file where the cache folder should be cannot hold one.
        let cache = root.join("cache");

        fs::write(&cache, "").unwrap();
        let error = unpack_under(&cache).unwrap_err();

        assert!(
            error.starts_with("cannot unpack the crash runtime into"),
            "{error}"
        );
        assert!(
            error.ends_with("\nnote: set JAIC_CACHE_DIR to a writable folder"),
            "{error}"
        );
        assert!(
            unpack_under(&root)
                .unwrap()
                .join("libportable_crash.a")
                .is_file()
        );
    }
}
