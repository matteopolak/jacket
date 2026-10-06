//! `identical_operands`: `a == a`, `x - x`, `flags & flags`, `ok && ok`.
//!
//! An operator whose two sides are the same expression has a fixed or trivial result
//! (`a == a` is `true`, `x - x` is `0`, `f & f` is `f`). Written on purpose that is rare; far
//! more often one side was meant to be something else (`a.x == b.x` typed as `a.x == a.x`
//! after copying a line).
//!
//! Fires for comparisons, `-`, `/`, `%`, `&`, `|`, `^`, `&&` and `||` when both operands are
//! the same side-effect-free expression (names, member paths, literals, indexing; no calls),
//! spelled the same, of a number, bool, pointer or enum type, and not compile-time constants.
//! Float `==`, `!=` and `-` are left alone: `x != x` tests for NaN and `x - x == 0` for a finite
//! value. Operator procedures are skipped: they may compare a value with itself on purpose.
use super::{finding, op_text, same_pure};
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{BinOp, ExprKind as E};
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() || p.header.operator.is_some() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op, a, b) = &e.kind else {
                return true;
            };
            let Some(ty) = cx.ty(a) else {
                return true;
            };
            let types = &cx.compiler.types;
            let scalar = matches!(
                types.kind(types.repr(ty)),
                TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::Bool
                    | TypeKind::Pointer(_)
            );
            let float = types.is_float(ty);
            let result = match op {
                BinOp::Eq | BinOp::Le | BinOp::Ge if !float => "always `true`",
                BinOp::Le | BinOp::Ge => "`true` unless it is NaN",
                BinOp::Ne | BinOp::Lt | BinOp::Gt if !float => "always `false`",
                BinOp::Lt | BinOp::Gt => "always `false`",
                BinOp::Sub | BinOp::BitXor if !float => "always `0`",
                BinOp::Div | BinOp::Rem if !float => "fixed (or a division by zero)",
                BinOp::BitAnd | BinOp::BitOr | BinOp::And | BinOp::Or => "the operand itself",
                _ => return true,
            };
            if !scalar || cx.constant(a) || !same_pure(cx, a, b) {
                return true;
            }
            let side = cx.whole_src(a.span).trim();
            out.push(finding(
                e.span,
                format!("both sides of `{}` are `{side}`", op_text(*op)),
                Some(format!(
                    "the result is {result}; was one side meant to be something else?"
                )),
            ));
            true
        });
    }
}
