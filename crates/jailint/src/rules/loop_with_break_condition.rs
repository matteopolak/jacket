//! `loop_with_break_condition`: `while true { if !c break; ... }` is `while c { ... }`.
//!
//! ```jai
//! while true {
//!     if i >= count break;
//!     ...
//! }
//! ```
//!
//! A loop that opens by leaving when its condition fails has that condition in the wrong place:
//! the `while` header is where a reader looks for it.
//!
//! Fires on a `while true` whose body starts with an `if` that has no `else` and only an
//! unlabelled `break`, in or out of braces. The fix moves the negated test into the header
//! (`!c` becomes `c`, a comparison of numbers, pointers or enums turns into the opposite one) and
//! removes the `if`; it is machine-applicable unless a comment sits inside the `if` or before it in the loop. Macros are
//! left alone.
use super::needless_bool::condition;
use crate::syntax::{Cx, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::While(w) = &s.kind else {
                return true;
            };
            if w.bind_label || !matches!(w.cond.kind, E::Bool(true)) {
                return true;
            }
            let S::Block(body) = &w.body.kind else {
                return true;
            };
            let Some(first) = body.stmts.iter().find(|s| !matches!(s.kind, S::Empty)) else {
                return true;
            };
            let S::If {
                cond,
                then_branch,
                else_branch: None,
            } = &first.kind
            else {
                return true;
            };
            if !only_break(then_branch) {
                return true;
            }
            let keep = negated(cx, cond);
            let (start, mut end) = cx.statement_removal(first.span);
            // The blank line that set the test apart goes with it.
            if cx.line_start(start).0 == start {
                while let Some(len) = blank_line(&cx.text[end..]) {
                    end += len;
                }
            }
            let commented =
                cx.has_comment_between(body.span.start as usize, first.span.end as usize);
            out.push(Finding {
                start: w.cond.span.start as usize,
                end: w.cond.span.end as usize,
                message: "this loop leaves on its first statement; the test belongs in the `while`"
                    .to_string(),
                label: None,
                help: Some(format!("write `while {keep}` and drop the `if`")),
                fix: Some(Fix {
                    title: format!("loop `while {keep}`"),
                    edits: vec![
                        Edit {
                            start: w.cond.span.start as usize,
                            end: w.cond.span.end as usize,
                            text: keep,
                        },
                        Edit {
                            start,
                            end,
                            text: String::new(),
                        },
                    ],
                    machine_applicable: !commented,
                }),
            });
            true
        });
    }
}

/// The length of the blank line `rest` starts with, newline included.
fn blank_line(rest: &str) -> Option<usize> {
    let trimmed = rest.trim_start_matches([' ', '\t']);
    let spaces = rest.len() - trimmed.len();
    trimmed.starts_with('\n').then_some(spaces + 1)
}

/// The test that keeps the loop going, as text: `!c` gives `c`.
fn negated(cx: &Cx, cond: &Expr) -> String {
    if let E::Unary(UnOp::Not, inner) = &cond.kind {
        let text = cx.whole_src(inner.span).trim();
        if cx.parenthesized(inner.span) {
            return text[1..text.len() - 1].trim().to_string();
        }
        return text.to_string();
    }
    condition(cx, cond, true, false)
}

/// The statement is `break;` or a block holding only that.
fn only_break(s: &Stmt) -> bool {
    match &s.kind {
        S::Break(None) => true,
        S::Block(b) => {
            let mut stmts = b.stmts.iter().filter(|s| !matches!(s.kind, S::Empty));
            matches!(
                stmts.next(),
                Some(Stmt {
                    kind: S::Break(None),
                    ..
                })
            ) && stmts.next().is_none()
        }
        _ => false,
    }
}
