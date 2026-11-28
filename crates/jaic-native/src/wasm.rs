//! Linking WebAssembly modules with `wasm-ld` (see `docs/native/wasm-target.md`).
//!
//! A module has 32-bit memory (`wasm32`) or Memory64 (`wasm64`), as the target says. A
//! foreign procedure the program does not define becomes an import (`lower.rs` names its
//! module), so the link needs no `--allow-undefined`; anything else left undefined, such as a
//! compiler-rt helper, is a link error.
use crate::toolchain::{command, lookup};
use jaic::ir::Library;
use jaic_toolchain::Overrides;
use jaic_toolchain::linker::find_linker;
use std::path::{Path, PathBuf};

/// Stack reserved below the data (`--stack-first`): the 8 MiB native programs get. An overflow
/// then runs off the bottom of memory and traps instead of overwriting globals.
const STACK_SIZE: u64 = 8 << 20;

/// How a wasm module is linked.
pub struct WasmLink<'a> {
    pub objects: &'a [PathBuf],
    /// Memory64 (`wasm64`) rather than 32-bit memory.
    pub memory64: bool,
    pub libraries: &'a [Library],
    pub output: &'a Path,
    /// Whether the program defines `_start` (a WASI command, as `Wasi_Runtime` provides).
    /// Without one the module is a library (`--no-entry`) the host calls through its exports.
    pub has_start: bool,
    /// Drop DWARF from the module.
    pub strip_debug: bool,
    /// `additional_linker_arguments`, after jaic's own, so they can override them.
    pub extra_args: &'a [std::ffi::OsString],
}

/// Link wasm objects into a module with `wasm-ld`.
pub fn link_wasm(link: &WasmLink) -> Result<(), String> {
    let target = jaic::target::Target::parse(if link.memory64 {
        "wasm64-unknown-unknown"
    } else {
        "wasm32-unknown-unknown"
    })?;
    let lookup = lookup()?;
    let wasm_ld = find_linker(&lookup, Some(target), &Overrides::from_env())
        .map_err(|missing| missing.to_string())?;

    crate::toolchain::report("Linker", &wasm_ld, &lookup, Some(target));
    let mut cmd = command(&wasm_ld);

    cmd.arg(if link.memory64 {
        "-mwasm64"
    } else {
        "-mwasm32"
    })
    .args(link.objects)
    .arg("-o")
    .arg(link.output);

    if !link.has_start {
        cmd.arg("--no-entry");
    }

    cmd.arg("--stack-first")
        .arg("-z")
        .arg(format!("stack-size={STACK_SIZE}"));

    if link.strip_debug {
        cmd.arg("--strip-debug");
    }

    for lib in link.libraries {
        if let Some(path) = crate::linker::static_library_file(lib) {
            cmd.arg(path);
        }
    }

    cmd.args(link.extra_args);
    let out =
        crate::linker::run_captured(&mut cmd).map_err(|e| crate::linker_not_run(&wasm_ld, &e))?;

    if out.status.success() {
        Ok(())
    } else {
        Err(crate::link_failure(&wasm_ld.program, &out))
    }
}
