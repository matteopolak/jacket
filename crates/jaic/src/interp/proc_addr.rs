//! How a procedure's address looks to the program.
//!
//! The interpreter runs procedures by id, so a procedure value is a tag and an id that no load
//! or store may dereference. With host-sized pointers the tag is the top 16 bits. A program
//! with 4-byte pointers has the top 1 GiB of its address space for them (above the window,
//! `window::SPAN`): a procedure is `FUNC_BASE32 | id`, a foreign one `FOREIGN_BASE32 | id`.
use crate::ir::{ForeignId, FuncId};

use super::{FOREIGN_TAG, FUNC_TAG, TAG_MASK};

/// Procedure values of a program with 4-byte pointers.
pub(super) const FUNC_BASE32: u64 = 0xE000_0000;

pub(super) const FOREIGN_BASE32: u64 = 0xF000_0000;
const ID_MASK32: u64 = 0x0FFF_FFFF;

/// The procedure a procedure value stands for.
pub(super) enum Proc {
    Func(FuncId),
    Foreign(ForeignId),
}

/// The value of interpreted procedure `id`.
pub(super) fn func_word(narrow: bool, id: FuncId) -> u64 {
    let base = if narrow {
        FUNC_BASE32
    } else {
        FUNC_TAG
    };
    base | u64::from(id.0)
}

/// The value of foreign procedure `id` that has no native address.
pub(super) fn foreign_word(narrow: bool, id: ForeignId) -> u64 {
    let base = if narrow {
        FOREIGN_BASE32
    } else {
        FOREIGN_TAG
    };
    base | u64::from(id.0)
}

/// What `value` names, when it is a procedure value.
pub(super) fn proc_of(narrow: bool, value: u64) -> Option<Proc> {
    if narrow {
        let id = (value & ID_MASK32) as u32;
        return match value & !ID_MASK32 {
            FUNC_BASE32 => Some(Proc::Func(FuncId(id))),
            FOREIGN_BASE32 => Some(Proc::Foreign(ForeignId(id))),
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
pub(super) fn data_limit(narrow: bool) -> u64 {
    if narrow {
        super::window::SPAN as u64
    } else {
        FUNC_TAG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_round_trip_for_both_widths() {
        for narrow in [false, true] {
            let f = func_word(narrow, FuncId(70_000));
            assert!(matches!(
                proc_of(narrow, f),
                Some(Proc::Func(FuncId(70_000)))
            ));
            let e = foreign_word(narrow, ForeignId(3));
            assert!(matches!(
                proc_of(narrow, e),
                Some(Proc::Foreign(ForeignId(3)))
            ));
            assert!(proc_of(narrow, 0x1234_5678).is_none());
            assert!(f >= data_limit(narrow) && e >= data_limit(narrow));
        }
        assert!(func_word(true, FuncId(0x0FFF_FFFF)) <= u64::from(u32::MAX));
    }
}
