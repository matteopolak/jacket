//! Where `jacket toolchain add` keeps what it fetches. Builds only read from here.
use crate::host::Host;
use jaic::target::Target;
use std::path::PathBuf;

/// The folder holding one folder per target: `~/Library/Caches/jacket/toolchains`,
/// `~/.cache/jacket/toolchains`, `%LOCALAPPDATA%\jacket\toolchains`.
pub fn root(host: &dyn Host) -> Option<PathBuf> {
    host.cache_root()
        .map(|cache| cache.join("jacket").join("toolchains"))
}

/// Everything fetched for `target`.
pub fn entry_dir(host: &dyn Host, target: Target) -> Option<PathBuf> {
    root(host).map(|root| root.join(target.to_string()))
}

/// The Linux sysroot fetched for `target`, when it is there.
pub fn sysroot(host: &dyn Host, target: Target) -> Option<PathBuf> {
    let dir = entry_dir(host, target)?.join("sysroot");
    host.is_dir(&dir.join("usr")).then_some(dir)
}
