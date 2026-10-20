//! Integer arithmetic, comparison and conversion on raw register words.
use super::FUNC_TAG;
use crate::ir::{BinOp, CmpOp, ConvOp, Ty};


/// Is `addr` neither in the first page nor a tagged procedure address? One comparison covers
/// both: the tags are the highest addresses used, and subtracting the first page wraps the
/// null page around to the top. An address that fails it is looked at again, closely.
#[inline(always)]
pub(super) fn plain_address(addr: u64) -> bool {
    addr.wrapping_sub(4096) < FUNC_TAG - 4096
}

/// Is `op` one of the binary operations that can fail (`Interp::divide`)?
#[inline]
pub(crate) fn divides(op: BinOp) -> bool {
    matches!(op, BinOp::SDiv | BinOp::SRem | BinOp::UDiv | BinOp::URem)
}

/// A binary operation other than a division, which cannot fail.
#[inline(always)]
pub(crate) fn bin_total(op: BinOp, ty: Ty, x: u64, y: u64) -> u64 {
    let bits = ty.size() as u32 * 8;
    match op {
        BinOp::Add => mask(ty, x.wrapping_add(y)),
        BinOp::Sub => mask(ty, x.wrapping_sub(y)),
        BinOp::Mul => mask(ty, x.wrapping_mul(y)),
        BinOp::SDiv | BinOp::SRem | BinOp::UDiv | BinOp::URem => {
            unreachable!("divisions go through `Interp::divide`")
        }
        BinOp::And => x & y,
        BinOp::Or => x | y,
        BinOp::Xor => x ^ y,
        BinOp::Shl => {
            if y >= bits as u64 {
                0
            } else {
                mask(ty, x << y)
            }
        }
        BinOp::LShr => {
            if y >= bits as u64 {
                0
            } else {
                x >> y
            }
        }
        BinOp::AShr => mask(ty, (sext(ty, x) >> y.min(63)) as u64),
        BinOp::Rotl | BinOp::Rotr => {
            let s = (y % bits as u64) as u32;
            let s = if op == BinOp::Rotr {
                (bits - s) % bits
            } else {
                s
            };
            rotate_left(ty, x, s)
        }
        BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
            if ty == Ty::F32 {
                let (a, b) = (f32::from_bits(x as u32), f32::from_bits(y as u32));
                (match op {
                    BinOp::FAdd => a + b,
                    BinOp::FSub => a - b,
                    BinOp::FMul => a * b,
                    _ => a / b,
                })
                .to_bits() as u64
            } else {
                let (a, b) = (f64::from_bits(x), f64::from_bits(y));
                (match op {
                    BinOp::FAdd => a + b,
                    BinOp::FSub => a - b,
                    BinOp::FMul => a * b,
                    _ => a / b,
                })
                .to_bits()
            }
        }
    }
}

/// `x` rotated left by `s` bits (`s` below the width of `ty`).
fn rotate_left(ty: Ty, x: u64, s: u32) -> u64 {
    if s == 0 {
        return x;
    }
    // The two halves share no bits, so joining them with `|` or `^` is the same.
    mask(ty, (x << s) | (x >> (ty.size() as u32 * 8 - s)))
}

#[inline]
pub(super) fn mask(ty: Ty, v: u64) -> u64 {
    match ty {
        Ty::I8 => v & 0xff,
        Ty::I16 => v & 0xffff,
        Ty::I32 | Ty::F32 => v & 0xffff_ffff,
        _ => v,
    }
}

#[inline]
pub(super) fn sext(ty: Ty, v: u64) -> i64 {
    match ty {
        Ty::I8 => v as u8 as i8 as i64,
        Ty::I16 => v as u16 as i16 as i64,
        Ty::I32 => v as u32 as i32 as i64,
        _ => v as i64,
    }
}

/// How far to shift a value of type `ty` left to put its top bit in bit 63: comparing the
/// shifted values compares the low bits only, signed or not, so `cmp` needs no masking.
#[inline]
pub(crate) fn shift_of(ty: Ty) -> u8 {
    match ty {
        Ty::I8 => 56,
        Ty::I16 => 48,
        Ty::I32 | Ty::F32 => 32,
        _ => 0,
    }
}

#[inline]
pub(super) fn cmp(op: CmpOp, ty: Ty, x: u64, y: u64) -> bool {
    cmp_shifted(op, shift_of(ty), x, y)
}

/// Compare the low `64 - shift` bits of `x` and `y`. A `shift` of 32 on a float comparison
/// means `f32`.
#[inline(always)]
#[expect(
    clippy::float_cmp,
    reason = "the interpreter implements Jai's `==` and `!=` on floats, which are exact IEEE comparisons"
)]
pub(crate) fn cmp_shifted(op: CmpOp, shift: u8, x: u64, y: u64) -> bool {
    let (ux, uy) = (x << shift, y << shift);
    let (sx, sy) = (ux as i64, uy as i64);
    let float = |x: u64| {
        if shift == 32 {
            f32::from_bits(x as u32) as f64
        } else {
            f64::from_bits(x)
        }
    };
    match op {
        CmpOp::Eq => ux == uy,
        CmpOp::Ne => ux != uy,
        CmpOp::SLt => sx < sy,
        CmpOp::SLe => sx <= sy,
        CmpOp::SGt => sx > sy,
        CmpOp::SGe => sx >= sy,
        CmpOp::ULt => ux < uy,
        CmpOp::ULe => ux <= uy,
        CmpOp::UGt => ux > uy,
        CmpOp::UGe => ux >= uy,
        CmpOp::FEq => float(x) == float(y),
        CmpOp::FNe => float(x) != float(y),
        CmpOp::FLt => float(x) < float(y),
        CmpOp::FLe => float(x) <= float(y),
        CmpOp::FGt => float(x) > float(y),
        CmpOp::FGe => float(x) >= float(y),
    }
}

pub(super) fn conv(op: ConvOp, from: Ty, to: Ty, v: u64) -> u64 {
    let f = |v: u64| {
        if from == Ty::F32 {
            f32::from_bits(v as u32) as f64
        } else {
            f64::from_bits(v)
        }
    };
    let out_f = |x: f64| {
        if to == Ty::F32 {
            (x as f32).to_bits() as u64
        } else {
            x.to_bits()
        }
    };
    match op {
        ConvOp::Trunc => mask(to, v),
        ConvOp::ZExt => mask(from, v),
        ConvOp::SExt => mask(to, sext(from, v) as u64),
        ConvOp::FToS => mask(
            to,
            match to {
                Ty::I8 => f(v) as i8 as u64,
                Ty::I16 => f(v) as i16 as u64,
                Ty::I32 => f(v) as i32 as u64,
                _ => f(v) as i64 as u64,
            },
        ),
        ConvOp::FToU => mask(
            to,
            match to {
                Ty::I8 => f(v) as u8 as u64,
                Ty::I16 => f(v) as u16 as u64,
                Ty::I32 => f(v) as u32 as u64,
                _ => f(v) as u64,
            },
        ),
        // Straight to the target width: going through f64 first rounds twice, which is wrong
        // for 64-bit integers whose bits below f64's precision decide an f32 tie.
        ConvOp::SToF if to == Ty::F32 => (sext(from, v) as f32).to_bits() as u64,
        ConvOp::UToF if to == Ty::F32 => (mask(from, v) as f32).to_bits() as u64,
        ConvOp::SToF => out_f(sext(from, v) as f64),
        ConvOp::UToF => out_f(mask(from, v) as f64),
        ConvOp::FExt | ConvOp::FTrunc => out_f(f(v)),
        ConvOp::Bitcast => mask(to, v),
    }
}

#[cfg(test)]
mod tests;
