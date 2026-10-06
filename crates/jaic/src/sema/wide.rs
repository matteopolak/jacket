//! `Long_Double` (the `Jaic_Extensions` module's name for `#jaic_type long_double`) on targets
//! where C's `long double` is wider than `float64`.
//!
//! Such a value is a memory-class type like a small struct: operands are addresses of 16-byte
//! values, and every operation is one `ir::Intrinsic::Wide`, which the LLVM backend lowers to
//! native `x86_fp80`/`fp128` arithmetic and the interpreter runs in software
//! (`crate::wide_float`). Constants are encoded at compile time into `Value::Bytes`.
use super::lower::{FnCtx, Operand};
use super::value::Aggregate;
use super::*;
use crate::ast::BinOp;
use crate::ir::{CmpOp, ConvOp, Intrinsic, Ty, Val, WideArith, WideFloat, WideOp};
use crate::types::TypeKind;
use crate::wide_float;

impl Compiler {
    /// The format of `ty` when it is a wide `long double`.
    pub(super) fn wide_float(&self, ty: TypeId) -> Option<WideFloat> {
        self.types.wide_float(ty)
    }

    /// Numbers (integer or float types) that convert to and from a wide float.
    fn wide_peer(&self, ty: TypeId) -> bool {
        let r = self.types.repr(ty);
        self.types.is_integer(r) || self.types.is_float(r)
    }

    /// An integer or float constant as a wide constant, when it is one.
    pub(super) fn wide_const(fmt: WideFloat, value: &Value) -> Option<Value> {
        let bytes = match value {
            Value::Int(i) => match i64::try_from(*i) {
                Ok(v) => wide_float::from_i64(fmt, v),
                Err(_) => wide_float::from_u64(fmt, u64::try_from(*i).ok()?),
            },
            Value::Float(x) => wide_float::from_f64(fmt, *x),
            _ => return None,
        };
        Some(Value::Bytes(Rc::new(Aggregate {
            bytes: bytes.to_vec(),
            relocs: Vec::new(),
        })))
    }

    /// The `float64` value of a wide constant (`cast(float64)` folds through this).
    fn wide_const_value(fmt: WideFloat, value: &Value) -> Option<f64> {
        let Value::Bytes(agg) = value else {
            return None;
        };
        let bytes: wide_float::Bytes = agg.bytes.as_slice().try_into().ok()?;
        Some(wide_float::to_f64(fmt, &bytes))
    }

    /// A fresh 16-byte slot for a result.
    fn wide_slot(f: &mut FnCtx) -> Val {
        f.b.alloca(16, 16)
    }

    /// `cast(to) op` (and implicit widening) when either side is a wide float. `None` when
    /// neither is.
    pub(super) fn wide_cast(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Option<Operand>> {
        let from = op.ty();
        let (from_wide, to_wide) = (self.wide_float(from), self.wide_float(to));
        if from_wide.is_none() && to_wide.is_none() {
            return Ok(None);
        }
        if let Operand::Const {
            value, ..
        } = &op
        {
            let folded = match (from_wide, to_wide) {
                (None, Some(fmt)) => Self::wide_const(fmt, value),
                (Some(a), Some(b)) if a == b => Some(value.clone()),
                (Some(fmt), None) if self.types.is_float(to) => Self::wide_const_value(fmt, value)
                    .map(|x| {
                        Value::Float(if self.types.repr(to) == TypeId::F32 {
                            x as f32 as f64
                        } else {
                            x
                        })
                    }),
                _ => None,
            };
            if let Some(value) = folded {
                return Ok(Some(Operand::Const {
                    ty: to,
                    value,
                    untyped: false,
                }));
            }
        }
        match (from_wide, to_wide) {
            (Some(a), Some(b)) if a == b => {
                let (_, addr) = self.rvalue(f, op, span)?;
                Ok(Some(Operand::Value {
                    ty: to,
                    val: addr,
                }))
            }
            (None, Some(fmt)) if self.wide_peer(from) => {
                let op = self.settle_untyped(op, None);
                let from = op.ty();
                let (_, v) = self.rvalue(f, op, span)?;
                let dst = Self::wide_slot(f);
                let (wop, v) = self.wide_source(f, from, v);
                f.b.intrinsic(Intrinsic::Wide(wop, fmt), vec![dst, v], &[]);
                Ok(Some(Operand::Value {
                    ty: to,
                    val: dst,
                }))
            }
            (Some(fmt), None) if self.wide_peer(to) => {
                let (_, addr) = self.rvalue(f, op, span)?;
                let val = self.wide_to_scalar(f, fmt, addr, to);
                Ok(Some(Operand::Value {
                    ty: to,
                    val,
                }))
            }
            // Anything else (`cast,force` to bytes...) is the general cast's business.
            _ => Ok(None),
        }
    }

    /// The conversion that brings scalar `v` of type `from` into a wide float, and its operand
    /// widened to what that conversion takes.
    fn wide_source(&mut self, f: &mut FnCtx, from: TypeId, v: Val) -> (WideOp, Val) {
        let r = self.types.repr(from);
        if r == TypeId::F32 {
            return (WideOp::FromF32, v);
        }
        if self.types.is_float(r) {
            return (WideOp::FromF64, v);
        }
        let (bits, signed) = self.types.int_info(r).unwrap_or((64, true));
        let t = Ty::int(bits as u64 / 8);
        let wide = match (t == Ty::I64, signed) {
            (true, _) => v,
            (false, true) => f.b.conv(ConvOp::SExt, t, Ty::I64, v),
            (false, false) => f.b.conv(ConvOp::ZExt, t, Ty::I64, v),
        };
        let op = if signed {
            WideOp::FromS64
        } else {
            WideOp::FromU64
        };
        (op, wide)
    }

    fn wide_to_scalar(&mut self, f: &mut FnCtx, fmt: WideFloat, addr: Val, to: TypeId) -> Val {
        let r = self.types.repr(to);
        if r == TypeId::F32 {
            return f
                .b
                .intrinsic(Intrinsic::Wide(WideOp::ToF32, fmt), vec![addr], &[Ty::F32])[0];
        }
        if self.types.is_float(r) {
            return f
                .b
                .intrinsic(Intrinsic::Wide(WideOp::ToF64, fmt), vec![addr], &[Ty::F64])[0];
        }
        let (bits, signed) = self.types.int_info(r).unwrap_or((64, true));
        let op = if signed {
            WideOp::ToS64
        } else {
            WideOp::ToU64
        };
        let v =
            f.b.intrinsic(Intrinsic::Wide(op, fmt), vec![addr], &[Ty::I64])[0];
        let t = Ty::int(bits as u64 / 8);
        if t == Ty::I64 {
            v
        } else {
            f.b.conv(ConvOp::Trunc, Ty::I64, t, v)
        }
    }

    /// Implicit conversion cost into a wide float: literals, `float32`/`float64`, and integers
    /// of up to 32 bits. These are `float64`'s rules, so code written for one target still
    /// compiles where `Long_Double` is `float64`.
    pub(super) fn wide_implicit_cost(
        &self,
        from: TypeId,
        untyped: bool,
        to: TypeId,
    ) -> Option<u32> {
        let (from_wide, to_wide) = (self.wide_float(from), self.wide_float(to));
        match (from_wide, to_wide) {
            (Some(a), Some(b)) if a == b && self.types.repr(from) == self.types.repr(to) => {
                Some(convert::EXACT)
            }
            (None, Some(_)) if untyped && self.wide_peer(from) => Some(convert::LITERAL + 1),
            (None, Some(_)) if self.types.is_float(from) => Some(convert::WIDEN),
            (None, Some(_))
                if self
                    .types
                    .int_info(from)
                    .is_some_and(|(bits, _)| bits <= 32) =>
            {
                Some(convert::INT_TO_FLOAT)
            }
            _ => None,
        }
    }

    /// `a op b` on two wide operands (already converted to the wide type `ty`).
    pub(super) fn wide_binary(
        &mut self,
        f: &mut FnCtx,
        ty: TypeId,
        op: BinOp,
        x: Val,
        y: Val,
        span: Span,
    ) -> Result<Operand> {
        let fmt = self.wide_float(ty).expect("a wide float type");
        let cmp = match op {
            BinOp::Eq => Some(CmpOp::FEq),
            BinOp::Ne => Some(CmpOp::FNe),
            BinOp::Lt => Some(CmpOp::FLt),
            BinOp::Le => Some(CmpOp::FLe),
            BinOp::Gt => Some(CmpOp::FGt),
            BinOp::Ge => Some(CmpOp::FGe),
            _ => None,
        };
        if let Some(cmp) = cmp {
            let r = f.b.intrinsic(
                Intrinsic::Wide(WideOp::Cmp(cmp), fmt),
                vec![x, y],
                &[Ty::I8],
            );
            return Ok(Operand::Value {
                ty: TypeId::BOOL,
                val: r[0],
            });
        }
        let kind = match op {
            BinOp::Add => WideArith::Add,
            BinOp::Sub => WideArith::Sub,
            BinOp::Mul => WideArith::Mul,
            BinOp::Div => WideArith::Div,
            _ => {
                return err(
                    span,
                    format!(
                        "operator {} is not defined for {}",
                        super::calls::binop_text(op),
                        self.types.name(ty)
                    ),
                );
            }
        };
        let dst = Self::wide_slot(f);
        f.b.intrinsic(
            Intrinsic::Wide(WideOp::Arith(kind), fmt),
            vec![dst, x, y],
            &[],
        );
        Ok(Operand::Value {
            ty,
            val: dst,
        })
    }

    pub(super) fn wide_negate(
        &mut self,
        f: &mut FnCtx,
        fmt: WideFloat,
        ty: TypeId,
        x: Val,
    ) -> Operand {
        let dst = Self::wide_slot(f);
        f.b.intrinsic(Intrinsic::Wide(WideOp::Neg, fmt), vec![dst, x], &[]);
        Operand::Value {
            ty,
            val: dst,
        }
    }

    /// The memory-only ABI class of a wide float, for `AggLayout`s.
    pub(super) fn wide_abi_class(&self, ty: TypeId) -> Option<Ty> {
        match self.types.kind(self.types.repr(ty)) {
            TypeKind::WideFloat(fmt) => Some(Ty::wide(*fmt)),
            _ => None,
        }
    }
}

impl Compiler {
    /// Fold `a op b` when both are constants and one is wide: exactly, in the wide format.
    pub(super) fn wide_fold(
        &self,
        op: BinOp,
        (lt, lv): (TypeId, &Value),
        (rt, rv): (TypeId, &Value),
    ) -> Option<Operand> {
        let ty = if self.wide_float(lt).is_some() {
            lt
        } else {
            rt
        };
        let fmt = self.wide_float(ty)?;
        let bytes = |v: &Value| -> Option<wide_float::Bytes> {
            let v = match v {
                Value::Bytes(_) => v.clone(),
                other => Self::wide_const(fmt, other)?,
            };
            let Value::Bytes(agg) = v else {
                return None;
            };
            agg.bytes.as_slice().try_into().ok()
        };
        let (a, b) = (bytes(lv)?, bytes(rv)?);
        let arith = |kind| wide_float::arith(fmt, kind, &a, &b);
        let result = match op {
            BinOp::Add => arith(WideArith::Add),
            BinOp::Sub => arith(WideArith::Sub),
            BinOp::Mul => arith(WideArith::Mul),
            BinOp::Div => arith(WideArith::Div),
            _ => {
                use std::cmp::Ordering::*;
                let order = wide_float::compare(fmt, &a, &b);
                let yes = match op {
                    BinOp::Eq => order == Some(Equal),
                    BinOp::Ne => order != Some(Equal),
                    BinOp::Lt => order == Some(Less),
                    BinOp::Le => matches!(order, Some(Less | Equal)),
                    BinOp::Gt => order == Some(Greater),
                    BinOp::Ge => matches!(order, Some(Greater | Equal)),
                    _ => return None,
                };
                return Some(Operand::bool(yes));
            }
        };
        Some(Operand::Const {
            ty,
            value: Value::Bytes(Rc::new(Aggregate {
                bytes: result.to_vec(),
                relocs: Vec::new(),
            })),
            untyped: false,
        })
    }
}
