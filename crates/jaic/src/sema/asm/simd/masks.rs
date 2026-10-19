//! Mask registers, compress and expand, ternary logic, AES and scatter.
use super::dispatch::SimdLocals;
use super::helpers::{load_dyn, mask_bit, mix_columns, store_dyn, when, xor8};
use super::{AesOp, VOp};
use crate::sema::asm::vec::{VOpd, load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, FnCtx, Result, Ty, UnOp, Val, bin, cmp, err, is_neg, konst,
    ptr_of, resize, select,
};

impl Compiler {
    pub(super) fn simd_masks(
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
            wm,
            ops,
            op,
            ..
        } = *c;
        match op {
            VOp::Aes(..) | VOp::Scatter(..) | VOp::MaskMov(..) => {
                self.simd_aes_scatter(f, cx, c, op)?;
            }
            VOp::Compress(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / ty.size();
                let all = konst(f, Ty::I64, u64::MAX);
                let bits = wm.map_or(all, |w| w.bits);
                // Pack active lanes to the front of a scratch buffer.
                let mut pos = konst(f, Ty::I64, 0);
                for i in 0..n {
                    let v = load_lane(f, s[0], i, ty);
                    store_dyn(f, tmp, pos, ty, v);
                    let on = mask_bit(f, bits, i);
                    let on = resize(f, on, Ty::I8, Ty::I64, false);
                    pos = bin(f, BinOp::Add, Ty::I64, pos, on);
                }
                if let VOpd::Mem(addr) = dst {
                    // Memory receives only the active elements.
                    let p = ptr_of(f, addr);
                    for j in 0..n {
                        let jv = konst(f, Ty::I64, j);
                        let inside = cmp(f, CmpOp::ULt, Ty::I64, jv, pos);
                        let v = load_lane(f, tmp, j, ty);
                        when(f, inside, |f| store_lane(f, p, j, ty, v));
                    }
                } else {
                    let old = self.vec_ptr(f, dst, span)?;
                    let zeroing = wm.is_some_and(|w| w.zeroing);
                    for j in 0..n {
                        let jv = konst(f, Ty::I64, j);
                        let inside = cmp(f, CmpOp::ULt, Ty::I64, jv, pos);
                        let v = load_lane(f, tmp, j, ty);
                        let keep = if zeroing {
                            konst(f, ty, 0)
                        } else {
                            load_lane(f, old, j, ty)
                        };
                        let r = select(f, ty, inside, v, keep);
                        store_lane(f, tmp, j, ty, r);
                    }
                    self.simd_out(f, cx, dst, tmp, width, span)?;
                }
            }
            VOp::Expand(ty) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / ty.size();
                let all = konst(f, Ty::I64, u64::MAX);
                let bits = wm.map_or(all, |w| w.bits);
                let mut pos = konst(f, Ty::I64, 0);
                let slot = f.b.alloca(8, 8);
                for i in 0..n {
                    let on = mask_bit(f, bits, i);
                    // Only active lanes read the source (it may be memory ending early).
                    let zero = konst(f, ty, 0);
                    f.b.store(ty, slot, zero);
                    let src = s[0];
                    let at = pos;
                    when(f, on, |f| {
                        let v = load_dyn(f, src, at, ty);
                        f.b.store(ty, slot, v);
                    });
                    let v = f.b.load(ty, slot);
                    store_lane(f, tmp, i, ty, v);
                    let on = resize(f, on, Ty::I8, Ty::I64, false);
                    pos = bin(f, BinOp::Add, Ty::I64, pos, on);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Ternlog(ty) => {
                let (s, k) = self.simd_args(f, ops, 3, true, span)?;
                for i in 0..width / 8 {
                    let a = load_lane(f, s[0], i, Ty::I64);
                    let b = load_lane(f, s[1], i, Ty::I64);
                    let c = load_lane(f, s[2], i, Ty::I64);
                    let na = f.b.un(UnOp::Not, Ty::I64, a);
                    let nb = f.b.un(UnOp::Not, Ty::I64, b);
                    let nc = f.b.un(UnOp::Not, Ty::I64, c);
                    let mut r = konst(f, Ty::I64, 0);
                    for t in 0..8u64 {
                        if (k >> t) & 1 == 0 {
                            continue;
                        }
                        // Truth-table row t = (a << 2) | (b << 1) | c.
                        let x = if t & 4 != 0 {
                            a
                        } else {
                            na
                        };
                        let y = if t & 2 != 0 {
                            b
                        } else {
                            nb
                        };
                        let z = if t & 1 != 0 {
                            c
                        } else {
                            nc
                        };
                        let term = bin(f, BinOp::And, Ty::I64, x, y);
                        let term = bin(f, BinOp::And, Ty::I64, term, z);
                        r = bin(f, BinOp::Or, Ty::I64, r, term);
                    }
                    store_lane(f, tmp, i, Ty::I64, r);
                }
                let _ = ty;
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Dp(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let per = 16 / ty.size();
                for blk in 0..width / 16 {
                    let base = blk * per;
                    let mut terms = Vec::new();
                    for i in 0..per {
                        let a = load_lane(f, s[0], base + i, ty);
                        let b = load_lane(f, s[1], base + i, ty);
                        let p = bin(f, BinOp::FMul, ty, a, b);
                        let z = f.b.fconst(ty, 0.0);
                        terms.push(if (k >> (4 + i)) & 1 != 0 {
                            p
                        } else {
                            z
                        });
                    }
                    // (t0 + t1) + (t2 + t3), as the hardware adds.
                    let sum = if per == 4 {
                        let x = bin(f, BinOp::FAdd, ty, terms[0], terms[1]);
                        let y = bin(f, BinOp::FAdd, ty, terms[2], terms[3]);
                        bin(f, BinOp::FAdd, ty, x, y)
                    } else {
                        bin(f, BinOp::FAdd, ty, terms[0], terms[1])
                    };
                    for i in 0..per {
                        let v = if (k >> i) & 1 != 0 {
                            sum
                        } else {
                            f.b.fconst(ty, 0.0)
                        };
                        store_lane(f, tmp, base + i, ty, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Pclmul(fixed) => {
                let (s, k) = match fixed {
                    Some(k) => (self.simd_args(f, ops, 2, false, span)?.0, k as u64),
                    None => self.simd_args(f, ops, 2, true, span)?,
                };
                for blk in 0..width / 16 {
                    let a = load_lane(f, s[0], 2 * blk + (k & 1), Ty::I64);
                    let b = load_lane(f, s[1], 2 * blk + ((k >> 4) & 1), Ty::I64);
                    let (lo, hi) = Self::clmul64(f, a, b);
                    store_lane(f, tmp, 2 * blk, Ty::I64, lo);
                    store_lane(f, tmp, 2 * blk + 1, Ty::I64, hi);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }

    fn simd_aes_scatter(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
        op: VOp,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let dst = c.dst;
        let tmp = c.tmp;
        let width = c.width;
        let wm = c.wm;
        let ops = c.ops;
        match op {
            VOp::Aes(aes) => {
                let (s, k) = match aes {
                    AesOp::Imc => self.simd_args(f, ops, 1, false, span)?,
                    AesOp::KeygenAssist => self.simd_args(f, ops, 1, true, span)?,
                    _ => self.simd_args(f, ops, 2, false, span)?,
                };
                let tables = self.aes_table_ptr(f);
                let inv_tables = f.b.ptr_offset(tables, 256);
                let blocks = if matches!(aes, AesOp::Imc | AesOp::KeygenAssist) {
                    1
                } else {
                    width / 16
                };
                for blk in 0..blocks {
                    let base = blk * 16;
                    let mut st = [Val(0); 16];
                    for (j, b) in st.iter_mut().enumerate() {
                        *b = load_lane(f, s[0], base + j as u64, Ty::I8);
                    }
                    let sub = |f: &mut FnCtx, table: Val, v: Val| {
                        let at = resize(f, v, Ty::I8, Ty::I64, false);
                        load_dyn(f, table, at, Ty::I8)
                    };
                    let out = match aes {
                        AesOp::Imc => mix_columns(f, &st, true),
                        AesOp::KeygenAssist => {
                            // [Sub(X1), Rot(Sub(X1)) ^ rcon, Sub(X3), Rot(Sub(X3)) ^ rcon]
                            let mut out = [Val(0); 16];
                            for (w, x) in [(0usize, 1usize), (2, 3)] {
                                let word: Vec<Val> =
                                    (0..4).map(|b| sub(f, tables, st[4 * x + b])).collect();
                                for b in 0..4 {
                                    out[4 * w + b] = word[b];
                                    out[4 * (w + 1) + b] = word[(b + 1) % 4];
                                }
                                let rcon = konst(f, Ty::I8, k);
                                out[4 * (w + 1)] = xor8(f, out[4 * (w + 1)], rcon);
                            }
                            out
                        }
                        _ => {
                            let enc = matches!(aes, AesOp::Enc | AesOp::EncLast);
                            // (Inv)ShiftRows + (Inv)SubBytes: row r rotates by r columns.
                            // Encryption rotates left by `r`, decryption right: `3 * r` is `-r` mod 4.
                            let (table, step) = if enc {
                                (tables, 1)
                            } else {
                                (inv_tables, 3)
                            };
                            let mut sh = [Val(0); 16];
                            for c in 0..4 {
                                for r in 0..4 {
                                    let from = (c + step * r) % 4;
                                    sh[4 * c + r] = sub(f, table, st[4 * from + r]);
                                }
                            }
                            match aes {
                                AesOp::Enc => mix_columns(f, &sh, false),
                                AesOp::Dec => mix_columns(f, &sh, true),
                                _ => sh,
                            }
                        }
                    };
                    for (j, &v) in out.iter().enumerate() {
                        let v = if matches!(aes, AesOp::Imc | AesOp::KeygenAssist) {
                            v
                        } else {
                            let key = load_lane(f, s[1], base + j as u64, Ty::I8);
                            xor8(f, v, key)
                        };
                        store_lane(f, tmp, base + j as u64, Ty::I8, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, blocks * 16, span)?;
            }
            VOp::Scatter(index_size, elem_size) => {
                let (
                    VOpd::Vsib {
                        base,
                        index,
                        scale,
                    },
                    Some(src),
                ) = (dst, ops.get(1))
                else {
                    return err(
                        span,
                        format!("`{name}` takes [base + vindex*scale] &mask, src"),
                    );
                };
                let Some(wm) = wm else {
                    return err(span, format!("`{name}` needs a mask (&k)"));
                };
                let src = self.vec_ptr(f, *src, span)?;
                let (it, et) = (Ty::int(index_size), Ty::int(elem_size));
                let lanes = width / index_size.max(elem_size);
                let scale = konst(f, Ty::I64, scale);
                for i in 0..lanes {
                    let on = mask_bit(f, wm.bits, i);
                    let idx = load_lane(f, index, i, it);
                    let idx = resize(f, idx, it, Ty::I64, true);
                    let off = bin(f, BinOp::Mul, Ty::I64, idx, scale);
                    let addr = bin(f, BinOp::Add, Ty::I64, base, off);
                    let v = load_lane(f, src, i, et);
                    when(f, on, |f| {
                        let p = ptr_of(f, addr);
                        f.b.store(et, p, v);
                    });
                }
                // The mask register is cleared once every element is stored.
                if let Some(reg) = wm.reg {
                    let z = konst(f, Ty::I64, 0);
                    f.b.store(Ty::I64, reg, z);
                }
            }
            VOp::MaskMov(ty) => {
                if ops.len() != 3 {
                    return err(span, format!("`{name}` takes 3 operands"));
                }
                let mask = self.vec_ptr(f, ops[1], span)?;
                let n = width / ty.size();
                match (ops[0], ops[2]) {
                    (VOpd::Mem(addr), src) => {
                        let src = self.vec_ptr(f, src, span)?;
                        let p = ptr_of(f, addr);
                        for i in 0..n {
                            let m = load_lane(f, mask, i, ty);
                            let on = is_neg(f, ty, m);
                            let v = load_lane(f, src, i, ty);
                            when(f, on, |f| store_lane(f, p, i, ty, v));
                        }
                    }
                    (_, VOpd::Mem(addr)) => {
                        let p = ptr_of(f, addr);
                        let slot = f.b.alloca(8, 8);
                        for i in 0..n {
                            let m = load_lane(f, mask, i, ty);
                            let on = is_neg(f, ty, m);
                            let zero = konst(f, ty, 0);
                            f.b.store(ty, slot, zero);
                            when(f, on, |f| {
                                let v = load_lane(f, p, i, ty);
                                f.b.store(ty, slot, v);
                            });
                            let v = f.b.load(ty, slot);
                            store_lane(f, tmp, i, ty, v);
                        }
                        self.simd_out(f, cx, dst, tmp, width, span)?;
                    }
                    _ => {
                        return err(
                            span,
                            format!("`{name}` moves between a register and memory"),
                        );
                    }
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }
}
