//! `remove_in_for`: `array_unordered_remove_by_index(*xs, it_index)` inside `for xs`.
//!
//! Removing from the array a `for` is walking forward moves another element into the slot just
//! visited (unordered) or shifts the rest down by one (ordered), and the loop then steps past
//! it: that element is never visited. `remove it;` removes the current element and keeps the
//! iteration in step.
//!
//! Fires on a call to a procedure declared with the note `@RemovesFromArray` (`Basic`'s
//! `array_*_remove_*` procedures carry it) whose array argument is
//! `*xs` (spelled the same) inside a forward `for xs` / `for *xs` over the same array, unless
//! the call is directly followed by `break` or `return` (removing the one element searched
//! for and stopping is fine), or the loop body assigns its index (`i -= 1;` after the removal
//! steps back by hand).
use super::{calls_noted, finding, statement_lists, suggesting};
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding};
use jaic::ast::{Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "remove_in_for",
    category: crate::config::Category::Correctness,
    summary: "removing from an array while a `for` walks it forward",
    check,
};

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
            if !calls_noted(cx, callee, "RemovesFromArray") {
                return true;
            }
            let walking = chain.iter().rev().find_map(|a| match a {
                Node::Stmt(Stmt {
                    kind: S::For(f),
                    ..
                }) => {
                    let index = f.index.map_or("it_index", |i| i.name.as_str());
                    (!f.reverse
                        && !assigns_index(&f.body, index)
                        && f.reverse_if.is_none()
                        && f.iterator.is_none()
                        && matches!(&f.over, ForOver::Collection(c) if cx.same_tokens(c.span, array.span)))
                    .then_some((f.it.map_or("it", |i| i.name.as_str()), index))
                }
                _ => None,
            });
            let Some((it, index)) = walking else {
                return true;
            };
            if stopped(chain, e, &stopping) {
                return true;
            }
            let name = cx.whole_src(callee.span).trim();
            let array_shown = cx.whole_src(array.span).trim();
            let message = format!("`{name}` changes `{array_shown}` while this `for` walks it");
            let help = format!(
                "the element moved into the freed slot is skipped; use `remove {it};` (or loop backwards with `for < {array_shown}`)"
            );
            // The call is a statement on its own that removes the element being visited.
            let removes_current = args.len() == 2
                && matches!(args[1].value.kind, E::Ident(n) if n.as_str() == it || n.as_str() == index)
                && matches!(chain.last(), Some(Node::Stmt(s)) if matches!(&s.kind, S::Expr(x) if x.span == e.span));
            out.push(if removes_current {
                suggesting(
                    e.span,
                    message,
                    help,
                    format!("use `remove {it}`"),
                    vec![Edit {
                        start: e.span.start as usize,
                        end: e.span.end as usize,
                        text: format!("remove {it}"),
                    }],
                )
            } else {
                finding(e.span, message, Some(help))
            });
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
