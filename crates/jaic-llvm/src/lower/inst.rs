//! Lowering instructions: arithmetic, conversions, loads, stores and inline copies.
use super::error::Error;
use super::state::FnState;
use super::{Backend, INLINE_COPY_MAX, R};
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::values::{BasicValue, BasicValueEnum, CallSiteValue, PointerValue, ValueKind};
use jaic::ir::{BinOp, Callee, CmpOp, ConvOp, Func, Inst, Ty, UnOp, Val};

impl<'ctx, 'p> Backend<'ctx, 'p> {
    pub(super) fn inst(&self, st: &mut FnState<'ctx>, func: &Func, inst: &Inst) -> R<()> {
        let b = &self.builder;
        match inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => {
                let v: BasicValueEnum = match ty {
                    Ty::Ptr => self
                        .ctx
                        .i64_type()
                        .const_int(*value, false)
                        .const_to_pointer(self.ptr_ty())
                        .into(),
                    Ty::F32 | Ty::F64 => return Err("integer constant of float type".into()),
                    _ => self.ll(*ty).into_int_type().const_int(*value, false).into(),
                };
                self.set(st, *dst, v);
            }
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                let v = match ty {
                    Ty::F32 => self.ctx.f32_type().const_float(*value),
                    _ => self.ctx.f64_type().const_float(*value),
                };
                self.set(st, *dst, v.into());
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b: rhs,
            } => {
                let v = self.bin(st, *op, *ty, *a, *rhs)?;
                // `!flag` on a comparison's result: negate the `i1`.
                if *op == BinOp::Xor
                    && *ty == Ty::I8
                    && let Some(flag) = st.flags[a.0 as usize]
                    && self.get_int(st, *rhs)?.get_zero_extended_constant() == Some(1)
                {
                    st.flags[dst.0 as usize] = Some(b.build_not(flag, "")?);
                }
                self.set(st, *dst, v);
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let v: BasicValueEnum = match op {
                    UnOp::Neg => b.build_int_neg(self.get_int(st, *a)?, "")?.into(),
                    UnOp::Not => b.build_not(self.get_int(st, *a)?, "")?.into(),
                    UnOp::FNeg => b
                        .build_float_neg(
                            self.coerce(self.get(st, *a)?, *ty)?.into_float_value(),
                            "",
                        )?
                        .into(),
                };
                let v = if *ty == Ty::Ptr {
                    self.coerce(v, Ty::Ptr)?
                } else {
                    v
                };
                self.set(st, *dst, v);
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b: rhs,
            } => {
                let (x, y) = (self.get(st, *a)?, self.get(st, *rhs)?);
                let r = if ty.is_float() {
                    let (x, y) = (
                        self.coerce(x, *ty)?.into_float_value(),
                        self.coerce(y, *ty)?.into_float_value(),
                    );
                    let pred = match op {
                        CmpOp::FEq => FloatPredicate::OEQ,
                        CmpOp::FNe => FloatPredicate::UNE,
                        CmpOp::FLt => FloatPredicate::OLT,
                        CmpOp::FLe => FloatPredicate::OLE,
                        CmpOp::FGt => FloatPredicate::OGT,
                        CmpOp::FGe => FloatPredicate::OGE,
                        _ => return Err("integer comparison on floats".into()),
                    };
                    b.build_float_compare(pred, x, y, "")?
                } else {
                    let pred = match op {
                        CmpOp::Eq => IntPredicate::EQ,
                        CmpOp::Ne => IntPredicate::NE,
                        CmpOp::SLt => IntPredicate::SLT,
                        CmpOp::SLe => IntPredicate::SLE,
                        CmpOp::SGt => IntPredicate::SGT,
                        CmpOp::SGe => IntPredicate::SGE,
                        CmpOp::ULt => IntPredicate::ULT,
                        CmpOp::ULe => IntPredicate::ULE,
                        CmpOp::UGt => IntPredicate::UGT,
                        CmpOp::UGe => IntPredicate::UGE,
                        _ => return Err("float comparison on integers".into()),
                    };
                    b.build_int_compare(pred, self.as_int(x)?, self.as_int(y)?, "")?
                };
                st.flags[dst.0 as usize] = Some(r);
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => {
                let v = self.conv(st, *op, *from, *to, *src)?;
                self.set(st, *dst, v);
            }
            Inst::SlotAddr {
                dst,
                slot,
            } => {
                let p = st.slots[slot.0 as usize];
                st.nonnull[dst.0 as usize] = true;
                self.set(st, *dst, p.into());
            }
            Inst::StackAlloc {
                dst,
                size,
                align,
            } => {
                let count = self.get(st, *size)?.into_int_value();
                let pointer =
                    self.builder
                        .build_array_alloca(self.ctx.i8_type(), count, "dynamic.frame")?;
                pointer
                    .as_instruction_value()
                    .unwrap()
                    .set_alignment(*align as u32)
                    .map_err(|e| Error(format!("dynamic stack alignment: {e:?}")))?;
                st.nonnull[dst.0 as usize] = true;
                self.set(st, *dst, pointer.into());
            }
            Inst::GlobalAddr {
                dst,
                global,
            } => {
                let p = self.global(global.0 as usize).as_pointer_value();
                st.nonnull[dst.0 as usize] = true;
                self.set(st, *dst, p.into());
            }
            Inst::FuncAddr {
                dst,
                func,
            } => {
                let p = self.func_ptr(*func)?;
                st.nonnull[dst.0 as usize] = true;
                self.set(st, *dst, p.into());
            }
            Inst::ForeignAddr {
                dst,
                foreign,
            } => {
                let p = self.foreign(foreign.0 as usize);
                st.nonnull[dst.0 as usize] = true;
                self.set(st, *dst, p.into());
            }
            Inst::Load {
                dst,
                ty,
                addr,
            } => {
                self.null_check(st, *addr, jaic::ir::NULL_READ)?;
                let p = self.get_ptr(st, *addr)?;
                let load = b.build_load(self.ll(*ty), p, "")?;
                if let Some(i) = load.as_instruction_value() {
                    let _ = i.set_alignment(ty.size() as u32);
                }
                if st.null_checks
                    && *ty == Ty::Ptr
                    && let Some(slot) = st.facts.tracked_slot(*addr)
                {
                    let facts = &mut st.facts;
                    facts.loaded_from[dst.0 as usize] = (slot as u32 + 1, facts.slot_version[slot]);
                }
                self.set(st, *dst, load);
            }
            Inst::Store {
                ty,
                addr,
                value,
            } => {
                self.null_check(st, *addr, jaic::ir::NULL_WRITE)?;
                let p = self.get_ptr(st, *addr)?;
                let v = self.coerce(self.get(st, *value)?, *ty)?;
                let store = b.build_store(p, v)?;
                let _ = store.set_alignment(ty.size() as u32);
                if st.null_checks
                    && let Some(slot) = st.facts.tracked_slot(*addr)
                {
                    let known = *ty == Ty::Ptr && st.known_nonnull(*value);
                    let facts = &mut st.facts;
                    facts.slot_version[slot] += 1;
                    facts.slot_checked[slot] = if known {
                        facts.epoch
                    } else {
                        0
                    };
                }
            }
            Inst::PtrAdd {
                dst,
                base,
                offset,
            } => {
                let base_val = *base;
                let base = self.get_ptr(st, *base)?;
                let off = self.get_int(st, *offset)?;
                let off = if off.get_type().get_bit_width() < 64 {
                    b.build_int_s_extend(off, self.ctx.i64_type(), "")?
                } else {
                    off
                };
                let p = self.gep(base, off)?;
                if st.nonnull[base_val.0 as usize] {
                    st.nonnull[dst.0 as usize] = true;
                } else if st.null_checks
                    && st.facts.constant[offset.0 as usize].is_some_and(|c| c < 4096)
                {
                    // A real address plus a field offset is not in the first page either.
                    st.facts.derived[dst.0 as usize] = base_val.0 + 1;
                }
                self.set(st, *dst, p.into());
            }
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                if *size > 0 {
                    self.null_check(st, *dst, jaic::ir::NULL_COPY)?;
                    self.null_check(st, *src, jaic::ir::NULL_COPY)?;
                }
                let (d, s) = (self.get_ptr(st, *dst)?, self.get_ptr(st, *src)?);
                if self.inline_small_copies && *size <= INLINE_COPY_MAX {
                    self.inline_copy(d, Some(s), *size)?;
                } else {
                    b.build_memcpy(d, 1, s, 1, self.ctx.i64_type().const_int(*size, false))?;
                }
            }
            Inst::Zero {
                dst,
                size,
            } => {
                if *size > 0 {
                    self.null_check(st, *dst, jaic::ir::NULL_FILL)?;
                }
                let d = self.get_ptr(st, *dst)?;
                if self.inline_small_copies && *size <= INLINE_COPY_MAX {
                    self.inline_copy(d, None, *size)?;
                } else {
                    b.build_memset(
                        d,
                        1,
                        self.ctx.i8_type().const_zero(),
                        self.ctx.i64_type().const_int(*size, false),
                    )?;
                }
            }
            Inst::Call(call) => {
                let jaic::ir::CallInst {
                    results,
                    callee,
                    args,
                } = &**call;
                let argv = args
                    .iter()
                    .map(|&a| self.get(st, a))
                    .collect::<R<Vec<_>>>()?;
                let (ptr, sig) = match callee {
                    Callee::Func(id) => {
                        let f = self.program.func(*id).ok_or_else(|| {
                            Error(format!(
                                "call to '{}' which was never lowered",
                                self.program.func_names[id.0 as usize]
                            ))
                        })?;
                        (self.func_ptr(*id)?, &f.sig)
                    }
                    Callee::Foreign(id) => (
                        self.foreign(id.0 as usize),
                        &self.program.foreigns[id.0 as usize].sig,
                    ),
                    Callee::Indirect(target, sig) => (self.get_ptr(st, *target)?, &**sig),
                };
                let out = self.call(st, ptr, sig, &argv)?;
                for (r, v) in results.iter().zip(out) {
                    self.set(st, *r, v);
                }
            }
            Inst::Intrinsic(call) => {
                let jaic::ir::IntrinsicInst {
                    results,
                    op,
                    args,
                } = &**call;
                let argv = args
                    .iter()
                    .map(|&a| self.get(st, a))
                    .collect::<R<Vec<_>>>()?;
                let want: Vec<Ty> = results.iter().map(|r| func.vals[r.0 as usize]).collect();
                let out = self.intrinsic(st, *op, &argv, &want)?;
                for (r, v) in results.iter().zip(out) {
                    self.set(st, *r, v);
                }
            }
            Inst::Loc {
                line,
                col,
                scope,
                file,
            } => {
                st.loc = (*file, *line);
                if let (Some(debug), Some(dbg)) = (&self.debug, &mut st.dbg) {
                    dbg.loc(debug, func, &self.builder, *line, *col, *scope);
                }
            }
        }
        Ok(())
    }

    /// `size` bytes from `src` to `dst` (zeros when there is no source) as the widest unaligned
    /// loads and stores that fit.
    fn inline_copy(
        &self,
        dst: PointerValue<'ctx>,
        src: Option<PointerValue<'ctx>>,
        size: u64,
    ) -> R<()> {
        let b = &self.builder;
        let mut at = 0;
        while at < size {
            let width = [8u64, 4, 2, 1]
                .into_iter()
                .find(|w| *w <= size - at)
                .expect("a width fits");
            let ty = match width {
                8 => self.ctx.i64_type(),
                4 => self.ctx.i32_type(),
                2 => self.ctx.i16_type(),
                _ => self.ctx.i8_type(),
            };
            let to = self.gep_const(dst, at)?;
            let value = match src {
                Some(src) => {
                    let load = b.build_load(ty, self.gep_const(src, at)?, "")?;
                    if let Some(i) = load.as_instruction_value() {
                        let _ = i.set_alignment(1);
                    }
                    load
                }
                None => ty.const_zero().into(),
            };
            let store = b.build_store(to, value)?;
            let _ = store.set_alignment(1);
            at += width;
        }
        Ok(())
    }

    fn bin(
        &self,
        st: &FnState<'ctx>,
        op: BinOp,
        ty: Ty,
        a: Val,
        rhs: Val,
    ) -> R<BasicValueEnum<'ctx>> {
        let b = &self.builder;
        if ty.is_float() {
            let x = self.coerce(self.get(st, a)?, ty)?.into_float_value();
            let y = self.coerce(self.get(st, rhs)?, ty)?.into_float_value();
            return Ok(match op {
                BinOp::FAdd => b.build_float_add(x, y, "")?,
                BinOp::FSub => b.build_float_sub(x, y, "")?,
                BinOp::FMul => b.build_float_mul(x, y, "")?,
                BinOp::FDiv => b.build_float_div(x, y, "")?,
                _ => return Err("integer operation on floats".into()),
            }
            .into());
        }
        let x = self.get_int(st, a)?;
        let y = self.get_int(st, rhs)?;
        let it = x.get_type();
        let bits = it.get_bit_width();
        let konst = |v: u64| it.const_int(v, false);
        let r = match op {
            BinOp::Add => b.build_int_add(x, y, "")?,
            BinOp::Sub => b.build_int_sub(x, y, "")?,
            BinOp::Mul => b.build_int_mul(x, y, "")?,
            BinOp::And => b.build_and(x, y, "")?,
            BinOp::Or => b.build_or(x, y, "")?,
            BinOp::Xor => b.build_xor(x, y, "")?,
            BinOp::UDiv | BinOp::URem => {
                // A constant divisor needs no check (the common `x % 2`).
                if y.get_zero_extended_constant().is_none_or(|c| c == 0) {
                    let zero = b.build_int_compare(IntPredicate::EQ, y, konst(0), "")?;
                    let none = self.ctx.i64_type().const_zero();
                    self.trap_if(st, zero, jaic::ir::CheckReason::DivideByZero, none, none)?;
                }
                if op == BinOp::UDiv {
                    b.build_int_unsigned_div(x, y, "")?
                } else {
                    b.build_int_unsigned_rem(x, y, "")?
                }
            }
            BinOp::SDiv | BinOp::SRem => {
                let constant = y.get_sign_extended_constant();
                if constant.is_none_or(|c| c == 0) {
                    let zero = b.build_int_compare(IntPredicate::EQ, y, konst(0), "")?;
                    let none = self.ctx.i64_type().const_zero();
                    self.trap_if(st, zero, jaic::ir::CheckReason::DivideByZero, none, none)?;
                }
                if constant.is_some_and(|c| c != 0 && c != -1) {
                    // Neither a zero divisor nor the overflowing `INT_MIN / -1`.
                    if op == BinOp::SDiv {
                        b.build_int_signed_div(x, y, "")?
                    } else {
                        b.build_int_signed_rem(x, y, "")?
                    }
                } else {
                    // x / -1 overflows for INT_MIN (undefined in LLVM); the IR wraps.
                    let minus_one =
                        b.build_int_compare(IntPredicate::EQ, y, it.const_all_ones(), "")?;
                    let safe_y = b.build_select(minus_one, konst(1), y, "")?.into_int_value();
                    if op == BinOp::SDiv {
                        let q = b.build_int_signed_div(x, safe_y, "")?;
                        let neg = b.build_int_neg(x, "")?;
                        b.build_select(minus_one, neg, q, "")?.into_int_value()
                    } else {
                        let r = b.build_int_signed_rem(x, safe_y, "")?;
                        b.build_select(minus_one, konst(0), r, "")?.into_int_value()
                    }
                }
            }
            BinOp::Shl | BinOp::LShr => {
                let shift = |x, y| {
                    if op == BinOp::Shl {
                        b.build_left_shift(x, y, "")
                    } else {
                        b.build_right_shift(x, y, false, "")
                    }
                };
                match y.get_zero_extended_constant() {
                    Some(c) if c < bits as u64 => shift(x, y)?,
                    Some(_) => konst(0),
                    None => {
                        let out =
                            b.build_int_compare(IntPredicate::UGE, y, konst(bits as u64), "")?;
                        let r = shift(x, y)?;
                        b.build_select(out, konst(0), r, "")?.into_int_value()
                    }
                }
            }
            BinOp::AShr => {
                let y = match y.get_zero_extended_constant() {
                    Some(c) if c < bits as u64 => y,
                    Some(_) => konst(bits as u64 - 1),
                    None => {
                        let out =
                            b.build_int_compare(IntPredicate::UGE, y, konst(bits as u64), "")?;
                        b.build_select(out, konst(bits as u64 - 1), y, "")?
                            .into_int_value()
                    }
                };
                b.build_right_shift(x, y, true, "")?
            }
            BinOp::Rotl | BinOp::Rotr => {
                let name = if op == BinOp::Rotl {
                    "llvm.fshl"
                } else {
                    "llvm.fshr"
                };
                self.call_intrinsic(name, &[it.into()], &[x.into(), x.into(), y.into()])?
                    .ok_or("rotate produced no value")?
                    .into_int_value()
            }
            BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
                return Err("float operation on integers".into());
            }
        };
        if ty == Ty::Ptr {
            self.coerce(r.into(), Ty::Ptr)
        } else {
            Ok(r.into())
        }
    }

    fn conv(
        &self,
        st: &FnState<'ctx>,
        op: ConvOp,
        from: Ty,
        to: Ty,
        src: Val,
    ) -> R<BasicValueEnum<'ctx>> {
        let b = &self.builder;
        let v = self.coerce(self.get(st, src)?, from)?;
        let to_ll = self.ll(to);
        Ok(match op {
            ConvOp::Trunc => {
                let i = self.as_int(v)?;
                b.build_int_truncate_or_bit_cast(i, to_ll.into_int_type(), "")?
                    .into()
            }
            ConvOp::ZExt => b
                .build_int_z_extend_or_bit_cast(self.as_int(v)?, to_ll.into_int_type(), "")?
                .into(),
            ConvOp::SExt => b
                .build_int_s_extend_or_bit_cast(self.as_int(v)?, to_ll.into_int_type(), "")?
                .into(),
            ConvOp::FToS | ConvOp::FToU => {
                let name = if op == ConvOp::FToS {
                    "llvm.fptosi.sat"
                } else {
                    "llvm.fptoui.sat"
                };
                // Saturating, matching the interpreter's Rust `as` casts.
                self.call_intrinsic(name, &[to_ll, v.get_type()], &[v.into()])?
                    .ok_or("float conversion produced no value")?
            }
            ConvOp::SToF => b
                .build_signed_int_to_float(self.as_int(v)?, to_ll.into_float_type(), "")?
                .into(),
            ConvOp::UToF => b
                .build_unsigned_int_to_float(self.as_int(v)?, to_ll.into_float_type(), "")?
                .into(),
            ConvOp::FExt => b
                .build_float_ext(v.into_float_value(), to_ll.into_float_type(), "")?
                .into(),
            ConvOp::FTrunc => b
                .build_float_trunc(v.into_float_value(), to_ll.into_float_type(), "")?
                .into(),
            ConvOp::Bitcast => self.coerce(v, to)?,
        })
    }
}

pub(super) fn basic<'ctx>(call: CallSiteValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
    match call.try_as_basic_value() {
        ValueKind::Basic(v) => Some(v),
        ValueKind::Instruction(_) => None,
    }
}
