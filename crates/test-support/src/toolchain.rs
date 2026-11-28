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
//! - [`require`]: a tool every CI host has. Missing on CI (`CI` is set), the test fails;
//!   elsewhere it is skipped.
//! - [`optional`]: a tool only some hosts have (MinGW, `node`, `lld-link`). Always skipped.
//!
//! [`buildable`] asks the same question of a whole target: can `jaic` link for it here, with the
//! linker, sysroot and builtins its lookup finds? The answer and the install notes come from
//! `jaic-toolchain`, the code the compiler itself runs.

use std::path::{Path, PathBuf};
use std::process::Command;

use jaic_toolchain::builtins::find_builtins;
use jaic_toolchain::linker::find_linker;
use jaic_toolchain::package::{Need, PackageManager};
use jaic_toolchain::{Lookup, Overrides, System};

static SYSTEM: System = System;

/// Whether a missing [`require`]d tool fails the test rather than skipping it: when
/// `JACKET_TEST_REQUIRE_TOOLS` is set.
#[must_use]
pub fn strict() -> bool {
    jacket_config::env::TEST_REQUIRE_TOOLS.is_set()
}

/// The program `name` as the toolchain lookup finds it: `PATH` (and, on Windows, `PATHEXT`),
/// then the places tools hide in.
#[must_use]
pub fn find(name: &str) -> Option<PathBuf> {
    let direct = Path::new(name);

    if direct.components().count() > 1 {
        return direct.is_file().then(|| direct.to_path_buf());
    }

    lookup().which(name)
}

fn lookup() -> Lookup<'static> {
    Lookup::new(&SYSTEM).with_llvm_bins(|| jacket_config::llvm::bin_dirs(None))
}

/// Whether `jaic` can link a program for `triple` on this host. When it cannot, the test skips
/// after printing the note that says how to get the missing tool, or on a [`strict`] host
/// fails with it. A freestanding target also needs its compiler builtins (`jacket toolchain
/// add` builds them).
///
/// ```ignore
/// if !toolchain::buildable("aarch64-none-elf") { return; }
/// ```
///
/// # Panics
/// On a [`strict`] host when something is missing, and when `triple` does not parse.
#[must_use]
pub fn buildable(triple: &str) -> bool {
    let target = jaic::target::Target::parse(triple).unwrap_or_else(|e| panic!("{e}"));

    let lookup = lookup();
    let overrides = Overrides::from_env();
    let mut problems = Vec::new();

    if let Err(missing) = find_linker(&lookup, Some(target), &overrides) {
        problems.push(missing);
    }

    if target.is_freestanding()
        && !target.is_wasm()
        && let Err(missing) = find_builtins(&lookup, target, &overrides)
    {
        problems.push(missing);
    }

    if problems.is_empty() {
        return true;
    }

    let text: Vec<String> = problems
        .iter()
        .map(|m| format!("{}\n{}", m.what, m.notes.join("\n")))
        .collect();
    let text = text.join("\n");

    assert!(!strict(), "cannot build for {triple} on this host:\n{text}");
    skip(&format!("cannot build for {triple}:\n{text}"));
    false
}

/// [`buildable`], under the name callers that need the builtins used when other freestanding
/// links did not.
#[must_use]
pub fn buildable_with_builtins(triple: &str) -> bool {
    buildable(triple)
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
    resolve(name, strict())
}

/// Whether the host has every one of `names` (each checked as with [`require`]; every missing
/// one is reported). A test that needs several tools starts with
/// `if !toolchain::require_all(&["clang", "llvm-objdump"]) { return; }`.
#[must_use]
pub fn require_all(names: &[&str]) -> bool {
    names.iter().filter(|name| require(name).is_none()).count() == 0
}

/// The tool `name`, or `None` (after a note) when this host lacks it. Never fails the test.
#[must_use]
pub fn optional(name: &str) -> Option<PathBuf> {
    resolve(name, false)
}

fn resolve(name: &str, must_exist: bool) -> Option<PathBuf> {
    let found = find(name);

    if found.is_none() {
        let lookup = lookup();
        let note = Need::for_program(name).note(lookup.host.os(), PackageManager::detect(&lookup));

        assert!(
            !must_exist,
            "required test tool `{name}` is not installed\nnote: {note}"
        );
        skip(&format!("`{name}` is not installed"));
    }

    found
}

/// The `jaic` that sits beside the `jacket` executable `jacket`, as in an unpacked archive. `None`
/// (after a note) when it has not been built: `cargo test -p jacket` alone does not build it.
#[must_use]
pub fn jaic_beside(jacket: &Path) -> Option<PathBuf> {
    let jaic = jacket
        .parent()?
        .join(format!("jaic{}", std::env::consts::EXE_SUFFIX));

    if !jaic.is_file() {
        skip("build `jaic` first (`cargo build -p jaic-cli`)");
        return None;
    }

    Some(jaic)
}

/// The `jaic` built beside `jacket`, for a test that cannot do without it. Panics, naming the
/// command that builds it, when it is missing: `cargo test -p jacket` builds only `jacket`, so
/// `jaic` has to be built into the same target directory first.
#[must_use]
pub fn paired_jaic(jacket: &Path) -> PathBuf {
    let jaic = jacket
        .parent()
        .expect("jacket has a folder")
        .join(format!("jaic{}", std::env::consts::EXE_SUFFIX));

    assert!(
        jaic.is_file(),
        "{} is missing: this test runs the `jaic` built beside `jacket`; build it into the same target directory with `cargo build -p jaic-cli` (same profile, same CARGO_TARGET_DIR)",
        jaic.display()
    );
    jaic
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
    fn a_path_is_used_as_given() {
        let dir = crate::tmp::TempDir::new("toolchain-direct");
        let file = dir.write("tool", "");

        assert_eq!(find(file.to_str().unwrap()), Some(file));
    }

    #[test]
    fn jaic_is_found_beside_jacket_and_a_missing_one_skips() {
        let dir = crate::tmp::TempDir::new("toolchain-jaic");
        let suffix = std::env::consts::EXE_SUFFIX;
        let jacket = dir.join(format!("jacket{suffix}"));

        assert_eq!(jaic_beside(&jacket), None);
        let jaic = dir.write(&format!("jaic{suffix}"), "");

        assert_eq!(jaic_beside(&jacket), Some(jaic));
    }

    #[test]
    fn node_versions_parse() {
        assert_eq!(parse_node_major("v24.12.0\n"), Some(24));
        assert_eq!(parse_node_major("20.1.0"), Some(20));
        assert_eq!(parse_node_major("nonsense"), None);
    }

    #[test]
    fn an_absent_optional_tool_is_skipped() {
        assert_eq!(optional("definitely-not-a-real-tool-name"), None);
    }

    #[test]
    fn a_target_is_buildable_when_the_host_can_link_it() {
        // The host's own target always links where the tests run.
        assert!(buildable(&jaic::target::Target::host().to_string()));
    }
}
