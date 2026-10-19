//! Comparisons and bit tests.
use super::VOp;
use super::dispatch::SimdLocals;
use super::helpers::mask_bit;
use crate::sema::asm::vec::{VOpd, all_ones_if, load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, Flags, FnCtx, Result, Ty, UnOp, bin, cmp, err, flag_not,
    flag_or, is_neg, is_zero, konst, resize,
};

impl Compiler {
    pub(super) fn simd_compare(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
    ) -> Result<()> {
        let SimdLocals {
            span,
            name,
            dst,
            tmp,
            width,
            wm,
            ops,
            op,
            ..
        } = *c;
        match op {
            VOp::FCmp(ty, scalar) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let lanes = if scalar {
                    1
                } else {
                    width / ty.size()
                };
                let mut flags = Vec::new();
                if scalar {
                    f.b.copy(tmp, s[0], 16);
                }
                for i in 0..lanes {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let c = Self::float_predicate(f, ty, a, b, k & 0xf);
                    if matches!(dst, VOpd::Mask(_)) {
                        flags.push(c);
                    } else {
                        let it = Ty::int(ty.size());
                        let v = all_ones_if(f, it, c);
                        store_lane(f, tmp, i, it, v);
                    }
                }
                if matches!(dst, VOpd::Mask(_)) {
                    self.simd_mask_out(f, dst, flags, wm, span)?;
                } else {
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
                }
            }
            VOp::IntCmp(ty, unsigned, fixed) => {
                let (s, k) = match fixed {
                    Some(p) => (self.simd_args(f, ops, 2, false, span)?.0, p as u64),
                    None => self.simd_args(f, ops, 2, true, span)?,
                };
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let (lt, le) = if unsigned {
                        (CmpOp::ULt, CmpOp::ULe)
                    } else {
                        (CmpOp::SLt, CmpOp::SLe)
                    };
                    let c = match k & 7 {
                        0 => cmp(f, CmpOp::Eq, ty, a, b),
                        1 => cmp(f, lt, ty, a, b),
                        2 => cmp(f, le, ty, a, b),
                        3 => konst(f, Ty::I8, 0),
                        4 => cmp(f, CmpOp::Ne, ty, a, b),
                        5 => {
                            let c = cmp(f, lt, ty, a, b);
                            flag_not(f, c)
                        }
                        6 => {
                            let c = cmp(f, le, ty, a, b);
                            flag_not(f, c)
                        }
                        _ => konst(f, Ty::I8, 1),
                    };
                    flags.push(c);
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            VOp::Comis(ty) => {
                // `comiss a, b`: unordered 111, a < b 001, a == b 100, a > b 000 (ZF, PF, CF).
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let pa = self.vec_ptr(f, ops[0], span)?;
                let pb = self.vec_ptr(f, ops[1], span)?;
                let a = load_lane(f, pa, 0, ty);
                let b = load_lane(f, pb, 0, ty);
                let ua = cmp(f, CmpOp::FNe, ty, a, a);
                let ub = cmp(f, CmpOp::FNe, ty, b, b);
                let uno = flag_or(f, ua, ub);
                let eq = cmp(f, CmpOp::FEq, ty, a, b);
                let lt = cmp(f, CmpOp::FLt, ty, a, b);
                let zero = konst(f, Ty::I8, 0);
                cx.flags = Flags {
                    zf: Some(flag_or(f, uno, eq)),
                    cf: Some(flag_or(f, uno, lt)),
                    // PF = unordered: an even-parity byte (0) sets it.
                    pf: Some(flag_not(f, uno)),
                    sf: Some(zero),
                    of: Some(zero),
                };
            }
            VOp::Pcmpstr(explicit, mask) => {
                self.asm_pcmpstr(f, cx, ops, explicit, mask, span, name)?;
            }
            VOp::Ptest => {
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let pa = self.vec_ptr(f, ops[0], span)?;
                let pb = self.vec_ptr(f, ops[1], span)?;
                let mut and = konst(f, Ty::I64, 0);
                let mut andn = konst(f, Ty::I64, 0);
                for i in 0..width / 8 {
                    let a = load_lane(f, pa, i, Ty::I64);
                    let b = load_lane(f, pb, i, Ty::I64);
                    let x = bin(f, BinOp::And, Ty::I64, a, b);
                    and = bin(f, BinOp::Or, Ty::I64, and, x);
                    let na = f.b.un(UnOp::Not, Ty::I64, a);
                    let y = bin(f, BinOp::And, Ty::I64, na, b);
                    andn = bin(f, BinOp::Or, Ty::I64, andn, y);
                }
                let zero = konst(f, Ty::I8, 0);
                let odd = konst(f, Ty::I8, 1);
                cx.flags = Flags {
                    zf: Some(is_zero(f, Ty::I64, and)),
                    cf: Some(is_zero(f, Ty::I64, andn)),
                    sf: Some(zero),
                    of: Some(zero),
                    pf: Some(odd),
                };
            }
            VOp::Testm(ty, not) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let x = bin(f, BinOp::And, ty, a, b);
                    let z = is_zero(f, ty, x);
                    flags.push(if not {
                        z
                    } else {
                        flag_not(f, z)
                    });
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            VOp::VecToMask(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let mut flags = Vec::new();
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    flags.push(is_neg(f, ty, a));
                }
                self.simd_mask_out(f, dst, flags, wm, span)?;
            }
            VOp::MaskToVec(ty) | VOp::BroadcastMask(ty) => {
                let VOpd::Mask(p) = ops.get(1).copied().unwrap_or(dst) else {
                    return err(span, format!("`{name}` reads a mask register"));
                };
                if ops.len() != 2 {
                    return err(span, format!("`{name}` takes 2 operands"));
                }
                let m = f.b.load(Ty::I64, p);
                for i in 0..width / ty.size() {
                    let v = if op == VOp::MaskToVec(ty) {
                        let b = mask_bit(f, m, i);
                        all_ones_if(f, ty, b)
                    } else {
                        let low = if ty == Ty::I32 {
                            0xffff
                        } else {
                            0xff
                        };
                        let low = konst(f, Ty::I64, low);
                        let v = bin(f, BinOp::And, Ty::I64, m, low);
                        resize(f, v, Ty::I64, ty, false)
                    };
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }
}
