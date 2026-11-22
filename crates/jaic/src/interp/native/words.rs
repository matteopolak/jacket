//! Register-sized reads and writes of program memory for native foreign calls.

/// Little-endian bytes `[addr, addr + len)` (len <= 8) as a `u64`.
///
/// SAFETY: interpreter addresses are host addresses of live memory.
#[allow(
    clippy::disallowed_methods,
    reason = "registers of a native foreign call hold host words"
)]
pub(super) unsafe fn read_bytes(addr: u64, len: u64) -> u64 {
    let mut out = [0u8; 8];
    let len = len.min(8) as usize;
    // SAFETY: the caller guarantees `[addr, addr + len)` is readable; `out` has room for 8.
    unsafe { std::ptr::copy_nonoverlapping(addr as *const u8, out.as_mut_ptr(), len) };
    u64::from_le_bytes(out)
}

/// SAFETY: as `read_bytes`.
#[allow(
    clippy::disallowed_methods,
    reason = "registers of a native foreign call hold host words"
)]
pub(super) unsafe fn write_bytes(addr: u64, value: u64, len: u64) {
    let bytes = value.to_le_bytes();
    let len = len.min(8) as usize;
    // SAFETY: the caller guarantees `[addr, addr + len)` is writable; `bytes` has 8.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, len) };
}
