//! Foreign release protocols come from ABI symbols, never ordinary wrapper names.
use super::*;

#[test]
fn fallible_windows_release_symbols_export_only_possible_frees() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        lib :: #system_library "kernel32";
        local_release :: (p:*void)->*void #foreign lib "LocalFree";
        global_release :: (p:*void)->*void #foreign lib "GlobalFree";
        key_release :: (p:*void)->u32 #foreign lib "RegCloseKey";
        release_local :: (p:*void) {local_release(p);}
        release_global :: (p:*void) {global_release(p);}
        release_key :: (p:*void) {key_release(p);}
        LocalFree :: (p:*void) {}
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    for name in ["release_local", "release_global", "release_key"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let s = &after.procs[&id];
        assert!(
            s.may
                .keys()
                .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0)),
            "{name}: {s:?}"
        );
        assert!(!s.guaranteed.keys().any(|e| e.kind == Kind::Free));
    }
    let named = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "LocalFree" && p.lit.body.is_some())
            .unwrap() as u32,
    );
    assert!(!after.procs[&named].may.keys().any(|e| e.kind == Kind::Free));
}
