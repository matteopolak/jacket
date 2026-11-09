//! Foreign release protocols come from the notes on `#foreign` declarations, never from symbol or wrapper names.
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
        local_release :: (p:*void)->*void #foreign lib "LocalFree"; @MaybeFreesArgument
        global_release :: (p:*void)->*void #foreign lib "GlobalFree"; @MaybeFreesArgument
        key_release :: (p:*void)->u32 #foreign lib "RegCloseKey"; @MaybeFreesArgument
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

#[test]
fn foreign_declarations_release_only_through_their_notes() {
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
        lib :: #system_library "c";
        bare_free :: (p: *void) #foreign lib "free";
        bare_close :: (fd: s32) -> s32 #foreign lib "close";
        bare_exit :: (code: s32) #foreign lib "exit";
        sure_free :: (p: *void) #foreign lib "free"; @FreesArgument
        maybe_close :: (fd: s32) -> s32 #foreign lib "close"; @MaybeFreesArgument
        second_free :: (a: s32, p: *void) #foreign lib "free"; @MaybeFreesArgument(1)
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    let summaries = Summaries::build(&c);
    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        &summaries.procs[&ProcId(id as u32)]
    };
    let frees = |s: &Summary, formal: usize| {
        s.may
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(formal))
    };
    for bare in ["bare_free", "bare_close"] {
        assert!(!frees(get(bare), 0), "{bare} has no note");
    }
    assert!(!get("bare_exit").observable.exit);
    assert!(frees(get("sure_free"), 0));
    assert!(
        get("sure_free")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free)
    );
    assert!(frees(get("maybe_close"), 0));
    assert!(
        !get("maybe_close")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free)
    );
    assert!(frees(get("second_free"), 1));
    assert!(!frees(get("second_free"), 0));
}

#[test]
fn narrowing_a_handle_keeps_it_the_released_argument() {
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
        lib :: #system_library "c";
        raw_close :: (fd: s32) -> s32 #foreign lib "close"; @FreesArgument
        close_wide :: (fd: s64) -> s64 { return cast(s64) raw_close(cast(s32) fd); }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    let summaries = Summaries::build(&c);
    let id =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "close_wide")
            .unwrap();
    assert!(
        summaries.procs[&ProcId(id as u32)]
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0))
    );
}
