//! Finding the other tools of the toolchain, which ship in one folder: next to the running
//! executable first, then on `PATH`.
use std::path::{Path, PathBuf};

/// The file name of `tool` on this platform.
#[must_use]
pub fn file_name(tool: &str) -> String {
    format!("{tool}{}", std::env::consts::EXE_SUFFIX)
}

/// The file to start for `tool`: next to this executable (through symlinks, so
/// `~/.local/bin/jacket` finds the installed release), else the first match on `PATH`.
#[must_use]
pub fn locate(tool: &str) -> Option<PathBuf> {
    let name = file_name(tool);
    if let Ok(me) = std::env::current_exe().and_then(|me| me.canonicalize())
        && let Some(dir) = me.parent()
    {
        let beside = dir.join(&name);
        if beside.is_file() {
            return Some(beside);
        }
    }
    on_path(Path::new(&name))
}

/// The file `program` names in a folder of `PATH`.
#[must_use]
pub fn on_path(program: &Path) -> Option<PathBuf> {
    let path = crate::env::PATH.os()?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}
