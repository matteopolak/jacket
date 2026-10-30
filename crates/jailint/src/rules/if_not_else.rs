//! `if_not_else`: an `if` with a negated condition and an `else`.
//!
//! ```jai
//! if !ready { wait(); } else { go(); }
//! if a != b { differ(); } else { same(); }
//! ```
//!
//! Testing the positive condition first (`if ready { go(); } else { wait(); }`) saves the
//! reader one negation. Off by default: there are good reasons to put the short or the
//! exceptional branch first.
//!
//! Fires on an `if` (or an `ifx`) whose condition is `!c`, or `a != b` for numbers, pointers,
//! enums and strings, and that has a plain `else` (an `else if` chain is left alone). The
//! editor offers the swap when both branches are blocks, or both `ifx` values; `--fix` leaves
//! it.
use super::needless_bool::condition;
use super::{finding, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::{BinOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "if_not_else",
    category: crate::config::Category::Pedantic,
    summary: "an `if` or `ifx` with a negated condition and an `else`",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, _| match n {
        crate::syntax::Node::Stmt(s) => {
            if let S::If {
                cond,
                then_branch,
                else_branch: Some(other),
            } = &s.kind
                && !matches!(other.kind, S::If { .. })
                && let Some(positive) = positive(cx, cond)
            {
                let header = Span::new(cx.file, s.span.start as usize, cx.whole(cond.span).1);
                let fixable = matches!(then_branch.kind, S::Block(_))
                    && matches!(other.kind, S::Block(_))
                    && !cx.has_comment_between(cond.span.end as usize, s.span.end as usize);
                out.push(report(
                    cx,
                    header,
                    &positive,
                    fixable.then(|| swap(cx, cond, then_branch, other, &positive)),
                ));
            }
        }
        crate::syntax::Node::Expr(e) => {
            if let E::Ifx {
                cond,
                then_value: Some(a),
                else_value: Some(b),
                is_static: false,
            } = &e.kind
                && let Some(positive) = positive(cx, cond)
            {
                let header = Span::new(cx.file, e.span.start as usize, cx.whole(cond.span).1);
                let fixable = !cx.has_comment_between(e.span.start as usize, e.span.end as usize);
                let edits = |a: &Expr, b: &Expr| {
                    vec![
                        replace(cx, cond.span, &positive),
                        replace(cx, a.span, cx.whole_src(b.span).trim()),
                        replace(cx, b.span, cx.whole_src(a.span).trim()),
                    ]
                };
                out.push(report(cx, header, &positive, fixable.then(|| edits(a, b))));
            }
        }
    });
}

fn report(cx: &Cx, header: Span, positive: &str, edits: Option<Vec<Edit>>) -> Finding {
    let message = format!("`{}` is negated, with an `else`", cx.src(header));
    let help = format!("test `{positive}` first and swap the branches");
    match edits {
        Some(edits) => suggesting(header, message, help, "swap the branches", edits),
        None => finding(header, message, Some(help)),
    }
}

fn replace(cx: &Cx, span: Span, text: &str) -> Edit {
    let (start, end) = cx.whole(span);
    Edit {
        start,
        end,
        text: text.to_string(),
    }
}

/// The three edits that swap the branches and drop the negation of the test.
fn swap(cx: &Cx, cond: &Expr, a: &Stmt, b: &Stmt, positive: &str) -> Vec<Edit> {
    let text = |s: &Stmt| cx.text[s.span.start as usize..s.span.end as usize].to_string();
    let edit = |s: &Stmt, text: String| Edit {
        start: s.span.start as usize,
        end: s.span.end as usize,
        text,
    };
    vec![
        replace(cx, cond.span, positive),
        edit(a, text(b)),
        edit(b, text(a)),
    ]
}

/// The condition without its negation, if it has one that can go: `!c`, or `a != b` of types
/// whose `==` is certain to exist.
fn positive(cx: &Cx, cond: &Expr) -> Option<String> {
    match &cond.kind {
        E::Unary(UnOp::Not, inner) => {
            let text = cx.whole_src(inner.span).trim();
            Some(if inner.parenthesized {
                text[1..text.len() - 1].trim().to_string()
            } else {
                text.to_string()
            })
        }
        E::Binary(BinOp::Ne, ..) => {
            let flipped = condition(cx, cond, true, false);
            // Only a comparison that really turned into `==` is simpler.
            (!flipped.starts_with('!')).then_some(flipped)
        }
        _ => None,
    }
}
