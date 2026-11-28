//! How a procedure's address looks to the program.
//!
//! The interpreter runs procedures by id, so a procedure value is a tag and an id that no load
//! or store may dereference. With host-sized pointers the tag is the top 16 bits. A program
//! with 4-byte pointers has the top 1 GiB of its address space for them (above the window,
//! `window::SPAN`): a procedure is `FUNC_BASE32 | id`, a foreign one `FOREIGN_BASE32 | id`. With
//! 2-byte pointers (AVR) each kind gets 8192 values at the top of the 64 KiB range.
//!
//! `width` below is the program's pointer size in bytes, 8 when its addresses are the host's.
use crate::ir::{ForeignId, FuncId};

use super::{FOREIGN_TAG, FUNC_TAG, TAG_MASK};

/// Procedure values of a program with 4-byte pointers.
pub(super) const FUNC_BASE32: u64 = 0xE000_0000;

pub(super) const FOREIGN_BASE32: u64 = 0xF000_0000;
const ID_MASK32: u64 = 0x0FFF_FFFF;

/// Procedure values of a program with 2-byte pointers.
pub(super) const FUNC_BASE16: u64 = 0xC000;

pub(super) const FOREIGN_BASE16: u64 = 0xE000;
const ID_MASK16: u64 = 0x1FFF;

/// The procedure a procedure value stands for.
pub(super) enum Proc {
    Func(FuncId),
    Foreign(ForeignId),
}

/// The `(function base, foreign base, id mask)` of a narrow width.
fn narrow_bases(width: u8) -> (u64, u64, u64) {
    if width == 2 {
        (FUNC_BASE16, FOREIGN_BASE16, ID_MASK16)
    } else {
        (FUNC_BASE32, FOREIGN_BASE32, ID_MASK32)
    }
}

/// The value of interpreted procedure `id`.
pub(super) fn func_word(width: u8, id: FuncId) -> u64 {
    let base = if width < 8 {
        narrow_bases(width).0
    } else {
        FUNC_TAG
    };
    base | u64::from(id.0)
}

/// The value of foreign procedure `id` that has no native address.
pub(super) fn foreign_word(width: u8, id: ForeignId) -> u64 {
    let base = if width < 8 {
        narrow_bases(width).1
    } else {
        FOREIGN_TAG
    };
    base | u64::from(id.0)
}

/// What `value` names, when it is a procedure value.
pub(super) fn proc_of(width: u8, value: u64) -> Option<Proc> {
    if width < 8 {
        let (func, foreign, mask) = narrow_bases(width);
        let id = (value & mask) as u32;

        return match value & !mask {
            v if v == func => Some(Proc::Func(FuncId(id))),
            v if v == foreign => Some(Proc::Foreign(ForeignId(id))),
            _ => None,
        };
    }

    let id = (value & !TAG_MASK) as u32;

    match value & TAG_MASK {
        FUNC_TAG => Some(Proc::Func(FuncId(id))),
        FOREIGN_TAG => Some(Proc::Foreign(ForeignId(id))),
        _ => None,
    }
}

/// The lowest address that is not data: below it, and above the first page, an address is
/// plain memory (`arith::plain_address`).
pub(super) fn data_limit(width: u8) -> u64 {
    match width {
        2 => super::window::SPAN16 as u64,
        4 => super::window::SPAN as u64,
        _ => FUNC_TAG,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_for_every_width() {
        for width in [8u8, 4, 2] {
            let id = if width == 2 {
                7000
            } else {
                70_000
            };
            let f = func_word(width, FuncId(id));

            assert!(matches!(proc_of(width, f), Some(Proc::Func(FuncId(n))) if n == id));
            let e = foreign_word(width, ForeignId(3));

            assert!(matches!(
                proc_of(width, e),
                Some(Proc::Foreign(ForeignId(3)))
            ));
            assert!(proc_of(width, 0x1234).is_none());
            assert!(f >= data_limit(width) && e >= data_limit(width));
        }

        assert!(u32::try_from(func_word(4, FuncId(0x0FFF_FFFF))).is_ok());
        assert!(u16::try_from(func_word(2, FuncId(0x1FFF))).is_ok());
    }
}
