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
pub mod lexer;
pub mod memory_limit;
#[doc(hidden)]
pub mod msvc;
pub mod parser;
pub mod record_build;
pub mod records;
pub mod render;
pub mod sema;
pub mod source;
pub mod stack_trace;
pub mod suggest;
pub mod target;
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
        .find(|dir| find_preload(&sema::NativeFs, dir).is_some())
        .unwrap_or(fallback)
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

/// `std::fs::canonicalize` without Windows' verbatim prefix: `\\?\C:\dir` becomes `C:\dir` and
/// `\\?\UNC\server\share` becomes `\\server\share`. A verbatim working directory or `#file` breaks
/// programs: `/tmp` relative to `\\?\C:\dir` is no folder on drive C, and the prefix shows up in
/// every path the program prints. Elsewhere this is `std::fs::canonicalize`.
pub fn canonicalize(path: impl AsRef<std::path::Path>) -> std::io::Result<std::path::PathBuf> {
    let real = std::fs::canonicalize(path)?;
    Ok(strip_verbatim(&real.to_string_lossy()).map_or(real, Into::into))
}

/// The plain form of a verbatim Windows path (see [`canonicalize`]), or `None` when `path` is not
/// verbatim or has no plain form (a component that Win32 path parsing would change).
pub fn strip_verbatim(path: &str) -> Option<String> {
    let rest = path.strip_prefix(r"\\?\")?;
    let plain = if let Some(share) = rest.strip_prefix(r"UNC\") {
        format!(r"\\{share}")
    } else {
        let drive = rest.as_bytes();
        if drive.len() < 2 || !drive[0].is_ascii_alphabetic() || drive[1] != b':' {
            return None;
        }
        rest.to_string()
    };
    // Win32 parsing trims trailing dots and spaces and treats `/` as a separator; such names only
    // mean the same thing in their verbatim form, as does anything longer than MAX_PATH.
    let odd = plain.contains('/')
        || plain.len() >= 260
        || plain
            .split('\\')
            .any(|c| c.ends_with('.') || c.ends_with(' '));
    (!odd).then_some(plain)
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
    use super::strip_verbatim;

    #[test]
    fn verbatim_paths_lose_their_prefix_when_win32_reads_them_the_same() {
        assert_eq!(
            strip_verbatim(r"\\?\D:\a\jai\tests").as_deref(),
            Some(r"D:\a\jai\tests")
        );
        assert_eq!(
            strip_verbatim(r"\\?\UNC\host\share\x").as_deref(),
            Some(r"\\host\share\x")
        );
        assert_eq!(strip_verbatim(r"C:\plain"), None);
        assert_eq!(strip_verbatim("/usr/lib"), None);
        assert_eq!(strip_verbatim(r"\\?\Volume{1234}\x"), None);
        assert_eq!(strip_verbatim(r"\\?\C:\trailing dot."), None);
        assert_eq!(
            strip_verbatim(&format!(r"\\?\C:\{}", "x".repeat(300))),
            None
        );
    }
}
