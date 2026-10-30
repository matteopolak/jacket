//! `push_in_static_if`: a scoped push directly in an `#if` arm, outside any `{ }` block.
//!
//! `#if` adds no scope, so the push lasts to the end of the enclosing block or procedure,
//! not to the end of the arm:
//!
//! ```jai
//! #if USE_ARENA {
//!     push_allocator(arena_allocator);    // still in effect after the `#if`
//! }
//! work();
//! ```
//!
//! Fires on a call of a procedure declared with the note `@PushesContext` (a macro that
//! changes the context and restores it with a backtick `defer`: the library's
//! `push_allocator` carries it; a program marks its own the same way), and on
//! `push_context,defer_pop`, when it is a direct statement of an `#if` or `#else` arm. A
//! push in a nested `{ }` block of the arm is scoped by the block and stays quiet, as does a
//! push outside any `#if`.
use super::calls_noted;
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::walk;
use jaic::ast::{ExprKind as E, Stmt, StmtKind as S};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "push_in_static_if",
    category: crate::config::Category::Suspicious,
    summary: "a scoped push directly in an `#if` arm, which lasts past the arm",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(S::StaticIf(arm)) = n.stmt().map(|s| &s.kind) {
                for s in arm.then_branch.iter().chain(&arm.else_branch) {
                    if pushes(cx, s) {
                        out.push(Finding {
                            start: s.span.start as usize,
                            end: s.span.end as usize,
                            message: "this push lasts until the end of the enclosing procedure, not the end of the `#if` arm"
                                .into(),
                            help: Some(
                                "wrap it and the code it covers in a `{ }` block, or move it out of the `#if`"
                                    .into(),
                            ),
                            ..Finding::default()
                        });
                    }
                }
            }
            true
        });
    }
}

/// The statement pushes something that is restored when the enclosing scope ends.
fn pushes(cx: &Cx, s: &Stmt) -> bool {
    match &s.kind {
        S::PushContextDefer(push) => !push.backtick,
        S::Expr(e) => match &e.kind {
            E::Call {
                callee, ..
            } => calls_noted(cx, callee, "PushesContext"),
            _ => false,
        },
        _ => false,
    }
}
