//! Checked stdlib allocation choices must agree with their temporary-result contracts.
use super::*;

#[test]
fn postgres_generated_row_shapes_use_temporary_results() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let stdlib = root.join("stdlib");
    for windows in [false, true] {
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        if windows {
            options.os = jaic::target::TargetOs::Windows;
            options.cpu = jaic::target::TargetCpu::X64;
        }
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![
            IdeRoot::under(stdlib.join("Extensions/Postgres")),
            IdeRoot::Virtual,
        ]);
        facts.lint = true;
        c.ed.record(facts);
        c.begin_sources(&[ProgramSource::File(
            stdlib.join("Extensions/Postgres/tests/typed_scan.jai"),
        )])
        .unwrap();
        c.finish_program().unwrap();
        assert!(c.ide_check_all().is_empty());
        without_body_notes(&mut c);
        c.release_lowered_code();
        let summaries = Summaries::build(&c);
        for name in ["row_shape_text", "row_shape"] {
            let id = ProcId(
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap() as u32,
            );
            assert!(c.ed.facts().unwrap().lifetime_bodies.contains_key(&id));
            let summary = &summaries.procs[&id];
            assert!(
                summary
                    .returns
                    .get(&0)
                    .is_some_and(|(p, _)| p.root == Root::Temporary),
                "{name} windows={windows}: {summary:?}"
            );
        }
        let address = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "address_text")
                .unwrap() as u32,
        );
        assert!(c.ed.facts().unwrap().lifetime_bodies.contains_key(&address));
        assert!(
            summaries.procs[&address].may_returns.contains(&0),
            "address_text windows={windows}: {:?}",
            summaries.procs[&address]
        );
    }
}

#[test]
fn stdlib_swap_contract_matches_every_checked_instance_without_notes() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        main :: () {
            a: int; b: int; swap(*a, *b);
            x: string; y: string; swap(*x, *y);
            p: *void; q: *void; swap(*p, *q);
        }
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    let ids: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.as_str() == "swap" && !p.is_poly && p.sig.is_some())
            .map(|(i, _)| ProcId(i as u32))
            .collect();
    assert!(ids.len() >= 3);
    without_body_notes(&mut c);
    for id in ids {
        assert!(super::super::swap::swaps_arguments(&c, id));
    }
}

fn checked_without_contracts(source: &str) -> (Compiler, Summaries) {
    checked_without_contracts_target(source, false)
}

fn checked_without_contracts_target(source: &str, windows: bool) -> (Compiler, Summaries) {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    if windows {
        options.os = jaic::target::TargetOs::Windows;
        options.cpu = jaic::target::TargetCpu::X64;
    }
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    c.release_lowered_code();
    let summaries = Summaries::build(&c);
    (c, summaries)
}

#[test]
fn builtin_cleanup_contracts_release_backing_storage_in_every_checked_instance() {
    let (c, all) = checked_without_contracts(
        r#"
        #import "Basic";
        main :: () {
            text: string; free(text);
            view: [] int; array_free(view);
            array: [..] int; array_free(array);
        }
    "#,
    );
    let mut checked = 0;
    for (i, p) in c.pr.procs.iter().enumerate() {
        if !["free", "array_free"].contains(&p.name.as_str())
            || p.is_poly
            || p.body_state != jaic::sema::procs::BodyState::Done
        {
            continue;
        }
        let Some(first) = p.sig.as_ref().and_then(|s| s.params.first()) else {
            continue;
        };
        if !matches!(
            c.ty.types.kind(first.ty),
            TypeKind::String
                | TypeKind::Array {
                    kind: ArrayKind::View | ArrayKind::Resizable,
                    ..
                }
        ) {
            continue;
        }
        let Some(summary) = all.procs.get(&ProcId(i as u32)) else {
            continue;
        };
        checked += 1;
        assert!(
            summary.guaranteed.keys().any(|e| e.kind == Kind::Free
                && e.target.root == Root::Parameter(0)
                && e.guard.is_none()
                && descriptor_storage(&e.target.steps)),
            "{summary:?}"
        );
        assert!(!summary.may.keys().any(|e| e.kind == Kind::Free
            && e.target.root == Root::Parameter(0)
            && e.target.steps.is_empty()));
    }
    assert!(checked >= 3);
}

#[test]
fn range_formatter_temporary_allocation_contract_is_inferred_from_its_body() {
    let (c, all) = checked_without_contracts(
        r#"
        F :: #import "Extensions/Jai_Format";
        main :: () { F.format_range("main::(){}", 0, 10); }
    "#,
    );
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "format_range_text")
            .unwrap() as u32,
    );
    let summary = &all.procs[&id];
    assert!(
        summary
            .allocation_effects
            .iter()
            .any(|(root, _)| *root == Root::Temporary),
        "{summary:?}"
    );
    assert!(
        !summary
            .returns
            .values()
            .any(|(path, _)| path.root == Root::Temporary)
    );
}

#[test]
fn debug_backtrace_cleanup_contract_follows_the_native_release_abi() {
    for windows in [false, true] {
        let (c, all) = checked_without_contracts_target(
            r#"
            #import "Debug";
            developer_debug_free_symbols :: (symbols: *void) {}
            main :: () { symbols: [] string; free_backtrace(symbols); }
        "#,
            windows,
        );
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "free_backtrace")
                .unwrap() as u32,
        );
        let summary = &all.procs[&id];
        assert!(
            summary.guaranteed.keys().any(|e| e.kind == Kind::Free
                && e.target.root == Root::Parameter(0)
                && e.guard.is_none()
                && descriptor_storage(&e.target.steps)),
            "{summary:?}"
        );
        let fake = ProcId(
            c.pr.procs
                .iter()
                .position(|p| {
                    p.name.as_str() == "developer_debug_free_symbols" && p.lit.body.is_some()
                })
                .unwrap() as u32,
        );
        assert!(!all.procs[&fake].may.keys().any(|e| e.kind == Kind::Free));
    }
}
