//! `unused_import`: an `#import` that nothing in its scope uses.
//!
//! An unused import costs compile time (the module is loaded and checked) and suggests a
//! dependency that is not there.
//!
//! An import counts as used when the compiler found any name through it while checking
//! (name lookups record which import answered, so operators and `for_expansion` count too),
//! or when any identifier in the files that can see the import is a name the module
//! exports (a guard for code the compiler did not check). A named import
//! (`M :: #import "X"`) is used when `M` is. Imports with module parameters, imports of
//! modules with `#program_export` (they add entry points), and files with bodies the
//! compiler could not check completely are not reported, except for named imports.
use crate::facts::Recorded;
use crate::syntax::Cx;
use crate::{Edit, Finding, Fix};
use jaic::ast::{File, Import, ImportSource, StmtKind as S};
use jaic::fxhash::{HashMap, HashSet};
use jaic::intern::Sym;
use jaic::lexer::{Tok, Token};
use jaic::sema::scope::{EntityKind, Found};
use jaic::sema::{Compiler, ModuleId, ScopeId};
use jaic::source::{FileId, Span};
use std::path::PathBuf;

/// An import found unused, before the rule decides whether to report it.
pub(crate) struct Unused {
    span: Span,
    text: String,
    /// `M :: #import`: reported even where bodies did not check.
    named: bool,
}

/// Find the unused imports of `file` (needs the compiler mutably: module lookups may load).
pub(crate) fn prepare(
    compiler: &mut Compiler,
    file: FileId,
    ast: &File,
    tokens: &[Token],
) -> Vec<Unused> {
    let mut out = Vec::new();
    let Some(info) = compiler.files.iter().find(|f| f.id == file) else {
        return out;
    };
    let (file_scope, module) = (info.scope, info.module);
    let module_scope = compiler.modules[module.0 as usize].scope;
    let mut names: Option<HashMap<Sym, usize>> = None;
    for s in &ast.stmts {
        let S::Import(imp) = &s.kind else {
            continue;
        };
        if !imp.params.is_empty() || !imp.flags.is_empty() || imp.using.is_some() {
            continue;
        }
        let text = describe(imp);
        if let Some(name) = imp.name {
            let used = [file_scope, module_scope].iter().any(|&sc| {
                compiler.scope(sc).names.get(&name.name).is_some_and(|ids| {
                    ids.iter().any(|&e| {
                        matches!(compiler.entity(e).kind, EntityKind::Import(_))
                            && compiler.is_used(e)
                    })
                })
            }) || tokens
                .iter()
                .any(|t| matches!(t.tok, Tok::Ident(n) if n == name.name) && t.span != name.span)
                || {
                    // Another file of the module (or one it `#load`s under an `#if`) may use it.
                    let names = names
                        .get_or_insert_with(|| module_identifiers(compiler, module, file, tokens));
                    names.get(&name.name).is_some_and(|&n| n > 1)
                };
            let declared = [file_scope, module_scope].iter().any(|&sc| {
                compiler.scope(sc).names.get(&name.name).is_some_and(|ids| {
                    ids.iter()
                        .any(|&e| matches!(compiler.entity(e).kind, EntityKind::Import(_)))
                })
            });
            if declared && !used {
                out.push(Unused {
                    span: s.span,
                    text,
                    named: true,
                });
            }
            continue;
        }
        let Some((scope, index, m)) = entry(compiler, &[file_scope, module_scope], imp.span) else {
            continue;
        };
        if compiler
            .ide
            .as_ref()
            .is_some_and(|i| i.used_imports.contains(&(scope, index)))
            || exports_entry_points(compiler, m)
        {
            continue;
        }
        let names = names.get_or_insert_with(|| module_identifiers(compiler, module, file, tokens));
        let mut found = false;
        for &n in names.keys() {
            match compiler.module_lookup(m, n) {
                Ok(Found::Entities(ids)) if ids.is_empty() => {}
                Ok(_) => {
                    found = true;
                    break;
                }
                Err(_) => {}
            }
        }
        if !found {
            out.push(Unused {
                span: s.span,
                text,
                named: false,
            });
        }
    }
    out
}

fn describe(imp: &Import) -> String {
    match &imp.source {
        ImportSource::Module(m) => format!("\"{m}\""),
        ImportSource::File(f) => format!(",file \"{f}\""),
        ImportSource::Dir(d) => format!(",dir \"{d}\""),
        ImportSource::String(_) => ",string".into(),
    }
}

/// The import entry written at `span`: its scope, index and loaded module.
fn entry(
    compiler: &Compiler,
    scopes: &[ScopeId],
    span: Span,
) -> Option<(ScopeId, usize, ModuleId)> {
    for &sc in scopes {
        for (i, e) in compiler.scope(sc).imports.iter().enumerate() {
            if e.import.span == span {
                return Some((sc, i, e.module?));
            }
        }
    }
    None
}

/// The module adds program entry points (`#program_export`): importing it has an effect.
fn exports_entry_points(compiler: &Compiler, m: ModuleId) -> bool {
    compiler
        .files
        .iter()
        .filter(|f| f.module == m)
        .any(|f| compiler.sources.get(f.id).text.contains("#program_export"))
}

/// How often each identifier is written in the files of `module`, including files they
/// `#load` that the compiler left out (an `#if` for another platform). `file`'s own tokens
/// are `tokens`, so that file is not lexed again.
fn module_identifiers(
    compiler: &Compiler,
    module: ModuleId,
    file: FileId,
    tokens: &[Token],
) -> HashMap<Sym, usize> {
    let mut out = HashMap::default();
    let mut seen: HashSet<PathBuf> = HashSet::default();
    let mut queue: Vec<(PathBuf, String)> = Vec::new();
    for f in compiler.files.iter().filter(|f| f.module == module) {
        let src = compiler.sources.get(f.id);
        let path = PathBuf::from(&src.path);
        seen.insert(std::fs::canonicalize(&path).unwrap_or(path.clone()));
        if f.id == file {
            count(tokens, &path, &mut out, &mut seen, &mut queue);
        } else {
            queue.push((path, src.text.to_string()));
        }
    }
    while let Some((path, text)) = queue.pop() {
        let Ok(tokens) = jaic::lexer::lex(FileId(0), &text) else {
            continue;
        };
        count(&tokens, &path, &mut out, &mut seen, &mut queue);
    }
    out
}

/// Add the identifiers of a file (at `path`, with `tokens`) to `out`, and queue the files it
/// `#load`s that are not `seen` yet.
fn count(
    tokens: &[Token],
    path: &std::path::Path,
    out: &mut HashMap<Sym, usize>,
    seen: &mut HashSet<PathBuf>,
    queue: &mut Vec<(PathBuf, String)>,
) {
    let dir = path.parent().map(PathBuf::from).unwrap_or_default();
    for (i, t) in tokens.iter().enumerate() {
        match &t.tok {
            Tok::Ident(n) => *out.entry(*n).or_insert(0) += 1,
            Tok::Directive(d) if d.as_str() == "load" => {
                if let Some(Tok::Str(s)) = tokens.get(i + 1).map(|t| &t.tok) {
                    let target = dir.join(String::from_utf8_lossy(s).as_ref());
                    let key = std::fs::canonicalize(&target).unwrap_or(target.clone());
                    if seen.insert(key)
                        && let Ok(text) = std::fs::read_to_string(&target)
                    {
                        queue.push((target, text));
                    }
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    // A compile that stopped early (a failed `#assert`, an error) left code unchecked.
    let all_checked = cx.complete && cx.procs.iter().all(|p| p.clean || p.is_macro);
    for u in &cx.imports {
        if !u.named && !all_checked {
            continue;
        }
        let (a, b) = cx.statement_removal(u.span);
        out.push(Finding {
            start: u.span.start as usize,
            end: u.span.end as usize,
            message: format!("unused import {}", u.text),
            label: None,
            help: Some("remove it".into()),
            fix: Some(Fix {
                title: "remove the import".into(),
                edits: vec![Edit {
                    start: a,
                    end: b,
                    text: String::new(),
                }],
                machine_applicable: true,
            }),
        });
    }
}
