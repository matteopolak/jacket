//! Native foreign calls: dynamic library loading and calling C functions
//! through fixed-shape trampolines.
//!
//! On the supported 64-bit C ABIs (System V x86-64 and AArch64) integer and
//! floating-point arguments are assigned to separate register files, so any
//! signature with at most 8 integer and 8 floating-point arguments can be
//! called through one prototype taking 8 of each. Structs passed or returned by
//! value are split into register pieces by `abi` (the same classification the
//! LLVM backend uses); larger ones go by address or through a hidden result pointer.
// The calling machinery is only reachable on the CPUs `call` supports (not on wasm).
#![cfg_attr(
    not(any(target_arch = "x86_64", target_arch = "aarch64")),
    allow(dead_code, unused_imports)
)]
use crate::abi::{self, Arch, Passing, Piece, PieceTy};
use crate::ir::{Sig, Ty};

mod callbacks;
pub use callbacks::{Reenter, callback_addr};

#[derive(Clone)]
pub struct Library {
    handle: usize,
}

/// Directories searched for libraries by name before the system's, set once by the driver
/// (the third-party libraries `tools/build_native_libs.py` builds; see docs/tools/native-libs.md).
static LIBRARY_DIRS: std::sync::OnceLock<Vec<std::path::PathBuf>> = std::sync::OnceLock::new();

pub fn set_library_dirs(dirs: Vec<std::path::PathBuf>) {
    let _ = LIBRARY_DIRS.set(dirs);
}

pub fn library_dirs() -> &'static [std::path::PathBuf] {
    LIBRARY_DIRS.get().map_or(&[], Vec::as_slice)
}

#[cfg(unix)]
mod sys {
    use std::ffi::{c_char, c_int, c_void};
    unsafe extern "C" {
        pub fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void;
        pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }
    pub const RTLD_NOW: c_int = 2;
    #[cfg(target_os = "macos")]
    pub const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;
    #[cfg(not(target_os = "macos"))]
    pub const RTLD_DEFAULT: *mut c_void = std::ptr::null_mut();
}

impl Library {
    /// Open a library by Jai name (`"c"`, `"SDL2"`, `"libs/foo"`...).
    #[cfg(unix)]
    pub fn open(name: &str, system: bool, base_dir: &str) -> Option<Library> {
        use std::ffi::CString;
        if system && matches!(name, "c" | "libc" | "m" | "libm" | "pthread" | "dl") {
            return Some(Library {
                handle: sys::RTLD_DEFAULT as usize,
            });
        }
        let ext = if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        let mut candidates = Vec::new();
        let file = std::path::Path::new(name)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = std::path::Path::new(name)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let joined = |d: &str, f: &str| {
            if d.is_empty() {
                f.to_string()
            } else {
                format!("{d}/{f}")
            }
        };
        if !system {
            let base = joined(base_dir, &dir);
            candidates.push(joined(&base, &format!("{file}.{ext}")));
            candidates.push(joined(&base, &format!("lib{file}.{ext}")));
        }
        let bare = name.strip_prefix("lib").unwrap_or(name);
        for d in library_dirs() {
            candidates.push(d.join(format!("lib{bare}.{ext}")).display().to_string());
        }
        candidates.push(format!("lib{name}.{ext}"));
        candidates.push(format!("{name}.{ext}"));
        if cfg!(target_os = "macos") {
            candidates.push(format!(
                "/System/Library/Frameworks/{name}.framework/{name}"
            ));
            candidates.push(format!("/opt/homebrew/lib/lib{name}.dylib"));
        }
        for c in candidates {
            let Ok(path) = CString::new(c) else {
                continue;
            };
            let handle = unsafe { sys::dlopen(path.as_ptr(), sys::RTLD_NOW) };
            if !handle.is_null() {
                return Some(Library {
                    handle: handle as usize,
                });
            }
        }
        None
    }

    #[cfg(not(unix))]
    pub fn open(_name: &str, _system: bool, _base_dir: &str) -> Option<Library> {
        None
    }
}

/// Resolve a symbol in `lib`, or anywhere in the process.
#[cfg(unix)]
pub fn lookup(lib: Option<&Library>, symbol: &str) -> Option<u64> {
    let name = std::ffi::CString::new(symbol).ok()?;
    let handle = lib.map_or(sys::RTLD_DEFAULT, |l| l.handle as *mut std::ffi::c_void);
    let mut p = unsafe { sys::dlsym(handle, name.as_ptr()) };
    if p.is_null() {
        p = unsafe { sys::dlsym(sys::RTLD_DEFAULT, name.as_ptr()) };
    }
    (!p.is_null()).then_some(p as u64)
}

#[cfg(not(unix))]
pub fn lookup(_lib: Option<&Library>, _symbol: &str) -> Option<u64> {
    None
}

/// Stack argument slots the call prototype passes after its registers.
const STACK_SLOTS: usize = 16;

/// Apple's arm64 ABI passes every variadic argument on the stack, in 8-byte slots.
const VARARGS_ON_STACK: bool = cfg!(all(target_vendor = "apple", target_arch = "aarch64"));

/// x86-64 System V has six integer argument registers; the prototype's last two integer
/// parameters are then its first two stack slots.
const X86_64: bool = cfg!(target_arch = "x86_64");

/// Arguments past the registers go to the stack in 8-byte slots, except that Apple's arm64
/// ABI packs those smaller than 8 bytes, which the prototype cannot express.
const PACKED_STACK: bool = VARARGS_ON_STACK;

/// The arguments of one call: integer and floating-point registers, then 8-byte stack slots
/// in argument order.
struct Regs {
    ints: [u64; 8],
    floats: [u64; 8],
    stack: [u64; STACK_SLOTS + 8],
    ni: usize,
    nf: usize,
    ns: usize,
    /// Integer registers available to arguments (on x86-64, one fewer when a hidden result
    /// pointer takes `rdi`).
    int_regs: usize,
}

impl Regs {
    fn new(sret: bool) -> Regs {
        Regs {
            ints: [0; 8],
            floats: [0; 8],
            stack: [0; STACK_SLOTS + 8],
            ni: 0,
            nf: 0,
            ns: 0,
            int_regs: if X86_64 {
                6 - sret as usize
            } else {
                8
            },
        }
    }
    fn int(&mut self, v: u64) -> Result<(), String> {
        if self.ni == self.int_regs {
            return self.stack(v);
        }
        self.ints[self.ni] = v;
        self.ni += 1;
        Ok(())
    }
    /// An `f32` travels in the low half of the register.
    fn float(&mut self, bits: u64) -> Result<(), String> {
        if self.nf == 8 {
            return self.stack(bits);
        }
        self.floats[self.nf] = bits;
        self.nf += 1;
        Ok(())
    }
    fn stack(&mut self, v: u64) -> Result<(), String> {
        // The prototype's integer parameters past `int_regs` are stack slots too.
        if self.ns == STACK_SLOTS + 8 - self.int_regs {
            return Err("foreign call has too many stack arguments".into());
        }
        self.stack[self.ns] = v;
        self.ns += 1;
        Ok(())
    }
    /// Whether `pieces` of one aggregate all fit the remaining registers: an aggregate goes
    /// entirely in registers or entirely on the stack.
    fn fits(&self, pieces: &[Piece]) -> bool {
        let ints = pieces.iter().filter(|p| p.ty == PieceTy::I64).count();
        self.ni + ints <= self.int_regs && self.nf + pieces.len() - ints <= 8
    }
    /// After an aggregate went to the stack, AAPCS64 gives later arguments of its register
    /// class no registers either.
    fn exhaust(&mut self, pieces: &[Piece]) {
        if X86_64 {
            return;
        }
        if pieces.iter().any(|p| p.ty == PieceTy::I64) {
            self.ni = self.int_regs;
        } else {
            self.nf = 8;
        }
    }
    /// The prototype's arguments: its 8 integer parameters (registers, then stack slots
    /// when there are fewer integer registers), 8 floats, and `STACK_SLOTS` more slots.
    fn prototype(&self) -> ([u64; 8], [f64; 8], [u64; STACK_SLOTS]) {
        let mut ints = [0; 8];
        let spill = 8 - self.int_regs;
        ints[..self.int_regs].copy_from_slice(&self.ints[..self.int_regs]);
        ints[self.int_regs..].copy_from_slice(&self.stack[..spill]);
        let mut stack = [0; STACK_SLOTS];
        stack.copy_from_slice(&self.stack[spill..spill + STACK_SLOTS]);
        (ints, self.floats.map(f64::from_bits), stack)
    }
}

/// Little-endian bytes `[addr, addr + len)` (len <= 8) as a `u64`.
///
/// SAFETY: interpreter addresses are host addresses of live memory.
unsafe fn read_bytes(addr: u64, len: u64) -> u64 {
    let mut out = [0u8; 8];
    let len = len.min(8) as usize;
    unsafe { std::ptr::copy_nonoverlapping(addr as *const u8, out.as_mut_ptr(), len) };
    u64::from_le_bytes(out)
}

/// SAFETY: as `read_bytes`.
unsafe fn write_bytes(addr: u64, value: u64, len: u64) {
    let bytes = value.to_le_bytes();
    let len = len.min(8) as usize;
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, len) };
}

// Return shapes. A C aggregate returned in registers comes back as one of these: Rust lays
// them out and classifies them like the C struct with the same register classes (an `f32`
// member arrives in the low half of its `f64` register).
#[repr(C)]
#[derive(Clone, Copy)]
struct II(u64, u64);
#[repr(C)]
#[derive(Clone, Copy)]
struct IF(u64, f64);
#[repr(C)]
#[derive(Clone, Copy)]
struct FI(f64, u64);
#[repr(C)]
#[derive(Clone, Copy)]
struct FF(f64, f64);
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)] // register classes, like `II` and `FF`
struct FFF(f64, f64, f64);
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
struct FFFF(f64, f64, f64, f64);
/// Large aggregates come back through a hidden pointer the callee fills.
const SRET_WORDS: usize = 64;
#[repr(C)]
#[derive(Clone, Copy)]
struct Sret([u64; SRET_WORDS]);

/// Call `addr` through a prototype taking 8 integer and 8 float parameters followed by
/// `STACK_SLOTS` 8-byte stack slots, returning `R` (see `Regs::prototype`).
///
/// On x86-64 the prototype is variadic after its first parameter: register assignment is the
/// same, and the caller then also sets `al`, which a variadic callee reads to find its
/// vector-register arguments.
///
/// SAFETY: `addr` is a C function whose arguments fit the registers and slots in `regs`.
unsafe fn call_as<R>(addr: u64, regs: &Regs) -> R {
    let ([i0, i1, i2, i3, i4, i5, i6, i7], [f0, f1, f2, f3, f4, f5, f6, f7], s) = regs.prototype();
    let [
        s0,
        s1,
        s2,
        s3,
        s4,
        s5,
        s6,
        s7,
        s8,
        s9,
        s10,
        s11,
        s12,
        s13,
        s14,
        s15,
    ] = s;
    #[cfg(target_arch = "x86_64")]
    {
        type Proto<R> = unsafe extern "C" fn(u64, ...) -> R;
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        unsafe {
            f(
                i0, i1, i2, i3, i4, i5, i6, i7, f0, f1, f2, f3, f4, f5, f6, f7, s0, s1, s2, s3, s4,
                s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15,
            )
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        #[rustfmt::skip]
        type Proto<R> = unsafe extern "C" fn(
            u64, u64, u64, u64, u64, u64, u64, u64,
            f64, f64, f64, f64, f64, f64, f64, f64,
            u64, u64, u64, u64, u64, u64, u64, u64,
            u64, u64, u64, u64, u64, u64, u64, u64,
        ) -> R;
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        unsafe {
            f(
                i0, i1, i2, i3, i4, i5, i6, i7, f0, f1, f2, f3, f4, f5, f6, f7, s0, s1, s2, s3, s4,
                s5, s6, s7, s8, s9, s10, s11, s12, s13, s14, s15,
            )
        }
    }
}

/// Call the C function at `addr`. Arguments are raw IR values classified by `sig`; a
/// by-value struct argument is a pointer to its memory, and a struct result is written
/// through the last IR argument (the out-pointer). Interpreted procedures the callee calls
/// back (see `callback_addr`) run through `reenter`.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn call(
    addr: u64,
    args: &[u64],
    sig: &Sig,
    reenter: &mut Reenter<'_>,
) -> Result<Vec<u64>, String> {
    #[cfg(target_os = "macos")]
    if let Some(result) = main_thread::forward(addr, args, sig, reenter) {
        return result;
    }
    callbacks::with_reenter(reenter, || call_with(addr, args, sig))
}

/// AppKit only works on the process's main thread, but the interpreter runs on a worker
/// with a big stack. The driver parks the main thread in `main_thread::serve`. Once the
/// program first calls into the Objective-C runtime or AppKit (`note_symbol`), the program's
/// primary thread hands every later foreign call over to it (the worker waits, so the
/// interpreter is never running on two threads at once). Until then calls stay on the worker:
/// a handoff costs microseconds, which programs that never open a window should not pay.
#[cfg(target_os = "macos")]
pub mod main_thread {
    use super::*;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::sync::{Mutex, OnceLock};

    type Job = Box<dyn FnOnce() + Send>;

    struct Route {
        jobs: Mutex<Sender<Job>>,
        worker: std::thread::ThreadId,
    }

    static ROUTE: OnceLock<Route> = OnceLock::new();

    static DISABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    /// Start forwarding at the first call into the Objective-C runtime or a Cocoa framework:
    /// from then on window, event and GL calls must share the main thread.
    pub fn note_symbol(symbol: &str) {
        if !ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
            && (symbol.starts_with("objc_")
                || symbol.starts_with("NS")
                || symbol.starts_with("CGL")
                || symbol.starts_with("sel_"))
        {
            ACTIVE.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Stop forwarding for good (a forked child has no main thread serving jobs).
    pub fn disable() {
        DISABLED.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    thread_local! {
        static DIRECT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }

    /// Run `f` with foreign calls made on the calling thread. `fork` needs this: the child
    /// only has the forking thread, so the call must come from the interpreter's own thread.
    pub fn direct<T>(f: impl FnOnce() -> T) -> T {
        let before = DIRECT.with(|d| d.replace(true));
        let result = f();
        DIRECT.with(|d| d.set(before));
        result
    }

    /// Run `body` on a new thread (`stack` bytes) while this thread serves its foreign calls.
    pub fn serve<T: Send + 'static>(
        stack: usize,
        body: impl FnOnce() -> T + Send + 'static,
    ) -> Option<T> {
        let (tx, rx): (Sender<Job>, Receiver<Job>) = channel();
        let worker = std::thread::Builder::new()
            .stack_size(stack)
            .spawn(move || {
                let _ = ROUTE.set(Route {
                    jobs: Mutex::new(tx),
                    worker: std::thread::current().id(),
                });
                body()
            });
        let handle = worker.ok()?;
        loop {
            match rx.recv_timeout(std::time::Duration::from_millis(5)) {
                Ok(job) => job(),
                Err(_) if handle.is_finished() => break,
                Err(_) => {}
            }
        }
        handle.join().ok()
    }

    /// libSystem (libc, libm, pthreads, libdispatch...) and libc++ are thread-safe and never
    /// touch AppKit: calling them directly avoids a thread handoff per call. Everything else
    /// (frameworks, and user libraries such as SDL or GLFW that call into Cocoa) is forwarded.
    fn needs_main_thread(addr: u64) -> bool {
        use std::collections::HashMap;
        thread_local! {
            static CACHE: std::cell::RefCell<HashMap<u64, bool>> = std::cell::RefCell::new(HashMap::default());
        }
        if let Some(known) = CACHE.with(|c| c.borrow().get(&addr).copied()) {
            return known;
        }
        #[repr(C)]
        struct DlInfo {
            fname: *const std::ffi::c_char,
            fbase: *mut std::ffi::c_void,
            sname: *const std::ffi::c_char,
            saddr: *mut std::ffi::c_void,
        }
        unsafe extern "C" {
            fn dladdr(addr: *const std::ffi::c_void, info: *mut DlInfo) -> i32;
        }
        let mut info = DlInfo {
            fname: std::ptr::null(),
            fbase: std::ptr::null_mut(),
            sname: std::ptr::null(),
            saddr: std::ptr::null_mut(),
        };
        // SAFETY: dladdr only reads the loaded-image tables and fills `info`.
        let found = unsafe { dladdr(addr as *const std::ffi::c_void, &mut info) } != 0;
        let main = if !found || info.fname.is_null() {
            true
        } else {
            // SAFETY: dladdr returns a NUL-terminated path owned by the loader.
            let path = unsafe { std::ffi::CStr::from_ptr(info.fname) }.to_string_lossy();
            !(path.starts_with("/usr/lib/system/")
                || path.starts_with("/usr/lib/libSystem")
                || path.starts_with("/usr/lib/libc++"))
        };
        CACHE.with(|c| c.borrow_mut().insert(addr, main));
        main
    }

    struct Carry<T>(T);
    // SAFETY: the sending thread blocks until the job finishes, so the pointers are never
    // used from two threads at once.
    unsafe impl<T> Send for Carry<T> {
    }

    pub(super) fn forward(
        addr: u64,
        args: &[u64],
        sig: &Sig,
        reenter: &mut Reenter<'_>,
    ) -> Option<Result<Vec<u64>, String>> {
        let route = ROUTE.get()?;
        if !ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
            || DISABLED.load(std::sync::atomic::Ordering::SeqCst)
            || DIRECT.with(|d| d.get())
        {
            return None;
        }
        if std::thread::current().id() != route.worker || !needs_main_thread(addr) {
            return None;
        }
        let (done_tx, done_rx) = channel();
        let reenter: *mut Reenter<'_> = reenter;
        // SAFETY: this frame waits for the job, so the callback outlives its use.
        let reenter: *mut Reenter<'static> = unsafe { std::mem::transmute(reenter) };
        let carried = Carry((args as *const [u64], sig as *const Sig, reenter));
        let job: Job = Box::new(move || {
            let carried = carried;
            let (args, sig, reenter) = carried.0;
            // SAFETY: see `Carry`; the caller's borrows outlive this job.
            let (args, sig, reenter) = unsafe { (&*args, &*sig, &mut *reenter) };
            let result = callbacks::with_reenter(reenter, || call_with(addr, args, sig));
            let _ = done_tx.send(Carry(result));
        });
        route.jobs.lock().ok()?.send(job).ok()?;
        done_rx.recv().ok().map(|c| c.0)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn call_with(addr: u64, args: &[u64], sig: &Sig) -> Result<Vec<u64>, String> {
    let arch = Arch::host().ok_or("native foreign calls are not available on this CPU")?;
    // The register model below is System V / AAPCS64; Windows hosts do not load native
    // libraries in the interpreter yet (`Library::open`), so this is not reached there.
    if arch == Arch::Win64 {
        return Err(
            "the interpreter does not implement the Microsoft x64 calling convention".into(),
        );
    }
    let cabi = sig.c_abi.as_deref();
    let ret_layout = cabi.and_then(|c| c.ret.as_ref());
    // `#cpp_return_type_is_non_pod` results always use the hidden result pointer.
    let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
    let ret_pieces = ret_layout
        .filter(|_| !forced_sret)
        .and_then(|l| abi::classify_ret(arch, l));
    let mut regs = Regs::new(ret_layout.is_some() && ret_pieces.is_none());
    // Copies of large aggregates passed by address; alive until the call returns.
    let mut copies: Vec<Vec<u64>> = Vec::new();
    let mut out_ptr = 0;
    for (i, &a) in args.iter().enumerate() {
        if ret_layout.is_some() && i + 1 == sig.params.len() {
            out_ptr = a;
            continue;
        }
        if VARARGS_ON_STACK && sig.c_varargs && i >= sig.c_fixed as usize {
            regs.stack(a)?;
            continue;
        }
        let layout = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref);
        let Some(layout) = layout else {
            let ty = sig.params.get(i).copied().unwrap_or(Ty::I64);
            let full = if ty.is_float() {
                regs.nf == 8
            } else {
                regs.ni == regs.int_regs
            };
            if full && PACKED_STACK && ty.size() < 8 {
                return Err(
                    "the interpreter cannot pass arguments smaller than 8 bytes on the stack on this CPU"
                        .into(),
                );
            }
            if ty.is_float() {
                regs.float(a)?;
            } else {
                regs.int(a)?;
            }
            continue;
        };
        match abi::classify_arg(arch, layout) {
            Passing::Registers(pieces) if !regs.fits(&pieces) => {
                regs.exhaust(&pieces);
                // SAFETY: as below.
                for k in 0..layout.size.div_ceil(8) {
                    regs.stack(unsafe { read_bytes(a + k * 8, layout.size - k * 8) })?;
                }
            }
            Passing::Registers(pieces) => {
                for p in pieces {
                    // SAFETY: `a` points at the aggregate, `layout.size` bytes long.
                    let v = unsafe { read_bytes(a + p.offset, layout.size - p.offset) };
                    if p.ty == PieceTy::I64 {
                        regs.int(v)?;
                    } else {
                        regs.float(v)?;
                    }
                }
            }
            Passing::Indirect => {
                let mut copy = vec![0u64; layout.size.div_ceil(8) as usize];
                // SAFETY: as above; the copy is at least `layout.size` bytes.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        a as *const u8,
                        copy.as_mut_ptr() as *mut u8,
                        layout.size as usize,
                    )
                };
                regs.int(copy.as_ptr() as u64)?;
                copies.push(copy);
            }
            // x86-64 `byval`: the aggregate's bytes are copied into the stack argument area.
            Passing::ByVal => {
                if layout.align > 8 {
                    return Err(format!(
                        "the interpreter cannot pass a {}-byte aligned struct by value",
                        layout.align
                    ));
                }
                for k in 0..layout.size.div_ceil(8) {
                    // SAFETY: `a` points at the aggregate, `layout.size` bytes long.
                    regs.stack(unsafe { read_bytes(a + k * 8, layout.size - k * 8) })?;
                }
            }
        }
    }
    let Some(layout) = ret_layout else {
        return Ok(scalar_call(addr, &regs, sig.returns.first().copied()));
    };
    // SAFETY (all calls below): the callee's declared C signature matches these registers.
    match ret_pieces {
        None => {
            if layout.size as usize > SRET_WORDS * 8 {
                return Err(format!(
                    "the interpreter cannot return a {}-byte struct from a foreign procedure",
                    layout.size
                ));
            }
            let r: Sret = unsafe { call_as(addr, &regs) };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    r.0.as_ptr() as *const u8,
                    out_ptr as *mut u8,
                    layout.size as usize,
                )
            };
        }
        Some(pieces) => {
            let int = |p: &Piece| p.ty == PieceTy::I64;
            let words: Vec<u64> = match pieces.as_slice() {
                [] => {
                    unsafe { call_as::<()>(addr, &regs) };
                    Vec::new()
                }
                [a] if int(a) => vec![unsafe { call_as::<u64>(addr, &regs) }],
                [_] => vec![unsafe { call_as::<f64>(addr, &regs) }.to_bits()],
                [a, b] => match (int(a), int(b)) {
                    (true, true) => {
                        let r: II = unsafe { call_as(addr, &regs) };
                        vec![r.0, r.1]
                    }
                    (true, false) => {
                        let r: IF = unsafe { call_as(addr, &regs) };
                        vec![r.0, r.1.to_bits()]
                    }
                    (false, true) => {
                        let r: FI = unsafe { call_as(addr, &regs) };
                        vec![r.0.to_bits(), r.1]
                    }
                    (false, false) => {
                        let r: FF = unsafe { call_as(addr, &regs) };
                        vec![r.0.to_bits(), r.1.to_bits()]
                    }
                },
                [_, _, _] => {
                    let r: FFF = unsafe { call_as(addr, &regs) };
                    vec![r.0.to_bits(), r.1.to_bits(), r.2.to_bits()]
                }
                [_, _, _, _] => {
                    let r: FFFF = unsafe { call_as(addr, &regs) };
                    vec![r.0.to_bits(), r.1.to_bits(), r.2.to_bits(), r.3.to_bits()]
                }
                _ => return Err("unsupported C aggregate return shape".into()),
            };
            for (p, w) in pieces.iter().zip(words) {
                let size = match p.ty {
                    PieceTy::F32 => 4,
                    _ => 8,
                };
                // SAFETY: the out-pointer addresses `layout.size` writable bytes.
                unsafe { write_bytes(out_ptr + p.offset, w, size.min(layout.size - p.offset)) };
            }
        }
    }
    drop(copies);
    Ok(Vec::new())
}

/// A call whose result is a scalar (or nothing).
fn scalar_call(addr: u64, regs: &Regs, ret: Option<Ty>) -> Vec<u64> {
    // SAFETY: the address comes from the dynamic linker for a declared foreign procedure.
    match ret {
        Some(t) if t.is_float() => {
            let bits = unsafe { call_as::<f64>(addr, regs) }.to_bits();
            vec![if t == Ty::F32 {
                bits & 0xffff_ffff
            } else {
                bits
            }]
        }
        Some(t) => {
            let r = unsafe { call_as::<u64>(addr, regs) };
            vec![match t {
                Ty::I8 => r & 0xff,
                Ty::I16 => r & 0xffff,
                Ty::I32 => r & 0xffff_ffff,
                _ => r,
            }]
        }
        None => {
            unsafe { call_as::<u64>(addr, regs) };
            Vec::new()
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn call(
    _addr: u64,
    _args: &[u64],
    _sig: &Sig,
    _reenter: &mut Reenter<'_>,
) -> Result<Vec<u64>, String> {
    Err("native foreign calls are not available on this platform".into())
}
