//! The address space of a program whose pointers are narrower than the host's.
//!
//! A target with 4-byte pointers cannot use host addresses as program addresses. The
//! interpreter reserves one contiguous host region and treats a program address as the offset
//! of a host address from the region's start. Everything the program can point at (stacks,
//! globals, read-only data, the heap the library stand-ins hand out) is carved from it, so
//! every program address fits in the target's pointer. Offsets below `FIRST` are never mapped,
//! which keeps null and small addresses faulting as on a native target. The top of the 32-bit
//! range holds procedure addresses (`interp::FUNC_BASE`), which no load or store may reach.
use super::host_pages::{self, Access};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Bytes of address space a window maps. The rest of the 32-bit range holds procedure addresses.
pub(super) const SPAN: usize = 0xC000_0000;

/// Offsets below this are never mapped.
const FIRST: usize = 64 << 10;

/// Bytes at the start of the window kept for read-only data (`read_only.rs`): `SLOTS` regions,
/// one for each interpreter using the window.
pub(super) const READ_ONLY: usize = 256 << 20;

/// Bytes of read-only data one interpreter can have; more goes to ordinary memory.
const SLOT: usize = 16 << 20;

/// How many interpreters can have read-only data at once.
const SLOTS: usize = READ_ONLY / SLOT;

/// Memory is committed in steps of this many bytes: a multiple of every host's page size and of
/// Windows' allocation granularity.
const STEP: usize = 1 << 20;

/// The window of one interpreter, shared by the blocks carved from it.
pub struct Window {
    base: usize,
    state: Mutex<State>,
}

struct State {
    /// Offset where the next fresh block starts.
    bump: usize,
    /// Offset up to which the heap area is committed.
    committed: usize,
    /// Freed blocks: offset to length.
    free: BTreeMap<usize, usize>,
    /// Which read-only regions an interpreter holds.
    slots: [bool; SLOTS],
}

impl Window {
    /// Reserve a window; `None` when the host has no room for it or its pointers are too narrow
    /// to hold one.
    pub(crate) fn new() -> Option<Arc<Window>> {
        if usize::BITS < 64 {
            return None;
        }
        let base = host_pages::reserve(SPAN)?;
        let heap = FIRST + READ_ONLY;
        Some(Arc::new(Window {
            base,
            state: Mutex::new(State {
                bump: heap,
                committed: heap,
                free: BTreeMap::new(),
                slots: [false; SLOTS],
            }),
        }))
    }

    /// The window every interpreter of a narrow-pointer program shares, so that a host the
    /// interpreters share (the sandbox) hands each of them addresses valid in all. Made on first
    /// use and kept for the life of the process.
    pub(crate) fn shared() -> Option<Arc<Window>> {
        static SHARED: std::sync::OnceLock<Option<Arc<Window>>> = std::sync::OnceLock::new();
        SHARED.get_or_init(Window::new).clone()
    }

    /// The host address of offset 0.
    pub(crate) fn base(&self) -> u64 {
        self.base as u64
    }

    /// A region of the host range kept for read-only data: its slot, start and length. `None`
    /// when every interpreter has one.
    pub(super) fn claim_read_only(&self) -> Option<(usize, usize, usize)> {
        let mut state = self.state.lock().unwrap();
        let slot = state.slots.iter().position(|taken| !taken)?;
        state.slots[slot] = true;
        Some((slot, self.base + FIRST + slot * SLOT, SLOT))
    }

    /// Give back the region `claim_read_only` made.
    pub(super) fn release_read_only(&self, slot: usize) {
        self.state.lock().unwrap().slots[slot] = false;
    }

    /// `size` zeroed bytes aligned to `align` (at most a page), as an offset; `None` when the
    /// window is full or the host cannot back it.
    pub(crate) fn alloc(&self, size: usize, align: usize) -> Option<u64> {
        let size = size.max(1).checked_next_multiple_of(16)?;
        let align = align.max(16);
        let mut state = self.state.lock().unwrap();
        let reused = state.free.iter().find_map(|(&at, &len)| {
            let start = at.next_multiple_of(align);
            (start + size <= at + len).then_some((at, len, start))
        });
        if let Some((at, len, start)) = reused {
            state.free.remove(&at);
            if start > at {
                state.free.insert(at, start - at);
            }
            if start + size < at + len {
                state.free.insert(start + size, at + len - start - size);
            }
            // SAFETY: `start..start + size` is committed window memory that nothing owns.
            unsafe { std::ptr::write_bytes((self.base + start) as *mut u8, 0, size) };
            return Some(start as u64);
        }
        let start = state.bump.next_multiple_of(align);
        let end = start.checked_add(size)?;
        if end > SPAN {
            return None;
        }
        if end > state.committed {
            let to = end.next_multiple_of(STEP).min(SPAN);
            let from = state.committed;
            if !host_pages::commit(self.base + from, to - from, Access::ReadWrite) {
                return None;
            }
            state.committed = to;
        }
        state.bump = end;
        Some(start as u64)
    }

    /// `len` bytes of whole pages, page aligned, for a mapping the program asked for.
    pub(crate) fn alloc_pages(&self, len: usize) -> Option<u64> {
        self.alloc(len, host_pages::page_size())
    }

    /// Give back pages `alloc_pages` made, whatever protection the program gave them.
    pub(crate) fn free_pages(&self, at: u64, len: usize) {
        host_pages::protect(self.base + at as usize, len, Access::ReadWrite);
        self.free(at, len);
    }

    /// Give back a block `alloc` made (its offset and the size it was asked for).
    pub(crate) fn free(&self, at: u64, size: usize) {
        let size = size.max(1).next_multiple_of(16);
        let mut state = self.state.lock().unwrap();
        let mut at = at as usize;
        let mut len = size;
        if let Some((&before, &before_len)) = state.free.range(..at).next_back()
            && before + before_len == at
        {
            state.free.remove(&before);
            at = before;
            len += before_len;
        }
        if let Some(after_len) = state.free.remove(&(at + len)) {
            len += after_len;
        }
        state.free.insert(at, len);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        host_pages::release(self.base, SPAN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_zeroed_aligned_and_reused() {
        let window = Window::new().expect("a window");
        let a = window.alloc(100, 64).unwrap();
        assert_eq!(a % 64, 0);
        assert!(a as usize >= FIRST + READ_ONLY);
        let host = (window.base() + a) as *mut u8;
        // SAFETY: the block is committed and ours.
        unsafe { host.write(7) };
        window.free(a, 100);
        let b = window.alloc(100, 64).unwrap();
        assert_eq!(a, b, "a freed block is reused");
        // SAFETY: as above.
        assert_eq!(unsafe { ((window.base() + b) as *const u8).read() }, 0);
        assert!(window.alloc(SPAN, 16).is_none());
    }

    #[test]
    fn freed_neighbours_merge() {
        let window = Window::new().expect("a window");
        let a = window.alloc(64, 16).unwrap();
        let b = window.alloc(64, 16).unwrap();
        window.free(a, 64);
        window.free(b, 64);
        assert_eq!(window.alloc(128, 16), Some(a));
    }

    #[test]
    fn every_byte_of_a_block_is_writable() {
        let window = Window::new().expect("a window");
        for size in [1usize, 100, 5000, 32 << 20, 3 << 20, 7] {
            let at = window.alloc(size, 4096).unwrap();
            let host = (window.base() + at) as *mut u8;
            // SAFETY: the block is committed and ours.
            unsafe {
                host.write(1);
                host.add(size - 1).write(2);
            }
        }
    }
}
