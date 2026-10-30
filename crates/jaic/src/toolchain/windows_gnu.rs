//! Windows with MinGW-w64: the driver links the C runtime statically.
use super::{DataImport, Linker, Runtime, Toolchain};

/// Library names that mean the C runtime on Windows, including the POSIX libraries that are part
/// of it there (`pthread`, `dl`).
pub(super) const RUNTIME_NAMES: &[&str] = &[
    "c",
    "libc",
    "crt",
    "msvcrt",
    "ucrt",
    "ucrtbase",
    "vcruntime",
    "libcmt",
    "m",
    "libm",
    "pthread",
    "libpthread",
    "dl",
    "libdl",
    "rt",
    "librt",
];

/// The names the interpreter loads the runtime DLL for.
pub(super) const INTERPRETER_NAMES: &[&str] = &["c", "libc", "m", "libm", "msvcrt"];

/// Windows reserves 1 MiB for the main thread's stack (and threads created without a size);
/// reserve the 8 MiB macOS and Linux give, so deep recursion behaves the same.
pub(super) const MAIN_STACK: u64 = 8 * 1024 * 1024;

pub(super) static WINDOWS_GNU: Toolchain = Toolchain {
    linker: Linker::MinGw,
    runtime: Runtime::NONE,
    runtime_names: RUNTIME_NAMES,
    names_fold_case: true,
    interpreter_names: INTERPRETER_NAMES,
    implicit_libraries: &[],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: Some(MAIN_STACK),
    foreign_modules: None,
};
