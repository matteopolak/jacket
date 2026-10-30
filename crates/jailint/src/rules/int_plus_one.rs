//! `int_plus_one`: `x >= y + 1` is `x > y`.
//!
//! An integer `>=` or `<=` with `1` added or taken off one side is a strict comparison written
//! the long way round:
//!
//! | written | is |
//! | --- | --- |
//! | `x >= y + 1`, `x - 1 >= y` | `x > y` |
//! | `x + 1 <= y`, `x <= y - 1` | `x < y` |
//!
//! Only integers are looked at (a float is not "one more" in the same way), and only the literal
//! `1`. The editor offers the strict comparison. `--fix` leaves it, since at the end of the
//! type's range `y + 1` wraps and the two forms differ.
use super::{int_literal, op_text, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::walk;
use jaic::ast::{BinOp, Expr, ExprKind as E};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "int_plus_one",
    category: crate::config::Category::Complexity,
    summary: "`x >= y + 1`, which is `x > y`",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op @ (BinOp::Ge | BinOp::Le), l, r) = &e.kind else {
                return true;
            };
            let types = &cx.compiler.ty.types;
            if !cx.ty(l).is_some_and(|t| types.is_integer(t))
                || !cx.ty(r).is_some_and(|t| types.is_integer(t))
            {
                return true;
            }
            // The side that carries the `1`, the plain operands, and the strict operator.
            let (strict, plain_left, plain_right) = if *op == BinOp::Ge {
                if let Some(y) = plus_one(r, BinOp::Add) {
                    (BinOp::Gt, l.as_ref(), y)
                } else if let Some(x) = plus_one(l, BinOp::Sub) {
                    (BinOp::Gt, x, r.as_ref())
                } else {
                    return true;
                }
            } else if let Some(x) = plus_one(l, BinOp::Add) {
                (BinOp::Lt, x, r.as_ref())
            } else if let Some(y) = plus_one(r, BinOp::Sub) {
                (BinOp::Lt, l.as_ref(), y)
            } else {
                return true;
            };
            // The comparison as written, parentheses around either side included.
            let (start, end) = (cx.whole(l.span).0, cx.whole(r.span).1);
            let written = cx.text.get(start..end).unwrap_or("");
            let (left, right) = (
                cx.whole_src(plain_left.span).trim(),
                cx.whole_src(plain_right.span).trim(),
            );
            let replacement = format!("{left} {} {right}", op_text(strict));
            out.push(suggesting(
                Span::new(cx.file, start, end),
                format!("`{written}` is a strict comparison written with `1`"),
                format!("write `{replacement}`"),
                format!("write `{replacement}`"),
                vec![Edit {
                    start,
                    end,
                    text: replacement.clone(),
                }],
            ));
            true
        });
    }
}

/// `e` is `x + 1` (with `Add`) or `x - 1` (with `Sub`): `x`.
fn plus_one(e: &Expr, op: BinOp) -> Option<&Expr> {
    match &e.kind {
        E::Binary(found, x, one) if *found == op && int_literal(one) == Some(1) => Some(x),
        _ => None,
    }
}
