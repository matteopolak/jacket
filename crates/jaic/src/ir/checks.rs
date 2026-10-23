//! Runtime check reasons and the messages that word them.
use crate::macros::raw_enum;

raw_enum! {
    /// Why a runtime check failed: the reason operand of `Intrinsic::Trap` and
    /// `Intrinsic::CheckFailed`. `stdlib/internal/runtime/checks.jai` words the same numbers for
    /// native code.
    pub enum CheckReason {
        /// A procedure with results fell off the end of its body (`Intrinsic::Trap`).
        MissingReturn = 1,
        /// An `#asm` divide faulted (`#DE`: divisor zero or quotient too big; `Intrinsic::Trap`).
        AsmDivide = 2,
        /// An array index out of range (a: the index, b: the count).
        Bounds = 3,
        /// An integer cast whose value does not fit the target (a: the value's bits, b:
        /// `cast_check_code`).
        CastOverflow = 4,
        /// A `#complete` switch without a default matched no case (a: the value).
        SwitchUnmatched = 5,
        /// An integer division or remainder by zero.
        DivideByZero = 6,
        /// A load, store or copy through a null pointer, or just past it, as in a member of a
        /// null struct pointer (a: `NULL_READ`..`NULL_FILL`, b: the address).
        NullPointer = 7,
        /// A misaligned atomic address (a: the address, b: the alignment it needs).
        AtomicAlignment = 8,
        /// A variadic segment larger than the pack's remaining room (a: the count, b: the room).
        VariadicSize = 9,
    }
}

/// `a` of a `CheckReason::NullPointer` failure: what the access did.
pub const NULL_READ: u64 = 0;

pub const NULL_WRITE: u64 = 1;
pub const NULL_COPY: u64 = 2;
pub const NULL_FILL: u64 = 3;

/// `b` of a `CheckReason::CastOverflow` failure: the target's size in bytes (low byte), whether the
/// target is signed (bit 8), whether the value is (bit 9) and whether the value is 64 bits wide
/// (bit 10).
pub fn cast_check_code(
    target_bytes: u64,
    target_signed: bool,
    value_signed: bool,
    value_bytes: u64,
) -> u64 {
    target_bytes
        | u64::from(target_signed) << 8
        | u64::from(value_signed) << 9
        | u64::from(value_bytes == 8) << 10
}

/// Whether a `cast_check_code` names a cast between `s64` and `u64`, which only changes how the
/// same bits are read.
pub fn cast_check_is_64_bit_sign_change(code: u64) -> bool {
    code & 0xff == 8 && code & 0x400 != 0 && (code >> 8 & 1) != (code >> 9 & 1)
}

/// The integer type a `cast_check_code` names (`u8`, `s32`, ...).
pub fn cast_check_target(code: u64) -> String {
    let signed = code & 0x100 != 0;
    format!(
        "{}{}",
        if signed {
            's'
        } else {
            'u'
        },
        (code & 0xff) * 8
    )
}

/// What a failed runtime check says, the same in the interpreter and in native code
/// (`stdlib/internal/runtime/checks.jai`, `__jaic_check_failed`).
/// `None` is a trap without a reason.
pub fn check_message(reason: impl Into<Option<CheckReason>>, a: u64, b: u64) -> String {
    let Some(reason) = reason.into() else {
        return "runtime check failed".into();
    };
    match reason {
        CheckReason::MissingReturn => {
            "reached the end of a procedure that must return a value".into()
        }
        CheckReason::AsmDivide => {
            "#asm division fault: the divisor is zero or the quotient does not fit".into()
        }
        CheckReason::Bounds => {
            let (index, count) = (a as i64, b as i64);
            format!(
                "array bounds check failed: index {index} is outside an array of {count} element{}",
                if count == 1 {
                    ""
                } else {
                    "s"
                }
            )
        }
        CheckReason::CastOverflow => {
            let value = if b & 0x200 != 0 {
                (a as i64).to_string()
            } else {
                a.to_string()
            };
            format!("cast of {value} to `{}` overflows", cast_check_target(b))
        }
        CheckReason::DivideByZero => "integer division by zero".into(),
        CheckReason::NullPointer => {
            let kind = match a {
                NULL_WRITE => "write",
                NULL_COPY => "memory copy",
                NULL_FILL => "memory fill",
                _ => "read",
            };
            if b == 0 {
                format!("null pointer dereference: {kind} through a null pointer")
            } else {
                format!(
                    "null pointer dereference: {kind} at address {b:#x}, just past null (a member of a null struct pointer?)"
                )
            }
        }
        CheckReason::SwitchUnmatched => {
            format!(
                "no case of the `#complete` switch matches its value, {}",
                a as i64
            )
        }
        CheckReason::AtomicAlignment => {
            format!("atomic address {a:#x} must be aligned to {b} bytes")
        }
        CheckReason::VariadicSize => {
            format!("variadic segment count {a} exceeds the remaining pack capacity {b}")
        }
    }
}
