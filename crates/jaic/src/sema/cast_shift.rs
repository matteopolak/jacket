//! `cast(u16) low << 8`: a cast binds more loosely than a shift in Jai, so the shift happens in
//! the operand's own type, before the cast widens it.
//!
//! The warning fires when the operand of an explicit integer cast has an unparenthesized `<<` on
//! its left spine and the operand's integer type is narrower than the cast's target. Parenthesizing
//! either side (`(cast(u16) low) << 8`, `cast(u16) (low << 8)`) says which order was meant.
use super::{Compiler, Span, TypeId, ast};
use crate::ast::{BinOp, ExprKind as E};
use crate::diagnostics::{Code, Diagnostic};

/// Whether an unparenthesized shift left sits at the start of `e`'s chain of bitwise operators:
/// the part of `e` that the cast's operand evaluates first.
fn starts_with_shift(e: &ast::Expr) -> bool {
    if e.parenthesized {
        return false;
    }
    match &e.kind {
        E::Binary(BinOp::Shl, ..) => true,
        E::Binary(BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shr, left, _) => {
            starts_with_shift(left)
        }
        _ => false,
    }
}

impl Compiler {
    /// Warns for `cast(target) value` when `value` is a shift done in a type narrower than
    /// `target`. `operand` is the type `value` had before the cast.
    pub(in crate::sema) fn warn_cast_shift_precedence(
        &mut self,
        span: Span,
        value: &ast::Expr,
        operand: TypeId,
        target: TypeId,
    ) {
        if !starts_with_shift(value) {
            return;
        }
        let types = &self.ty.types;
        let (Some((narrow, _)), Some((wide, _))) =
            (types.int_info(operand), types.int_info(target))
        else {
            return;
        };
        if narrow >= wide {
            return;
        }
        let from = types.name(operand);
        let to = types.name(target);
        self.warn(
            Diagnostic::new(
                Code::CastShiftPrecedence,
                span,
                format!("this shifts in `{from}` before casting to `{to}`, so bits shifted past `{from}` are lost"),
            )
            .with_label("a cast binds more loosely than a shift")
            .with_help(format!(
                "widen first with `(cast({to}) value) << count`, or write `cast({to}) (value << count)` to keep the narrow shift"
            )),
        );
    }
}
