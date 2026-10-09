//! Copies between the compiler and program memory that never fault.
//!
//! Interpreted code loads and stores through real pointers, so a program that makes a pointer
//! from an integer and follows it crashes like a native program would. The compiler itself must
//! not: when it reads a `#run` result, a string a metaprogram handed to `add_build_string`, or
//! writes an out-parameter of a `compiler_*` call, the address comes from the program and may
//! be anything. Before such an access touches a page, the kernel is asked whether it can, which
//! reports an unmapped or protected address as an error rather than a signal:
//!
//! - macOS: `mach_vm_region` describes the whole mapping the address lies in and what it allows.
//! - Other Unix: one byte of the page goes through a per-thread pipe. `write(pipe, addr)` fails
//!   with `EFAULT` when `addr` cannot be read, and `read(pipe, addr)` when it cannot be written.
//! - Windows: `VirtualQuery` says whether the pages are committed and allow the access.
//! - wasm32: program memory is the linear memory, so the address only has to lie inside it.
//!
//! Asking costs system calls, and a metaprogram's messages are read a field at a time, so what
//! was found accessible is remembered, as pages and as the larger ranges the system described,
//! until memory may have been released: every foreign call (`free`, `munmap`, or C code that
//! frees) and every interpreter block freed calls `invalidate`.
#![allow(unsafe_code)]

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

/// The granularity accesses are checked at: the smallest page size of the hosts.
const PAGE: u64 = 4096;

/// Pages remembered per thread and kind of access: a direct-mapped table, so a lookup is one
/// index and one comparison.
const REMEMBERED: usize = 256;

/// Ranges the system described, remembered per thread and kind of access, oldest replaced first.
const REGIONS: usize = 8;

/// Bumped whenever memory may have been unmapped; remembered pages are good only while it
/// keeps the value they were checked under.
static EPOCH: AtomicU64 = AtomicU64::new(0);

/// A page found accessible: `(page + 1, epoch)`. An empty entry is `(0, 0)`, which no page
/// matches (pages below 4 KiB are refused, so `page + 1` is at least 2).
type Entry = Cell<(u64, u64)>;

/// A range found accessible: `(start, end, epoch)`. An empty one has `end == 0`.
type Region = Cell<(u64, u64, u64)>;

/// Forget every page found accessible: memory may have been released.
pub fn invalidate() {
    EPOCH.fetch_add(1, Ordering::Relaxed);
}

/// What one interpreter has found accessible. Interpreters on different threads each have
/// their own, and all forget at `invalidate`.
pub struct Probe {
    /// Pages found accessible, for reads and for writes.
    known: [[Entry; REMEMBERED]; 2],
    /// The ranges found accessible for reads and for writes.
    regions: [[Region; REGIONS]; 2],
    /// Which range is replaced next.
    next: Cell<usize>,
}

impl Default for Probe {
    fn default() -> Self {
        Probe {
            known: std::array::from_fn(|_| std::array::from_fn(|_| Cell::new((0, 0)))),
            regions: std::array::from_fn(|_| std::array::from_fn(|_| Cell::new((0, 0, 0)))),
            next: Cell::new(0),
        }
    }
}

impl Probe {
    /// Copy `out.len()` bytes of program memory at `addr` into `out`. False when any of them
    /// cannot be read.
    pub fn read(&self, addr: u64, out: &mut [u8]) -> bool {
        if !self.accessible(addr, out.len(), false) {
            return false;
        }
        if !out.is_empty() {
            // SAFETY: every page of the range was found readable.
            unsafe {
                std::ptr::copy_nonoverlapping(addr as *const u8, out.as_mut_ptr(), out.len())
            };
        }
        true
    }

    /// Compare `len` bytes of program memory at `a` and at `b`, in place. A range that cannot
    /// be read orders before one that can, and equal to another that cannot.
    pub fn compare(&self, a: u64, b: u64, len: usize) -> std::cmp::Ordering {
        let view = |addr: u64| {
            // SAFETY: every page of the range was found readable.
            self.accessible(addr, len, false)
                .then(|| unsafe { std::slice::from_raw_parts(addr as *const u8, len) })
        };
        // An empty range may start anywhere, `from_raw_parts` wants a non-null pointer.
        if len == 0 {
            return std::cmp::Ordering::Equal;
        }
        view(a).cmp(&view(b))
    }

    /// Copy `bytes` into program memory at `addr`. False when any of it cannot be written.
    pub fn write(&self, addr: u64, bytes: &[u8]) -> bool {
        if !self.accessible(addr, bytes.len(), true) {
            return false;
        }
        if !bytes.is_empty() {
            // SAFETY: every page of the range was found writable.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()) };
        }
        true
    }

    /// Can every page of `addr..addr + len` be read (or written)? Asks the system about the
    /// pages not known already.
    #[inline(always)]
    fn accessible(&self, addr: u64, len: usize, write: bool) -> bool {
        // The usual access is one more field of a record just read: inside one known page.
        // Addresses below 4 KiB are never mapped (and are how null dereferences show).
        if addr >= PAGE && len != 0 && (len as u64) <= PAGE - addr % PAGE {
            let page = addr / PAGE;
            let known = &self.known[usize::from(write)][page as usize % REMEMBERED];
            if known.get() == (page + 1, EPOCH.load(Ordering::Relaxed)) {
                return true;
            }
        }
        self.check(addr, len, write)
    }

    #[inline(never)]
    fn check(&self, addr: u64, len: usize, write: bool) -> bool {
        if len == 0 {
            return true;
        }
        // A range may not wrap.
        let Some(end) = addr.checked_add(len as u64) else {
            return false;
        };
        if addr < PAGE || end > usize::MAX as u64 {
            return false;
        }
        let epoch = EPOCH.load(Ordering::Relaxed);
        let known = &self.known[usize::from(write)];
        let mut page = addr / PAGE;
        while page * PAGE < end {
            let slot = &known[page as usize % REMEMBERED];
            if slot.get() != (page + 1, epoch)
                && !self.mapped(page, (page * PAGE).max(addr), write, epoch)
            {
                return false;
            }
            slot.set((page + 1, epoch));
            page += 1;
        }
        true
    }

    /// Is `page` inside a range found accessible, or can the system say so? `at`, a byte of
    /// the page inside the range being checked, stands for the page.
    #[cold]
    fn mapped(&self, page: u64, at: u64, write: bool, epoch: u64) -> bool {
        let regions = &self.regions[usize::from(write)];
        let (start, end) = (page * PAGE, page * PAGE + PAGE);
        if regions.iter().any(|r| {
            let (s, e, ep) = r.get();
            ep == epoch && s <= start && end <= e
        }) {
            return true;
        }
        let Some((s, e)) = sys::region(at, write) else {
            return false;
        };
        regions[self.next.get() % REGIONS].set((s, e, epoch));
        self.next.set(self.next.get() + 1);
        true
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
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

    /// The page of `at`, if its byte can be read (and, if asked, written).
    pub fn region(at: u64, write: bool) -> Option<(u64, u64)> {
        let mut byte = 0u8;
        let local = (&raw mut byte) as u64;
        // A write check stores back the byte just read, so only the access is tested.
        if !(copy_byte(at, local) && (!write || copy_byte(local, at))) {
            return None;
        }
        let page = at & !(super::PAGE - 1);
        Some((page, page + super::PAGE))
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::c_int;

    /// The mapping `at` lies in, if it allows reading (and, if asked, writing).
    pub fn region(at: u64, write: bool) -> Option<(u64, u64)> {
        unsafe extern "C" {
            static mach_task_self_: u32;

            fn mach_vm_region(
                task: u32,
                address: *mut u64,
                size: *mut u64,
                flavor: c_int,
                info: *mut c_int,
                count: *mut u32,
                object: *mut u32,
            ) -> c_int;
        }
        const VM_REGION_BASIC_INFO_64: c_int = 9;
        const READ: c_int = 1;
        const WRITE: c_int = 2;
        let (mut start, mut size) = (at, 0u64);
        // `vm_region_basic_info_64`: nine words, the first being the current protection.
        let mut info = [0 as c_int; 9];
        let (mut count, mut object) = (info.len() as u32, 0u32);
        // SAFETY: the buffers are as big as the flavor says; the address is only described.
        let status = unsafe {
            mach_vm_region(
                mach_task_self_,
                &mut start,
                &mut size,
                VM_REGION_BASIC_INFO_64,
                info.as_mut_ptr(),
                &mut count,
                &mut object,
            )
        };
        // The call describes the next mapping at or after `at`: one that starts later means
        // `at` is in a gap.
        if status != 0 || start > at || info[0] & READ == 0 || (write && info[0] & WRITE == 0) {
            return None;
        }
        Some((start, start + size))
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

    /// The pages around `at` that are committed, readable and (if asked) writable like it is.
    pub fn region(at: u64, write: bool) -> Option<(u64, u64)> {
        // SAFETY: an all-zero struct of plain fields is valid; VirtualQuery fills it.
        let mut info: MemoryBasicInformation = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of::<MemoryBasicInformation>();
        // SAFETY: VirtualQuery accepts any address and writes at most `size` bytes.
        if unsafe { VirtualQuery(at as *const c_void, &mut info, size) } != size {
            return None;
        }
        let allowed = info.state == MEM_COMMIT
            && info.protect & (PAGE_NOACCESS | PAGE_GUARD) == 0
            && (!write || info.protect & WRITABLE != 0);
        let start = info.base as u64;
        allowed.then_some((start, start + info.region_size as u64))
    }
}

#[cfg(target_arch = "wasm32")]
mod sys {
    /// The linear memory, if `at` is inside it.
    pub fn region(at: u64, _write: bool) -> Option<(u64, u64)> {
        let end = core::arch::wasm32::memory_size(0) as u64 * 65536;
        (at < end).then_some((0, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(addr: u64, out: &mut [u8]) -> bool {
        PROBE.with(|p| p.read(addr, out))
    }

    fn write(addr: u64, bytes: &[u8]) -> bool {
        PROBE.with(|p| p.write(addr, bytes))
    }

    thread_local! {
        static PROBE: Probe = Probe::default();
    }

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

    #[cfg(unix)]
    #[test]
    fn a_protected_page_in_the_middle_of_a_known_mapping_is_found() {
        use std::ffi::{c_int, c_void};
        unsafe extern "C" {
            fn mmap(
                addr: *mut c_void,
                len: usize,
                prot: c_int,
                flags: c_int,
                fd: c_int,
                off: i64,
            ) -> *mut c_void;
            fn mprotect(addr: *mut c_void, len: usize, prot: c_int) -> c_int;
            fn munmap(addr: *mut c_void, len: usize) -> c_int;
        }
        #[cfg(target_os = "linux")]
        const MAP_ANON: c_int = 0x20;
        #[cfg(not(target_os = "linux"))]
        const MAP_ANON: c_int = 0x1000;
        // Three of the largest pages any host has, so the middle one is a page of its own.
        const SIZE: usize = 64 << 10;
        // SAFETY: an anonymous private mapping, unmapped below.
        let base = unsafe { mmap(std::ptr::null_mut(), 3 * SIZE, 3, 2 | MAP_ANON, -1, 0) };
        assert_ne!(base as usize, usize::MAX);
        let start = base as u64;
        let mut byte = [0u8; 8];
        // The whole mapping is known readable and writable first.
        assert!(read(start, &mut byte) && write(start + 2 * SIZE as u64, &byte));
        // SAFETY: the middle third of the mapping.
        assert_eq!(
            unsafe { mprotect(base.wrapping_byte_add(SIZE), SIZE, 0) },
            0
        );
        invalidate();
        assert!(read(start, &mut byte));
        assert!(!read(start + SIZE as u64 + 8, &mut byte));
        assert!(!write(start + SIZE as u64, &byte));
        assert!(
            !read(start + SIZE as u64 - 4, &mut byte),
            "a range that runs into it"
        );
        assert!(read(start + 2 * SIZE as u64, &mut byte));
        // SAFETY: the mapping made above.
        unsafe { munmap(base, 3 * SIZE) };
        invalidate();
        assert!(!read(start, &mut byte));
    }
}
