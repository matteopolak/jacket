//! `misrefactored_assign_op`: `total += total + x;`, an update that applies the operator twice.
//!
//! Turning `total = total + x;` into a compound assignment goes wrong when the old left side
//! stays behind: `total += total + x;` adds `total` to itself as well as `x`, which is
//! `total = total + total + x`.
//!
//! Fires on `a op= a op b` for the arithmetic and bitwise operators, with the same `op` in both
//! places, a side-effect-free target (names, member paths, indexing; no calls) of a number type
//! spelled the same on both sides. For `+`, `*`, `&`, `|` and `^` on integers, `a op= b op a` is
//! reported too, when `b` has no side effects. The editor offers `a op= b`; which of the two
//! updates was meant is a guess, so `--fix` leaves it alone.
use super::{op_text, pure, suggesting};
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding};
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
                op: AssignOp::Op(assign_op),
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
            if op != assign_op || !pure(target) {
                return true;
            }
            let types = &cx.compiler.ty.types;
            let Some(ty) = cx.ty(target) else {
                return true;
            };
            if !(types.is_integer(ty) || types.is_float(ty)) {
                return true;
            }
            let same = |e: &jaic::ast::Expr| cx.same_tokens(target.span, e.span);
            let commutes = matches!(
                op,
                BinOp::Add | BinOp::Mul | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor
            ) && types.is_integer(ty);
            let other = if same(a) {
                b
            } else if commutes && same(b) && pure(a) {
                a
            } else {
                return true;
            };
            let target_text = cx.whole_src(target.span).trim();
            let other_text = cx.whole_src(other.span).trim();
            let sym = op_text(*op);
            let (start, end) = cx.whole(value.span);
            out.push(suggesting(
                s.span,
                format!("`{target_text} {sym}= {target_text} {sym} ...` applies `{sym}` to `{target_text}` twice"),
                format!(
                    "`{sym}=` already starts from `{target_text}`: write `{target_text} {sym}= {other_text}`"
                ),
                format!("write `{target_text} {sym}= {other_text}`"),
                vec![Edit {
                    start,
                    end,
                    text: other_text.to_string(),
                }],
            ));
            true
        });
    }
}
