//! `identical_branches`: `if c { A } else { A }`, `ifx c then a else a`.
//!
//! When both branches do the same thing the condition decides nothing. Either one branch was
//! meant to differ (a copied block not edited afterwards), or the `if` can go.
//!
//! Fires when the two branches of an `if`/`else` or an `ifx` are the same text, ignoring
//! whitespace and comments, and are not empty. Only a lone `if`/`else` is compared: the last
//! link of an `else if` chain (`else if f == .LAST { A } else { A }`) usually names its case
//! before the default on purpose. `#if` is left alone: its branches are often the same code
//! written for two platforms.
use super::finding;
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{ExprKind as E, Stmt, StmtKind as S};
use jaic::lexer::Token;
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, up| {
            match n {
                crate::syntax::Node::Stmt(s) => {
                    if let S::If {
                        then_branch,
                        else_branch: Some(other),
                        ..
                    } = &s.kind
                        && !matches!(other.kind, S::If { .. })
                        && !empty(then_branch)
                        && !chain_tail(s, up)
                        && same_tokens(cx, then_branch.span, other.span)
                    {
                        out.push(finding(
                            s.span,
                            "both branches of this `if` are the same".into(),
                            Some(
                                "the condition decides nothing: was one branch meant to differ?"
                                    .into(),
                            ),
                        ));
                    }
                }
                crate::syntax::Node::Expr(e) => {
                    if let E::Ifx {
                        then_value: Some(a),
                        else_value: Some(b),
                        is_static: false,
                        ..
                    } = &e.kind
                        && same_tokens(cx, a.span, b.span)
                    {
                        out.push(finding(
                            e.span,
                            "both branches of this `ifx` are the same".into(),
                            Some(
                                "the condition decides nothing: was one branch meant to differ?"
                                    .into(),
                            ),
                        ));
                    }
                }
            }
            true
        });
    }
}

/// `s` is the `else if` that ends a longer chain. `... else if x == LAST { A } else { A }`
/// spells out the last case before the default on purpose, so it is left alone.
fn chain_tail(s: &Stmt, up: &[crate::syntax::Node]) -> bool {
    matches!(up.last(), Some(crate::syntax::Node::Stmt(parent))
        if matches!(&parent.kind, S::If { else_branch: Some(e), .. } if std::ptr::eq(&**e, s)))
}

fn empty(s: &Stmt) -> bool {
    match &s.kind {
        S::Block(b) => b.stmts.iter().all(|s| matches!(s.kind, S::Empty)),
        S::Empty => true,
        _ => false,
    }
}

/// The tokens of `a` and `b` (comments and layout are not tokens) are the same.
fn same_tokens(cx: &Cx, a: Span, b: Span) -> bool {
    let (a, b) = (tokens(cx, a), tokens(cx, b));
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| cx.src(x.span) == cx.src(y.span))
}

fn tokens<'a>(cx: &Cx<'a>, span: Span) -> &'a [Token] {
    let (start, end) = cx.whole(span);
    let first = cx
        .tokens
        .partition_point(|t| (t.span.start as usize) < start);
    let last = cx.tokens.partition_point(|t| (t.span.end as usize) <= end);
    &cx.tokens[first..last.max(first)]
}
