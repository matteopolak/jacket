//! The sandboxed program's memory, reached through the addresses it passes to the libc
//! stand-ins.
//!
//! The interpreter hands programs real host addresses and checks none of them: an interpreted
//! load or store dereferences its address as is. A stand-in that reads or writes through an
//! argument is therefore exactly as trusted as the program's own accesses, and a wrong address
//! is the program's bug, as it would be in native code. These functions are the one place that
//! says so; the stand-ins call them instead of dereferencing raw addresses themselves.

/// The byte at `addr`.
pub(super) fn read_u8(addr: u64) -> u8 {
    // SAFETY: see the module documentation.
    unsafe { *(addr as *const u8) }
}

/// The unaligned `u32` at `addr`.
#[cfg(test)]
pub(super) fn read_u32(addr: u64) -> u32 {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::read_unaligned(addr as *const u32) }
}

/// The unaligned `u64` at `addr`.
pub(super) fn read_u64(addr: u64) -> u64 {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::read_unaligned(addr as *const u64) }
}

/// The unaligned `i64` at `addr`.
pub(super) fn read_i64(addr: u64) -> i64 {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::read_unaligned(addr as *const i64) }
}

/// A copy of the `len` bytes at `addr`.
pub(super) fn read_bytes(addr: u64, len: usize) -> Vec<u8> {
    if len == 0 {
        // A zero-length read may pass a null address, which a slice must not hold.
        return Vec::new();
    }
    // SAFETY: see the module documentation.
    unsafe { std::slice::from_raw_parts(addr as *const u8, len) }.to_vec()
}

/// Stores the byte `value` at `addr`.
pub(super) fn write_u8(addr: u64, value: u8) {
    // SAFETY: see the module documentation.
    unsafe { *(addr as *mut u8) = value }
}

/// Stores the unaligned `u16` `value` at `addr`.
pub(super) fn write_u16(addr: u64, value: u16) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_unaligned(addr as *mut u16, value) }
}

/// Stores the unaligned `u32` `value` at `addr`.
pub(super) fn write_u32(addr: u64, value: u32) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_unaligned(addr as *mut u32, value) }
}

/// Stores the unaligned `i32` `value` at `addr`.
pub(super) fn write_i32(addr: u64, value: i32) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_unaligned(addr as *mut i32, value) }
}

/// Stores the unaligned `u64` `value` at `addr`.
pub(super) fn write_u64(addr: u64, value: u64) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_unaligned(addr as *mut u64, value) }
}

/// Stores `bytes` at `addr`.
pub(super) fn write_bytes(addr: u64, bytes: &[u8]) {
    // SAFETY: see the module documentation; `bytes` is host memory, so it cannot overlap.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), addr as *mut u8, bytes.len()) };
}

/// Stores `units` at `addr`, which need not be aligned for `T`: a name inside a structure the
/// program laid out is as aligned as the program made it.
pub(super) fn write_units<T: Copy>(addr: u64, units: &[T]) {
    // SAFETY: see the module documentation; `units` is host memory, so it cannot overlap. The copy
    // is of bytes, so `addr` has no alignment to keep.
    unsafe {
        std::ptr::copy_nonoverlapping(
            units.as_ptr().cast::<u8>(),
            addr as *mut u8,
            std::mem::size_of_val(units),
        );
    }
}

/// Copies `len` bytes from `src` to `dst`; the ranges may overlap.
pub(super) fn copy(dst: u64, src: u64, len: usize) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::copy(src as *const u8, dst as *mut u8, len) };
}

/// Sets the `len` bytes at `dst` to `byte`.
pub(super) fn fill(dst: u64, byte: u8, len: usize) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_bytes(dst as *mut u8, byte, len) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_are_stored_at_an_odd_address() {
        let mut buffer = [0u8; 8];
        let at = buffer.as_mut_ptr() as u64 + 1;
        write_units(at, &[0x0201u16, 0x0403]);
        assert_eq!(buffer[1..5], [1, 2, 3, 4]);
    }
}
