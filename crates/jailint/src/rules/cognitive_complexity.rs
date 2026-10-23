//! `cognitive_complexity`: a procedure that is hard to follow, by how much branching it has
//! and how deeply that branching nests.
//!
//! A procedure scores:
//! - 1 for each `if`, `while`, `for`, `switch` and `ifx`, plus 1 for each level of those it
//!   sits inside, so nested branching costs more than the same branching in sequence;
//! - 1 for each `else if` and `else` (no nesting cost: the chain reads as one decision);
//! - 1 for each run of `&&` or `||` (`a && b && c` is one, `a && b || c` two);
//! - 1 for each labelled `break` or `continue`, which jumps further than it looks.
//!
//! `#if` and `#run` are not counted, and procedures written inside a body are scored on their
//! own. A procedure scoring above `[lint] max_complexity` (default 25) is reported on its name.
//! Split it into smaller procedures, return early, or name parts of long conditions. Generated
//! files are skipped. Off by default.
use crate::Finding;
use crate::syntax::{Cx, Node, walk};
use jaic::ast::{BinOp, ExprKind as E, Stmt, StmtKind as S};
use jaic::source::Span;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        let score = score(&p.body.stmts);
        if score <= cx.max_complexity {
            continue;
        }
        let (span, name) = match p.decl.and_then(|d| d.names.first()) {
            Some(name) => (name.span, format!("`{}`", name.name.as_str())),
            None => (
                Span {
                    end: p.header.span.start + 1,
                    ..p.header.span
                },
                "this procedure".to_string(),
            ),
        };
        out.push(Finding {
            start: span.start as usize,
            end: span.end as usize,
            message: format!(
                "{name} has a cognitive complexity of {score}, past the limit of {}",
                cx.max_complexity
            ),
            label: None,
            help: Some(
                "split it into smaller procedures, return early, or name parts of long conditions"
                    .into(),
            ),
            fix: None,
        });
    }
}

/// The cognitive complexity of a body.
pub(crate) fn score(stmts: &[Stmt]) -> usize {
    let mut total = 0;
    walk(stmts, &mut |n, ancestors| {
        let parent = ancestors.last().copied();
        match n {
            Node::Stmt(s) => match &s.kind {
                S::If {
                    else_branch, ..
                } => {
                    total += if else_if(s, parent) {
                        1
                    } else {
                        1 + nesting(ancestors)
                    };
                    // A plain `else`; an `else if` counts itself.
                    if else_branch
                        .as_ref()
                        .is_some_and(|b| !matches!(b.kind, S::If { .. }))
                    {
                        total += 1;
                    }
                }
                S::While(_) | S::For(_) | S::Switch(_) => total += 1 + nesting(ancestors),
                S::Break(Some(_)) | S::Continue(Some(_)) => total += 1,
                S::StaticIf(_) | S::Run(_) => return false,
                _ => {}
            },
            Node::Expr(e) => match &e.kind {
                E::Ifx {
                    is_static, ..
                } if !is_static => total += 1 + nesting(ancestors),
                E::Binary(op @ (BinOp::And | BinOp::Or), ..) => {
                    // A run of the same operator counts once, at its top.
                    let same = matches!(
                        parent.and_then(Node::expr).map(|p| &p.kind),
                        Some(E::Binary(outer, ..)) if outer == op
                    );
                    if !same {
                        total += 1;
                    }
                }
                E::Run {
                    ..
                } => return false,
                _ => {}
            },
        }
        true
    });
    total
}

/// The `if` statement `s` is the `else` branch of the `if` above it.
fn else_if(s: &Stmt, parent: Option<Node>) -> bool {
    matches!(
        parent.and_then(Node::stmt).map(|p| &p.kind),
        Some(S::If { else_branch: Some(b), .. }) if std::ptr::eq(&raw const **b, s)
    )
}

/// Branching levels around a node: `if` (not counting `else if` links), loops, `switch`
/// and `ifx`.
fn nesting(ancestors: &[Node]) -> usize {
    let mut levels = 0;
    for (i, n) in ancestors.iter().enumerate() {
        let parent = i.checked_sub(1).map(|j| ancestors[j]);
        levels += match n {
            Node::Stmt(s) => match &s.kind {
                S::If {
                    ..
                } => usize::from(!else_if(s, parent)),
                S::While(_) | S::For(_) | S::Switch(_) => 1,
                _ => 0,
            },
            Node::Expr(e) => usize::from(matches!(
                e.kind,
                E::Ifx {
                    is_static: false,
                    ..
                }
            )),
        };
    }
    levels
}
