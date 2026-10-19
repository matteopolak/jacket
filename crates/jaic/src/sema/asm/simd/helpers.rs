//! Lane helpers: saturation, rounding, AES tables and write masks.
use crate::sema::asm::vec::{float_to_s32, fselect};
use crate::sema::asm::{
    BinOp, CmpOp, ConvOp, FnCtx, Intrinsic, Ty, UnOp, Val, bin, bits, cmp, konst, resize, select,
};

/// Saturate a value of the wide type `wt` to `ty` lanes (signed or unsigned range).
pub(super) fn saturate(f: &mut FnCtx, wt: Ty, v: Val, ty: Ty, signed: bool) -> Val {
    let b = bits(ty);
    let (lo, hi) = if signed {
        ((1u64 << (b - 1)).wrapping_neg(), (1u64 << (b - 1)) - 1)
    } else {
        (0, (1u64 << b) - 1)
    };
    let lo_v = konst(f, wt, lo);
    let hi_v = konst(f, wt, hi);
    let below = cmp(f, CmpOp::SLt, wt, v, lo_v);
    let above = cmp(f, CmpOp::SGt, wt, v, hi_v);
    let v = select(f, wt, below, lo_v, v);
    let v = select(f, wt, above, hi_v, v);
    resize(f, v, wt, ty, false)
}

/// Round an `F64` to an integral value: n(earest even), d(own), u(p) or z (toward zero).
pub(super) fn round_f64(f: &mut FnCtx, x: Val, mode: char) -> Val {
    match mode {
        'd' => f.b.intrinsic(Intrinsic::Floor, vec![x], &[Ty::F64])[0],
        'u' => f.b.intrinsic(Intrinsic::Ceil, vec![x], &[Ty::F64])[0],
        'z' => f.b.intrinsic(Intrinsic::Trunc, vec![x], &[Ty::F64])[0],
        _ => {
            // Below 2^52, adding and removing 2^52 rounds to nearest even; above, x is integral.
            let ax = f.b.intrinsic(Intrinsic::Fabs, vec![x], &[Ty::F64])[0];
            let big = f.b.fconst(Ty::F64, 4_503_599_627_370_496.0);
            let up = bin(f, BinOp::FAdd, Ty::F64, ax, big);
            let r = bin(f, BinOp::FSub, Ty::F64, up, big);
            let zero = f.b.fconst(Ty::F64, 0.0);
            let neg = cmp(f, CmpOp::FLt, Ty::F64, x, zero);
            let nr = f.b.un(UnOp::FNeg, Ty::F64, r);
            let r = fselect(f, Ty::F64, neg, nr, r);
            let small = cmp(f, CmpOp::FLt, Ty::F64, ax, big);
            fselect(f, Ty::F64, small, r, x)
        }
    }
}

pub(super) fn to_f64(f: &mut FnCtx, ty: Ty, x: Val) -> Val {
    if ty == Ty::F32 {
        f.b.conv(ConvOp::FExt, Ty::F32, Ty::F64, x)
    } else {
        x
    }
}

pub(super) fn from_f64(f: &mut FnCtx, ty: Ty, x: Val) -> Val {
    if ty == Ty::F32 {
        f.b.conv(ConvOp::FTrunc, Ty::F64, Ty::F32, x)
    } else {
        x
    }
}

/// Float to integer the way the SSE/AVX conversions do: out-of-range and NaN give the
/// "integer indefinite" value (the minimum for signed, all ones for unsigned).
pub(super) fn float_to_int(
    f: &mut FnCtx,
    from: Ty,
    x: Val,
    to: Ty,
    mode: char,
    unsigned: bool,
) -> Val {
    if from == Ty::F32 && to == Ty::I32 && !unsigned {
        return float_to_s32(f, x, mode);
    }
    let x = to_f64(f, from, x);
    let r = round_f64(f, x, mode);
    let w = bits(to) as i32;
    let (lo, hi) = if unsigned {
        (0.0, 2f64.powi(w))
    } else {
        (-(2f64.powi(w - 1)), 2f64.powi(w - 1))
    };
    let lo_v = f.b.fconst(Ty::F64, lo);
    let hi_v = f.b.fconst(Ty::F64, hi);
    let ge = cmp(f, CmpOp::FGe, Ty::F64, r, lo_v);
    let lt = cmp(f, CmpOp::FLt, Ty::F64, r, hi_v);
    let ok = bin(f, BinOp::And, Ty::I8, ge, lt);
    let zero = f.b.fconst(Ty::F64, 0.0);
    let safe = fselect(f, Ty::F64, ok, r, zero);
    let v = if unsigned {
        f.b.conv(ConvOp::FToU, Ty::F64, Ty::I64, safe)
    } else {
        f.b.conv(ConvOp::FToS, Ty::F64, Ty::I64, safe)
    };
    let v = resize(f, v, Ty::I64, to, false);
    let bad = if unsigned {
        konst(f, to, u64::MAX)
    } else {
        konst(f, to, 1u64 << (w - 1))
    };
    select(f, to, ok, v, bad)
}

/// The AES S-box, its inverse and the GF(2^8) multiplicative inverse (0 for 0, used by
/// `gf2p8affineinvqb`), generated from the field inverse and the affine map.
pub(super) fn aes_tables() -> Vec<u8> {
    let mul = |mut a: u8, mut b: u8| {
        let mut p = 0u8;
        while b != 0 {
            if b & 1 != 0 {
                p ^= a;
            }
            let hi = a & 0x80;
            a <<= 1;
            if hi != 0 {
                a ^= 0x1b;
            }
            b >>= 1;
        }
        p
    };
    let mut sbox = [0u8; 256];
    let mut inv = [0u8; 256];
    let mut field_inv = [0u8; 256];
    for x in 0..=255u8 {
        let mut y = 0u8;
        if x != 0 {
            // x^254 is the multiplicative inverse in GF(2^8).
            let mut acc = 1u8;
            let mut base = x;
            let mut e = 254u32;
            while e > 0 {
                if e & 1 != 0 {
                    acc = mul(acc, base);
                }
                base = mul(base, base);
                e >>= 1;
            }
            y = acc;
        }
        field_inv[x as usize] = y;
        let s =
            y ^ y.rotate_left(1) ^ y.rotate_left(2) ^ y.rotate_left(3) ^ y.rotate_left(4) ^ 0x63;
        sbox[x as usize] = s;
        inv[s as usize] = x;
    }
    let mut out = sbox.to_vec();
    out.extend_from_slice(&inv);
    out.extend_from_slice(&field_inv);
    out
}

/// Multiply an `I8` by 2 in GF(2^8).
pub(super) fn xtime(f: &mut FnCtx, x: Val) -> Val {
    let one = konst(f, Ty::I8, 1);
    let seven = konst(f, Ty::I8, 7);
    let t = bin(f, BinOp::Shl, Ty::I8, x, one);
    let hi = bin(f, BinOp::AShr, Ty::I8, x, seven);
    let poly = konst(f, Ty::I8, 0x1b);
    let m = bin(f, BinOp::And, Ty::I8, hi, poly);
    bin(f, BinOp::Xor, Ty::I8, t, m)
}

pub(super) fn xor8(f: &mut FnCtx, a: Val, b: Val) -> Val {
    bin(f, BinOp::Xor, Ty::I8, a, b)
}

/// AES `MixColumns` (or `InvMixColumns`) of the 16 bytes of `s`.
pub(super) fn mix_columns(f: &mut FnCtx, s: &[Val; 16], inverse: bool) -> [Val; 16] {
    let mut out = *s;
    for c in 0..4 {
        let a: Vec<Val> = (0..4).map(|r| s[4 * c + r]).collect();
        let x2: Vec<Val> = a.iter().map(|&v| xtime(f, v)).collect();
        if inverse {
            let x4: Vec<Val> = x2.iter().map(|&v| xtime(f, v)).collect();
            let x8: Vec<Val> = x4.iter().map(|&v| xtime(f, v)).collect();
            // 14 = 8^4^2, 11 = 8^2^1, 13 = 8^4^1, 9 = 8^1
            let m14: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x4[k]);
                    xor8(f, t, x2[k])
                })
                .collect();
            let m11: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x2[k]);
                    xor8(f, t, a[k])
                })
                .collect();
            let m13: Vec<Val> = (0..4)
                .map(|k| {
                    let t = xor8(f, x8[k], x4[k]);
                    xor8(f, t, a[k])
                })
                .collect();
            let m9: Vec<Val> = (0..4).map(|k| xor8(f, x8[k], a[k])).collect();
            for r in 0..4 {
                let t = xor8(f, m14[r], m11[(r + 1) % 4]);
                let t = xor8(f, t, m13[(r + 2) % 4]);
                out[4 * c + r] = xor8(f, t, m9[(r + 3) % 4]);
            }
        } else {
            for r in 0..4 {
                // 2*a[r] ^ 3*a[r+1] ^ a[r+2] ^ a[r+3]
                let n1 = a[(r + 1) % 4];
                let t = xor8(f, x2[r], x2[(r + 1) % 4]);
                let t = xor8(f, t, n1);
                let t = xor8(f, t, a[(r + 2) % 4]);
                out[4 * c + r] = xor8(f, t, a[(r + 3) % 4]);
            }
        }
    }
    out
}

/// Load `ty` at `base + index * ty.size()` for a runtime lane index (`I64`).
pub(super) fn load_dyn(f: &mut FnCtx, base: Val, index: Val, ty: Ty) -> Val {
    let size = konst(f, Ty::I64, ty.size());
    let off = bin(f, BinOp::Mul, Ty::I64, index, size);
    let p = f.b.ptr_add(base, off);
    f.b.load(ty, p)
}

pub(super) fn store_dyn(f: &mut FnCtx, base: Val, index: Val, ty: Ty, v: Val) {
    let size = konst(f, Ty::I64, ty.size());
    let off = bin(f, BinOp::Mul, Ty::I64, index, size);
    let p = f.b.ptr_add(base, off);
    f.b.store(ty, p, v);
}

/// Bit `i` of a 64-bit mask value as an `I8` flag.
pub(super) fn mask_bit(f: &mut FnCtx, mask: Val, i: u64) -> Val {
    let at = konst(f, Ty::I64, i);
    let b = bin(f, BinOp::LShr, Ty::I64, mask, at);
    let one = konst(f, Ty::I64, 1);
    let b = bin(f, BinOp::And, Ty::I64, b, one);
    resize(f, b, Ty::I64, Ty::I8, false)
}

/// Run `body` only when `cond` is set (for per-lane memory accesses that must not fault).
pub(super) fn when(f: &mut FnCtx, cond: Val, body: impl FnOnce(&mut FnCtx)) {
    let then = f.b.new_block();
    let join = f.b.new_block();
    f.b.branch(cond, then, join);
    f.b.switch_to(then);
    body(f);
    f.b.jump(join);
    f.b.switch_to(join);
}

/// A `{k}` write mask: its bits, zeroing (`&*`) and the register (cleared by scatters).
#[derive(Clone, Copy)]
pub(in crate::sema::asm) struct WriteMask {
    pub bits: Val,
    pub zeroing: bool,
    pub reg: Option<Val>,
}
