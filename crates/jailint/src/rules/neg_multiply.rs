//! `neg_multiply`: `x * -1` is `-x`.
//!
//! Multiplying by `-1` (or `-1.0`) is a negation, and `-x` says so.
//!
//! Fires on a product with the literal `-1` on either side, when the other side is a signed
//! integer or a float (an unsigned `-1` is its own mistake, and a struct may define `operator *`
//! to mean something else). The fix writes `-x`, with parentheses when `x` is not a name, call
//! or other single term; it is machine-applicable unless a comment sits inside.
use super::replacing;
use crate::Finding;
use crate::syntax::{Cx, is_atom, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E, UnOp};
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(BinOp::Mul, a, b) = &e.kind else {
                return true;
            };
            let other = match (minus_one(a), minus_one(b)) {
                (false, true) => a,
                (true, false) => b,
                _ => return true,
            };
            let types = &cx.compiler.ty.types;
            if !cx.ty(other).is_some_and(|t| {
                types.is_float(t) || types.int_info(t).is_some_and(|(_, signed)| signed)
            }) {
                return true;
            }
            let (start, end) = (
                cx.whole(a.span).0.min(cx.whole(b.span).0),
                cx.whole(b.span).1.max(cx.whole(a.span).1),
            );
            let written = cx.text.get(start..end).unwrap_or("");
            let value = cx.whole_src(other.span).trim();
            let negated = if is_atom(other) || cx.parenthesized(other.span) {
                format!("-{value}")
            } else {
                format!("-({value})")
            };
            out.push(replacing(
                Span::new(cx.file, start, end),
                format!("`{written}` is a negation"),
                format!("write `{negated}`"),
                (start, end),
                negated.clone(),
                !cx.has_comment_between(start, end),
            ));
            true
        });
    }
}

/// The literal `-1` or `-1.0`.
fn minus_one(e: &Expr) -> bool {
    let E::Unary(UnOp::Neg, inner) = &e.kind else {
        return false;
    };
    match &inner.kind {
        E::Int(v) => v.get() == 1,
        E::Float(v) => v.to_bits() == 1.0_f64.to_bits(),
        _ => false,
    }
}
