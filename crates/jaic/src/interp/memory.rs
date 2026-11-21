//! Reading and writing program memory, for the compiler and for interpreted loads and stores.
use super::arith::plain_address;
use super::proc_addr::proc_of;
use super::trap::{Trap, null_access};
use super::{Interp, Res, code};
use crate::ir::Ty;

impl Interp {
    /// The host address of program address `addr`: the same address, or its offset into the
    /// window when the program's pointers are narrower than the host's.
    #[inline(always)]
    pub(crate) fn host_addr(&self, addr: u64) -> u64 {
        addr.wrapping_add(self.base)
    }

    /// The type an access moves: a pointer is as wide as the target's.
    #[inline(always)]
    fn memory_ty(&self, ty: Ty) -> Ty {
        if ty == Ty::Ptr {
            self.ptr_ty
        } else {
            ty
        }
    }

    /// The unsigned integer of `size` bytes at `addr`, in the program's byte order.
    ///
    /// SAFETY: `addr..addr + size` is readable program memory.
    pub(super) unsafe fn peek(&self, addr: u64, size: usize) -> u64 {
        let mut bytes = [0; 8];
        // SAFETY: the caller checked that the bytes are readable.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.host_addr(addr) as *const u8,
                bytes.as_mut_ptr(),
                size,
            );
        }
        self.layout.read_uint(&bytes[..size])
    }

    /// Store the `size`-byte unsigned integer `value` at `addr`, in the program's byte order.
    ///
    /// SAFETY: `addr..addr + size` is writable program memory.
    pub(super) unsafe fn poke(&self, addr: u64, size: usize, value: u64) {
        let mut bytes = [0; 8];
        self.layout.write_uint(&mut bytes[..size], value);
        // SAFETY: the caller checked that the bytes are writable.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.host_addr(addr) as *mut u8, size);
        }
    }

    /// The program address of host address `host`, the inverse of `host_addr`.
    #[inline(always)]
    pub(crate) fn program_addr(&self, host: u64) -> u64 {
        host.wrapping_sub(self.base)
    }

    // The compiler's own accesses to program memory (reading `#run` results, the arguments of
    // `compiler_*` calls, writing their out-parameters) go through `probe`, so an address the
    // program made up is an error instead of a crash. Interpreted loads and stores do not: they
    // behave like the native program's.

    /// `len` bytes of program memory at `addr`, or `None` when they cannot all be read.
    pub fn read(&self, addr: u64, len: usize) -> Option<Vec<u8>> {
        self.probe.read_vec(self.host_addr(addr), len)
    }

    /// Fill `out` from program memory at `addr`; false when it cannot all be read.
    pub fn read_into(&self, addr: u64, out: &mut [u8]) -> bool {
        self.probe.read(self.host_addr(addr), out)
    }

    /// The `s64`/`u64` at `addr` in program memory, or `None` when it cannot be read.
    pub fn read_u64(&self, addr: u64) -> Option<u64> {
        let mut out = [0; 8];
        self.probe
            .read(self.host_addr(addr), &mut out)
            .then(|| self.layout.read_u64(&out))
    }

    /// The pointer at `addr` in program memory, or `None` when it cannot be read.
    pub fn read_ptr(&self, addr: u64) -> Option<u64> {
        let mut out = [0; 8];
        let out = &mut out[..self.layout.pointer_size as usize];
        self.probe
            .read(self.host_addr(addr), out)
            .then(|| self.layout.read_ptr(out))
    }

    /// The count and data pointer of the `string` or view at `addr`, or `None` when they
    /// cannot be read.
    pub fn read_view(&self, addr: u64) -> Option<(u64, u64)> {
        let view = self.layout.builtins().view;
        Some((
            self.read_u64(addr + view.count)?,
            self.read_ptr(addr + view.data)?,
        ))
    }

    /// Store `bytes` at `addr` in program memory; false when it cannot be written.
    #[must_use]
    pub fn write(&mut self, addr: u64, bytes: &[u8]) -> bool {
        self.probe.write(self.host_addr(addr), bytes)
    }

    /// `read` for a pointer an intrinsic or emulated library call was given: a trap when the
    /// memory cannot be read.
    pub(super) fn fetch(&self, addr: u64, len: usize) -> Res<Vec<u8>> {
        match self.read(addr, len) {
            Some(bytes) => Ok(bytes),
            None => self.unreadable(addr, len),
        }
    }

    /// `read_u64` for a pointer an intrinsic or emulated library call was given.
    pub(super) fn fetch_u64(&self, addr: u64) -> Res<u64> {
        match self.read_u64(addr) {
            Some(v) => Ok(v),
            None => self.unreadable(addr, 8),
        }
    }

    /// `read_view` for a `string` or view an intrinsic or emulated library call was given.
    pub(super) fn fetch_view(&self, addr: u64) -> Res<(usize, u64)> {
        let size = self.layout.builtins().view.size as usize;
        match self.read_view(addr) {
            Some((count, data)) => Ok((count as usize, data)),
            None => self.unreadable(addr, size),
        }
    }

    /// Store the pointer-sized `value` at `addr`: a trap when the memory cannot be written.
    pub(super) fn put_ptr(&mut self, addr: u64, value: u64) -> Res<()> {
        let size = self.layout.pointer_size as usize;
        let mut bytes = [0; 8];
        self.layout.write_ptr(&mut bytes, value);
        self.put(addr, &bytes[..size])
    }

    /// `write` for a pointer an intrinsic or emulated library call was given: a trap when the
    /// memory cannot be written.
    pub(super) fn put(&mut self, addr: u64, bytes: &[u8]) -> Res<()> {
        self.check_writable(addr, bytes.len())?;
        if self.write(addr, bytes) {
            return Ok(());
        }
        if addr < 4096 {
            return self.null_trap(null_access("write", addr));
        }
        Err(Trap::unwritable(addr, bytes.len()).at(self.loc))
    }

    fn unreadable<T>(&self, addr: u64, len: usize) -> Res<T> {
        if addr < 4096 {
            return self.null_trap(null_access("read", addr));
        }
        Err(Trap::unreadable(addr, len).at(self.loc))
    }

    /// Load a `ty` from program memory, as interpreted code does: only an address in the
    /// never-mapped first page, or a procedure's tagged address, is refused.
    #[inline(always)]
    pub(super) fn load(&self, ty: Ty, addr: u64) -> Res<u64> {
        let ty = self.memory_ty(ty);
        if !plain_address(addr, self.data_limit) {
            return self.load_unusual(ty, addr);
        }
        // SAFETY: the address is the program's own, as for native code.
        Ok(unsafe { code::load_raw(ty, self.host_addr(addr)) })
    }

    #[cold]
    fn load_unusual(&self, ty: Ty, addr: u64) -> Res<u64> {
        if addr < 4096 {
            return self.null_trap(null_access("read", addr));
        }
        if proc_of(self.window.is_some(), addr).is_some() {
            return self.trap("read through a procedure address");
        }
        if addr >= self.data_limit {
            return self.trap(format!("read through an address no memory has ({addr:#x})"));
        }
        // SAFETY: the address is the program's own, as for native code.
        Ok(unsafe { code::load_raw(ty, self.host_addr(addr)) })
    }

    #[inline(always)]
    pub(super) fn store(&self, ty: Ty, addr: u64, v: u64) -> Res<()> {
        let ty = self.memory_ty(ty);
        let host = self.host_addr(addr);
        if addr.wrapping_sub(4096) >= self.data_limit - 4096 || self.read_only.contains(host) {
            return self.refused_store(addr);
        }
        // SAFETY: the address is the program's own, as for native code.
        unsafe { code::store_raw(ty, host, v) };
        Ok(())
    }

    /// A trap when interpreted code would write `len` bytes at `addr` in read-only data.
    #[inline(always)]
    pub(super) fn check_writable(&self, addr: u64, len: usize) -> Res<()> {
        if self.read_only.overlaps(self.host_addr(addr), len) {
            return self.refused_store(addr);
        }
        Ok(())
    }

    #[cold]
    pub(super) fn refused_store<T>(&self, addr: u64) -> Res<T> {
        if addr < 4096 {
            return self.null_trap(null_access("write", addr));
        }
        if addr >= self.data_limit {
            return self.trap(format!(
                "write through an address no memory has ({addr:#x})"
            ));
        }
        self.trap(format!(
            "cannot write to a string literal or other constant (read-only memory at {addr:#x}); copy it first, for example with `copy_string`"
        ))
    }
}
