//! `manual_clamp`: keeping a value between two bounds by hand instead of with `clamp`.
//!
//! ```jai
//! x = max(0, min(100, x));
//! if x < 0 { x = 0; } else if x > 100 { x = 100; }
//! y := ifx v < lo then lo else ifx v > hi then hi else v;
//! ```
//!
//! `clamp(x, 0, 100)` names the intent once and cannot get a bound wrong in one of the two
//! places.
//!
//! Fires on three shapes, all of numbers whose bounds are constants with the lower one not above
//! the upper (otherwise the two forms differ):
//!
//! - a call of the `@ReturnsLarger` procedure around a call of the `@ReturnsSmaller` one (or the
//!   other way round), with the value as the inner call's other argument;
//! - an `if` that assigns a side-effect-free target its lower bound when it is below it, with an
//!   `else if` doing the same for the upper bound (in either order);
//! - an `ifx` that picks the lower bound, then the upper bound, then the value.
//!
//! The editor offers the `clamp` call; `--fix` leaves it, since `clamp` needs `Basic` in scope
//! and a statement may sit inside a chain that the rewrite would break.
use super::min_max::{literal, two_arg_call};
use super::{pure, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, Stmt, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "manual_clamp",
    category: crate::config::Category::Complexity,
    summary: "a value kept between two constant bounds by hand instead of with `clamp`",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, up| {
            match n {
                Node::Expr(e) => {
                    if let Some(found) = calls(cx, e).or_else(|| ifx(cx, e)) {
                        report(cx, e.span, found, true, out);
                    }
                }
                Node::Stmt(s) => {
                    if let Some(found) = chain(cx, s) {
                        // Inside an `else if` chain the replacement would need its own braces.
                        let editable = !matches!(up.last(), Some(Node::Stmt(parent))
                            if matches!(&parent.kind, S::If { else_branch: Some(e), .. }
                                if std::ptr::eq(&raw const **e, s)));
                        report(cx, s.span, found, editable, out);
                    }
                }
            }
            true
        });
    }
}

/// What a clamp keeps between which bounds.
struct Clamp<'a> {
    value: &'a Expr,
    lo: &'a Expr,
    hi: &'a Expr,
    /// The code is an assignment to `value`.
    assigns: bool,
}

fn report(cx: &Cx, span: jaic::source::Span, found: Clamp, editable: bool, out: &mut Vec<Finding>) {
    let text = |e: &Expr| cx.whole_src(e).trim();
    let call = format!(
        "clamp({}, {}, {})",
        text(found.value),
        text(found.lo),
        text(found.hi)
    );
    let replacement = if found.assigns {
        format!("{} = {call};", text(found.value))
    } else {
        call.clone()
    };
    let message = format!(
        "`{}` is kept between `{}` and `{}` by hand",
        text(found.value),
        text(found.lo),
        text(found.hi)
    );
    let help = format!("write `{call}`");
    let mut finding = suggesting(
        span,
        message,
        help,
        format!("write `{call}`"),
        vec![Edit {
            start: span.start as usize,
            end: span.end as usize,
            text: replacement,
        }],
    );
    if !editable || cx.has_comment_between(span.start as usize, span.end as usize) {
        finding.fix = None;
    }
    out.push(finding);
}

/// The bounds are constants, with the lower not above the upper.
fn ordered(cx: &Cx, lo: &Expr, hi: &Expr) -> bool {
    if !cx.constant(lo) || !cx.constant(hi) {
        return false;
    }
    match (literal(lo), literal(hi)) {
        (Some(a), Some(b)) => a <= b,
        _ => true,
    }
}

fn is_number(cx: &Cx, e: &Expr) -> bool {
    let types = &cx.compiler.ty.types;
    cx.ty(e)
        .is_some_and(|t| types.is_integer(t) || types.is_float(t))
}

/// `max(lo, min(hi, x))` and `min(hi, max(lo, x))`.
fn calls<'a>(cx: &Cx, e: &'a Expr) -> Option<Clamp<'a>> {
    let (outer, [p, q]) = two_arg_call(cx, e)?;
    // The outer call has the inner call as one argument and the bound as the other.
    for (bound, inner) in [(p, q), (q, p)] {
        let Some((inner_name, [r, s])) = two_arg_call(cx, inner) else {
            continue;
        };
        if inner_name == outer {
            continue;
        }
        for (other, value) in [(r, s), (s, r)] {
            let (lo, hi) = if outer == "max" {
                (bound, other)
            } else {
                (other, bound)
            };
            if ordered(cx, lo, hi) && is_number(cx, value) {
                return Some(Clamp {
                    value,
                    lo,
                    hi,
                    assigns: false,
                });
            }
        }
    }
    None
}

/// Which bound a comparison tests a value against, and what it tests.
#[derive(Clone, Copy, PartialEq)]
enum Side {
    Lower,
    Upper,
}

/// The readings of `cond` as "`value` is past a bound": the value, the bound and which side.
fn readings(cond: &Expr) -> Vec<(&Expr, &Expr, Side)> {
    let E::Binary(op, a, b) = &cond.kind else {
        return Vec::new();
    };
    match op {
        BinOp::Lt | BinOp::Le => vec![(a, b, Side::Lower), (b, a, Side::Upper)],
        BinOp::Gt | BinOp::Ge => vec![(a, b, Side::Upper), (b, a, Side::Lower)],
        _ => Vec::new(),
    }
}

/// The bound and the value a pair of tests clamp, when `first` tests one side and `second`
/// the other, and both test the same side-effect-free value.
fn tests<'a>(
    cx: &Cx,
    first: &'a Expr,
    second: &'a Expr,
) -> Vec<(&'a Expr, &'a Expr, &'a Expr, Side)> {
    let mut found = Vec::new();
    for (v1, b1, s1) in readings(first) {
        for (v2, b2, s2) in readings(second) {
            if s1 != s2 && pure(v1) && cx.same_tokens(v1, v2) {
                found.push((v1, b1, b2, s1));
            }
        }
    }
    found
}

/// `lo` and `hi` of a pair where `first` tests `side1`.
fn bounds<'a>(side1: Side, b1: &'a Expr, b2: &'a Expr) -> (&'a Expr, &'a Expr) {
    if side1 == Side::Lower {
        (b1, b2)
    } else {
        (b2, b1)
    }
}

/// `if x < lo { x = lo; } else if x > hi { x = hi; }`, in either order.
fn chain<'a>(cx: &Cx, s: &'a Stmt) -> Option<Clamp<'a>> {
    let S::If {
        cond,
        then_branch,
        else_branch: Some(other),
    } = &s.kind
    else {
        return None;
    };
    let S::If {
        cond: cond2,
        then_branch: then2,
        else_branch: None,
    } = &other.kind
    else {
        return None;
    };
    let (target, set1) = assignment(then_branch)?;
    let (target2, set2) = assignment(then2)?;
    if !cx.same_tokens(target, target2) {
        return None;
    }
    for (value, b1, b2, side1) in tests(cx, cond, cond2) {
        // Each branch assigns the value its own bound.
        if cx.same_tokens(value, target) && cx.same_tokens(set1, b1) && cx.same_tokens(set2, b2) {
            let (lo, hi) = bounds(side1, b1, b2);
            if ordered(cx, lo, hi) && is_number(cx, value) {
                return Some(Clamp {
                    value,
                    lo,
                    hi,
                    assigns: true,
                });
            }
        }
    }
    None
}

/// `x = v;`, alone or as the only statement of a block: the target and the value.
fn assignment(s: &Stmt) -> Option<(&Expr, &Expr)> {
    match &s.kind {
        S::Assign {
            op: AssignOp::Assign,
            lhs,
            rhs,
        } => match (lhs.as_slice(), rhs.as_slice()) {
            ([target], [value]) => Some((target, value)),
            _ => None,
        },
        S::Block(b) => {
            let mut stmts = b.stmts.iter().filter(|s| !matches!(s.kind, S::Empty));
            let only = stmts.next()?;
            if stmts.next().is_some() {
                return None;
            }
            assignment(only)
        }
        _ => None,
    }
}

/// `ifx v < lo then lo else ifx v > hi then hi else v`, in either order.
fn ifx<'a>(cx: &Cx, e: &'a Expr) -> Option<Clamp<'a>> {
    let E::Ifx {
        cond,
        then_value: Some(pick1),
        else_value: Some(rest),
        is_static: false,
    } = &e.kind
    else {
        return None;
    };
    let E::Ifx {
        cond: cond2,
        then_value: Some(pick2),
        else_value: Some(last),
        is_static: false,
    } = &rest.kind
    else {
        return None;
    };
    for (value, b1, b2, side1) in tests(cx, cond, cond2) {
        if cx.same_tokens(value, last) && cx.same_tokens(pick1, b1) && cx.same_tokens(pick2, b2) {
            let (lo, hi) = bounds(side1, b1, b2);
            if ordered(cx, lo, hi) && is_number(cx, value) {
                return Some(Clamp {
                    value,
                    lo,
                    hi,
                    assigns: false,
                });
            }
        }
    }
    None
}
