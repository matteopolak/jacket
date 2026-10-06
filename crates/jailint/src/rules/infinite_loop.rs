//! `infinite_loop`: a `while` whose condition nothing in the loop can change.
//!
//! ```jai
//! i := 0;
//! while i < count {
//!     total += values[i];     // forgot `i += 1;`
//! }
//! ```
//!
//! If the condition holds once it holds forever: the loop never ends (and if it does not hold,
//! the body never runs).
//!
//! Fires when the condition is not a constant, is made only of local variables and parameters
//! of number, bool or enum type (and members of local structs and arrays, such as `xs.count`), literals and operators, and the
//! loop body:
//!
//! - never assigns those variables or takes their address;
//! - has no `break`, `return`, `remove`, `#insert`, `#asm` or `using`;
//! - calls no macro (`#expand` code can assign the caller's variables).
//!
//! Variables whose address is taken anywhere in the procedure, or that a `using` exposes, are
//! skipped: a pointer made before the loop could write them.
use super::{PlaceUse, any_stmt, callee_procs, finding, place_use};
use crate::Finding;
use crate::syntax::{Cx, Node, walk, walk_node};
use jaic::ast::{Expr, ExprKind as E, StmtKind as S, UnOp};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::types::TypeKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        // `using` anywhere in the procedure makes names alias fields.
        let has_using = p.header.params.iter().any(|q| q.using)
            || p.body
                .stmts
                .iter()
                .any(|s| any_stmt(s, &mut |s| matches!(s.kind, S::Using { .. })));
        if has_using {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt()
                && let S::While {
                    cond,
                    body,
                    bind_label: false,
                    ..
                } = &s.kind
                && let Some(vars) = condition_vars(cx, cond)
                && !vars.is_empty()
                && !escapes(cx, body)
                && !modified(cx, body, &vars)
                && !address_taken(cx, &p.body.stmts, &vars)
            {
                let names: Vec<String> = vars
                    .iter()
                    .map(|&e| format!("`{}`", cx.compiler.entity(e).name))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                out.push(finding(
                    cond.span,
                    "this loop's condition never changes inside it".into(),
                    Some(format!(
                        "nothing in the loop assigns {}, and it has no `break` or `return`: once entered it never ends",
                        names.join(" or ")
                    )),
                ));
            }
            true
        });
    }
}

/// The variables the condition reads, if it reads only scalar locals (directly or through
/// struct members), literals and operators.
fn condition_vars(cx: &Cx, cond: &Expr) -> Option<Vec<EntityId>> {
    if cx.constant(cond) {
        return None;
    }
    let mut vars = Vec::new();
    collect(cx, cond, &mut vars).then_some(vars)
}

fn collect(cx: &Cx, e: &Expr, vars: &mut Vec<EntityId>) -> bool {
    let types = &cx.compiler.types;
    match &e.kind {
        E::Int(_) | E::Float(_) | E::Bool(_) | E::Char(_) | E::InferredMember(_) | E::Null => true,
        E::Ident(_) => {
            if cx.constant(e) {
                return true;
            }
            let Some(ty) = cx.ty(e) else {
                return false;
            };
            let found = cx.entities(e.span);
            if found.is_empty()
                || !found
                    .iter()
                    .all(|&id| matches!(cx.compiler.entity(id).kind, EntityKind::Local { .. }))
            {
                return false;
            }
            let kind = types.kind(types.repr(ty));
            if !matches!(
                kind,
                TypeKind::Int { .. }
                    | TypeKind::Float { .. }
                    | TypeKind::Bool
                    | TypeKind::Struct(_)
                    | TypeKind::Array { .. }
            ) {
                return false;
            }
            vars.extend_from_slice(found);
            true
        }
        // A member of a local struct (not through a pointer).
        E::Member(base, _) => {
            cx.ty(base).is_some_and(|t| {
                matches!(types.kind(t), TypeKind::Struct(_) | TypeKind::Array { .. })
            }) && cx.ty(e).is_some_and(|t| {
                matches!(
                    types.kind(types.repr(t)),
                    TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
                )
            }) && collect(cx, base, vars)
        }
        E::Binary(_, a, b) => collect(cx, a, vars) && collect(cx, b, vars),
        E::Unary(UnOp::Not | UnOp::Neg | UnOp::BitNot | UnOp::Plus, a) => collect(cx, a, vars),
        E::Cast {
            value, ..
        } => collect(cx, value, vars),
        _ => false,
    }
}

/// The body can leave the loop, or runs code this rule cannot see into.
fn escapes(cx: &Cx, body: &jaic::ast::Stmt) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(body), &mut stack, &mut |n, _| {
        match n {
            Node::Stmt(s) => {
                if matches!(
                    s.kind,
                    S::Break(_)
                        | S::Return { .. }
                        | S::Remove(_)
                        | S::Insert { .. }
                        | S::Using { .. }
                ) {
                    found = true;
                }
            }
            Node::Expr(e) => match &e.kind {
                E::Insert {
                    ..
                }
                | E::Asm(_)
                | E::Backtick(_) => found = true,
                E::Call {
                    callee, ..
                } => {
                    let procs = callee_procs(cx, callee);
                    if procs
                        .iter()
                        .any(|&p| cx.compiler.procs[p.0 as usize].lit.header.flags.expand)
                    {
                        found = true;
                    }
                }
                _ => {}
            },
        }
        !found
    });
    found
}

/// Something in `body` assigns one of `vars` (or a part of it), or takes its address.
fn modified(cx: &Cx, body: &jaic::ast::Stmt, vars: &[EntityId]) -> bool {
    let mut found = false;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(body), &mut stack, &mut |n, chain| {
        if let Node::Expr(e) = n
            && let E::Ident(_) = e.kind
            && cx.entities(e.span).iter().any(|id| vars.contains(id))
            && place_use(e, chain) != PlaceUse::Read
        {
            found = true;
        }
        // `for x: 0..n` or `while x := ...` could rebind a name; declarations of the same
        // name inside the body are new variables and do not count.
        !found
    });
    found
}

/// One of `vars` has its address taken somewhere in the procedure.
fn address_taken(cx: &Cx, stmts: &[jaic::ast::Stmt], vars: &[EntityId]) -> bool {
    let mut found = false;
    walk(stmts, &mut |n, chain| {
        if let Node::Expr(e) = n
            && let E::Ident(_) = e.kind
            && cx.entities(e.span).iter().any(|id| vars.contains(id))
            && place_use(e, chain) == PlaceUse::Address
        {
            found = true;
        }
        !found
    });
    found
}
