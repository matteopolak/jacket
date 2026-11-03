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
