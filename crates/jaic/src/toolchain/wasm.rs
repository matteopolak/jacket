//! WebAssembly: no C runtime is linked; the host supplies foreign functions as imports.
use super::{DataImport, ForeignModules, Linker, Runtime, RuntimeName, Toolchain};

pub(super) static WASM: Toolchain = Toolchain {
    linker: Linker::Unix,
    runtime: Runtime::NONE,
    runtime_names: &[RuntimeName::plain("c"), RuntimeName::plain("libc")],
    names_fold_case: false,
    implicit_libraries: &[],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: None,
    foreign_modules: Some(ForeignModules {
        default: "env",
        runtime: &["c", "libc", "crt", "msvcrt", "m", "libm"],
    }),
};
