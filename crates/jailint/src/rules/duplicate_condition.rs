//! `duplicate_condition`: an `else if` or a `case` that repeats an earlier one.
//!
//! ```jai
//! if key == .LEFT       move(-1);
//! else if key == .RIGHT move(1);
//! else if key == .LEFT  jump();      // never runs
//!
//! if mode == {
//!     case .READ;  open_read();
//!     case .READ;  open_write();     // never runs: `.WRITE` was meant
//! }
//! ```
//!
//! The second branch can never be taken, so whatever it was written for does not happen.
//!
//! Fires when a condition in an `if`/`else if` chain is the same side-effect-free expression
//! (names, member paths, literals, operators; no calls), spelled the same, as an earlier one
//! in the chain; and when a value in an `if x == { case ...; }` (or `#if x == {}`) appears in
//! an earlier `case`, spelled the same.
use super::{finding, same_pure};
use crate::Finding;
use crate::syntax::{Cx, Node, squash, walk};
use jaic::ast::{Expr, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(s) = n.stmt() else {
                return true;
            };
            match &s.kind {
                S::If {
                    ..
                } => {
                    // The head of a chain only: links are handled from there.
                    let link = matches!(chain.last(), Some(Node::Stmt(parent))
                        if matches!(&parent.kind, S::If { else_branch: Some(e), .. } if e.span == s.span));
                    if !link {
                        chain_conditions(cx, s, out);
                    }
                }
                S::Switch {
                    cases, ..
                }
                | S::StaticSwitch {
                    cases, ..
                } => {
                    let mut seen: Vec<String> = Vec::new();
                    for c in cases {
                        for v in &c.values {
                            let text = squash(cx.whole_src(v.span));
                            if seen.contains(&text) {
                                out.push(finding(
                                    v.span,
                                    format!("`case {}` repeats an earlier case", text),
                                    Some("only the first matching case runs: was another value meant?".into()),
                                ));
                            } else {
                                seen.push(text);
                            }
                        }
                    }
                }
                _ => {}
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
