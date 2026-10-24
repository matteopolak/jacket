//! `unsigned_range_end_minus_one`: `for i: 0..n-1` with an unsigned `n`.
//!
//! Ranges in Jai include their end, so `0..n-1` visits the `n` indices below `n`. When `n` is
//! unsigned and zero, `n - 1` wraps around to the largest value (or stops the program, with
//! overflow checks on) and the loop runs for billions of iterations over memory that is not
//! there. A signed `n` gives `0..-1`, an empty loop.
//!
//! Fires on a `for` over a range whose end is `n - 1`, where `n` is a non-constant unsigned
//! integer. Guard the zero case (`if n > 0`), or make the count signed. No fix.
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{BinOp, ExprKind as E, ForOver, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let types = &cx.compiler.ty.types;
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::For(f) = &s.kind else {
                return true;
            };
            let ForOver::Range(_, hi) = &f.over else {
                return true;
            };
            let E::Binary(BinOp::Sub, count, one) = &hi.kind else {
                return true;
            };
            if !matches!(one.kind, E::Int(v) if v.get() == 1) || cx.constant(count) {
                return true;
            }
            let Some(ty) = cx.ty(count) else {
                return true;
            };
            if types.int_info(ty).is_none_or(|(_, signed)| signed) {
                return true;
            }
            out.push(super::finding(
                hi.span,
                format!(
                    "`{}` is unsigned, so this range runs past the end when it is 0",
                    cx.src(count.span).trim()
                ),
                Some(format!(
                    "`{}` has type `{}` and `- 1` wraps around at 0: check for 0 first, or count with a signed integer",
                    cx.src(count.span).trim(),
                    types.name(ty)
                )),
            ));
            true
        });
    }
}
