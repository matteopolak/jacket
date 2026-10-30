//! Per-lane arithmetic and multiplies.
use super::dispatch::SimdLocals;
use super::helpers::{from_f64, round_f64, saturate, to_f64};
use super::{B2, FmaKind, U1, VOp};
use crate::diagnostics::Code;
use crate::sema::asm::vec::{load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, FnCtx, Intrinsic, Result, Ty, UnOp, bin, bit_intrinsic, bits,
    cmp, err, konst, resize, select, wide_mul,
};

impl Compiler {
    pub(super) fn simd_lanes(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            width,
            ops,
            op,
            ..
        } = *c;
        match op {
            VOp::Bin(b2, ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let r = Self::simd_bin(f, b2, ty, a, b);
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::RotImm(left, ty) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let c = konst(f, ty, k % bits(ty));
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let r = bin(
                        f,
                        if left {
                            BinOp::Rotl
                        } else {
                            BinOp::Rotr
                        },
                        ty,
                        a,
                        c,
                    );
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Unary(u, ty) => self.simd_unary(f, cx, c, u, ty)?,
            VOp::Round(ty, scalar) => self.simd_round(f, cx, c, ty, scalar)?,
            VOp::Fma(kind, order, ty, scalar) => {
                self.simd_fma(f, cx, c, (kind, order), ty, scalar)?;
            }
            VOp::Horizontal(sub, sat, ty) => self.simd_horizontal(f, cx, c, sub, sat, ty)?,
            VOp::AddSub(ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let x = load_lane(f, s[0], i, ty);
                    let y = load_lane(f, s[1], i, ty);
                    let r = bin(
                        f,
                        if i % 2 == 0 {
                            BinOp::FSub
                        } else {
                            BinOp::FAdd
                        },
                        ty,
                        x,
                        y,
                    );
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }

    fn simd_unary(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        u: U1,
        ty: Ty,
    ) -> Result<()> {
        let SimdLocals {
            span,
            name,
            dst,
            tmp,
            width,
            ops,
            ..
        } = *c;
        let (s, _) = self.simd_args(f, ops, 1, false, span)?;
        let (lane, fty) = match ty {
            Ty::F32 => (Ty::F32, true),
            Ty::F64 => (Ty::F64, true),
            t => (t, false),
        };
        let n = width / lane.size();
        for i in 0..n {
            let a = load_lane(f, s[0], i, lane);
            let r = match u {
                U1::Popcnt => bit_intrinsic(f, Intrinsic::Popcount, lane, a),
                U1::Lzcnt => bit_intrinsic(f, Intrinsic::Ctlz, lane, a),
                U1::Rcp | U1::Rsqrt if fty => {
                    let x = to_f64(f, lane, a);
                    let x = if u == U1::Rsqrt {
                        f.b.intrinsic(Intrinsic::Sqrt, vec![x], &[Ty::F64])[0]
                    } else {
                        x
                    };
                    let one = f.b.fconst(Ty::F64, 1.0);
                    let r = bin(f, BinOp::FDiv, Ty::F64, one, x);
                    from_f64(f, lane, r)
                }
                U1::Conflict => {
                    let mut m = konst(f, lane, 0);
                    for j in 0..i {
                        let b = load_lane(f, s[0], j, lane);
                        let eq = cmp(f, CmpOp::Eq, lane, a, b);
                        let eq = resize(f, eq, Ty::I8, lane, false);
                        let at = konst(f, lane, j);
                        let eq = bin(f, BinOp::Shl, lane, eq, at);
                        m = bin(f, BinOp::Or, lane, m, eq);
                    }
                    m
                }
                _ => {
                    return err(
                        Code::AsmVectorOperand,
                        span,
                        format!("`{name}` needs float lanes"),
                    );
                }
            };
            store_lane(f, tmp, i, lane, r);
        }
        self.simd_out(f, cx, dst, tmp, width, span)?;

        Ok(())
    }

    fn simd_round(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        ty: Ty,
        scalar: bool,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            width,
            ops,
            inst,
            ..
        } = *c;
        let (s, k) = self.simd_args(
            f,
            ops,
            if scalar {
                2
            } else {
                1
            },
            true,
            span,
        )?;
        let mode = match (inst.evex.round, k & 4 != 0) {
            (Some(Some(m)), _) => m,
            (_, true) => 'n',
            _ => ['n', 'd', 'u', 'z'][(k & 3) as usize],
        };
        // rndscale: round to a multiple of 2^-M (M = imm[7:4]).
        let scale = (1u64 << (k >> 4)) as f64;
        let lanes = if scalar {
            1
        } else {
            width / ty.size()
        };
        let src = *s.last().unwrap();
        if scalar {
            f.b.copy(tmp, s[0], 16);
        }
        for i in 0..lanes {
            let x = load_lane(f, src, i, ty);
            let x = to_f64(f, ty, x);
            let sc = f.b.fconst(Ty::F64, scale);
            let y = bin(f, BinOp::FMul, Ty::F64, x, sc);
            let y = round_f64(f, y, mode);
            let y = bin(f, BinOp::FDiv, Ty::F64, y, sc);
            let y = from_f64(f, ty, y);
            store_lane(f, tmp, i, ty, y);
        }
        self.simd_out(
            f,
            cx,
            dst,
            tmp,
            if scalar {
                16
            } else {
                width
            },
            span,
        )?;

        Ok(())
    }

    fn simd_fma(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        (kind, order): (FmaKind, u16),
        ty: Ty,
        scalar: bool,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            width,
            ops,
            ..
        } = *c;
        let (s, _) = self.simd_args(f, ops, 3, false, span)?;
        let (d, s2, s3) = (s[0], s[1], s[2]);
        let (pa, pb, pc) = match order {
            132 => (d, s3, s2),
            213 => (s2, d, s3),
            _ => (s2, s3, d),
        };
        let lanes = if scalar {
            1
        } else {
            width / ty.size()
        };
        if scalar {
            f.b.copy(tmp, d, 16);
        }
        for i in 0..lanes {
            let a = load_lane(f, pa, i, ty);
            let b = load_lane(f, pb, i, ty);
            let c = load_lane(f, pc, i, ty);
            let (a, b, c) = (to_f64(f, ty, a), to_f64(f, ty, b), to_f64(f, ty, c));
            let even = i % 2 == 0;
            let (neg_ab, neg_c) = match kind {
                FmaKind::Madd => (false, false),
                FmaKind::Msub => (false, true),
                FmaKind::Nmadd => (true, false),
                FmaKind::Nmsub => (true, true),
                FmaKind::MaddSub => (false, even),
                FmaKind::MsubAdd => (false, !even),
            };
            let a = if neg_ab {
                f.b.un(UnOp::FNeg, Ty::F64, a)
            } else {
                a
            };
            let c = if neg_c {
                f.b.un(UnOp::FNeg, Ty::F64, c)
            } else {
                c
            };
            let r = f.b.intrinsic(Intrinsic::Fma, vec![a, b, c], &[Ty::F64])[0];
            let r = from_f64(f, ty, r);
            store_lane(f, tmp, i, ty, r);
        }
        self.simd_out(
            f,
            cx,
            dst,
            tmp,
            if scalar {
                16
            } else {
                width
            },
            span,
        )?;

        Ok(())
    }

    fn simd_horizontal(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        sub: bool,
        sat: bool,
        ty: Ty,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            block,
            width,
            ops,
            ..
        } = *c;
        let (s, _) = self.simd_args(f, ops, 2, false, span)?;
        let per = block / ty.size();
        for blk in 0..width / block {
            let base = blk * per;
            for j in 0..per {
                let src = if j < per / 2 {
                    s[0]
                } else {
                    s[1]
                };
                let k = j % (per / 2);
                let x = load_lane(f, src, base + 2 * k, ty);
                let y = load_lane(f, src, base + 2 * k + 1, ty);
                let r = if ty.is_float() {
                    bin(
                        f,
                        if sub {
                            BinOp::FSub
                        } else {
                            BinOp::FAdd
                        },
                        ty,
                        x,
                        y,
                    )
                } else if sat {
                    Self::simd_bin(
                        f,
                        if sub {
                            B2::SubSatS
                        } else {
                            B2::AddSatS
                        },
                        ty,
                        x,
                        y,
                    )
                } else {
                    bin(
                        f,
                        if sub {
                            BinOp::Sub
                        } else {
                            BinOp::Add
                        },
                        ty,
                        x,
                        y,
                    )
                };
                store_lane(f, tmp, base + j, ty, r);
            }
        }
        self.simd_out(f, cx, dst, tmp, width, span)?;

        Ok(())
    }

    pub(super) fn simd_multiply(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            width,
            ops,
            op,
            ..
        } = *c;
        match op {
            VOp::Pmaddwd | VOp::Pmaddubsw => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let (src, out) = if op == VOp::Pmaddwd {
                    (Ty::I16, Ty::I32)
                } else {
                    (Ty::I8, Ty::I16)
                };
                for i in 0..width / out.size() {
                    let mut sum = konst(f, Ty::I32, 0);
                    for k in 0..2 {
                        let a = load_lane(f, s[0], 2 * i + k, src);
                        let b = load_lane(f, s[1], 2 * i + k, src);
                        // pmaddubsw: unsigned bytes of the first source times signed bytes.
                        let a = resize(f, a, src, Ty::I32, op == VOp::Pmaddwd);
                        let b = resize(f, b, src, Ty::I32, true);
                        let p = bin(f, BinOp::Mul, Ty::I32, a, b);
                        sum = bin(f, BinOp::Add, Ty::I32, sum, p);
                    }
                    let r = if out == Ty::I16 {
                        saturate(f, Ty::I32, sum, Ty::I16, true)
                    } else {
                        sum
                    };
                    store_lane(f, tmp, i, out, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Psadbw => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / 8 {
                    let mut sum = konst(f, Ty::I64, 0);
                    for k in 0..8 {
                        let a = load_lane(f, s[0], 8 * i + k, Ty::I8);
                        let b = load_lane(f, s[1], 8 * i + k, Ty::I8);
                        let a = resize(f, a, Ty::I8, Ty::I64, false);
                        let b = resize(f, b, Ty::I8, Ty::I64, false);
                        let d = bin(f, BinOp::Sub, Ty::I64, a, b);
                        let nd = bin(f, BinOp::Sub, Ty::I64, b, a);
                        let lt = cmp(f, CmpOp::ULt, Ty::I64, a, b);
                        let d = select(f, Ty::I64, lt, nd, d);
                        sum = bin(f, BinOp::Add, Ty::I64, sum, d);
                    }
                    store_lane(f, tmp, i, Ty::I64, sum);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Mpsadbw => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                // Each 128-bit block takes 3 bits of the immediate: the first source starts
                // at byte 4 * bit 2, the second at byte 4 * bits 1:0.
                for blk in 0..width / 16 {
                    let ctl = k >> (3 * blk);
                    let (aoff, boff) = (((ctl >> 2) & 1) * 4, (ctl & 3) * 4);
                    for i in 0..8 {
                        let mut sum = konst(f, Ty::I16, 0);
                        for m in 0..4 {
                            let a = load_lane(f, s[0], blk * 16 + aoff + i + m, Ty::I8);
                            let b = load_lane(f, s[1], blk * 16 + boff + m, Ty::I8);
                            let a = resize(f, a, Ty::I8, Ty::I16, false);
                            let b = resize(f, b, Ty::I8, Ty::I16, false);
                            let d = bin(f, BinOp::Sub, Ty::I16, a, b);
                            let nd = bin(f, BinOp::Sub, Ty::I16, b, a);
                            let lt = cmp(f, CmpOp::ULt, Ty::I16, a, b);
                            let d = select(f, Ty::I16, lt, nd, d);
                            sum = bin(f, BinOp::Add, Ty::I16, sum, d);
                        }
                        store_lane(f, tmp, blk * 8 + i, Ty::I16, sum);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Madd52(high) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let mask = konst(f, Ty::I64, (1 << 52) - 1);
                for i in 0..width / 8 {
                    let acc = load_lane(f, s[0], i, Ty::I64);
                    let a = load_lane(f, s[1], i, Ty::I64);
                    let b = load_lane(f, s[2], i, Ty::I64);
                    let a = bin(f, BinOp::And, Ty::I64, a, mask);
                    let b = bin(f, BinOp::And, Ty::I64, b, mask);
                    let (hi, lo) = wide_mul(f, Ty::I64, a, b, false);
                    let part = if high {
                        let up = konst(f, Ty::I64, 12);
                        let down = konst(f, Ty::I64, 52);
                        let h = bin(f, BinOp::Shl, Ty::I64, hi, up);
                        let l = bin(f, BinOp::LShr, Ty::I64, lo, down);
                        bin(f, BinOp::Or, Ty::I64, h, l)
                    } else {
                        lo
                    };
                    let part = bin(f, BinOp::And, Ty::I64, part, mask);
                    let r = bin(f, BinOp::Add, Ty::I64, acc, part);
                    store_lane(f, tmp, i, Ty::I64, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::MulEven(signed) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / 8 {
                    let a = load_lane(f, s[0], 2 * i, Ty::I32);
                    let b = load_lane(f, s[1], 2 * i, Ty::I32);
                    let a = resize(f, a, Ty::I32, Ty::I64, signed);
                    let b = resize(f, b, Ty::I32, Ty::I64, signed);
                    let r = bin(f, BinOp::Mul, Ty::I64, a, b);
                    store_lane(f, tmp, i, Ty::I64, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::DotAcc(src) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let per = 4 / src.size();
                for i in 0..width / 4 {
                    let mut sum = load_lane(f, s[0], i, Ty::I32);
                    for k in 0..per {
                        let a = load_lane(f, s[1], per * i + k, src);
                        let b = load_lane(f, s[2], per * i + k, src);
                        // pdpbusd: unsigned bytes times signed bytes; pdpwssd: signed words.
                        let a = resize(f, a, src, Ty::I32, src == Ty::I16);
                        let b = resize(f, b, src, Ty::I32, true);
                        let p = bin(f, BinOp::Mul, Ty::I32, a, b);
                        sum = bin(f, BinOp::Add, Ty::I32, sum, p);
                    }
                    store_lane(f, tmp, i, Ty::I32, sum);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Phminposuw => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let mut best = load_lane(f, s[0], 0, Ty::I16);
                let mut at = konst(f, Ty::I16, 0);
                for i in 1..8 {
                    let v = load_lane(f, s[0], i, Ty::I16);
                    let lt = cmp(f, CmpOp::ULt, Ty::I16, v, best);
                    best = select(f, Ty::I16, lt, v, best);
                    let iv = konst(f, Ty::I16, i);
                    at = select(f, Ty::I16, lt, iv, at);
                }
                store_lane(f, tmp, 0, Ty::I16, best);
                store_lane(f, tmp, 1, Ty::I16, at);
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }
}
