//! The vector instruction groups `asm_vec_inst` hands an opcode to.
use super::*;

impl Compiler {
    pub(super) fn vec_moves(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &VecCall,
        op: VOp,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let width = c.width;
        let ops = &c.ops;
        let tmp = c.tmp;
        let n = c.ops.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            VOp::Move => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                f.b.copy(tmp, src, width);
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::MovScalarInt(size) => {
                want(2, 2)?;
                let it = Ty::int(size);
                match (ops[0], ops[1]) {
                    (VOpd::Reg(dst), VOpd::Reg(src)) => {
                        f.b.zero(tmp, 16);
                        f.b.copy(tmp, src, size);
                        self.vec_store(f, cx, VOpd::Reg(dst), tmp, 16, span)?;
                    }
                    (VOpd::Reg(_), src) => {
                        let v = self.vec_scalar_read(f, src, it, span)?;
                        f.b.zero(tmp, 16);
                        f.b.store(it, tmp, v);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (dst, VOpd::Reg(src)) => {
                        let v = f.b.load(it, src);
                        self.vec_scalar_write(f, dst, it, v, span)?;
                    }
                    _ => return err(span, format!("`{name}` needs a vector register operand")),
                }
            }
            VOp::MovScalarFloat(ty) => {
                want(2, 3)?;
                let size = ty.size();
                match (n, ops[0], ops[1]) {
                    (2, VOpd::Reg(dst), VOpd::Reg(src)) => {
                        // Register to register keeps the destination's other lanes.
                        f.b.copy(tmp, dst, 16);
                        f.b.copy(tmp, src, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (2, VOpd::Reg(_), src) => {
                        let p = self.vec_ptr(f, src, span)?;
                        f.b.zero(tmp, 16);
                        f.b.copy(tmp, p, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    (2, dst, VOpd::Reg(src)) => {
                        let p = self.vec_ptr(f, dst, span)?;
                        f.b.copy(p, src, size);
                    }
                    (3, VOpd::Reg(_), a) => {
                        let a = self.vec_ptr(f, a, span)?;
                        let b = self.vec_ptr(f, ops[2], span)?;
                        f.b.copy(tmp, a, 16);
                        f.b.copy(tmp, b, size);
                        self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                    }
                    _ => return err(span, format!("`{name}` needs a vector register operand")),
                }
            }
            VOp::Broadcast(size) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let mut at = 0;
                while at < width {
                    let p = lane_addr(f, tmp, at);
                    f.b.copy(p, src, size);
                    at += size;
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Movmsk(size) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let ty = Ty::int(size);
                let mut mask = konst(f, Ty::I64, 0);
                if size == 1 && matches!(width, 16 | 32 | 64) {
                    mask =
                        f.b.intrinsic(Intrinsic::SignMask(width as u8), vec![src], &[Ty::I64])[0];
                } else {
                    for i in 0..width / size {
                        let x = load_lane(f, src, i, ty);
                        let neg = is_neg(f, ty, x);
                        let bit = resize(f, neg, Ty::I8, Ty::I64, false);
                        let at = konst(f, Ty::I64, i);
                        let bit = bin(f, BinOp::Shl, Ty::I64, bit, at);
                        mask = bin(f, BinOp::Or, Ty::I64, mask, bit);
                    }
                }
                self.vec_scalar_write(f, ops[0], Ty::I64, mask, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    pub(super) fn vec_lane_ops(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        inst: &AsmInst,
        c: &VecCall,
        op: VOp,
    ) -> Result<()> {
        let span = c.span;
        let width = c.width;
        let ops = &c.ops;
        let tmp = c.tmp;
        let n = c.ops.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            VOp::Bin(lane, ty, scalar) => {
                want(2, 3)?;
                let (a, b) = self.vec_sources(f, ops, span)?;
                if scalar {
                    f.b.copy(tmp, a, 16);
                    let x = load_lane(f, a, 0, ty);
                    let y = load_lane(f, b, 0, ty);
                    let r = lane_op(f, lane, ty, x, y);
                    store_lane(f, tmp, 0, ty, r);
                    self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                } else {
                    for i in 0..width / ty.size() {
                        let x = load_lane(f, a, i, ty);
                        let y = load_lane(f, b, i, ty);
                        let r = lane_op(f, lane, ty, x, y);
                        store_lane(f, tmp, i, ty, r);
                    }
                    self.vec_store(f, cx, ops[0], tmp, width, span)?;
                }
            }
            VOp::Sqrt(ty, scalar) => {
                want(2, 3)?;
                if scalar {
                    // `sqrtss dst, src` / `sqrtss dst, a, src`: lane 0 from src, the rest from a.
                    let (a, b) = self.vec_sources(f, ops, span)?;
                    f.b.copy(tmp, a, 16);
                    let x = load_lane(f, b, 0, ty);
                    let r = float_unary(f, Intrinsic::Sqrt, ty, x);
                    store_lane(f, tmp, 0, ty, r);
                    self.vec_store(f, cx, ops[0], tmp, 16, span)?;
                } else {
                    want(2, 2)?;
                    let src = self.vec_ptr(f, ops[1], span)?;
                    for i in 0..width / ty.size() {
                        let x = load_lane(f, src, i, ty);
                        let r = float_unary(f, Intrinsic::Sqrt, ty, x);
                        store_lane(f, tmp, i, ty, r);
                    }
                    self.vec_store(f, cx, ops[0], tmp, width, span)?;
                }
            }
            VOp::Abs(ty) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                for i in 0..width / ty.size() {
                    let x = load_lane(f, src, i, ty);
                    let neg = is_neg(f, ty, x);
                    let zero = konst(f, ty, 0);
                    let minus = bin(f, BinOp::Sub, ty, zero, x);
                    let r = select(f, ty, neg, minus, x);
                    store_lane(f, tmp, i, ty, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Shift(kind, ty) => {
                want(2, 3)?;
                // (dst, count) or (dst, src, count); the count is an immediate or a register's low quadword.
                let src = if n == 3 {
                    self.vec_ptr(f, ops[1], span)?
                } else {
                    self.vec_ptr(f, ops[0], span)?
                };
                let count = match ops[n - 1] {
                    VOpd::Imm(v) => konst(f, Ty::I64, (v as u64) & 0xff),
                    other => {
                        let p = self.vec_ptr(f, other, span)?;
                        f.b.load(Ty::I64, p)
                    }
                };
                let lane_bits = bits(ty);
                let max = konst(f, Ty::I64, lane_bits - 1);
                let over = cmp(f, CmpOp::UGt, Ty::I64, count, max);
                let clamped = select(f, Ty::I64, over, max, count);
                let amount = resize(f, clamped, Ty::I64, ty, false);
                for i in 0..width / ty.size() {
                    let x = load_lane(f, src, i, ty);
                    let r = match kind {
                        Shift::Left | Shift::Logical => {
                            let op = if kind == Shift::Left {
                                BinOp::Shl
                            } else {
                                BinOp::LShr
                            };
                            let s = bin(f, op, ty, x, amount);
                            let zero = konst(f, ty, 0);
                            select(f, ty, over, zero, s)
                        }
                        // An oversized arithmetic shift fills with the sign: shifting by bits-1 does that.
                        Shift::Arith => bin(f, BinOp::AShr, ty, x, amount),
                    };
                    store_lane(f, tmp, i, ty, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Cvt(kind) => {
                want(2, 2)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                for i in 0..width / 4 {
                    let r = match kind {
                        Cvt::IntToFloat => {
                            let x = load_lane(f, src, i, Ty::I32);
                            f.b.conv(ConvOp::SToF, Ty::I32, Ty::F32, x)
                        }
                        Cvt::FloatToInt | Cvt::FloatToIntTrunc => {
                            let x = load_lane(f, src, i, Ty::F32);
                            let mode = match (kind, inst.evex.round) {
                                (_, Some(Some(m))) => m,
                                (Cvt::FloatToIntTrunc, _) => 'z',
                                _ => 'n',
                            };
                            let v = float_to_s32(f, x, mode);
                            bitcast(f, Ty::I32, Ty::F32, v)
                        }
                    };
                    store_lane(f, tmp, i, Ty::F32, r);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    pub(super) fn vec_shuffles(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &VecCall,
        op: VOp,
    ) -> Result<()> {
        let span = c.span;
        let width = c.width;
        let ops = &c.ops;
        let tmp = c.tmp;
        let n = c.ops.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            VOp::Pshufd => {
                want(3, 3)?;
                let src = self.vec_ptr(f, ops[1], span)?;
                let imm = self.vec_imm(ops[2], span)?;
                for i in 0..width / 4 {
                    let group = i / 4 * 4;
                    let pick = (imm >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, src, group + pick, Ty::I32);
                    store_lane(f, tmp, i, Ty::I32, v);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            VOp::Shufps => {
                want(3, 4)?;
                let (a, b) = if n == 4 {
                    (
                        self.vec_ptr(f, ops[1], span)?,
                        self.vec_ptr(f, ops[2], span)?,
                    )
                } else {
                    (
                        self.vec_ptr(f, ops[0], span)?,
                        self.vec_ptr(f, ops[1], span)?,
                    )
                };
                let imm = self.vec_imm(ops[n - 1], span)?;
                for i in 0..width / 4 {
                    let group = i / 4 * 4;
                    let from = if i % 4 < 2 {
                        a
                    } else {
                        b
                    };
                    let pick = (imm >> (2 * (i % 4))) & 3;
                    let v = load_lane(f, from, group + pick, Ty::I32);
                    store_lane(f, tmp, i, Ty::I32, v);
                }
                self.vec_store(f, cx, ops[0], tmp, width, span)?;
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }

    pub(super) fn vec_gather(
        &mut self,
        f: &mut FnCtx,
        cx: &mut AsmCtx,
        c: &VecCall,
        op: VOp,
    ) -> Result<()> {
        let span = c.span;
        let name = c.name;
        let width = c.width;
        let ops = &c.ops;
        let writemask = c.writemask;
        let tmp = c.tmp;
        let n = c.ops.len();
        let want = |lo: usize, hi: usize| c.want(lo, hi);
        match op {
            VOp::Gather(index_size, elem_size) => {
                // AVX2: `gatherdps dst, [vsib], vmask`; AVX-512: `gatherdps dst &k, [vsib]`.
                want(2, 3)?;
                if n == 2 && writemask.is_none() {
                    return err(span, format!("`{name}` needs a mask: a vector or &k"));
                }
                let VOpd::Vsib {
                    base,
                    index,
                    scale,
                } = ops[1]
                else {
                    return err(
                        span,
                        format!(
                            "`{name}` needs a vector-indexed memory operand ([base + vindex*scale])"
                        ),
                    );
                };
                let mask = if n == 3 {
                    Some(self.vec_ptr(f, ops[2], span)?)
                } else {
                    None
                };
                let dst = self.vec_ptr(f, ops[0], span)?;
                let (it, et) = (Ty::int(index_size), Ty::int(elem_size));
                let lanes = width / index_size.max(elem_size);
                f.b.copy(tmp, dst, 64);
                let scale = konst(f, Ty::I64, scale);
                for i in 0..lanes {
                    // Only lanes whose mask element has its sign bit (or mask bit) set load.
                    let on = match (mask, writemask) {
                        (Some(mask), _) => {
                            let m = load_lane(f, mask, i, et);
                            is_neg(f, et, m)
                        }
                        (None, Some(wm)) => {
                            let at = konst(f, Ty::I64, i);
                            let b = bin(f, BinOp::LShr, Ty::I64, wm.bits, at);
                            let b = resize(f, b, Ty::I64, Ty::I8, false);
                            let one = konst(f, Ty::I8, 1);
                            bin(f, BinOp::And, Ty::I8, b, one)
                        }
                        _ => unreachable!(),
                    };
                    let idx = load_lane(f, index, i, it);
                    let idx = resize(f, idx, it, Ty::I64, true);
                    let off = bin(f, BinOp::Mul, Ty::I64, idx, scale);
                    let addr = bin(f, BinOp::Add, Ty::I64, base, off);
                    let old = load_lane(f, tmp, i, et);
                    let take = f.b.new_block();
                    let join = f.b.new_block();
                    let slot = f.b.alloca(elem_size, elem_size);
                    f.b.store(et, slot, old);
                    f.b.branch(on, take, join);
                    f.b.switch_to(take);
                    let p = ptr_of(f, addr);
                    let v = f.b.load(et, p);
                    f.b.store(et, slot, v);
                    f.b.jump(join);
                    f.b.switch_to(join);
                    let v = f.b.load(et, slot);
                    store_lane(f, tmp, i, et, v);
                }
                self.vec_store(f, cx, ops[0], tmp, lanes * elem_size, span)?;
                // The mask is cleared once every element has been gathered.
                if n == 3 {
                    let zeros = f.b.alloca(64, 16);
                    f.b.zero(zeros, 64);
                    self.vec_store(f, cx, ops[2], zeros, 64, span)?;
                } else if let Some(reg) = writemask.and_then(|w| w.reg) {
                    let z = konst(f, Ty::I64, 0);
                    f.b.store(Ty::I64, reg, z);
                }
            }
            _ => unreachable!("dispatched only for this group's operations"),
        }
        Ok(())
    }
}
