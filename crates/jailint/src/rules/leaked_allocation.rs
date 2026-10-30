//! `leaked_allocation`: `buffer := alloc(1024);` in a procedure that never frees it, returns it
//! or hands it on.
//!
//! A procedure declared with the note `@Allocates` returns something newly allocated that the
//! caller owns. If the caller keeps it in a local and does nothing that could pass ownership on
//! or release it, the memory is leaked.
//!
//! ```jai
//! scratch := New(Parser);          // never freed, returned or passed on
//! scratch.depth = 0;
//! if scratch.depth > 3 return;
//! ```
//!
//! Fires on a declaration of one local (`x := f(...)`, `x: T = cast(*T) f(...)`) whose value is
//! a call of an `@Allocates` procedure, when every later mention of `x` in the procedure is
//! harmless: reading or writing a number, bool or enum member through it (`x.count`,
//! `x.value = 1`, `x[i] = 0`), comparing it, testing it in an `if` or `while`, or standing alone
//! as a statement. Any other mention counts as handling the memory and silences the rule:
//! passing `x` (or anything reached through it) to a call, `@FreesArgument` calls and `defer`red
//! ones included; returning it; assigning it or a pointer into it elsewhere; taking its
//! address; putting it in a literal; reassigning `x`. Procedure-wide, it stays quiet when the
//! procedure uses `push_context`, assigns `context`, calls a macro (`push_allocator` is one),
//! has `#insert`, `#asm` or `using`, when the allocating call carries a `,,` context change, or
//! when the name is also mentioned where the rule cannot follow it (a nested procedure, a code
//! block): the allocator in force may be temporary storage, and then nothing is leaked.
//!
//! Off by default: ownership conventions differ, and a program that allocates once for its whole
//! run may leak on purpose.
use super::{callee_procs, calls_noted, finding, statement_range, suggesting};
use crate::syntax::Cx;
use crate::{Edit, Finding};
use jaic::ast::walk::{Node, walk};
use jaic::ast::{BinOp, Block, Decl, Expr, ExprKind as E, ProcHeader, Stmt, StmtKind as S, UnOp};
use jaic::sema::EntityId;
use jaic::types::TypeKind;

/// The rule's name, group and one-line summary, and its check.
pub(crate) const INFO: super::RuleInfo = super::RuleInfo {
    name: "leaked_allocation",
    category: crate::config::Category::Pedantic,
    summary: "a local holding memory from an `@Allocates` call that is never freed, returned or handed on",
    check,
};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro || changes_allocator(cx, p.header, &p.body.stmts) {
            continue;
        }
        let mut decls: Vec<(&Decl, &Expr)> = Vec::new();
        walk(&p.body.stmts, &mut |n, _| {
            if let Some(Stmt {
                kind: S::Decl(d), ..
            }) = n.stmt()
                && let Some(call) = allocation(cx, d)
            {
                decls.push((d, call));
            }
            true
        });
        for (d, call) in decls {
            let name = d.names[0];
            let Some(var) = (if let [only] = &*cx.locals(name.span) {
                Some(*only)
            } else {
                None
            }) else {
                continue;
            };
            if leaks(cx, p.body, d, var) {
                let callee = match &call.kind {
                    E::Call {
                        callee, ..
                    } => cx.whole_src(callee.span).trim(),
                    _ => "",
                };
                let message = format!(
                    "`{}` holds memory from `{callee}` that is never freed, returned or handed on",
                    name.name.as_str()
                );
                let help = format!(
                    "free it when done, for example `defer free({});` right after this line",
                    name.name.as_str()
                );
                let pointer = d
                    .value
                    .as_ref()
                    .and_then(|v| cx.ty(v))
                    .is_some_and(|t| cx.compiler.ty.types.is_pointer(t));
                out.push(if pointer {
                    let end = statement_range(cx, d.span).1;
                    let (_, indent) = cx.line_start(d.span.start as usize);
                    suggesting(
                        name.span,
                        message,
                        help,
                        "free it when the procedure ends",
                        vec![Edit {
                            start: end,
                            end,
                            text: format!("\n{indent}defer free({});", name.name.as_str()),
                        }],
                    )
                } else {
                    finding(name.span, message, Some(help))
                });
            }
        }
    }
}

/// The declaration names one local and initialises it from a call of an `@Allocates`
/// procedure, possibly cast: that call.
fn allocation<'a>(cx: &Cx, d: &'a Decl) -> Option<&'a Expr> {
    if d.names.len() != 1 || d.rare.is_some() || d.using || d.backtick {
        return None;
    }
    let mut value = d.value.as_ref()?;
    while let E::Cast {
        value: inner, ..
    } = &value.kind
    {
        value = inner;
    }
    let E::Call {
        callee,
        args,
        ..
    } = &value.kind
    else {
        return None;
    };
    (calls_noted(cx, callee, "Allocates") && !args.iter().any(|a| a.context)).then_some(value)
}

/// The procedure may run under another allocator, or does something the scan cannot follow.
pub(super) fn changes_allocator(cx: &Cx, header: &ProcHeader, body: &[Stmt]) -> bool {
    if header.params.iter().any(|q| q.using) {
        return true;
    }
    let mut found = false;
    walk(body, &mut |n, _| {
        match n {
            Node::Stmt(s) => match &s.kind {
                S::PushContext(_) | S::PushContextDefer(_) | S::Using(_) | S::Insert(_) => {
                    found = true;
                }
                S::Assign {
                    lhs, ..
                } => {
                    found |= lhs.iter().any(mentions_context);
                }
                _ => {}
            },
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

/// The access path starts at `context`.
fn mentions_context(e: &Expr) -> bool {
    match &e.kind {
        E::Context => true,
        E::Member(base, _) | E::Index(base, _) | E::Unary(UnOp::Deref, base) => {
            mentions_context(base)
        }
        _ => false,
    }
}

/// Every mention of `var` after its declaration is harmless, and none is hidden from the scan.
fn leaks(cx: &Cx, body: &Block, decl: &Decl, var: EntityId) -> bool {
    let name = decl.names[0].name;
    let after = decl.span.end;
    let mut handled = false;
    let mut seen = 0usize;
    walk(&body.stmts, &mut |n, chain| {
        if handled {
            return false;
        }
        let Node::Expr(e) = n else {
            return true;
        };
        if matches!(e.kind, E::Ident(sym) if sym == name) && e.span.start >= after {
            seen += 1;
            if cx.entities(e.span).contains(&var) && !harmless(cx, e, chain) {
                handled = true;
            }
        }
        true
    });
    // A mention the walk did not reach (a nested procedure, a code block) may free or keep it.
    let written = cx
        .name_tokens(name, after, body.span.end.max(after))
        .count();
    !handled && written == seen
}

/// A mention of the allocated local that neither frees it nor lets it escape.
fn harmless(cx: &Cx, e: &Expr, chain: &[Node<'_>]) -> bool {
    let mut cur = e;
    let mut i = chain.len();
    while i > 0 {
        let Node::Expr(p) = chain[i - 1] else {
            break;
        };
        match &p.kind {
            E::Member(base, _) | E::Index(base, _) if base.span == cur.span => cur = p,
            E::Unary(UnOp::Deref, inner) if inner.span == cur.span => cur = p,
            _ => break,
        }
        i -= 1;
    }
    let projected = cur.span != e.span;
    let parent = i.checked_sub(1).map(|k| chain[k]);
    match parent {
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
            E::Unary(UnOp::Star, _) => false,
            _ => projected && scalar(cx, cur),
        },
        None => false,
    }
}

/// The expression is a number, bool or enum: copying it keeps nothing alive.
fn scalar(cx: &Cx, e: &Expr) -> bool {
    let types = &cx.compiler.ty.types;
    cx.ty(e).is_some_and(|t| {
        matches!(
            types.kind(types.repr(t)),
            TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
        )
    })
}
