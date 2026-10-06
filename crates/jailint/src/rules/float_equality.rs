//! `float_equality`: `a == b` or `a != b` between two computed floats.
//!
//! Rounding makes results that are equal on paper differ in the last bits (`0.1 + 0.2 !=
//! 0.3`), so exact comparison of computed values is fragile. Compare the difference with a
//! tolerance instead.
//!
//! Comparisons with a constant (`x == 0`, `t != 1.0`, `v == NAN_SENTINEL`) are not reported:
//! testing for an exact value that was stored is a common, correct use. Neither are `x != x`
//! (the NaN test) or comparisons inside `operator ==` and friends. Off by default
//! (`allow`): "did this value change" checks (`if new != old`) compare exactly on purpose.
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{BinOp, ExprKind as E};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        // Equality operators compare exactly by definition.
        if !p.typed() || p.header.operator.is_some() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(BinOp::Eq | BinOp::Ne, a, b) = &e.kind else {
                return true;
            };
            let float =
                |x| cx.ty(x).is_some_and(|t| cx.compiler.types.is_float(t)) && !cx.constant(x);
            // `x != x` is the NaN test.
            let same =
                crate::syntax::squash(cx.src(a.span)) == crate::syntax::squash(cx.src(b.span));
            if float(a) && float(b) && !same {
                out.push(Finding {
                    start: e.span.start as usize,
                    end: e.span.end as usize,
                    message: "comparing floats for exact equality".into(),
                    label: None,
                    help: Some(
                        "compare the difference with a tolerance: `abs(a - b) < epsilon`".into(),
                    ),
                    fix: None,
                });
            }
            true
        });
    }
}
