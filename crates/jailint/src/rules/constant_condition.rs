//! `constant_condition`: an `if` or `while` whose condition the compiler can show always has
//! the same value, so one branch never runs.
//!
//! ```jai
//! if count > 10 {
//!     if count < 5 { ... }      // never: `count` is at least 11 here
//! }
//! ```
//!
//! The compiler already works out which code can never run (the editor fades it, see
//! `jaic::sema::reachability` and `jaic::ide::values`): from the range of values each operand can take, from what an
//! enclosing test narrowed, and from the width of each type. This rule reports the test behind
//! such a branch. It adds no analysis of its own; where the compiler is unsure, so is the rule.
//!
//! Left alone: conditions that are compile-time constants or combine one with `&&`/`||` (`if DEBUG`,
//! `while true`, `if flag && TARGET_OS != .WINDOWS`: written on purpose), tests inside code that is dead anyway, and comparisons with a literal that
//! `absurd_comparison` already reports.
use super::{finding, int_literal};
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::ide::interval::{self, Interval};
use jaic::source::Span;
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    let Some(ide) = cx.compiler.ed.facts() else {
        return;
    };
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        // The runs of this body every analysis of it found dead.
        let evals = ide.body_evals.get(&p.body.span).copied();
        let dead: Vec<Span> = ide
            .dead_runs
            .iter()
            .filter(|((body, _), n)| *body == p.body.span && Some(**n) == evals)
            .map(|((_, run), _)| *run)
            .collect();
        if dead.is_empty() {
            continue;
        }
        let covers =
            |outer: Span, inner: Span| outer.start <= inner.start && inner.end <= outer.end;
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            if dead.iter().any(|&run| covers(run, s.span)) {
                // Dead already: the report belongs to whatever made it so.
                return false;
            }
            let (cond, then, other) = match &s.kind {
                S::If {
                    cond,
                    then_branch,
                    else_branch,
                } => (&**cond, &**then_branch, else_branch.as_deref()),
                S::While(w) => (&w.cond, &*w.body, None),
                _ => return true,
            };
            if constant_part(cx, cond) || decided_by_type(cx, cond) {
                return true;
            }
            let never = |branch: &Stmt| dead.contains(&contents(branch));
            let (value, what) = if never(then) {
                (false, "the code it guards never runs")
            } else if other.is_some_and(never) {
                (true, "its `else` never runs")
            } else {
                return true;
            };
            out.push(finding(
                cond.span,
                format!("this condition is always `{value}`"),
                Some(format!(
                    "{what}: the values its operands can take here decide it; remove the test or fix the logic around it"
                )),
            ));
            true
        });
    }
}

/// `cond` is a compile-time constant, or a `&&`/`||` with one as an operand: the target or a
/// build setting decides it, so it is constant only on some builds.
fn constant_part(cx: &Cx, cond: &Expr) -> bool {
    if cx.constant(cond) || matches!(cond.kind, E::Bool(_)) {
        return true;
    }
    match &cond.kind {
        E::Binary(jaic::ast::BinOp::And | jaic::ast::BinOp::Or, a, b) => {
            constant_part(cx, a) || constant_part(cx, b)
        }
        _ => false,
    }
}

/// What the compiler marks dead when a branch never runs: the statements of a block, without
/// its braces.
fn contents(branch: &Stmt) -> Span {
    match &branch.kind {
        S::Block(b) => match (b.stmts.first(), b.stmts.last()) {
            (Some(first), Some(last)) => first.span.to(last.span),
            _ => branch.span,
        },
        _ => branch.span,
    }
}

/// `u >= 0` and the like: the type alone decides them, and `absurd_comparison` says so.
fn decided_by_type(cx: &Cx, cond: &Expr) -> bool {
    let E::Binary(op, a, b) = &cond.kind else {
        return false;
    };
    let (value, literal) = match (int_literal(a), int_literal(b)) {
        (None, Some(c)) => (a, c),
        (Some(c), None) => (b, c),
        _ => return false,
    };
    let Some(ty) = cx.ty(value) else {
        return false;
    };
    let TypeKind::Int {
        bits,
        signed,
    } = *cx.compiler.ty.types.kind(ty)
    else {
        return false;
    };
    let Some(range) = Interval::of_type(bits, signed) else {
        return false;
    };
    let (x, y) = if int_literal(a).is_some() {
        (Interval::point(literal), range)
    } else {
        (range, Interval::point(literal))
    };
    interval::compare(*op, x, y).is_some()
}
