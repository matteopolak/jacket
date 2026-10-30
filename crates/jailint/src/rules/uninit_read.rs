//! `uninit_read`: `total: int = ---; for xs total += it;`.
//!
//! `= ---` leaves a variable's memory as it was: whatever the stack held before. Reading it
//! before writing it gives a different value from run to run.
//!
//! Fires on a local declared `name: T = ---;` whose first mention in the statements after it,
//! in the same block, reads it: as a value (`print("%", x)`, `return x;`), through a field or
//! element (`x.a`, `x[0]`), or as the target of an operator assignment (`x += 1;`). Anything
//! else as the first mention ends the check: an assignment to it or a part of it, taking its
//! address (`init(*x)`, `for *x`), a `.data` or `.count` of a fixed-size array, a compiler
//! built-in such as `type_of(x)`, or code the scan cannot follow (`#insert`, `#asm`, a macro).
//! Procedures that use `using` are skipped.
use super::frees::{local_of, opaque};
use super::{PlaceUse, finding, place_use, statement_lists, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk_node};
use jaic::ast::{AssignOp, DeclKind, Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::types::{ArrayKind, TypeKind};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "uninit_read",
    category: crate::config::Category::Correctness,
    summary: "a variable declared `= ---` read before anything is written to it",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro || p.header.params.iter().any(|q| q.using) {
            continue;
        }
        if p.body
            .stmts
            .iter()
            .any(|s| super::any_stmt(s, &mut |s| matches!(s.kind, S::Using(_))))
        {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            for (k, s) in list.iter().enumerate() {
                let Some((var, name)) = uninitialized(cx, s) else {
                    continue;
                };
                if let Some(read) = first_read(cx, var, &list[k + 1..]) {
                    let message = format!("`{name}` is read here before anything is written to it");
                    let help = format!(
                        "`= ---` leaves `{name}` holding whatever was in memory: give it a \
                         value first, or drop the `= ---`"
                    );
                    // Without `= ---` the variable starts out zeroed.
                    let init = match &s.kind {
                        S::Decl(d) => d.ty.as_ref().zip(d.value.as_ref()),
                        _ => None,
                    };
                    out.push(match init {
                        Some((ty, value)) => suggesting(
                            read.span,
                            message,
                            help,
                            "drop the `= ---` so it starts out zeroed",
                            vec![Edit {
                                start: cx.whole(ty.span).1,
                                end: cx.whole(value.span).1,
                                text: String::new(),
                            }],
                        ),
                        None => finding(read.span, message, Some(help)),
                    });
                }
            }
        });
    }
}

/// `name: T = ---;`: the variable and its name.
fn uninitialized<'a>(cx: &'a Cx, s: &Stmt) -> Option<(EntityId, &'a str)> {
    let S::Decl(d) = &s.kind else {
        return None;
    };
    if d.kind != DeclKind::Var || d.names.len() != 1 || d.using || !d.notes.is_empty() {
        return None;
    }
    if !matches!(d.value.as_ref()?.kind, E::Uninit) {
        return None;
    }
    let name = &d.names[0];
    let ents = cx.facts.locals.get(&name.span)?;
    let [var] = ents.as_slice() else {
        return None;
    };
    Some((*var, name.name.as_str()))
}

/// The first mention of `var` in `rest`, when it reads the variable.
fn first_read<'a>(cx: &Cx, var: EntityId, rest: &'a [Stmt]) -> Option<&'a Expr> {
    for s in rest {
        if opaque(cx, s) {
            return None;
        }
        let mut decided: Option<Option<&Expr>> = None;
        let mut stack = Vec::new();
        walk_node(Node::Stmt(s), &mut stack, &mut |n, chain| {
            if decided.is_some() {
                return false;
            }
            let Node::Expr(e) = n else {
                return true;
            };
            if !matches!(e.kind, E::Ident(_)) || local_of(cx, e) != Some(var) {
                return true;
            }
            decided = Some(match place_use(e, chain) {
                PlaceUse::Address => None,
                PlaceUse::Write if operator_assigned(e, chain) => Some(e),
                PlaceUse::Write => None,
                PlaceUse::Read if not_a_read(cx, e, chain) => None,
                PlaceUse::Read => Some(e),
            });
            false
        });
        if let Some(found) = decided {
            return found;
        }
    }
    None
}

/// `x += 1;` with `x` itself as the target: it reads `x` first.
fn operator_assigned(e: &Expr, chain: &[Node<'_>]) -> bool {
    matches!(chain.last(), Some(Node::Stmt(s)) if matches!(&s.kind,
        S::Assign { op: AssignOp::Op(_), lhs, .. } if lhs.iter().any(|l| l.span == e.span)))
}

/// A mention that does not read the contents: `.data`/`.count` of a fixed-size array, or an
/// argument of a compiler built-in (`type_of(x)`, `size_of(type_of(x))`).
fn not_a_read(cx: &Cx, e: &Expr, chain: &[Node<'_>]) -> bool {
    if let Some(Node::Expr(p)) = chain.last()
        && let E::Member(base, member) = &p.kind
        && base.span == e.span
        && matches!(member.name.as_str(), "data" | "count")
        && fixed_array(cx, e)
    {
        return true;
    }
    chain.iter().any(|n| {
        matches!(n, Node::Expr(Expr { kind: E::Call { callee, .. }, .. })
        if matches!(callee.kind, E::Ident(_))
            && cx.entities(callee.span).iter().any(|&c| {
                matches!(cx.compiler.entity(c).kind, EntityKind::Builtin(_))
            }))
    })
}

fn fixed_array(cx: &Cx, e: &Expr) -> bool {
    let Some(ty) = cx.ty(e) else {
        return false;
    };
    let types = &cx.compiler.ty.types;
    matches!(
        types.kind(types.repr_struct(ty)),
        TypeKind::Array {
            kind: ArrayKind::Fixed(_),
            ..
        }
    )
}
