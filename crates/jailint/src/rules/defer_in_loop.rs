//! `defer_in_loop`: a `defer` directly in a loop body that cleans up something from outside
//! the loop.
//!
//! `defer` runs when its scope ends, and a loop body's scope ends every iteration:
//!
//! ```jai
//! buffer := alloc(SIZE);
//! for files {
//!     defer free(buffer);    // freed after the first file, then again and again
//!     read_into(buffer, it);
//! }
//! ```
//!
//! Fires when the deferred statement uses only variables declared before the loop, and none
//! of them appears in the body before the `defer`. That leaves out the usual per-iteration
//! pairs (`lock(*m); defer unlock(*m);`, `f := open(it); defer close(f);`), whose resource is
//! taken in the same iteration, and loop steps such as `while i < n { defer i += 1; ... }`,
//! whose variable the loop's header reads.
use crate::Finding;
use crate::syntax::{Cx, Node, walk, walk_node};
use jaic::ast::{ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::scope::EntityKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(loop_stmt) = n.stmt() else {
                return true;
            };
            let body = match &loop_stmt.kind {
                S::For(f) => &*f.body,
                S::While {
                    body, ..
                } => &**body,
                _ => return true,
            };
            let list: &[Stmt] = match &body.kind {
                S::Block(b) => &b.stmts,
                _ => std::slice::from_ref(body),
            };
            for (k, s) in list.iter().enumerate() {
                let S::Defer {
                    body: deferred,
                    backtick: false,
                } = &s.kind
                else {
                    continue;
                };
                if let Some(f) = examine(cx, loop_stmt, body, &list[..k], deferred) {
                    out.push(Finding {
                        start: s.span.start as usize,
                        end: deferred.span.end as usize,
                        ..f
                    });
                }
            }
            true
        });
    }
}

fn examine(
    cx: &Cx,
    loop_stmt: &Stmt,
    body: &Stmt,
    before: &[Stmt],
    deferred: &Stmt,
) -> Option<Finding> {
    // The variables the deferred code uses, and where they were declared.
    let mut names = Vec::new();
    let mut unknown = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(deferred), &mut stack, &mut |n, _| {
        if let Some(e) = n.expr()
            && let E::Ident(name) = e.kind
        {
            for &ent in cx.entities(e.span) {
                let entity = cx.compiler.entity(ent);
                if matches!(entity.kind, EntityKind::Local { .. }) {
                    if entity.span.file != cx.file {
                        unknown = true;
                    } else if entity.span.start >= loop_stmt.span.start {
                        // Declared by or inside the loop: per-iteration cleanup.
                        unknown = true;
                    } else {
                        names.push(name);
                    }
                }
            }
        }
        true
    });
    if unknown || names.is_empty() {
        return None;
    }
    let (start, end) = match (before.first(), before.last()) {
        (Some(a), Some(b)) => (a.span.start, b.span.end),
        _ => (0, 0),
    };
    // A variable the loop's header reads is the loop's own state: deferring its step makes
    // `continue` take the step too.
    let no_span = jaic::source::Span::NONE;
    if names
        .iter()
        .any(|&n| cx.mentions(n, loop_stmt.span.start, body.span.start, no_span))
    {
        return None;
    }
    if start < end && names.iter().any(|&n| cx.mentions(n, start, end, no_span)) {
        return None;
    }
    Some(Finding {
        message: "this `defer` runs at the end of every iteration, not when the procedure returns"
            .into(),
        help: Some("move it before the loop if it should run once".into()),
        ..Finding::default()
    })
}
