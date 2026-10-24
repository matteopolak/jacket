//! `needless_continue`: a `continue` as the last thing a loop body does.
//!
//! ```jai
//! for items {
//!     if it.skip continue;
//!     process(it);
//!     continue;           // the loop goes on anyway
//! }
//! ```
//!
//! When nothing follows a `continue` in the loop body, the loop would have continued without it.
//! That includes the end of a branch of an `if` that is the body's last statement.
//!
//! Fires on an unlabelled `continue` at the end of a `for` or `while` body, directly or at the
//! end of the `if`/`else` branches that close it. The fix removes the statement when other
//! statements share its block; a `continue` that is its block's only statement (`if c continue;`
//! last in the loop) leaves an `if` with no effect, so it is reported without a fix.
use super::{finding, replacing};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::{Stmt, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    super::walk_code(cx, &mut |n, _| {
        let Some(s) = n.stmt() else {
            return;
        };
        let body = match &s.kind {
            S::For(f) => &f.body,
            S::While(w) => &w.body,
            _ => return,
        };
        let mut ends = Vec::new();
        tails(body, &mut ends);
        for (cont, removable) in ends {
            let message = "this `continue` is the last thing the loop does".to_string();
            let help = "the loop goes on without it".to_string();
            let (start, end) = cx.tidy_removal(cont.span);
            if removable && !cx.has_comment_between(start, end) {
                out.push(replacing(
                    cont.span,
                    message,
                    "remove it".into(),
                    (start, end),
                    String::new(),
                    true,
                ));
            } else {
                out.push(finding(cont.span, message, Some(help)));
            }
        }
    });
}

/// The `continue` statements that end `s` (a loop body) as the last statement to run, and
/// whether each can be removed without leaving its block empty.
fn tails<'a>(s: &'a Stmt, out: &mut Vec<(&'a Stmt, bool)>) {
    match &s.kind {
        S::Continue(None) => out.push((s, false)),
        S::Block(b) => {
            let stmts: Vec<&Stmt> = b
                .stmts
                .iter()
                .filter(|s| !matches!(s.kind, S::Empty))
                .collect();
            if let Some(last) = stmts.last() {
                if matches!(last.kind, S::Continue(None)) {
                    out.push((last, stmts.len() > 1));
                } else {
                    tails(last, out);
                }
            }
        }
        S::If {
            then_branch,
            else_branch,
            ..
        } => {
            tails(then_branch, out);
            if let Some(e) = else_branch {
                tails(e, out);
            }
        }
        _ => {}
    }
}
