//! Native foreign calls: dynamic library loading and calling C functions
//! through fixed-shape trampolines.
//!
//! On the supported 64-bit C ABIs (System V x86-64 and AArch64) integer and
//! floating-point arguments are assigned to separate register files, so any
//! signature with at most 8 integer and 8 floating-point arguments can be
//! called through one prototype taking 8 of each.
use crate::ir::{Sig, Ty};

#[derive(Clone)]
pub struct Library {
    handle: usize,
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

type IntFn = unsafe extern "C" fn(
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
) -> u64;
type FloatFn = unsafe extern "C" fn(
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    u64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
) -> f64;

/// Call the C function at `addr`. Arguments are raw IR values classified by `sig`.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub fn call(addr: u64, args: &[u64], sig: &Sig) -> Result<Vec<u64>, String> {
    let mut ints = [0u64; 8];
    let mut floats = [0f64; 8];
    let (mut ni, mut nf) = (0, 0);
    for (i, &a) in args.iter().enumerate() {
        let ty = sig.params.get(i).copied().unwrap_or(Ty::I64);
        if ty.is_float() {
            if nf == 8 {
                return Err("foreign call has more than 8 floating-point arguments".into());
            }
            // An f32 travels in the low half of the register.
            floats[nf] = f64::from_bits(a);
            nf += 1;
        } else {
            if ni == 8 {
                return Err("foreign call has more than 8 integer arguments".into());
            }
            ints[ni] = a;
            ni += 1;
        }
    }
    let [i0, i1, i2, i3, i4, i5, i6, i7] = ints;
    let [f0, f1, f2, f3, f4, f5, f6, f7] = floats;
    match sig.returns.first() {
        Some(t) if t.is_float() => {
            // SAFETY: the address comes from the dynamic linker for a declared foreign procedure.
            let f: FloatFn = unsafe { std::mem::transmute::<usize, FloatFn>(addr as usize) };
            let r = unsafe {
                f(
                    i0, i1, i2, i3, i4, i5, i6, i7, f0, f1, f2, f3, f4, f5, f6, f7,
                )
            };
            let bits = r.to_bits();
            Ok(vec![if *t == Ty::F32 {
                bits & 0xffff_ffff
            } else {
                bits
            }])
        }
        ret => {
            let f: IntFn = unsafe { std::mem::transmute::<usize, IntFn>(addr as usize) };
            let r = unsafe {
                f(
                    i0, i1, i2, i3, i4, i5, i6, i7, f0, f1, f2, f3, f4, f5, f6, f7,
                )
            };
            Ok(match ret {
                Some(t) => vec![match t {
                    Ty::I8 => r & 0xff,
                    Ty::I16 => r & 0xffff,
                    Ty::I32 => r & 0xffff_ffff,
                    _ => r,
                }],
                None => vec![],
            })
        }
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn call(_addr: u64, _args: &[u64], _sig: &Sig) -> Result<Vec<u64>, String> {
    Err("native foreign calls are not available on this platform".into())
}
