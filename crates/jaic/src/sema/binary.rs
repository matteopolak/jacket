//! Binary operators: operand checking and typing, constant folding, pointer arithmetic and
//! the lowered operation.
mod emit;
mod fold;
mod logical;
mod operands;
mod pointers;
mod typing;

use super::lower::{FnCtx, Operand};
use super::*;
use crate::ast::{BinOp, ExprKind as E};
use crate::ir::CmpOp;

impl Compiler {
    /// `a op b` for every operator but `&&` and `||`: check the operands, then fold, overload,
    /// do pointer arithmetic, or convert both sides to one type and emit the operation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_binary(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        op: BinOp,
        a: &ast::Expr,
        b: &ast::Expr,
        expected: Option<TypeId>,
        span: Span,
    ) -> Result<Operand> {
        if matches!(op, BinOp::And | BinOp::Or) {
            return self.check_logical(f, scope, op, a, b, span);
        }
        // `cast(float32) ((ifx c then a else b) & mask)`: a float target cannot be the type of an
        // operand of an operator that floats lack, so it does not reach the operands (an `ifx`
        // would widen to it, and the operator would then fail on a float).
        let expected = if is_bitwise(op) || op == BinOp::Rem {
            expected.filter(|&t| !self.types.is_float(self.types.repr(t)))
        } else {
            expected
        };
        let (lhs, rhs) = self.binary_operands(f, scope, op, a, b, expected)?;
        // Type comparisons fold at compile time.
        if let (Operand::Type(x), Operand::Type(y)) = (&lhs, &rhs) {
            return match op {
                BinOp::Eq => Ok(Operand::bool(x == y)),
                BinOp::Ne => Ok(Operand::bool(x != y)),
                _ => err(span, "types can only be compared with == and !="),
            };
        }
        if let Some(result) = self.try_binary_operator_overload(f, scope, op, &lhs, &rhs, span)? {
            return Ok(result);
        }
        if let (
            Operand::Const {
                ..
            },
            Operand::Const {
                ..
            },
        ) = (&lhs, &rhs)
            && let Some(folded) = self.fold_binary(op, &lhs, &rhs, span)?
        {
            return Ok(folded);
        }
        let (lhs, rhs) = self.procedure_operands(f, lhs, rhs, span)?;
        let (lty, rty) = (lhs.ty(), rhs.ty());
        if matches!(op, BinOp::Eq | BinOp::Ne) {
            self.reject_array_comparison(op, lty, rty, span)?;
        }
        if self.types.is_pointer(lty) && matches!(op, BinOp::Add | BinOp::Sub) {
            return self.pointer_offset(f, op, lhs, rhs, span);
        }
        if self.types.is_pointer(rty) && op == BinOp::Add && self.types.is_integer(lty) {
            return self.check_binary(f, scope, op, b, a, expected, span);
        }
        let pointer_and_int = (self.types.is_pointer(lty) && self.types.is_integer(rty))
            || (self.types.is_pointer(rty) && self.types.is_integer(lty));
        if is_bitwise(op) && pointer_and_int {
            return self.mask_pointer(f, op, lhs, rhs, span);
        }
        let (ty, rhs_ty, x, y) =
            self.unify_binary_operands(f, op, (a, b), lhs, rhs, expected, span)?;
        self.emit_binary(f, op, ty, rhs_ty, x, y, span)
    }
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Int(i) => *i as f64,
        Value::Float(f, _) => *f,
        _ => 0.0,
    }
}

fn cmp_op(op: BinOp, float: bool, signed: bool) -> CmpOp {
    match (op, float, signed) {
        (BinOp::Eq, true, _) => CmpOp::FEq,
        (BinOp::Ne, true, _) => CmpOp::FNe,
        (BinOp::Lt, true, _) => CmpOp::FLt,
        (BinOp::Le, true, _) => CmpOp::FLe,
        (BinOp::Gt, true, _) => CmpOp::FGt,
        (BinOp::Ge, true, _) => CmpOp::FGe,
        (BinOp::Eq, _, _) => CmpOp::Eq,
        (BinOp::Ne, _, _) => CmpOp::Ne,
        (BinOp::Lt, _, true) => CmpOp::SLt,
        (BinOp::Le, _, true) => CmpOp::SLe,
        (BinOp::Gt, _, true) => CmpOp::SGt,
        (BinOp::Ge, _, true) => CmpOp::SGe,
        (BinOp::Lt, _, false) => CmpOp::ULt,
        (BinOp::Le, _, false) => CmpOp::ULe,
        (BinOp::Gt, _, false) => CmpOp::UGt,
        _ => CmpOp::UGe,
    }
}

fn arith_op(op: BinOp, float: bool, signed: bool) -> Option<ir::BinOp> {
    use ir::BinOp as I;
    Some(match (op, float) {
        (BinOp::Add, true) => I::FAdd,
        (BinOp::Sub, true) => I::FSub,
        (BinOp::Mul, true) => I::FMul,
        (BinOp::Div, true) => I::FDiv,
        (_, true) => return None,
        (BinOp::Add, _) => I::Add,
        (BinOp::Sub, _) => I::Sub,
        (BinOp::Mul, _) => I::Mul,
        (BinOp::Div, _) => {
            if signed {
                I::SDiv
            } else {
                I::UDiv
            }
        }
        (BinOp::Rem, _) => {
            if signed {
                I::SRem
            } else {
                I::URem
            }
        }
        (BinOp::BitAnd, _) => I::And,
        (BinOp::BitOr, _) => I::Or,
        (BinOp::BitXor, _) => I::Xor,
        (BinOp::Shl, _) => I::Shl,
        (BinOp::Shr, _) => {
            if signed {
                I::AShr
            } else {
                I::LShr
            }
        }
        (BinOp::Rotl, _) => I::Rotl,
        (BinOp::Rotr, _) => I::Rotr,
        _ => return None,
    })
}

/// `~7`, `~0`: the complement of a constant. It is a 64-bit pattern whatever the other operand
/// is, so it does not adapt to the width of that operand.
fn is_complement_literal(e: &ast::Expr) -> bool {
    matches!(&e.kind, E::Unary(ast::UnOp::BitNot, x) if is_number_literal(x))
}

/// A numeric literal, possibly signed or parenthesised arithmetic of literals (`-1`, `(2 * 3)`):
/// an expression that is an untyped constant whatever the context.
fn is_number_literal(e: &ast::Expr) -> bool {
    match &e.kind {
        E::Int(_) | E::Float(_) => true,
        E::Unary(ast::UnOp::Neg | ast::UnOp::Plus | ast::UnOp::BitNot, x) => is_number_literal(x),
        E::Binary(_, a, b) => is_number_literal(a) && is_number_literal(b),
        _ => false,
    }
}

fn is_comparison(op: BinOp) -> bool {
    matches!(
        op,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    )
}

fn is_shift(op: BinOp) -> bool {
    matches!(op, BinOp::Shl | BinOp::Shr | BinOp::Rotl | BinOp::Rotr)
}

fn is_bitwise(op: BinOp) -> bool {
    matches!(op, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
}
