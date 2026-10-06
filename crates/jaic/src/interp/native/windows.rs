//! Native foreign calls on Windows: DLL loading (x64 and arm64) and the Microsoft x64
//! convention. Windows on arm64 calls through the AAPCS64 path in `native.rs`.
//!
//! x64 arguments occupy positional 8-byte slots: the first four in RCX/RDX/R8/R9 or, for
//! floating-point ones, XMM0-XMM3; the rest on the stack. Calls go through a C-variadic
//! prototype whose variadic doubles the caller places in both the XMM and the integer
//! register of their slot, so the callee finds each of the first four arguments wherever its
//! own signature reads it. Only the first slot is a fixed parameter: two prototypes cover it
//! (integer or float). Aggregates follow `abi::classify_arg(Arch::Win64)`: 1, 2, 4 or 8 bytes
//! in one slot, anything else as a pointer to a copy; non-register results come back through
//! a hidden pointer in the first slot.
#[cfg(target_arch = "x86_64")]
use super::{read_bytes, write_bytes};
#[cfg(target_arch = "x86_64")]
use crate::abi::{self, Arch, Passing};
#[cfg(target_arch = "x86_64")]
use crate::ir::{Sig, Ty};
use std::ffi::c_void;
use std::sync::Mutex;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

/// Every module opened so far, searched (after the defaults) for symbols that name no
/// library, like `dlsym(RTLD_DEFAULT)`.
static OPENED: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// The C runtime and Win32 core: where library-less foreign symbols and the stdlib's `libc`
/// names resolve.
const DEFAULT_MODULES: [&str; 4] = ["msvcrt.dll", "ucrtbase.dll", "kernel32.dll", "ntdll.dll"];

fn load(path: &str) -> Option<usize> {
    let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
    // SAFETY: `wide` is NUL-terminated.
    let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
    if handle.is_null() {
        return None;
    }
    let handle = handle as usize;
    if let Ok(mut opened) = OPENED.lock()
        && !opened.contains(&handle)
    {
        opened.push(handle);
    }
    Some(handle)
}

/// Open a library by Jai name: `"kernel32"`, `"msvcrt"`, a path next to the source...
pub fn open(name: &str, system: bool, base_dir: &str) -> Option<usize> {
    let lower = name.to_ascii_lowercase();
    if system && matches!(lower.as_str(), "c" | "libc" | "m" | "libm" | "msvcrt") {
        return load("msvcrt.dll");
    }
    let path = std::path::Path::new(name);
    let file = path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut candidates = Vec::new();
    if !system {
        let dir = std::path::Path::new(base_dir).join(path.parent().unwrap_or("".as_ref()));
        candidates.push(dir.join(format!("{file}.dll")).display().to_string());
        candidates.push(dir.join(format!("lib{file}.dll")).display().to_string());
    }
    let bare = name.strip_prefix("lib").unwrap_or(name);
    for d in super::library_dirs() {
        candidates.push(d.join(format!("{bare}.dll")).display().to_string());
    }
    candidates.push(format!("{name}.dll"));
    candidates.push(format!("{bare}.dll"));
    candidates.into_iter().find_map(|c| load(&c))
}

/// Resolve `symbol` in `lib`, then in the default modules and everything opened so far.
pub fn lookup(lib: Option<usize>, symbol: &str) -> Option<u64> {
    let name: Vec<u8> = symbol.bytes().chain(Some(0)).collect();
    let find = |module: usize| {
        // SAFETY: `module` is a loaded module handle and `name` is NUL-terminated.
        let p = unsafe { GetProcAddress(module as *mut c_void, name.as_ptr()) };
        (!p.is_null()).then_some(p as u64)
    };
    if let Some(found) = lib.and_then(find) {
        return Some(found);
    }
    let defaults: Vec<usize> = DEFAULT_MODULES.iter().filter_map(|m| load(m)).collect();
    let opened = OPENED.lock().map(|o| o.clone()).unwrap_or_default();
    defaults.into_iter().chain(opened).find_map(find)
}

/// Positional argument slots the prototype passes: four register slots and 16 stack slots.
#[cfg(target_arch = "x86_64")]
const SLOTS: usize = 20;

/// Call `addr` with `slots`, reading the result as `R` (`u64` from RAX, `f64` from XMM0).
///
/// SAFETY: `addr` is a C function whose parameters fit `slots` positionally.
#[cfg(target_arch = "x86_64")]
unsafe fn call_slots<R>(addr: u64, slots: &[u64; SLOTS], first_float: bool) -> R {
    let f = |i: usize| f64::from_bits(slots[i]);
    let [
        _,
        _,
        _,
        _,
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
        s16,
        s17,
        s18,
        s19,
    ] = *slots;
    if first_float {
        type Proto<R> = unsafe extern "C" fn(f64, ...) -> R;
        let p: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        unsafe {
            p(
                f(0),
                f(1),
                f(2),
                f(3),
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
                s16,
                s17,
                s18,
                s19,
            )
        }
    } else {
        type Proto<R> = unsafe extern "C" fn(u64, ...) -> R;
        let p: Proto<R> = unsafe { std::mem::transmute::<usize, Proto<R>>(addr as usize) };
        unsafe {
            p(
                slots[0],
                f(1),
                f(2),
                f(3),
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
                s16,
                s17,
                s18,
                s19,
            )
        }
    }
}

/// `super::call_with` for the Microsoft x64 convention.
#[cfg(target_arch = "x86_64")]
pub fn call(addr: u64, args: &[u64], sig: &Sig) -> Result<Vec<u64>, String> {
    let cabi = sig.c_abi.as_deref();
    let ret_layout = cabi.and_then(|c| c.ret.as_ref());
    let forced_sret = cabi.is_some_and(|c| c.ret_indirect);
    let ret_pieces = ret_layout
        .filter(|_| !forced_sret)
        .and_then(|l| abi::classify_ret(Arch::Win64, l));
    let out_index = ret_layout.map(|_| sig.params.len().wrapping_sub(1));
    let out_ptr = out_index.and_then(|i| args.get(i).copied()).unwrap_or(0);
    let mut slots: Vec<u64> = Vec::new();
    let mut first_float = false;
    // The hidden result pointer is the first argument.
    if ret_layout.is_some() && ret_pieces.is_none() {
        slots.push(out_ptr);
    }
    // Copies of aggregates passed by reference; alive until the call returns.
    let mut copies: Vec<Vec<u64>> = Vec::new();
    for (i, &a) in args.iter().enumerate() {
        if Some(i) == out_index {
            continue;
        }
        let layout = cabi.and_then(|c| c.params.get(i)).and_then(Option::as_ref);
        let Some(layout) = layout else {
            let ty = sig.params.get(i).copied().unwrap_or(Ty::I64);
            if slots.is_empty() && ty.is_float() {
                first_float = true;
            }
            slots.push(a);
            continue;
        };
        match abi::classify_arg(Arch::Win64, layout) {
            Passing::Registers(pieces) if pieces.is_empty() => {}
            // SAFETY: `a` points at the aggregate, `layout.size` (<= 8) bytes long.
            Passing::Registers(_) => slots.push(unsafe { read_bytes(a, layout.size) }),
            Passing::Indirect | Passing::ByVal => {
                let mut copy = vec![0u64; layout.size.div_ceil(8) as usize];
                // SAFETY: `a` points at the aggregate; the copy is at least as long.
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        a as *const u8,
                        copy.as_mut_ptr() as *mut u8,
                        layout.size as usize,
                    )
                };
                slots.push(copy.as_ptr() as u64);
                copies.push(copy);
            }
        }
    }
    if slots.len() > SLOTS {
        return Err(format!(
            "the interpreter passes at most {SLOTS} arguments to a foreign procedure"
        ));
    }
    let mut padded = [0u64; SLOTS];
    padded[..slots.len()].copy_from_slice(&slots);
    // SAFETY (all calls): the callee's declared C signature matches these slots.
    let result = match (ret_layout, ret_pieces) {
        (Some(_), None) => {
            unsafe { call_slots::<u64>(addr, &padded, first_float) };
            Vec::new()
        }
        (Some(layout), Some(pieces)) => {
            let value = unsafe { call_slots::<u64>(addr, &padded, first_float) };
            if !pieces.is_empty() {
                // SAFETY: the out-pointer addresses `layout.size` writable bytes.
                unsafe { write_bytes(out_ptr, value, layout.size) };
            }
            Vec::new()
        }
        (None, _) => match sig.returns.first().copied() {
            Some(t) if t.is_float() => {
                let bits = unsafe { call_slots::<f64>(addr, &padded, first_float) }.to_bits();
                vec![if t == Ty::F32 {
                    bits & 0xffff_ffff
                } else {
                    bits
                }]
            }
            Some(t) => {
                let r = unsafe { call_slots::<u64>(addr, &padded, first_float) };
                vec![match t {
                    Ty::I8 => r & 0xff,
                    Ty::I16 => r & 0xffff,
                    Ty::I32 => r & 0xffff_ffff,
                    _ => r,
                }]
            }
            None => {
                unsafe { call_slots::<u64>(addr, &padded, first_float) };
                Vec::new()
            }
        },
    };
    drop(copies);
    Ok(result)
}
