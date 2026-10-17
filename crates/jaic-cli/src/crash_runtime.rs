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

/// Where unpacked helpers live: `JAIC_CACHE_DIR`, else the platform's user cache.
fn cache_root() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("JAIC_CACHE_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir));
    }
    if let Some(dir) = std::env::var_os("XDG_CACHE_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(dir).join("jaic"));
    }
    let home = PathBuf::from(std::env::var_os("HOME").filter(|d| !d.is_empty())?);
    Some(if cfg!(target_os = "macos") {
        home.join("Library/Caches/jaic")
    } else {
        home.join(".cache/jaic")
    })
}

/// The directory holding the embedded crash runtime, unpacking it if needed. `None` when this
/// build embeds none or no writable cache exists (programs then fail to load it, as before).
pub fn dir() -> Option<PathBuf> {
    if embedded::FILES.is_empty() {
        return None;
    }
    [cache_root(), Some(std::env::temp_dir().join("jaic-cache"))]
        .into_iter()
        .flatten()
        .map(|root| root.join("crash-runtime").join(embedded::DIGEST))
        .find(|dir| unpack(dir).is_ok())
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
