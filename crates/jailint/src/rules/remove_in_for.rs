//! `remove_in_for`: `array_unordered_remove_by_index(*xs, it_index)` inside `for xs`.
//!
//! Removing from the array a `for` is walking forward moves another element into the slot just
//! visited (unordered) or shifts the rest down by one (ordered), and the loop then steps past
//! it: that element is never visited. `remove it;` removes the current element and keeps the
//! iteration in step.
//!
//! Fires on a call to one of `Basic`'s `array_*_remove_*` procedures whose array argument is
//! `*xs` (spelled the same) inside a forward `for xs` / `for *xs` over the same array, unless
//! the call is directly followed by `break` or `return` (removing the one element searched
//! for and stopping is fine), or the loop body assigns its index (`i -= 1;` after the removal
//! steps back by hand).
use super::{calls_library, finding, statement_lists};
use crate::Finding;
use crate::syntax::{Cx, Node, squash, walk};
use jaic::ast::{Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp};
use jaic::source::Span;

const REMOVERS: &[&str] = &[
    "array_unordered_remove_by_index",
    "array_ordered_remove_by_index",
    "array_unordered_remove_by_value",
    "array_ordered_remove_by_value",
];

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean {
            continue;
        }
        // Calls followed directly by `break` or `return`.
        let mut stopping: Vec<Span> = Vec::new();
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            for pair in list.windows(2) {
                if matches!(pair[1].kind, S::Break(_) | S::Return { .. }) {
                    stopping.push(pair[0].span);
                }
            }
        });
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(e) = n.expr() else {
                return true;
            };
            let E::Call {
                callee,
                args,
                ..
            } = &e.kind
            else {
                return true;
            };
            let Some(first) = args.first() else {
                return true;
            };
            let E::Unary(UnOp::Star, array) = &first.value.kind else {
                return true;
            };
            if !calls_library(cx, callee, &["Basic"], REMOVERS) {
                return true;
            }
            let array_text = squash(cx.whole_src(array.span));
            let walking = chain.iter().any(|a| match a {
                Node::Stmt(Stmt {
                    kind: S::For(f),
                    ..
                }) => {
                    let index = f.index.map_or("it_index", |i| i.name.as_str());
                    !f.reverse
                        && !assigns_index(&f.body, index)
                        && f.reverse_if.is_none()
                        && f.iterator.is_none()
                        && matches!(&f.over, ForOver::Collection(c) if squash(cx.whole_src(c.span)) == array_text)
                }
                _ => false,
            });
            if !walking || stopped(chain, e, &stopping) {
                return true;
            }
            let name = cx.whole_src(callee.span).trim();
            let array_shown = cx.whole_src(array.span).trim();
            out.push(finding(
                e.span,
                format!("`{name}` changes `{array_shown}` while this `for` walks it"),
                Some(format!(
                    "the element moved into the freed slot is skipped; use `remove it;` (or loop backwards with `for < {array_shown}`)"
                )),
            ));
            true
        });
    }
}

/// The call is the whole statement directly before a `break` or `return`.
fn stopped(chain: &[Node<'_>], call: &Expr, stopping: &[Span]) -> bool {
    matches!(chain.last(), Some(Node::Stmt(s)) if matches!(&s.kind, S::Expr(x) if x.span == call.span) && stopping.contains(&s.span))
}

/// The loop body assigns its index variable.
fn assigns_index(body: &Stmt, index: &str) -> bool {
    super::any_stmt(body, &mut |s| {
        matches!(&s.kind, S::Assign { lhs, .. }
            if lhs.iter().any(|l| matches!(l.kind, E::Ident(n) if n.as_str() == index)))
    })
}
