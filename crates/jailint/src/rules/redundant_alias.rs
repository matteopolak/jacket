//! `redundant_alias`: a local that exists only to be copied into another local, or returned.
//!
//! ```jai
//! total := a + b;      // fix: result := a + b;
//! result := total;
//!
//! x := compute();      // fix: return compute();
//! return x;
//! ```
//!
//! A style rule: the first name may have been chosen to describe the value. The conditions are
//! deliberately narrow so that applying the fix never changes behaviour or types:
//!
//! - The two statements are adjacent in one statement list, so the initializer is not
//!   evaluated any earlier or later relative to other code, and nothing sits between them
//!   (not even a comment).
//! - The first declares one plain local with an initializer (not `---`, no `#align`, notes
//!   or flags), outside `#run` / `#code` / `#insert` operands and macro bodies. Its name is
//!   written exactly twice in the procedure (declaration and the one use), which rules out
//!   address-taking, mutation, captures by nested procedures, `defer`, backticks and
//!   `#insert` in one go. The use must resolve to that local.
//! - Copy: `b := a;` with no explicit type, one name, and `b` not named in `a`'s
//!   initializer. The fix keeps the name `b` on the declaration of `a`.
//! - Return: `return x;` as the only returned value of a procedure with one unnamed return
//!   whose type is exactly `x`'s type (no implicit conversion), `x` declared without an
//!   explicit type, and an initializer that is not a call of something returning several
//!   values or of an unresolved callee.
//!
//! Macros, files that say they are generated and bodies that did not check are skipped.
use super::{callee_procs, statement_lists};
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{Decl, DeclKind, Expr, ExprKind as E, Stmt, StmtKind as S};
use jaic::sema::EntityId;
use jaic::sema::scope::EntityKind;
use jaic::source::Span;
use jaic::types::TypeId;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let mut opaque: Vec<Span> = Vec::new();
        walk(&p.body.stmts, &mut |n, _| {
            if let Node::Expr(e) = n
                && matches!(e.kind, E::Run { .. } | E::Code(_) | E::Insert { .. })
            {
                opaque.push(e.span);
            }
            true
        });
        let return_tys = return_types(cx, p.header.span);
        let body = p.body.span;
        statement_lists(&p.body.stmts, &mut |list| {
            for pair in list.windows(2) {
                let (first, next) = (&pair[0], &pair[1]);
                if opaque
                    .iter()
                    .any(|o| o.start <= first.span.start && next.span.end <= o.end)
                {
                    continue;
                }
                let S::Decl(a) = &first.kind else {
                    continue;
                };
                let Some(a) = alias_candidate(cx, first, a, next, body) else {
                    continue;
                };
                match &next.kind {
                    S::Decl(b) => copy(cx, a, b, out),
                    S::Return {
                        values,
                        backtick: false,
                    } => {
                        if let [v] = values.as_slice()
                            && v.name.is_none()
                            && v.target.is_none()
                            && !v.spread
                            && !v.context
                        {
                            returned(cx, a, &v.value, return_tys.as_deref(), out);
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

struct Candidate<'a> {
    decl: &'a Decl,
    init: &'a Expr,
    /// Types of the local, per check of the body.
    tys: Vec<TypeId>,
    ents: Vec<EntityId>,
}

/// `first` declares a plain local with an initializer, only written twice in the body, with
/// nothing but whitespace and `;` up to `next`.
fn alias_candidate<'a>(
    cx: &Cx,
    first: &Stmt,
    d: &'a Decl,
    next: &Stmt,
    body: Span,
) -> Option<Candidate<'a>> {
    let init = d.value.as_ref()?;
    if !plain(d) || !first.notes.is_empty() || matches!(init.kind, E::Uninit) {
        return None;
    }
    let name = &d.names[0];
    let between = cx
        .text
        .get(first.span.end as usize..next.span.start as usize)?;
    if !between.chars().all(|c| c.is_whitespace() || c == ';') {
        return None;
    }
    if cx.has_insert(body.start, body.end) {
        return None;
    }
    if cx.name_tokens(name.name, body.start, body.end).count() != 2 {
        return None;
    }
    let ents = cx.facts.locals.get(&name.span)?.clone();
    let tys = local_types(cx, &ents)?;
    Some(Candidate {
        decl: d,
        init,
        tys,
        ents,
    })
}

/// A declaration of one ordinary variable: no `using`, backtick, notes or flags.
fn plain(d: &Decl) -> bool {
    d.kind == DeclKind::Var
        && !d.backtick
        && !d.using
        && !d.as_
        && d.names.len() == 1
        && d.extra_values.is_empty()
        && d.existing.is_empty()
        && d.backtick_names.is_empty()
        && d.notes.is_empty()
        && d.flags.is_empty()
        && d.align.is_none()
        && d.foreign.is_none()
        && d.using_filter.is_none()
}

fn local_types(cx: &Cx, ents: &[EntityId]) -> Option<Vec<TypeId>> {
    let mut tys = Vec::new();
    for &e in ents {
        match cx.compiler.entity(e).kind {
            EntityKind::Local {
                ty, ..
            } => tys.push(ty),
            _ => return None,
        }
    }
    tys.sort_by_key(|t| t.0);
    tys.dedup();
    (!tys.is_empty()).then_some(tys)
}

/// The use resolves to exactly the candidate's local.
fn is_use(cx: &Cx, a: &Candidate, e: &Expr) -> bool {
    let E::Ident(n) = e.kind else {
        return false;
    };
    n == a.decl.names[0].name
        && !cx.entities(e.span).is_empty()
        && cx.entities(e.span).iter().all(|x| a.ents.contains(x))
}

fn copy(cx: &Cx, a: Candidate, b: &Decl, out: &mut Vec<Finding>) {
    let Some(v) = &b.value else {
        return;
    };
    if !plain(b) || b.ty.is_some() || !is_use(cx, &a, v) {
        return;
    }
    let bname = &b.names[0];
    let (init_start, init_end) = cx.whole(a.init.span);
    // `b` named in the initializer would mean the outer `b` there, and the new one after.
    if cx
        .name_tokens(bname.name, init_start as u32, init_end as u32)
        .next()
        .is_some()
    {
        return;
    }
    let Some(btys) = cx
        .facts
        .locals
        .get(&bname.span)
        .and_then(|e| local_types(cx, e))
    else {
        return;
    };
    if btys != a.tys {
        return;
    }
    let aname = &a.decl.names[0];
    let (_, use_end) = cx.whole(v.span);
    push(
        aname.span,
        format!(
            "`{}` only exists to be copied into `{}`",
            aname.name, bname.name
        ),
        format!("declare `{}` with the value directly", bname.name),
        vec![
            Edit {
                start: aname.span.start as usize,
                end: aname.span.end as usize,
                text: bname.name.as_str().to_string(),
            },
            Edit {
                start: init_end,
                end: use_end,
                text: String::new(),
            },
        ],
        out,
    );
}

fn returned(
    cx: &Cx,
    a: Candidate,
    v: &Expr,
    return_tys: Option<&[TypeId]>,
    out: &mut Vec<Finding>,
) {
    if a.decl.ty.is_some() || !is_use(cx, &a, v) {
        return;
    }
    // The returned type is the local's type: no conversion happens at the `return`.
    let Some(rt) = return_tys else {
        return;
    };
    if rt != a.tys.as_slice() {
        return;
    }
    // A call's several results must not all end up in the `return`.
    if let E::Call {
        callee, ..
    } = &a.init.kind
    {
        let procs = callee_procs(cx, callee);
        if procs.is_empty()
            || procs
                .iter()
                .any(|&p| cx.compiler.procs[p.0 as usize].lit.header.returns.len() != 1)
        {
            return;
        }
    }
    let aname = &a.decl.names[0];
    let (init_start, init_end) = cx.whole(a.init.span);
    let (_, use_end) = cx.whole(v.span);
    push(
        aname.span,
        format!("`{}` is returned right after it is declared", aname.name),
        "return the value directly".into(),
        vec![Edit {
            start: aname.span.start as usize,
            end: use_end,
            text: format!("return {}", &cx.text[init_start..init_end]),
        }],
        out,
    );
}

fn push(span: Span, message: String, help: String, edits: Vec<Edit>, out: &mut Vec<Finding>) {
    out.push(Finding {
        start: span.start as usize,
        end: span.end as usize,
        message,
        label: None,
        help: Some(help.clone()),
        fix: Some(Fix {
            title: help,
            edits,
            machine_applicable: true,
        }),
    });
}

/// The type of the procedure's only return value, per check; `None` for any other shape
/// (several returns, a named one, an unresolved signature).
fn return_types(cx: &Cx, header: Span) -> Option<Vec<TypeId>> {
    let procs = cx.facts.procs_by_header.get(&header)?;
    let mut tys = Vec::new();
    for &p in procs {
        let info = &cx.compiler.procs[p.0 as usize];
        let sig = info.sig.as_ref()?;
        if sig.returns.len() != 1 || sig.return_names.iter().any(|n| n.is_some()) {
            return None;
        }
        tys.push(sig.returns[0]);
    }
    tys.sort_by_key(|t| t.0);
    tys.dedup();
    (!tys.is_empty()).then_some(tys)
}
