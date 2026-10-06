//! `identity_op`: `x + 0`, `x * 1`, `x | 0`.
//!
//! The operation leaves `x` unchanged, so it only adds noise, and it often marks an edit that
//! went wrong (`x * 1` where a scale factor was meant).
//!
//! Fires when one side is the integer literal `0` (for `|`, `^`; on the right for `+`, `-`)
//! or `1` (for `*`; on the right for `/`) and the other side is a non-constant
//! integer. `0 + x` is left alone: it reads as a base plus an offset, usually next to
//! `K + x` (`ifx mated then -INF + ply else 0 + ply`). Inside an index (`data[i * 4 + 0]`) and
//! in the elements of an array literal the `+ 0` usually lines up with neighbouring `+ 1`,
//! `+ 2`, so it is left alone too, and so are shifts by `0`: `(rgb >> 0) & 0xFF` sits beside
//! `>> 8` and `>> 16` for the same reason. The fix (drop the operation) is machine-applicable.
use super::{int_literal, op_text, replacing};
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
            let Some(kept) = identity(*op, a, b) else {
                return true;
            };
            let types = &cx.compiler.types;
            if !cx.ty(kept).is_some_and(|t| types.is_integer(t))
                || cx.constant(kept)
                || cx.ty(e) != cx.ty(kept)
                || in_layout(chain)
            {
                return true;
            }
            let kept_text = cx.whole_src(kept.span).trim().to_string();
            let written = cx.whole_src(e.span).trim();
            out.push(replacing(
                e.span,
                format!(
                    "`{written}` is just `{kept_text}`: `{}` with this operand changes nothing",
                    op_text(*op)
                ),
                format!("write `{kept_text}`"),
                cx.whole(e.span),
                kept_text,
                true,
            ));
            true
        });
    }
}

/// The operand `op` leaves unchanged, when the other is its identity literal.
fn identity<'e>(op: BinOp, a: &'e Expr, b: &'e Expr) -> Option<&'e Expr> {
    let (la, lb) = (int_literal(a), int_literal(b));
    match op {
        BinOp::Add | BinOp::BitOr | BinOp::BitXor if lb == Some(0) => Some(a),
        // `0 + x` reads as a base plus an offset, often next to `K + x`; it is left alone.
        BinOp::BitOr | BinOp::BitXor if la == Some(0) => Some(b),
        BinOp::Sub if lb == Some(0) => Some(a),
        BinOp::Mul if lb == Some(1) => Some(a),
        BinOp::Mul if la == Some(1) => Some(b),
        BinOp::Div if lb == Some(1) => Some(a),
        _ => None,
    }
}

/// Inside an index or an array literal's elements.
fn in_layout(chain: &[Node<'_>]) -> bool {
    chain.iter().any(|n| {
        matches!(
            n,
            Node::Expr(Expr {
                kind: E::Index(..) | E::ArrayLit { .. },
                ..
            })
        )
    })
}
