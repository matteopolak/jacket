//! The library folders the interpreter searches by name, from `jaic-toolchain`'s lookups.
use super::crash_runtime;
use jaic_toolchain::{System, apple, native_libs};
use std::path::{Path, PathBuf};

/// The crash runtime's folder, the built third-party libraries (`JAIC_NATIVE_LIBS`) and, on a
/// Mac, the selected toolchain's `libclang.dylib` folder. When that folder has no libclang, the
/// interpreter is told why, and says so if a program opens it.
pub(super) fn native_lib_dirs(stdlib: &Path) -> Vec<PathBuf> {
    // The crash runtime is built into jaic itself (docs/native/crash-runtime-build.md). It comes
    // first so an older copy left in a configured folder never shadows the one this stdlib needs;
    // the folder holds only those runtimes, so no other library is affected.
    let mut dirs: Vec<PathBuf> = crash_runtime::dir().into_iter().collect();
    dirs.extend(native_libs::host_lib_dirs(&System, stdlib));
    if cfg!(target_os = "macos") {
        match apple::libclang_dir(&System) {
            Ok(dir) => dirs.push(dir),
            Err(missing) => {
                jaic::interp::set_missing_libraries(vec![("clang".into(), missing.to_string())]);
            }
        }
    }
    dirs
}
