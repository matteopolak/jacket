//! Jai compiler core.
//!
//! Pipeline: `lexer` → `parser` (→ `ast`) → `sema` (demand-driven typing into
//! the checked tree) → `ir` (lowered procedures) → `interp` (compile-time
//! execution, scripting, browser) or the LLVM backend in `jaic-llvm`.
pub mod abi;
pub mod ast;
pub mod build;
pub mod builtin_keys;
pub mod clang;
pub mod docs;
pub mod fiber;
pub mod fxhash;
pub mod intern;
pub mod interp;
pub mod ir;
pub mod knobs;
pub mod lexer;
mod macros;
pub mod memory_limit;
#[doc(hidden)]
pub mod msvc;
pub mod parse_ahead;
pub mod parser;
pub mod paths;
pub mod project;
pub mod record_build;
pub mod records;
pub mod render;
pub mod sema;
pub mod source;
pub mod stack_trace;
pub mod suggest;
pub mod target;
pub use paths::{canonicalize, default_stdlib_dir, stdlib_dir, strip_verbatim};
pub mod types;
pub mod wide_float;

/// The folder of the standard library that holds the modules only jaic has (`Long_Double`,
/// `Jai_Format`, `WebGPU`, `Wasi_Runtime`, ...). It is not searched on its own: a program imports them by
/// path, as `#import "Extensions/Long_Double";`, so the import itself says it is jaic-only.
pub const STDLIB_EXTENSIONS_DIR: &str = "Extensions";

/// The sole bootstrap entry, relative to the standard-library root.
pub const PRELOAD_ENTRY: &str = "prelude/module.jai";

/// Find a bootstrap entry through the same filesystem used to compile the program.
/// Native tools and the browser therefore accept the same standard-library layouts.
pub fn find_preload(
    fs: &dyn sema::FileSystem,
    stdlib: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let entry = stdlib.join(PRELOAD_ENTRY);
    fs.is_file(&entry).then_some(entry)
}

/// The nearest folder at or above `path` that is itself a standard library (it holds the
/// bootstrap entry). A file inside one is library code and resolves against that library,
/// whichever library the tool would otherwise use.
pub fn enclosing_stdlib(
    fs: &dyn sema::FileSystem,
    path: &std::path::Path,
) -> Option<std::path::PathBuf> {
    path.ancestors()
        .skip(1)
        .find(|dir| find_preload(fs, dir).is_some())
        .map(std::path::Path::to_path_buf)
}

/// Recover the library root from a selected bootstrap entry. Explicit custom preload
/// files retain the established convention that their parent is the library root.
pub fn stdlib_root_from_preload(preload: &std::path::Path) -> Option<&std::path::Path> {
    let parent = preload.parent()?;
    if preload.ends_with(PRELOAD_ENTRY) {
        parent.parent()
    } else {
        Some(parent)
    }
}

/// The directory the user started the tool in, for showing paths in messages (jaic itself
/// works from the main file's directory). Unset, paths are shown as they are.
static DISPLAY_BASE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// The directory paths in messages are shown relative to, when one was set.
pub fn display_base() -> Option<&'static std::path::Path> {
    DISPLAY_BASE.get().map(|p| p.as_path())
}

/// Set the directory paths in messages are shown relative to (once per process).
pub fn set_display_base(dir: std::path::PathBuf) {
    let _ = DISPLAY_BASE.set(dir);
}

/// `path` as the user would type it: relative to the display base when inside it or up to two
/// directories above it.
pub fn display_path(path: &std::path::Path) -> String {
    let Some(base) = DISPLAY_BASE.get() else {
        return path.display().to_string();
    };
    if path == base {
        return ".".to_string();
    }
    for (up, dir) in base.ancestors().take(3).enumerate() {
        if let Ok(relative) = path.strip_prefix(dir)
            && !relative.as_os_str().is_empty()
        {
            let mut shown = std::path::PathBuf::new();
            for _ in 0..up {
                shown.push("..");
            }
            return shown.join(relative).display().to_string();
        }
    }
    path.display().to_string()
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

/// The error for a standard library directory without a bootstrap entry.
pub fn missing_stdlib(dir: &std::path::Path) -> Option<String> {
    if find_preload(&sema::NativeFs, dir).is_some() {
        return None;
    }
    Some(format!(
        "cannot find the standard library (looked for bootstrap {}).\n\
         Keep the `stdlib` folder from the release archive next to the executable (a symlink to \
         the executable is fine), or set JAIC_STDLIB to its path.",
        dir.join(PRELOAD_ENTRY).display()
    ))
}

#[cfg(test)]
mod tests {
    use super::enclosing_stdlib;

    #[test]
    fn a_file_inside_a_stdlib_finds_that_stdlib() {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let file = stdlib.join("internal/allocation/unmapping.jai");
        let found = enclosing_stdlib(&crate::sema::NativeFs, &file).unwrap();
        assert_eq!(found, stdlib);
        let outside = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
        assert_eq!(enclosing_stdlib(&crate::sema::NativeFs, &outside), None);
    }
}
