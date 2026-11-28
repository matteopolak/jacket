//! Freestanding targets (`*-none-elf`): no C runtime and no libc. Programs link with `ld.lld` and
//! a linker script, plus the compiler builtins for the exact target (64-bit division, soft
//! float and the like), which `jaic-toolchain` finds. See `docs/native/embedded.md`.
use super::{DataImport, Linker, Runtime, Toolchain};

pub(super) static FREESTANDING: Toolchain = Toolchain {
    linker: Linker::Unix,
    runtime: Runtime::NONE,
    // No library name means "the C runtime": there is none.
    runtime_names: &[],
    names_fold_case: false,
    implicit_libraries: &[],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: None,
    foreign_modules: None,
};
