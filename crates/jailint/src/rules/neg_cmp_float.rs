//! `neg_cmp_float`: `!(a < b)` with floats, which is not `a >= b`.
//!
//! Every comparison with a NaN is false, so `!(a < b)` is true when `a` or `b` is NaN while
//! `a >= b` is false. Rewriting one as the other changes what happens to NaN. Writing the
//! negation hides which of the two was meant: if NaN should take the branch, say so with an
//! explicit test (`a != a`); if it should not, use the inverted comparison.
//!
//! Fires on a negated `<`, `<=`, `>` or `>=` whose operands are floats. `!(a == b)` is exactly
//! `a != b` and is not reported. The editor offers the inverted comparison (it changes the
//! result for NaN, so `--fix` leaves it alone). Off by default (`allow`): `!(x < limit)` is a
//! common way to stay on the safe side of NaN on purpose.
use super::{op_text, suggesting};
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding};
use jaic::ast::{BinOp, ExprKind as E, UnOp};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Unary(UnOp::Not, inner) = &e.kind else {
                return true;
            };
            let E::Binary(op @ (BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge), a, b) = &inner.kind
            else {
                return true;
            };
            if !(cx.ty(a).is_some_and(|t| types.is_float(t))
                || cx.ty(b).is_some_and(|t| types.is_float(t)))
            {
                return true;
            }
            let inverse = match op {
                BinOp::Lt => BinOp::Ge,
                BinOp::Le => BinOp::Gt,
                BinOp::Gt => BinOp::Le,
                _ => BinOp::Lt,
            };
            let (a_src, b_src) = (cx.whole_src(a.span).trim(), cx.whole_src(b.span).trim());
            let bare = matches!(
                chain.last(),
                Some(
                    Node::Stmt(_)
                        | Node::Expr(jaic::ast::Expr {
                            kind: E::Binary(BinOp::And | BinOp::Or, ..),
                            ..
                        })
                ) | None
            );
            let text = format!("{a_src} {} {b_src}", op_text(inverse));
            let text = if bare {
                text
            } else {
                format!("({text})")
            };
            let (start, end) = cx.whole(e.span);
            out.push(suggesting(
                e.span,
                format!(
                    "`!({a_src} {} {b_src})` is not `{a_src} {} {b_src}` when a float is NaN",
                    op_text(*op),
                    op_text(inverse)
                ),
                "write the inverted comparison if NaN should not take the branch, or test for NaN explicitly if it should".into(),
                format!("write `{text}`"),
                vec![Edit {
                    start,
                    end,
                    text,
                }],
            ));
            true
        });
    }
}
