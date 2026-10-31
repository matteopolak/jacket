//! Read-only program data: string literals and the other globals the IR marks `read_only`.
//!
//! Interpreted code stores through real pointers, so constant data sits in pages the host
//! refuses to write, as in a native build. Without that, `s[0] = #char "z"` on a literal would
//! quietly change every use of it (equal literals share one global). Interpreted stores check
//! the range (`contains`) and fail with an error at the line that made them; the protection
//! also stops foreign code. Pages are opened for writing while the interpreter fills globals in
//! (`fill`) and stay open until the next foreign call or callback return (`seal`), so a run of
//! new globals costs one protection change, not one each.
#![allow(unsafe_code)]

/// Address space reserved for one interpreter's read-only data. Pages cost nothing until
/// used; data beyond this goes to ordinary (writable) memory.
const RESERVE: usize = 256 << 20;

/// Protection is changed and memory committed in steps of this many bytes: a multiple of
/// every host's page size (4 KiB, 16 KiB on Apple silicon) and of Windows' allocation
/// granularity.
const STEP: usize = 64 * 1024;

/// One interpreter's read-only data: a reservation filled from the start, released with it.
#[derive(Default)]
pub(super) struct ReadOnlyData {
    /// Start of the reservation; 0 before the first `place`.
    base: usize,
    /// Bytes from `base` handed out.
    used: usize,
    /// Bytes from `base` usable (committed on Windows).
    committed: usize,
    /// The host refused the reservation: never ask again.
    refused: bool,
    /// Offsets from `base` of pages left writable by `fill` and not yet sealed.
    open: Option<(usize, usize)>,
}

impl ReadOnlyData {
    /// Whether `addr` lies in read-only data. On the path of every interpreted store.
    #[inline(always)]
    pub(super) fn contains(&self, addr: u64) -> bool {
        (addr as usize).wrapping_sub(self.base) < self.used
    }

    /// Whether any of `addr..addr + len` lies in read-only data.
    pub(super) fn overlaps(&self, addr: u64, len: usize) -> bool {
        // `start` wraps when `addr` is below the data, which the range then reaches only when
        // it is longer than the distance up to it.
        let start = (addr as usize).wrapping_sub(self.base);
        self.used > 0 && (start < self.used || start.wrapping_neg() < len)
    }

    /// Room for `size` bytes at `align` (zeroed, not writable), or `None` when the reservation
    /// is full or the host has none to give.
    pub(super) fn place(&mut self, size: usize, align: usize) -> Option<u64> {
        if self.base == 0 {
            if self.refused || align > STEP {
                return None;
            }
            let Some(base) = sys::reserve(RESERVE) else {
                self.refused = true;
                return None;
            };
            self.base = base;
        }
        let start = self.used.checked_next_multiple_of(align)?;
        let end = start.checked_add(size.max(1))?;
        if end > RESERVE || !(self.base + start).is_multiple_of(align) {
            return None;
        }
        if end > self.committed {
            let to = end.next_multiple_of(STEP);
            if !sys::commit(self.base + self.committed, to - self.committed) {
                return None;
            }
            self.committed = to;
        }
        self.used = end;
        Some((self.base + start) as u64)
    }

    /// Run `write` with the pages under `addr..addr + len` writable. They stay writable until
    /// `seal`. Memory outside the reservation is written as it is.
    pub(super) fn fill<R>(&mut self, addr: u64, len: usize, write: impl FnOnce() -> R) -> R {
        if !self.contains(addr) {
            return write();
        }
        let offset = addr as usize - self.base;
        let start = offset / STEP * STEP;
        let end = (offset + len.max(1))
            .next_multiple_of(STEP)
            .min(self.committed);
        match self.open {
            Some((lo, hi)) if lo <= start && end <= hi => {}
            Some((lo, hi)) => {
                // Grow to one span; the pages in between are read-only already or about to
                // be sealed with the rest.
                let (lo, hi) = (lo.min(start), hi.max(end));
                sys::protect(self.base + lo, hi - lo, true);
                self.open = Some((lo, hi));
            }
            None => {
                sys::protect(self.base + start, end - start, true);
                self.open = Some((start, end));
            }
        }
        write()
    }

    /// Make every page `fill` opened read-only again. Called before C code can run.
    #[inline]
    pub(super) fn seal(&mut self) {
        if let Some((lo, hi)) = self.open.take() {
            sys::protect(self.base + lo, hi - lo, false);
        }
    }
}

impl Drop for ReadOnlyData {
    fn drop(&mut self) {
        if self.base != 0 {
            sys::release(self.base, RESERVE);
            // Its pages are unmapped now.
            super::probe::invalidate();
        }
    }
}

#[cfg(unix)]
mod sys {
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

    const READ: i32 = 1;
    const READ_WRITE: i32 = 3;

    // MAP_PRIVATE | MAP_ANON | MAP_NORESERVE (Linux; macOS does not reserve swap anyway).
    #[cfg(target_os = "macos")]
    const FLAGS: i32 = 0x0002 | 0x1000;

    #[cfg(not(target_os = "macos"))]
    const FLAGS: i32 = 0x0002 | 0x0020 | 0x4000;

    /// Zeroed read-only address space, its pages made on first touch.
    pub(super) fn reserve(len: usize) -> Option<usize> {
        // SAFETY: an anonymous mapping of fresh address space.
        let base = unsafe { mmap(std::ptr::null_mut(), len, READ, FLAGS, -1, 0) };
        (base as isize != -1).then_some(base as usize)
    }

    pub(super) fn commit(_base: usize, _len: usize) -> bool {
        true
    }

    pub(super) fn protect(base: usize, len: usize, writable: bool) {
        let prot = if writable {
            READ_WRITE
        } else {
            READ
        };
        // SAFETY: `base..base + len` lies inside the reservation, at a page boundary.
        let status = unsafe { mprotect(base as *mut c_void, len, prot) };
        assert_eq!(
            status, 0,
            "cannot change the protection of read-only program data"
        );
    }

    pub(super) fn release(base: usize, len: usize) {
        // SAFETY: the reservation, released once.
        unsafe { munmap(base as *mut c_void, len) };
    }
}

#[cfg(windows)]
mod sys {
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
    const NO_ACCESS: u32 = 0x01;
    const READ: u32 = 0x02;
    const READ_WRITE: u32 = 0x04;

    pub(super) fn reserve(len: usize) -> Option<usize> {
        // SAFETY: fresh address space, not yet backed.
        let base = unsafe { VirtualAlloc(std::ptr::null_mut(), len, RESERVE, NO_ACCESS) };
        (!base.is_null()).then_some(base as usize)
    }

    /// Back `base..base + len` of the reservation with zeroed read-only pages.
    pub(super) fn commit(base: usize, len: usize) -> bool {
        // SAFETY: a range inside the reservation, at a page boundary.
        !unsafe { VirtualAlloc(base as *mut c_void, len, COMMIT, READ) }.is_null()
    }

    pub(super) fn protect(base: usize, len: usize, writable: bool) {
        let mut old = 0u32;
        let prot = if writable {
            READ_WRITE
        } else {
            READ
        };
        // SAFETY: committed pages of the reservation, at a page boundary.
        let status = unsafe { VirtualProtect(base as *mut c_void, len, prot, &raw mut old) };
        assert_ne!(
            status, 0,
            "cannot change the protection of read-only program data"
        );
    }

    pub(super) fn release(base: usize, _len: usize) {
        // SAFETY: the reservation, released once.
        unsafe { VirtualFree(base as *mut c_void, 0, RELEASE) };
    }
}

/// Hosts without page protection (wasm32): no reservation, so read-only data lives in
/// ordinary memory.
#[cfg(not(any(unix, windows)))]
mod sys {
    pub(super) fn reserve(_len: usize) -> Option<usize> {
        None
    }

    pub(super) fn commit(_base: usize, _len: usize) -> bool {
        false
    }

    pub(super) fn protect(_base: usize, _len: usize, _writable: bool) {
    }

    pub(super) fn release(_base: usize, _len: usize) {
    }
}
