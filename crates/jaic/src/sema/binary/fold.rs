//! Constant folding of binary expressions.
use super::num;
use crate::ast::BinOp;
use crate::sema::expr::wrap_int;
use crate::sema::lower::Operand;
use crate::sema::*;
use crate::types::TypeKind;

impl Compiler {
    pub(in crate::sema) fn fold_binary(
        &mut self,
        op: BinOp,
        lhs: &Operand,
        rhs: &Operand,
        span: Span,
    ) -> Result<Option<Operand>> {
        let (
            Operand::Const {
                ty: lt,
                value: lv,
                untyped: lu,
            },
            Operand::Const {
                ty: rt,
                value: rv,
                untyped: ru,
            },
        ) = (lhs, rhs)
        else {
            return Ok(None);
        };
        if self.wide_float(*lt).is_some() || self.wide_float(*rt).is_some() {
            return Ok(self.wide_fold(op, (*lt, lv), (*rt, rv)));
        }
        let untyped = *lu && *ru;
        let is_shift = matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr);
        let plain_int = |c: &Self, t: TypeId| {
            c.types.int_info(t).is_some() && !matches!(c.types.kind(t), TypeKind::Enum(_))
        };
        let ty = if *lu && !*ru && !is_shift {
            *rt
        } else if !*lu
            && !*ru
            && !is_shift
            && lt != rt
            && plain_int(self, *lt)
            && plain_int(self, *rt)
        {
            // Two typed integer constants meet in the wider type, as variables do
            // (`cast(s16) 1 + cast(s32) 2` is `s32`).
            if self.implicit_cost(*rt, false, *lt).is_some() {
                *lt
            } else if self.implicit_cost(*lt, false, *rt).is_some() {
                *rt
            } else if self.types.int_info(*lt).unwrap().0 >= self.types.int_info(*rt).unwrap().0 {
                *lt
            } else {
                *rt
            }
        } else {
            *lt
        };
        let cmp = |o: std::cmp::Ordering| -> Option<Value> {
            use std::cmp::Ordering::*;
            Some(Value::Bool(match op {
                BinOp::Eq => o == Equal,
                BinOp::Ne => o != Equal,
                BinOp::Lt => o == Less,
                BinOp::Le => o != Greater,
                BinOp::Gt => o == Greater,
                BinOp::Ge => o != Less,
                _ => return None,
            }))
        };
        let result = match (lv, rv) {
            (Value::Int(a), Value::Int(b)) if !self.types.is_float(ty) => {
                let (a, b) = (*a, *b);
                let v = match op {
                    BinOp::Add => Some(Value::Int(a.wrapping_add(b))),
                    BinOp::Sub => Some(Value::Int(a.wrapping_sub(b))),
                    BinOp::Mul => Some(Value::Int(a.wrapping_mul(b))),
                    BinOp::Div => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_div(b)))
                    }
                    BinOp::Rem => {
                        if b == 0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(Value::Int(a.wrapping_rem(b)))
                    }
                    BinOp::BitAnd => Some(Value::Int(a & b)),
                    BinOp::BitOr => Some(Value::Int(a | b)),
                    BinOp::BitXor => Some(Value::Int(a ^ b)),
                    BinOp::Shl | BinOp::Shr if b < 0 => {
                        return err(
                            span,
                            format!("negative shift amount {b} in constant expression"),
                        );
                    }
                    BinOp::Shl => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a << b
                    })),
                    BinOp::Shr => Some(Value::Int(if b >= 128 {
                        0
                    } else {
                        a >> b
                    })),
                    BinOp::Rotl | BinOp::Rotr => None,
                    _ => cmp(a.cmp(&b)),
                };
                // Wrap typed results to their width.
                match v {
                    Some(Value::Int(x)) if !untyped => {
                        // A typed integer constant may carry a non-integer type (an enum value, a
                        // `bool`); those wrap like a 64-bit signed integer.
                        let (bits, signed) = self.types.int_info(ty).unwrap_or((64, true));
                        Some(Value::Int(wrap_int(x, bits, signed)))
                    }
                    Some(Value::Int(x)) => Some(Value::Int(x)),
                    other => other,
                }
            }
            (Value::Int(_) | Value::Float(_, _), Value::Int(_) | Value::Float(_, _)) => {
                let mut precision = crate::lexer::FloatLiteralPrecision::default();
                for value in [lv, rv] {
                    if let Value::Float(_, p) = value {
                        precision.defaults_to_float64 |= p.defaults_to_float64;
                        precision.requires_float64 |= p.requires_float64;
                        precision.requires_float64_due_to_significant_digits |=
                            p.requires_float64_due_to_significant_digits;
                    }
                }
                if self.types.repr(ty) == TypeId::F32 && !untyped && precision.requires_float64 {
                    return err(
                        span,
                        "this floating literal requires float64 precision (use an explicit cast to round it)",
                    );
                }
                let a = num(lv);
                let b = num(rv);
                let float = |value| {
                    let mut p = precision;
                    p.requires_float64 |= crate::lexer::has_big_float32_exponent(value);
                    Value::Float(value, p)
                };
                match op {
                    BinOp::Add => Some(float(a + b)),
                    BinOp::Sub => Some(float(a - b)),
                    BinOp::Mul => Some(float(a * b)),
                    BinOp::Div => {
                        if b == 0.0 {
                            return err(span, "division by zero in constant expression");
                        }
                        Some(float(a / b))
                    }
                    _ => a.partial_cmp(&b).and_then(cmp),
                }
            }
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                BinOp::BitAnd => Some(Value::Bool(*a & *b)),
                BinOp::BitOr => Some(Value::Bool(*a | *b)),
                BinOp::BitXor => Some(Value::Bool(*a ^ *b)),
                _ => None,
            },
            (Value::String(a), Value::String(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            (Value::Null, Value::Null) => cmp(std::cmp::Ordering::Equal),
            (Value::Proc(a), Value::Proc(b)) => match op {
                BinOp::Eq => Some(Value::Bool(a == b)),
                BinOp::Ne => Some(Value::Bool(a != b)),
                _ => None,
            },
            _ => None,
        };
        let Some(value) = result else {
            return Ok(None);
        };
        let is_bool = matches!(value, Value::Bool(_));
        let rty = if is_bool
            && !matches!((lv, rv), (Value::Bool(_), Value::Bool(_)) if !matches!(op, BinOp::Eq | BinOp::Ne))
        {
            TypeId::BOOL
        } else if matches!(value, Value::Float(_, _)) && !self.types.is_float(ty) {
            if self.types.is_float(*rt) {
                *rt
            } else {
                TypeId::F32
            }
        } else {
            ty
        };
        Ok(Some(Operand::Const {
            ty: rty,
            value,
            untyped: untyped && !is_bool,
        }))
    }
}
