//! The one definition of integer arithmetic on a type of a given width and signedness.
//!
//! Constant folding in `sema`, the interpreter's register operations, its constant propagation
//! and the editor's interval analysis all compute integer results here, so they cannot disagree
//! with each other. The compiled program's behaviour is the authority:
//!
//! - `+`, `-` and `*` wrap to the width.
//! - Division and remainder by zero are an error for the caller to report; `MIN / -1` wraps to
//!   `MIN` and `MIN % -1` is `0`.
//! - A shift by the width or more (or by a negative amount, which is a huge unsigned one) gives
//!   `0`, except a signed right shift, which fills with the sign bit.
//! - A rotate takes its amount modulo the width.
//! - A float converts to an integer by truncating toward zero, saturating at the type's range;
//!   `NaN` is `0`.

/// A binary integer operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    /// Arithmetic when the type is signed, logical when it is not.
    Shr,
    Rotl,
    Rotr,
}

/// The divisor of a `Div` or `Rem` was zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DivideByZero;

/// `v` as a value of an integer type of `bits` bits: its low bits, sign-extended when `signed`.
/// A width of 128 or more keeps `v` whole.
pub fn wrap(v: i128, bits: u8, signed: bool) -> i128 {
    if bits >= 128 {
        return v;
    }
    let mask = (1i128 << bits) - 1;
    let x = v & mask;
    if signed && bits > 0 && (x >> (bits - 1)) & 1 == 1 {
        x - (1i128 << bits)
    } else {
        x
    }
}

/// `lhs op rhs` on a type of `bits` bits (at most 128), `signed` or not.
///
/// `lhs` and, except for a shift or rotate (where it is the amount), `rhs` are values of the
/// type. The result is too.
#[inline(always)]
pub fn int_binop(
    op: IntOp,
    lhs: i128,
    rhs: i128,
    bits: u8,
    signed: bool,
) -> Result<i128, DivideByZero> {
    let (a, b) = (lhs, rhs);
    let in_range = (0..i128::from(bits)).contains(&b);
    let r = match op {
        IntOp::Add => a.wrapping_add(b),
        IntOp::Sub => a.wrapping_sub(b),
        IntOp::Mul => a.wrapping_mul(b),
        IntOp::Div | IntOp::Rem if b == 0 => return Err(DivideByZero),
        IntOp::Div => a.wrapping_div(b),
        IntOp::Rem => a.wrapping_rem(b),
        IntOp::And => a & b,
        IntOp::Or => a | b,
        IntOp::Xor => a ^ b,
        IntOp::Shl if in_range => a << b,
        IntOp::Shl => 0,
        IntOp::Shr if in_range => a >> b,
        IntOp::Shr if signed && a < 0 => -1,
        IntOp::Shr => 0,
        IntOp::Rotl | IntOp::Rotr => {
            let width = u32::from(bits);
            let by = b.rem_euclid(i128::from(bits)) as u32;
            let by = if op == IntOp::Rotr {
                (width - by) % width
            } else {
                by
            };
            let pattern = if bits >= 128 {
                a as u128
            } else {
                a as u128 & ((1u128 << bits) - 1)
            };
            if by == 0 {
                a
            } else {
                ((pattern << by) | (pattern >> (width - by))) as i128
            }
        }
    };
    Ok(wrap(r, bits, signed))
}

/// The integer a float casts to in a type of `bits` bits (at most 64), `signed` or not.
pub fn float_to_int(x: f64, bits: u8, signed: bool) -> i128 {
    let (lo, hi) = if signed {
        (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
    } else {
        (0, (1i128 << bits) - 1)
    };
    // `as` saturates at the range of `i128` and turns `NaN` into 0.
    (x as i128).clamp(lo, hi)
}

#[cfg(test)]
mod tests;
