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
pub mod memory_limit;
pub mod parser;
pub mod records;
pub mod render;
pub mod sema;
pub mod source;
pub mod stack_trace;
pub mod suggest;
pub mod types;
pub mod wide_float;

/// The standard library directory for a native `jaic` or `jailsp`: `JAIC_STDLIB`, else `stdlib/`
/// next to the executable (release archives), else `fallback` (the repository's, for development).
/// "Next to the executable" is checked both where it was started from and, through symlinks, where
/// it really is: an archive unpacked into `/opt` and linked from `~/bin` finds `/opt/.../stdlib`
/// (macOS reports the symlink's path as the executable).
pub fn stdlib_dir(fallback: std::path::PathBuf) -> std::path::PathBuf {
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
        return dir.into();
    }
    let Ok(exe) = std::env::current_exe() else {
        return fallback;
    };
    let real = std::fs::canonicalize(&exe).ok();
    [Some(exe), real]
        .into_iter()
        .flatten()
        .filter_map(|exe| Some(exe.parent()?.join("stdlib")))
        .find(|dir| dir.join("prelude/module.jai").is_file())
        .unwrap_or(fallback)
}

/// An I/O error as a lowercase phrase, without Rust's `(os error N)` suffix.
pub fn io_reason(e: &std::io::Error) -> String {
    let text = e.to_string();
    let text = text.split(" (os error").next().unwrap_or(&text);
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().chain(chars).collect(),
        None => text.to_string(),
    }
}

/// The error for a standard library directory without `prelude/module.jai`, or `None` when it has one.
pub fn missing_stdlib(dir: &std::path::Path) -> Option<String> {
    if dir.join("prelude/module.jai").is_file() {
        return None;
    }
    Some(format!(
        "cannot find the standard library (looked in {}).\n\
         Keep the `stdlib` folder from the release archive next to the executable (a symlink to \
         the executable is fine), or set JAIC_STDLIB to its path.",
        dir.display()
    ))
}
