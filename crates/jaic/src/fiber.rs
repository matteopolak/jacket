//! The stack-switching code behind fibers (`docs/compiler/interpreter.md`).
//!
//! A fiber's saved state is a [`Context`] (`Fiber_Context` in `stdlib/Extensions/Io/fiber_context.jai`):
//! the stack pointer, with the callee-saved registers pushed on that stack, plus the stack bounds
//! that Windows keeps in the thread information block. Two routines, written once per CPU and
//! calling convention (`fiber/<cpu>-<abi>.s`), do all of it:
//!
//! - `jaic_fiber_switch(from, to)` pushes the callee-saved registers, stores the stack pointer in
//!   `from`, loads it from `to`, pops the registers it finds there and returns on the new stack;
//! - `jaic_fiber_prepare(top, entry, arg) -> sp` lays out the frame a switch pops for a new fiber, so
//!   the first switch to it calls `entry(arg)` on the stack that ends at `top`. `entry` never returns.
//!
//! The same text serves the LLVM backend, which puts it into every module that switches fibers (as
//! module assembly with local symbols: each object file has its own copy), and the interpreter,
//! which assembles it into `jaic` itself and runs its own interpretation state on separate native
//! stacks (`interp/fibers.rs`).
//!
//! The routine files call two assembler macros that a small prelude defines per use: `fiber_symbol`
//! (defines a routine's label: with the Mach-O underscore, and exported only in the host copy) and,
//! for `AArch64`, `fiber_save_tib` / `fiber_load_tib` (the Windows stack bounds; empty elsewhere).
//! The trampoline label has no local-label prefix, so one text serves ELF, Mach-O and COFF.

/// What the routines save per fiber. The layout is `Fiber_Context`'s.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    /// The stack pointer while the fiber is not running.
    pub sp: u64,
    /// Lowest address of the stack (Windows `StackLimit`).
    pub stack_low: u64,
    /// One past the highest address (Windows `StackBase`).
    pub stack_high: u64,
    /// Windows `DeallocationStack`.
    pub stack_dealloc: u64,
}

/// Byte offsets of `Context`'s fields.
pub const SP: u64 = 0;

pub const STACK_LOW: u64 = 8;
pub const STACK_HIGH: u64 = 16;
pub const STACK_DEALLOC: u64 = 24;

/// The symbols the assembly defines (without the Mach-O underscore).
pub const SWITCH: &str = "jaic_fiber_switch";

pub const PREPARE: &str = "jaic_fiber_prepare";

/// The ELF note that keeps the linker from making the stack executable.
const ELF_NOTE: &str = include_str!("fiber/gnu-stack-note.s");

/// The text for `arch`, to put into a module that switches fibers: the symbols are local to the
/// module, so every object file of a program may carry a copy. `macho` selects the Mach-O symbol
/// prefix; `windows` and `macho` otherwise pick the object format by the architecture.
pub fn module_asm(arch: crate::abi::Arch, macho: bool) -> Option<String> {
    use crate::abi::Arch;
    let symbols = if macho {
        include_str!("fiber/symbols-local-macho.s")
    } else {
        include_str!("fiber/symbols-local.s")
    };
    let no_tib = include_str!("fiber/aarch64-no-tib.s");
    let (tib, routines, elf) = match arch {
        Arch::X86_64 => ("", include_str!("fiber/x86_64-sysv.s"), !macho),
        Arch::Win64 => ("", include_str!("fiber/x86_64-win64.s"), false),
        Arch::Aarch64 => (no_tib, include_str!("fiber/aarch64-aapcs64.s"), !macho),
        Arch::Win64Arm => (
            include_str!("fiber/aarch64-win64-tib.s"),
            include_str!("fiber/aarch64-aapcs64.s"),
            false,
        ),
        Arch::Wasm64 => return None,
    };
    let note = if elf {
        ELF_NOTE
    } else {
        ""
    };
    Some(format!("{symbols}{tib}{routines}{note}"))
}

/// The routines linked into `jaic` for the interpreter (the host's own architecture only).
#[allow(unsafe_code)]
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub mod host {
    use super::Context;

    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host-macho.s"),
        include_str!("fiber/x86_64-sysv.s"),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "x86_64", unix, not(target_os = "macos")))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host.s"),
        include_str!("fiber/x86_64-sysv.s"),
        include_str!("fiber/gnu-stack-note.s"),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "x86_64", windows))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host.s"),
        include_str!("fiber/x86_64-win64.s"),
        options(att_syntax)
    );

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host-macho.s"),
        include_str!("fiber/aarch64-no-tib.s"),
        include_str!("fiber/aarch64-aapcs64.s"),
    );

    #[cfg(all(target_arch = "aarch64", unix, not(target_os = "macos")))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host.s"),
        include_str!("fiber/aarch64-no-tib.s"),
        include_str!("fiber/aarch64-aapcs64.s"),
        include_str!("fiber/gnu-stack-note.s"),
    );

    #[cfg(all(target_arch = "aarch64", windows))]
    std::arch::global_asm!(
        include_str!("fiber/symbols-host.s"),
        include_str!("fiber/aarch64-win64-tib.s"),
        include_str!("fiber/aarch64-aapcs64.s"),
    );

    unsafe extern "C" {
        /// Save the running fiber into `from` and run `to`; returns when something switches back.
        pub fn jaic_fiber_switch(from: *mut Context, to: *mut Context);

        /// The stack pointer to store in a `Context` so the first switch to it calls `entry(arg)`
        /// on the stack ending at `top`.
        pub fn jaic_fiber_prepare(top: u64, entry: extern "C" fn(u64) -> !, arg: u64) -> u64;
    }
}

#[cfg(all(
    test,
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
#[allow(unsafe_code)]
mod tests {
    use super::{Context, host};
    use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

    static MAIN: AtomicPtr<Context> = AtomicPtr::new(std::ptr::null_mut());
    static CHILD: AtomicPtr<Context> = AtomicPtr::new(std::ptr::null_mut());
    static SEEN: AtomicU64 = AtomicU64::new(0);

    /// Records its argument, then hands control back to the test each time it is resumed.
    extern "C" fn counting_entry(arg: u64) -> ! {
        let mut turn = 0;

        loop {
            turn += 1;
            SEEN.store(arg + turn, Ordering::SeqCst);
            // SAFETY: both contexts outlive the test and only this thread touches them.
            unsafe {
                host::jaic_fiber_switch(CHILD.load(Ordering::SeqCst), MAIN.load(Ordering::SeqCst));
            }
        }
    }

    /// `prepare` lays out a frame that the first `switch` runs as `entry(arg)`, and later
    /// switches resume the fiber where it left off (callee-saved registers and the loop counter
    /// survive).
    #[test]
    fn a_prepared_fiber_runs_and_resumes() {
        let mut stack = vec![0u8; 64 * 1024];
        let low = stack.as_mut_ptr() as u64;
        let high = low + stack.len() as u64;
        let mut main = Context::default();
        let mut child = Context::default();
        child.stack_low = low;
        child.stack_high = high;
        child.stack_dealloc = low;
        MAIN.store(&raw mut main, Ordering::SeqCst);
        CHILD.store(&raw mut child, Ordering::SeqCst);
        // SAFETY: `stack` is a live allocation whose end is `high`; `counting_entry` never
        // returns; `main` and `child` stay alive and are used from this thread only.
        unsafe {
            child.sp = host::jaic_fiber_prepare(high, counting_entry, 100);
            for turn in 1..=3 {
                host::jaic_fiber_switch(&raw mut main, &raw mut child);
                assert_eq!(SEEN.load(Ordering::SeqCst), 100 + turn);
            }
        }
        assert!(
            child.sp > low && child.sp < high,
            "the saved pointer is on the fiber's stack"
        );
    }

    #[test]
    fn every_target_has_module_text_defining_both_routines() {
        use crate::abi::Arch;
        for (arch, macho) in [
            (Arch::X86_64, false),
            (Arch::X86_64, true),
            (Arch::Win64, false),
            (Arch::Aarch64, false),
            (Arch::Aarch64, true),
            (Arch::Win64Arm, false),
        ] {
            let text = super::module_asm(arch, macho).unwrap();
            for routine in [super::SWITCH, super::PREPARE] {
                assert!(
                    text.contains(&format!("fiber_symbol {routine}")),
                    "{arch:?} lacks {routine}"
                );
            }
            assert_eq!(
                text.contains(".note.GNU-stack"),
                matches!(arch, Arch::X86_64 | Arch::Aarch64) && !macho
            );
        }
        assert!(super::module_asm(Arch::Wasm64, false).is_none());
    }
}
