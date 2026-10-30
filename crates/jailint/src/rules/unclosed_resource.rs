//! `unclosed_resource`: `file, ok := file_open(path);` or `append(*builder, text);` on a local
//! that nothing ever closes.
//!
//! A file handle, string builder or similar local owns something that has to be given back:
//! `file_close(*file)`, `free_buffers(*builder)`, `builder_to_string(*builder)`. A local that is
//! opened and then only looked at leaks it.
//!
//! ```jai
//! builder: String_Builder;
//! append(*builder, "total: ");        // buffers allocated here
//! print_to_builder(*builder, "%", n);
//! // no builder_to_string, free_buffers or reset
//! ```
//!
//! Procedures declare their part with notes: `@Opens` on one that acquires something, `@Closes`
//! on one that gives it back (a `@FreesArgument` procedure closes too). An opener is either
//! assigned (`file, ok := file_open(path);`, the first name is the resource) or called as a
//! statement on a local's address (`append(*builder, s);`, `init_string_builder(*builder);`).
//! The local must be declared in the procedure.
//!
//! Fires when, after the opening, every mention of the local is another `@Opens` call, a
//! comparison, a test in an `if` or `while`, or a member of a number, bool or enum read or
//! written. Any other mention may close the resource or hand it on and keeps the rule quiet:
//! passing the local (or its address) to any other call, `defer`red ones included; returning
//! it; assigning it elsewhere; putting it in a literal; a name used where the scan cannot
//! follow (a nested procedure, a code block). It does not check that every path closes.
use super::{calls_noted, finding};
use crate::Finding;
use crate::syntax::Cx;
use jaic::ast::walk::{Node, walk};
use jaic::ast::{BinOp, Block, Expr, ExprKind as E, StmtKind as S, UnOp};
use jaic::intern::Sym;
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::types::TypeKind;

/// A local that an `@Opens` call acquired something for.
struct Opened<'a> {
    name: Sym,
    var: EntityId,
    /// The opening call.
    call: &'a Expr,
    /// Mentions before this offset are the opening itself.
    after: u32,
}

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "unclosed_resource",
    category: crate::config::Category::Suspicious,
    summary: "a local opened by an `@Opens` call (`file_open`, `append(*builder, ..)`) that nothing closes",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let mut opened: Vec<Opened> = Vec::new();
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(s) = n.stmt() {
                match &s.kind {
                    S::Decl(d) if d.rare.is_none() && !d.using && !d.backtick => {
                        if let Some(value) = &d.value
                            && let Some(call) = opening(cx, value)
                            && let Some(name) = d.names.first()
                            && name.name.as_str() != "_"
                            && let Some(var) = single(&cx.locals(name.span))
                        {
                            opened.push(Opened {
                                name: name.name,
                                var,
                                call,
                                after: d.span.end,
                            });
                        }
                    }
                    S::Expr(e) => {
                        if let E::Call {
                            args, ..
                        } = &e.kind
                            && opening(cx, e).is_some()
                            && let Some(first) = args.first()
                            && !first.spread
                            && let E::Unary(UnOp::Star, target) = &first.value.kind
                            && let E::Ident(name) = &target.kind
                            && let Some(var) = single(&cx.entities(target.span))
                            && declared_in(cx, var, p.body)
                            && !opened.iter().any(|o| o.var == var)
                        {
                            opened.push(Opened {
                                name: *name,
                                var,
                                call: e,
                                after: cx.compiler.entity(var).span.end,
                            });
                        }
                    }
                    _ => {}
                }
            }
            true
        });
        for o in opened {
            if left_open(cx, p.body, &o) {
                let callee = match &o.call.kind {
                    E::Call {
                        callee, ..
                    } => cx.whole_src(callee.span).trim(),
                    _ => "",
                };
                out.push(finding(
                    o.call.span,
                    format!(
                        "`{}` is opened by `{callee}` but nothing closes it",
                        o.name.as_str()
                    ),
                    Some(
                        "close it when done, for example with a `defer` right after opening".into(),
                    ),
                ));
            }
        }
    }
}

/// The only element of the list.
fn single(list: &[EntityId]) -> Option<EntityId> {
    match list {
        [only] => Some(*only),
        _ => None,
    }
}

/// The entity is a local variable whose declaration lies inside `body`.
fn declared_in(cx: &Cx, var: EntityId, body: &Block) -> bool {
    let e = cx.compiler.entity(var);
    matches!(e.kind, EntityKind::Local { .. })
        && e.span.file == body.span.file
        && body.span.start <= e.span.start
        && e.span.end <= body.span.end
}

/// The expression is a call of an `@Opens` procedure, possibly cast: that call.
fn opening<'a>(cx: &Cx, e: &'a Expr) -> Option<&'a Expr> {
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
        } if calls_noted(cx, callee, "Opens") => Some(cur),
        _ => None,
    }
}

/// Every mention of the local after its opening is quiet, and none is hidden from the scan.
fn left_open(cx: &Cx, body: &Block, o: &Opened) -> bool {
    let mut handled = false;
    let mut seen = 0usize;
    walk(&body.stmts, &mut |n, chain| {
        if handled {
            return false;
        }
        let Node::Expr(e) = n else {
            return true;
        };
        if matches!(e.kind, E::Ident(sym) if sym == o.name) && e.span.start >= o.after {
            seen += 1;
            if cx.entities(e.span).contains(&o.var) && !quiet(cx, e, chain) {
                handled = true;
            }
        }
        true
    });
    let written = cx
        .name_tokens(o.name, o.after, body.span.end.max(o.after))
        .count();
    !handled && written == seen
}

/// A mention that neither closes the resource nor lets it escape.
fn quiet(cx: &Cx, e: &Expr, chain: &[Node<'_>]) -> bool {
    let mut cur = e;
    let mut i = chain.len();
    let mut projected = false;
    while i > 0 {
        let Node::Expr(p) = chain[i - 1] else {
            break;
        };
        match &p.kind {
            E::Member(base, _) | E::Index(base, _) if base.span == cur.span => projected = true,
            E::Unary(UnOp::Deref | UnOp::Star, inner) if inner.span == cur.span => {}
            _ => break,
        }
        cur = p;
        i -= 1;
    }
    match i.checked_sub(1).map(|k| chain[k]) {
        Some(Node::Stmt(s)) => match &s.kind {
            S::Assign {
                lhs, ..
            } if lhs.iter().any(|l| l.span == cur.span) => projected,
            S::Expr(x) if x.span == cur.span => true,
            S::If {
                cond, ..
            } if cond.span == cur.span => true,
            S::While(w) if w.cond.span == cur.span => true,
            _ => projected && scalar(cx, cur),
        },
        Some(Node::Expr(p)) => match &p.kind {
            E::Binary(
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge,
                ..,
            ) => true,
            E::Call {
                callee,
                args,
                ..
            } if args.iter().any(|a| a.value.span == cur.span) => calls_noted(cx, callee, "Opens"),
            _ => projected && scalar(cx, cur),
        },
        None => false,
    }
}

/// The expression is a number, bool or enum: reading it keeps nothing alive.
fn scalar(cx: &Cx, e: &Expr) -> bool {
    let types = &cx.compiler.ty.types;
    cx.ty(e).is_some_and(|t| {
        matches!(
            types.kind(types.repr(t)),
            TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
        )
    })
}
