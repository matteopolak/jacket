//! `erasing_op`: `x * 0`, `x & 0`, `x % 1`.
//!
//! The result is `0` whatever `x` is, so `x` might as well not be there. Usually the literal is
//! wrong (`x * 0` for `x * 10`, `flags & 0` for `flags & MASK`), or the code is a leftover.
//!
//! Fires when an integer operation with a non-constant integer operand is `0` for every value
//! of it: `x * 0`, `0 * x`, `x & 0`, `0 & x`, `x % 1`, `0 % x`, `0 / x`, `0 << x`, `0 >> x`.
//! Inside an index or an array literal (`m[0 * 4 + 1]`, lining up with neighbouring rows) it
//! is left alone.
use super::{finding, int_literal, op_text};
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op, a, b) = &e.kind else {
                return true;
            };
            let (la, lb) = (int_literal(a), int_literal(b));
            let erased = match op {
                BinOp::Mul | BinOp::BitAnd if lb == Some(0) => a,
                BinOp::Mul | BinOp::BitAnd if la == Some(0) => b,
                BinOp::Rem if lb == Some(1) => a,
                BinOp::Rem | BinOp::Div | BinOp::Shl | BinOp::Shr if la == Some(0) => b,
                _ => return true,
            };
            let types = &cx.compiler.types;
            if !cx.ty(erased).is_some_and(|t| types.is_integer(t))
                || cx.constant(erased)
                || chain.iter().any(|n| {
                    matches!(
                        n,
                        Node::Expr(Expr {
                            kind: E::Index(..) | E::ArrayLit { .. },
                            ..
                        })
                    )
                })
            {
                return true;
            }
            let written = cx.whole_src(e.span).trim();
            out.push(finding(
                e.span,
                format!("`{written}` is always `0`"),
                Some(format!(
                    "`{}` with this literal erases `{}`: was another value meant?",
                    op_text(*op),
                    cx.whole_src(erased.span).trim()
                )),
            ));
            true
        });
    }
}
