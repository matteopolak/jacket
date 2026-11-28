//! Where `jacket toolchain add` keeps what it fetches. Builds only read from here.
use std::path::{Path, PathBuf};

use jaic::target::Target;

use crate::host::Host;

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

/// The folder the builtins `jacket toolchain add` built for `target` are in.
pub fn builtins_dir(host: &dyn Host, target: Target) -> Option<PathBuf> {
    entry_dir(host, target).map(|dir| dir.join("builtins"))
}

/// A file beside the archive whose name says what the builtins were built from and which source
/// list built them: `source-compiler-rt-21.1.8-0f3a9c21d4e85b67`.
pub const SOURCE_MARK: &str = "source-";

/// The file name of an archive of builtins.
pub const BUILTINS_FILE: &str = "libclang_rt.builtins.a";

/// The builtins built for `target`: the archive and what it was built from (`compiler-rt
/// 21.1.8`), when both are there.
pub fn builtins(host: &dyn Host, target: Target) -> Option<(PathBuf, String)> {
    let dir = builtins_dir(host, target)?;
    let file = dir.join(BUILTINS_FILE);

    if !host.is_file(&file) {
        return None;
    }

    let (source, _) = built_from(host, &dir)?;
    Some((file, source))
}

/// The pin stamp the cached builtins of `target` were built from (`CompilerRt::stamp`), so a
/// later jacket with a different source list knows to rebuild them.
pub fn builtins_stamp(host: &dyn Host, target: Target) -> Option<String> {
    built_from(host, &builtins_dir(host, target)?).map(|(_, stamp)| stamp)
}

/// What the marker file of the builtins folder `dir` says: the source (`compiler-rt 21.1.8`) and
/// the stamp.
fn built_from(host: &dyn Host, dir: &Path) -> Option<(String, String)> {
    host.read_dir(dir).into_iter().find_map(|path| {
        let name = path.file_name()?.to_str()?.strip_prefix(SOURCE_MARK)?;
        let (rest, stamp) = name.rsplit_once('-')?;
        let (what, version) = rest.rsplit_once('-')?;

        Some((format!("{what} {version}"), stamp.to_string()))
    })
}

/// What the cache holds for one target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The target's folder name in the cache.
    pub target: String,
    /// Whether a sysroot is cached for it.
    pub sysroot: bool,
    /// What the builtins were built from, when there are some.
    pub builtins: Option<String>,
}

/// Every target the cache has something for, in name order.
pub fn entries(host: &dyn Host) -> Vec<Entry> {
    let Some(root) = root(host) else {
        return Vec::new();
    };
    let mut found = Vec::new();

    for dir in host.read_dir(&root) {
        let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Ok(target) = Target::parse(name) else {
            continue;
        };
        let entry = Entry {
            target: name.to_string(),
            sysroot: sysroot(host, target).is_some(),
            builtins: builtins(host, target).map(|(_, source)| source),
        };

        if entry.sysroot || entry.builtins.is_some() {
            found.push(entry);
        }
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeHost;

    #[test]
    fn the_cache_lists_what_was_fetched_per_target() {
        let mut host = FakeHost::mac();
        let base = root(&host).unwrap();
        let armv7 = base.join("armv7-unknown-linux-gnueabihf");

        host.dir(armv7.join("sysroot/usr").to_str().unwrap());
        let cortex = base.join("thumbv7em-none-eabihf/builtins");

        host.file(cortex.join(BUILTINS_FILE).to_str().unwrap());
        host.file(
            cortex
                .join("source-compiler-rt-21.1.8-0123456789abcdef")
                .to_str()
                .unwrap(),
        );
        // A folder that is not a target is left alone.
        host.dir(base.join("downloads").to_str().unwrap());
        assert_eq!(
            entries(&host),
            [
                Entry {
                    target: "armv7-unknown-linux-gnueabihf".into(),
                    sysroot: true,
                    builtins: None,
                },
                Entry {
                    target: "thumbv7em-none-eabihf".into(),
                    sysroot: false,
                    builtins: Some("compiler-rt 21.1.8".into()),
                },
            ]
        );
    }
}
