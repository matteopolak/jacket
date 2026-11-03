//! Descriptor reconstruction must preserve the storage selected by its data pointer.
use super::*;

#[test]
fn returned_descriptors_join_storage_origins_across_return_slots() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let stdlib = root.join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = std::fs::read_to_string(
        root.join("tests/lint/pointer_lifetime/strict/reconstructed-descriptor.jai"),
    )
    .unwrap();
    c.begin_sources(&[ProgramSource::String(source)]).unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "modified_view")
            .unwrap() as u32,
    );
    assert!(
        after.procs[&id]
            .returns
            .get(&0)
            .is_some_and(|(p, _)| p.root == Root::Temporary),
        "{:?}",
        after.procs[&id]
    );

    let mixed = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "mixed_view")
            .unwrap() as u32,
    );
    assert!(
        !after.procs[&mixed].returns.contains_key(&0),
        "{:?}",
        after.procs[&mixed]
    );
    assert!(
        after.procs[&mixed].may_returns.contains(&0),
        "{:?}",
        after.procs[&mixed]
    );
    let forwarded = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "forwarded_mixed_view")
            .unwrap() as u32,
    );
    assert!(!after.procs[&forwarded].returns.contains_key(&0));
    assert!(
        after.procs[&forwarded].may_returns.contains(&0),
        "{:?}",
        after.procs[&forwarded]
    );
    let durable = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "durable_view")
            .unwrap() as u32,
    );
    assert!(
        after.procs[&durable]
            .returns
            .get(&0)
            .is_some_and(|(p, _)| p.root == Root::CallerAllocator),
        "{:?}",
        after.procs[&durable]
    );
    assert!(!after.procs[&durable].may_returns.contains(&0));
}

#[test]
fn nested_record_fields_keep_distinct_origins_after_copies_and_ir_release() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let stdlib = root.join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = std::fs::read_to_string(
        root.join("tests/lint/pointer_lifetime/strict/nested-temporary-record.jai"),
    )
    .unwrap();
    c.begin_sources(&[ProgramSource::String(source)]).unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let file = vec![
        Step::Field(jaic::intern::Sym::intern("failure")),
        Step::Field(jaic::intern::Sym::intern("file")),
    ];
    let durable = vec![
        Step::Field(jaic::intern::Sym::intern("failure")),
        Step::Field(jaic::intern::Sym::intern("durable")),
    ];
    for (name, temporary) in [
        ("assemble", true),
        ("durable_nested", false),
        ("replaced_nested", false),
    ] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let s = &after.procs[&id];
        assert_eq!(
            s.return_fields
                .get(&(0, file.clone()))
                .is_some_and(|p| p.root == Root::Temporary),
            temporary,
            "{name}: {s:?}"
        );
        assert!(
            !s.may_return_fields.contains(&(0, durable.clone())),
            "{name}: {s:?}"
        );
    }
}
