//! macOS: the system `cc` links libSystem by itself.
use super::{DataImport, Linker, Runtime, RuntimeName, Toolchain};

pub(super) static MACOS: Toolchain = Toolchain {
    linker: Linker::Unix,
    runtime: Runtime::NONE,
    runtime_names: &[RuntimeName::plain("c"), RuntimeName::plain("libc")],
    names_fold_case: false,
    // libm is part of libSystem, so `-lm` is a no-op that keeps the Unix link command alike.
    implicit_libraries: &["m"],
    renames: |_| None,
    runtime_data: DataImport::Direct,
    main_stack: None,
    foreign_modules: None,
};
