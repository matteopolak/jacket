//! Copies between the compiler and program memory that never fault.
//!
//! Interpreted code loads and stores through real pointers, so a program that makes a pointer
//! from an integer and follows it crashes like a native program would. The compiler itself must
//! not: when it reads a `#run` result, a string a metaprogram handed to `add_build_string`, or
//! writes an out-parameter of a `compiler_*` call, the address comes from the program and may
//! be anything. Before such an access touches a page, the kernel is asked whether it can, which
//! reports an unmapped or protected address as an error rather than a signal:
//!
//! - Unix: one byte of the page goes through a per-thread pipe. `write(pipe, addr)` fails with
//!   `EFAULT` when `addr` cannot be read, and `read(pipe, addr)` when it cannot be written.
//! - Windows: `VirtualQuery` says whether the page is committed and allows the access.
//! - wasm32: program memory is the linear memory, so the address only has to lie inside it.
//!
//! Asking costs system calls, and a metaprogram's messages are read a field at a time, so the
//! pages found accessible are remembered until memory may have been released: every foreign
//! call (`free`, `munmap`, or C code that frees) and every interpreter block freed calls
//! `invalidate`.
#![allow(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};

/// The granularity accesses are checked at: the smallest page size of the hosts.
const PAGE: u64 = 4096;

/// Pages remembered per thread and kind of access.
const REMEMBERED: usize = 64;

/// Bumped whenever memory may have been unmapped; remembered pages are good only while it
/// keeps the value they were checked under.
static EPOCH: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct Checked {
    epoch: u64,
    readable: Vec<u64>,
    writable: Vec<u64>,
}

thread_local! {
    static CHECKED: RefCell<Checked> = RefCell::new(Checked::default());
}

thread_local! {
    /// The page `accessible` answered for last, as (epoch, page + 1) for reads and for writes:
    /// the usual access is one more field of the record just read, so it needs no lookup.
    /// `const`, so reading it costs no lazy-initialization check.
    static LAST: [Cell<(u64, u64)>; 2] = const { [Cell::new((0, 0)), Cell::new((0, 0))] };
}

/// Forget every page found accessible: memory may have been released.
pub fn invalidate() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// Copy `out.len()` bytes of program memory at `addr` into `out`. False when any of them cannot
/// be read.
pub fn read(addr: u64, out: &mut [u8]) -> bool {
    if !accessible(addr, out.len(), false) {
        return false;
    }
    if !out.is_empty() {
        // SAFETY: every page of the range was found readable.
        unsafe { std::ptr::copy_nonoverlapping(addr as *const u8, out.as_mut_ptr(), out.len()) };
    }
    true
}

/// Copy `bytes` into program memory at `addr`. False when any of it cannot be written.
pub fn write(addr: u64, bytes: &[u8]) -> bool {
    if !accessible(addr, bytes.len(), true) {
        return false;
    }
    if !bytes.is_empty() {
        // SAFETY: every page of the range was found writable.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()) };
    }
    true
}

/// Can every page of `addr..addr + len` be read (or written)? Asks the kernel about the pages
/// not known already.
fn accessible(addr: u64, len: usize, write: bool) -> bool {
    if len == 0 {
        return true;
    }
    // Addresses below 4 KiB are never mapped (and are how null dereferences show), and a range
    // may not wrap.
    let Some(end) = addr.checked_add(len as u64) else {
        return false;
    };
    if addr < 4096 || end > usize::MAX as u64 {
        return false;
    }
    let epoch = EPOCH.load(Ordering::Relaxed);
    let first = addr / PAGE;
    let single = (end - 1) / PAGE == first;
    if single && LAST.with(|last| last[usize::from(write)].get()) == (epoch, first + 1) {
        return true;
    }
    let ok = CHECKED.with(|checked| {
        let mut checked = checked.borrow_mut();
        if checked.epoch != epoch {
            *checked = Checked {
                epoch,
                ..Checked::default()
            };
        }
        let known = if write {
            &mut checked.writable
        } else {
            &mut checked.readable
        };
        let mut page = addr / PAGE;
        while page * PAGE < end {
            if !known.contains(&page) {
                // A byte of the page inside the range stands for the page.
                if !sys::accessible((page * PAGE).max(addr), write) {
                    return false;
                }
                if known.len() == REMEMBERED {
                    known.remove(0);
                }
                known.push(page);
            }
            page += 1;
        }
        true
    });
    if ok && single {
        LAST.with(|last| last[usize::from(write)].set((epoch, first + 1)));
    }
    ok
}

#[cfg(unix)]
mod sys {
    use std::cell::Cell;
    use std::ffi::{c_int, c_void};

    unsafe extern "C" {
        fn pipe(fds: *mut c_int) -> c_int;
        fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;

        #[link_name = "read"]
        fn sys_read(fd: c_int, buf: *mut c_void, len: usize) -> isize;

        #[link_name = "write"]
        fn sys_write(fd: c_int, buf: *const c_void, len: usize) -> isize;
    }

    const F_SETFD: c_int = 2;
    const FD_CLOEXEC: c_int = 1;
    const F_SETFL: c_int = 4;

    #[cfg(target_os = "linux")]
    const O_NONBLOCK: c_int = 0o4000;

    #[cfg(not(target_os = "linux"))]
    const O_NONBLOCK: c_int = 4;

    thread_local! {
        /// The read and write ends, made on first use. Never closed: one per thread that checks.
        static PIPE: Cell<Option<(c_int, c_int)>> = const { Cell::new(None) };
    }

    fn pipe_fds() -> Option<(c_int, c_int)> {
        if let Some(fds) = PIPE.get() {
            return Some(fds);
        }
        let mut fds = [0 as c_int; 2];
        // SAFETY: `pipe` writes two descriptors into the array; `fcntl` takes plain integers.
        unsafe {
            if pipe(fds.as_mut_ptr()) != 0 {
                return None;
            }
            // Non-blocking, so a pipe left full by a failure can never stall the compiler.
            for fd in fds {
                fcntl(fd, F_SETFD, FD_CLOEXEC);
                fcntl(fd, F_SETFL, O_NONBLOCK);
            }
        }
        let fds = (fds[0], fds[1]);
        PIPE.set(Some(fds));
        Some(fds)
    }

    /// Copy one byte from `src` to `dst` through the pipe: the kernel makes the accesses that
    /// could fault, and fails with EFAULT instead.
    fn copy_byte(src: u64, dst: u64) -> bool {
        let Some((from, to)) = pipe_fds() else {
            return false;
        };
        // SAFETY: the kernel reads `src`.
        if unsafe { sys_write(to, src as *const c_void, 1) } != 1 {
            return false;
        }
        // SAFETY: the kernel writes `dst`.
        if unsafe { sys_read(from, dst as *mut c_void, 1) } == 1 {
            return true;
        }
        // Take the byte back out, so the pipe is empty for the next check.
        let mut scratch = 0u8;
        // SAFETY: reads into a local byte.
        unsafe { sys_read(from, (&raw mut scratch).cast(), 1) };
        false
    }

    /// Can the byte at `at` be read (and, if asked, written)?
    pub fn accessible(at: u64, write: bool) -> bool {
        let mut byte = 0u8;
        let local = (&raw mut byte) as u64;
        // A write check stores back the byte just read, so only the access is tested.
        copy_byte(at, local) && (!write || copy_byte(local, at))
    }
}

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;

    #[repr(C)]
    struct MemoryBasicInformation {
        base: *mut c_void,
        allocation_base: *mut c_void,
        allocation_protect: u32,
        partition_id: u16,
        region_size: usize,
        state: u32,
        protect: u32,
        kind: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn VirtualQuery(
            address: *const c_void,
            info: *mut MemoryBasicInformation,
            length: usize,
        ) -> usize;
    }

    const MEM_COMMIT: u32 = 0x1000;
    const PAGE_NOACCESS: u32 = 0x01;
    const PAGE_GUARD: u32 = 0x100;

    /// Protections that allow writing: read-write, write-copy and their executable forms.
    const WRITABLE: u32 = 0x04 | 0x08 | 0x40 | 0x80;

    /// Is the byte at `at` committed, readable and (if asked) writable?
    pub fn accessible(at: u64, write: bool) -> bool {
        // SAFETY: an all-zero struct of plain fields is valid; VirtualQuery fills it.
        let mut info: MemoryBasicInformation = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<MemoryBasicInformation>();
        // SAFETY: VirtualQuery accepts any address and writes at most `size` bytes.
        if unsafe { VirtualQuery(at as *const c_void, &mut info, size) } != size {
            return false;
        }
        info.state == MEM_COMMIT
            && info.protect & (PAGE_NOACCESS | PAGE_GUARD) == 0
            && (!write || info.protect & WRITABLE != 0)
    }
}

#[cfg(target_arch = "wasm32")]
mod sys {
    /// Is the byte inside the linear memory?
    pub fn accessible(at: u64, _write: bool) -> bool {
        at < core::arch::wasm32::memory_size(0) as u64 * 65536
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_readable_memory_and_refuses_the_rest() {
        let data: Vec<u8> = (0..2000u32).map(|i| i as u8).collect();
        let mut out = vec![0u8; data.len()];
        assert!(read(data.as_ptr() as u64, &mut out));
        assert_eq!(out, data);
        // The fuzz input's address: an integer constant cast to a pointer.
        assert!(!read(80_000_000, &mut [0u8; 16]));
        assert!(!read(0x40, &mut [0u8; 8]));
        assert!(!read(u64::MAX - 4, &mut [0u8; 8]));
        // A range of many pages, more than are remembered.
        let big: Vec<u8> = (0..(3 << 20) + 7u32).map(|i| (i * 31) as u8).collect();
        let mut copy = vec![0u8; big.len()];
        assert!(read(big.as_ptr() as u64, &mut copy));
        assert!(copy == big);
        // The pipe is empty again after a failure.
        assert!(read(data.as_ptr() as u64, &mut out[..10]));
        assert_eq!(out[..10], data[..10]);
    }

    #[test]
    fn writes_writable_memory_and_refuses_the_rest() {
        let mut target = vec![0u8; 1500];
        let bytes: Vec<u8> = (0..1500u32).map(|i| (i * 7) as u8).collect();
        assert!(write(target.as_mut_ptr() as u64, &bytes));
        assert_eq!(target, bytes);
        assert!(!write(80_000_000, &[1, 2, 3]));
        // Read-only memory: a string literal. Readable, not writable.
        let literal: &'static str = "read-only data";
        assert!(!write(literal.as_ptr() as u64, b"x"));
        assert!(read(literal.as_ptr() as u64, &mut [0u8; 4]));
    }

    #[test]
    fn released_memory_is_checked_again() {
        let block = vec![7u8; 64];
        let addr = block.as_ptr() as u64;
        assert!(read(addr, &mut [0u8; 8]));
        invalidate();
        // Still mapped (the allocator keeps it), so still readable: checked afresh.
        assert!(read(addr, &mut [0u8; 8]));
        drop(block);
    }
}
