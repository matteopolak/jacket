//! Finding the Microsoft toolchain on a Windows host without a developer prompt.
//!
//! `jaic` prefers Clang's driver, which locates Visual Studio and the Windows SDK itself.
//! Without one on `PATH`, `link.exe` (or `lld-link`) needs the `LIB` variable that a developer
//! prompt sets. Visual Studio and Windows SDK discovery (`vswhere`, the setup COM API, the
//! registry, the `LIB`/`PATH` values) is the `find-msvc-tools` crate's; this module asks it for
//! `link.exe` and `lib.exe` and keeps what it does not cover: a Clang installed outside `PATH`
//! and telling Microsoft's `link.exe` apart from the coreutils `link` that Git for Windows and
//! MSYS2 put on `PATH` (`link FILE1 FILE2`, which stops with `extra operand`).
//! See `docs/native/windows.md`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The CPU folder name of `cpu` in the MSVC and Windows SDK layouts.
pub fn arch_dir(cpu: crate::sema::TargetCpu) -> &'static str {
    match cpu {
        crate::sema::TargetCpu::Arm64 => "arm64",
        _ => "x64",
    }
}

fn lib_is_set() -> bool {
    jacket_config::env::LIB.os().is_some_and(|v| !v.is_empty())
}

fn program_files() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for (var, default) in [
        (jacket_config::env::PROGRAM_FILES, r"C:\Program Files"),
        (
            jacket_config::env::PROGRAM_FILES_X86,
            r"C:\Program Files (x86)",
        ),
    ] {
        let dir = var
            .non_empty()
            .map_or_else(|| PathBuf::from(default), PathBuf::from);
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
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

/// A Clang that is installed but not on `PATH`: the LLVM installer's default folder, then the
/// one Visual Studio's "C++ Clang tools" component adds.
pub fn find_clang() -> Option<PathBuf> {
    let installed = program_files()
        .into_iter()
        .map(|base| base.join(r"LLVM\bin\clang.exe"))
        .find(|p| p.is_file());
    installed.or_else(|| {
        let arch = arch_dir(crate::sema::Options::host().cpu);
        find_msvc_tools::find_tool(arch, "clang.exe").map(|t| t.path().to_path_buf())
    })
}

/// Every `name` (or `name.exe`) on `PATH`, in order.
pub fn on_path(name: &str) -> Vec<PathBuf> {
    let Some(path) = jacket_config::env::PATH.os() else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for dir in std::env::split_paths(&path) {
        for file in [name.to_string(), format!("{name}.exe")] {
            let candidate = dir.join(file);
            if candidate.is_file() {
                found.push(candidate);
                break;
            }
        }
    }
    found
}

/// Whether the output of `link /?` is Microsoft's linker's rather than another program named
/// `link` (coreutils prints `link: missing operand`).
pub(crate) fn is_microsoft_linker_banner(output: &str) -> bool {
    output.contains("Microsoft (R)") || output.contains("Incremental Linker")
}

/// Whether `program` is Microsoft's `link.exe`.
pub fn is_microsoft_linker(program: &Path) -> bool {
    Command::new(program).arg("/?").output().is_ok_and(|out| {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        is_microsoft_linker_banner(&text)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coreutils_link_is_not_the_microsoft_linker() {
        assert!(!is_microsoft_linker_banner(
            "link: missing operand after '/?'\nTry 'link --help' for more information.\n"
        ));
        assert!(is_microsoft_linker_banner(
            "Microsoft (R) Incremental Linker Version 14.44.35215.0\nCopyright (C) Microsoft Corporation.  All rights reserved.\n"
        ));
        assert_eq!(arch_dir(crate::sema::TargetCpu::Arm64), "arm64");
        assert_eq!(arch_dir(crate::sema::TargetCpu::X64), "x64");
    }
}
