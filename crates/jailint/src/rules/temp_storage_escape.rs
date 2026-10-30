//! `temp_storage_escape`: `config.name = tprint("%", id);` keeps a string that the next
//! `reset_temporary_storage()` recycles.
//!
//! Temporary storage is cheap because the program resets it wholesale, usually once per frame
//! or request. Anything allocated there must not outlive the reset.
//!
//! ```jai
//! label: string;                         // a global
//! set_label :: (id: int) {
//!     label = tprint("item %", id);      // dangles after the next reset
//! }
//! ```
//!
//! A procedure declared with the note `@TempAllocates` returns memory from temporary storage
//! (the library's `tprint`, `talloc`, `copy_temporary_string`, ...). Fires when such a result, or
//! a local that was initialised with one, is:
//!
//! - assigned to a global, to a member reached through a pointer or `.*`, or to a global
//!   variable's declaration (`name := tprint(...)` at file scope);
//! - added to such an array with a `@AddsToArray` procedure (`array_add(*registry, tprint(...))`);
//! - returned from a procedure that is not itself noted `@TempAllocates` (callers cannot tell
//!   that the string is temporary).
//!
//! Storing into a local, or into a member of a local struct, is not reported. Add the note to a
//! procedure that deliberately returns temporary memory and the rule stops reporting its
//! `return`s and starts reporting the uses of its result. Off by default: returning temporary
//! strings from helpers is a common, documented convention.
use super::{calls_noted, finding, has_note};
use crate::Finding;
use crate::syntax::{Cx, root_ident};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{AssignOp, Expr, ExprKind as E, Stmt, StmtKind as S, UnOp};
use jaic::fxhash::HashSet;
use jaic::intern::Sym;
use jaic::sema::scope::EntityKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "temp_storage_escape",
    category: crate::config::Category::Pedantic,
    summary: "a `@TempAllocates` result (`tprint`, `talloc`) stored in a global or through a pointer, or returned",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    // File-scope variables: the value is made once, then lives for the whole run.
    for s in cx.stmts {
        if let S::Decl(d) = &s.kind
            && d.names.len() == 1
            && d.rare.is_none()
            && d.kind == jaic::ast::DeclKind::Var
            && let Some(v) = &d.value
            && let Some(call) = temp_call(cx, v)
        {
            out.push(report(cx, call, Escape::Kept("a global variable")));
        }
    }
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let wrapper = cx
            .procs_with_header(p.header.span)
            .iter()
            .any(|&id| has_note(cx, id, "TempAllocates"));
        let mut locals: HashSet<Sym> = HashSet::default();
        walk(&p.body.stmts, &mut |n, _| {
            if let Node::Stmt(Stmt {
                kind: S::Decl(d), ..
            }) = n
                && d.names.len() == 1
                && d.rare.is_none()
                && d.value.as_ref().is_some_and(|v| temp_call(cx, v).is_some())
            {
                locals.insert(d.names[0].name);
            }
            true
        });
        walk(&p.body.stmts, &mut |n, _| {
            match n {
                Node::Stmt(s) => match &s.kind {
                    S::Return {
                        values, ..
                    } if !wrapper => {
                        for v in values {
                            if let Some(call) = tempish(cx, &v.value, &locals) {
                                out.push(report(cx, call, Escape::Returned));
                            }
                        }
                    }
                    S::Assign {
                        op: AssignOp::Assign,
                        lhs,
                        rhs,
                    } if lhs.len() == rhs.len() => {
                        for (l, r) in lhs.iter().zip(rhs) {
                            if let Some(call) = tempish(cx, r, &locals)
                                && let Some(place) = long_lived(cx, l)
                            {
                                out.push(report(cx, call, Escape::Kept(place)));
                            }
                        }
                    }
                    _ => {}
                },
                Node::Expr(e) => {
                    if let E::Call {
                        callee,
                        args,
                        ..
                    } = &e.kind
                        && args.len() == 2
                        && !args[1].spread
                        && calls_noted(cx, callee, "AddsToArray")
                        && let E::Unary(UnOp::Star, array) = &args[0].value.kind
                        && let Some(call) = tempish(cx, &args[1].value, &locals)
                        && let Some(place) = long_lived(cx, array)
                    {
                        out.push(report(cx, call, Escape::Kept(place)));
                    }
                }
            }
            true
        });
    }
}

/// Where a temporary string ends up.
enum Escape {
    Returned,
    Kept(&'static str),
}

fn report(cx: &Cx, call: &Expr, how: Escape) -> Finding {
    let name = match &call.kind {
        E::Call {
            callee, ..
        } => cx.whole_src(callee.span).trim(),
        _ => cx.whole_src(call.span).trim(),
    };
    let (message, help) = match how {
        Escape::Returned => (
            format!("`{name}` holds temporary storage, which is reset, but it is returned here"),
            "copy it (`copy_string`), or declare this procedure `@TempAllocates` so its callers know".to_string(),
        ),
        Escape::Kept(place) => (
            format!("`{name}` holds temporary storage, which is reset, but it is kept in {place}"),
            "copy it first (`copy_string`) so it outlives the reset".to_string(),
        ),
    };
    finding(call.span, message, Some(help))
}

/// The expression is a call of a `@TempAllocates` procedure, possibly cast: that call.
fn temp_call<'a>(cx: &Cx, e: &'a Expr) -> Option<&'a Expr> {
    let mut cur = e;
    while let E::Cast {
        value, ..
    } = &cur.kind
    {
        cur = value;
    }
    match &cur.kind {
        E::Call {
            callee, ..
        } if calls_noted(cx, callee, "TempAllocates") => Some(cur),
        _ => None,
    }
}

/// [`temp_call`], or a local that was initialised with one.
fn tempish<'a>(cx: &Cx, e: &'a Expr, locals: &HashSet<Sym>) -> Option<&'a Expr> {
    if let Some(c) = temp_call(cx, e) {
        return Some(c);
    }
    let mut cur = e;
    while let E::Cast {
        value, ..
    } = &cur.kind
    {
        cur = value;
    }
    matches!(&cur.kind, E::Ident(n) if locals.contains(n)).then_some(cur)
}

/// The assigned place outlives the procedure's temporary storage: what it is, if so.
fn long_lived(cx: &Cx, place: &Expr) -> Option<&'static str> {
    let (_, root) = root_ident(place)?;
    let local = cx
        .entities(root)
        .iter()
        .any(|&e| matches!(cx.compiler.entity(e).kind, EntityKind::Local { .. }));
    let known = !cx.entities(root).is_empty();
    // The path, from the place down to its root: a pointer anywhere in it leads elsewhere.
    let mut cur = place;
    let mut through_pointer = false;
    loop {
        match &cur.kind {
            E::Member(base, _) => {
                through_pointer |= cx
                    .ty(base)
                    .is_some_and(|t| cx.compiler.ty.types.is_pointer(t));
                cur = base;
            }
            E::Unary(UnOp::Deref, base) => {
                through_pointer = true;
                cur = base;
            }
            E::Index(base, _) => {
                through_pointer |= cx
                    .ty(base)
                    .is_some_and(|t| cx.compiler.ty.types.is_pointer(t));
                cur = base;
            }
            _ => break,
        }
    }
    if known && !local {
        Some("a global")
    } else if through_pointer {
        Some("a place reached through a pointer")
    } else {
        None
    }
}
