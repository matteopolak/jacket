//! IR helpers for building the values an instruction computes.
use crate::ir::{BinOp, CmpOp, ConvOp, Intrinsic, Ty, UnOp, Val};
use crate::sema::lower::FnCtx;

pub(super) fn bits(ty: Ty) -> u64 {
    ty.size() * 8
}

pub(super) fn konst(f: &mut FnCtx, ty: Ty, v: u64) -> Val {
    f.b.iconst(ty, v)
}

pub(super) fn bin(f: &mut FnCtx, op: BinOp, ty: Ty, a: Val, b: Val) -> Val {
    f.b.bin(op, ty, a, b)
}

pub(super) fn cmp(f: &mut FnCtx, op: CmpOp, ty: Ty, a: Val, b: Val) -> Val {
    f.b.cmp(op, ty, a, b)
}

pub(super) fn resize(f: &mut FnCtx, v: Val, from: Ty, to: Ty, signed: bool) -> Val {
    if from == to {
        v
    } else if from.size() > to.size() {
        f.b.conv(ConvOp::Trunc, from, to, v)
    } else if signed {
        f.b.conv(ConvOp::SExt, from, to, v)
    } else {
        f.b.conv(ConvOp::ZExt, from, to, v)
    }
}

pub(super) fn is_zero(f: &mut FnCtx, ty: Ty, v: Val) -> Val {
    let z = konst(f, ty, 0);
    cmp(f, CmpOp::Eq, ty, v, z)
}

pub(super) fn is_neg(f: &mut FnCtx, ty: Ty, v: Val) -> Val {
    let z = konst(f, ty, 0);
    cmp(f, CmpOp::SLt, ty, v, z)
}

pub(super) fn flag_not(f: &mut FnCtx, v: Val) -> Val {
    let one = konst(f, Ty::I8, 1);
    bin(f, BinOp::Xor, Ty::I8, v, one)
}

pub(super) fn flag_or(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Or, Ty::I8, a, b)
}

pub(super) fn flag_xor(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Xor, Ty::I8, a, b)
}

/// `cond ? a : b` for integers of class `ty` without branching.
pub(super) fn select(f: &mut FnCtx, ty: Ty, cond: Val, a: Val, b: Val) -> Val {
    let wide = resize(f, cond, Ty::I8, ty, false);
    let zero = konst(f, ty, 0);
    let mask = bin(f, BinOp::Sub, ty, zero, wide);
    let inv = f.b.un(UnOp::Not, ty, mask);
    let x = bin(f, BinOp::And, ty, a, mask);
    let y = bin(f, BinOp::And, ty, b, inv);
    bin(f, BinOp::Or, ty, x, y)
}

pub(super) fn sign_flag_of(f: &mut FnCtx, ty: Ty, a: Val, b: Val, res: Val, sub: bool) -> Val {
    // Add: ((a ^ res) & (b ^ res)) < 0.  Sub: ((a ^ b) & (a ^ res)) < 0.
    let (x, y) = if sub {
        (bin(f, BinOp::Xor, ty, a, b), bin(f, BinOp::Xor, ty, a, res))
    } else {
        (
            bin(f, BinOp::Xor, ty, a, res),
            bin(f, BinOp::Xor, ty, b, res),
        )
    };
    let both = bin(f, BinOp::And, ty, x, y);
    is_neg(f, ty, both)
}

pub(super) fn bit_intrinsic(f: &mut FnCtx, op: Intrinsic, ty: Ty, x: Val) -> Val {
    let width = konst(f, Ty::I64, bits(ty));
    f.b.intrinsic(op, vec![x, width], &[ty])[0]
}

pub(super) fn ptr_of(f: &mut FnCtx, addr: Val) -> Val {
    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, addr)
}

/// 64x64 -> 128 multiply: (high, low). Narrower sizes multiply in 64 bits.
pub(super) fn wide_mul(f: &mut FnCtx, sz: Ty, a: Val, b: Val, signed: bool) -> (Val, Val) {
    if sz != Ty::I64 {
        let a = resize(f, a, sz, Ty::I64, signed);
        let b = resize(f, b, sz, Ty::I64, signed);
        let p = bin(f, BinOp::Mul, Ty::I64, a, b);
        let shift = konst(f, Ty::I64, bits(sz));
        let hi = bin(f, BinOp::LShr, Ty::I64, p, shift);
        return (
            resize(f, hi, Ty::I64, sz, false),
            resize(f, p, Ty::I64, sz, false),
        );
    }
    let m32 = konst(f, Ty::I64, 0xffff_ffff);
    let s32 = konst(f, Ty::I64, 32);
    let a0 = bin(f, BinOp::And, Ty::I64, a, m32);
    let a1 = bin(f, BinOp::LShr, Ty::I64, a, s32);
    let b0 = bin(f, BinOp::And, Ty::I64, b, m32);
    let b1 = bin(f, BinOp::LShr, Ty::I64, b, s32);
    let p00 = bin(f, BinOp::Mul, Ty::I64, a0, b0);
    let p01 = bin(f, BinOp::Mul, Ty::I64, a0, b1);
    let p10 = bin(f, BinOp::Mul, Ty::I64, a1, b0);
    let p11 = bin(f, BinOp::Mul, Ty::I64, a1, b1);
    let t0 = bin(f, BinOp::LShr, Ty::I64, p00, s32);
    let t1 = bin(f, BinOp::And, Ty::I64, p01, m32);
    let t2 = bin(f, BinOp::And, Ty::I64, p10, m32);
    let mid = bin(f, BinOp::Add, Ty::I64, t0, t1);
    let mid = bin(f, BinOp::Add, Ty::I64, mid, t2);
    let carry = bin(f, BinOp::LShr, Ty::I64, mid, s32);
    let u01 = bin(f, BinOp::LShr, Ty::I64, p01, s32);
    let u10 = bin(f, BinOp::LShr, Ty::I64, p10, s32);
    let mut hi = bin(f, BinOp::Add, Ty::I64, p11, u01);
    hi = bin(f, BinOp::Add, Ty::I64, hi, u10);
    hi = bin(f, BinOp::Add, Ty::I64, hi, carry);
    let lo = bin(f, BinOp::Mul, Ty::I64, a, b);
    if signed {
        // High word of the signed product: subtract the operand when the other is negative.
        let s63 = konst(f, Ty::I64, 63);
        let ma = bin(f, BinOp::AShr, Ty::I64, a, s63);
        let mb = bin(f, BinOp::AShr, Ty::I64, b, s63);
        let fix_a = bin(f, BinOp::And, Ty::I64, ma, b);
        let fix_b = bin(f, BinOp::And, Ty::I64, mb, a);
        hi = bin(f, BinOp::Sub, Ty::I64, hi, fix_a);
        hi = bin(f, BinOp::Sub, Ty::I64, hi, fix_b);
    }
    (hi, lo)
}
