//! The editor releases executable IR before asking for lifetime diagnostics.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::path::Path;
use std::rc::Rc;

fn checked(path: &Path) -> (Compiler, jaic::source::FileId) {
    checked_in(path, None)
}

fn checked_in(path: &Path, extra: Option<&Path>) -> (Compiler, jaic::source::FileId) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let stdlib = root.join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    if let Some(extra) = extra {
        options.import_paths.insert(0, extra.into());
    }
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::under(root)]).with_files(vec![path.into()]);
    facts.lint = true;
    compiler.ed.record(facts);
    compiler
        .compile_sources(&[ProgramSource::File(path.into())])
        .unwrap();
    assert!(compiler.ide_check_all().is_empty());
    let file = compiler
        .src
        .files
        .iter()
        .find(|f| compiler.src.sources.get(f.id).path.as_file() == Some(path))
        .unwrap()
        .id;
    (compiler, file)
}

fn returned_locals(compiler: &mut Compiler, file: jaic::source::FileId) -> Vec<crate::Lint> {
    let mut config = crate::config::Config::default();
    for rule in crate::RULES {
        config
            .levels
            .insert(rule.name.into(), crate::config::Level::Allow);
    }
    config
        .levels
        .insert("returned_local_address".into(), crate::config::Level::Warn);
    crate::lint_files(
        compiler,
        &[file],
        &crate::config::Configs::new(&config),
        true,
    )
}

#[test]
fn frozen_execution_actual_diagnostics_survive_release() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/regressions/run-result-frozen-values.jai")
        .canonicalize()
        .unwrap();
    let (mut compiler, file) = checked(&path);
    assert!(returned_locals(&mut compiler, file).is_empty());
    compiler.release_lowered_code_with_lifetime_execution(true);
    let after = returned_locals(&mut compiler, file);
    assert!(after.is_empty(), "post-release diagnostics: {after:?}");
}

#[test]
fn frozen_execution_runtime_negative_diagnostics_survive_release() {
    for name in [
        "mixed-run-retention",
        "frozen-callback-retention",
        "callable-entrypoint-retention",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!(
                "../../tests/lint/pointer_lifetime/strict/{name}.jai"
            ))
            .canonicalize()
            .unwrap();
        let (mut compiler, file) = checked(&path);
        assert_eq!(
            returned_locals(&mut compiler, file).len(),
            1,
            "{name} before release"
        );
        compiler.release_lowered_code_with_lifetime_execution(true);
        assert_eq!(
            returned_locals(&mut compiler, file).len(),
            1,
            "{name} after release"
        );
    }
}

#[test]
fn frozen_execution_selected_runtime_handler_is_not_compile_time_only() {
    let folder = test_support::TempDir::new("frozen-execution-runtime-handler");
    std::fs::create_dir(folder.join("Runtime_Support")).unwrap();
    std::fs::write(folder.join("Runtime_Support/module.jai"), r#"
        #module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);
        Context_Base :: struct {}
        helper :: () -> int #no_context { return 9; }
        run_for_test :: () -> int #no_context { __jaic_check_failed(0,0,0,false,0,null); return helper(); }
        #scope_file
        __jaic_check_failed :: (reason:s64, a:s64, b:s64, fatal:bool, line:s64, filename:*u8) #no_context { helper(); }
    "#).unwrap();
    let path = folder.join("main.jai");
    std::fs::write(
        &path,
        r#"
        R :: #import "Runtime_Support";
        VALUE :: #run R.run_for_test();
        read :: (i:int) -> int #no_context { values:[1]int; return values[i]; }
        main :: () #no_context { value:=read(0); }
    "#,
    )
    .unwrap();
    let (mut c, _) = checked_in(&path, Some(&folder));
    let selected = c
        .out
        .program
        .check_failed
        .expect("the real checked array access selected its ABI-checked handler");
    let helper = jaic::sema::ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "helper")
            .unwrap() as u32,
    );
    assert!(c.out.program.funcs.iter().flatten().flat_map(|f|&f.blocks).flat_map(|b|&b.insts).any(|i|matches!(i,jaic::ir::Inst::Intrinsic(call) if call.op==jaic::ir::Intrinsic::BoundsCheck)),"actual runtime check IR is present");
    let before = super::Summaries::build(&c);
    assert!(
        !before.compile_time_only.contains(&helper),
        "selected handler {selected:?} reaches helper at runtime"
    );
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(
        !super::Summaries::build(&c)
            .compile_time_only
            .contains(&helper)
    );
}

#[test]
fn frozen_execution_export_and_global_relocation_runtime_paths_survive_release() {
    let cases = [
        r#"#import "Basic"; #program_export "runtime_build" build :: () -> *int #c_call { local:=42; return *local; } FROZEN :: #run build(); main :: () {}"#,
        r#"#import "Basic"; build :: () -> *int { local:=42; return *local; } select :: () -> (() -> *int) {return build;} INITIAL :: #run select(); Holder :: struct { callback: () -> *int; } holder: Holder = .{INITIAL}; link: *Holder = *holder; main :: () {print("%",link.callback().*);}"#,
    ];
    for (i, source) in cases.iter().enumerate() {
        let folder = test_support::TempDir::new("frozen-execution-runtime-path");
        let path = folder.join("main.jai");
        std::fs::write(&path, source).unwrap();
        let (mut c, file) = checked(&path);
        if i == 1 {
            assert!(
                c.out
                    .program
                    .globals
                    .iter()
                    .flat_map(|g| &g.relocs)
                    .any(|r| matches!(r.target, jaic::ir::RelocTarget::Func(_)))
            );
            assert!(
                c.out
                    .program
                    .globals
                    .iter()
                    .flat_map(|g| &g.relocs)
                    .any(|r| matches!(r.target, jaic::ir::RelocTarget::Global(_)))
            );
        }
        assert_eq!(
            returned_locals(&mut c, file).len(),
            1,
            "runtime path {i} before release"
        );
        c.release_lowered_code_with_lifetime_execution(true);
        assert_eq!(
            returned_locals(&mut c, file).len(),
            1,
            "runtime path {i} after release"
        );
    }
}

fn frozen_program() -> (Compiler, jaic::source::FileId) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/compiler/regressions/run-result-frozen-values.jai")
        .canonicalize()
        .unwrap();
    checked(&path)
}

#[test]
fn frozen_execution_repeated_release_clear_and_incomplete_are_fail_closed() {
    let (mut c, _) = frozen_program();
    c.release_lowered_code_with_lifetime_execution(true);
    let usage = c.lifetime_execution_usage();
    eprintln!(
        "frozen-result regression: {} owned bytes; {} charged visits",
        usage.0, usage.1
    );
    assert!(usage.0 > 0 && usage.0 <= 8 * 1024 * 1024 && usage.1 > 0);
    c.release_lowered_code_with_lifetime_execution(true);
    assert_eq!(c.lifetime_execution_usage(), usage);
    c.ed.facts_mut().unwrap().clear_lifetimes();
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
    let (mut c, _) = frozen_program();
    c.release_lowered_code();
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
    let (mut c, _) = frozen_program();
    c.release_lowered_code_with_lifetime_execution(true);
    c.release_lowered_code_with_lifetime_execution(false);
    assert!(c.lifetime_execution().is_none());
    let mut replacement = IdeFacts::new(vec![IdeRoot::Virtual]);
    replacement.lint = true;
    c.ed.record(replacement);
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());

    assert_eq!(c.lifetime_execution_usage(), (0, 0));
}

#[test]
fn frozen_execution_same_id_ir_cannot_be_hidden_by_a_second_release() {
    let (mut c, _) = frozen_program();
    let (id, blocks) = c
        .out
        .program
        .funcs
        .iter()
        .enumerate()
        .find_map(|(i, f)| {
            f.as_ref()
                .filter(|f| !f.blocks.is_empty())
                .map(|f| (i, f.blocks.clone()))
        })
        .unwrap();
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution_usage().0 > 0);
    c.out.program.funcs[id].as_mut().unwrap().blocks = blocks;
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
    let (mut c, _) = frozen_program();
    let (id, relocs) = c
        .out
        .program
        .globals
        .iter()
        .enumerate()
        .find(|(_, g)| !g.relocs.is_empty())
        .map(|(i, g)| (i, g.relocs.clone()))
        .unwrap();
    c.release_lowered_code_with_lifetime_execution(true);
    c.out.program.globals[id].relocs = relocs;
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
    c.release_lowered_code_with_lifetime_execution(true);
    assert!(c.lifetime_execution().is_none());
}

#[test]
fn frozen_execution_changed_target_entry_and_selected_handler_drop_storage() {
    let mutations: [fn(&mut Compiler); 4] = [
        |c| {
            c.options.cpu = if c.options.cpu == jaic::target::TargetCpu::X64 {
                jaic::target::TargetCpu::Arm64
            } else {
                jaic::target::TargetCpu::X64
            }
        },
        |c| c.options.entry_point.push_str("_changed"),
        |c| c.out.program.check_failed = None,
        |c| c.out.program.globals[0].export = Some("changed_export".into()),
    ];
    for mutate in mutations {
        let (mut c, _) = frozen_program();
        c.release_lowered_code_with_lifetime_execution(true);
        assert!(c.lifetime_execution_usage().0 > 0);
        mutate(&mut c);
        assert!(c.lifetime_execution().is_none());
        assert_eq!(c.lifetime_execution_usage(), (0, 0));
        c.release_lowered_code_with_lifetime_execution(true);
        assert!(c.lifetime_execution().is_none());
    }
}

#[test]
fn frozen_execution_changed_procedure_name_rejects_the_certificate() {
    let (mut c, _) = frozen_program();
    let main =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == c.options.entry_point)
            .unwrap();
    c.release_lowered_code_with_lifetime_execution(true);
    c.pr.procs[main].name = jaic::intern::Sym::intern("renamed_configured_entry");
    assert!(
        c.lifetime_execution().is_none(),
        "configured entry membership changed without changing its AST or function target"
    );
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
}

#[test]
fn frozen_execution_invalid_selected_entry_cannot_panic_or_authorize_capture() {
    let (mut c, _) = frozen_program();
    c.pr.entry_point = Some(jaic::sema::ProcId(u32::MAX));
    let live =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| super::Summaries::build(&c)));
    c.release_lowered_code_with_lifetime_execution(true);
    let retained = c.lifetime_execution().is_some();
    assert!(
        live.is_ok() && !retained,
        "malformed selected entry: live panic={}, retained certificate={retained}",
        live.is_err()
    );
    assert_eq!(c.lifetime_execution_usage(), (0, 0));
}
