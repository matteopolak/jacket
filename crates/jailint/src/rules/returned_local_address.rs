//! `returned_local_address`: `return *x;` with `x` a local variable.
//!
//! ```jai
//! counter :: () -> *int {
//!     count := 0;
//!     return *count;            // `count` is gone once the procedure returns
//! }
//! ```
//!
//! A local lives in the procedure's stack frame, which the next call reuses. The caller gets a
//! pointer that reads whatever is written there next.
//!
//! Fires on a returned value (under any casts) that is the address of a variable declared in
//! the procedure body (`*x`), of a part of one held in place (`*x.field`, `*x[i]` with `x` a
//! fixed-size array), or the `.data` of a fixed-size array local. Parameters are left alone,
//! and so is anything reached through a pointer, view or resizable array (`*p.field` with
//! `p: *T` points at memory the local does not own).
use super::finding;
use crate::Finding;
use crate::syntax::{Cx, walk};
use jaic::ast::{DeclKind, Expr, ExprKind as E, StmtKind as S, UnOp};
use jaic::sema::EntityId;
use jaic::types::{ArrayKind, TypeKind};

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "returned_local_address",
    category: crate::config::Category::Correctness,
    summary: "`return *x;` with `x` a local, a pointer into a stack frame that is gone",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if p.is_macro || !p.typed() {
            continue;
        }
        let mut locals: Vec<EntityId> = Vec::new();
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(S::Decl(d)) = n.stmt().map(|s| &s.kind)
                && d.kind != DeclKind::Const
            {
                for name in &d.names {
                    if let Some(ents) = cx.facts.locals.get(&name.span) {
                        locals.extend_from_slice(ents);
                    }
                }
            }
            true
        });
        if locals.is_empty() {
            continue;
        }
        walk(&p.body.stmts, &mut |n, _| {
            let Some(S::Return {
                values,
                backtick: false,
            }) = n.stmt().map(|s| &s.kind)
            else {
                return true;
            };
            for v in values {
                let value = uncast(&v.value);
                let Some((name, how)) = local_address(cx, value, &locals) else {
                    continue;
                };
                out.push(finding(
                    value.span,
                    format!("this returns {how} the local `{name}`, which is gone once the procedure returns"),
                    Some(
                        "return the value itself, take the memory from an allocator (`New`, \
                         `alloc`), or have the caller pass the place to fill"
                            .into(),
                    ),
                ));
            }
            true
        });
    }
}

/// The value is the address of (part of) one of `locals`: the local's name, and how the
/// message says it.
fn local_address<'a>(cx: &'a Cx, e: &Expr, locals: &[EntityId]) -> Option<(&'a str, &'static str)> {
    match &e.kind {
        E::Unary(UnOp::Star, place) => held(cx, place, locals).map(|n| (n, "the address of")),
        E::Member(base, member) if member.name.as_str() == "data" && fixed_array(cx, base) => {
            held(cx, base, locals).map(|n| (n, "the data of"))
        }
        _ => None,
    }
}

/// `place` is stored inside a local itself: the local, or a field or element reached from it
/// without going through a pointer. The local's name.
fn held<'a>(cx: &'a Cx, place: &Expr, locals: &[EntityId]) -> Option<&'a str> {
    match &place.kind {
        E::Ident(_) => cx
            .entities(place.span)
            .iter()
            .any(|e| locals.contains(e))
            .then(|| cx.src(place.span).trim()),
        E::Member(base, _) if in_place(cx, base) => held(cx, base, locals),
        E::Index(base, _) if fixed_array(cx, base) => held(cx, base, locals),
        _ => None,
    }
}

/// A value whose fields are stored in it: a struct or a fixed-size array.
fn in_place(cx: &Cx, e: &Expr) -> bool {
    let Some(ty) = cx.ty(e) else {
        return false;
    };
    let types = &cx.compiler.ty.types;
    matches!(types.kind(types.repr_struct(ty)), TypeKind::Struct(_)) || fixed_array(cx, e)
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

/// The value under any casts.
fn uncast(e: &Expr) -> &Expr {
    match &e.kind {
        E::Cast {
            value, ..
        } => uncast(value),
        _ => e,
    }
}
