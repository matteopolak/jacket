//! The code generation backend as a shared library.
//!
//! `jaic` built without LLVM opens this library the first time a build writes native code
//! (`jaic_native::backend`). It exports two functions: the stamp that says which sources and
//! compiler built it, and the emit entry point. Everything else stays inside; see
//! `docs/compiler/lazy-backend.md`.
#![allow(unsafe_code)]

use std::ffi::c_void;

/// The bytes of [`jaic_native::backend::stamp`]; `len` receives their count.
///
/// # Safety
/// `len` is a valid pointer to write one `usize` to.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jaic_backend_stamp(len: *mut usize) -> *const u8 {
    let stamp = jaic_native::backend::stamp();

    // SAFETY: the caller passes a writable `usize`.
    unsafe { *len = stamp.len() };
    stamp.as_ptr()
}

/// Write the objects a request asks for; false (with a failure reported) when it cannot.
///
/// # Safety
/// `request` points to the `EmitRequest` of the driver whose stamp matched this library's.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jaic_backend_emit(request: *mut c_void) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe { jaic_native::backend::serve(request, jaic_llvm::emit_object, jaic_llvm::emit_objects) }
}
