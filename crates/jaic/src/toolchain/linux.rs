//! Linux: the system `cc` links libc by itself.
use super::{DataImport, Linker, Runtime, RuntimeName, Toolchain};

pub(super) static LINUX: Toolchain = Toolchain {
    linker: Linker::Unix,
    runtime: Runtime::NONE,
    runtime_names: &[RuntimeName::plain("c"), RuntimeName::plain("libc")],
    names_fold_case: false,
    // LLVM lowers float rounding (`llvm.trunc`, `floor`, `ceil`, `round`, from `#asm` and casts)
    // to libm calls on CPUs without SSE4.1 (baseline x86-64), so libm is always linked.
    implicit_libraries: &["m"],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: None,
    foreign_modules: None,
};
