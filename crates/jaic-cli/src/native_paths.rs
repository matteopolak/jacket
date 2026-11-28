//! The library folders the interpreter searches by name, from `jaic-toolchain`'s lookups.
use std::path::{Path, PathBuf};

use jaic_toolchain::{System, apple, native_libs};

use super::crash_runtime;

/// The crash runtime's folder, the built third-party libraries (`JAIC_NATIVE_LIBS`) and, on a
/// Mac, the selected toolchain's `libclang.dylib` folder. When the crash runtime cannot be
/// unpacked, or that folder has no libclang, the interpreter and the linker are told why, and say
/// so if a program needs the library.
pub(super) fn native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    // The crash runtime is built into jaic itself (docs/native/crash-runtime-build.md). It comes
    // first so an older copy left in a configured folder never shadows the one this stdlib needs;
    // the folder holds only those runtimes, so no other library is affected.
    let mut dirs: Vec<PathBuf> = match crash_runtime::dir() {
        Ok(dir) => dir.into_iter().collect(),
        Err(why) => {
            missing.push((crash_runtime::LIBRARY.to_string(), why));
            Vec::new()
        }
    };

    match native_libs::host_lib_dirs(&System, stdlib) {
        Ok(found) => dirs.extend(found),
        // A misnamed folder is a mistake in the environment, reported before anything runs.
        Err(message) => {
            super::parse::print_error(&message);
            std::process::exit(1);
        }
    }

    if cfg!(target_os = "macos")
        && let Err(why) = apple::libclang_dir(&System).map(|dir| dirs.push(dir))
    {
        missing.push(("clang".into(), why.to_string()));
    }

    jaic::interp::set_missing_libraries(missing);
    dirs
}
