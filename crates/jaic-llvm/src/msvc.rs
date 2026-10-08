//! Finding the Microsoft toolchain on a Windows host without a developer prompt.
//!
//! `jaic build` prefers Clang's driver, which locates Visual Studio and the Windows SDK itself.
//! Without one on `PATH`, `link.exe` (or `lld-link`) needs the `LIB` variable that a developer
//! prompt sets. This module finds Visual Studio (through `vswhere`, then the default folders),
//! its `link.exe` and `lib.exe`, a Clang installed outside `PATH`, and the library folders for
//! `LIB`. It also tells Microsoft's `link.exe` apart from the coreutils `link` that Git for
//! Windows and MSYS2 put on `PATH` (`link FILE1 FILE2`, which stops with `extra operand`).
//! See `docs/native/windows.md`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The CPU folder name of `triple` in the MSVC and Windows SDK layouts.
pub(crate) fn arch_dir(triple: &str) -> &'static str {
    if triple.starts_with("aarch64") || triple.starts_with("arm64") {
        "arm64"
    } else {
        "x64"
    }
}

/// The `Host<cpu>` folders whose tools run on this machine, best first: Windows on Arm runs
/// x64 tools under emulation when the native ones are not installed.
fn host_dirs() -> &'static [&'static str] {
    if cfg!(target_arch = "aarch64") {
        &["HostARM64", "Hostx64"]
    } else {
        &["Hostx64", "Hostx86"]
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn program_files() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for (var, default) in [
        ("ProgramFiles", r"C:\Program Files"),
        ("ProgramFiles(x86)", r"C:\Program Files (x86)"),
    ] {
        let dir = env_path(var).unwrap_or_else(|| PathBuf::from(default));
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

fn subdirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect()
}

/// A version folder name (`14.44.35207`, `10.0.26100.0`) as numbers, for ordering.
fn version_key(path: &Path) -> Vec<u64> {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .split('.')
        .map(|part| part.parse().unwrap_or(0))
        .collect()
}

/// The subfolders of `dir`, newest version first.
fn versions_newest_first(dir: &Path) -> Vec<PathBuf> {
    let mut dirs = subdirs(dir);
    dirs.sort_by_key(|d| std::cmp::Reverse(version_key(d)));
    dirs
}

/// Visual Studio installation folders: what `vswhere` (installed with Visual Studio 2017 and
/// later, build tools included) reports, then `<Program Files>\Microsoft Visual Studio\<year>\<edition>`.
fn visual_studio_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for base in program_files() {
        let vswhere = base.join(r"Microsoft Visual Studio\Installer\vswhere.exe");
        if !vswhere.is_file() {
            continue;
        }
        let args = [
            "-all",
            "-prerelease",
            "-products",
            "*",
            "-format",
            "value",
            "-property",
            "installationPath",
        ];
        if let Ok(out) = Command::new(&vswhere).args(args).output() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                let line = line.trim();
                if !line.is_empty() {
                    roots.push(PathBuf::from(line));
                }
            }
        }
    }
    for base in program_files() {
        for year in subdirs(&base.join("Microsoft Visual Studio")) {
            roots.extend(subdirs(&year).into_iter().filter(|e| e.join("VC").is_dir()));
        }
    }
    let mut unique = Vec::new();
    for root in roots {
        if !unique.contains(&root) {
            unique.push(root);
        }
    }
    unique
}

/// MSVC tool sets (`VC\Tools\MSVC\<version>`): the developer prompt's (`VCToolsInstallDir`),
/// then each installation's, newest first.
fn msvc_tool_sets(vs_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut sets: Vec<PathBuf> = env_path("VCToolsInstallDir").into_iter().collect();
    for root in vs_roots {
        sets.extend(versions_newest_first(&root.join(r"VC\Tools\MSVC")));
    }
    sets
}

/// `tool` (`link.exe`, `lib.exe`) of an MSVC tool set for the `arch` target, run on this host.
fn msvc_tool_in(set: &Path, tool: &str, arch: &str) -> Option<PathBuf> {
    host_dirs()
        .iter()
        .map(|host| set.join("bin").join(host).join(arch).join(tool))
        .find(|p| p.is_file())
}

/// The library folders `link.exe` needs in `LIB` to link for `arch`: the tool set's C++ and C
/// runtime libraries, and the newest Windows 10/11 SDK's `ucrt` and `um` (system DLL import
/// libraries).
pub(crate) fn lib_dirs(tool_set: &Path, sdk_roots: &[PathBuf], arch: &str) -> Option<Vec<PathBuf>> {
    let runtime = tool_set.join("lib").join(arch);
    if !runtime.join("msvcrt.lib").is_file() {
        return None;
    }
    for sdk in sdk_roots {
        for version in versions_newest_first(&sdk.join("Lib")) {
            let um = version.join("um").join(arch);
            let ucrt = version.join("ucrt").join(arch);
            if um.join("kernel32.lib").is_file() && ucrt.join("ucrt.lib").is_file() {
                return Some(vec![runtime, ucrt, um]);
            }
        }
    }
    None
}

/// Windows SDK folders (`Windows Kits\10`): the developer prompt's (`WindowsSdkDir`), then the
/// default install location.
fn sdk_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = env_path("WindowsSdkDir").into_iter().collect();
    for base in program_files() {
        roots.push(base.join(r"Windows Kits\10"));
    }
    roots
}

/// A linker found by searching the installation, with the `LIB` value to run it with (`None`:
/// `LIB` is already set, or no libraries were found and the link will say what is missing).
pub(crate) struct Tool {
    pub program: PathBuf,
    pub lib: Option<OsString>,
}

/// `LIB` for `arch` when the environment does not set it: the first tool set that has the
/// runtime libraries, with the SDK's.
pub(crate) fn lib_env(arch: &str) -> Option<OsString> {
    if std::env::var_os("LIB").is_some_and(|v| !v.is_empty()) {
        return None;
    }
    let sdks = sdk_roots();
    msvc_tool_sets(&visual_studio_roots())
        .iter()
        .find_map(|set| lib_dirs(set, &sdks, arch))
        .and_then(|dirs| std::env::join_paths(dirs).ok())
}

/// Microsoft's `tool` (`link.exe`, `lib.exe`) from a Visual Studio installation, for `arch`.
pub(crate) fn find_tool(tool: &str, arch: &str) -> Option<Tool> {
    let sets = msvc_tool_sets(&visual_studio_roots());
    let set = sets
        .iter()
        .find(|set| msvc_tool_in(set, tool, arch).is_some())?;
    let program = msvc_tool_in(set, tool, arch)?;
    let lib = if std::env::var_os("LIB").is_some_and(|v| !v.is_empty()) {
        None
    } else {
        lib_dirs(set, &sdk_roots(), arch).and_then(|dirs| std::env::join_paths(dirs).ok())
    };
    Some(Tool {
        program,
        lib,
    })
}

/// A Clang that is installed but not on `PATH`: the LLVM installer's default folder, then the
/// one Visual Studio's "C++ Clang tools" component adds.
pub(crate) fn find_clang() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = program_files()
        .into_iter()
        .map(|base| base.join(r"LLVM\bin\clang.exe"))
        .collect();
    let host = if cfg!(target_arch = "aarch64") {
        "ARM64"
    } else {
        "x64"
    };
    for root in visual_studio_roots() {
        candidates.push(
            root.join(r"VC\Tools\Llvm")
                .join(host)
                .join(r"bin\clang.exe"),
        );
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Every `name` (or `name.exe`) on `PATH`, in order.
pub(crate) fn on_path(name: &str) -> Vec<PathBuf> {
    let Some(path) = std::env::var_os("PATH") else {
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
pub(crate) fn is_microsoft_linker(program: &Path) -> bool {
    Command::new(program)
        .arg("/?")
        .output()
        .map(|out| {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            is_microsoft_linker_banner(&text)
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn library_folders_come_from_the_newest_complete_sdk() {
        let root = std::env::temp_dir().join(format!("jaic-msvc-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let set = root.join(r"VC/Tools/MSVC/14.44.35207");
        touch(&set.join("lib/x64/msvcrt.lib"));
        let sdk = root.join("Windows Kits/10");
        // The newest SDK lacks the x64 libraries (an arm64-only install): the older one is used.
        touch(&sdk.join("Lib/10.0.26100.0/um/arm64/kernel32.lib"));
        touch(&sdk.join("Lib/10.0.26100.0/ucrt/arm64/ucrt.lib"));
        touch(&sdk.join("Lib/10.0.9200.0/um/x64/kernel32.lib"));
        touch(&sdk.join("Lib/10.0.9200.0/ucrt/x64/ucrt.lib"));
        let dirs = lib_dirs(&set, std::slice::from_ref(&sdk), "x64").unwrap();
        assert_eq!(
            dirs,
            vec![
                set.join("lib/x64"),
                sdk.join("Lib/10.0.9200.0/ucrt/x64"),
                sdk.join("Lib/10.0.9200.0/um/x64"),
            ]
        );
        let arm = lib_dirs(&set, std::slice::from_ref(&sdk), "arm64");
        assert!(arm.is_none(), "the tool set has no arm64 runtime libraries");
        // Versions compare as numbers, not text.
        assert!(version_key(Path::new("10.0.26100.0")) > version_key(Path::new("10.0.9200.0")));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn coreutils_link_is_not_the_microsoft_linker() {
        assert!(!is_microsoft_linker_banner(
            "link: missing operand after '/?'\nTry 'link --help' for more information.\n"
        ));
        assert!(is_microsoft_linker_banner(
            "Microsoft (R) Incremental Linker Version 14.44.35215.0\nCopyright (C) Microsoft Corporation.  All rights reserved.\n"
        ));
        assert_eq!(arch_dir("aarch64-pc-windows-msvc"), "arm64");
        assert_eq!(arch_dir("x86_64-pc-windows-msvc"), "x64");
    }
}
