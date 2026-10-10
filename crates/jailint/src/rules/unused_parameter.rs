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
//!
//! A procedure whose name is defined more than once nearby is skipped as well: platform
//! variants behind `#if`, build-selected backends in sibling files and overload sets all
//! share one signature, so a parameter one variant ignores is another's. The build may load
//! only one variant, so the compiler cannot see the rest; instead the `.jai` files under the
//! file's own directory (a few levels deep, scanned once per directory) are searched for
//! other `name :: (` definitions.
use crate::Finding;
use crate::facts::Recorded;
use crate::syntax::{Cx, ProcSite};
use jaic::sema::scope::EntityKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    if cx.generated() {
        return;
    }
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
        if cx.has_insert(p.body.span.start, p.body.span.end) {
            continue;
        }
        let name_shared = p
            .decl
            .and_then(|d| d.names.first())
            .is_some_and(|n| defined_nearby(cx, n.name.as_str()) > 1);
        if name_shared {
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
    let Some(file) = cx.compiler.sources.try_get(span.file) else {
        return false;
    };
    file.text
        .get(span.end as usize..)
        .is_some_and(|rest| rest.trim_start().starts_with('('))
}

type Definitions = HashMap<String, usize>;

/// How many `name :: (` definitions the `.jai` files around this one hold (this one's
/// included). Cached per directory.
fn defined_nearby(cx: &Cx, name: &str) -> usize {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<Definitions>>>> = OnceLock::new();
    let Some(file) = cx.compiler.sources.try_get(cx.file) else {
        return 0;
    };
    let Some(dir) = Path::new(&file.path).parent() else {
        return 0;
    };
    let cache = CACHE.get_or_init(Mutex::default);
    let cached = cache.lock().ok().and_then(|c| c.get(dir).cloned());
    let defs = cached.unwrap_or_else(|| {
        let mut defs = Definitions::new();
        let mut budget = 2000;
        scan_dir(dir, 3, &mut budget, &mut defs);
        let defs = Arc::new(defs);
        if let Ok(mut c) = cache.lock() {
            c.insert(dir.to_path_buf(), defs.clone());
        }
        defs
    });
    defs.get(name).copied().unwrap_or(0)
}

fn scan_dir(dir: &Path, depth: u32, budget: &mut u32, defs: &mut Definitions) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        let hidden = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_none_or(|n| n.starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            if depth > 0 {
                scan_dir(&path, depth - 1, budget, defs);
            }
        } else if path.extension().is_some_and(|e| e == "jai")
            && !path.to_string_lossy().ends_with(".fixed.jai")
        {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if let Ok(text) = std::fs::read_to_string(&path) {
                for line in text.lines() {
                    if let Some(name) = definition(line) {
                        *defs.entry(name.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
}

/// The name a line defines as a procedure: `name :: (`, `name :: inline (`.
fn definition(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let end = line
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(line.len());
    if end == 0 {
        return None;
    }
    let rest = line[end..].trim_start().strip_prefix("::")?.trim_start();
    let rest = rest
        .strip_prefix("inline")
        .or_else(|| rest.strip_prefix("no_inline"))
        .map_or(rest, str::trim_start);
    rest.starts_with('(').then_some(&line[..end])
}
