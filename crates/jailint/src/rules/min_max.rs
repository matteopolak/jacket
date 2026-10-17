//! `min_max`: `min(0, max(100, x))`: a clamp with its bounds swapped.
//!
//! `max(100, x)` is at least 100, so `min(0, ...)` of it is always 0. The intent was
//! `clamp(x, 0, 100)` (or `max(0, min(100, x))`).
//!
//! Fires on `min(a, max(b, x))` with literal bounds `a < b`, and `max(a, min(b, x))` with
//! `a > b`, in either argument order, when the outer and inner calls are to procedures
//! declared with the notes `@Minimum` and `@Maximum` (the library's `min` and `max` carry
//! them, whatever they are called where they are used).
use super::{calls_noted, finding, int_literal};
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{Expr, ExprKind as E};

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
            let other = match (literal(p), literal(q)) {
                (Some(c), None) | (None, Some(c)) => c,
                _ => return true,
            };
            let always = match (outer, inner_name) {
                ("min", "max") => bound < other,
                ("max", "min") => bound > other,
                _ => false,
            };
            if always {
                let (lo, hi) = (bound.min(other), bound.max(other));
                out.push(finding(
                    e.span,
                    format!(
                        "this is always `{}`: the bounds are swapped",
                        cx.whole_src(match literal(x) {
                            Some(_) => x.span,
                            None => y.span,
                        })
                        .trim()
                    ),
                    Some(format!(
                        "to keep a value between {lo} and {hi}, write `clamp(value, {lo}, {hi})`"
                    )),
                ));
            }
            true
        });
    }
}

/// An integer or float literal's value.
fn literal(e: &Expr) -> Option<f64> {
    match &e.kind {
        E::Float(v) => Some(*v),
        E::Unary(jaic::ast::UnOp::Neg, inner) => literal(inner).map(|v| -v),
        _ => int_literal(e).map(|v| v as f64),
    }
}

/// A call of a `@Minimum` or `@Maximum` procedure: `min` or `max`, and the two arguments.
fn two_arg_call<'e>(cx: &Cx, e: &'e Expr) -> Option<(&'static str, [&'e Expr; 2])> {
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
    let name = [("min", "Minimum"), ("max", "Maximum")]
        .into_iter()
        .find(|(_, note)| calls_noted(cx, callee, note))?
        .0;
    Some((name, [&a.value, &b.value]))
}
