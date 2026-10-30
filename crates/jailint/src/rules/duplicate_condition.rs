//! `duplicate_condition`: an `else if` that repeats an earlier condition.
//!
//! ```jai
//! if key == .LEFT       move(-1);
//! else if key == .RIGHT move(1);
//! else if key == .LEFT  jump();      // never runs
//! ```
//!
//! The second branch can never be taken, so whatever it was written for does not happen.
//!
//! Fires when a condition in an `if`/`else if` chain is the same side-effect-free expression
//! (names, member paths, literals, operators; no calls), spelled the same, as an earlier one
//! in the chain. A repeated `case` value in a `switch` is not checked: the compiler rejects it.
use super::{finding, same_pure, statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{Expr, StmtKind as S};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "duplicate_condition",
    category: crate::config::Category::Suspicious,
    summary: "an `else if` that repeats an earlier condition",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(s) = n.stmt() else {
                return true;
            };
            if matches!(s.kind, S::If { .. }) {
                // The head of a chain only: links are handled from there.
                let link = matches!(chain.last(), Some(Node::Stmt(parent))
                    if matches!(&parent.kind, S::If { else_branch: Some(e), .. } if e.span == s.span));
                if !link {
                    chain_conditions(cx, s, out);
                }
            }
            true
        });
    }
}

fn chain_conditions(cx: &Cx, head: &jaic::ast::Stmt, out: &mut Vec<Finding>) {
    let mut earlier: Vec<&Expr> = Vec::new();
    let mut at = head;
    // The branch before the current link: the dead link is cut out from its end.
    let mut before: Option<Span> = None;
    while let S::If {
        cond,
        then_branch,
        else_branch,
    } = &at.kind
    {
        if earlier.iter().any(|e| same_pure(cx, e, cond)) {
            let message = format!(
                "`{}` was already tested earlier in this chain",
                cx.whole_src(cond.span).trim()
            );
            let help = "this branch can never run: was another condition meant?".to_string();
            let cut = before.map(|b| {
                (
                    statement_range(cx, b).1,
                    statement_range(cx, then_branch.span).1,
                )
            });
            out.push(match cut {
                Some((start, end)) if !cx.has_comment_between(start, end) => suggesting(
                    cond.span,
                    message,
                    help,
                    "remove the branch that never runs",
                    vec![Edit {
                        start,
                        end,
                        text: String::new(),
                    }],
                ),
                _ => finding(cond.span, message, Some(help)),
            });
        } else if super::pure(cond) {
            earlier.push(cond);
        } else {
            // A call may change what the earlier conditions read.
            earlier.clear();
        }
        before = Some(then_branch.span);
        match else_branch {
            Some(next) => at = next,
            None => break,
        }
    }
}
