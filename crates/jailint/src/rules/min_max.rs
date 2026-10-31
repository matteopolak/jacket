//! `min_max`: `min(0, max(100, x))`: a clamp with its bounds swapped.
//!
//! `max(100, x)` is at least 100, so `min(0, ...)` of it is always 0. The intent was
//! `clamp(x, 0, 100)` (or `max(0, min(100, x))`).
//!
//! Fires on `min(a, max(b, x))` with literal bounds `a < b`, and `max(a, min(b, x))` with
//! `a > b`, in either argument order, when the outer and inner calls are to procedures
//! declared with the notes `@ReturnsSmaller` and `@ReturnsLarger` (the library's `min` and `max` carry
//! them, whatever they are called where they are used).
use super::{calls_noted, int_literal, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::walk;
use jaic::ast::{Expr, ExprKind as E};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "min_max",
    category: crate::config::Category::Correctness,
    summary: "`min(0, max(100, x))`, a clamp that is always one bound",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(e) = n.expr() else {
                return true;
            };
            let Some((outer, [x, y])) = two_arg_call(cx, e) else {
                return true;
            };
            let (bound, inner) = match (literal(x), literal(y)) {
                (Some(c), None) => (c, y),
                (None, Some(c)) => (c, x),
                _ => return true,
            };
            let Some((inner_name, [p, q])) = two_arg_call(cx, inner) else {
                return true;
            };
            let ((Some(other), None) | (None, Some(other))) = (literal(p), literal(q)) else {
                return true;
            };
            let always = match (outer, inner_name) {
                ("min", "max") => bound < other,
                ("max", "min") => bound > other,
                _ => false,
            };
            if always {
                let (lo, hi) = (bound.min(other), bound.max(other));
                let outer_literal = if literal(x).is_some() {
                    x
                } else {
                    y
                };
                let (inner_literal, value) = if literal(p).is_some() {
                    (p, q)
                } else {
                    (q, p)
                };
                let (lo_expr, hi_expr) = if bound <= other {
                    (outer_literal, inner_literal)
                } else {
                    (inner_literal, outer_literal)
                };
                let (lo_text, hi_text) =
                    (cx.whole_src(lo_expr).trim(), cx.whole_src(hi_expr).trim());
                let value = cx.whole_src(value).trim();
                out.push(suggesting(
                    e.span,
                    format!(
                        "this is always `{}`: the bounds are swapped",
                        cx.whole_src(match literal(x) {
                            Some(_) => x,
                            None => y,
                        })
                        .trim()
                    ),
                    format!(
                        "to keep a value between {lo} and {hi}, write `clamp(value, {lo}, {hi})`"
                    ),
                    "clamp between the two bounds",
                    vec![Edit {
                        start: e.span.start as usize,
                        end: e.span.end as usize,
                        text: format!("clamp({value}, {lo_text}, {hi_text})"),
                    }],
                ));
            }
            true
        });
    }
}

/// An integer or float literal's value.
pub(super) fn literal(e: &Expr) -> Option<f64> {
    match &e.kind {
        E::Float(v, ..) => Some(*v),
        E::Unary(jaic::ast::UnOp::Neg, inner) => literal(inner).map(|v| -v),
        _ => int_literal(e).map(|v| v as f64),
    }
}

/// A call of a `@ReturnsSmaller` or `@ReturnsLarger` procedure: `min` or `max`, and the two arguments.
pub(super) fn two_arg_call<'e>(cx: &Cx, e: &'e Expr) -> Option<(&'static str, [&'e Expr; 2])> {
    let E::Call {
        callee,
        args,
        ..
    } = &e.kind
    else {
        return None;
    };
    let [a, b] = args.as_slice() else {
        return None;
    };
    if a.name.is_some() || b.name.is_some() || a.spread || b.spread {
        return None;
    }
    let name = [("min", "ReturnsSmaller"), ("max", "ReturnsLarger")]
        .into_iter()
        .find(|(_, note)| calls_noted(cx, callee, note))?
        .0;
    Some((name, [&a.value, &b.value]))
}
