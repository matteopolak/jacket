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
use super::{finding, same_pure};
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{Expr, StmtKind as S};

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
    while let S::If {
        cond,
        else_branch,
        ..
    } = &at.kind
    {
        if earlier.iter().any(|e| same_pure(cx, e, cond)) {
            out.push(finding(
                cond.span,
                format!(
                    "`{}` was already tested earlier in this chain",
                    cx.whole_src(cond.span).trim()
                ),
                Some("this branch can never run: was another condition meant?".into()),
            ));
        } else if super::pure(cond) {
            earlier.push(cond);
        } else {
            // A call may change what the earlier conditions read.
            earlier.clear();
        }
        match else_branch {
            Some(next) => at = next,
            None => break,
        }
    }
}
