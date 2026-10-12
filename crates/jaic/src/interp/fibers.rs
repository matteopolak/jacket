//! Fibers under the interpreter (`fiber_init`, `fiber_switch`; see `docs/compiler/fibers.md`).
//!
//! A fiber here is a native stack of its own for the interpreter's Rust frames, plus an
//! `ExecState` (the interpreted value stack, call depth and trace state). Switching saves the
//! running `ExecState` into the record of the context being left, installs the one of the target,
//! and swaps the native stacks with the same assembly routine native code uses (`crate::fiber`).
//! The program's own stack memory (what `Fiber_Context` was given) is not used: interpreted frames
//! live on the value stack.
//!
//! Records are kept by the address of the `Fiber_Context` they stand for, so the program's
//! memory holds nothing the interpreter has to trust: a context that was never initialised and
//! is only switched away from (the thread's own stack) gets a record when it first leaves.
//!
//! A fiber never returns from its entry procedure (the stdlib switches away for good). A trap in
//! a fiber ends the program as it would anywhere else: the fiber hands it to the context that
//! started the chain it belongs to, whose `fiber_switch` returns it as an error.
#![allow(unsafe_code)]

use super::*;
use crate::fiber::Context;

/// Native stack for the interpreter's own frames in a fiber: address space the OS commits as the
/// fiber touches it, with an inaccessible guard at the low end. Windows commits it up front, so
/// it is smaller there and fibers nest less deeply.
const NATIVE_STACK: usize = if cfg!(windows) {
    1 << 20
} else if cfg!(debug_assertions) {
    32 << 20
} else {
    4 << 20
};

/// Bytes at the low end of a native stack that fault when touched: a multiple of every page size.
const GUARD: usize = 64 << 10;

/// Native stack one interpreted call can use, at the very most: a tiny recursive call takes about
/// 800 bytes in a release build, and the paths through foreign calls and `#c_call` thunks more.
const BYTES_PER_CALL: usize = if cfg!(debug_assertions) {
    16 << 10
} else {
    3_500
};

/// Interpreted call depth a fiber gets, set so that its native stack cannot run out first.
const FIBER_DEPTH: usize = (NATIVE_STACK - GUARD) / BYTES_PER_CALL;

/// Value stack for interpreted frames, in the same mapping as the native stack.
const VALUE_STACK: usize = 512 << 10;

pub(super) struct FiberRec {
    ctx: Context,
    /// Saved while the fiber is not running; `None` while it is.
    exec: Option<ExecState>,
    /// The native stack the fiber runs on (empty for a context that was already running).
    native: Region,
    interp: *mut Interp,
    program: *const Program,
    entry: FuncId,
    data: u64,
    /// The address of this record's own `Fiber_Context`.
    key: u64,
    /// The context that first switched to this fiber; 0 for one the thread started on.
    parent: u64,
    started: bool,
}

impl FiberRec {
    fn root(key: u64) -> Self {
        FiberRec {
            ctx: Context::default(),
            exec: None,
            native: Region::empty(),
            interp: std::ptr::null_mut(),
            program: std::ptr::null(),
            entry: FuncId(0),
            data: 0,
            key,
            parent: 0,
            started: true,
        }
    }
}

/// A stack's worth of address space, mapped with its guard. Unmapped when dropped.
struct Region {
    base: *mut u8,
    len: usize,
}

impl Region {
    fn empty() -> Self {
        Region {
            base: std::ptr::null_mut(),
            len: 0,
        }
    }
}

#[cfg(all(unix, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod map {
    use super::{GUARD, Region};
    use std::ffi::c_void;

    unsafe extern "C" {
        fn mmap(
            addr: *mut c_void,
            len: usize,
            prot: i32,
            flags: i32,
            fd: i32,
            off: i64,
        ) -> *mut c_void;

        fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
        fn munmap(addr: *mut c_void, len: usize) -> i32;
    }

    // PROT_READ | PROT_WRITE; MAP_PRIVATE | MAP_ANON | MAP_NORESERVE.
    const READ_WRITE: i32 = 3;

    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0x0002 | 0x1000 | 0x0040;

    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x0002 | 0x0020 | 0x4000;

    impl Region {
        pub(super) fn new(len: usize) -> Option<Self> {
            // SAFETY: an anonymous mapping of fresh address space.
            let base = unsafe { mmap(std::ptr::null_mut(), len, READ_WRITE, FLAGS, -1, 0) };
            if base as isize == -1 {
                return None;
            }
            // SAFETY: the first `GUARD` bytes of the mapping.
            if unsafe { mprotect(base, GUARD, 0) } != 0 {
                // SAFETY: unmapping what was just mapped.
                unsafe { munmap(base, len) };
                return None;
            }
            Some(Region {
                base: base.cast(),
                len,
            })
        }
    }

    impl Drop for Region {
        fn drop(&mut self) {
            if self.len > 0 {
                // SAFETY: the mapping `new` made.
                unsafe { munmap(self.base.cast(), self.len) };
            }
        }
    }
}

#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod map {
    use super::{GUARD, Region};
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn VirtualAlloc(addr: *mut c_void, len: usize, kind: u32, protect: u32) -> *mut c_void;
        fn VirtualProtect(addr: *mut c_void, len: usize, protect: u32, old: *mut u32) -> i32;
        fn VirtualFree(addr: *mut c_void, len: usize, kind: u32) -> i32;
    }

    const COMMIT_RESERVE: u32 = 0x1000 | 0x2000;
    const READ_WRITE: u32 = 0x04;
    const NO_ACCESS: u32 = 0x01;
    const RELEASE: u32 = 0x8000;

    impl Region {
        pub(super) fn new(len: usize) -> Option<Self> {
            // SAFETY: fresh address space.
            let base =
                unsafe { VirtualAlloc(std::ptr::null_mut(), len, COMMIT_RESERVE, READ_WRITE) };
            if base.is_null() {
                return None;
            }
            let mut old = 0u32;
            // SAFETY: the first `GUARD` bytes of the allocation.
            if unsafe { VirtualProtect(base, GUARD, NO_ACCESS, &mut old) } == 0 {
                // SAFETY: releasing what was just allocated.
                unsafe { VirtualFree(base, 0, RELEASE) };
                return None;
            }
            Some(Region {
                base: base.cast(),
                len,
            })
        }
    }

    impl Drop for Region {
        fn drop(&mut self) {
            if self.len > 0 {
                // SAFETY: the allocation `new` made.
                unsafe { VirtualFree(self.base.cast(), 0, RELEASE) };
            }
        }
    }
}

/// The unsupported-host message (wasm32 builds of the compiler, CPUs without the assembly).
#[cfg(not(all(
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
impl Interp {
    pub(super) fn fiber_intrinsic(
        &mut self,
        _program: &Program,
        _op: ir::Intrinsic,
        _a: &[u64],
    ) -> Res<()> {
        self.trap("fibers are not supported by the interpreter on this host")
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
impl Interp {
    /// `Intrinsic::FiberInit` `(context, stack, size, entry, data)` and `Intrinsic::FiberSwitch`
    /// `(from, to)`.
    pub(super) fn fiber_intrinsic(
        &mut self,
        program: &Program,
        op: ir::Intrinsic,
        a: &[u64],
    ) -> Res<()> {
        if !self.host.native_linking() {
            return self.trap("fibers are not available in the sandbox (WebAssembly) host");
        }
        match op {
            ir::Intrinsic::FiberInit => self.fiber_init(program, a[0], a[3], a[4]),
            ir::Intrinsic::FiberFree => {
                // The running fiber's record stays: its stack is in use.
                if self.fibers.get(&a[0]).is_some_and(|r| r.exec.is_some()) {
                    self.fibers.remove(&a[0]);
                }
                Ok(())
            }
            _ => self.fiber_switch(program, a[0], a[1]),
        }
    }

    fn fiber_init(&mut self, program: &Program, key: u64, entry: u64, data: u64) -> Res<()> {
        if key < 4096 {
            return self.null_trap("fiber_init through a null pointer");
        }
        let Some(entry) = self.func_of(entry) else {
            return self.trap("fiber_init needs an interpreted entry procedure");
        };
        // A pooled fiber's context is initialised again: keep its memory.
        let native = match self.fibers.remove(&key) {
            Some(mut old) if old.native.len > 0 => {
                std::mem::replace(&mut old.native, Region::empty())
            }
            _ => match Region::new(GUARD + NATIVE_STACK + VALUE_STACK) {
                Some(region) => region,
                None => return self.trap("fiber_init: could not map a stack for the interpreter"),
            },
        };
        // The region is the guard, the native stack and, above them, the value stack.
        let low = native.base as u64;
        let top = low + (GUARD + NATIVE_STACK) as u64;
        let value_stack = ValueStack::borrowed(top as *mut u64, VALUE_STACK / 8);
        let mut rec = Box::new(FiberRec {
            ctx: Context::default(),
            exec: Some(ExecState {
                stack: value_stack,
                depth: MAX_DEPTH - FIBER_DEPTH,
                ..ExecState::default()
            }),
            native,
            interp: self,
            program,
            entry,
            data,
            key,
            parent: 0,
            started: false,
        });
        let arg = &*rec as *const FiberRec as u64;
        // SAFETY: `top` ends a stack this record owns, and `fiber_main` never returns.
        rec.ctx = Context {
            sp: unsafe { crate::fiber::host::jaic_fiber_prepare(top, fiber_main, arg) },
            stack_low: low,
            stack_high: top,
            stack_dealloc: low,
        };
        self.fibers.insert(key, rec);
        Ok(())
    }

    fn fiber_switch(&mut self, program: &Program, from: u64, to: u64) -> Res<()> {
        if from < 4096 || to < 4096 {
            return self.null_trap("fiber_switch through a null pointer");
        }
        if from == to {
            return Ok(());
        }
        let this: *mut Interp = self;
        let Some(target) = self.fibers.get_mut(&to) else {
            return self.trap("fiber_switch to a context that fiber_init never set up");
        };
        let Some(exec) = target.exec.take() else {
            return self.trap("fiber_switch to a fiber that is running");
        };
        target.interp = this;
        target.program = program;
        if !target.started {
            target.started = true;
            target.parent = from;
        }
        let to_ctx: *mut Context = &mut target.ctx;
        let state = self.take_exec_state();
        let leaving = self
            .fibers
            .entry(from)
            .or_insert_with(|| Box::new(FiberRec::root(from)));
        leaving.exec = Some(state);
        let from_ctx: *mut Context = &mut leaving.ctx;
        self.put_exec_state(exec);
        // SAFETY: both contexts belong to boxed records, which the map keeps until a fiber is
        // initialised again, and `to` holds a prepared or saved stack.
        unsafe { crate::fiber::host::jaic_fiber_switch(from_ctx, to_ctx) };
        // Running again: whoever switched here installed this fiber's state first.
        match self.fiber_trap.take() {
            Some(trap) => Err(trap),
            None => Ok(()),
        }
    }

    /// The fiber `key` ended in `trap`: report it from the context the chain of fibers started
    /// in, and never come back.
    fn fiber_died(&mut self, key: u64, trap: Trap) -> ! {
        let mut at = key;
        for _ in 0..1_000_000 {
            match self.fibers.get(&at) {
                Some(rec) if rec.parent != 0 => at = rec.parent,
                _ => break,
            }
        }
        let has_root = at != key && self.fibers.get(&at).is_some_and(|r| r.exec.is_some());
        if !has_root {
            eprintln!("error in a fiber: {}", trap.message);
            std::process::exit(1);
        }
        self.fiber_trap = Some(trap);
        let mine = self.take_exec_state();
        let root = self.fibers.get_mut(&at).expect("root checked above");
        let root_ctx: *mut Context = &mut root.ctx;
        let exec = root.exec.take().expect("root checked above");
        self.put_exec_state(exec);
        drop(mine);
        let me = self
            .fibers
            .get_mut(&key)
            .expect("a running fiber has a record");
        let my_ctx: *mut Context = &mut me.ctx;
        // SAFETY: as in `fiber_switch`; the root is suspended in its own `fiber_switch`.
        unsafe { crate::fiber::host::jaic_fiber_switch(my_ctx, root_ctx) };
        std::process::abort()
    }
}

/// First code of a fiber on its own native stack: runs the entry procedure.
#[cfg(all(
    not(target_arch = "wasm32"),
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
extern "C" fn fiber_main(rec: u64) -> ! {
    // SAFETY: `rec` is the address of the boxed record `fiber_init` made and the map keeps.
    let (interp, program, entry, data, key) = unsafe {
        let rec = &*(rec as *const FiberRec);
        (rec.interp, rec.program, rec.entry, rec.data, rec.key)
    };

    // SAFETY: the interpreter and program outlive every fiber that runs.
    let (interp, program) = unsafe { (&mut *interp, &*program) };

    let trap = match interp.exec(program, entry, &[data]) {
        Err(trap) => trap,
        Ok(_) => Trap::new(
            "a fiber's entry procedure returned; it must switch to another fiber for good instead",
        ),
    };
    interp.fiber_died(key, trap)
}
