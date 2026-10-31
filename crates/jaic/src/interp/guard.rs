//! Turning a hardware fault in interpreted memory accesses into an error.
//!
//! Interpreted loads and stores use the program's own addresses directly (`memory.rs`), so a
//! wild pointer in `#run` code faults inside jaic. `guarded` runs a closure so that such a fault
//! comes back as a `Fault` instead of ending the process:
//!
//! - entering saves the callee-saved registers and the stack pointer in a `Guard` on the caller's
//!   stack (a few stores; nothing happens per load or store);
//! - the fault handler in `crash.rs` finds the innermost guard of the faulting thread, records
//!   the address, and edits the interrupted context so that returning from the handler lands in
//!   `jacket_guard_recover`, which restores those registers and returns `1` from `guard_run`, as
//!   if the closure had returned early. No stack unwinding runs: the frames in between are
//!   abandoned, so values they owned leak, and the caller must put any state they were in the
//!   middle of changing back (`Interp::call` restores its stacks).
//!
//! Only SIGSEGV and SIGBUS on macOS (arm64) and Linux (x86-64, arm64) are recovered; elsewhere
//! `guarded` just runs the closure and a fault is reported by `crash.rs` and ends the process.
#![allow(unsafe_code)]

/// A memory access that faulted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fault {
    /// The address the access used.
    pub address: u64,
    /// Whether it was a store, when the processor says.
    pub write: Option<bool>,
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )
))]
mod imp {
    use super::Fault;
    use std::cell::Cell;
    use std::ffi::c_void;

    /// Words `guard_run` saves.
    const SAVED: usize = 24;

    #[repr(C)]
    struct Guard {
        regs: [u64; SAVED],
        previous: *mut Guard,
        fault: Fault,
    }

    thread_local! {
        // Constant-initialized without a destructor, so the fault handler may read it.
        static INNERMOST: Cell<*mut Guard> = const { Cell::new(std::ptr::null_mut()) };
    }

    #[cfg(target_arch = "aarch64")]
    #[unsafe(naked)]
    unsafe extern "C" fn guard_run(
        _regs: *mut [u64; SAVED],
        _f: extern "C" fn(*mut c_void),
        _data: *mut c_void,
    ) -> u64 {
        core::arch::naked_asm!(
            "stp x19, x20, [x0, #0]",
            "stp x21, x22, [x0, #16]",
            "stp x23, x24, [x0, #32]",
            "stp x25, x26, [x0, #48]",
            "stp x27, x28, [x0, #64]",
            "stp x29, x30, [x0, #80]",
            "mov x9, sp",
            "str x9, [x0, #96]",
            "stp d8, d9, [x0, #104]",
            "stp d10, d11, [x0, #120]",
            "stp d12, d13, [x0, #136]",
            "stp d14, d15, [x0, #152]",
            "stp x29, x30, [sp, #-16]!",
            "mov x29, sp",
            "mov x0, x2",
            "blr x1",
            "ldp x29, x30, [sp], #16",
            "mov x0, #0",
            "ret",
        )
    }

    /// Entered with the pointer to the saved registers in `x0`.
    #[cfg(target_arch = "aarch64")]
    #[unsafe(naked)]
    unsafe extern "C" fn recover(_regs: *mut [u64; SAVED]) -> ! {
        core::arch::naked_asm!(
            "ldp x19, x20, [x0, #0]",
            "ldp x21, x22, [x0, #16]",
            "ldp x23, x24, [x0, #32]",
            "ldp x25, x26, [x0, #48]",
            "ldp x27, x28, [x0, #64]",
            "ldp x29, x30, [x0, #80]",
            "ldr x9, [x0, #96]",
            "ldp d8, d9, [x0, #104]",
            "ldp d10, d11, [x0, #120]",
            "ldp d12, d13, [x0, #136]",
            "ldp d14, d15, [x0, #152]",
            "mov sp, x9",
            "mov x0, #1",
            "ret",
        )
    }

    #[cfg(target_arch = "x86_64")]
    #[unsafe(naked)]
    unsafe extern "C" fn guard_run(
        _regs: *mut [u64; SAVED],
        _f: extern "C" fn(*mut c_void),
        _data: *mut c_void,
    ) -> u64 {
        core::arch::naked_asm!(
            "mov [rdi], rbx",
            "mov [rdi + 8], rbp",
            "mov [rdi + 16], r12",
            "mov [rdi + 24], r13",
            "mov [rdi + 32], r14",
            "mov [rdi + 40], r15",
            "lea rax, [rsp + 8]",
            "mov [rdi + 48], rax",
            "mov rax, [rsp]",
            "mov [rdi + 56], rax",
            "push rbp",
            "mov rdi, rdx",
            "call rsi",
            "pop rbp",
            "xor eax, eax",
            "ret",
        )
    }

    /// Entered with the pointer to the saved registers in `rdi`.
    #[cfg(target_arch = "x86_64")]
    #[unsafe(naked)]
    unsafe extern "C" fn recover(_regs: *mut [u64; SAVED]) -> ! {
        core::arch::naked_asm!(
            "mov rbx, [rdi]",
            "mov rbp, [rdi + 8]",
            "mov r12, [rdi + 16]",
            "mov r13, [rdi + 24]",
            "mov r14, [rdi + 32]",
            "mov r15, [rdi + 40]",
            "mov rsp, [rdi + 48]",
            "mov eax, 1",
            "jmp [rdi + 56]",
        )
    }

    struct Job<F, R> {
        f: Option<F>,
        out: Option<R>,
        panic: Option<Box<dyn std::any::Any + Send>>,
    }

    extern "C" fn run<F: FnOnce() -> R, R>(data: *mut c_void) {
        // SAFETY: `data` is the `Job` `guarded` passed, alive for the whole call.
        let job = unsafe { &mut *data.cast::<Job<F, R>>() };

        let Some(f) = job.f.take() else {
            return;
        };

        // A panic must not unwind through `guard_run`'s frame: it is carried across and resumed.
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(out) => job.out = Some(out),
            Err(payload) => job.panic = Some(payload),
        }
    }

    pub fn guarded<F: FnOnce() -> R, R>(f: F) -> Result<R, Fault> {
        crate::interp::crash::install();
        let mut job = Job {
            f: Some(f),
            out: None,
            panic: None,
        };
        let mut guard = Guard {
            regs: [0; SAVED],
            previous: INNERMOST.with(Cell::get),
            fault: Fault::default(),
        };
        let guard_ptr: *mut Guard = &raw mut guard;
        INNERMOST.with(|g| g.set(guard_ptr));
        let f_ptr: extern "C" fn(*mut c_void) = run::<F, R>;
        // SAFETY: `run` takes the job pointer; the registers buffer is this guard's.
        let status = unsafe {
            guard_run(
                &raw mut (*guard_ptr).regs,
                f_ptr,
                (&raw mut job).cast::<c_void>(),
            )
        };
        INNERMOST.with(|g| g.set(guard.previous));
        if status == 0 {
            if let Some(payload) = job.panic {
                std::panic::resume_unwind(payload);
            }
            return Ok(job.out.expect("the guarded closure ran"));
        }
        // SAFETY: the handler wrote the field through the pointer stored in `INNERMOST`.
        Err(unsafe { std::ptr::read_volatile(&raw const (*guard_ptr).fault) })
    }

    /// If the thread has a guard, record the fault and arrange for the interrupted context to
    /// resume in `recover`. `context` is the `ucontext_t` the kernel passed the handler.
    pub fn redirect(address: usize, context: *mut c_void) -> bool {
        let guard = INNERMOST.with(Cell::get);
        if guard.is_null() || context.is_null() {
            return false;
        }
        // SAFETY: `guard` is live while it is `INNERMOST`; `context` is the kernel's.
        unsafe {
            let resume = recover as *const () as usize;
            let write = set_resume(context, resume, &raw mut (*guard).regs as usize);
            std::ptr::write_volatile(
                &raw mut (*guard).fault,
                Fault {
                    address: address as u64,
                    write,
                },
            );
        }
        true
    }

    /// Make the context resume at `pc` with the first argument register holding `arg`; returns
    /// whether the faulting access was a store, when known.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    unsafe fn set_resume(context: *mut c_void, pc: usize, arg: usize) -> Option<bool> {
        // `ucontext_t`: the pointer to the `mcontext` is at offset 48; it holds the exception
        // state (far, esr, exception: 16 bytes) and then x0..x28, fp, lr, sp, pc.
        // SAFETY: `context` is the kernel's `ucontext_t`, laid out as described above.
        unsafe {
            let mcontext = std::ptr::read_unaligned(context.cast::<u8>().add(48).cast::<*mut u8>());
            let esr = std::ptr::read_unaligned(mcontext.add(8).cast::<u32>());
            std::ptr::write_unaligned(mcontext.add(16).cast::<usize>(), arg);
            std::ptr::write_unaligned(mcontext.add(272).cast::<usize>(), pc);
            // A data abort (class 0x24 or 0x25) has the write-not-read bit at 6.
            matches!(esr >> 26, 0x24 | 0x25).then_some(esr & 0x40 != 0)
        }
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    unsafe fn set_resume(context: *mut c_void, pc: usize, arg: usize) -> Option<bool> {
        // `ucontext_t`'s `mcontext` is at offset 176: fault address, x0..x30, sp, pc.
        // SAFETY: `context` is the kernel's `ucontext_t`, laid out as described above.
        unsafe {
            let mcontext = context.cast::<u8>().add(176);
            std::ptr::write_unaligned(mcontext.add(8).cast::<usize>(), arg);
            std::ptr::write_unaligned(mcontext.add(8 + 31 * 8 + 8).cast::<usize>(), pc);
            // The records after the registers (`_aarch64_ctx`: magic, size) hold the exception
            // syndrome when the kernel has one; the list ends with a zero magic.
            let mut record = mcontext.add(288);
            for _ in 0..16 {
                let magic = std::ptr::read_unaligned(record.cast::<u32>());
                let size = std::ptr::read_unaligned(record.add(4).cast::<u32>()) as usize;
                if magic == 0 || size < 8 {
                    break;
                }
                if magic == 0x4553_5201 {
                    let esr = std::ptr::read_unaligned(record.add(8).cast::<u64>());
                    return matches!(esr >> 26, 0x24 | 0x25).then_some(esr & 0x40 != 0);
                }
                record = record.add(size);
            }
        }
        None
    }

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe fn set_resume(context: *mut c_void, pc: usize, arg: usize) -> Option<bool> {
        // `ucontext_t`'s general registers start at offset 40: REG_RDI is 8, REG_RIP 16 and
        // REG_ERR 19 (bit 1 set for a write).
        // SAFETY: `context` is the kernel's `ucontext_t`, laid out as described above.
        unsafe {
            #[expect(
                clippy::cast_ptr_alignment,
                reason = "every access below is read_unaligned or write_unaligned"
            )]
            let gregs = context.cast::<u8>().add(40).cast::<u64>();
            let err = std::ptr::read_unaligned(gregs.add(19));
            std::ptr::write_unaligned(gregs.add(8), arg as u64);
            std::ptr::write_unaligned(gregs.add(16), pc as u64);
            Some(err & 2 != 0)
        }
    }
}

#[cfg(not(all(
    not(target_arch = "wasm32"),
    any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )
)))]
mod imp {
    use super::Fault;

    pub fn guarded<R>(f: impl FnOnce() -> R) -> Result<R, Fault> {
        Ok(f())
    }

    #[cfg(all(unix, any(target_os = "macos", target_os = "linux")))]
    pub fn redirect(_address: usize, _context: *mut std::ffi::c_void) -> bool {
        false
    }
}

pub use imp::guarded;
// Only the signal handler of `crash.rs` redirects faults.
#[cfg(all(unix, any(target_os = "macos", target_os = "linux")))]
pub use imp::redirect;

#[cfg(all(
    test,
    any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )
))]
mod tests {
    use super::*;

    fn read(address: usize) -> u64 {
        // SAFETY: deliberately wild; the guard recovers from it.
        unsafe { std::ptr::read_volatile(address as *const u64) }
    }

    fn write(address: usize) {
        // SAFETY: deliberately wild; the guard recovers from it.
        unsafe { std::ptr::write_volatile(address as *mut u64, 1) }
    }

    #[test]
    fn a_closure_that_does_not_fault_returns_its_value() {
        assert_eq!(guarded(|| 7), Ok(7));
    }

    #[test]
    fn a_wild_read_and_write_come_back_as_faults() {
        let fault = guarded(|| read(0xdead_0000)).unwrap_err();
        assert_eq!(fault.address, 0xdead_0000);
        let fault = guarded(|| write(0xdead_0008)).unwrap_err();
        assert_eq!(fault.address, 0xdead_0008);
        // The processor reports the direction on these targets.
        assert_eq!(fault.write, Some(true));
    }

    #[test]
    fn the_thread_keeps_running_and_guards_nest() {
        let inner = guarded(|| {
            let before = String::from("kept");
            let inner = guarded(|| read(0xdead_0000));
            assert!(inner.is_err());
            // The outer guard is still armed after the inner one recovered.
            let outer = read(0xbeef_0000);
            (before, outer)
        });
        assert_eq!(inner.unwrap_err().address, 0xbeef_0000);
        for _ in 0..100 {
            assert!(guarded(|| read(0x1000_0000_0000)).is_err());
        }
    }

    #[test]
    fn panics_pass_through_the_guard() {
        let caught = std::panic::catch_unwind(|| {
            let _ = guarded(|| panic!("through"));
        });
        assert!(caught.is_err());
    }
}
