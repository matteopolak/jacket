//! Finding the Microsoft toolchain on a Windows host without a developer prompt.
//!
//! `jaic` prefers Clang's driver, which locates Visual Studio and the Windows SDK itself.
//! Without one on `PATH`, `link.exe` (or `lld-link`) needs the `LIB` variable that a developer
//! prompt sets. Visual Studio and Windows SDK discovery (`vswhere`, the setup COM API, the
//! registry, the `LIB`/`PATH` values) is the `find-msvc-tools` crate's; this module asks it for
//! `link.exe`, `lib.exe` and `LIB`. It lives here because the interpreter opens import libraries
//! from `LIB`; the rest of linker discovery is `jaic-toolchain`'s (`msvc.rs`).
//! See `docs/native/windows.md`.

use std::ffi::OsString;
use std::path::PathBuf;

/// The CPU folder name of `cpu` in the MSVC and Windows SDK layouts.
pub fn arch_dir(cpu: crate::target::TargetCpu) -> &'static str {
    match cpu {
        crate::target::TargetCpu::Arm64 => "arm64",
        _ => "x64",
    }
}

fn lib_is_set() -> bool {
    jacket_config::env::LIB.os().is_some_and(|v| !v.is_empty())
}

/// A Microsoft tool found by searching the installation, with the environment to run it with
/// (`LIB`, `PATH`, `INCLUDE`; empty when a developer prompt already set them).
pub struct Tool {
    pub program: PathBuf,
    pub env: Vec<(OsString, OsString)>,
}

/// `LIB` for `arch` when the environment does not set it.
pub fn lib_env(arch: &str) -> Option<OsString> {
    if lib_is_set() {
        return None;
    }

    find_tool("link.exe", arch)?
        .env
        .into_iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("LIB"))
        .map(|(_, value)| value)
}

/// Library directories selected by the same LIB/SDK discovery as native linking.
#[cfg(windows)]
pub(crate) fn library_dirs(arch: &str) -> Vec<PathBuf> {
    let value = jacket_config::env::LIB
        .os()
        .filter(|v| !v.is_empty())
        .or_else(|| lib_env(arch));

    value
        .map(|v| std::env::split_paths(&v).collect())
        .unwrap_or_default()
}

/// Microsoft's `tool` (`link.exe`, `lib.exe`) from a Visual Studio installation, for `arch`.
pub fn find_tool(tool: &str, arch: &str) -> Option<Tool> {
    let found = find_msvc_tools::find_tool(arch, tool)?;
    let env = if lib_is_set() {
        Vec::new()
    } else {
        found.env().into_iter().cloned().collect()
    };

    Some(Tool {
        program: found.path().to_path_buf(),
        env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_folders_are_named_as_msvc_names_them() {
        assert_eq!(arch_dir(crate::target::TargetCpu::Arm64), "arm64");
        assert_eq!(arch_dir(crate::target::TargetCpu::X64), "x64");
    }
}
