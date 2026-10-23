//! `impossible_comparison`: `x > 5 && x < 3`, `x != 1 || x != 2`, comparisons of one integer
//! that can never pass together, or between them always pass.
//!
//! ```jai
//! if code != 404 || code != 410  forget(url);   // always true: `&&` was meant
//! if n > 10 && n < 5             warn();        // never runs
//! ```
//!
//! Fires on a run of `&&` (or of `||`) whose operands include two or more comparisons of the
//! same side-effect-free integer expression, spelled the same, with integer literals: when no
//! integer passes all of them (`&&`), or every integer passes one of them (`||`). The whole
//! range of integers is used, so the answer holds whatever the expression's width. Floats are
//! left alone: `x > 1 && x < 2` holds for `1.5`.
use super::{finding, int_literal, pure};
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{BinOp, Expr, ExprKind as E};
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, ancestors| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Binary(op @ (BinOp::And | BinOp::Or), ..) = &e.kind else {
                return true;
            };
            // The top of a run of the same operator.
            let inner = matches!(
                ancestors.last().copied().and_then(Node::expr).map(|p| &p.kind),
                Some(E::Binary(outer, ..)) if outer == op
            );
            if !inner {
                run(cx, e, *op, out);
            }
            true
        });
    }
}

/// `value OP literal`, one operand of the run.
struct Test<'a> {
    value: &'a Expr,
    op: BinOp,
    literal: i128,
}

impl Test<'_> {
    fn passes(&self, x: i128) -> bool {
        let c = self.literal;
        match self.op {
            BinOp::Eq => x == c,
            BinOp::Ne => x != c,
            BinOp::Lt => x < c,
            BinOp::Le => x <= c,
            BinOp::Gt => x > c,
            BinOp::Ge => x >= c,
            _ => unreachable!(),
        }
    }
}

fn run(cx: &Cx, e: &Expr, op: BinOp, out: &mut Vec<Finding>) {
    let mut operands = Vec::new();
    flatten(e, op, &mut operands);
    let tests: Vec<Test> = operands.iter().filter_map(|o| test(cx, o)).collect();
    let mut done = vec![false; tests.len()];
    for i in 0..tests.len() {
        if done[i] {
            continue;
        }
        let group: Vec<&Test> = (i..tests.len())
            .filter(|&j| cx.same_tokens(tests[i].value.span, tests[j].value.span))
            .inspect(|&j| done[j] = true)
            .map(|j| &tests[j])
            .collect();
        if group.len() < 2 {
            continue;
        }
        // Each test changes its answer only next to its literal, so these points see every
        // combination of answers there is.
        let mut points = vec![i128::MIN, i128::MAX];
        for t in &group {
            points.extend([t.literal - 1, t.literal, t.literal + 1]);
        }
        let and = op == BinOp::And;
        let impossible = if and {
            !points.iter().any(|&x| group.iter().all(|t| t.passes(x)))
        } else {
            points.iter().all(|&x| group.iter().any(|t| t.passes(x)))
        };
        if !impossible {
            continue;
        }
        let shown = cx.whole_src(e.span).trim();
        let (always, help) = if and {
            (
                "false",
                "no integer passes all of these comparisons: was `||` or another bound meant?",
            )
        } else {
            (
                "true",
                "every integer passes one of these comparisons: was `&&` meant?",
            )
        };
        out.push(finding(
            e.span,
            format!("`{shown}` is always `{always}`"),
            Some(help.into()),
        ));
        return;
    }
}

/// The operands of a run of `op`: `a && b && c` gives `a`, `b`, `c`.
fn flatten<'a>(e: &'a Expr, op: BinOp, out: &mut Vec<&'a Expr>) {
    match &e.kind {
        E::Binary(o, a, b) if *o == op => {
            flatten(a, op, out);
            flatten(b, op, out);
        }
        _ => out.push(e),
    }
}

/// An operand that compares a pure integer expression with an integer literal.
fn test<'a>(cx: &Cx, e: &'a Expr) -> Option<Test<'a>> {
    let E::Binary(op, a, b) = &e.kind else {
        return None;
    };
    if !matches!(
        op,
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
    ) {
        return None;
    }
    let (value, op, literal) = match (int_literal(a), int_literal(b)) {
        (None, Some(c)) => (&**a, *op, c),
        (Some(c), None) => (&**b, flip(*op), c),
        _ => return None,
    };
    // Room for the points around the literal.
    if !pure(value) || literal == i128::MIN || literal == i128::MAX {
        return None;
    }
    let ty = cx.ty(value)?;
    matches!(cx.compiler.ty.types.kind(ty), TypeKind::Int { .. }).then_some(Test {
        value,
        op,
        literal,
    })
}

/// `c OP x` as `x OP' c`.
fn flip(op: BinOp) -> BinOp {
    match op {
        BinOp::Lt => BinOp::Gt,
        BinOp::Gt => BinOp::Lt,
        BinOp::Le => BinOp::Ge,
        BinOp::Ge => BinOp::Le,
        other => other,
    }
}
