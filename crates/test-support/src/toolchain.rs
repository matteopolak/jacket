//! Finding the external tools tests drive (C compiler, `wasm-ld`, `node`, MinGW), and skipping
//! a test when one is missing.
//!
//! Rust has no runtime "skipped" result, so a test that lacks a tool prints a note and returns:
//!
//! ```ignore
//! let Some(clang) = toolchain::require("clang") else { return };
//! ```
//!
//! Two strengths decide what a missing tool does:
//!
//! - [`require`]: a tool every CI host has. Missing on CI (`CI` is set) or with
//!   `JAIC_REQUIRE_TOOLCHAIN=1`, the test fails; elsewhere it is skipped.
//! - [`optional`]: a tool only some hosts have (MinGW, `node`, `lld-link`). Always skipped,
//!   unless `JAIC_REQUIRE_TOOLCHAIN` names it (a CI job that installs it sets that).
//!
//! `JAIC_REQUIRE_TOOLCHAIN` is `1` (every tool is required) or a comma-separated list of tool
//! names (`node,wasm-ld`).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Whether `JAIC_REQUIRE_TOOLCHAIN` (`1`, or a list of tool names) requires `name`.
#[must_use]
pub fn explicitly_required(name: &str) -> bool {
    requires(jacket_config::env::REQUIRE_TOOLCHAIN.get().as_deref(), name)
}

fn requires(setting: Option<&str>, name: &str) -> bool {
    setting.is_some_and(|list| list == "1" || list.split(',').any(|item| item.trim() == name))
}

/// Whether a missing [`require`]d tool fails the test rather than skipping it.
#[must_use]
pub fn strict(name: &str) -> bool {
    explicitly_required(name) || jacket_config::env::CI.os().is_some()
}

/// The program `name` as `PATH` (and, on Windows, `PATHEXT`) would resolve it.
#[must_use]
pub fn find(name: &str) -> Option<PathBuf> {
    let path = jacket_config::env::PATH.os()?;
    find_in(
        name,
        std::env::split_paths(&path),
        jacket_config::env::PATHEXT.os().as_deref(),
    )
}

/// [`find`] over explicit search directories and Windows executable extensions.
fn find_in<I>(name: &str, dirs: I, pathext: Option<&OsStr>) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    let direct = Path::new(name);
    if direct.components().count() > 1 {
        return direct.is_file().then(|| direct.to_path_buf());
    }
    let extensions: Vec<String> = if cfg!(windows) {
        let list = pathext.map_or_else(|| ".EXE".to_string(), |v| v.to_string_lossy().into_owned());
        std::iter::once(String::new())
            .chain(
                list.split(';')
                    .filter(|e| !e.is_empty())
                    .map(str::to_string),
            )
            .collect()
    } else {
        vec![String::new()]
    };
    dirs.into_iter().find_map(|dir| {
        extensions.iter().find_map(|extension| {
            let candidate = dir.join(format!("{name}{extension}"));
            candidate.is_file().then_some(candidate)
        })
    })
}

/// A command that starts the tool `name` as [`find`] resolves it (or by its bare name, for the
/// OS to resolve, when it is not on `PATH`). Check availability first with [`require`] or
/// [`require_all`].
#[must_use]
pub fn command(name: &str) -> Command {
    Command::new(find(name).unwrap_or_else(|| PathBuf::from(name)))
}

/// Records that the current test is skipped, and why.
pub fn skip(reason: &str) {
    eprintln!("skipped: {reason}");
}

/// The tool `name`, or `None` (after a note) when this host lacks it and the test may skip.
/// Panics instead on a [`strict`] host.
#[must_use]
pub fn require(name: &str) -> Option<PathBuf> {
    resolve(name, strict(name))
}

/// Whether the host has every one of `names` (each checked as with [`require`]; every missing
/// one is reported). A test that needs several tools starts with
/// `if !toolchain::require_all(&["clang", "llvm-objdump"]) { return; }`.
#[must_use]
pub fn require_all(names: &[&str]) -> bool {
    names.iter().filter(|name| require(name).is_none()).count() == 0
}

/// The tool `name`, or `None` (after a note) when this host lacks it. Panics only with
/// `JAIC_REQUIRE_TOOLCHAIN=1`.
#[must_use]
pub fn optional(name: &str) -> Option<PathBuf> {
    resolve(name, explicitly_required(name))
}

fn resolve(name: &str, must_exist: bool) -> Option<PathBuf> {
    let found = find(name);
    if found.is_none() {
        assert!(!must_exist, "required test tool `{name}` is not installed");
        skip(&format!("`{name}` is not installed"));
    }
    found
}

/// The C compiler for host builds: `CC`, else `clang` on Windows (MSVC hosts have no `cc`)
/// and `cc` elsewhere.
#[must_use]
pub fn host_cc() -> Option<PathBuf> {
    if let Some(cc) = jacket_config::env::CC.os().filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(cc));
    }
    require(if cfg!(windows) {
        "clang"
    } else {
        "cc"
    })
}

/// `clang`, which tests that need its flags (`-dynamiclib`, `--target`) ask for by name.
#[must_use]
pub fn clang() -> Option<PathBuf> {
    require("clang")
}

/// Builds the C `source` into the shared library `dir/<prefix><name><suffix>` for this host
/// (`libname.dylib`, `libname.so`, `name.dll`) with clang, warnings as errors, and returns its
/// path. `None` (after a note) when clang is missing and the test may skip.
///
/// # Panics
/// When the compilation fails.
#[must_use]
pub fn build_shared_library(source: &Path, dir: &Path, name: &str) -> Option<PathBuf> {
    let clang = clang()?;
    let library = dir.join(format!(
        "{}{name}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let mut command = Command::new(clang);
    command.args(["-std=c11", "-Wall", "-Wextra", "-Werror"]);
    command.arg(if cfg!(target_os = "macos") {
        "-dynamiclib"
    } else {
        "-shared"
    });
    if !cfg!(windows) {
        command.arg("-fPIC");
    }
    crate::process::run(command.arg(source).arg("-o").arg(&library));
    Some(library)
}

/// `wasm-ld`, the linker for WebAssembly objects.
#[must_use]
pub fn wasm_ld() -> Option<PathBuf> {
    require("wasm-ld")
}

/// The MinGW-w64 GCC that links Windows executables from other hosts.
#[must_use]
pub fn mingw_gcc() -> Option<PathBuf> {
    optional("x86_64-w64-mingw32-gcc")
}

/// `node`'s major version, when it runs.
#[must_use]
pub fn node_major() -> Option<u32> {
    let output = Command::new("node").arg("--version").output().ok()?;
    parse_node_major(&String::from_utf8_lossy(&output.stdout))
}

fn parse_node_major(version: &str) -> Option<u32> {
    version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// `node` with the flags WebAssembly modules from jaic need (Memory64 is on by default from
/// node 24), as a command to add a script and arguments to. `None` after a note when node is
/// missing; see [`optional`].
#[must_use]
pub fn node() -> Option<Command> {
    optional("node")?;
    let major = node_major()?;
    let mut command = Command::new("node");
    command.arg("--no-warnings");
    if major < 24 {
        command.arg("--experimental-wasm-memory64");
    }
    Some(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_program_in_a_search_directory() {
        let dir = crate::tmp::TempDir::new("toolchain");
        let name = if cfg!(windows) {
            "fake-tool.exe"
        } else {
            "fake-tool"
        };
        dir.write(name, "");
        let found = find_in(
            "fake-tool",
            [dir.path().to_path_buf()],
            Some(OsStr::new(".EXE")),
        );
        assert_eq!(found, Some(dir.join(name)));
        assert_eq!(
            find_in("absent-tool", [dir.path().to_path_buf()], None),
            None
        );
    }

    #[test]
    fn a_path_is_used_as_given() {
        let dir = crate::tmp::TempDir::new("toolchain-direct");
        let file = dir.write("tool", "");
        assert_eq!(find_in(file.to_str().unwrap(), [], None), Some(file));
    }

    #[test]
    fn node_versions_parse() {
        assert_eq!(parse_node_major("v24.12.0\n"), Some(24));
        assert_eq!(parse_node_major("20.1.0"), Some(20));
        assert_eq!(parse_node_major("nonsense"), None);
    }

    #[test]
    fn the_requirement_setting_names_tools() {
        assert!(requires(Some("1"), "node"));
        assert!(requires(Some("node, wasm-ld"), "wasm-ld"));
        assert!(!requires(Some("node"), "clang"));
        assert!(!requires(None, "node"));
    }

    #[test]
    fn an_absent_optional_tool_is_skipped() {
        if !explicitly_required("definitely-not-a-real-tool-name") {
            assert_eq!(optional("definitely-not-a-real-tool-name"), None);
        }
    }
}
