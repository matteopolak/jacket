//! The editor facts a few programs record, compared with saved dumps. Every fact list is
//! sorted, so only what is recorded counts, not the order it was recorded in. Ids are written
//! as what they name (`E(name@file:offset)`), never as numbers: those depend on what the
//! host's library declared first.
//!
//! Library locations are written as the file alone (a procedure also gets `#n` among its
//! same-named siblings in that file), so editing the library never rewrites a dump.
//!
//! The dumps are `insta` snapshots in `tests/snapshots/`. After an intended change to what is
//! recorded, rewrite them with `INSTA_UPDATE=always cargo test -p jaic --test ide_facts_golden`
//! (or `cargo insta review`) and read the diff.
use jaic::fxhash::HashMap;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, EntityId, NativeFs, Options, ProcId, ProgramSource, ScopeId};
use jaic::source::{FileId, Span};
use jaic::types::{StructId, TypeId};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn repo() -> PathBuf {
    test_support::repo::root()
}

/// The facts recorded for `entry` and the files under `root`, as an editor analysis records
/// them: compile, check the rest, sketch the polymorphs nothing called.
fn facts(entry: &Path, root: &Path) -> String {
    let stdlib = repo().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&jaic::sema::NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let mut ide = IdeFacts::new(vec![IdeRoot::under(root)]);
    ide.lint = true;
    compiler.ed.record(ide);
    let mut errors: Vec<String> = Vec::new();
    if let Err(e) = compiler
        .begin_sources(&[ProgramSource::File(entry.to_path_buf())])
        .and_then(|()| compiler.finish_program())
    {
        errors.push(e.message.clone());
    }
    errors.extend(compiler.ide_check_all().into_iter().map(|e| e.message));
    compiler.ide_sketch_polymorphs();
    dump(&compiler, errors)
}

/// `FileId(n)`, `EntityId(n)` and the other numbered ids in `line`, as what they name.
fn symbolic(c: &Compiler, scopes: &HashMap<ScopeId, Span>, line: &str) -> String {
    const IDS: [&str; 6] = [
        "FileId", "EntityId", "ProcId", "TypeId", "ScopeId", "StructId",
    ];
    let mut out = String::new();
    let mut rest = line;
    while let Some((at, kind)) = IDS
        .iter()
        .filter_map(|kind| {
            rest.match_indices(&format!("{kind}("))
                .find(|(i, _)| !rest[..*i].ends_with(|ch: char| ch.is_alphanumeric()))
                .map(|(i, _)| (i, *kind))
        })
        .min()
    {
        let digits = &rest[at + kind.len() + 1..];
        let len = digits
            .find(|ch: char| !ch.is_ascii_digit())
            .unwrap_or(digits.len());
        if len == 0 || !digits[len..].starts_with(')') {
            out.push_str(&rest[..=at + kind.len()]);
            rest = digits;
            continue;
        }
        let n: u32 = digits[..len].parse().unwrap();
        out.push_str(&rest[..at]);
        out.push_str(&match kind {
            "FileId" => file(c, FileId(n)),
            "EntityId" => {
                let e = c.entity(EntityId(n));
                format!(
                    "E({}@{})",
                    e.name.to_string().escape_debug(),
                    at_span(c, e.span)
                )
            }
            "ProcId" => {
                let p = c.proc(ProcId(n));
                format!(
                    "P({}@{})",
                    p.name.to_string().escape_debug(),
                    proc_at(c, ProcId(n))
                )
            }
            "TypeId" => format!("T({})", c.ty.types.name(TypeId(n))),
            "ScopeId" => scopes.get(&ScopeId(n)).map_or_else(
                || format!("S({:?})", c.scope(ScopeId(n)).kind),
                |s| format!("S({})", span(c, *s)),
            ),
            _ => {
                let s = c.ty.types.struct_info(StructId(n));
                format!(
                    "St({}@{})",
                    s.name.to_string().escape_debug(),
                    at_span(c, s.span)
                )
            }
        });
        rest = &digits[len + 1..];
    }
    out.push_str(rest);
    out
}

/// A file as its path under the repository.
fn file(c: &Compiler, f: FileId) -> String {
    let repo = repo();
    match c.src.sources.get(f).path.as_file() {
        Some(path) => path
            .strip_prefix(&repo)
            .unwrap_or(path)
            .display()
            .to_string(),
        None => "virtual".into(),
    }
}

/// Whether `f` is a library file. Where something sits in one moves whenever the library is
/// edited, so the dumps pin it no further than the file.
fn is_library(c: &Compiler, f: FileId) -> bool {
    c.src
        .sources
        .get(f)
        .path
        .as_file()
        .is_some_and(|p| p.starts_with(repo().join("stdlib")))
}

/// Where a procedure is declared. A library one is its file and, when the file declares other
/// procedures of that name, `#n`: its rank among them by position, so an overload set stays
/// distinguishable (and in a stable order) without pinning any offset.
fn proc_at(c: &Compiler, id: ProcId) -> String {
    let p = c.proc(id);
    if p.span == Span::NONE || !is_library(c, p.span.file) {
        return at_span(c, p.span);
    }
    let mut starts: Vec<u32> = (0..c.pr.procs.len() as u32)
        .map(|i| c.proc(ProcId(i)))
        .filter(|q| q.name == p.name && q.span.file == p.span.file)
        .map(|q| q.span.start)
        .collect();
    starts.sort_unstable();
    starts.dedup();
    let file = file(c, p.span.file);
    if starts.len() < 2 {
        return file;
    }
    let rank = starts.iter().position(|&s| s == p.span.start).unwrap() + 1;
    format!("{file}#{rank}")
}

fn span(c: &Compiler, s: Span) -> String {
    if s == Span::NONE {
        return "-".into();
    }
    if is_library(c, s.file) {
        return file(c, s.file);
    }
    format!("{}:{}..{}", file(c, s.file), s.start, s.end)
}

fn at_span(c: &Compiler, s: Span) -> String {
    if s == Span::NONE {
        return "-".into();
    }
    if is_library(c, s.file) {
        return file(c, s.file);
    }
    format!("{}:{}", file(c, s.file), s.start)
}

fn dump(c: &Compiler, errors: Vec<String>) -> String {
    let ide = c.ed.facts().unwrap();
    let scopes: HashMap<ScopeId, Span> = ide.scopes.iter().map(|(s, id)| (*id, *s)).collect();
    let span = |s: &Span| span(c, *s);
    let ty = |t: &TypeId| c.ty.types.name(*t);
    let opt_ty = |t: &Option<TypeId>| t.as_ref().map_or("-".into(), ty);
    let mut sections: Vec<(&str, Vec<String>)> = vec![
        ("errors", errors),
        (
            "refs",
            ide.refs
                .iter()
                .map(|r| {
                    format!(
                        "{} {:?} {} decl={}",
                        span(&r.span),
                        r.what,
                        ty(&r.ty),
                        r.decl
                    )
                })
                .collect(),
        ),
        (
            "scopes",
            ide.scopes
                .iter()
                .map(|(s, id)| format!("{} {id:?}", span(s)))
                .collect(),
        ),
        (
            "calls",
            ide.calls.iter().map(|call| format!("{call:?}")).collect(),
        ),
        (
            "expansions",
            ide.expansions.iter().map(|e| format!("{e:?}")).collect(),
        ),
        ("skipped", ide.skipped.iter().map(span).collect()),
        (
            "static_branches",
            ide.static_branches
                .iter()
                .map(|(s, ran)| format!("{} {ran}", span(s)))
                .collect(),
        ),
        (
            "body_evals",
            ide.body_evals
                .iter()
                .map(|(s, n)| format!("{} {n}", span(s)))
                .collect(),
        ),
        (
            "dead_runs",
            ide.dead_runs
                .iter()
                .map(|((body, run), n)| format!("{} {} {n}", span(body), span(run)))
                .collect(),
        ),
        (
            "exprs",
            ide.exprs
                .iter()
                .map(|(s, e)| format!("{} {} {} {}", span(s), ty(&e.ty), e.constant, e.conflicting))
                .collect(),
        ),
        (
            "casts",
            ide.casts
                .iter()
                .map(|(s, e)| {
                    let (target, from) = (ty(&e.target), ty(&e.from));
                    format!(
                        "{} {target} {from} {} {}",
                        span(s),
                        e.from_constant,
                        e.conflicting
                    )
                })
                .collect(),
        ),
        // Entities of the recorded files that something used (the library's are not listed).
        (
            "used",
            ide.used
                .iter()
                .filter(|&&e| {
                    ide.refs.iter().any(|r| {
                        r.decl && matches!(r.what, jaic::ide::IdeWhat::Entity(d) if d == e)
                    })
                })
                .map(|e| format!("{e:?}"))
                .collect(),
        ),
        (
            "used_imports",
            ide.used_imports.iter().map(|u| format!("{u:?}")).collect(),
        ),
        (
            "receivers",
            ide.receivers
                .iter()
                .map(|(s, t)| format!("{} {}", span(s), opt_ty(t)))
                .collect(),
        ),
        (
            "instances",
            ide.instances
                .iter()
                .map(|(s, t)| format!("{} {}", span(s), opt_ty(t)))
                .collect(),
        ),
        (
            "fields",
            ide.fields
                .iter()
                .map(|(s, f)| format!("{} {f:?}", span(s)))
                .collect(),
        ),
        (
            "member_uses",
            ide.member_uses
                .iter()
                .map(|(a, b)| format!("{} {}", span(a), span(b)))
                .collect(),
        ),
        ("refused", ide.refused.iter().map(span).collect()),
    ];
    let mut out = String::new();
    for (name, lines) in &mut sections {
        for line in lines.iter_mut() {
            *line = symbolic(c, &scopes, line);
        }
        lines.sort_unstable();
        lines.dedup();
        writeln!(out, "## {name} ({})", lines.len()).unwrap();
        for line in lines.iter() {
            writeln!(out, "{line}").unwrap();
        }
    }
    out
}

fn check(name: &str, entry: &str, root: &str) {
    let repo = repo();
    let actual = facts(&repo.join(entry), &repo.join(root));
    test_support::assert_snapshot!(name, actual);
}

/// Runs `body` on a thread with room for deep checking.
fn big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn type_variants() {
    big_stack(|| {
        check(
            "type-variants",
            "examples/type-variants/main.jai",
            "examples/type-variants/main.jai",
        );
    });
}

#[test]
fn polymorphs_and_static_branches() {
    big_stack(|| {
        for name in [
            "auto-bake-omitted-default",
            "baked-default-uses-poly",
            "dead-code-elimination",
            "macro-body-constants",
            "poly-infer-from-ifx",
            "backtick-defer-names",
        ] {
            let path = format!("tests/compiler/regressions/{name}.jai");
            check(name, &path, &path);
        }
    });
}
