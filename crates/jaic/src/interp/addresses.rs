//! How a program's addresses map to host memory: the one place that knows a program address is
//! an offset into a window (`window.rs`) and is only as wide as the target's pointer.
//!
//! A program value of pointer type is kept in a register as the target's pointer: an integer of
//! `pointer_size` bytes, zero-extended. Every computation that makes an address (pointer
//! arithmetic, an integer cast to a pointer) wraps at that width, as the native program's does,
//! and every access turns the address into a host address here. A fault the host reports at a
//! host address is shown to the user as the program's address. Byte order stays with the
//! `DataLayout`: this map moves addresses, not the bytes stored at them.
use crate::ir::Ty;

/// The address space of one interpreted program. With host-sized pointers it is the host's own
/// (`HOST`); a narrower program gets one from its window (`Window::addresses`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AddressMap {
    /// The host address of program address 0.
    base: u64,
    /// The program's pointer size in bytes.
    pointer_size: u8,
    /// The bits a program address keeps.
    mask: u64,
    /// The lowest address that is a procedure value, not data (`proc_addr::data_limit`).
    data_limit: u64,
    /// The lowest address the window ever hands out; 0 without a window.
    first: u64,
}

impl AddressMap {
    /// Program addresses are host addresses.
    pub(crate) const HOST: AddressMap = AddressMap {
        base: 0,
        pointer_size: 8,
        mask: u64::MAX,
        data_limit: super::FUNC_TAG,
        first: 0,
    };

    /// The addresses of a window at host address `base` for a program with `pointer_size`-byte
    /// pointers, whose memory starts at offset `first` and ends before `data_limit`.
    pub(super) fn window(base: u64, pointer_size: u8, first: u64, data_limit: u64) -> Self {
        Self {
            base,
            pointer_size,
            mask: u64::MAX >> (64 - 8 * u32::from(pointer_size)),
            data_limit,
            first,
        }
    }

    /// The program's pointer size in bytes: 8 when its addresses are the host's.
    #[inline(always)]
    pub(crate) fn pointer_size(self) -> u8 {
        self.pointer_size
    }

    /// The lowest address that is a procedure value, not data.
    #[inline(always)]
    pub(crate) fn data_limit(self) -> u64 {
        self.data_limit
    }

    /// The host address of program address `addr`.
    #[inline(always)]
    pub(crate) fn host(self, addr: u64) -> u64 {
        addr.wrapping_add(self.base)
    }

    /// The program address of host address `host`, the inverse of `host`.
    #[inline(always)]
    pub(crate) fn program(self, host: u64) -> u64 {
        host.wrapping_sub(self.base) & self.mask
    }

    /// The address to name for a fault the host reports at `host`: the program's address when
    /// `host` is in its address space, else `host` itself (memory outside the program).
    pub(crate) fn reported(self, host: u64) -> u64 {
        let offset = host.wrapping_sub(self.base);

        if offset <= self.mask {
            offset
        } else {
            host
        }
    }

    /// `addr + offset`, wrapping at the program's pointer width.
    #[inline(always)]
    pub(crate) fn offset(self, addr: u64, offset: u64) -> u64 {
        addr.wrapping_add(offset) & self.mask
    }

    /// Whether `offset` added to an address of the program's own memory can never wrap: true
    /// for every offset with host-sized pointers, and for those that stay inside the narrow
    /// address space from every address the window hands out.
    pub(crate) fn never_wraps(self, offset: i64) -> bool {
        if self.mask == u64::MAX {
            return true;
        }

        let room = self.mask - self.data_limit;
        -(self.first as i64) <= offset && offset <= room as i64
    }

    /// The type a value of IR type `ty` has in a register and in memory: a pointer is the
    /// integer of the program's pointer width.
    #[inline(always)]
    pub(crate) fn int_ty(self, ty: Ty) -> Ty {
        if ty == Ty::Ptr {
            Ty::int(u64::from(self.pointer_size))
        } else {
            ty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn narrow() -> AddressMap {
        AddressMap::window(0x7000_0000_0000, 4, 64 << 10, 0xC000_0000)
    }

    #[test]
    fn host_addresses_map_to_themselves() {
        let map = AddressMap::HOST;

        assert_eq!(map.host(0x1234), 0x1234);
        assert_eq!(map.program(0x1234), 0x1234);
        assert_eq!(map.offset(u64::MAX - 15, 32), 16);
        assert_eq!(map.int_ty(Ty::Ptr), Ty::I64);
        assert!(map.never_wraps(i64::MIN) && map.never_wraps(i64::MAX));
    }

    #[test]
    fn a_narrow_address_wraps_at_its_width_and_is_reported_as_the_program_sees_it() {
        let map = narrow();

        assert_eq!(map.offset(0xFFFF_FFF0, 0x20), 0x10);
        assert_eq!(map.offset(0x10, (-32i64) as u64), 0xFFFF_FFF0);
        assert_eq!(map.host(0x8000_0000), 0x7000_8000_0000);
        assert_eq!(map.program(0x7000_8000_0000), 0x8000_0000);
        assert_eq!(map.reported(0x7000_8000_0000), 0x8000_0000);
        assert_eq!(
            map.reported(0x10),
            0x10,
            "outside the window the host address stays"
        );
        assert_eq!(map.int_ty(Ty::Ptr), Ty::I32);
        assert!(map.never_wraps(-(64 << 10)) && map.never_wraps(0x3FFF_FFFF));
        assert!(!map.never_wraps(0x4000_0000) && !map.never_wraps(-(64 << 10) - 1));
    }
}
