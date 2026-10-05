//! Jai compiler core.
//!
//! Pipeline: `lexer` → `parser` (→ `ast`) → `sema` (demand-driven typing into
//! the checked tree) → `ir` (lowered procedures) → `interp` (compile-time
//! execution, scripting, browser) or the LLVM backend in `jaic-llvm`.
pub mod abi;
pub mod ast;
pub mod build;
pub mod clang;
pub mod fxhash;
pub mod intern;
pub mod interp;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod records;
pub mod sema;
pub mod source;
pub mod stack_trace;
pub mod types;

/// The standard library directory for a native `jaic` or `jai-lsp`: `JAIC_STDLIB`, else `stdlib/`
/// next to the executable (release archives), else `fallback` (the repository's, for development).
pub fn stdlib_dir(fallback: std::path::PathBuf) -> std::path::PathBuf {
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
        return dir.into();
    }
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("stdlib")));
    match beside {
        Some(dir) if dir.join("prelude/module.jai").is_file() => dir,
        _ => fallback,
    }
}
