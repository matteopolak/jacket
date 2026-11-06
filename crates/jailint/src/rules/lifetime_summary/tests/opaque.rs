//! Opaque machine operations cannot establish protocol or storage guarantees.
use super::*;

#[test]
fn opaque_memory_barriers_remove_lock_and_storage_certainty() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    // Only check the target body. No native instructions or foreign calls execute.
    options.cpu = jaic::target::TargetCpu::X64;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        lib :: #library,system "kernel32";
        enter_native :: (p: *void) #foreign lib "EnterCriticalSection";
        Holder :: struct { storage: *void; }
        probe_opaque_native :: (p: *void) { enter_native(p); #asm { mfence; } }
        probe_opaque_storage :: (p: *Holder, replacement: *void) {
            p.storage = replacement;
            #asm { mfence; }
            free(p.storage);
        }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let all = Summaries::build(&c);
    for name in ["probe_opaque_native", "probe_opaque_storage"] {
        let index =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let id = ProcId(index as u32);
        assert!(
            c.ed.facts().unwrap().lifetime_bodies[&id]
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .any(|e| matches!(e, Event::Barrier))
        );
        let summary = &all.procs[&id];
        assert!(summary.locks.is_none(), "{name}: {summary:?}");
        if name == "probe_opaque_storage" {
            assert!(
                !summary
                    .guaranteed
                    .keys()
                    .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(1)),
                "{summary:?}"
            );
            assert!(
                summary
                    .may
                    .keys()
                    .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(1)),
                "{summary:?}"
            );
        }
    }
}
