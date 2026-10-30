//! `useless_resizable`: a local `[..] T` that gets a fixed number of elements and never grows.
//!
//! ```jai
//! parts: [..] string;
//! array_add(*parts, "a");
//! array_add(*parts, "b");
//! ```
//!
//! A resizable array allocates and has to be freed; two elements known up front fit in a fixed
//! array (`parts: [2] string`, or `parts := string.["a", "b"]`) on the stack.
//!
//! The rule is deliberately narrow. It fires on a local declared `name: [..] T;` (no value)
//! when every later use in the procedure is one of: a statement of the same block that is just
//! `array_add(*name, item)` (a procedure noted `@AddsToArray`) and so adds a known number of
//! elements, an index, `.count` or `.data`, the collection of a `for`, or a call that frees it
//! (`@FreesArgument`). Passing the array anywhere else (to a procedure, into another variable,
//! out of a `return`) may need the `[..]` type, so those locals are left alone, as are adds
//! made inside loops, branches or with several items at once. No fix.
use super::{calls_noted, statement_lists};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk};
use jaic::ast::{
    ArraySize, Decl, DeclKind, Expr, ExprKind as E, ForOver, Stmt, StmtKind as S, UnOp,
};
use jaic::fxhash::HashSet;
use jaic::source::Span;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "useless_resizable",
    category: crate::config::Category::Perf,
    summary: "a local `[..] T` that is only filled with a known number of `array_add` calls and never resized",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if !p.typed() {
            continue;
        }
        statement_lists(&p.body.stmts, &mut |list| {
            for (i, s) in list.iter().enumerate() {
                if let S::Decl(d) = &s.kind
                    && let Some(found) = examine(cx, &p.body.stmts, list, i, d)
                {
                    out.push(found);
                }
            }
        });
    }
}

fn is_resizable(d: &Decl) -> bool {
    d.kind == DeclKind::Var
        && d.value.is_none()
        && d.names.len() == 1
        && !d.using
        && !d.backtick
        && d.rare.is_none()
        && matches!(
            d.ty.as_ref().map(|t| &t.kind),
            Some(E::ArrayType {
                size: ArraySize::Resizable,
                ..
            })
        )
}

fn examine(cx: &Cx, body: &[Stmt], list: &[Stmt], at: usize, d: &Decl) -> Option<Finding> {
    if !is_resizable(d) {
        return None;
    }
    let name = d.names[0];
    let entities: HashSet<_> = cx.facts.locals.get(&name.span)?.iter().copied().collect();
    let is_it = |e: &Expr| {
        matches!(e.kind, E::Ident(_)) && cx.entities(e.span).iter().any(|x| entities.contains(x))
    };
    // The adds that follow in the same list, and the identifiers inside them.
    let mut adds = 0usize;
    let mut allowed: HashSet<Span> = HashSet::default();
    for s in &list[at + 1..] {
        let S::Expr(call) = &s.kind else {
            continue;
        };
        let E::Call {
            callee,
            args,
            ..
        } = &call.kind
        else {
            continue;
        };
        let Some(first) = args.first() else {
            continue;
        };
        let E::Unary(UnOp::Star, target) = &first.value.kind else {
            continue;
        };
        if !is_it(target)
            || !calls_noted(cx, callee, "AddsToArray")
            || args
                .iter()
                .any(|a| a.spread || a.context || a.name.is_some())
        {
            continue;
        }
        adds += 1;
        allowed.insert(target.span);
    }
    if adds == 0 {
        return None;
    }
    // Every other mention must be a harmless read or a free.
    let mut ok = true;
    let mut freed = false;
    walk(body, &mut |n, chain| {
        if !ok {
            return false;
        }
        let Node::Expr(e) = n else {
            return true;
        };
        if !is_it(e) || e.span == name.span || allowed.contains(&e.span) {
            return true;
        }
        let (mut at, mut up) = (e, chain.len());
        // `*name` as an argument is its address.
        if let Some(Node::Expr(parent)) = chain.last()
            && let E::Unary(UnOp::Star, inner) = &parent.kind
            && inner.span == e.span
        {
            at = parent;
            up -= 1;
        }
        let parent = up.checked_sub(1).map(|i| chain[i]);
        let fine = match parent {
            Some(Node::Expr(p)) => match &p.kind {
                E::Index(base, _) => base.span == at.span && std::ptr::eq(at, e),
                E::Member(base, member) => {
                    base.span == at.span
                        && std::ptr::eq(at, e)
                        && matches!(member.name.as_str(), "count" | "data")
                }
                E::Call {
                    callee,
                    args,
                    ..
                } if args.first().is_some_and(|a| a.value.span == at.span)
                    && calls_noted(cx, callee, "FreesArgument") =>
                {
                    freed = true;
                    true
                }
                _ => false,
            },
            Some(Node::Stmt(s)) => match &s.kind {
                S::For(f) => {
                    !f.by_pointer
                        && f.pointer_if.is_none()
                        && matches!(&f.over, ForOver::Collection(c) if c.span == e.span)
                }
                _ => false,
            },
            None => false,
        };
        ok &= fine;
        true
    });
    if !ok {
        return None;
    }
    let ty = cx.src(d.ty.as_ref()?.span);
    let elem = ty
        .trim_start_matches('[')
        .trim_start_matches("..")
        .trim_start_matches(']')
        .trim();
    Some(super::finding(
        name.span,
        format!(
            "`{}` is a resizable array that only ever holds {adds} element{}",
            name.name.as_str(),
            if adds == 1 {
                ""
            } else {
                "s"
            }
        ),
        Some(format!(
            "a fixed array does not allocate: `{}: [{adds}] {elem};`{}",
            name.name.as_str(),
            if freed {
                ", and the call that frees it goes away"
            } else {
                ""
            }
        )),
    ))
}
