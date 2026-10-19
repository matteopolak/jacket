//! The string-compare instructions.
use crate::sema::asm::vec::{VOpd, load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, Flags, FnCtx, Result, Span, Ty, Val, bin, cmp, err, flag_not,
    is_neg, is_zero, konst, resize, select,
};

impl Compiler {
    /// `pcmpistri`, `pcmpestri`, `pcmpistrm`, `pcmpestrm`: string comparison of two 16-byte
    /// operands. The operands are Jai's explicit form: `ecx` (or `xmm0`), then for the explicit
    /// length forms `eax` and `edx`, the two vectors and the control byte. The control byte is
    /// a constant, so the aggregation is chosen here and only the per-element booleans are
    /// computed at run time.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn asm_pcmpstr(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        ops: &[VOpd],
        explicit: bool,
        mask: bool,
        span: Span,
        name: &str,
    ) -> Result<()> {
        let want = if explicit {
            6
        } else {
            4
        };
        if ops.len() != want {
            return err(span, format!("`{name}` takes {want} operands"));
        }
        let Some(VOpd::Imm(ctl)) = ops.last().copied() else {
            return err(span, format!("`{name}` needs an 8-bit control constant"));
        };
        if !(0..=255).contains(&ctl) {
            return err(span, format!("`{name}` needs an 8-bit control constant"));
        }
        let ctl = ctl as u8;
        let (ty, signed) = match ctl & 3 {
            0 => (Ty::I8, false),
            1 => (Ty::I16, false),
            2 => (Ty::I8, true),
            _ => (Ty::I16, true),
        };
        let n = (16 / ty.size()) as usize;
        let first = if explicit {
            3
        } else {
            1
        };
        let pa = self.vec_ptr(f, ops[first], span)?;
        let pb = self.vec_ptr(f, ops[first + 1], span)?;
        let a: Vec<Val> = (0..n).map(|i| load_lane(f, pa, i as u64, ty)).collect();
        let b: Vec<Val> = (0..n).map(|i| load_lane(f, pb, i as u64, ty)).collect();
        // Which elements are part of the string: before the first zero (implicit lengths) or
        // below the length in `eax` / `edx` (absolute value, at most the element count).
        let valid_of = |this: &mut Self, f: &mut FnCtx, len: Option<VOpd>, v: &[Val]| {
            let mut out = Vec::with_capacity(n);
            if let Some(len) = len {
                let x = this.vec_scalar_read(f, len, Ty::I32, span)?;
                let neg = is_neg(f, Ty::I32, x);
                let zero = konst(f, Ty::I32, 0);
                let negated = bin(f, BinOp::Sub, Ty::I32, zero, x);
                let abs = select(f, Ty::I32, neg, negated, x);
                for i in 0..n {
                    let k = konst(f, Ty::I32, i as u64);
                    out.push(cmp(f, CmpOp::ULt, Ty::I32, k, abs));
                }
            } else {
                let mut run = konst(f, Ty::I8, 1);
                for &e in v {
                    let z = is_zero(f, ty, e);
                    let nz = flag_not(f, z);
                    run = bin(f, BinOp::And, Ty::I8, run, nz);
                    out.push(run);
                }
            }
            Result::Ok(out)
        };
        let (la, lb) = if explicit {
            (Some(ops[1]), Some(ops[2]))
        } else {
            (None, None)
        };
        let va = valid_of(self, f, la, &a)?;
        let vb = valid_of(self, f, lb, &b)?;
        let one = konst(f, Ty::I8, 1);
        let zero = konst(f, Ty::I8, 0);
        let and = |f: &mut FnCtx, x: Val, y: Val| bin(f, BinOp::And, Ty::I8, x, y);
        let or = |f: &mut FnCtx, x: Val, y: Val| bin(f, BinOp::Or, Ty::I8, x, y);
        let eq = |f: &mut FnCtx, x: Val, y: Val| cmp(f, CmpOp::Eq, ty, x, y);
        // Per-element result of the aggregation (`IntRes1`).
        let mut res: Vec<Val> = Vec::with_capacity(n);
        for i in 0..n {
            let r = match (ctl >> 2) & 3 {
                // Equal any: some valid element of `a` equals this valid element of `b`.
                0 => {
                    let mut acc = zero;
                    for j in 0..n {
                        let e = eq(f, a[j], b[i]);
                        let both = and(f, va[j], vb[i]);
                        let t = and(f, both, e);
                        acc = or(f, acc, t);
                    }
                    acc
                }
                // Ranges: `a` holds (low, high) pairs; this element lies in one of them.
                1 => {
                    let (ge, le) = if signed {
                        (CmpOp::SGe, CmpOp::SLe)
                    } else {
                        (CmpOp::UGe, CmpOp::ULe)
                    };
                    let mut acc = zero;
                    for p in 0..n / 2 {
                        let (lo, hi) = (2 * p, 2 * p + 1);
                        let c1 = cmp(f, ge, ty, b[i], a[lo]);
                        let c2 = cmp(f, le, ty, b[i], a[hi]);
                        let inside = and(f, c1, c2);
                        let valid = and(f, va[lo], va[hi]);
                        let valid = and(f, valid, vb[i]);
                        let t = and(f, valid, inside);
                        acc = or(f, acc, t);
                    }
                    acc
                }
                // Equal each: pairwise; two missing elements match, one missing does not.
                2 => {
                    let e = eq(f, a[i], b[i]);
                    let both = and(f, va[i], vb[i]);
                    let hit = and(f, both, e);
                    let na = flag_not(f, va[i]);
                    let nb = flag_not(f, vb[i]);
                    let none = and(f, na, nb);
                    or(f, hit, none)
                }
                // Equal ordered: `a` occurs in `b` starting here (a missing `a` element matches).
                _ => {
                    let mut acc = one;
                    for j in 0..n - i {
                        let e = eq(f, a[j], b[i + j]);
                        let both = and(f, va[j], vb[i + j]);
                        let hit = and(f, both, e);
                        let missing = flag_not(f, va[j]);
                        let t = or(f, hit, missing);
                        acc = and(f, acc, t);
                    }
                    acc
                }
            };
            res.push(r);
        }
        // Polarity: negative inverts, masked negative inverts only the valid elements of `b`.
        let res: Vec<Val> = match (ctl >> 4) & 3 {
            1 => res.iter().map(|&r| flag_not(f, r)).collect(),
            3 => res
                .iter()
                .zip(&vb)
                .map(|(&r, &v)| {
                    let inverted = flag_not(f, r);
                    let outside = flag_not(f, v);
                    let inv = and(f, v, inverted);
                    let keep = and(f, outside, r);
                    or(f, inv, keep)
                })
                .collect(),
            _ => res,
        };
        let mut any = zero;
        for &r in &res {
            any = or(f, any, r);
        }
        let tmp = f.b.alloca(16, 16);
        f.b.zero(tmp, 16);
        if mask {
            if ctl & 0x40 != 0 {
                // One element of all ones per set bit.
                for (i, &r) in res.iter().enumerate() {
                    let wide = resize(f, r, Ty::I8, ty, false);
                    let z = konst(f, ty, 0);
                    let ones = bin(f, BinOp::Sub, ty, z, wide);
                    store_lane(f, tmp, i as u64, ty, ones);
                }
            } else {
                let mut bits_set = konst(f, Ty::I16, 0);
                for (i, &r) in res.iter().enumerate() {
                    let wide = resize(f, r, Ty::I8, Ty::I16, false);
                    let sh = konst(f, Ty::I16, i as u64);
                    let moved = bin(f, BinOp::Shl, Ty::I16, wide, sh);
                    bits_set = bin(f, BinOp::Or, Ty::I16, bits_set, moved);
                }
                store_lane(f, tmp, 0, Ty::I16, bits_set);
            }
            self.simd_out(f, cx, ops[0], tmp, 16, span)?;
        } else {
            // The index of the lowest (or highest) set element; the element count when none.
            let mut idx = konst(f, Ty::I32, n as u64);
            let order: Vec<usize> = if ctl & 0x40 != 0 {
                (0..n).collect()
            } else {
                (0..n).rev().collect()
            };
            for i in order {
                let k = konst(f, Ty::I32, i as u64);
                idx = select(f, Ty::I32, res[i], k, idx);
            }
            self.vec_scalar_write(f, ops[0], Ty::I32, idx, span)?;
        }
        // AF and PF are cleared (PF is the lazily evaluated parity byte, odd = clear).
        let odd = konst(f, Ty::I8, 1);
        cx.flags = Flags {
            cf: Some(any),
            zf: Some(flag_not(f, vb[n - 1])),
            sf: Some(flag_not(f, va[n - 1])),
            of: Some(res[0]),
            pf: Some(odd),
        };
        Ok(())
    }
}
