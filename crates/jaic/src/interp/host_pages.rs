//! Reserving, committing and protecting the host's own pages.
//!
//! A reservation is address space with no access; committing makes part of it usable with the
//! given access, and its pages read zero the first time. Unix commits by changing protection,
//! since its pages are made on first touch anyway; Windows commits explicitly.
#![allow(unsafe_code)]

/// What code may do with committed pages. Nothing here makes pages executable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Access {
    None,
    Read,
    ReadWrite,
}

pub(super) use sys::{commit, page_size, protect, release, reserve};

/// The alignment of every reservation's start.
pub(super) fn granularity() -> usize {
    if cfg!(windows) {
        64 * 1024
    } else {
        page_size()
    }
}

#[cfg(unix)]
mod sys {
    use super::Access;
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
        fn getpagesize() -> i32;
    }

    // MAP_PRIVATE | MAP_ANON | MAP_NORESERVE (Linux; macOS does not reserve swap anyway).
    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0x0002 | 0x1000;

    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x0002 | 0x0020 | 0x4000;

    fn prot(access: Access) -> i32 {
        match access {
            Access::None => 0,
            Access::Read => 1,
            Access::ReadWrite => 3,
        }
    }

    pub(in crate::interp) fn page_size() -> usize {
        // SAFETY: a process-local configuration query with no arguments.
        (unsafe { getpagesize() }) as usize
    }

    /// `len` bytes of fresh address space with no access, or `None` when the host has none.
    pub(in crate::interp) fn reserve(len: usize) -> Option<usize> {
        // SAFETY: an anonymous mapping of fresh address space.
        let base = unsafe { mmap(std::ptr::null_mut(), len, 0, FLAGS, -1, 0) };
        (base as isize != -1).then_some(base as usize)
    }

    /// Make `base..base + len`, page aligned and inside a reservation, usable with `access`.
    pub(in crate::interp) fn commit(base: usize, len: usize, access: Access) -> bool {
        protect(base, len, access)
    }

    /// Change the access of committed pages `base..base + len`.
    pub(in crate::interp) fn protect(base: usize, len: usize, access: Access) -> bool {
        // SAFETY: the caller owns `base..base + len`, a page-aligned range of a reservation.
        unsafe { mprotect(base as *mut c_void, len, prot(access)) == 0 }
    }

    /// Give back a whole reservation; `len` is its length from `reserve`.
    pub(in crate::interp) fn release(base: usize, len: usize) {
        // SAFETY: the caller owns the reservation and releases it once.
        unsafe { munmap(base as *mut c_void, len) };
        crate::interp::probe::invalidate();
    }
}

#[cfg(windows)]
mod sys {
    use super::Access;
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn VirtualAlloc(addr: *mut c_void, len: usize, kind: u32, protect: u32) -> *mut c_void;
        fn VirtualProtect(addr: *mut c_void, len: usize, protect: u32, old: *mut u32) -> i32;
        fn VirtualFree(addr: *mut c_void, len: usize, kind: u32) -> i32;
    }

    const COMMIT: u32 = 0x1000;
    const RESERVE: u32 = 0x2000;
    const RELEASE: u32 = 0x8000;

    fn prot(access: Access) -> u32 {
        match access {
            Access::None => 0x01,
            Access::Read => 0x02,
            Access::ReadWrite => 0x04,
        }
    }

    pub(in crate::interp) fn page_size() -> usize {
        4096
    }

    pub(in crate::interp) fn reserve(len: usize) -> Option<usize> {
        // SAFETY: fresh address space, not yet backed.
        let base = unsafe { VirtualAlloc(std::ptr::null_mut(), len, RESERVE, prot(Access::None)) };
        (!base.is_null()).then_some(base as usize)
    }

    pub(in crate::interp) fn commit(base: usize, len: usize, access: Access) -> bool {
        // Committing pages that are committed already keeps their protection, so set it after.
        // SAFETY: a page-aligned range inside a reservation the caller owns.
        !unsafe { VirtualAlloc(base as *mut c_void, len, COMMIT, prot(access)) }.is_null()
            && protect(base, len, access)
    }

    pub(in crate::interp) fn protect(base: usize, len: usize, access: Access) -> bool {
        let mut old = 0u32;
        // SAFETY: committed pages of a reservation the caller owns, at a page boundary.
        unsafe { VirtualProtect(base as *mut c_void, len, prot(access), &raw mut old) != 0 }
    }

    pub(in crate::interp) fn release(base: usize, _len: usize) {
        // SAFETY: the caller owns the reservation and releases it once.
        unsafe { VirtualFree(base as *mut c_void, 0, RELEASE) };
        crate::interp::probe::invalidate();
    }
}

/// Hosts without page protection (wasm32): there is nothing to reserve.
#[cfg(not(any(unix, windows)))]
mod sys {
    use super::Access;

    pub(in crate::interp) fn page_size() -> usize {
        4096
    }

    pub(in crate::interp) fn reserve(_len: usize) -> Option<usize> {
        None
    }

    pub(in crate::interp) fn commit(_base: usize, _len: usize, _access: Access) -> bool {
        false
    }

    pub(in crate::interp) fn protect(_base: usize, _len: usize, _access: Access) -> bool {
        false
    }

    pub(in crate::interp) fn release(_base: usize, _len: usize) {
    }
}
