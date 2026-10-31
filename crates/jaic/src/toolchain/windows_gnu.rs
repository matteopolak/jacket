//! Windows with MinGW-w64: the driver links the C runtime statically.
use super::{DataImport, Linker, Runtime, RuntimeName, Toolchain};

/// The DLL of the generic C runtime. It lacks many of `ucrtbase.dll`'s exports and has its own
/// heap, `FILE*` and `errno`, so the two are never folded into one.
pub(super) const MSVCRT_DLL: &str = "msvcrt.dll";

/// The DLL of the universal C runtime.
pub(super) const UCRTBASE_DLL: &str = "ucrtbase.dll";

const fn row(name: &'static str, dll: &'static str) -> RuntimeName {
    RuntimeName {
        name,
        dll: Some(dll),
    }
}

/// Library names that mean the C runtime on Windows, including the POSIX libraries that are part
/// of it there (`pthread`, `dl`), each with the DLL the interpreter loads for it. The names of
/// the universal runtime load `ucrtbase.dll`; everything else loads `msvcrt.dll`.
pub(super) const RUNTIME_NAMES: &[RuntimeName] = &[
    row("c", MSVCRT_DLL),
    row("libc", MSVCRT_DLL),
    row("crt", MSVCRT_DLL),
    row("msvcrt", MSVCRT_DLL),
    row("libcmt", MSVCRT_DLL),
    row("m", MSVCRT_DLL),
    row("libm", MSVCRT_DLL),
    row("pthread", MSVCRT_DLL),
    row("libpthread", MSVCRT_DLL),
    row("dl", MSVCRT_DLL),
    row("libdl", MSVCRT_DLL),
    row("rt", MSVCRT_DLL),
    row("librt", MSVCRT_DLL),
    row("ucrt", UCRTBASE_DLL),
    row("ucrtbase", UCRTBASE_DLL),
    row("vcruntime", UCRTBASE_DLL),
];

/// Windows reserves 1 MiB for the main thread's stack (and threads created without a size);
/// reserve the 8 MiB macOS and Linux give, so deep recursion behaves the same.
pub(super) const MAIN_STACK: u64 = 8 * 1024 * 1024;

pub(super) static WINDOWS_GNU: Toolchain = Toolchain {
    linker: Linker::MinGw,
    runtime: Runtime::NONE,
    runtime_names: RUNTIME_NAMES,
    names_fold_case: true,
    implicit_libraries: &[],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: Some(MAIN_STACK),
    foreign_modules: None,
};
