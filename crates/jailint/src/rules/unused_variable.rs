//! `unused_variable`: a local variable nothing reads or writes after declaring it.
//!
//! Usually a leftover from an edit, or a sign that the wrong variable is used further down.
//! Name it `_` (or start its name with `_`) when the value is deliberately dropped. Results
//! at the end of a multiple-value declaration can be left out instead: `a, unused := f();`
//! becomes `a := f();`.
//!
//! The compiler's record of what each name resolved to decides, across every instance of a
//! polymorphic body and through macro expansions elsewhere (a backticked name). As a guard
//! for code the compiler did not check (`#if` branches left out, `#asm`), a variable whose
//! name appears anywhere else in its block is never reported. Bodies that failed to check
//! and `#expand` macros are skipped.
use crate::facts::Recorded;
use crate::syntax::{Cx, Node, walk};
use crate::{Edit, Finding, Fix};
use jaic::ast::{DeclKind, Expr, ExprKind as E, StmtKind as S};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro {
            continue;
        }
        let body_span = p.body.span;
        walk(&p.body.stmts, &mut |n, chain| {
            let Some(s) = n.stmt() else {
                return true;
            };
            let S::Decl(d) = &s.kind else {
                return true;
            };
            if d.kind != DeclKind::Var || d.backtick || d.using || !d.notes.is_empty() {
                return true;
            }
            // Declarations inside `#run` / `#code` / `#insert` operands are not this body's.
            if chain.iter().any(|a| {
                matches!(
                    a,
                    Node::Expr(Expr {
                        kind: E::Run { .. } | E::Code(_) | E::Insert { .. },
                        ..
                    })
                )
            }) {
                return false;
            }
            let block = chain
                .iter()
                .rev()
                .find_map(|a| match a {
                    Node::Stmt(st) if matches!(st.kind, S::Block(_)) => Some(st.span),
                    _ => None,
                })
                .unwrap_or(body_span);
            // Code `#insert`ed into the block resolves its names there and may use any of
            // them; what it uses is not written in the block.
            if cx.has_insert(block.start, block.end) {
                return true;
            }
            let single = d.names.len() == 1;
            let unused: Vec<bool> = d
                .names
                .iter()
                .enumerate()
                .map(|(j, name)| {
                    d.existing.get(j) != Some(&true)
                        && !name.name.as_str().starts_with('_')
                        && cx.facts.locals.get(&name.span).is_some_and(|ents| {
                            !ents.iter().any(|&e| cx.compiler.is_used(e))
                                && !cx.mentions(name.name, block.start, block.end, name.span)
                        })
                })
                .collect();
            // `a, b := f();` where `b` and everything after it is unused: a call's later
            // results can simply be left out (`a := f();`).
            let droppable_tail = !single
                && d.value.is_some()
                && d.extra_values.is_empty()
                && d.ty.is_none()
                && d.existing.is_empty()
                && d.backtick_names.is_empty();
            for (j, name) in d.names.iter().enumerate() {
                if !unused[j] {
                    continue;
                }
                let pure = match &d.value {
                    None => true,
                    Some(v) => pure(v),
                };
                let (help, edits) = if single && pure && d.extra_values.is_empty() {
                    let (a, b) = cx.statement_removal(s.span);
                    (
                        "remove it".to_string(),
                        vec![Edit {
                            start: a,
                            end: b,
                            text: String::new(),
                        }],
                    )
                } else if droppable_tail && j > 0 && unused[j..].iter().all(|&u| u) {
                    (
                        "leave it out: later results of a call can be dropped".to_string(),
                        vec![Edit {
                            start: d.names[j - 1].span.end as usize,
                            end: name.span.end as usize,
                            text: String::new(),
                        }],
                    )
                } else if !single
                    && d.value.is_none()
                    && d.existing.is_empty()
                    && d.backtick_names.is_empty()
                {
                    // `a, b: T;`: drop the name from the list.
                    let (start, end) = if j > 0 {
                        (d.names[j - 1].span.end, name.span.end)
                    } else {
                        (name.span.start, d.names[1].span.start)
                    };
                    (
                        "remove it from the declaration".to_string(),
                        vec![Edit {
                            start: start as usize,
                            end: end as usize,
                            text: String::new(),
                        }],
                    )
                } else {
                    (
                        "name it `_` if the value is not needed".to_string(),
                        vec![Edit {
                            start: name.span.start as usize,
                            end: name.span.end as usize,
                            text: "_".into(),
                        }],
                    )
                };
                out.push(Finding {
                    start: name.span.start as usize,
                    end: name.span.end as usize,
                    message: format!("unused variable `{}`", name.name),
                    label: None,
                    help: Some(help.clone()),
                    fix: Some(Fix {
                        title: help,
                        edits,
                        machine_applicable: true,
                    }),
                });
            }
            true
        });
    }
}

/// Evaluating `e` has no effect beyond its value.
fn pure(e: &Expr) -> bool {
    match &e.kind {
        E::Int(_)
        | E::Float(_)
        | E::Str(_)
        | E::Bool(_)
        | E::Null
        | E::Uninit
        | E::Char(_)
        | E::Ident(_) => true,
        E::Member(base, _) => pure(base),
        E::Unary(_, a) => pure(a),
        E::Binary(_, a, b) => pure(a) && pure(b),
        E::StructLit {
            fields, ..
        } => fields.iter().all(|f| pure(&f.value)),
        E::ArrayLit {
            elems, ..
        } => elems.iter().all(pure),
        _ => false,
    }
}
