//! Loader-owned provider identity, separate from the process's interposable symbol bindings.
#![allow(unsafe_code)]

#[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
#[repr(C)]
struct DlInfo {
    filename: *const std::ffi::c_char,
    base: *mut std::ffi::c_void,
    symbol: *const std::ffi::c_char,
    symbol_address: *mut std::ffi::c_void,
}

#[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
unsafe extern "C" {
    fn dlclose(handle: *mut std::ffi::c_void) -> i32;
    fn dlerror() -> *const std::ffi::c_char;
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn dladdr(address: *const std::ffi::c_void, info: *mut DlInfo) -> i32;
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
unsafe extern "C" {
    fn dladdr1(
        address: *const std::ffi::c_void,
        info: *mut DlInfo,
        extra: *mut *mut std::ffi::c_void,
        flags: i32,
    ) -> i32;

    fn dlinfo(
        handle: *mut std::ffi::c_void,
        request: i32,
        output: *mut *mut std::ffi::c_void,
    ) -> i32;
}

#[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
fn empty_info() -> DlInfo {
    DlInfo {
        filename: std::ptr::null(),
        base: std::ptr::null_mut(),
        symbol: std::ptr::null(),
        symbol_address: std::ptr::null_mut(),
    }
}

#[cfg(target_os = "macos")]
fn kernel_image() -> Option<usize> {
    static IMAGE: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *IMAGE.get_or_init(|| {
        type ImageHeader = unsafe extern "C" fn(*mut std::ffi::c_void) -> *const std::ffi::c_void;
        // The optional dyld handle-to-image query exists from macOS 13. Resolve it dynamically;
        // older loaders keep the conservative scheduling policy, with no deployment-target bump.
        // SAFETY: the symbol is queried only in this live native process.
        let query = unsafe {
            super::super::sys::dlsym(
                super::super::sys::RTLD_DEFAULT,
                c"_dyld_get_dlopen_image_header".as_ptr(),
            )
        };
        if query.is_null() {
            return None;
        }
        // SAFETY: the loader's query has exactly this SDK signature.
        let image_header: ImageHeader = unsafe { std::mem::transmute(query) };
        const RTLD_NOLOAD: i32 = 0x10;
        // SAFETY: inspect an already loaded canonical kernel-library handle; no new code or
        // constructor runs. Its loader-owned Mach header is the identity, not the path string.
        let handle = unsafe {
            super::super::sys::dlopen(
                c"/usr/lib/system/libsystem_kernel.dylib".as_ptr(),
                super::super::sys::RTLD_NOW | RTLD_NOLOAD,
            )
        };
        if handle.is_null() {
            return None;
        }
        // SAFETY: the live handle is the one returned by dyld above.
        let image = unsafe { image_header(handle) };
        if image.is_null() {
            // SAFETY: release only this failed inspection's live handle reference.
            unsafe { dlclose(handle) };
            return None;
        }
        // Keep the handle reference alive, matching native::Library's process-long ownership.
        Some(image as usize)
    })
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
fn libc_map() -> Option<usize> {
    static MAP: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *MAP.get_or_init(|| {
        // GNU dlfcn.h: RTLD_NOLOAD=4, RTLD_DI_LINKMAP=2. Only inspect the already loaded libc;
        // never load a new library or run constructors from a scheduling query.
        const RTLD_NOLOAD: i32 = 4;
        const RTLD_DI_LINKMAP: i32 = 2;
        // SAFETY: a fixed NUL-terminated SONAME and valid loader flags.
        let handle = unsafe {
            super::super::sys::dlopen(
                c"libc.so.6".as_ptr(),
                super::super::sys::RTLD_NOW | RTLD_NOLOAD,
            )
        };
        if handle.is_null() {
            return None;
        }
        let mut map = std::ptr::null_mut();
        // SAFETY: the live loader handle and writable pointer match RTLD_DI_LINKMAP's ABI.
        if unsafe { dlinfo(handle, RTLD_DI_LINKMAP, &raw mut map) } != 0 || map.is_null() {
            // SAFETY: release only our unsuccessful inspection's reference.
            unsafe { dlclose(handle) };
            return None;
        }
        // Keep the reference alive, like native::Library: the loader-owned map must not disappear.
        Some(map as usize)
    })
}

pub(super) fn trusted(address: u64) -> bool {
    // Loader introspection must not replace a guest's errno/last-error value.
    let _error = crate::interp::last_error::keep();
    #[cfg(target_os = "macos")]
    {
        let mut info = empty_info();
        // SAFETY: dladdr only queries the address; info is a complete writable SDK structure.
        if unsafe { dladdr(address as *const _, &raw mut info) } == 0 || info.filename.is_null() {
            return false;
        }
        kernel_image().is_some_and(|canonical| info.base as usize == canonical)
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        let Some(canonical) = libc_map() else {
            return false;
        };
        const RTLD_DL_LINKMAP: i32 = 2;
        let mut info = empty_info();
        let mut actual = std::ptr::null_mut();
        // SAFETY: GNU dladdr1 writes a loader-owned link-map pointer for this request.
        let found = unsafe {
            dladdr1(
                address as *const _,
                &raw mut info,
                &raw mut actual,
                RTLD_DL_LINKMAP,
            )
        };
        found != 0 && actual as usize == canonical
    }
    #[cfg(not(any(target_os = "macos", all(target_os = "linux", target_env = "gnu"))))]
    {
        let _ = address;
        false
    }
}

/// Discard only errors from the one-time inspection, before any guest code has run.
pub(super) fn clear_inspection_error() {
    #[cfg(any(target_os = "macos", all(target_os = "linux", target_env = "gnu")))]
    // SAFETY: read and clear the calling thread's loader-owned error slot.
    unsafe {
        dlerror()
    };
}
