//! `float_loop_counter`: `while t < 1.0 { ...; t += 0.1; }`, a loop counted in floats.
//!
//! Adding `0.1` ten times does not give `1.0`, so whether the loop runs ten or eleven times
//! depends on rounding error, and the error grows with the iteration count. Count with an
//! integer and compute the float from it (`for i: 0..9 { t := i * 0.1; ... }`).
//!
//! `for` loops cannot count in floats (the compiler rejects a float range), so this looks at
//! `while`: the condition compares a float variable with another expression, and the body adds
//! to or subtracts from that variable (`t += dt`, `t = t - dt`) a step that does not mention
//! it. Loops that update the variable some other way (a convergence loop: `x = x - f(x) / d`)
//! are not counting and are left alone. Off by default (`allow`): a loop that drains a float
//! budget by a computed amount reads fine this way. No fix.
use crate::Finding;
use crate::syntax::{Cx, Node, walk, walk_node};
use jaic::ast::{AssignOp, BinOp, Expr, ExprKind as E, StmtKind as S};
use jaic::intern::Sym;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "float_loop_counter",
    category: crate::config::Category::Pedantic,
    summary: "a `while` loop counted in floats (`while t < 1.0 { t += 0.1; }`)",
    check,
};

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
            let S::While(w) = &s.kind else {
                return true;
            };
            let E::Binary(BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Ne, a, b) =
                &w.cond.kind
            else {
                return true;
            };
            for (counter, other) in [(a, b), (b, a)] {
                let E::Ident(name) = counter.kind else {
                    continue;
                };
                if !cx.ty(counter).is_some_and(|t| types.is_float(t))
                    || cx.constant(counter)
                    || mentions(other, name)
                    || !steps(&w.body, name)
                {
                    continue;
                }
                out.push(super::finding(
                    counter.span,
                    format!(
                        "`{}` counts this loop in floats, so rounding error decides how many times it runs",
                        name.as_str()
                    ),
                    Some(
                        "count with an integer and compute the float from it: `for i: 0..n-1 { x := start + i * step; }`"
                            .into(),
                    ),
                ));
                break;
            }
            true
        });
    }
}

/// The expression names `name`.
fn mentions(e: &Expr, name: Sym) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Expr(e), &mut stack, &mut |n, _| {
        if let Some(x) = n.expr()
            && matches!(x.kind, E::Ident(i) if i == name)
        {
            found = true;
        }
        !found
    });
    found
}

/// The body adds a step to `name`, or takes one from it.
fn steps(body: &jaic::ast::Stmt, name: Sym) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(body), &mut stack, &mut |n, _| {
        if let Some(s) = n.stmt()
            && let S::Assign {
                op,
                lhs,
                rhs,
            } = &s.kind
            && let ([target], [value]) = (lhs.as_slice(), rhs.as_slice())
            && matches!(target.kind, E::Ident(i) if i == name)
        {
            found |= match op {
                AssignOp::Op(BinOp::Add | BinOp::Sub) => !mentions(value, name),
                AssignOp::Assign => matches!(
                    &value.kind,
                    E::Binary(BinOp::Add | BinOp::Sub, l, r)
                        if matches!(l.kind, E::Ident(i) if i == name) && !mentions(r, name)
                ),
                AssignOp::Op(_) => false,
            };
        }
        !found
    });
    found
}
