//! Copies between the compiler and program memory that never fault.
//!
//! Interpreted code loads and stores through real pointers, so a program that makes a pointer
//! from an integer and follows it crashes like a native program would. The compiler itself must
//! not: when it reads a `#run` result, a string a metaprogram handed to `add_build_string`, or
//! writes an out-parameter of a `compiler_*` call, the address comes from the program and may
//! be anything. Mapping queries bound allocations, but are never proof that a later access
//! is safe: another native thread can change protections at any time. Actual transfers use
//! fault-safe kernel copies, which report an unmapped/protected address as an error:
//!
//! - macOS: `mach_vm_read_overwrite` and `mach_vm_write` transfer actual bytes.
//! - Other Unix: actual bytes go through a per-thread pipe, returning `EFAULT` on bad pointers.
//! - Windows: `ReadProcessMemory` / `WriteProcessMemory` transfer actual bytes.
//! - wasm32: program memory is the linear memory, so the address only has to lie inside it.
//!
//! Asking costs system calls, and a metaprogram's messages are read a field at a time, so what
//! was found accessible is remembered, as pages and as the larger ranges the system described,
//! until memory may have been released: every foreign call (`free`, `munmap`, or C code that
//! frees) and every interpreter block freed calls `invalidate`.
#![allow(unsafe_code)]

use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "linux")]
#[path = "probe_maps.rs"]
mod maps;

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
    /// The `len` bytes of program memory at `addr`, or `None` when any of them cannot be
    /// read. Nothing is allocated for a range that is not there (a garbage length).
    pub fn read_vec(&self, addr: u64, len: usize) -> Option<Vec<u8>> {
        if !self.accessible(addr, len, false) {
            return None;
        }
        let mut out = Vec::new();
        out.try_reserve_exact(len).ok()?;
        out.resize(len, 0);
        self.read(addr, &mut out).then_some(out)
    }

    /// Copy `out.len()` bytes of program memory at `addr` into `out`. False when any of them
    /// cannot be read.
    pub fn read(&self, addr: u64, out: &mut [u8]) -> bool {
        if !self.accessible(addr, out.len(), false) {
            return false;
        }
        out.is_empty() || sys::copy(addr, out.as_mut_ptr() as u64, out.len(), false)
    }

    /// Compare readable byte ranges with bounded scratch storage. Invalid ranges are errors,
    /// never ordered as if they contained data. Empty ranges may start anywhere.
    pub fn compare(&self, a: u64, b: u64, len: usize) -> Option<std::cmp::Ordering> {
        a.checked_add(len as u64)?;
        b.checked_add(len as u64)?;
        if !self.accessible(a, len, false) || !self.accessible(b, len, false) {
            return None;
        }
        let mut left = [0; PAGE as usize];
        let mut right = [0; PAGE as usize];
        let mut offset = 0;
        while offset < len {
            let count = left.len().min(len - offset);
            if !self.read(a + offset as u64, &mut left[..count])
                || !self.read(b + offset as u64, &mut right[..count])
            {
                return None;
            }
            let ordering = left[..count].cmp(&right[..count]);
            if !ordering.is_eq() {
                return Some(ordering);
            }
            offset += count;
        }
        Some(std::cmp::Ordering::Equal)
    }

    /// Fault-safe memmove, including overlapping source/destination ranges.
    pub fn copy(&self, destination: u64, source: u64, len: usize) -> bool {
        let Some(source_end) = source.checked_add(len as u64) else {
            return false;
        };
        if destination.checked_add(len as u64).is_none() {
            return false;
        }
        let backward = destination > source && destination < source_end;
        let mut bytes = [0; PAGE as usize];
        let mut remaining = len;
        while remaining > 0 {
            let count = bytes.len().min(remaining);
            let offset = if backward {
                remaining - count
            } else {
                len - remaining
            };
            if !self.read(source + offset as u64, &mut bytes[..count])
                || !self.write(destination + offset as u64, &bytes[..count])
            {
                return false;
            }
            remaining -= count;
        }
        true
    }

    /// Fault-safe repeated-byte fill, without allocating storage proportional to `len`.
    pub fn fill(&self, destination: u64, value: u8, len: usize) -> bool {
        if destination.checked_add(len as u64).is_none() {
            return false;
        }
        let bytes = [value; PAGE as usize];
        let mut offset = 0;
        while offset < len {
            let count = bytes.len().min(len - offset);
            if !self.write(destination + offset as u64, &bytes[..count]) {
                return false;
            }
            offset += count;
        }
        true
    }

    /// Copy `bytes` into program memory at `addr`. False when any of it cannot be written.
    pub fn write(&self, addr: u64, bytes: &[u8]) -> bool {
        if !self.accessible(addr, bytes.len(), true) {
            return false;
        }
        bytes.is_empty() || sys::copy(bytes.as_ptr() as u64, addr, bytes.len(), true)
    }

    /// Can every page of `addr..addr + len` be read (or written)? Asks the system about the
    /// pages not known already.
    #[inline(always)]
    pub(super) fn accessible(&self, addr: u64, len: usize, write: bool) -> bool {
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
        fn close(fd: c_int) -> c_int;
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

    struct ProbePipe(Cell<Option<(c_int, c_int)>>);

    impl Drop for ProbePipe {
        fn drop(&mut self) {
            if let Some((read, write)) = self.0.get() {
                // SAFETY: these descriptors belong exclusively to this thread's probe.
                unsafe {
                    close(read);
                    close(write);
                }
            }
        }
    }

    thread_local! {
        /// One kernel-copy channel per probing thread, closed when the thread exits.
        static PIPE: ProbePipe = const { ProbePipe(Cell::new(None)) };
    }

    fn pipe_fds() -> Option<(c_int, c_int)> {
        PIPE.with(|cached| {
            if let Some(fds) = cached.0.get() {
                return Some(fds);
            }
            let mut fds = [0 as c_int; 2];
            // SAFETY: `pipe` writes two descriptors; subsequent calls take plain integers.
            unsafe {
                if pipe(fds.as_mut_ptr()) != 0 {
                    return None;
                }
                // A failed setup must not leave a blocking pipe or leak descriptors.
                for fd in fds {
                    if fcntl(fd, F_SETFD, FD_CLOEXEC) < 0 || fcntl(fd, F_SETFL, O_NONBLOCK) < 0 {
                        close(fds[0]);
                        close(fds[1]);
                        return None;
                    }
                }
            }
            let fds = (fds[0], fds[1]);
            cached.0.set(Some(fds));
            Some(fds)
        })
    }

    /// Copy one byte from `src` to `dst` through the pipe: the kernel makes the accesses that
    /// could fault, and fails with EFAULT instead.
    #[cfg(not(target_os = "linux"))]
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

    /// Transfer through kernel buffers; a protection change yields EFAULT, never
    /// an unchecked userspace dereference. Chunks fit the minimum pipe capacity.
    pub fn copy(mut src: u64, mut dst: u64, mut len: usize, _write: bool) -> bool {
        let Some((from, to)) = pipe_fds() else {
            return false;
        };
        while len != 0 {
            let count = len.min(512);
            // SAFETY: the kernel validates both user ranges while copying.
            let written = unsafe { sys_write(to, src as *const c_void, count) };
            if written <= 0 {
                return false;
            }
            // SAFETY: likewise, invalid destinations are reported by read.
            let read = unsafe { sys_read(from, dst as *mut c_void, written as usize) };
            if read != written {
                let mut scratch = [0u8; 512];
                let remaining = written - read.max(0);
                // SAFETY: drain the outstanding bytes into a local buffer.
                unsafe { sys_read(from, scratch.as_mut_ptr().cast(), remaining as usize) };
                return false;
            }
            src += written as u64;
            dst += written as u64;
            len -= written as usize;
        }
        true
    }

    /// Query our mapping permissions without reading or writing the allocation.
    /// A write-back permission probe can corrupt a concurrent atomic update.
    #[cfg(target_os = "linux")]
    pub fn region(at: u64, write: bool) -> Option<(u64, u64)> {
        let maps = std::fs::read("/proc/self/maps").ok()?;
        super::maps::region(&maps, at, write)
    }

    /// The page of `at`, if its byte can be read (and, if asked, written).
    #[cfg(not(target_os = "linux"))]
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

    pub fn copy(src: u64, dst: u64, len: usize, write: bool) -> bool {
        unsafe extern "C" {
            static mach_task_self_: u32;
            fn mach_vm_read_overwrite(
                task: u32,
                address: u64,
                size: u64,
                data: u64,
                out: *mut u64,
            ) -> c_int;
            fn mach_vm_write(task: u32, address: u64, data: usize, count: u32) -> c_int;
        }
        if !write {
            let mut copied = 0;
            // SAFETY: Mach validates the source and destination mappings in-kernel.
            return unsafe {
                mach_vm_read_overwrite(mach_task_self_, src, len as u64, dst, &mut copied)
            } == 0
                && copied == len as u64;
        }
        let mut offset = 0;
        while offset < len {
            let count = (len - offset).min(u32::MAX as usize);
            // SAFETY: Mach validates the target mapping; source is our owned slice.
            if unsafe {
                mach_vm_write(
                    mach_task_self_,
                    dst + offset as u64,
                    src as usize + offset,
                    count as u32,
                )
            } != 0
            {
                return false;
            }
            offset += count;
        }
        true
    }

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
        fn GetCurrentProcess() -> *mut c_void;

        fn ReadProcessMemory(
            process: *mut c_void,
            address: *const c_void,
            buffer: *mut c_void,
            size: usize,
            copied: *mut usize,
        ) -> i32;

        fn WriteProcessMemory(
            process: *mut c_void,
            address: *mut c_void,
            buffer: *const c_void,
            size: usize,
            copied: *mut usize,
        ) -> i32;

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

    pub fn copy(src: u64, dst: u64, len: usize, write: bool) -> bool {
        let mut copied = 0;
        // SAFETY: these APIs validate program addresses in the kernel and report faults.
        let status = unsafe {
            let process = GetCurrentProcess();
            if write {
                WriteProcessMemory(
                    process,
                    dst as *mut c_void,
                    src as *const c_void,
                    len,
                    &mut copied,
                )
            } else {
                ReadProcessMemory(
                    process,
                    src as *const c_void,
                    dst as *mut c_void,
                    len,
                    &mut copied,
                )
            }
        };
        status != 0 && copied == len
    }

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
    pub fn copy(src: u64, dst: u64, len: usize, _write: bool) -> bool {
        let end = core::arch::wasm32::memory_size(0) as u64 * 65536;
        if src.checked_add(len as u64).is_none_or(|x| x > end)
            || dst.checked_add(len as u64).is_none_or(|x| x > end)
        {
            return false;
        }
        // SAFETY: memory cannot shrink; both ranges are inside linear memory.
        unsafe { std::ptr::copy(src as *const u8, dst as *mut u8, len) };
        true
    }

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
    fn read_vec_copies_readable_memory_and_refuses_garbage_lengths() {
        let data: Vec<u8> = (0..300u32).map(|i| i as u8).collect();
        let got = PROBE.with(|p| p.read_vec(data.as_ptr() as u64, data.len()));
        assert_eq!(got.as_deref(), Some(&data[..]));
        // A garbage length reads nothing and allocates nothing.
        assert!(
            PROBE
                .with(|p| p.read_vec(data.as_ptr() as u64, usize::MAX >> 4))
                .is_none()
        );
        assert!(PROBE.with(|p| p.read_vec(80_000_000, 16)).is_none());
        assert!(PROBE.with(|p| p.read_vec(0x40, 1 << 40)).is_none());
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

    #[cfg(any(unix, windows))]
    #[test]
    fn memory_intrinsics_reject_protected_ranges() {
        const CHILD: &str = "JAIC_MEMORY_INTRINSIC_CHILD";
        let Ok(operation) = std::env::var(CHILD) else {
            let results: Vec<_> = ["compare", "copy", "fill"]
                .into_iter()
                .map(|operation| {
                    let status = std::process::Command::new(std::env::current_exe().unwrap())
                        .args([
                            "--exact",
                            "interp::probe::tests::memory_intrinsics_reject_protected_ranges",
                            "--nocapture",
                        ])
                        .env(CHILD, operation)
                        .status()
                        .unwrap();
                    (operation, status)
                })
                .collect();
            assert!(
                results.iter().all(|(_, status)| status.success()),
                "{results:?}"
            );
            return;
        };
        #[cfg(unix)]
        use std::ffi::c_int;
        use std::ffi::c_void;
        #[cfg(unix)]
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
        #[cfg(windows)]
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn VirtualAlloc(
                addr: *mut c_void,
                len: usize,
                allocation: u32,
                protection: u32,
            ) -> *mut c_void;
            fn VirtualProtect(addr: *mut c_void, len: usize, protection: u32, old: *mut u32)
            -> i32;
            fn VirtualFree(addr: *mut c_void, len: usize, operation: u32) -> i32;
        }
        #[cfg(target_os = "linux")]
        const ANON: c_int = 0x20;
        #[cfg(all(unix, not(target_os = "linux")))]
        const ANON: c_int = 0x1000;
        const SIZE: usize = 64 << 10;
        // SAFETY: an owned anonymous mapping, protected only after warming the cache.
        #[cfg(unix)]
        let base = unsafe { mmap(std::ptr::null_mut(), SIZE, 3, 2 | ANON, -1, 0) };
        #[cfg(windows)]
        let base = unsafe { VirtualAlloc(std::ptr::null_mut(), SIZE, 0x3000, 4) };
        assert_ne!(base as usize, usize::MAX);
        assert!(!base.is_null());
        let addr = base as u64;
        let mut interp = super::super::Interp::default();
        let mut bytes = [0u8; 8];
        assert!(interp.probe.read(addr, &mut bytes));
        assert!(interp.probe.write(addr, &bytes));
        // SAFETY: no other thread accesses the mapping during this protection change.
        #[cfg(unix)]
        assert_eq!(unsafe { mprotect(base, SIZE, 0) }, 0);
        #[cfg(windows)]
        {
            let mut previous = 0;
            assert_ne!(unsafe { VirtualProtect(base, SIZE, 1, &mut previous) }, 0);
        }
        let (intrinsic, arguments, diagnostic) = match operation.as_str() {
            "compare" => (
                crate::ir::Intrinsic::Memcmp,
                [addr, bytes.as_ptr() as u64, 8],
                "memcmp through unreadable memory",
            ),
            "copy" => (
                crate::ir::Intrinsic::Memcpy,
                [bytes.as_mut_ptr() as u64, addr, 8],
                "memcpy through inaccessible memory",
            ),
            "fill" => (
                crate::ir::Intrinsic::Memset,
                [addr, 42, 8],
                "memset through unwritable memory",
            ),
            _ => panic!("unknown child operation"),
        };
        let result = interp.intrinsic(intrinsic, &arguments, None);
        // SAFETY: the mapping has no remaining users.
        #[cfg(unix)]
        assert_eq!(unsafe { munmap(base, SIZE) }, 0);
        #[cfg(windows)]
        assert_ne!(unsafe { VirtualFree(base, 0, 0x8000) }, 0);
        let error = result.expect_err("an inaccessible range must produce an interpreter error");
        assert!(error.message.contains(diagnostic), "{error:?}");
    }

    #[test]
    fn memory_intrinsics_preserve_overlap_and_empty_ranges() {
        let mut interp = super::super::Interp::default();
        let original: Vec<u8> = (0..15000).map(|i| (i % 251) as u8).collect();
        for (source, destination) in [(0, 3), (3, 0), (0, 7500)] {
            let mut actual = original.clone();
            let mut expected = original.clone();
            expected.copy_within(source..source + 7000, destination);
            let base = actual.as_mut_ptr() as u64;
            interp
                .intrinsic(
                    crate::ir::Intrinsic::Memcpy,
                    &[base + destination as u64, base + source as u64, 7000],
                    None,
                )
                .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(
                interp
                    .intrinsic(
                        crate::ir::Intrinsic::Memcmp,
                        &[base, expected.as_ptr() as u64, actual.len() as u64],
                        None
                    )
                    .unwrap()
                    .into_vec(),
                [0]
            );
        }
        let mut filled = vec![0u8; 10000];
        interp
            .intrinsic(
                crate::ir::Intrinsic::Memset,
                &[filled.as_mut_ptr() as u64, 0xa5, filled.len() as u64],
                None,
            )
            .unwrap();
        assert!(filled.iter().all(|&byte| byte == 0xa5));
        for operation in [
            crate::ir::Intrinsic::Memcpy,
            crate::ir::Intrinsic::Memset,
            crate::ir::Intrinsic::Memcmp,
        ] {
            assert!(interp.intrinsic(operation, &[0, 0, 0], None).is_ok());
        }
        assert!(!interp.probe.copy(u64::MAX, 0, 2));
        assert!(!interp.probe.fill(u64::MAX, 0, 2));
        assert!(interp.probe.compare(u64::MAX, 0, 2).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn stale_mapping_cache_cannot_fault_compiler_memory_transfers() {
        const CHILD: &str = "JAIC_PROBE_STALE_MAPPING_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let status=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "interp::probe::tests::stale_mapping_cache_cannot_fault_compiler_memory_transfers", "--nocapture"])
                .env(CHILD,"1").status().unwrap();
            assert!(
                status.success(),
                "guarded memory transfer child failed: {status}"
            );
            return;
        }
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
        const ANON: c_int = 0x20;
        #[cfg(not(target_os = "linux"))]
        const ANON: c_int = 0x1000;
        const SIZE: usize = 64 << 10;
        // SAFETY: owned anonymous mapping, released after all probe operations.
        let base = unsafe { mmap(std::ptr::null_mut(), SIZE, 3, 2 | ANON, -1, 0) };
        assert_ne!(base as usize, usize::MAX);
        let addr = base as u64;
        let probe = Probe::default();
        let mut bytes = [0u8; 8];
        assert!(probe.read(addr, &mut bytes));
        assert!(probe.write(addr, &bytes));
        std::thread::spawn(move || {
            // SAFETY: native thread changes the owned mapping while no transfer is active.
            assert_eq!(unsafe { mprotect(addr as *mut c_void, SIZE, 0) }, 0);
        })
        .join()
        .unwrap();
        // No epoch invalidation: real native protection changes bypass compiler hooks.
        assert!(!probe.read(addr, &mut bytes));
        assert!(!probe.write(addr, &bytes));
        assert!(probe.read_vec(addr, 8).is_none());
        assert_eq!(probe.compare(addr, bytes.as_ptr() as u64, 8), None);
        // SAFETY: the mapping is no longer accessed by any native thread.
        assert_eq!(unsafe { munmap(base, SIZE) }, 0);
        assert!(!probe.read(addr, &mut bytes));
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
        // A native thread can change page protections without calling the compiler.
        // Deliberately keep any cached mapping information from the earlier reads.
        assert!(read(start, &mut byte));
        assert!(!read(start + SIZE as u64 + 8, &mut byte));
        assert!(!write(start + SIZE as u64, &byte));
        assert!(
            PROBE
                .with(|p| p.read_vec(start + SIZE as u64, byte.len()))
                .is_none()
        );
        assert_eq!(
            PROBE.with(|p| p.compare(start + SIZE as u64, start, byte.len())),
            None
        );
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
