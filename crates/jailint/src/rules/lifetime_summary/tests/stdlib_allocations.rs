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
    }
}
