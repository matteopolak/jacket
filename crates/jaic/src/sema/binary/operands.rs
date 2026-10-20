//! Checking the operands of a binary expression.
use super::{is_comparison, is_complement_literal, is_number_literal, is_shift};
use crate::ast::{BinOp, ExprKind as E};
use crate::sema::TypeState;
use crate::sema::calls::binop_text;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::{Compiler, Diagnostic, Result, ScopeId, Span, TypeId, Value, ast};
use crate::types::TypeKind;

impl Compiler {
    /// Check both operands of `a op b`, each with the type the other or `expected` gives it.
    pub(super) fn binary_operands(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        expected: Option<TypeId>,
    ) -> Result<(Operand, Operand)> {
        let is_cmp = is_comparison(op);
        let is_shift = is_shift(op);
        let lhs_expected = if is_cmp {
            None
        } else if is_shift {
            // `cast(float) (1 << n)` shifts an integer.
            expected.filter(|&t| self.ty.types.is_integer(self.ty.types.repr(t)))
        } else {
            expected
        };
        // `.FIRST == x`: an inferred member on the left takes the right operand's type, so
        // the right side is checked first.
        let mut early_rhs = None;
        let is_xx = |e: &ast::Expr| {
            matches!(
                e.kind,
                E::Cast {
                    ty: None,
                    ..
                }
            )
        };
        // `.FLAG & x.flags` too: the member takes the flags type, not an expected `int`.
        // An expected enum type already names the member (`.A & ~.B` as a flags argument).
        let bitwise = matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
            && !lhs_expected.is_some_and(|t| {
                matches!(
                    self.ty.types.kind(self.ty.types.repr_struct(t)),
                    TypeKind::Enum(_)
                )
            });
        let mut lhs = if ((is_cmp
            && (matches!(a.kind, E::InferredMember(_)) || (is_xx(a) && !is_xx(b))))
            || (bitwise && matches!(a.kind, E::InferredMember(_)))
            || (!is_cmp && !is_shift && is_xx(a) && !is_xx(b)))
            && !matches!(b.kind, E::InferredMember(_))
        {
            // `xx err == GL_FALSE` casts to the other operand's type too, and so does `xx` in
            // arithmetic next to an operand that has a type; the expected type of the whole
            // expression does not reach it.
            let r = self.check_expr(f, scope, b, None)?;
            let typed = !matches!(
                r,
                Operand::Const {
                    untyped: true,
                    ..
                }
            );
            let l = self.check_expr(f, scope, a, (is_cmp || typed).then(|| r.ty()))?;
            early_rhs = Some(r);
            l
        } else if lhs_expected.is_some() && is_number_literal(a) {
            // `x: float32 = 0 - w`: the literal is matched with `w`, not with the declaration, so
            // the subtraction is in `u16` and only its result converts. It stays untyped here and
            // takes the right operand's type below; with an untyped right operand, both fold
            // under the expected type as before {#num.16}.
            self.check_expr(f, scope, a, None)?
        } else {
            let own = is_complement_literal(a) && !is_shift && !is_cmp;
            f.ifx_operand = matches!(a.kind, E::Ifx { .. });
            let l = self.check_expr(
                f,
                scope,
                a,
                if own {
                    None
                } else {
                    lhs_expected
                },
            )?;
            f.ifx_operand = false;
            l
        };
        if let Operand::PolyStruct(ps) = lhs {
            lhs = Operand::Type(self.ty.poly_struct_type(ps));
        }
        // Inferred enum members on the right take the left operand's type.
        let pointer_lhs = self.ty.types.is_pointer(self.ty.types.repr(lhs.ty()));
        let pointer_offset = matches!(op, BinOp::Add | BinOp::Sub) && pointer_lhs;
        let pointer_bits =
            matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor) && pointer_lhs;
        let rhs_expected = if is_shift || pointer_bits {
            None
        } else if pointer_offset {
            // `p += ifx c then 3 else 1`: the offset is an integer.
            Some(TypeId::S64)
        } else {
            (!matches!(
                lhs,
                Operand::Const {
                    untyped: true,
                    ..
                }
            ))
            .then(|| lhs.ty())
        };
        let mut rhs = if let Some(r) = early_rhs {
            r
        } else {
            f.ifx_operand = matches!(b.kind, E::Ifx { .. });
            let r = self.check_expr(
                f,
                scope,
                b,
                if is_complement_literal(b) && !is_shift && !is_cmp {
                    None
                } else {
                    rhs_expected.or(if is_cmp || is_shift {
                        None
                    } else {
                        expected
                    })
                },
            )?;
            f.ifx_operand = false;
            r
        };
        if let Operand::PolyStruct(ps) = rhs {
            rhs = Operand::Type(self.ty.poly_struct_type(ps));
        }
        // `s[0] == "-"`, `c - "0"`: a one-byte string constant next to a `u8` compares and
        // offsets as that byte. Wider integers do not take the conversion.
        if is_cmp || matches!(op, BinOp::Add | BinOp::Sub) {
            let byte_of = |c: &Compiler, op: &Operand, other: &Operand| match op {
                Operand::Const {
                    value: Value::String(bytes),
                    ..
                } if bytes.len() == 1 && c.ty.types.repr(other.ty()) == TypeId::U8 => {
                    Some(Operand::untyped_int(bytes[0] as i128))
                }
                _ => None,
            };
            if let Some(b) = byte_of(self, &rhs, &lhs) {
                rhs = b;
            } else if let Some(b) = byte_of(self, &lhs, &rhs) {
                lhs = b;
            }
        }
        if matches!(
            lhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !matches!(
            rhs,
            Operand::Const {
                untyped: true,
                ..
            }
        ) && !is_shift
        {
            // Re-check the left side with the right side's type for inferred members.
            if matches!(a.kind, E::InferredMember(_)) {
                lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
            }
        }
        if matches!(a.kind, E::InferredMember(_)) && !matches!(b.kind, E::InferredMember(_)) {
            lhs = self.check_expr(f, scope, a, Some(rhs.ty()))?;
        }
        if matches!(b.kind, E::InferredMember(_)) {
            rhs = self.check_expr(f, scope, b, Some(lhs.ty()))?;
        }
        Ok((lhs, rhs))
    }

    /// Two procedure names compare as their addresses, and a procedure name compared with a
    /// procedure value takes the value's type.
    pub(super) fn procedure_operands(
        &mut self,
        f: &mut FnCtx,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    ) -> Result<(Operand, Operand)> {
        match (&lhs, &rhs) {
            (Operand::Procs(l), Operand::Procs(r)) if l.len() == 1 && r.len() == 1 => {
                let (lt, rt) = (self.proc_type(l[0], span)?, self.proc_type(r[0], span)?);
                Ok((
                    self.value_as(f, lhs, lt, span)?,
                    self.value_as(f, rhs, rt, span)?,
                ))
            }
            (Operand::Procs(_), Operand::Procs(_)) => Ok((lhs, rhs)),
            (_, Operand::Procs(_)) => {
                let ty = lhs.ty();
                Ok((lhs, self.value_as(f, rhs, ty, span)?))
            }
            (Operand::Procs(_), _) => {
                let ty = rhs.ty();
                Ok((self.value_as(f, lhs, ty, span)?, rhs))
            }
            _ => Ok((lhs, rhs)),
        }
    }

    /// `op` converted to `ty` and loaded as a value.
    fn value_as(&mut self, f: &mut FnCtx, op: Operand, ty: TypeId, span: Span) -> Result<Operand> {
        let converted = self.convert(f, op, ty, span)?;
        let (ty, val) = self.rvalue(f, converted, span)?;
        Ok(Operand::Value {
            ty,
            val,
        })
    }
}

impl TypeState {
    /// Arrays (`[]T`, `[N]T`, `[..]T`) have no `==`; strings do.
    pub(super) fn reject_array_comparison(
        &self,
        op: BinOp,
        lty: TypeId,
        rty: TypeId,
        span: Span,
    ) -> Result<()> {
        for t in [lty, rty] {
            if matches!(self.types.kind(self.types.repr(t)), TypeKind::Array { .. }) {
                let symbol = binop_text(op);
                return Err(Box::new(
                    Diagnostic::error(span, format!("arrays cannot be compared with `{symbol}`"))
                        .with_note(
                            span,
                            format!("the compared type is `{}`", self.types.name(t)),
                        )
                        .with_help("compare the lengths and elements in a loop"),
                ));
            }
        }
        Ok(())
    }
}
