//! Read-only program data: string literals and the other globals the IR marks `read_only`.
//!
//! Interpreted code stores through real pointers, so constant data sits in pages the host
//! refuses to write, as in a native build. Without that, `s[0] = #char "z"` on a literal would
//! quietly change every use of it (equal literals share one global). Interpreted stores check
//! the range (`contains`) and fail with an error at the line that made them; the protection
//! also stops foreign code. Pages are opened for writing while the interpreter fills globals in
//! (`fill`) and stay open until the next foreign call or callback return (`seal`), so a run of
//! new globals costs one protection change, not one each.
use super::host_pages::{self, Access};

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
    /// Bytes of the reservation; 0 before the first `place`.
    limit: usize,
    /// The reservation is a part of the program's window, which owns it: the window and the
    /// slot of it this data holds.
    in_window: Option<(std::sync::Arc<super::Window>, usize)>,
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
    /// Read-only data kept in `len` bytes of the program's window starting at host address
    /// `base`, which the window owns and keeps inaccessible until used.
    pub(super) fn in_window(window: &std::sync::Arc<super::Window>) -> Self {
        let Some((slot, base, len)) = window.claim_read_only() else {
            // Every region is taken: the data goes to ordinary memory.
            let mut none = Self::default();
            none.refused = true;
            return none;
        };
        let mut data = Self::default();
        data.base = base;
        data.limit = len;
        data.in_window = Some((window.clone(), slot));
        data
    }

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
            let Some(base) = host_pages::reserve(RESERVE) else {
                self.refused = true;
                return None;
            };
            self.base = base;
            self.limit = RESERVE;
        }
        let start = self.used.checked_next_multiple_of(align)?;
        let end = start.checked_add(size.max(1))?;
        if end > self.limit || !(self.base + start).is_multiple_of(align) {
            return None;
        }
        if end > self.committed {
            let to = end.next_multiple_of(STEP);
            if !host_pages::commit(
                self.base + self.committed,
                to - self.committed,
                Access::Read,
            ) {
                return None;
            }
            self.committed = to;
        }
        self.used = end;
        Some((self.base + start) as u64)
    }

    /// Forget everything placed, and zero it, for a reservation the window owns and so cannot
    /// give back and take again.
    pub(super) fn clear(&mut self) {
        let (base, used) = (self.base as u64, self.used);
        if used > 0 {
            // SAFETY: `base..base + used` is the committed, placed part of this reservation.
            self.fill(base, used, || unsafe {
                std::ptr::write_bytes(base as *mut u8, 0, used);
            });
            self.seal();
        }
        self.used = 0;
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
                protect(self.base + lo, hi - lo, Access::ReadWrite);
                self.open = Some((lo, hi));
            }
            None => {
                protect(self.base + start, end - start, Access::ReadWrite);
                self.open = Some((start, end));
            }
        }
        write()
    }

    /// Make every page `fill` opened read-only again. Called before C code can run.
    #[inline]
    pub(super) fn seal(&mut self) {
        if let Some((lo, hi)) = self.open.take() {
            protect(self.base + lo, hi - lo, Access::Read);
        }
    }
}

impl Drop for ReadOnlyData {
    fn drop(&mut self) {
        if let Some((window, slot)) = self.in_window.take() {
            // The region goes back zeroed for the next interpreter.
            self.clear();
            window.release_read_only(slot);
        } else if self.base != 0 {
            host_pages::release(self.base, self.limit);
        }
    }
}

fn protect(base: usize, len: usize, access: Access) {
    assert!(
        host_pages::protect(base, len, access),
        "cannot change the protection of read-only program data"
    );
}
