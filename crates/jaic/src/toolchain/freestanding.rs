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

/// Whether linker output names an undefined compiler-rt builtin (`__udivti3`, `__addtf3`,
/// `__aarch64_ldadd4_acq_rel`): those come from the builtins archive, not from the program.
pub fn undefined_builtin(log: &str) -> bool {
    log.lines()
        .filter(|line| line.contains("undefined symbol"))
        .any(|line| {
            line.split("undefined symbol:")
                .nth(1)
                .is_some_and(|name| name.trim_start().starts_with("__"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undefined_compiler_rt_symbols_are_told_apart_from_the_programs() {
        assert!(undefined_builtin(
            "ld.lld: error: undefined symbol: __udivti3\n>>> referenced by a.o"
        ));
        assert!(!undefined_builtin(
            "ld.lld: error: undefined symbol: uart_write"
        ));
    }
}
