//! Jai compiler core.
//!
//! Pipeline: `lexer` → `parser` (→ `ast`) → `sema` (demand-driven typing into
//! the checked tree) → `ir` (lowered procedures) → `interp` (compile-time
//! execution, scripting, browser) or the LLVM backend in `jaic-llvm`.

// What the compiler prints to stderr takes the build status line down first (`progress`).
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        $crate::progress::before_output();
        ::std::eprintln!($($arg)*)
    }};
}

pub mod abi;
pub mod ast;
pub mod build;
pub mod builtin_keys;
pub mod docs;
pub mod fiber;
pub mod format_string;
pub mod fxhash;
pub mod ide;
pub mod intern;
pub mod interp;
pub mod interpolate;
pub mod ir;
pub mod knobs;
pub mod lexer;
pub mod lexicon;
mod macros;
pub mod memory_limit;
#[doc(hidden)]
pub mod msvc;
pub mod parse_ahead;
pub mod parser;
pub mod paths;
pub mod pool;
pub mod progress;
pub mod project;
pub mod record_build;
pub mod records;
pub mod render;
pub mod sema;
pub mod service;
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

/// Ordered prelude candidates, relative to the standard-library root.
/// Add new layouts here and cover their order and root recovery in the tests below.
/// `Preload.jai` is a single file at the root, for libraries laid out that way.
pub const PRELUDE_ENTRIES: &[&str] = &["prelude/module.jai", "Preload.jai"];

fn prelude_override() -> Option<std::path::PathBuf> {
    std::env::var_os("JAIC_PRELUDE")
        .filter(|value| !value.is_empty())
        .map(Into::into)
}

fn prelude_paths(
    stdlib: &std::path::Path,
    entries: &[&str],
    explicit: Option<&std::path::Path>,
) -> Vec<std::path::PathBuf> {
    if let Some(path) = explicit {
        vec![stdlib.join(path)]
    } else {
        entries.iter().map(|entry| stdlib.join(entry)).collect()
    }
}

fn find_preload_with(
    fs: &dyn sema::FileSystem,
    stdlib: &std::path::Path,
    entries: &[&str],
    explicit: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    prelude_paths(stdlib, entries, explicit)
        .into_iter()
        .find(|path| fs.is_file(path))
}

/// Resolve the prelude through the compiler's filesystem, including browser files.
/// An explicit setting wins even when it names a missing file.
pub fn find_preload(
    fs: &dyn sema::FileSystem,
    stdlib: &std::path::Path,
) -> Option<std::path::PathBuf> {
    find_preload_with(fs, stdlib, PRELUDE_ENTRIES, prelude_override().as_deref())
}

/// Select the prelude, retaining the requested path when missing for diagnostics.
pub fn preload_path(fs: &dyn sema::FileSystem, stdlib: &std::path::Path) -> std::path::PathBuf {
    let explicit = prelude_override();
    find_preload_with(fs, stdlib, PRELUDE_ENTRIES, explicit.as_deref())
        .unwrap_or_else(|| prelude_paths(stdlib, PRELUDE_ENTRIES, explicit.as_deref()).remove(0))
}

fn enclosing_stdlib_with(
    fs: &dyn sema::FileSystem,
    path: &std::path::Path,
    entries: &[&str],
    explicit: Option<&std::path::Path>,
    configured: &std::path::Path,
) -> Option<std::path::PathBuf> {
    // An explicit file does not redefine the configured module root.
    if explicit.is_some() {
        return (fs.canonical(path).starts_with(fs.canonical(configured))
            && find_preload_with(fs, configured, entries, explicit).is_some())
        .then(|| configured.to_path_buf());
    }
    path.ancestors()
        .skip(1)
        .find(|dir| find_preload_with(fs, dir, entries, explicit).is_some())
        .map(std::path::Path::to_path_buf)
}

/// The nearest enclosing library, using the same prelude selection as compilation.
pub fn enclosing_stdlib(
    fs: &dyn sema::FileSystem,
    path: &std::path::Path,
) -> Option<std::path::PathBuf> {
    enclosing_stdlib_with(
        fs,
        path,
        PRELUDE_ENTRIES,
        prelude_override().as_deref(),
        &default_stdlib_dir(),
    )
}

fn stdlib_root_with<'a>(
    preload: &'a std::path::Path,
    entries: &[&str],
) -> Option<&'a std::path::Path> {
    for entry in entries {
        if preload.ends_with(entry) {
            return preload
                .ancestors()
                .nth(std::path::Path::new(entry).components().count());
        }
    }
    preload.parent()
}

/// Recover the root of a detected prelude. Explicit selections keep their configured
/// library root in `Options`, since an external file cannot identify that root.
pub fn stdlib_root_from_preload(preload: &std::path::Path) -> Option<&std::path::Path> {
    stdlib_root_with(preload, PRELUDE_ENTRIES)
}

/// The directory the user started the tool in, for showing paths in messages (jaic itself
/// works from the main file's directory). Unset, paths are shown as they are.
static DISPLAY_BASE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// The directory paths in messages are shown relative to, when one was set.
pub fn display_base() -> Option<&'static std::path::Path> {
    DISPLAY_BASE.get().map(std::path::PathBuf::as_path)
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
    Some(missing_stdlib_message(
        dir,
        PRELUDE_ENTRIES,
        prelude_override().as_deref(),
    ))
}

fn missing_stdlib_message(
    dir: &std::path::Path,
    entries: &[&str],
    explicit: Option<&std::path::Path>,
) -> String {
    let mut paths = prelude_paths(dir, entries, None);
    if let Some(explicit) = explicit {
        paths.insert(0, dir.join(explicit));
    }
    format!(
        "cannot find the standard library (looked for prelude {}).\n\
         Keep the `stdlib` folder from the release archive next to the executable (a symlink to \
         the executable is fine), or set JAIC_STDLIB to its path. \
         JAIC_PRELUDE selects a file instead of automatic detection.",
        paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::{PRELUDE_ENTRIES, enclosing_stdlib};

    use super::{
        enclosing_stdlib_with, find_preload_with, missing_stdlib_message, stdlib_root_with,
    };
    use crate::sema::VirtualFs;
    use std::path::Path;

    #[test]
    fn candidate_order_and_both_layouts_share_root_detection() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Base.jai", b"".as_slice());
        fs.insert("/lib/prelude/module.jai", b"".as_slice());
        for entries in [
            vec!["Base.jai", "prelude/module.jai"],
            vec!["prelude/module.jai", "Base.jai"],
        ] {
            let selected = find_preload_with(&fs, Path::new("/lib"), &entries, None).unwrap();
            assert_eq!(selected, Path::new("/lib").join(entries[0]));
            assert_eq!(
                stdlib_root_with(&selected, &entries),
                Some(Path::new("/lib"))
            );
            assert_eq!(
                enclosing_stdlib_with(
                    &fs,
                    Path::new("/lib/internal/file.jai"),
                    &entries,
                    None,
                    Path::new("/other")
                ),
                Some("/lib".into())
            );
        }
        assert_eq!(
            find_preload_with(&fs, Path::new("/lib"), &["absent.jai", "Base.jai"], None),
            Some("/lib/Base.jai".into())
        );
        assert_eq!(
            find_preload_with(
                &fs,
                Path::new("/lib"),
                &["Base.jai"],
                Some(Path::new("missing.jai"))
            ),
            None
        );
    }

    #[test]
    fn a_root_preload_file_is_detected_after_the_prelude_folder() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Preload.jai", b"".as_slice());
        let selected = find_preload_with(&fs, Path::new("/lib"), PRELUDE_ENTRIES, None).unwrap();
        assert_eq!(selected, Path::new("/lib/Preload.jai"));
        assert_eq!(
            stdlib_root_with(&selected, PRELUDE_ENTRIES),
            Some(Path::new("/lib"))
        );
        assert_eq!(
            enclosing_stdlib_with(
                &fs,
                Path::new("/lib/Basic/module.jai"),
                PRELUDE_ENTRIES,
                None,
                Path::new("/other")
            ),
            Some("/lib".into())
        );
        fs.insert("/lib/prelude/module.jai", b"".as_slice());
        assert_eq!(
            find_preload_with(&fs, Path::new("/lib"), PRELUDE_ENTRIES, None),
            Some("/lib/prelude/module.jai".into())
        );
    }

    #[test]
    fn relative_override_finds_an_enclosing_root_file() {
        let mut fs = VirtualFs::default();
        fs.insert("/lib/Base.jai", b"".as_slice());
        assert_eq!(
            enclosing_stdlib_with(
                &fs,
                Path::new("/lib/internal/file.jai"),
                PRELUDE_ENTRIES,
                Some(Path::new("Base.jai")),
                Path::new("/lib")
            ),
            Some("/lib".into())
        );
    }

    #[test]
    fn external_prelude_keeps_the_configured_library_root() {
        let mut fs = VirtualFs::default();
        fs.insert("/external/Base.jai", b"".as_slice());
        let entries = ["prelude/module.jai"];
        let explicit = Some(Path::new("/external/Base.jai"));
        assert_eq!(
            enclosing_stdlib_with(
                &fs,
                Path::new("/lib/internal/file.jai"),
                &entries,
                explicit,
                Path::new("/lib")
            ),
            Some("/lib".into())
        );
        assert_eq!(
            enclosing_stdlib_with(
                &fs,
                Path::new("/app/main.jai"),
                &entries,
                explicit,
                Path::new("/lib")
            ),
            None
        );
        assert_eq!(
            enclosing_stdlib_with(
                &fs,
                Path::new("/external/Base.jai"),
                &entries,
                explicit,
                Path::new("/lib")
            ),
            None
        );
    }

    #[test]
    fn missing_message_lists_candidates_and_explicit_path() {
        let message = missing_stdlib_message(
            Path::new("/lib"),
            &["prelude/module.jai", "Base.jai"],
            Some(Path::new("missing.jai")),
        );
        for path in [
            "/lib/prelude/module.jai",
            "/lib/Base.jai",
            "/lib/missing.jai",
            "JAIC_PRELUDE",
        ] {
            assert!(message.contains(path), "{message}");
        }
    }

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
