//! Shuffles, blends, permutes and element insertion and extraction.
use super::dispatch::SimdLocals;
use super::helpers::{load_dyn, mask_bit, saturate};
use super::{DupKind, Sat, VOp};
use crate::diagnostics::Code;
use crate::sema::asm::vec::{VOpd, lane_addr, load_lane, store_lane};
use crate::sema::asm::{
    AsmCtx, BinOp, CmpOp, Compiler, FnCtx, Intrinsic, Result, Ty, bin, bits, cmp, err, is_neg,
    konst, resize, select,
};

impl Compiler {
    pub(super) fn simd_shuffle(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &SimdLocals<'_>,
    ) -> Result<()> {
        let SimdLocals {
            span,
            dst,
            tmp,
            block,
            width,
            wm,
            ops,
            op,
            ..
        } = *c;
        match op {
            VOp::Pshufb
            | VOp::Perm(..)
            | VOp::PermTwo(..)
            | VOp::PermilImm(..)
            | VOp::Perm2x128
            | VOp::Shuf128(..) => self.simd_permutes(f, cx, c, op)?,
            VOp::Unpack(high, ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let per = block / ty.size();
                for blk in 0..width / block {
                    let base = blk * per;
                    let from = base
                        + if high {
                            per / 2
                        } else {
                            0
                        };
                    for k in 0..per / 2 {
                        let a = load_lane(f, s[0], from + k, ty);
                        let b = load_lane(f, s[1], from + k, ty);
                        store_lane(f, tmp, base + 2 * k, ty, a);
                        store_lane(f, tmp, base + 2 * k + 1, ty, b);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::ShufHalf(high) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                f.b.copy(tmp, s[0], width);
                for blk in 0..width / 16 {
                    let base = blk * 8
                        + if high {
                            4
                        } else {
                            0
                        };
                    for j in 0..4 {
                        let pick = (k >> (2 * j)) & 3;
                        let v = load_lane(f, s[0], base + pick, Ty::I16);
                        store_lane(f, tmp, base + j, Ty::I16, v);
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Shufpd => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for i in 0..width / 8 {
                    let blk = i / 2 * 2;
                    let src = if i % 2 == 0 {
                        s[0]
                    } else {
                        s[1]
                    };
                    let pick = (k >> i) & 1;
                    let v = load_lane(f, src, blk + pick, Ty::I64);
                    store_lane(f, tmp, i, Ty::I64, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Palignr => {
                // Per block: bytes [imm, imm + block) of hi:lo (lo = second source).
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for blk in 0..width / block {
                    let base = blk * block;
                    for j in 0..block {
                        let at = j + k;
                        if at < block {
                            let v = load_lane(f, s[1], base + at, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        } else if at < 2 * block {
                            let v = load_lane(f, s[0], base + at - block, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Valign(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let n = width / ty.size();
                let k = k % n;
                for j in 0..n {
                    let at = j + k;
                    let v = if at < n {
                        load_lane(f, s[1], at, ty)
                    } else {
                        load_lane(f, s[0], at - n, ty)
                    };
                    store_lane(f, tmp, j, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::ByteShift(left) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                for blk in 0..width / 16 {
                    let base = blk * 16;
                    for j in 0..16u64 {
                        let from = if left {
                            j.checked_sub(k)
                        } else {
                            Some(j + k)
                        };
                        if let Some(from) = from.filter(|&x| x < 16) {
                            let v = load_lane(f, s[0], base + from, Ty::I8);
                            store_lane(f, tmp, base + j, Ty::I8, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::BlendImm(ty) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for i in 0..width / ty.size() {
                    let src = if (k >> (i % 8)) & 1 != 0 {
                        s[1]
                    } else {
                        s[0]
                    };
                    let v = load_lane(f, src, i, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::BlendVar(ty) => {
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let m = load_lane(f, s[2], i, ty);
                    let on = is_neg(f, ty, m);
                    let r = select(f, ty, on, b, a);
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::BlendMask(ty) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                for i in 0..width / ty.size() {
                    let a = load_lane(f, s[0], i, ty);
                    let b = load_lane(f, s[1], i, ty);
                    let r = match wm {
                        Some(wm) => {
                            let on = mask_bit(f, wm.bits, i);
                            let off = if wm.zeroing {
                                konst(f, ty, 0)
                            } else {
                                a
                            };
                            select(f, ty, on, b, off)
                        }
                        None => b,
                    };
                    store_lane(f, tmp, i, ty, r);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }

    fn simd_permutes(
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
        let block = c.block;
        let last_imm = c.last_imm;
        let width = c.width;
        let ops = c.ops;
        match op {
            VOp::Pshufb => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                // `ByteLookup` zeroes every index of 16 or more, pshufb only those with the top
                // bit set: `& 0x8F` maps one onto the other (bits 4-6 are ignored by pshufb).
                let keep = konst(f, Ty::I8, 0x8F);
                let adj = f.b.alloca(64, 16);
                for i in 0..width {
                    let ctl = load_lane(f, s[1], i, Ty::I8);
                    let ctl = bin(f, BinOp::And, Ty::I8, ctl, keep);
                    store_lane(f, adj, i, Ty::I8, ctl);
                }
                for blk in 0..width / block {
                    let at = blk * block;
                    let table = lane_addr(f, s[0], at);
                    let idx = lane_addr(f, adj, at);
                    let out = lane_addr(f, tmp, at);
                    f.b.intrinsic(Intrinsic::ByteLookup(16), vec![out, table, idx], &[]);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Perm(ty) if last_imm => {
                // permq/permpd imm: 64-bit lanes within each 256-bit half.
                if ty != Ty::I64 {
                    return err(
                        Code::AsmImmediate,
                        span,
                        format!("`{name}` does not take an immediate"),
                    );
                }
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                for i in 0..width / 8 {
                    let base = i / 4 * 4;
                    let pick = (k >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, s[0], base + pick, Ty::I64);
                    store_lane(f, tmp, i, Ty::I64, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Perm(ty) => {
                // `perm* dst, index, table`
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let n = width / ty.size();
                let m = konst(f, Ty::I64, n - 1);
                for i in 0..n {
                    let idx = load_lane(f, s[0], i, ty);
                    let idx = resize(f, idx, ty, Ty::I64, false);
                    let idx = bin(f, BinOp::And, Ty::I64, idx, m);
                    let v = load_dyn(f, s[1], idx, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::PermTwo(ty, index_in_dst) => {
                // permt2: dst = table1, (index, table2); permi2: dst = index, (table1, table2).
                let (s, _) = self.simd_args(f, ops, 3, false, span)?;
                let (idx, t1, t2) = if index_in_dst {
                    (s[0], s[1], s[2])
                } else {
                    (s[1], s[0], s[2])
                };
                let n = width / ty.size();
                let m = konst(f, Ty::I64, n - 1);
                let sel_bit = konst(f, Ty::I64, n);
                for i in 0..n {
                    let ix = load_lane(f, idx, i, ty);
                    let ix = resize(f, ix, ty, Ty::I64, false);
                    let second = bin(f, BinOp::And, Ty::I64, ix, sel_bit);
                    let zero = konst(f, Ty::I64, 0);
                    let second = cmp(f, CmpOp::Ne, Ty::I64, second, zero);
                    let at = bin(f, BinOp::And, Ty::I64, ix, m);
                    let a = load_dyn(f, t1, at, ty);
                    let b = load_dyn(f, t2, at, ty);
                    let v = select(f, ty, second, b, a);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::PermilImm(ty) if !last_imm => {
                // permilps/pd with a vector control: `permilps dst, src, control`.
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let per = 16 / ty.size();
                for i in 0..width / ty.size() {
                    let c = load_lane(f, s[1], i, ty);
                    let c = resize(f, c, ty, Ty::I64, false);
                    let c = if ty == Ty::I64 {
                        let one = konst(f, Ty::I64, 1);
                        bin(f, BinOp::LShr, Ty::I64, c, one)
                    } else {
                        c
                    };
                    let m = konst(f, Ty::I64, per - 1);
                    let c = bin(f, BinOp::And, Ty::I64, c, m);
                    let base = lane_addr(f, s[0], i / per * 16);
                    let v = load_dyn(f, base, c, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::PermilImm(ty) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let per = 16 / ty.size();
                for i in 0..width / ty.size() {
                    let pick = if ty == Ty::I64 {
                        (k >> (i % 8)) & 1
                    } else {
                        (k >> (2 * (i % 4))) & 3
                    };
                    let v = load_lane(f, s[0], i / per * per + pick, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Perm2x128 => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                for half in 0..2 {
                    let sel = (k >> (4 * half)) & 0xf;
                    if sel & 8 == 0 {
                        let src = if sel & 2 == 0 {
                            s[0]
                        } else {
                            s[1]
                        };
                        let from = lane_addr(f, src, (sel & 1) * 16);
                        let to = lane_addr(f, tmp, half * 16);
                        f.b.copy(to, from, 16);
                    }
                }
                self.simd_out(f, cx, dst, tmp, 32, span)?;
            }
            VOp::Shuf128(_) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let chunks = width / 16;
                let bits_per = if chunks == 2 {
                    1
                } else {
                    2
                };
                for c in 0..chunks {
                    let src = if c < chunks / 2 {
                        s[0]
                    } else {
                        s[1]
                    };
                    let pick = (k >> (bits_per * c)) & ((1 << bits_per) - 1);
                    let from = lane_addr(f, src, pick * 16);
                    let to = lane_addr(f, tmp, c * 16);
                    f.b.copy(to, from, 16);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }


    pub(super) fn simd_insert(
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
            block,
            width,
            ops,
            inst,
            op,
            ..
        } = *c;
        match op {
            VOp::Insert(chunk, _) => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                f.b.copy(tmp, s[0], width);
                let slots = (width / chunk).max(1);
                let at = lane_addr(f, tmp, (k % slots) * chunk);
                f.b.copy(at, s[1], chunk);
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Extract(chunk, _) => {
                let (s, k) = self.simd_args(f, ops, 1, true, span)?;
                let slots = (width / chunk).max(1);
                let from = lane_addr(f, s[0], (k % slots) * chunk);
                f.b.copy(tmp, from, chunk);
                self.simd_out(f, cx, dst, tmp, chunk, span)?;
            }
            VOp::Pinsr(ty) => {
                // `pinsrb dst, [a,] src, imm`: src is a general-purpose register or memory.
                let (a, src, k) = match ops {
                    [_, src, VOpd::Imm(k)] => (self.vec_ptr(f, dst, span)?, *src, *k as u64),
                    [_, a, src, VOpd::Imm(k)] => (self.vec_ptr(f, *a, span)?, *src, *k as u64),
                    _ => {
                        return err(
                            Code::AsmOperandForm,
                            span,
                            format!("`{name}` takes dst, [src,] value, imm8"),
                        );
                    }
                };
                let v = self.vec_scalar_read(f, src, ty, span)?;
                f.b.copy(tmp, a, 16);
                store_lane(f, tmp, k % (16 / ty.size()), ty, v);
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            VOp::Pextr(_) | VOp::Extractps => {
                let ty = match op {
                    VOp::Pextr(ty) => ty,
                    _ => Ty::I32,
                };
                let (src, k) = match ops {
                    [_, src, VOpd::Imm(k)] => (self.vec_ptr(f, *src, span)?, *k as u64),
                    _ => {
                        return err(
                            Code::AsmOperandForm,
                            span,
                            format!("`{name}` takes dst, src, imm8"),
                        );
                    }
                };
                let v = load_lane(f, src, k % (16 / ty.size()), ty);
                match dst {
                    // Into a register the value is zero-extended (a 32-bit write below .q).
                    VOpd::Gpr(_) => {
                        let wide = if ty == Ty::I64 {
                            Ty::I64
                        } else {
                            Ty::I32
                        };
                        let v = resize(f, v, ty, wide, false);
                        self.vec_scalar_write(f, dst, wide, v, span)?;
                    }
                    _ => self.vec_scalar_write(f, dst, ty, v, span)?,
                }
            }
            VOp::Insertps => {
                let (s, k) = self.simd_args(f, ops, 2, true, span)?;
                let from_mem = matches!(ops[ops.len() - 2], VOpd::Mem(_));
                let pick = if from_mem {
                    0
                } else {
                    (k >> 6) & 3
                };
                f.b.copy(tmp, s[0], 16);
                let v = load_lane(f, s[1], pick, Ty::I32);
                store_lane(f, tmp, (k >> 4) & 3, Ty::I32, v);
                for j in 0..4 {
                    if k & (1 << j) != 0 {
                        let z = konst(f, Ty::I32, 0);
                        store_lane(f, tmp, j, Ty::I32, z);
                    }
                }
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            VOp::MovHalf(high) => {
                let at = if high {
                    8
                } else {
                    0
                };
                if let [VOpd::Mem(_), src] = ops {
                    let src = self.vec_ptr(f, *src, span)?;
                    let p = self.vec_ptr(f, dst, span)?;
                    let from = lane_addr(f, src, at);
                    f.b.copy(p, from, 8);
                } else {
                    let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                    f.b.copy(tmp, s[0], 16);
                    let to = lane_addr(f, tmp, at);
                    f.b.copy(to, s[1], 8);
                    self.simd_out(f, cx, dst, tmp, 16, span)?;
                }
            }
            VOp::Movhlps | VOp::Movlhps => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let hi = lane_addr(f, tmp, 8);
                if op == VOp::Movhlps {
                    let b_hi = lane_addr(f, s[1], 8);
                    let a_hi = lane_addr(f, s[0], 8);
                    f.b.copy(tmp, b_hi, 8);
                    f.b.copy(hi, a_hi, 8);
                } else {
                    f.b.copy(tmp, s[0], 8);
                    f.b.copy(hi, s[1], 8);
                }
                self.simd_out(f, cx, dst, tmp, 16, span)?;
            }
            VOp::Dup(kind) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let (ty, pick) = match kind {
                    DupKind::Low64 => (Ty::I64, 0),
                    DupKind::EvenF32 => (Ty::I32, 0),
                    DupKind::OddF32 => (Ty::I32, 1),
                };
                for i in 0..width / ty.size() {
                    let v = load_lane(f, s[0], i / 2 * 2 + pick, ty);
                    store_lane(f, tmp, i, ty, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Extend(from, to, signed) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                for i in 0..width / to.size() {
                    let v = load_lane(f, s[0], i, from);
                    let v = resize(f, v, from, to, signed);
                    store_lane(f, tmp, i, to, v);
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Narrow(from, to, sat) => {
                let (s, _) = self.simd_args(f, ops, 1, false, span)?;
                let n = width / from.size();
                for i in 0..n {
                    let v = load_lane(f, s[0], i, from);
                    let v = match sat {
                        Sat::Wrap => resize(f, v, from, to, false),
                        Sat::Signed => saturate(f, from, v, to, true),
                        Sat::Unsigned => {
                            let max = konst(f, from, (1u64 << bits(to)) - 1);
                            let over = cmp(f, CmpOp::UGt, from, v, max);
                            let v = select(f, from, over, max, v);
                            resize(f, v, from, to, false)
                        }
                    };
                    store_lane(f, tmp, i, to, v);
                }
                self.simd_out(f, cx, dst, tmp, n * to.size(), span)?;
            }
            VOp::Pack(from, unsigned) => {
                let (s, _) = self.simd_args(f, ops, 2, false, span)?;
                let to = Ty::int(from.size() / 2);
                let per = block / from.size();
                for blk in 0..width / block {
                    for (h, src) in [s[0], s[1]].into_iter().enumerate() {
                        for j in 0..per {
                            let v = load_lane(f, src, blk * per + j, from);
                            let wide = if from == Ty::I16 {
                                Ty::I32
                            } else {
                                Ty::I64
                            };
                            let v = resize(f, v, from, wide, true);
                            let v = saturate(f, wide, v, to, !unsigned);
                            let at = blk * 2 * per + h as u64 * per + j;
                            store_lane(f, tmp, at, to, v);
                        }
                    }
                }
                self.simd_out(f, cx, dst, tmp, width, span)?;
            }
            VOp::Cvt(conv) => self.simd_convert(f, cx, c, conv)?,
            VOp::Half(to_half) => self.simd_half(f, cx, c, to_half)?,
            VOp::Sha(sha) => self.simd_sha(f, cx, inst, sha, ops, tmp)?,
            VOp::Gf(gf) => self.simd_gf(f, cx, c, gf)?,
            _ => unreachable!("routed by asm_simd"),
        }
        Ok(())
    }
}
