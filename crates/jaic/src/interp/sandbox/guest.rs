//! The sandboxed program's memory, reached through the addresses it passes to the libc
//! stand-ins, read and written at the program's own layout: where its addresses are in host
//! memory comes from the interpreter's `AddressMap`, the byte order and the widths of a pointer
//! and a `size_t` from its `DataLayout`, for the length of each call (`with_memory`).
//!
//! The interpreter hands programs real host addresses and checks none of them: an interpreted
//! load or store dereferences its address as is. A stand-in that reads or writes through an
//! argument is therefore exactly as trusted as the program's own accesses, and a wrong address
//! is the program's bug, as it would be in native code. These functions are the one place that
//! says so; the stand-ins call them instead of dereferencing raw addresses themselves.

use crate::interp::addresses::AddressMap;
use crate::layout::DataLayout;
use std::cell::Cell;

/// The program memory a stand-in sees.
#[derive(Clone, Copy)]
struct Memory {
    /// Where the program's addresses are in host memory.
    addresses: AddressMap,
    layout: DataLayout,
}

thread_local! {
    /// Set only while a stand-in runs (`with_memory`): there is no program memory otherwise,
    /// and no layout to guess.
    static MEMORY: Cell<Option<Memory>> = const { Cell::new(None) };
}

/// The program memory of the stand-in running on this thread.
#[inline]
fn memory() -> Memory {
    MEMORY
        .get()
        .expect("a libc stand-in reached program memory outside `with_memory`")
}

/// Run `f`, a stand-in, on program memory laid out as `layout` and mapped by `addresses`.
pub(in crate::interp) fn with_memory<R>(
    addresses: AddressMap,
    layout: DataLayout,
    f: impl FnOnce() -> R,
) -> R {
    let outer = MEMORY.replace(Some(Memory {
        addresses,
        layout,
    }));
    let result = f();
    MEMORY.set(outer);
    result
}

/// Make host memory the program memory of this thread's stand-ins from now on, for tests that
/// call them directly.
#[cfg(test)]
pub(in crate::interp) fn use_host_memory() {
    MEMORY.set(Some(Memory {
        addresses: AddressMap::HOST,
        layout: DataLayout::host(),
    }));
}

/// The layout of the program memory stand-ins are reaching.
pub(in crate::interp) fn layout() -> DataLayout {
    memory().layout
}

/// The host address of program address `addr`.
#[inline]
fn host(addr: u64) -> u64 {
    memory().addresses.host(addr)
}

/// The unsigned integer of `width` bytes at `addr`, in the program's byte order.
pub(in crate::interp) fn read_uint(addr: u64, width: u8) -> u64 {
    let mut bytes = [0u8; 8];
    let width = usize::from(width);
    // SAFETY: see the module documentation; `bytes` is host memory, so it cannot overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(host(addr) as *const u8, bytes.as_mut_ptr(), width);
    }
    layout().read_uint(&bytes[..width])
}

/// Stores the low `width` bytes of `value` at `addr`, in the program's byte order.
pub(in crate::interp) fn write_uint(addr: u64, width: u8, value: u64) {
    let mut bytes = [0u8; 8];
    let width = usize::from(width);
    layout().write_uint(&mut bytes[..width], value);
    write_bytes(addr, &bytes[..width]);
}

/// The pointer at `addr`, as wide as the program's.
pub(in crate::interp) fn read_word(addr: u64) -> u64 {
    read_uint(addr, layout().pointer_size)
}

/// Stores the pointer `value` at `addr`, as wide as the program's.
pub(in crate::interp) fn write_word(addr: u64, value: u64) {
    write_uint(addr, layout().pointer_size, value);
}

/// The `size_t` at `addr`.
pub(in crate::interp) fn read_size(addr: u64) -> u64 {
    read_uint(addr, layout().c.size_t)
}

/// Stores the `size_t` `value` at `addr`.
pub(in crate::interp) fn write_size(addr: u64, value: u64) {
    write_uint(addr, layout().c.size_t, value);
}

/// The byte at `addr`.
pub(in crate::interp) fn read_u8(addr: u64) -> u8 {
    // SAFETY: see the module documentation.
    unsafe { *(host(addr) as *const u8) }
}

/// The `u32` at `addr`, in the program's byte order.
#[cfg(test)]
pub(super) fn read_u32(addr: u64) -> u32 {
    read_uint(addr, 4) as u32
}

/// The `u64` at `addr`, in the program's byte order.
pub(in crate::interp) fn read_u64(addr: u64) -> u64 {
    read_uint(addr, 8)
}

/// The `i64` at `addr`, in the program's byte order.
pub(super) fn read_i64(addr: u64) -> i64 {
    read_uint(addr, 8) as i64
}

/// A copy of the `len` bytes at `addr`.
pub(in crate::interp) fn read_bytes(addr: u64, len: usize) -> Vec<u8> {
    if len == 0 {
        // A zero-length read may pass a null address, which a slice must not hold.
        return Vec::new();
    }
    // SAFETY: see the module documentation.
    unsafe { std::slice::from_raw_parts(host(addr) as *const u8, len) }.to_vec()
}

/// Stores the byte `value` at `addr`.
pub(super) fn write_u8(addr: u64, value: u8) {
    // SAFETY: see the module documentation.
    unsafe { *(host(addr) as *mut u8) = value }
}

/// Stores the `u16` `value` at `addr`, in the program's byte order.
pub(in crate::interp) fn write_u16(addr: u64, value: u16) {
    write_uint(addr, 2, value as u64);
}

/// Stores the `u32` `value` at `addr`, in the program's byte order.
pub(in crate::interp) fn write_u32(addr: u64, value: u32) {
    write_uint(addr, 4, value as u64);
}

/// Stores the `i32` `value` at `addr`, in the program's byte order.
pub(super) fn write_i32(addr: u64, value: i32) {
    write_uint(addr, 4, value as u32 as u64);
}

/// Stores the `u64` `value` at `addr`, in the program's byte order.
pub(in crate::interp) fn write_u64(addr: u64, value: u64) {
    write_uint(addr, 8, value);
}

/// Stores `bytes` at `addr`.
pub(in crate::interp) fn write_bytes(addr: u64, bytes: &[u8]) {
    // SAFETY: see the module documentation; `bytes` is host memory, so it cannot overlap.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), host(addr) as *mut u8, bytes.len()) };
}

/// Stores `units` at `addr`, which need not be aligned for `T`: a name inside a structure the
/// program laid out is as aligned as the program made it.
pub(in crate::interp) fn write_units<T: Copy>(addr: u64, units: &[T]) {
    // SAFETY: see the module documentation; `units` is host memory, so it cannot overlap. The copy
    // is of bytes, so `addr` has no alignment to keep.
    unsafe {
        std::ptr::copy_nonoverlapping(
            units.as_ptr().cast::<u8>(),
            host(addr) as *mut u8,
            std::mem::size_of_val(units),
        );
    }
}

/// Copies `len` bytes from `src` to `dst`; the ranges may overlap.
pub(super) fn copy(dst: u64, src: u64, len: usize) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::copy(host(src) as *const u8, host(dst) as *mut u8, len) };
}

/// Sets the `len` bytes at `dst` to `byte`.
pub(in crate::interp) fn fill(dst: u64, byte: u8, len: usize) {
    // SAFETY: see the module documentation.
    unsafe { std::ptr::write_bytes(host(dst) as *mut u8, byte, len) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_are_stored_at_an_odd_address() {
        use_host_memory();
        let mut buffer = [0u8; 8];
        let at = buffer.as_mut_ptr() as u64 + 1;
        write_units(at, &[0x0201u16, 0x0403]);
        assert_eq!(buffer[1..5], [1, 2, 3, 4]);
    }
}
