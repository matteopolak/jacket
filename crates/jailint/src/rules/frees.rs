//! Small syntax utilities shared by non-lifetime rules. Lifetime checks use the semantic CFG.
use super::callee_procs;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk_node};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;

/// The one local variable or parameter an identifier names.
pub(super) fn local_of(cx: &Cx, e: &Expr) -> Option<EntityId> {
    match &*cx.entities(e.span) {
        [only] if matches!(cx.compiler.entity(*only).kind, EntityKind::Local { .. }) => Some(*only),
        _ => None,
    }
}

/// Code the scan cannot follow: it may touch the variable by name or leave the procedure.
pub(super) fn opaque(cx: &Cx, s: &Stmt) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
        match n {
            Node::Stmt(t) => {
                found |= matches!(t.kind, S::Insert(_) | S::Using(_));
            }
            Node::Expr(e) => match &e.kind {
                E::Insert(_) | E::Asm(_) | E::Backtick(_) => found = true,
                E::Call {
                    callee, ..
                } => {
                    found |= callee_procs(cx, callee)
                        .iter()
                        .any(|&p| cx.compiler.pr.procs[p.0 as usize].lit.header.flags.expand());
                }
                _ => {}
            },
        }
        !found
    });
    found
}
