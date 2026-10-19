//! Reading and writing program memory, for the compiler and for interpreted loads and stores.
use super::arith::plain_address;
use super::trap::{Trap, null_access};
use super::{FOREIGN_TAG, FUNC_TAG, Interp, Res, TAG_MASK, code};
use crate::ir::Ty;

impl Interp {
    // The compiler's own accesses to program memory (reading `#run` results, the arguments of
    // `compiler_*` calls, writing their out-parameters) go through `probe`, so an address the
    // program made up is an error instead of a crash. Interpreted loads and stores do not: they
    // behave like the native program's.

    /// `len` bytes of program memory at `addr`, or `None` when they cannot all be read.
    pub fn read(&self, addr: u64, len: usize) -> Option<Vec<u8>> {
        self.probe.read_vec(addr, len)
    }

    /// Fill `out` from program memory at `addr`; false when it cannot all be read.
    pub fn read_into(&self, addr: u64, out: &mut [u8]) -> bool {
        self.probe.read(addr, out)
    }

    /// The `u64` at `addr` in program memory, or `None` when it cannot be read.
    pub fn read_u64(&self, addr: u64) -> Option<u64> {
        let mut out = [0; 8];
        self.probe
            .read(addr, &mut out)
            .then(|| u64::from_le_bytes(out))
    }

    /// The two `u64`s at `addr` (a Jai `string` or view: count, then data), or `None` when
    /// they cannot be read.
    pub fn read_pair(&self, addr: u64) -> Option<(u64, u64)> {
        let mut out = [0; 16];
        if !self.probe.read(addr, &mut out) {
            return None;
        }
        let (a, b) = out.split_at(8);
        Some((
            u64::from_le_bytes(a.try_into().ok()?),
            u64::from_le_bytes(b.try_into().ok()?),
        ))
    }

    /// Store `bytes` at `addr` in program memory; false when it cannot be written.
    #[must_use]
    pub fn write(&mut self, addr: u64, bytes: &[u8]) -> bool {
        self.probe.write(addr, bytes)
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
        if !plain_address(addr) {
            return self.load_unusual(ty, addr);
        }
        // SAFETY: the address is the program's own, as for native code.
        Ok(unsafe { code::load_raw(ty, addr) })
    }

    #[cold]
    fn load_unusual(&self, ty: Ty, addr: u64) -> Res<u64> {
        if addr < 4096 {
            return self.null_trap(null_access("read", addr));
        }
        if addr & TAG_MASK == FUNC_TAG || addr & TAG_MASK == FOREIGN_TAG {
            return self.trap("read through a procedure address");
        }
        // SAFETY: the address is the program's own, as for native code.
        Ok(unsafe { code::load_raw(ty, addr) })
    }

    #[inline(always)]
    pub(super) fn store(&self, ty: Ty, addr: u64, v: u64) -> Res<()> {
        if addr < 4096 || self.read_only.contains(addr) {
            return self.refused_store(addr);
        }
        // SAFETY: the address is the program's own, as for native code.
        unsafe { code::store_raw(ty, addr, v) };
        Ok(())
    }

    /// A trap when interpreted code would write `len` bytes at `addr` in read-only data.
    #[inline(always)]
    pub(super) fn check_writable(&self, addr: u64, len: usize) -> Res<()> {
        if self.read_only.overlaps(addr, len) {
            return self.refused_store(addr);
        }
        Ok(())
    }

    #[cold]
    pub(super) fn refused_store<T>(&self, addr: u64) -> Res<T> {
        if addr < 4096 {
            return self.null_trap(null_access("write", addr));
        }
        self.trap(format!(
            "cannot write to a string literal or other constant (read-only memory at {addr:#x}); copy it first, for example with `copy_string`"
        ))
    }
}
