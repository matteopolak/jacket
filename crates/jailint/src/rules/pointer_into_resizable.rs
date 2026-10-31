//! `pointer_into_resizable`: `p := *xs[i]; array_add(*xs, v); use(p);` reads memory the array
//! may have moved.
//!
//! A resizable array that outgrows its storage allocates a new block and frees the old one, so
//! a pointer taken into the old block dangles.
//!
//! ```jai
//! first := *items[0];
//! array_add(*items, extra);      // may reallocate `items`
//! first.count += 1;              // writes to freed memory
//! ```
//!
//! A procedure declared with the note `@GrowsArray` may reallocate the array its first argument
//! points to (the library's `array_add`, `array_insert_at`, `array_reserve`, `array_resize`).
//! Fires on:
//!
//! - a local initialised with the address of an element (`p := *xs[i];`, `p := *xs[i].field;`)
//!   or with a call of a `@GrowsArray` procedure that returns a pointer (`p := array_add(*xs);`),
//!   when a later statement of the same block grows the same array and a statement after that
//!   mentions `p`;
//! - the iterator of `for *xs` (`it`, or the name given), when a statement of the body grows
//!   `xs` and a later statement of the body mentions the iterator.
//!
//! A by-value `for xs` is safe: each iteration reads the count and data again. Both arrays are
//! compared as written (`xs` and `xs` or `s.items` and `s.items`). The rule is quiet when the
//! growing statement can leave the block early (`return`, `break`, `continue`, `remove`), when
//! the pointer is assigned to again, or when the procedure is a macro.
use super::{any_stmt, calls_noted, finding, statement_lists};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk, walk_node};
use jaic::ast::{Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp};
use jaic::intern::Sym;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "pointer_into_resizable",
    category: crate::config::Category::Suspicious,
    summary: "a pointer into a resizable array used after a `@GrowsArray` call may have moved the array (`p := *xs[i]; array_add(*xs, v); use(p);`)",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list| {
            for (k, s) in list.iter().enumerate() {
                match &s.kind {
                    S::Decl(d) if d.names.len() == 1 && d.rare.is_none() => {
                        if let Some(array) = d.value.as_ref().and_then(|v| pointer_source(cx, v)) {
                            let name = d.names[0].name;
                            dangling(cx, &list[k + 1..], array, name, out);
                        }
                    }
                    S::For(f) if f.by_pointer && f.pointer_if.is_none() => {
                        if let ForOver::Collection(c) = &f.over
                            && let S::Block(b) = &f.body.kind
                        {
                            let name = f.it.map_or_else(|| Sym::intern("it"), |i| i.name);
                            dangling(cx, &b.stmts, c, name, out);
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

/// The expression yields a pointer into a resizable array: the array.
fn pointer_source<'a>(cx: &Cx, value: &'a Expr) -> Option<&'a Expr> {
    match &value.kind {
        E::Unary(UnOp::Star, inner) => {
            let mut cur: &Expr = inner;
            while let E::Member(base, _) = &cur.kind {
                cur = base;
            }
            match &cur.kind {
                E::Index(base, _) => Some(base),
                _ => None,
            }
        }
        E::Call {
            callee,
            args,
            ..
        } if args.len() == 1 && calls_noted(cx, callee, "GrowsArray") => {
            grown_array(&args[0].value)
        }
        _ => None,
    }
}

/// `*xs`: the array.
fn grown_array(arg: &Expr) -> Option<&Expr> {
    match &arg.kind {
        E::Unary(UnOp::Star, inner) => Some(inner),
        _ => None,
    }
}

/// A statement of `list` grows `array` and a later one mentions `name`.
fn dangling(cx: &Cx, list: &[Stmt], array: &Expr, name: Sym, out: &mut Vec<Finding>) {
    let Some(end) = list.last().map(|s| s.span.end) else {
        return;
    };
    if reassigned(list, name) {
        return;
    }
    for s in list {
        if let Some((call, procedure)) = growth(cx, s, array)
            && !leaves_early(s)
        {
            let from = call.span.end;
            let Some(i) = cx.name_tokens(name, from, end).next() else {
                return;
            };
            let at = cx.tokens[i].span;
            out.push(finding(
                at,
                format!(
                    "`{}` points into `{}`, which `{procedure}` may have moved by now",
                    name.as_str(),
                    cx.whole_src(array).trim()
                ),
                Some(
                    "take the pointer again after the array has grown, or keep the index instead"
                        .into(),
                ),
            ));
            return;
        }
    }
}

/// The first call inside `s` of a `@GrowsArray` procedure on `array`: the call and the name of
/// the procedure.
fn growth<'a>(cx: &Cx, s: &'a Stmt, array: &Expr) -> Option<(&'a Expr, String)> {
    let mut found = None;
    let mut stack = Vec::new();
    walk_node(Node::Stmt(s), &mut stack, &mut |n, _| {
        if found.is_none()
            && let Node::Expr(e) = n
            && let E::Call {
                callee,
                args,
                ..
            } = &e.kind
            && let Some(first) = args.first()
            && let Some(target) = grown_array(&first.value)
            && cx.same_tokens(target, array)
            && calls_noted(cx, callee, "GrowsArray")
        {
            found = Some((e, cx.whole_src(callee).trim().to_string()));
        }
        found.is_none()
    });
    found
}

/// The statement can end the block early, so a later statement may not run after it.
fn leaves_early(s: &Stmt) -> bool {
    any_stmt(s, &mut |t| {
        matches!(
            t.kind,
            S::Return { .. } | S::Break(_) | S::Continue(_) | S::Remove(_)
        )
    })
}

/// Some statement gives `name` a new value.
fn reassigned(list: &[Stmt], name: Sym) -> bool {
    let mut found = false;
    walk(list, &mut |n, _| {
        if let Some(Stmt {
            kind: S::Assign {
                lhs, ..
            },
            ..
        }) = n.stmt()
        {
            found |= lhs
                .iter()
                .any(|l| matches!(&l.kind, E::Ident(x) if *x == name));
        }
        !found
    });
    found
}
