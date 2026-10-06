//! An exact cap on the memory a `jaic` process allocates (`JAIC_MEMORY_LIMIT`).
//!
//! [`CountingAllocator`] is the `jaic` binary's global allocator: it forwards to the system
//! allocator and, once [`arm`]ed, keeps a running total of live bytes. The interpreter's own
//! memory (the stack, globals, the sandbox heap) comes from Rust allocations; the C `malloc`
//! family a natively linked program calls at compile time or under `jaic run` is resolved to the
//! counting wrappers in [`foreign_override`] instead of libc's. Crossing the limit prints one
//! line and ends the process with [`EXIT_CODE`], so callers (the corpus sweep) can tell it from
//! an ordinary failure.
//!
//! Memory nobody here sees: allocations made by C or C++ libraries on their own (LLVM, libclang,
//! a `#foreign` library that calls `malloc` internally or maps pages itself), and child processes
//! (the linker, programs a metaprogram launches).
#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering::Relaxed};

/// Exit status of a process stopped by the limit.
pub const EXIT_CODE: u8 = 120;

/// The environment variable holding the limit in bytes (a `K`, `M` or `G` suffix scales it).
pub const ENV_VAR: &str = "JAIC_MEMORY_LIMIT";

/// The limit in bytes; 0 while unarmed, which makes the allocator a plain pass-through.
static LIMIT: AtomicUsize = AtomicUsize::new(0);

/// Live bytes allocated since arming. Blocks allocated before arming and freed after make it
/// slightly low, hence signed.
static LIVE: AtomicIsize = AtomicIsize::new(0);

/// Set by the first allocation over the limit, so the report itself is not limited.
static TRIPPED: AtomicBool = AtomicBool::new(false);

/// Starts enforcing `bytes` (0 turns enforcement off).
pub fn arm(bytes: usize) {
    LIVE.store(0, Relaxed);
    LIMIT.store(bytes, Relaxed);
}

/// Reads [`ENV_VAR`] and arms the limit it names. An unset variable leaves the limit off; a
/// malformed one is an error message for the caller to print.
pub fn arm_from_env() -> Result<(), String> {
    let Some(value) = std::env::var_os(ENV_VAR) else {
        return Ok(());
    };
    let text = value.to_string_lossy();
    let bytes = parse_size(&text)
        .ok_or_else(|| format!("{ENV_VAR}={text:?} is not a byte count (e.g. 3221225472 or 3G)"))?;
    arm(bytes);
    Ok(())
}

/// `1234`, `512K`, `64M` or `3G` (binary multiples) in bytes.
pub fn parse_size(text: &str) -> Option<usize> {
    let text = text.trim();
    let (digits, shift) = match text.as_bytes().last()?.to_ascii_uppercase() {
        b'K' => (&text[..text.len() - 1], 10),
        b'M' => (&text[..text.len() - 1], 20),
        b'G' => (&text[..text.len() - 1], 30),
        _ => (text, 0),
    };
    let n: usize = digits.trim().parse().ok()?;
    n.checked_mul(1usize << shift)
}

/// The armed limit in bytes, if any.
pub fn limit() -> Option<usize> {
    match LIMIT.load(Relaxed) {
        0 => None,
        n => Some(n),
    }
}

/// Bytes counted as live right now (0 while unarmed).
pub fn live() -> usize {
    LIVE.load(Relaxed).max(0) as usize
}

/// Counts `size` new bytes, stopping the process if that crosses the limit.
#[inline]
fn charge(size: usize) {
    let limit = LIMIT.load(Relaxed);
    if limit == 0 {
        return;
    }
    let now = LIVE.fetch_add(size as isize, Relaxed) + size as isize;
    if now > limit as isize && !TRIPPED.swap(true, Relaxed) {
        exceeded(limit);
    }
}

#[inline]
fn refund(size: usize) {
    if LIMIT.load(Relaxed) != 0 {
        LIVE.fetch_sub(size as isize, Relaxed);
    }
}

/// Reports the overrun and exits. Nothing here allocates: the message is formatted into a
/// stack buffer and written straight to descriptor 2, and the exit skips `atexit` handlers,
/// which could wait on a lock another thread holds mid-allocation.
#[cold]
fn exceeded(limit: usize) -> ! {
    use std::fmt::Write;
    struct Buf {
        bytes: [u8; 128],
        len: usize,
    }
    impl Write for Buf {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            let room = self.bytes.len() - self.len;
            let n = s.len().min(room);
            self.bytes[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
            self.len += n;
            Ok(())
        }
    }
    let mut buf = Buf {
        bytes: [0; 128],
        len: 0,
    };
    let _ = if limit.is_multiple_of(1 << 20) {
        writeln!(buf, "error: memory limit of {} MiB exceeded", limit >> 20)
    } else {
        writeln!(buf, "error: memory limit of {limit} bytes exceeded")
    };
    sys::write_stderr(&buf.bytes[..buf.len]);
    sys::exit_now(EXIT_CODE as i32)
}

/// The system allocator with live-byte accounting; install it with `#[global_allocator]`.
pub struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        charge(layout.size());
        unsafe { System.alloc(layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        charge(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        refund(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size >= layout.size() {
            charge(new_size - layout.size());
        } else {
            refund(layout.size() - new_size);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

/// The address of a counting replacement for the C allocator function `symbol`, used while the
/// limit is armed so that a program's `malloc` (the default Jai allocator) is counted too. The
/// replacements return real libc blocks, so C code may still free them, and libc blocks they
/// did not hand out are freed normally.
pub fn foreign_override(symbol: &str) -> Option<u64> {
    if LIMIT.load(Relaxed) == 0 {
        return None;
    }
    sys::foreign_override(symbol)
}

#[cfg(unix)]
mod sys {
    use super::{charge, refund};
    use std::ffi::c_void;

    unsafe extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn calloc(count: usize, size: usize) -> *mut c_void;
        fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
        fn free(ptr: *mut c_void);
        fn posix_memalign(out: *mut *mut c_void, align: usize, size: usize) -> i32;
        fn write(fd: i32, buf: *const c_void, len: usize) -> isize;
        fn _exit(status: i32) -> !;

        #[cfg(target_vendor = "apple")]
        fn malloc_size(ptr: *const c_void) -> usize;

        #[cfg(not(target_vendor = "apple"))]
        fn malloc_usable_size(ptr: *mut c_void) -> usize;
    }

    pub(super) fn write_stderr(bytes: &[u8]) {
        unsafe { write(2, bytes.as_ptr().cast(), bytes.len()) };
    }

    pub(super) fn exit_now(code: i32) -> ! {
        unsafe { _exit(code) }
    }

    /// Bytes libc reserved for `ptr`: what the counting wrappers charge and refund, so both
    /// sides agree whatever rounding the allocator applies.
    fn block_size(ptr: *mut c_void) -> usize {
        if ptr.is_null() {
            return 0;
        }
        #[cfg(target_vendor = "apple")]
        unsafe {
            malloc_size(ptr)
        }
        #[cfg(not(target_vendor = "apple"))]
        unsafe {
            malloc_usable_size(ptr)
        }
    }

    extern "C" fn counted_malloc(size: usize) -> *mut c_void {
        charge(size);
        let p = unsafe { malloc(size) };
        settle(size, p);
        p
    }

    extern "C" fn counted_calloc(count: usize, size: usize) -> *mut c_void {
        let Some(total) = count.checked_mul(size) else {
            return std::ptr::null_mut();
        };

        charge(total);
        let p = unsafe { calloc(count, size) };
        settle(total, p);
        p
    }

    extern "C" fn counted_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
        let old = block_size(ptr);
        charge(size);
        let p = unsafe { realloc(ptr, size) };

        if p.is_null() {
            refund(size);
        } else {
            settle(size, p);
            refund(old);
        }
        p
    }

    extern "C" fn counted_free(ptr: *mut c_void) {
        refund(block_size(ptr));
        unsafe { free(ptr) }
    }

    extern "C" fn counted_posix_memalign(out: *mut *mut c_void, align: usize, size: usize) -> i32 {
        charge(size);
        let status = unsafe { posix_memalign(out, align, size) };

        settle(
            size,
            if status == 0 {
                unsafe { *out }
            } else {
                std::ptr::null_mut()
            },
        );
        status
    }

    extern "C" fn counted_aligned_alloc(align: usize, size: usize) -> *mut c_void {
        let mut p = std::ptr::null_mut();

        match counted_posix_memalign(&mut p, align.max(size_of::<usize>()), size) {
            0 => p,
            _ => std::ptr::null_mut(),
        }
    }

    /// Replaces the requested size charged up front (checked before the block exists) by the
    /// size libc actually reserved, or refunds it when the allocation failed.
    fn settle(requested: usize, p: *mut c_void) {
        if p.is_null() {
            refund(requested);
            return;
        }
        let actual = block_size(p);
        if actual > requested {
            charge(actual - requested);
        } else {
            refund(requested - actual);
        }
    }

    pub(super) fn foreign_override(symbol: &str) -> Option<u64> {
        let f = match symbol {
            "malloc" => counted_malloc as *const () as usize,
            "calloc" => counted_calloc as *const () as usize,
            "realloc" => counted_realloc as *const () as usize,
            "free" => counted_free as *const () as usize,
            "posix_memalign" => counted_posix_memalign as *const () as usize,
            "aligned_alloc" => counted_aligned_alloc as *const () as usize,
            _ => return None,
        };
        Some(f as u64)
    }
}

#[cfg(not(unix))]
mod sys {
    pub(super) fn write_stderr(bytes: &[u8]) {
        use std::io::Write;
        let _ = std::io::stderr().write_all(bytes);
    }

    pub(super) fn exit_now(code: i32) -> ! {
        std::process::exit(code)
    }

    /// Windows programs allocate through the CRT or `HeapAlloc`, which are not counted.
    pub(super) fn foreign_override(_symbol: &str) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn sizes() {
        assert_eq!(parse_size("1234"), Some(1234));
        assert_eq!(parse_size("64M"), Some(64 << 20));
        assert_eq!(parse_size("3g"), Some(3 << 30));
        assert_eq!(parse_size("2 K"), Some(2048));
        assert_eq!(parse_size("lots"), None);
        assert_eq!(parse_size(""), None);
    }
}
