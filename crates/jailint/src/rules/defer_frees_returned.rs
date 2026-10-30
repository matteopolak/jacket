//! `defer_frees_returned`: `defer free(p); ... return p;`.
//!
//! ```jai
//! make_name :: (first: string) -> string {
//!     name := join(first, " Smith");
//!     defer free(name);
//!     return name;          // `name` is freed on the way out
//! }
//! ```
//!
//! A `defer` runs when its scope ends, after the `return` has picked its values. The caller
//! gets memory or a resource the deferred call has already released.
//!
//! Fires on `return v` (or `return xx v`, `return cast(T) v`) below a statement
//! `defer F(v);` in the same or an enclosing block, where `F` is a `@FreesArgument` procedure
//! and `v` a local variable or parameter. Quiet when anything after the `defer` assigns `v` or
//! takes its address (`v = null;` hands ownership on), when the `defer` holds more than the one
//! call (`defer if failed free(v);`), and in macros.
use super::frees::local_of;
use super::{PlaceUse, calls_noted, place_use, statement_lists, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::EntityId;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "defer_frees_returned",
    category: crate::config::Category::Correctness,
    summary: "`defer free(p);` followed by `return p;`, which hands back freed memory",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if p.is_macro || !p.typed() {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list: &[Stmt]| {
            for (k, s) in list.iter().enumerate() {
                let Some((var, name)) = deferred_free(cx, s) else {
                    continue;
                };
                let rest = &list[k + 1..];
                if changed(cx, rest, var) {
                    continue;
                }
                walk(rest, &mut |n, _| {
                    let Some(S::Return {
                        values,
                        backtick: false,
                    }) = n.stmt().map(|s| &s.kind)
                    else {
                        return true;
                    };
                    for v in values {
                        let value = uncast(&v.value);
                        if matches!(value.kind, E::Ident(_))
                            && cx.entities(value.span).contains(&var)
                        {
                            let (start, end) = cx.statement_removal(s.span);
                            out.push(suggesting(
                                value.span,
                                format!("`{name}` is returned, but the `defer` above frees it"),
                                "the deferred call runs after the value is returned: drop the \
                                 `defer`, or return a copy"
                                    .into(),
                                "remove the `defer` that frees the returned value",
                                vec![Edit {
                                    start,
                                    end,
                                    text: String::new(),
                                }],
                            ));
                        }
                    }
                    true
                });
            }
        });
    }
}

/// `defer F(v);` with `F` a `@FreesArgument` procedure and `v` a local: `v` and its name.
fn deferred_free<'a>(cx: &'a Cx, s: &Stmt) -> Option<(EntityId, &'a str)> {
    let S::Defer {
        body,
        backtick: false,
    } = &s.kind
    else {
        return None;
    };
    let S::Expr(call) = &body.kind else {
        return None;
    };
    let E::Call {
        callee,
        args,
        ..
    } = &call.kind
    else {
        return None;
    };
    if !calls_noted(cx, callee, "FreesArgument") {
        return None;
    }
    let first = args.first()?;
    if first.spread || first.context || !matches!(first.value.kind, E::Ident(_)) {
        return None;
    }
    Some((local_of(cx, &first.value)?, cx.src(first.value.span).trim()))
}

/// Something in `stmts` assigns `var` or takes its address.
fn changed(cx: &Cx, stmts: &[Stmt], var: EntityId) -> bool {
    let mut found = false;
    walk(stmts, &mut |n, chain| {
        if let Node::Expr(e) = n
            && matches!(e.kind, E::Ident(_))
            && cx.entities(e.span).contains(&var)
            && matches!(place_use(e, chain), PlaceUse::Write | PlaceUse::Address)
        {
            found = true;
        }
        !found
    });
    found
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
