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
//! A procedure with a variant is skipped as well: platform variants behind `#if` and
//! build-selected backends in other files share one signature, so a parameter one variant
//! ignores is another's. A variant is another definition of the same name whose parameters
//! have the same names in the same order (types may differ across platforms), either in the
//! same file (the `#if` branch the build did not select) or in a `.jai` file the program does
//! not load: first the siblings in the file's directory, then the files of the directory above
//! and of its immediate subdirectories, parsed once per directory. Overload sets are skipped
//! by the compiler's own view of the scope.
use crate::Finding;
use crate::facts::Recorded;
use crate::syntax::{Cx, Node, ProcSite};
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
        let proc_name = p
            .decl
            .and_then(|d| d.names.first())
            .map_or("", |n| n.name.as_str());
        // Looked up lazily: it reads files, and most procedures have no unused parameter.
        let mut variant = None;
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
            if *variant.get_or_insert_with(|| has_variant(cx, p, proc_name)) {
                break;
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

/// A procedure's parameter names in order (`""` for an unnamed one).
type Signature = Vec<String>;

/// The procedure definitions found in the `.jai` files around a directory, by file.
type Variants = Vec<(PathBuf, HashMap<String, Vec<Signature>>)>;

/// Most files looked at for one directory.
const MAX_FILES: usize = 300;

/// Largest file parsed (bytes).
const MAX_FILE_SIZE: u64 = 1 << 20;

fn signature(header: &jaic::ast::ProcHeader) -> Signature {
    header
        .params
        .iter()
        .map(|p| {
            p.name
                .map_or(String::new(), |n| n.name.as_str().to_string())
        })
        .collect()
}

/// Another definition of the procedure `name` takes the same parameter names in the same
/// order: a variant of it in the same file (`#if` branches the build did not select) or in a
/// file this program does not load.
fn has_variant(cx: &Cx, own: &ProcSite, name: &str) -> bool {
    let sig = signature(own.header);
    let same_file = cx.procs.iter().any(|q| {
        !std::ptr::eq(q.header, own.header)
            && q.decl
                .and_then(|d| d.names.first())
                .is_some_and(|n| n.name.as_str() == name)
            && signature(q.header) == sig
    });
    same_file || variant_in_unloaded_file(cx, name, &sig)
}

/// Files near this one that the program does not load define `name` with signature `sig`.
/// The candidates are the siblings in the file's directory, then the files of the directory
/// above and of its immediate subdirectories (`src/platform/macos.jai` and `windows.jai`, or
/// `utils/macos_window.jai` and `utils/win32_window.jai`). They are parsed once per directory.
fn variant_in_unloaded_file(cx: &Cx, name: &str, sig: &Signature) -> bool {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<Variants>>>> = OnceLock::new();
    let Some(file) = cx.compiler.sources.try_get(cx.file) else {
        return false;
    };
    let path = Path::new(&file.path);
    let Some(dir) = path.parent() else {
        return false;
    };
    let cache = CACHE.get_or_init(Mutex::default);
    let cached = cache.lock().ok().and_then(|c| c.get(dir).cloned());
    let variants = cached.unwrap_or_else(|| {
        let v = Arc::new(load_variants(dir));
        if let Ok(mut c) = cache.lock() {
            c.insert(dir.to_path_buf(), v.clone());
        }
        v
    });
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let own = canon(path);
    let loaded: Vec<PathBuf> = cx.compiler.files.iter().map(|f| canon(&f.path)).collect();
    variants.iter().any(|(p, defs)| {
        let p = canon(p);
        p != own
            && !loaded.contains(&p)
            && defs.get(name).is_some_and(|l| l.iter().any(|s| s == sig))
    })
}

fn load_variants(dir: &Path) -> Variants {
    let mut dirs = vec![dir.to_path_buf()];
    if let Some(up) = dir.parent() {
        dirs.push(up.to_path_buf());
        let mut subs: Vec<PathBuf> = std::fs::read_dir(up)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir() && p != dir && !is_hidden(p))
            .collect();
        subs.sort();
        dirs.extend(subs);
    }
    let mut out = Variants::new();
    let mut budget = MAX_FILES;
    for d in dirs {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&d)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().is_some_and(|e| e == "jai")
                    && !p.to_string_lossy().ends_with(".fixed.jai")
                    && !is_hidden(p)
                    && p.metadata()
                        .is_ok_and(|m| m.is_file() && m.len() <= MAX_FILE_SIZE)
            })
            .collect();
        files.sort();
        for f in files {
            if budget == 0 {
                return out;
            }
            budget -= 1;
            let Ok(text) = std::fs::read_to_string(&f) else {
                continue;
            };
            let Ok(ast) = jaic::parser::parse_file(jaic::source::FileId(u32::MAX), &text) else {
                continue;
            };
            let mut defs = HashMap::new();
            top_level(&ast.stmts, &mut defs);
            if !defs.is_empty() {
                out.push((f, defs));
            }
        }
    }
    out
}

fn is_hidden(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_none_or(|n| n.starts_with('.'))
}

/// The procedures declared under `list`: at file scope, inside `#if` branches and blocks, and
/// nested in other procedures (`wt_idx` inside a platform's `read_hidden_weights`).
fn top_level(list: &[jaic::ast::Stmt], defs: &mut HashMap<String, Vec<Signature>>) {
    use jaic::ast::{ExprKind as E, StmtKind as S};
    crate::syntax::walk(list, &mut |node, _| {
        match node {
            Node::Stmt(s) => {
                if let S::Decl(d) = &s.kind
                    && let (Some(n), Some(v)) = (d.names.first(), &d.value)
                    && let E::Proc(lit) = &v.kind
                {
                    defs.entry(n.name.as_str().to_string())
                        .or_default()
                        .push(signature(&lit.header));
                }
            }
            Node::Expr(x) => {
                if let E::Proc(lit) = &x.kind
                    && let Some(body) = &lit.body
                {
                    top_level(&body.stmts, defs);
                }
            }
        }
        true
    });
}
