//! A global allocator for the `jailsp` executable that hands large blocks straight back to the
//! operating system when they are freed.
//!
//! The server allocates and frees blocks of megabytes all the time: the syntax tree of an open
//! file while its symbols are built, the token list for a semantic-tokens answer, the response
//! text. The system allocators keep such blocks cached after `free` (macOS keeps hundreds of MiB
//! of them; glibc keeps them until the next trim), so the resident set after a minute of editing
//! a large file was several times the live data. Blocks of `LARGE` bytes or more come from
//! `mmap` and go back with `munmap`; everything else is the system allocator's.
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::ffi::c_void;

/// Blocks at least this big are mapped directly.
const LARGE: usize = 128 * 1024;

/// The mapping granule used to round lengths (the real page size divides it on every target).
const GRANULE: usize = 16 * 1024;

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;

    fn munmap(addr: *mut c_void, len: usize) -> i32;
}

const PROT_READ_WRITE: i32 = 0x1 | 0x2;
const MAP_PRIVATE: i32 = 0x2;

#[cfg(any(target_os = "linux", target_os = "android"))]
const MAP_ANON: i32 = 0x20;

#[cfg(not(any(target_os = "linux", target_os = "android")))]
const MAP_ANON: i32 = 0x1000;

pub struct LargeBlocksToOs;

fn mapped(size: usize, align: usize) -> bool {
    size >= LARGE && align <= GRANULE
}

fn length(size: usize) -> usize {
    size.div_ceil(GRANULE) * GRANULE
}

unsafe impl GlobalAlloc for LargeBlocksToOs {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !mapped(layout.size(), layout.align()) {
            return unsafe { System.alloc(layout) };
        }
        // Fresh anonymous pages are zero, so this also serves `alloc_zeroed`.
        let at = unsafe {
            mmap(
                std::ptr::null_mut(),
                length(layout.size()),
                PROT_READ_WRITE,
                MAP_PRIVATE | MAP_ANON,
                -1,
                0,
            )
        };
        if at as usize == usize::MAX {
            std::ptr::null_mut()
        } else {
            at.cast()
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if mapped(layout.size(), layout.align()) {
            unsafe { self.alloc(layout) }
        } else {
            unsafe { System.alloc_zeroed(layout) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if mapped(layout.size(), layout.align()) {
            unsafe { munmap(ptr.cast(), length(layout.size())) };
        } else {
            unsafe { System.dealloc(ptr, layout) };
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let from = mapped(layout.size(), layout.align());
        let to = mapped(new_size, layout.align());
        if !from && !to {
            return unsafe { System.realloc(ptr, layout, new_size) };
        }
        if from && to && length(layout.size()) == length(new_size) {
            return ptr;
        }
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        let fresh = unsafe { self.alloc(new_layout) };
        if !fresh.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(ptr, fresh, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
        }
        fresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_keep_their_contents_across_the_size_classes() {
        let a = LargeBlocksToOs;
        let sizes = [64, LARGE - 1, LARGE, 3 * LARGE + 5, 64, LARGE * 8];
        unsafe {
            let mut layout = Layout::from_size_align(sizes[0], 8).unwrap();
            let mut at = a.alloc(layout);
            assert!(!at.is_null());
            for (round, &size) in sizes.iter().enumerate() {
                let fill = (round + 1) as u8;
                if round > 0 {
                    // The bytes both sizes have must survive the move.
                    let keep = layout.size().min(size);
                    at = a.realloc(at, layout, size);
                    assert!(!at.is_null());
                    layout = Layout::from_size_align(size, 8).unwrap();
                    assert!(
                        std::slice::from_raw_parts(at, keep)
                            .iter()
                            .all(|&b| b == round as u8)
                    );
                }
                std::ptr::write_bytes(at, fill, size);
            }
            a.dealloc(at, layout);
            // Large blocks arrive zeroed and page aligned enough for big alignments.
            let layout = Layout::from_size_align(LARGE * 2, 4096).unwrap();
            let zeroed = a.alloc_zeroed(layout);
            assert_eq!(zeroed as usize % 4096, 0);
            assert!(
                std::slice::from_raw_parts(zeroed, layout.size())
                    .iter()
                    .all(|&b| b == 0)
            );
            a.dealloc(zeroed, layout);
        }
    }
}
