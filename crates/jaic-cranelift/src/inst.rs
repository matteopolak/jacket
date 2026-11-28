//! Lowering instructions: constants, arithmetic, comparisons, conversions, addresses and memory.
use cranelift_codegen::ir::{
    self, InstBuilder, MemFlagsData, Value,
    condcodes::{FloatCC, IntCC},
    immediates::Ieee32,
    types,
};
use jaic::ir::{BinOp, CheckReason, CmpOp, ConvOp, Inst, Ty, UnOp, Val};

use super::R;
use super::clif_ty;
use super::data::ForeignSym;
use super::function::Cx;

impl Cx<'_, '_> {
    pub(crate) fn inst(&mut self, inst: &Inst) -> R<()> {
        match inst {
            Inst::IConst {
                dst,
                ty,
                value,
            } => {
                if ty.is_float() {
                    return Err("integer constant of float type".into());
                }

                let v = self.iconst(clif_ty(*ty)?, *value);

                self.consts[dst.0 as usize] = Some(*value);
                self.set(*dst, v);
            }
            Inst::FConst {
                dst,
                ty,
                value,
            } => {
                let v = match ty {
                    Ty::F32 => self.b.ins().f32const(Ieee32::with_float(*value as f32)),
                    _ => self.b.ins().f64const(*value),
                };

                self.set(*dst, v);
            }
            Inst::Bin {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let v = self.bin(*op, *ty, *a, *b)?;

                self.set(*dst, v);
            }
            Inst::Un {
                dst,
                op,
                ty,
                a,
            } => {
                let x = self.get_as(*a, *ty)?;
                let v = match op {
                    UnOp::Neg => self.b.ins().ineg(x),
                    UnOp::Not => self.b.ins().bnot(x),
                    UnOp::FNeg => self.b.ins().fneg(x),
                };

                self.set(*dst, v);
            }
            Inst::Cmp {
                dst,
                op,
                ty,
                a,
                b,
            } => {
                let (x, y) = (self.get_as(*a, *ty)?, self.get_as(*b, *ty)?);
                let v = if ty.is_float() {
                    let cc = match op {
                        CmpOp::FEq => FloatCC::Equal,
                        CmpOp::FNe => FloatCC::NotEqual,
                        CmpOp::FLt => FloatCC::LessThan,
                        CmpOp::FLe => FloatCC::LessThanOrEqual,
                        CmpOp::FGt => FloatCC::GreaterThan,
                        CmpOp::FGe => FloatCC::GreaterThanOrEqual,
                        _ => return Err("integer comparison on floats".into()),
                    };
                    self.b.ins().fcmp(cc, x, y)
                } else {
                    self.b.ins().icmp(int_cc(*op)?, x, y)
                };

                self.set(*dst, v);
            }
            Inst::Conv {
                dst,
                op,
                from,
                to,
                src,
            } => {
                let v = self.conv(*op, *from, *to, *src)?;

                self.set(*dst, v);
            }
            Inst::SlotAddr {
                dst,
                slot,
            } => {
                let v = self.slot_addr(slot.0 as usize);

                self.checks.address(*dst);
                self.set(*dst, v);
            }
            Inst::StackAlloc {
                ..
            } => {
                return Err(self.unsupported(
                    "runtime-sized stack allocations (variadic arguments mixing spread arrays and values)",
                ));
            }
            Inst::GlobalAddr {
                dst,
                global,
            } => {
                let id = self.sh.symbols.globals[global.0 as usize];
                let v = self.data_addr(id, true);

                self.checks.address(*dst);
                self.set(*dst, v);
            }
            Inst::FuncAddr {
                dst,
                func,
            } => {
                let id = self.program_func(*func)?;
                let v = self.func_addr(id);

                self.checks.address(*dst);
                self.set(*dst, v);
            }
            Inst::ForeignAddr {
                dst,
                foreign,
            } => {
                let v = match &self.sh.symbols.foreigns[foreign.0 as usize] {
                    &ForeignSym::Func(id) => self.func_addr(id),
                    &ForeignSym::Data(id) => self.data_addr(id, false),
                    ForeignSym::Llvm(name) => {
                        return Err(
                            self.unsupported(format!("the address of the intrinsic `llvm.{name}`"))
                        );
                    }
                };

                self.checks.address(*dst);
                self.set(*dst, v);
            }
            Inst::PtrAdd {
                dst,
                base,
                offset,
                ..
            } => {
                let p = self.get_i64(*base)?;
                let off = self.get(*offset)?;
                let off = if self.b.func.dfg.value_type(off).bits() < 64 {
                    self.b.ins().sextend(types::I64, off)
                } else {
                    self.coerce(off, types::I64)
                };
                let v = self.b.ins().iadd(p, off);

                self.checks.ptr_add(*dst, *base, *offset);
                self.set(*dst, v);
            }
            Inst::Load {
                dst,
                ty,
                addr,
            } => {
                self.null_check(*addr, jaic::ir::NULL_READ)?;
                let p = self.get_i64(*addr)?;
                let v = self.b.ins().load(clif_ty(*ty)?, MemFlagsData::new(), p, 0);

                self.checks.load(*dst, *ty, *addr);
                self.set(*dst, v);
            }
            Inst::Store {
                ty,
                addr,
                value,
            } => {
                self.null_check(*addr, jaic::ir::NULL_WRITE)?;
                let p = self.get_i64(*addr)?;
                let v = self.get_as(*value, *ty)?;

                self.b.ins().store(MemFlagsData::new(), v, p, 0);
                self.checks.store(*ty, *addr, *value);
            }
            Inst::Copy {
                dst,
                src,
                size,
            } => {
                if *size > 0 {
                    self.null_check(*dst, jaic::ir::NULL_COPY)?;
                    self.null_check(*src, jaic::ir::NULL_COPY)?;
                }

                let (d, s) = (self.get_i64(*dst)?, self.get_i64(*src)?);

                self.copy(d, s, *size);
            }
            Inst::Zero {
                dst,
                size,
            } => {
                if *size > 0 {
                    self.null_check(*dst, jaic::ir::NULL_FILL)?;
                }

                let d = self.get_i64(*dst)?;

                self.zero(d, *size);
            }
            Inst::Call(call) => self.call_inst(&call.results, &call.callee, &call.args)?,
            Inst::Intrinsic(call) => {
                let want: Vec<Ty> = call
                    .results
                    .iter()
                    .map(|r| self.func.vals[r.0 as usize])
                    .collect();
                let out = self.intrinsic(call.op, &call.args, &want)?;

                for (r, v) in call.results.iter().zip(out) {
                    self.set(*r, v);
                }
            }
            Inst::Loc {
                line,
                file,
                ..
            } => {
                self.loc = (*file, *line);
                self.b.set_srcloc(ir::SourceLoc::new(*line));
            }
        }

        Ok(())
    }

    fn bin(&mut self, op: BinOp, ty: Ty, a: Val, b: Val) -> R<Value> {
        let x = self.get_as(a, ty)?;
        let y = self.get_as(b, ty)?;

        if ty.is_float() {
            return Ok(match op {
                BinOp::FAdd => self.b.ins().fadd(x, y),
                BinOp::FSub => self.b.ins().fsub(x, y),
                BinOp::FMul => self.b.ins().fmul(x, y),
                BinOp::FDiv => self.b.ins().fdiv(x, y),
                _ => return Err("integer operation on floats".into()),
            });
        }

        let it = clif_ty(ty)?;
        let bits = u64::from(it.bits());
        let konst = self.consts[b.0 as usize].map(|c| {
            if bits >= 64 {
                c
            } else {
                c & ((1 << bits) - 1)
            }
        });

        Ok(match op {
            BinOp::Add => self.b.ins().iadd(x, y),
            BinOp::Sub => self.b.ins().isub(x, y),
            BinOp::Mul => self.b.ins().imul(x, y),
            BinOp::And => self.b.ins().band(x, y),
            BinOp::Or => self.b.ins().bor(x, y),
            BinOp::Xor => self.b.ins().bxor(x, y),
            BinOp::UDiv | BinOp::URem => {
                // A constant divisor needs no check (the common `x % 2`).
                if konst.is_none_or(|c| c == 0) {
                    let zero = self.b.ins().icmp_imm_s(IntCC::Equal, y, 0);
                    let none = self.b.ins().iconst(types::I64, 0);

                    self.trap_if(zero, CheckReason::DivideByZero, none, none)?;
                }

                if op == BinOp::UDiv {
                    self.b.ins().udiv(x, y)
                } else {
                    self.b.ins().urem(x, y)
                }
            }
            BinOp::SDiv | BinOp::SRem => {
                let signed = konst.map(|c| sign_extend(c, bits));

                if signed.is_none_or(|c| c == 0) {
                    let zero = self.b.ins().icmp_imm_s(IntCC::Equal, y, 0);
                    let none = self.b.ins().iconst(types::I64, 0);

                    self.trap_if(zero, CheckReason::DivideByZero, none, none)?;
                }

                if signed.is_some_and(|c| c != 0 && c != -1) {
                    if op == BinOp::SDiv {
                        self.b.ins().sdiv(x, y)
                    } else {
                        self.b.ins().srem(x, y)
                    }
                } else {
                    // x / -1 overflows (and faults) for the most negative x; the IR wraps.
                    let all_ones = self.iconst(it, u64::MAX);
                    let minus_one = self.b.ins().icmp(IntCC::Equal, y, all_ones);
                    let one = self.b.ins().iconst(it, 1);
                    let safe = self.b.ins().select(minus_one, one, y);

                    if op == BinOp::SDiv {
                        let q = self.b.ins().sdiv(x, safe);
                        let neg = self.b.ins().ineg(x);

                        self.b.ins().select(minus_one, neg, q)
                    } else {
                        let r = self.b.ins().srem(x, safe);
                        let zero = self.b.ins().iconst(it, 0);

                        self.b.ins().select(minus_one, zero, r)
                    }
                }
            }
            BinOp::Shl | BinOp::LShr => {
                // A shift by the width or more gives 0 (Cranelift takes the amount modulo it).
                let shift = |cx: &mut Self, x, y| {
                    if op == BinOp::Shl {
                        cx.b.ins().ishl(x, y)
                    } else {
                        cx.b.ins().ushr(x, y)
                    }
                };

                match konst {
                    Some(c) if c < bits => shift(self, x, y),
                    Some(_) => self.b.ins().iconst(it, 0),
                    None => {
                        let out = self.b.ins().icmp_imm_s(
                            IntCC::UnsignedGreaterThanOrEqual,
                            y,
                            bits as i64,
                        );
                        let r = shift(self, x, y);
                        let zero = self.b.ins().iconst(it, 0);

                        self.b.ins().select(out, zero, r)
                    }
                }
            }
            BinOp::AShr => match konst {
                Some(c) if c < bits => self.b.ins().sshr(x, y),
                Some(_) => self.b.ins().sshr_imm_u(x, bits as i64 - 1),
                None => {
                    let out =
                        self.b
                            .ins()
                            .icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, y, bits as i64);
                    let most = self.b.ins().iconst(it, bits as i64 - 1);
                    let amount = self.b.ins().select(out, most, y);

                    self.b.ins().sshr(x, amount)
                }
            },
            BinOp::Rotl => self.b.ins().rotl(x, y),
            BinOp::Rotr => self.b.ins().rotr(x, y),
            BinOp::FAdd | BinOp::FSub | BinOp::FMul | BinOp::FDiv => {
                return Err("float operation on integers".into());
            }
        })
    }

    fn conv(&mut self, op: ConvOp, from: Ty, to: Ty, src: Val) -> R<Value> {
        let v = self.get_as(src, from)?;
        let (have, want) = (clif_ty(from)?, clif_ty(to)?);

        Ok(match op {
            ConvOp::Trunc if have.bits() > want.bits() => self.b.ins().ireduce(want, v),
            ConvOp::ZExt if have.bits() < want.bits() => self.b.ins().uextend(want, v),
            ConvOp::SExt if have.bits() < want.bits() => self.b.ins().sextend(want, v),
            ConvOp::Trunc | ConvOp::ZExt | ConvOp::SExt | ConvOp::Bitcast => self.coerce(v, want),
            // Saturating, matching the interpreter's Rust `as` casts.
            ConvOp::FToS | ConvOp::FToU => self.float_to_int(op == ConvOp::FToS, v, want),
            ConvOp::SToF | ConvOp::UToF => {
                // Through 64 bits: the instruction takes 32- and 64-bit integers.
                let v = if have.bits() < 32 {
                    if op == ConvOp::SToF {
                        self.b.ins().sextend(types::I64, v)
                    } else {
                        self.b.ins().uextend(types::I64, v)
                    }
                } else {
                    v
                };

                if op == ConvOp::SToF {
                    self.b.ins().fcvt_from_sint(want, v)
                } else {
                    self.b.ins().fcvt_from_uint(want, v)
                }
            }
            ConvOp::FExt if have != want => self.b.ins().fpromote(want, v),
            ConvOp::FTrunc if have != want => self.b.ins().fdemote(want, v),
            ConvOp::FExt | ConvOp::FTrunc => v,
        })
    }

    /// A saturating float to integer conversion; narrow results saturate to their own range.
    fn float_to_int(&mut self, signed: bool, v: Value, want: ir::Type) -> Value {
        if want.bits() >= 32 {
            return if signed {
                self.b.ins().fcvt_to_sint_sat(want, v)
            } else {
                self.b.ins().fcvt_to_uint_sat(want, v)
            };
        }

        let bits = want.bits();
        let wide = if signed {
            let r = self.b.ins().fcvt_to_sint_sat(types::I32, v);
            let max = self.b.ins().iconst(types::I32, (1i64 << (bits - 1)) - 1);
            let min = self.b.ins().iconst(types::I32, -(1i64 << (bits - 1)));
            let r = self.b.ins().smin(r, max);

            self.b.ins().smax(r, min)
        } else {
            let r = self.b.ins().fcvt_to_uint_sat(types::I32, v);
            let max = self.b.ins().iconst(types::I32, (1i64 << bits) - 1);

            self.b.ins().umin(r, max)
        };

        self.b.ins().ireduce(want, wide)
    }
}

pub(crate) fn int_cc(op: CmpOp) -> R<IntCC> {
    Ok(match op {
        CmpOp::Eq => IntCC::Equal,
        CmpOp::Ne => IntCC::NotEqual,
        CmpOp::SLt => IntCC::SignedLessThan,
        CmpOp::SLe => IntCC::SignedLessThanOrEqual,
        CmpOp::SGt => IntCC::SignedGreaterThan,
        CmpOp::SGe => IntCC::SignedGreaterThanOrEqual,
        CmpOp::ULt => IntCC::UnsignedLessThan,
        CmpOp::ULe => IntCC::UnsignedLessThanOrEqual,
        CmpOp::UGt => IntCC::UnsignedGreaterThan,
        CmpOp::UGe => IntCC::UnsignedGreaterThanOrEqual,
        _ => return Err("float comparison on integers".into()),
    })
}

/// `value`'s low `bits` bits as a signed number.
fn sign_extend(value: u64, bits: u64) -> i64 {
    if bits >= 64 {
        return value as i64;
    }

    let shift = 64 - bits;
    ((value << shift) as i64) >> shift
}
