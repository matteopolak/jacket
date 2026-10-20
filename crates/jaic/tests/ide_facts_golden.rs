//! The editor facts a few programs record, compared with saved dumps. Every fact list is
//! sorted, so only what is recorded counts, not the order it was recorded in.
//!
//! After an intended change to what is recorded, rewrite the dumps with
//! `JAIC_BLESS=1 cargo test -p jaic --test ide_facts_golden`.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::source::Span;
use jaic::types::TypeId;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// The facts recorded for `entry` and the files under `root`, as an editor analysis records
/// them: compile, check the rest, sketch the polymorphs nothing called.
fn facts(entry: &Path, root: &Path) -> String {
    let stdlib = repo().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
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

fn dump(c: &Compiler, errors: Vec<String>) -> String {
    let ide = c.ed.facts().unwrap();
    let span = |s: &Span| format!("{}:{}..{}", s.file.0, s.start, s.end);
    let ty = |t: &TypeId| c.types.name(*t);
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
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/ide_facts")
        .join(format!("{name}.txt"));
    if std::env::var_os("JAIC_BLESS").is_some() {
        std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
        std::fs::write(&golden, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&golden)
        .unwrap_or_else(|e| panic!("{}: {e} (bless with JAIC_BLESS=1)", golden.display()));
    if actual != expected {
        let first = actual
            .lines()
            .zip(expected.lines())
            .position(|(a, e)| a != e)
            .unwrap_or(actual.lines().count().min(expected.lines().count()));
        panic!(
            "{name}: facts differ from {} at line {}:\n  actual:   {:?}\n  expected: {:?}",
            golden.display(),
            first + 1,
            actual.lines().nth(first),
            expected.lines().nth(first),
        );
    }
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
fn tour() {
    big_stack(|| check("tour", "examples/tour/main.jai", "examples/tour"));
}

#[test]
fn type_variants() {
    big_stack(|| {
        check(
            "type-variants",
            "examples/type-variants.jai",
            "examples/type-variants.jai",
        )
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
