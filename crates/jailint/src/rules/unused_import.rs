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
use jaic::project::{self, LinkSource};
use jaic::sema::scope::{EntityKind, Found};
use jaic::sema::{Compiler, ModuleId, ScopeId};
use jaic::source::{FileId, Span};
use std::path::PathBuf;
use std::rc::Rc;

/// An import found unused, before the rule decides whether to report it.
pub(crate) struct Unused {
    span: Span,
    text: String,
    /// `M :: #import`: reported even where bodies did not check.
    named: bool,
}

/// What is learned about a module once and reused for each of its files.
#[derive(Default)]
pub(crate) struct ModuleFacts {
    identifiers: HashMap<ModuleId, Rc<HashMap<Sym, usize>>>,
    entry_points: HashMap<ModuleId, bool>,
}

/// Find the unused imports of `file` (needs the compiler mutably: module lookups may load).
pub(crate) fn prepare(
    compiler: &mut Compiler,
    cache: &mut ModuleFacts,
    file: FileId,
    ast: &File,
    tokens: &[Token],
) -> Vec<Unused> {
    let mut out = Vec::new();
    let Some(info) = compiler.src.files.iter().find(|f| f.id == file) else {
        return out;
    };
    let (file_scope, module) = (info.scope, info.module);
    let module_scope = compiler.ns.modules[module.0 as usize].scope;
    let mut names: Option<Rc<HashMap<Sym, usize>>> = None;
    for s in &ast.stmts {
        let S::Import(imp) = &s.kind else {
            continue;
        };
        // The expression that holds an expression's import uses it, and removing the directive
        // from the middle of the expression would leave the rest of it behind.
        if !imp.params.is_empty()
            || !imp.flags.is_empty()
            || imp.using.is_some()
            || imp.is_expression()
        {
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
                    let names = names.get_or_insert_with(|| {
                        module_identifiers(compiler, cache, module, file, tokens)
                    });
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
            .ed
            .facts()
            .is_some_and(|i| i.used_imports.contains(&(scope, index)))
            || exports_entry_points(compiler, cache, m)
        {
            continue;
        }
        let names =
            names.get_or_insert_with(|| module_identifiers(compiler, cache, module, file, tokens));
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
fn exports_entry_points(compiler: &Compiler, cache: &mut ModuleFacts, m: ModuleId) -> bool {
    *cache.entry_points.entry(m).or_insert_with(|| {
        compiler
            .src
            .files
            .iter()
            .filter(|f| f.module == m)
            // A file that does not lex is assumed to export: importing it may have an effect.
            .any(|f| {
                file_facts(&compiler.src.sources.get(f.id).text).is_none_or(|k| k.program_export)
            })
    })
}

/// How often each identifier is written in the files of `module`, including files they
/// `#load` that the compiler left out (an `#if` for another platform). `file`'s own tokens
/// are `tokens`, so that file is not lexed again. The same for every file of the module, so
/// it is worked out once, for whichever file asks first.
fn module_identifiers(
    compiler: &Compiler,
    cache: &mut ModuleFacts,
    module: ModuleId,
    file: FileId,
    tokens: &[Token],
) -> Rc<HashMap<Sym, usize>> {
    if let Some(found) = cache.identifiers.get(&module) {
        return found.clone();
    }
    let mut out = HashMap::default();
    let mut seen: HashSet<PathBuf> = HashSet::default();
    let mut queue: Vec<(PathBuf, String)> = Vec::new();
    let mut own: Option<PathBuf> = None;
    for f in compiler.src.files.iter().filter(|f| f.module == module) {
        let src = compiler.src.sources.get(f.id);
        let Some(path) = src.path.as_file().map(std::path::Path::to_path_buf) else {
            continue;
        };
        // Every file of the module counts once, however it is reached.
        seen.insert(jaic::canonicalize(&path).unwrap_or(path.clone()));
        if f.id == file {
            own = Some(path);
        } else {
            queue.push((path, src.text.to_string()));
        }
    }
    if let Some(path) = own {
        apply(
            &FileFacts::of(tokens),
            &path,
            &mut out,
            &mut seen,
            &mut queue,
        );
    }
    while let Some((path, text)) = queue.pop() {
        let Some(facts) = file_facts(&text) else {
            continue;
        };
        apply(&facts, &path, &mut out, &mut seen, &mut queue);
    }
    let out = Rc::new(out);
    cache.identifiers.insert(module, out.clone());
    out
}

/// The identifiers a file writes (and how often) and the files it names with `#load`, from its
/// tokens alone,
/// and whether it exports program entry points.
struct FileFacts {
    identifiers: Vec<(Sym, usize)>,
    links: Vec<project::DirectiveLink>,
    /// It writes `#program_export`.
    program_export: bool,
}

impl FileFacts {
    fn of(tokens: &[Token]) -> Self {
        let mut counts: HashMap<Sym, usize> = HashMap::default();
        for t in tokens {
            if let Tok::Ident(n) = &t.tok {
                *counts.entry(*n).or_insert(0) += 1;
            }
        }
        let links = project::scan_directives(tokens)
            .into_iter()
            .filter(|l| matches!(l.source, LinkSource::Load(_)))
            .collect();
        FileFacts {
            identifiers: counts.into_iter().collect(),
            links,
            program_export: tokens
                .iter()
                .any(|t| matches!(&t.tok, Tok::Directive(d) if d.as_str() == "program_export")),
        }
    }
}

thread_local! {
    /// What the files lexed lately held, by hash of their text. A lint run on every edit looks
    /// at the other files of the module, which do not change between edits; lexing them was a
    /// third of a lint of a large program.
    static FILES: std::cell::RefCell<HashMap<(u64, usize), Rc<FileFacts>>> =
        std::cell::RefCell::new(HashMap::default());
}

/// The most files remembered.
const FILES_KEPT: usize = 256;

/// `FileFacts` of `text`, which must lex.
fn file_facts(text: &str) -> Option<Rc<FileFacts>> {
    use std::hash::{Hash, Hasher};
    let mut h = jaic::fxhash::FxHasher::default();
    text.hash(&mut h);
    let key = (h.finish(), text.len());
    if let Some(found) = FILES.with(|f| f.borrow().get(&key).cloned()) {
        return Some(found);
    }
    let tokens = jaic::lexer::lex(FileId(0), text).ok()?;
    let facts = Rc::new(FileFacts::of(&tokens));
    FILES.with(|f| {
        let mut f = f.borrow_mut();
        if f.len() >= FILES_KEPT {
            f.clear();
        }
        f.insert(key, facts.clone());
    });
    Some(facts)
}

/// Add the identifiers of a file (at `path`) to `out`, and queue the files it `#load`s that
/// are not `seen` yet.
fn apply(
    facts: &FileFacts,
    path: &std::path::Path,
    out: &mut HashMap<Sym, usize>,
    seen: &mut HashSet<PathBuf>,
    queue: &mut Vec<(PathBuf, String)>,
) {
    for (n, count) in &facts.identifiers {
        *out.entry(*n).or_insert(0) += count;
    }
    let mut links = facts.links.clone();
    project::resolve_targets(
        &mut links,
        path.parent().unwrap_or(std::path::Path::new("")),
    );
    for link in links {
        let Some(target) = link.target else {
            continue;
        };
        let key = jaic::canonicalize(&target).unwrap_or(target.clone());
        if seen.insert(key)
            && let Ok(text) = std::fs::read_to_string(&target)
        {
            queue.push((target, text));
        }
    }
}

pub(crate) fn check(cx: &Cx, out: &mut Vec<Finding>) {
    // Generators import what their output might need.
    if cx.generated() {
        return;
    }
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
