//! Native foreign calls: dynamic library loading and calling C functions
//! through fixed-shape trampolines.
//!
//! On the supported 64-bit C ABIs (System V x86-64 and `AArch64`) integer and
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
use crate::abi::{Arch, Passing, Piece, PieceTy};
use crate::interp::Rets;
use crate::ir::{Sig, Ty};

mod callbacks;
pub(super) mod waits;
pub use callbacks::{
    Gate, Reenter, callback_addr, caller, calling_out, release as release_callbacks,
};
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod wide;
#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod windows;
#[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
pub(super) use windows::set_last_error;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod plan;
#[cfg(windows)]
mod windows_imports;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub use plan::Plan;
mod words;
use words::{read_bytes, write_bytes};

#[derive(Clone)]
pub struct Library {
    #[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
    handle: usize,
    #[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
    windows: windows::Library,
}

/// Directories searched for libraries by name before the system's, set once by the driver
/// (the third-party libraries `tools/native/build_native_libs.py` builds; see docs/tools/native-libs.md).
static LIBRARY_DIRS: std::sync::OnceLock<Vec<std::path::PathBuf>> = std::sync::OnceLock::new();

pub fn set_library_dirs(dirs: Vec<std::path::PathBuf>) {
    let _ = LIBRARY_DIRS.set(dirs);
}

/// Why the driver found no folder for a library it looks for on the interpreter's behalf, by the
/// library's name without `lib` and extension (`clang` for `libclang.dylib`). Set once.
static MISSING_LIBRARIES: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();

/// Records why libraries the driver looked for were not found, so a failed open says so.
///
/// ```
/// jaic::interp::set_missing_libraries(vec![("clang".into(), "no libclang here".into())]);
/// assert_eq!(jaic::interp::missing_library("modules/clang"), Some("no libclang here"));
/// assert_eq!(jaic::interp::missing_library("libclang"), Some("no libclang here"));
/// assert_eq!(jaic::interp::missing_library("z"), None);
/// ```
pub fn set_missing_libraries(notes: Vec<(String, String)>) {
    let _ = MISSING_LIBRARIES.set(notes);
}

/// The driver's note for the library `name` (as a `#library` names it), when it has one.
pub fn missing_library(name: &str) -> Option<&'static str> {
    let file = std::path::Path::new(name).file_name()?.to_str()?;
    let bare = file.strip_prefix("lib").unwrap_or(file);
    MISSING_LIBRARIES
        .get()?
        .iter()
        .find(|(library, _)| library == bare)
        .map(|(_, note)| note.as_str())
}

/// Directories a metaprogram added with `compiler_add_library_search_directory`; searched
/// before the driver's.
static EXTRA_LIBRARY_DIRS: std::sync::Mutex<Vec<std::path::PathBuf>> =
    std::sync::Mutex::new(Vec::new());

pub fn add_library_dir(dir: std::path::PathBuf) {
    let mut dirs = EXTRA_LIBRARY_DIRS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !dirs.contains(&dir) {
        dirs.push(dir);
    }
}

/// The directories added by [`add_library_dir`], in the order they were added.
pub fn extra_library_dirs() -> Vec<std::path::PathBuf> {
    EXTRA_LIBRARY_DIRS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Every directory libraries are looked up in by name: those a metaprogram added, then the
/// driver's.
pub fn library_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = extra_library_dirs();
    dirs.extend(LIBRARY_DIRS.get().into_iter().flatten().cloned());
    dirs
}

/// Homebrew's library directory on this Mac: `/opt/homebrew/lib` on Apple silicon,
/// `/usr/local/lib` on Intel (searched after the system's own libraries).
pub fn homebrew_lib_dir() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "/usr/local/lib"
    } else {
        "/opt/homebrew/lib"
    }
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

/// Make the Linux x86-64 system call `args[0]` with `args[1..7]` and return the kernel's raw
/// result (`-errno` on failure). `None` on any other host.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn syscall(args: &[u64]) -> Option<i64> {
    unsafe extern "C" {
        fn syscall(number: i64, ...) -> i64;
    }
    let arg = |i: usize| crate::interp::word_or_zero(args, i);
    // SAFETY: the program asked for this system call; it is as unsafe as the `#asm` it wrote.
    let ret = unsafe {
        syscall(
            arg(0) as i64,
            arg(1),
            arg(2),
            arg(3),
            arg(4),
            arg(5),
            arg(6),
        )
    };
    if ret == -1 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(1);
        return Some(-i64::from(errno));
    }
    Some(ret)
}

/// Make the macOS x86-64 system call `args[0]` (the BSD class, `0x2000000`, plus the number) with
/// `args[1..7]` and return `rax` as the kernel leaves it: the result, or on failure the positive
/// errno (the carry flag, which a caller cannot see through `#asm`, is set then). `None` for any
/// other class.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub fn syscall(args: &[u64]) -> Option<i64> {
    const CLASS_MASK: u64 = 0xff00_0000;
    const UNIX_CLASS: u64 = 0x0200_0000;
    let arg = |i: usize| crate::interp::word_or_zero(args, i);
    if arg(0) & CLASS_MASK != UNIX_CLASS {
        return None;
    }
    let mut rax = arg(0);
    // SAFETY: the program asked for this system call; it is as unsafe as the `#asm` it wrote.
    unsafe {
        std::arch::asm!(
            "syscall",
            inout("rax") rax,
            in("rdi") arg(1),
            in("rsi") arg(2),
            in("rdx") arg(3),
            in("r10") arg(4),
            in("r8") arg(5),
            in("r9") arg(6),
            lateout("rcx") _,
            lateout("r11") _,
            options(nostack),
        );
    }
    Some(rax as i64)
}

#[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "x86_64")
)))]
pub fn syscall(_args: &[u64]) -> Option<i64> {
    None
}

#[cfg(all(test, target_os = "macos", target_arch = "x86_64"))]
mod syscall_tests {
    use super::syscall;

    const GETPID: u64 = 0x200_0000 + 20;
    const MMAP: u64 = 0x200_0000 + 197;
    const MUNMAP: u64 = 0x200_0000 + 73;
    const CLOSE: u64 = 0x200_0000 + 6;

    #[test]
    fn bsd_class_calls_reach_the_kernel() {
        assert_eq!(syscall(&[GETPID]), Some(i64::from(std::process::id())));
        // Anonymous private read-write mapping (MAP_ANON | MAP_PRIVATE), then released.
        let address = syscall(&[MMAP, 0, 4096, 3, 0x1002, u64::MAX, 0]).unwrap();
        assert!(address > 4096, "mmap returned {address}");
        assert_eq!(syscall(&[MUNMAP, address as u64, 4096]), Some(0));
    }

    #[test]
    fn a_failing_call_returns_the_positive_errno() {
        const EBADF: i64 = 9;
        assert_eq!(syscall(&[CLOSE, 9999]), Some(EBADF));
    }

    #[test]
    fn a_number_outside_the_bsd_class_is_refused() {
        assert_eq!(syscall(&[20]), None);
    }
}

/// Whether the Linux x86-64 system call in `args` only maps or releases memory, so a sandbox
/// may make it: anonymous `mmap` (not `MAP_FIXED`), `munmap`, `mremap` (not `MREMAP_FIXED`)
/// and `madvise`. Allocators built on raw system calls need these to run at all.
pub fn memory_only_syscall(args: &[u64]) -> bool {
    const MMAP: u64 = 9;
    const MUNMAP: u64 = 11;
    const MREMAP: u64 = 25;
    const MADVISE: u64 = 28;
    const MAP_FIXED: u64 = 0x10;
    const MAP_ANONYMOUS: u64 = 0x20;
    const MREMAP_FIXED: u64 = 2;
    let arg = |i: usize| crate::interp::word_or_zero(args, i);
    match arg(0) {
        MMAP => arg(4) & MAP_ANONYMOUS != 0 && arg(4) & MAP_FIXED == 0,
        MREMAP => arg(4) & MREMAP_FIXED == 0,
        MUNMAP | MADVISE => true,
        _ => false,
    }
}

/// Whether the library a program names `name` is the C library (with its math, thread and
/// loader parts), which every Unix process already has loaded.
pub(super) fn is_c_library(name: &str, system: bool) -> bool {
    system && matches!(name, "c" | "libc" | "m" | "libm" | "pthread" | "dl")
}

impl Library {
    /// Open a library by Jai name (`"c"`, `"SDL2"`, `"libs/foo"`...).
    #[cfg(unix)]
    pub fn open(name: &str, system: bool, base_dir: &str) -> Option<Library> {
        use std::ffi::CString;
        if is_c_library(name, system) {
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
        // A name that already carries the library extension (`libstdc++.so.6`, a versioned
        // soname) is a complete file name: hand it to the loader as written.
        if file.contains(&format!(".{ext}")) {
            candidates.push(name.to_string());
        }
        // A name with a directory part (`crash-native/portable_crash`) that is not beside the
        // source is looked up by file name.
        let bare = file.strip_prefix("lib").unwrap_or(&file);
        for d in library_dirs() {
            candidates.push(d.join(format!("lib{bare}.{ext}")).display().to_string());
        }
        candidates.push(format!("lib{name}.{ext}"));
        candidates.push(format!("{name}.{ext}"));
        // Without the development package only the versioned name exists (`libatomic.so.1`,
        // which GCC links through its own `libatomic.so`).
        if cfg!(target_os = "linux") {
            for version in 0..=9 {
                candidates.push(format!("lib{bare}.so.{version}"));
            }
        }
        if cfg!(target_os = "macos") {
            candidates.push(format!(
                "/System/Library/Frameworks/{name}.framework/{name}"
            ));
            candidates.push(format!("{}/lib{name}.dylib", homebrew_lib_dir()));
        }
        for c in candidates {
            let Ok(path) = CString::new(c) else {
                continue;
            };
            #[cfg(target_os = "macos")]
            let forwarded = {
                // Loading runs native initializers, including ones that require
                // the process main thread. The worker retains the CString until
                // the existing synchronous dispatcher finishes the load.
                let sig = Sig {
                    params: vec![crate::ir::Ty::Ptr, crate::ir::Ty::I32],
                    returns: vec![crate::ir::Ty::Ptr],
                    conv: crate::ir::Conv::C,
                    c_varargs: false,
                    c_fixed: 2,
                    c_abi: None,
                };
                main_thread::forward_loader(
                    sys::dlopen as *const () as u64,
                    &[path.as_ptr() as u64, sys::RTLD_NOW as u64],
                    &sig,
                    &Plan::new(&sig),
                )
            };
            #[cfg(target_os = "macos")]
            let handle = match forwarded {
                Some(result) => result.ok()?.first().copied()? as *mut std::ffi::c_void,
                None => {
                    // SAFETY: this is a live NUL-terminated CString. A root,
                    // foreign callback or explicitly direct caller stays direct.
                    unsafe { sys::dlopen(path.as_ptr(), sys::RTLD_NOW) }
                }
            };
            // SAFETY: this is a live NUL-terminated CString.
            #[cfg(not(target_os = "macos"))]
            let handle = unsafe { sys::dlopen(path.as_ptr(), sys::RTLD_NOW) };
            if !handle.is_null() {
                return Some(Library {
                    handle: handle as usize,
                });
            }
        }
        None
    }

    /// Windows: `LoadLibraryW` of `name.dll` (see `windows::open`).
    #[cfg(all(test, windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
    pub fn open(name: &str, system: bool, base_dir: &str) -> Option<Library> {
        windows::open(name, system, base_dir)
            .ok()
            .map(|windows| Library {
                windows,
            })
    }

    /// Preserve concrete Windows import-provider errors for required-library diagnostics.
    pub fn open_checked(name: &str, system: bool, base_dir: &str) -> Result<Library, String> {
        #[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            windows::open(name, system, base_dir).map(|windows| Library {
                windows,
            })
        }
        #[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
        {
            Self::open(name, system, base_dir).ok_or_else(|| match missing_library(name) {
                Some(note) => format!("dynamic loader could not open the library\n{note}"),
                None => "dynamic loader could not open the library".into(),
            })
        }
    }

    #[cfg(not(any(
        unix,
        all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))
    )))]
    pub fn open(_name: &str, _system: bool, _base_dir: &str) -> Option<Library> {
        None
    }
}

/// Resolve a symbol in `lib`, or anywhere in the process.
#[cfg(unix)]
pub fn lookup(lib: Option<&Library>, symbol: &str) -> Option<u64> {
    let name = std::ffi::CString::new(symbol).ok()?;
    let handle = lib.map_or(sys::RTLD_DEFAULT, |l| l.handle as *mut std::ffi::c_void);
    // SAFETY: `handle` is `RTLD_DEFAULT` or the live handle of a library this module opened and
    // never closes; `name` is NUL-terminated.
    let mut p = unsafe { sys::dlsym(handle, name.as_ptr()) };
    if p.is_null() {
        // SAFETY: `RTLD_DEFAULT` is always a valid handle; `name` is NUL-terminated.
        p = unsafe { sys::dlsym(sys::RTLD_DEFAULT, name.as_ptr()) };
    }
    (!p.is_null()).then_some(p as u64)
}

#[cfg(all(test, windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn lookup(lib: Option<&Library>, symbol: &str) -> Option<u64> {
    lookup_checked(lib, symbol).ok().flatten()
}

/// Preserve structured import-library ambiguity/CPU errors at the interpreter boundary.
pub fn lookup_checked(lib: Option<&Library>, symbol: &str) -> Result<Option<u64>, String> {
    #[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        windows::lookup_library(lib.map(|library| &library.windows), symbol)
    }
    #[cfg(not(all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))))]
    {
        Ok(lookup(lib, symbol))
    }
}

#[cfg(not(any(
    unix,
    all(windows, any(target_arch = "x86_64", target_arch = "aarch64"))
)))]
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
/// ABI packs those smaller than 8 bytes at their own size and alignment (`Regs::packed`).
const PACKED_STACK: bool = VARARGS_ON_STACK;

/// The arguments of one call: integer and floating-point registers, then 8-byte stack slots
/// in argument order.
struct Regs {
    ints: [u64; 8],
    /// Vector registers: an `f64` (or `f32`) uses the low bits, a binary128 `long double`
    /// all of them.
    floats: [u128; 8],
    /// Whether any vector register holds a 128-bit value (which `call_as` cannot pass).
    quads: bool,
    stack: [u64; STACK_SLOTS + 8],
    ni: usize,
    nf: usize,
    ns: usize,
    /// Bytes of the last stack slot that packed arguments filled (0: none, so the next packed
    /// argument starts a new slot).
    tail: usize,
    /// Integer registers available to arguments (on x86-64, one fewer when a hidden result
    /// pointer takes `rdi`).
    int_avail: usize,
}

impl Regs {
    fn new(sret: bool) -> Regs {
        Regs {
            ints: [0; 8],
            floats: [0; 8],
            quads: false,
            stack: [0; STACK_SLOTS + 8],
            ni: 0,
            nf: 0,
            ns: 0,
            tail: 0,
            int_avail: if X86_64 {
                6 - sret as usize
            } else {
                8
            },
        }
    }

    fn int(&mut self, v: u64) -> Result<(), String> {
        if self.ni == self.int_avail {
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
        self.floats[self.nf] = bits as u128;
        self.nf += 1;
        Ok(())
    }

    /// A binary128 value: a whole vector register, or a 16-byte aligned stack slot.
    fn quad(&mut self, bits: u128) -> Result<(), String> {
        if self.nf == 8 {
            self.align_stack()?;
            self.stack(bits as u64)?;
            return self.stack((bits >> 64) as u64);
        }
        self.floats[self.nf] = bits;
        self.quads = true;
        self.nf += 1;
        Ok(())
    }

    /// Pad the stack arguments so the next one is 16-byte aligned (slot `k` is at `sp + 8k`).
    fn align_stack(&mut self) -> Result<(), String> {
        self.tail = 0;
        if self.ns % 2 == 1 {
            self.stack(0)?;
        }
        Ok(())
    }

    fn stack(&mut self, v: u64) -> Result<(), String> {
        // The prototype's integer parameters past `int_avail` are stack slots too.
        if self.ns == STACK_SLOTS + 8 - self.int_avail {
            return Err("foreign call has too many stack arguments".into());
        }
        self.stack[self.ns] = v;
        self.ns += 1;
        self.tail = 0;
        Ok(())
    }

    /// A stack argument of `size` (1, 2 or 4) bytes as Apple's arm64 ABI places it: at the
    /// next offset aligned to its size, sharing a slot with the small arguments before it.
    fn packed(&mut self, v: u64, size: usize) -> Result<(), String> {
        let at = self.tail.next_multiple_of(size);
        if self.tail == 0 || at + size > 8 {
            self.stack(0)?;
            self.stack[self.ns - 1] = v & ((1u64 << (size * 8)) - 1);
            self.tail = size;
            return Ok(());
        }
        let mask = (1u64 << (size * 8)) - 1;
        self.stack[self.ns - 1] |= (v & mask) << (at * 8);
        self.tail = at + size;
        Ok(())
    }

    /// Whether `pieces` of one aggregate all fit the remaining registers: an aggregate goes
    /// entirely in registers or entirely on the stack.
    fn fits(&self, pieces: &[Piece]) -> bool {
        let ints = pieces.iter().filter(|p| p.ty == PieceTy::I64).count();
        self.ni + ints <= self.int_avail && self.nf + pieces.len() - ints <= 8
    }

    /// After an aggregate went to the stack, AAPCS64 gives later arguments of its register
    /// class no registers either.
    fn exhaust(&mut self, pieces: &[Piece]) {
        if X86_64 {
            return;
        }
        if pieces.iter().any(|p| p.ty == PieceTy::I64) {
            self.ni = self.int_avail;
        } else {
            self.nf = 8;
        }
    }

    /// The prototype's arguments: its 8 integer parameters (registers, then stack slots
    /// when there are fewer integer registers), 8 floats, and `STACK_SLOTS` more slots.
    fn prototype(&self) -> ([u64; 8], [f64; 8], [u64; STACK_SLOTS]) {
        let mut ints = [0; 8];
        let spill = 8 - self.int_avail;
        ints[..self.int_avail].copy_from_slice(&self.ints[..self.int_avail]);
        ints[self.int_avail..].copy_from_slice(&self.stack[..spill]);
        let mut stack = [0; STACK_SLOTS];
        stack.copy_from_slice(&self.stack[spill..spill + STACK_SLOTS]);
        (ints, self.floats.map(|f| f64::from_bits(f as u64)), stack)
    }
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

/// Results up to this many words come back through a hidden pointer the callee fills, by value
/// (`Sret`); bigger ones get their pointer passed explicitly.
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
/// The interpreter only passes an `addr` it got from the dynamic linker (or one of its own
/// callback stubs) for a foreign procedure the program declared, with `regs` packed from that
/// declaration, so the callee is a C function whose arguments fit the registers and slots in
/// `regs` and whose result is an `R`.
fn call_as<R>(addr: u64, regs: &Regs) -> R {
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
        // SAFETY: `addr` is a C function (see above); the prototype passes the registers and
        // slots `regs` holds in the order the C ABI assigns them.
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        // SAFETY: as above.
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
        // SAFETY: `addr` is a C function (see above); the prototype passes the registers and
        // slots `regs` holds in the order the C ABI assigns them.
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        // SAFETY: as above.
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
/// back (see `callback_addr`) run through the interpreter's `Gate`; the caller marks the call
/// with `calling_out` so that those on this thread run as its own.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn call(addr: u64, args: &[u64], sig: &Sig) -> Result<Rets, String> {
    call_planned(addr, args, sig, &Plan::new(sig))
}

/// [`call`] with the signature's `Plan`, which a caller that makes the same call again keeps.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn call_planned(addr: u64, args: &[u64], sig: &Sig, plan: &Plan) -> Result<Rets, String> {
    #[cfg(target_os = "macos")]
    if let Some(result) = main_thread::forward(addr, args, sig, plan) {
        return result;
    }
    call_with(addr, args, sig, plan)
}

#[cfg(target_os = "macos")]
pub mod main_thread;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn call_with(addr: u64, args: &[u64], sig: &Sig, plan: &Plan) -> Result<Rets, String> {
    #[cfg(windows)]
    return windows::keeping_last_error(|| call_with_unkept(addr, args, sig, plan));
    #[cfg(not(windows))]
    call_with_unkept(addr, args, sig, plan)
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn call_with_unkept(addr: u64, args: &[u64], sig: &Sig, plan: &Plan) -> Result<Rets, String> {
    let arch = plan
        .arch
        .ok_or("native foreign calls are not available on this CPU")?;
    // A foreign call follows the host's C ABI, whatever the program's own layout.
    let host = &plan.host;
    // The register model below is System V / AAPCS64 (Windows on arm64 included); the
    // Microsoft x64 convention has its own (`windows.rs`).
    if arch == Arch::Win64 {
        #[cfg(all(windows, target_arch = "x86_64"))]
        return windows::call(addr, args, sig);
        #[cfg(not(all(windows, target_arch = "x86_64")))]
        return Err(
            "the interpreter does not implement the Microsoft x64 calling convention".into(),
        );
    }
    if plan.scalar
        && let Some(result) = register_call(addr, args, sig)
    {
        return Ok(result);
    }
    let cabi = sig.c_abi.as_deref();
    let ret_layout = cabi.and_then(|c| c.ret.as_ref());
    let ret_pieces = plan.ret_pieces.as_ref();
    let (sret, oversize, result_in_x0, general_only) = (
        plan.sret,
        plan.oversize,
        plan.result_in_x0,
        plan.general_only,
    );
    let result_after_this = plan.result_after_this;
    let mut regs = Regs::new(sret && !(oversize && X86_64) && !result_after_this);
    // Copies of large aggregates passed by address; alive until the call returns.
    let mut copies: Vec<Vec<u64>> = Vec::new();
    let mut out_ptr = 0;
    if (result_in_x0 || oversize && X86_64)
        && let Some(&out) = args.get(sig.out_index())
    {
        regs.int(out)?;
    }
    for (i, &a) in args.iter().enumerate() {
        // A C++ member function's result pointer follows its object.
        if result_after_this
            && i == 1
            && let Some(&out) = args.get(sig.out_index())
        {
            regs.int(out)?;
        }
        if ret_layout.is_some() && i == sig.out_index() {
            out_ptr = a;
            continue;
        }
        let layout = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref);
        let arg_plan = plan.params.get(i).and_then(Option::as_ref);
        if VARARGS_ON_STACK && sig.c_varargs && i >= sig.c_fixed as usize {
            // Apple's arm64 puts every variadic argument on the stack: an aggregate as its own
            // words, or as the address of a copy when it is over 16 bytes and not a float aggregate.
            match layout {
                None => regs.stack(a)?,
                Some(l) if arg_plan.is_some_and(|p| p.regular == Passing::Indirect) => {
                    regs.stack(indirect_copy(a, l.size, &mut copies))?;
                }
                Some(l) => {
                    for k in 0..l.size.div_ceil(8) {
                        // SAFETY: `a` points at the aggregate, `l.size` bytes long.
                        regs.stack(unsafe { read_bytes(a + k * 8, l.size - k * 8) })?;
                    }
                }
            }
            continue;
        }
        if general_only {
            match layout.zip(arg_plan.and_then(|p| p.vararg.as_ref())) {
                None => regs.int(a)?,
                Some((l, Passing::Registers(pieces))) => {
                    for p in pieces {
                        // SAFETY: `a` points at the aggregate, `l.size` bytes long.
                        regs.int(unsafe { read_bytes(a + p.offset, l.size - p.offset) })?;
                    }
                }
                Some((l, _)) => regs.int(indirect_copy(a, l.size, &mut copies))?,
            }
            continue;
        }
        let Some(layout) = layout else {
            let ty = sig.params.get(i).copied().unwrap_or(Ty::I64);
            let full = if ty.is_float() {
                regs.nf == 8
            } else {
                regs.ni == regs.int_avail
            };
            if full && PACKED_STACK && ty.size(host) < 8 {
                regs.packed(a, ty.size(host) as usize)?;
                continue;
            }
            if ty.is_float() {
                regs.float(a)?;
            } else {
                regs.int(a)?;
            }
            continue;
        };
        let Some(arg_plan) = arg_plan else {
            continue;
        };
        match &arg_plan.regular {
            // An x87 `long double` is memory class: a 16-byte aligned stack slot.
            Passing::Registers(pieces) if pieces.iter().any(|p| p.ty == PieceTy::X87) => {
                regs.align_stack()?;
                // SAFETY: `a` points at the 16-byte value.
                let (low, high) = unsafe { (read_bytes(a, 8), read_bytes(a + 8, 8)) };
                regs.stack(low)?;
                regs.stack(high)?;
            }
            Passing::Registers(pieces) if !regs.fits(pieces) => {
                regs.exhaust(pieces);
                if layout.align >= 16 {
                    regs.align_stack()?;
                }
                for k in 0..layout.size.div_ceil(8) {
                    // SAFETY: `a` points at the aggregate, `layout.size` bytes long.
                    regs.stack(unsafe { read_bytes(a + k * 8, layout.size - k * 8) })?;
                }
            }
            Passing::Registers(pieces) => {
                for p in pieces {
                    // SAFETY: `a` points at the aggregate, `layout.size` bytes long.
                    let v = unsafe { read_bytes(a + p.offset, layout.size - p.offset) };
                    if p.ty == PieceTy::F128 {
                        // SAFETY: a binary128 piece spans 16 bytes of the aggregate.
                        let high = unsafe { read_bytes(a + p.offset + 8, 8) };
                        regs.quad(v as u128 | (high as u128) << 64)?;
                    } else if p.ty == PieceTy::I64 {
                        regs.int(v)?;
                    } else {
                        regs.float(v)?;
                    }
                }
            }
            Passing::Indirect => regs.int(indirect_copy(a, layout.size, &mut copies))?,
            // x86-64 `byval`: the aggregate's bytes are copied into the stack argument area.
            Passing::ByVal => {
                if layout.align > 16 {
                    return Err(format!(
                        "the interpreter cannot pass a {}-byte aligned struct by value",
                        layout.align
                    ));
                }
                if layout.align == 16 {
                    regs.align_stack()?;
                }
                for k in 0..layout.size.div_ceil(8) {
                    // SAFETY: `a` points at the aggregate, `layout.size` bytes long.
                    regs.stack(unsafe { read_bytes(a + k * 8, layout.size - k * 8) })?;
                }
            }
        }
    }
    // `long double` results in `st(0)` and binary128 values in vector registers need the
    // assembly call of `wide.rs`.
    let wide_ret = plan.wide_ret;
    if regs.quads || wide_ret {
        let ret = ret_layout.map(|l| (l.size, ret_pieces.map(Vec::as_slice)));
        // SAFETY: the callee's declared C signature matches these registers.
        let result = unsafe { wide::call(addr, &regs, sig, ret, out_ptr) };
        drop(copies);
        return result;
    }
    let Some(layout) = ret_layout else {
        return Ok(scalar_call(addr, &regs, sig.returns.first().copied()));
    };
    if result_in_x0 || result_after_this {
        // The callee writes the result through the pointer in x0 (or x1, after `this`).
        call_as::<u64>(addr, &regs);
        return Ok(Rets::default());
    }
    if oversize {
        let result = if X86_64 {
            // The callee fills the buffer `rdi` addresses.
            call_as::<u64>(addr, &regs);
            Ok(Rets::default())
        } else {
            // SAFETY: the callee's declared C signature matches these registers, and it fills the
            // buffer `x8` addresses.
            unsafe { wide::call(addr, &regs, sig, Some((layout.size, None)), out_ptr) }
        };
        drop(copies);
        return result;
    }
    match ret_pieces {
        None => {
            let r: Sret = call_as(addr, &regs);
            // SAFETY: the out-pointer addresses `layout.size` writable bytes, and `r` holds at
            // least that many (results over `SRET_WORDS` words take the `oversize` path).
            unsafe {
                std::ptr::copy_nonoverlapping(
                    r.0.as_ptr().cast::<u8>(),
                    out_ptr as *mut u8,
                    layout.size as usize,
                );
            };
        }
        Some(pieces) => {
            let int = |p: &Piece| p.ty == PieceTy::I64;
            let words: Vec<u64> = match pieces.as_slice() {
                [] => {
                    call_as::<()>(addr, &regs);
                    Vec::new()
                }
                [a] if int(a) => vec![call_as::<u64>(addr, &regs)],
                [_] => vec![call_as::<f64>(addr, &regs).to_bits()],
                [a, b] => match (int(a), int(b)) {
                    (true, true) => {
                        let r: II = call_as(addr, &regs);
                        vec![r.0, r.1]
                    }
                    (true, false) => {
                        let r: IF = call_as(addr, &regs);
                        vec![r.0, r.1.to_bits()]
                    }
                    (false, true) => {
                        let r: FI = call_as(addr, &regs);
                        vec![r.0.to_bits(), r.1]
                    }
                    (false, false) => {
                        let r: FF = call_as(addr, &regs);
                        vec![r.0.to_bits(), r.1.to_bits()]
                    }
                },
                [_, _, _] => {
                    let r: FFF = call_as(addr, &regs);
                    vec![r.0.to_bits(), r.1.to_bits(), r.2.to_bits()]
                }
                [_, _, _, _] => {
                    let r: FFFF = call_as(addr, &regs);
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
    Ok(Rets::default())
}

/// A copy of the `size`-byte aggregate at `a`, kept in `copies` until the call returns, for
/// passing by reference; returns its address.
fn indirect_copy(a: u64, size: u64, copies: &mut Vec<Vec<u64>>) -> u64 {
    let mut copy = vec![0u64; size.div_ceil(8) as usize];
    // SAFETY: `a` points at the aggregate, `size` bytes long; the copy is at least as long.
    unsafe {
        std::ptr::copy_nonoverlapping(
            a as *const u8,
            copy.as_mut_ptr().cast::<u8>(),
            size as usize,
        );
    };
    let addr = copy.as_ptr() as u64;
    copies.push(copy);
    addr
}

/// Integer registers a scalar call can fill: x86-64 System V has six, `AArch64` eight.
const INT_REGISTERS: usize = if X86_64 {
    6
} else {
    8
};

/// A call of scalar arguments and a scalar result that fit the registers, which is almost every
/// foreign call: no argument layout, no stack slots, no hidden pointer. `None` when an argument
/// does not fit, and the general path packs it.
fn register_call(addr: u64, args: &[u64], sig: &Sig) -> Option<Rets> {
    let (mut ints, mut floats) = ([0u64; 8], [0f64; 8]);
    let (mut ni, mut nf) = (0, 0);
    for (i, &a) in args.iter().enumerate() {
        if sig.params.get(i).copied().unwrap_or(Ty::I64).is_float() {
            *floats.get_mut(nf)? = f64::from_bits(a);
            nf += 1;
        } else {
            if ni == INT_REGISTERS {
                return None;
            }
            ints[ni] = a;
            ni += 1;
        }
    }
    Some(match sig.returns.first().copied() {
        Some(t) if t.is_float() => {
            let bits = call_registers::<f64>(addr, &ints, &floats).to_bits();
            Rets::one(if t == Ty::F32 {
                bits & 0xffff_ffff
            } else {
                bits
            })
        }
        Some(t) => Rets::one(mask_int(t, call_registers::<u64>(addr, &ints, &floats))),
        None => {
            call_registers::<u64>(addr, &ints, &floats);
            Rets::default()
        }
    })
}

/// Call `addr` with the integer and floating-point argument registers and nothing on the stack,
/// as `call_as` does for `register_call`'s arguments.
fn call_registers<R>(addr: u64, ints: &[u64; 8], floats: &[f64; 8]) -> R {
    let [i0, i1, i2, i3, i4, i5, i6, i7] = *ints;
    let [f0, f1, f2, f3, f4, f5, f6, f7] = *floats;
    #[cfg(target_arch = "x86_64")]
    {
        let _ = (i6, i7);
        type Proto<R> = unsafe extern "C" fn(u64, ...) -> R;
        // SAFETY: `addr` is a C function the program declared (see `call_as`), whose scalar
        // arguments are these registers; the variadic form also sets `al`.
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        // SAFETY: as above.
        unsafe { f(i0, i1, i2, i3, i4, i5, f0, f1, f2, f3, f4, f5, f6, f7) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        #[rustfmt::skip]
        type Proto<R> = unsafe extern "C" fn(
            u64, u64, u64, u64, u64, u64, u64, u64,
            f64, f64, f64, f64, f64, f64, f64, f64,
        ) -> R;
        // SAFETY: as above.
        let f: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        // SAFETY: as above.
        unsafe {
            f(
                i0, i1, i2, i3, i4, i5, i6, i7, f0, f1, f2, f3, f4, f5, f6, f7,
            )
        }
    }
}

/// A call whose result is a scalar (or nothing).
fn scalar_call(addr: u64, regs: &Regs, ret: Option<Ty>) -> Rets {
    match ret {
        Some(t) if t.is_float() => {
            let bits = call_as::<f64>(addr, regs).to_bits();
            Rets::one(if t == Ty::F32 {
                bits & 0xffff_ffff
            } else {
                bits
            })
        }
        Some(t) => Rets::one(mask_int(t, call_as::<u64>(addr, regs))),
        None => {
            call_as::<u64>(addr, regs);
            Rets::default()
        }
    }
}

/// An integer result register narrowed to its type.
fn mask_int(t: Ty, r: u64) -> u64 {
    match t {
        Ty::I8 => r & 0xff,
        Ty::I16 => r & 0xffff,
        Ty::I32 => r & 0xffff_ffff,
        _ => r,
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn call(_addr: u64, _args: &[u64], _sig: &Sig) -> Result<Rets, String> {
    Err("native foreign calls are not available on this platform".into())
}

/// Nothing to precompute where native calls are unavailable.
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub struct Plan;

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
impl Plan {
    pub fn new(_sig: &Sig) -> Plan {
        Plan
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn call_planned(addr: u64, args: &[u64], sig: &Sig, _plan: &Plan) -> Result<Rets, String> {
    call(addr, args, sig)
}

#[cfg(all(test, target_os = "linux"))]
mod library_name_tests {
    use super::Library;

    /// A versioned soname is a complete file name and loads as written.
    #[test]
    fn versioned_soname_opens_verbatim() {
        assert!(Library::open("libm.so.6", true, "").is_some());
    }
}
