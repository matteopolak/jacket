//! Linking WebAssembly modules with `wasm-ld` (see `docs/native/wasm-target.md`).
//!
//! jaic targets wasm64 (Memory64) only: Jai's pointers and `s64` counts are 8 bytes. A
//! foreign procedure the program does not define becomes an import (`lower.rs` names its
//! module), so the link needs no `--allow-undefined`; anything else left undefined, such as a
//! compiler-rt helper, is a link error.
use jaic::ir::Library;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The triple `jaic build -os wasm` and an `os_target = .WASM` workspace use by default.
pub const WASM_TRIPLE: &str = "wasm64-unknown-unknown";

/// Whether `triple` names a WebAssembly target.
pub fn is_wasm_target(triple: &str) -> bool {
    triple.starts_with("wasm")
}

/// Stack reserved below the data (`--stack-first`): the 8 MiB native programs get. An overflow
/// then runs off the bottom of memory and traps instead of overwriting globals.
const STACK_SIZE: u64 = 8 << 20;

/// How a wasm module is linked.
pub struct WasmLink<'a> {
    pub objects: &'a [PathBuf],
    pub libraries: &'a [Library],
    pub output: &'a Path,
    /// Whether the program defines `_start` (a WASI command, as `Wasi_Runtime` provides).
    /// Without one the module is a library (`--no-entry`) the host calls through its exports.
    pub has_start: bool,
    /// Drop DWARF from the module.
    pub strip_debug: bool,
    /// `additional_linker_arguments`, after jaic's own, so they can override them.
    pub extra_args: &'a [String],
}

/// Link wasm objects into a module with `wasm-ld`.
pub fn link_wasm(link: &WasmLink) -> Result<(), String> {
    let wasm_ld = find_wasm_ld()?;
    let mut cmd = Command::new(&wasm_ld);
    cmd.arg("-mwasm64")
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
        if let Some(path) = wasm_library_file(lib) {
            cmd.arg(path);
        }
    }
    cmd.args(link.extra_args);
    let program = wasm_ld.display().to_string();
    let out = cmd
        .output()
        .map_err(|e| crate::linker_not_run(&program, &e, WASM_LD_INSTALL))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(crate::link_failure(&program, &out))
    }
}

/// A non-system `#library` with a wasm archive or object next to the declaring file. Other
/// libraries link nothing: their procedures are imports from a module of the library's name.
fn wasm_library_file(lib: &Library) -> Option<PathBuf> {
    if lib.system {
        return None;
    }
    let path = Path::new(&lib.name);
    let file = path.file_name()?.to_string_lossy().into_owned();
    let dir = Path::new(&lib.base_dir).join(path.parent().unwrap_or(Path::new("")));
    [
        format!("{file}.a"),
        format!("lib{file}.a"),
        format!("{file}.o"),
    ]
    .into_iter()
    .map(|name| dir.join(name))
    .find(|p| p.is_file())
}

/// `wasm-ld`: `JAIC_WASM_LD`, else next to the LLVM jaic was built with, else an LLD install
/// (Homebrew's `lld` kegs, Debian's `/usr/lib/llvm-N`), else `PATH`.
pub fn find_wasm_ld() -> Result<PathBuf, String> {
    if let Some(program) = std::env::var_os("JAIC_WASM_LD") {
        return Ok(PathBuf::from(program));
    }
    find_llvm_tool("wasm-ld")
        .map(PathBuf::from)
        .ok_or_else(|| format!("wasm-ld not found: {WASM_LD_INSTALL}"))
}

/// How to get `wasm-ld`.
const WASM_LD_INSTALL: &str =
    "install LLD (Homebrew `lld`, Debian `lld-23`) or set JAIC_WASM_LD to its path";

/// An LLVM tool (`wasm-ld`, `llvm-ar`) from the LLVM or LLD installs jaic knows about.
pub(crate) fn find_llvm_tool(name: &str) -> Option<String> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(prefix) = std::env::var_os("LLVM_SYS_231_PREFIX") {
        dirs.push(Path::new(&prefix).join("bin"));
    }
    if let Some(prefix) = option_env!("LLVM_SYS_231_PREFIX") {
        dirs.push(Path::new(prefix).join("bin"));
    }
    for config in ["llvm-config-23", "llvm-config"] {
        if let Ok(out) = Command::new(config).arg("--bindir").output()
            && out.status.success()
        {
            dirs.push(PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()));
        }
    }
    // LLD is a separate package from LLVM on Homebrew and Debian.
    for keg in ["lld", "lld@23"] {
        for root in ["/opt/homebrew/opt", "/usr/local/opt"] {
            dirs.push(Path::new(root).join(keg).join("bin"));
        }
    }
    dirs.push(PathBuf::from("/usr/lib/llvm-23/bin"));
    if let Some(found) = dirs.iter().map(|d| d.join(name)).find(|p| p.is_file()) {
        return Some(found.to_string_lossy().into_owned());
    }
    let path = std::env::var_os("PATH")?;
    for candidate in [format!("{name}-23"), name.to_string()] {
        for dir in std::env::split_paths(&path) {
            let full = dir.join(&candidate);
            if full.is_file() {
                return Some(full.to_string_lossy().into_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_triples_are_recognized() {
        assert!(is_wasm_target(WASM_TRIPLE));
        assert!(is_wasm_target("wasm64-unknown-wasi"));
        assert!(!is_wasm_target("x86_64-unknown-linux-gnu"));
    }
}
