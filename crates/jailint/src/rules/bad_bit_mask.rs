//! `bad_bit_mask`: `(x & 4) == 1`, `(x | 2) == 0`: a mask test that can never match.
//!
//! `x & mask` has only bits that are in `mask`, so comparing it with a constant that has a bit
//! outside the mask is always false (and `!=` always true). Likewise `x | mask` always has the
//! bits of `mask`, so it cannot equal a constant that lacks one of them. The typical cause is
//! a wrong constant (`(flags & 4) == 1` meant `== 4`, or `!= 0`) or the wrong operator.
//!
//! Fires on `==` and `!=` between a non-constant integer expression `x & c` or `x | c` (the
//! other operand of the operator an integer literal) and a non-negative integer literal. The
//! editor offers to write the answer.
use super::{int_literal, replacing};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::walk;
use jaic::ast::{BinOp, ExprKind as E};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "bad_bit_mask",
    category: crate::config::Category::Correctness,
    summary: "a mask test that can never match (`(x & 4) == 1`, `(x | 2) == 0`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(cmp @ (BinOp::Eq | BinOp::Ne), a, b) = &e.kind else {
                return true;
            };
            let (masked, k) = match (int_literal(a), int_literal(b)) {
                (None, Some(k)) => (&**a, k),
                (Some(k), None) => (&**b, k),
                _ => return true,
            };
            let E::Binary(op @ (BinOp::BitAnd | BinOp::BitOr), x, y) = &masked.kind else {
                return true;
            };
            let (x, c) = match (int_literal(x), int_literal(y)) {
                (None, Some(c)) => (&**x, c),
                (Some(c), None) => (&**y, c),
                _ => return true,
            };
            if k < 0 || c < 0 || cx.constant(x) || !cx.ty(x).is_some_and(|t| types.is_integer(t)) {
                return true;
            }
            let impossible = match op {
                BinOp::BitAnd => k & !c != 0,
                _ => c & !k != 0,
            };
            if !impossible {
                return true;
            }
            let always = *cmp == BinOp::Ne;
            let shown = cx.whole_src(e.span).trim();
            let why = if *op == BinOp::BitAnd {
                format!("`& {c}` leaves only the bits of {c}, and {k} has others")
            } else {
                format!("`| {c}` always sets the bits of {c}, and {k} lacks some of them")
            };
            out.push(replacing(
                e.span,
                format!("`{shown}` is always `{always}`: {why}"),
                "compare with the mask itself (`(x & m) == m`) or test for non-zero".into(),
                cx.whole(e.span),
                always.to_string(),
                false,
            ));
            true
        });
    }
}
