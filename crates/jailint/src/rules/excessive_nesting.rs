//! `excessive_nesting`: code nested more blocks deep than the configured limit.
//!
//! ```jai
//! for rows {
//!     if it.ok {
//!         for it.cells {
//!             if it.dirty {
//!                 // ...                 // four levels deep
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! Deep nesting hides which conditions hold where. Return early, `continue`, or move the inner
//! part into a procedure.
//!
//! A level is a braced block or a `switch` (its cases share one level). Procedure bodies do not
//! count themselves, but a procedure written inside a block carries that block's depth with it,
//! so a nested procedure (or a lambda) cannot hide the nesting. `else if` chains stay at the
//! level of the first `if`, and `#if` does not count. Reports the first block past the limit on
//! each path and does not look inside it. The limit is `[lint] max_nesting` in `jacket.toml` or
//! `--max-nesting` (default 6). Off by default.
use crate::Finding;
use crate::syntax::{Cx, ProcSite};
use jaic::ast::StmtKind as S;
use jaic::ast::walk::{Node, walk};
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "excessive_nesting",
    category: crate::config::Category::Pedantic,
    summary: "blocks nested deeper than `max_nesting` (default 6)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        // Inside a block that was already reported: the outer finding covers it.
        let base = enclosing_depth(cx, p);
        if base > cx.max_nesting {
            continue;
        }
        walk(&p.body.stmts, &mut |n, ancestors| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let (opens, header) = match &s.kind {
                S::Block(_) => (
                    true,
                    Span {
                        end: s.span.start + 1,
                        ..s.span
                    },
                ),
                S::Switch(switch) => (
                    true,
                    Span {
                        end: switch.value.span.end,
                        ..s.span
                    },
                ),
                _ => (false, s.span),
            };
            if !opens {
                return true;
            }
            let depth = base + levels(ancestors) + 1;
            if depth <= cx.max_nesting {
                return true;
            }
            out.push(Finding {
                start: header.start as usize,
                end: header.end as usize,
                message: format!(
                    "this block is nested {depth} deep, past the limit of {}",
                    cx.max_nesting
                ),
                label: None,
                help: Some(
                    "return or `continue` early, or move the inner part into its own procedure"
                        .into(),
                ),
                fix: None,
            });
            false
        });
    }
}

/// Blocks and switches among `ancestors`.
fn levels(ancestors: &[Node]) -> usize {
    ancestors
        .iter()
        .filter_map(|n| n.stmt())
        .filter(|s| matches!(s.kind, S::Block(_) | S::Switch(_)))
        .count()
}

/// Levels the procedure sits inside, from the procedures around it.
fn enclosing_depth(cx: &Cx, p: &ProcSite) -> usize {
    let inside = p.body.span;
    // The nearest enclosing body: the smallest one containing this body.
    let outer = cx
        .procs
        .iter()
        .filter(|o| {
            let s = o.body.span;
            s.start <= inside.start && inside.end <= s.end && s != inside
        })
        .min_by_key(|o| o.body.span.end - o.body.span.start);
    let Some(outer) = outer else {
        return 0;
    };
    let mut depth = 0;
    walk(&outer.body.stmts, &mut |n, ancestors| {
        let span = match n {
            Node::Stmt(s) => s.span,
            Node::Expr(e) => e.span,
        };
        let contains = span.start <= inside.start && inside.end <= span.end;
        if contains {
            depth = levels(ancestors);
        }
        contains
    });
    enclosing_depth(cx, outer) + depth
}
