//! `manual_assign_op`: `total = total + x;` → `total += x;`.
//!
//! The compound form names the target once, so a long target (`state.players[i].score`) cannot
//! be misspelled on one side, and the reader sees at once that the value is updated in place.
//!
//! Fires on `a = a OP b` for the arithmetic and bitwise operators that have a compound form,
//! when the target is a side-effect-free expression (names, member paths, indexing; no calls)
//! of a number type, spelled the same on both sides, and `a OP b` has the target's type (so the
//! compound form converts nothing differently). For `+`, `*`, `&`, `|` and `^` on integers,
//! `a = b OP a` is reported too. The fix is machine-applicable.
use super::{op_text, pure, replacing};
use crate::Finding;
use crate::syntax::{Cx, squash, walk};
use jaic::ast::{AssignOp, BinOp, ExprKind as E, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::Assign {
                op: AssignOp::Assign,
                lhs,
                rhs,
            } = &s.kind
            else {
                return true;
            };
            let ([target], [value]) = (lhs.as_slice(), rhs.as_slice()) else {
                return true;
            };
            let E::Binary(op, a, b) = &value.kind else {
                return true;
            };
            if !matches!(
                op,
                BinOp::Add
                    | BinOp::Sub
                    | BinOp::Mul
                    | BinOp::Div
                    | BinOp::Rem
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXor
                    | BinOp::Shl
                    | BinOp::Shr
            ) || !pure(target)
            {
                return true;
            }
            let types = &cx.compiler.types;
            let Some(ty) = cx.ty(target) else {
                return true;
            };
            if !(types.is_integer(ty) || types.is_float(ty)) || cx.ty(value) != Some(ty) {
                return true;
            }
            let written = squash(cx.whole_src(target.span));
            let same = |e: &jaic::ast::Expr| squash(cx.whole_src(e.span)) == written;
            let commutes = matches!(
                op,
                BinOp::Add | BinOp::Mul | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor
            ) && types.is_integer(ty);
            let other = if same(a) {
                b
            } else if commutes && same(b) {
                a
            } else {
                return true;
            };
            let target_text = cx.whole_src(target.span).trim();
            let other_text = cx.whole_src(other.span).trim();
            let replacement = format!("{target_text} {}= {other_text}", op_text(*op));
            let (start, end) = cx.whole(value.span);
            let help = format!("write `{replacement}`");
            out.push(replacing(
                s.span,
                format!(
                    "`{target_text}` is updated with `{}` spelled out",
                    op_text(*op)
                ),
                help,
                (cx.whole(target.span).0, end.max(start)),
                replacement,
                true,
            ));
            true
        });
    }
}
