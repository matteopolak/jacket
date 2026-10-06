//! `unused_parameter`: a parameter the procedure's body never uses.
//!
//! Callers compute and pass a value for nothing, and readers wonder what it is for. Remove
//! it, or start its name with `_` when the signature is fixed by something else.
//!
//! Many signatures are fixed from outside, so this only looks at procedures whose every
//! caller is in view: named procedures that are only ever called (never taken as a value,
//! which could make them callbacks), that something calls, that are not overloaded (an
//! overload set shares a shape), not exported from a module (its users are elsewhere), and
//! not `#c_call`, `#program_export`, `#foreign`, `#expand`, operators or `for_expansion`.
//! Procedures with notes are skipped too: metaprograms find procedures by their notes and
//! call them with a fixed signature. Empty bodies (stubs), bodies with `#insert` (inserted
//! code may use any parameter) and parameters whose type has a `$` (the argument settles a
//! polymorphic type) are left alone.
use crate::Finding;
use crate::facts::Recorded;
use crate::syntax::{Cx, ProcSite};
use jaic::sema::scope::EntityKind;

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    for p in &cx.procs {
        if !p.clean || p.is_macro || !eligible(cx, p) {
            continue;
        }
        let h = p.header;
        // An empty body is a stub or a placeholder: its signature is the point.
        if p.body.stmts.is_empty() {
            continue;
        }
        // Code `#insert`ed into the body may use any parameter.
        if cx.tokens.iter().any(|t| {
            matches!(&t.tok, jaic::lexer::Tok::Directive(d) if d.as_str() == "insert")
                && p.body.span.start <= t.span.start
                && t.span.end <= p.body.span.end
        }) {
            continue;
        }
        for param in &h.params {
            let Some(name) = param.name else {
                continue;
            };
            if name.name.as_str().starts_with('_') || param.baked || param.auto_bake {
                continue;
            }
            // `x: [$N] $T`: the argument is how the call settles `N` and `T`.
            if cx.src(param.span).contains('$') {
                continue;
            }
            let Some(ents) = cx.facts.locals.get(&param.span) else {
                continue;
            };
            if ents.iter().any(|&e| cx.compiler.is_used(e))
                || cx.mentions(name.name, h.span.start, p.body.span.end, name.span)
            {
                continue;
            }
            out.push(Finding {
                start: name.span.start as usize,
                end: name.span.end as usize,
                message: format!("unused parameter `{}`", name.name),
                label: None,
                help: Some(
                    "remove it, or start its name with `_` if the signature must stay".into(),
                ),
                fix: None,
            });
        }
    }
}

fn eligible(cx: &Cx, p: &ProcSite) -> bool {
    let h = p.header;
    let f = &h.flags;
    let Some(decl) = p.decl else {
        return false;
    };
    if f.c_call
        || f.expand
        || f.compiler
        || f.intrinsic
        || f.elsewhere.is_some()
        || f.program_export.is_some()
        || f.cpp_method
        || f.runtime_support
        || h.foreign.is_some()
        || h.operator.is_some()
        || !h.notes.is_empty()
        || !decl.notes.is_empty()
        || h.params.iter().any(|q| !q.notes.is_empty())
    {
        return false;
    }
    let name = decl.names.first().map(|n| n.name.as_str()).unwrap_or("");
    if matches!(name, "main" | "for_expansion") || name.starts_with("operator") {
        return false;
    }
    let Some(ids) = cx.facts.procs_by_header.get(&h.span) else {
        return false;
    };
    let compiler = cx.compiler;
    let Some(file) = compiler.files.iter().find(|f| f.id == cx.file) else {
        return false;
    };
    let module_scope = compiler.modules[file.module.0 as usize].scope;
    let in_main = Some(file.module) == compiler.main_module;
    for &id in ids {
        let info = compiler.proc(id);
        // The name as the file and its module see it (a `#scope_file` overload lives in the
        // file's scope, a public one in the module's).
        let mut seen = Vec::new();
        for sc in [info.scope, file.scope, module_scope] {
            for &e in compiler
                .scope(sc)
                .names
                .get(&info.name)
                .into_iter()
                .flatten()
            {
                if !seen.contains(&e) {
                    seen.push(e);
                }
            }
        }
        // Overloads share a shape.
        if seen.len() > 1 {
            return false;
        }
        // Exported from a module: callers may be anywhere.
        let private = seen.iter().all(|&e| {
            let ent = compiler.entity(e);
            ent.file_private || matches!(ent.kind, EntityKind::Local { .. })
        });
        if !in_main && !private {
            return false;
        }
    }
    // Every mention is a call, and there is at least one.
    let mut calls = 0;
    for &id in ids {
        for use_ in cx.facts.proc_uses.get(&id).into_iter().flatten() {
            if !is_callee(cx, *use_) {
                return false;
            }
            calls += 1;
        }
    }
    calls > 0
}

/// The name at `span` is called: `(` follows it.
fn is_callee(cx: &Cx, span: jaic::source::Span) -> bool {
    let text = &cx.compiler.sources.get(span.file).text;
    text.get(span.end as usize..)
        .is_some_and(|rest| rest.trim_start().starts_with('('))
}
