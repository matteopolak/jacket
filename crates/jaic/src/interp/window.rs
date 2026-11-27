//! The address space of a program whose pointers are narrower than the host's.
//!
//! A target with 4-byte pointers cannot use host addresses as program addresses. The
//! interpreter reserves one contiguous host region and treats a program address as the offset
//! of a host address from the region's start. Everything the program can point at (stacks,
//! globals, read-only data, the heap the library stand-ins hand out) is carved from it, so
//! every program address fits in the target's pointer. Offsets below `FIRST` are never mapped,
//! which keeps null and small addresses faulting as on a native target. The top of the 32-bit
//! range holds procedure addresses (`interp::FUNC_BASE`), which no load or store may reach.
//!
//! A target with 2-byte pointers (AVR) gets a 64 KiB window: the top 16 KiB hold procedure
//! addresses and the first 256 bytes are never mapped. It has no read-only region, so read-only
//! data lives in ordinary memory, as it does in the RAM of the real part.
use super::host_pages::{self, Access};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Bytes of address space a window maps. The rest of the 32-bit range holds procedure addresses.
pub(super) const SPAN: usize = 0xC000_0000;

/// The same for a program with 2-byte pointers.
pub(super) const SPAN16: usize = 0xC000;

/// Offsets below this are never mapped.
const FIRST: usize = 64 << 10;

/// The same for a program with 2-byte pointers.
const FIRST16: usize = 0x100;

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
    /// Bytes of address space mapped, and the first offset ever handed out.
    span: usize,
    first: usize,
    /// Bytes at the start kept for read-only data; 0 when the window has none.
    read_only: usize,
    /// The program's pointer size in bytes.
    pointer_size: u8,
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
    /// Reserve a window for a program with `pointer_size`-byte pointers (2 or 4); `None` when
    /// the host has no room for it or its pointers are too narrow to hold one.
    pub(crate) fn new(pointer_size: u8) -> Option<Arc<Window>> {
        if usize::BITS < 64 {
            return None;
        }
        let (span, first, read_only) = if pointer_size == 2 {
            (SPAN16, FIRST16, 0)
        } else {
            (SPAN, FIRST, READ_ONLY)
        };
        let base = host_pages::reserve(span)?;
        let heap = first + read_only;
        Some(Arc::new(Window {
            base,
            span,
            first,
            read_only,
            pointer_size,
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
    pub(crate) fn shared(pointer_size: u8) -> Option<Arc<Window>> {
        static SHARED: std::sync::OnceLock<Option<Arc<Window>>> = std::sync::OnceLock::new();
        static SHARED16: std::sync::OnceLock<Option<Arc<Window>>> = std::sync::OnceLock::new();
        let cell = if pointer_size == 2 {
            &SHARED16
        } else {
            &SHARED
        };
        cell.get_or_init(|| Window::new(pointer_size)).clone()
    }

    /// Whether program address `p` can never be memory of this window: below the first offset
    /// handed out or past the mapped span, where only integers cast to pointers and procedure
    /// addresses are.
    pub(crate) fn never_memory(&self, p: u64) -> bool {
        p < self.first as u64 || p >= self.span as u64
    }

    /// The program's pointer size in bytes.
    pub(crate) fn pointer_size(&self) -> u8 {
        self.pointer_size
    }

    /// The host address of offset 0.
    pub(crate) fn base(&self) -> u64 {
        self.base as u64
    }

    /// A region of the host range kept for read-only data: its slot, start and length. `None`
    /// when every interpreter has one.
    pub(super) fn claim_read_only(&self) -> Option<(usize, usize, usize)> {
        if self.read_only == 0 {
            return None;
        }
        let mut state = self.state.lock().unwrap();
        let slot = state.slots.iter().position(|taken| !taken)?;
        state.slots[slot] = true;
        Some((slot, self.base + self.first + slot * SLOT, SLOT))
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
        if end > self.span {
            return None;
        }
        if end > state.committed {
            let to = end.next_multiple_of(STEP).min(self.span);
            // A window whose first block starts inside a page (the 2-byte one) commits from that
            // page's start; pages committed already stay as they are.
            let from = state.committed & !(host_pages::page_size() - 1);
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
        host_pages::release(self.base, self.span);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_are_zeroed_aligned_and_reused() {
        let window = Window::new(4).expect("a window");
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

    /// A 2-byte program's window is 64 KiB less its procedure addresses: every block lies below
    /// `SPAN16`, none in the first 256 bytes, and it has no read-only region.
    #[test]
    fn a_16_bit_window_keeps_every_block_in_16_bits() {
        let window = Window::new(2).expect("a window");
        assert_eq!(window.pointer_size(), 2);
        assert!(window.claim_read_only().is_none());
        let a = window.alloc(100, 1).unwrap();
        assert!(a as usize >= FIRST16);
        let mut last = a;
        while let Some(at) = window.alloc(1024, 1) {
            last = at;
        }
        assert!(last as usize + 1024 <= SPAN16);
        assert!(window.alloc(SPAN16, 16).is_none());
        // SAFETY: the block is committed and ours.
        unsafe { ((window.base() + last) as *mut u8).add(1023).write(1) };
    }

    #[test]
    fn freed_neighbours_merge() {
        let window = Window::new(4).expect("a window");
        let a = window.alloc(64, 16).unwrap();
        let b = window.alloc(64, 16).unwrap();
        window.free(a, 64);
        window.free(b, 64);
        assert_eq!(window.alloc(128, 16), Some(a));
    }

    #[test]
    fn every_byte_of_a_block_is_writable() {
        let window = Window::new(4).expect("a window");
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
