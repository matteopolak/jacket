//! Private-library lint coverage and application privacy regressions.
use jailint::{
    config::Config,
    driver::{Options, run},
};
use std::{path::PathBuf, process::Command};
use test_support::TempDir;

fn fixture(name: &str) -> TempDir {
    let root = TempDir::new(name);
    for dir in [
        "stdlib/prelude",
        "stdlib/Runtime_Support",
        "stdlib/internal/tools",
        "app",
    ] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"),"#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);").unwrap();
    root
}

fn options(root: &std::path::Path, path: PathBuf) -> Options {
    Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![path],
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        stdlib: root.join("stdlib"),
        config: Config::default(),
        jobs: 1,
        verbose: false,
    }
}

#[test]
fn nested_library_file_lints_in_library_scope_and_application_stays_private() {
    let root = fixture("private");
    let helper = root.join("stdlib/internal/tools/utility.jai");
    std::fs::write(&helper, "work :: () { lost := 1; }").unwrap();
    let result = run(&options(&root, helper));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    assert!(
        !result.lints.is_empty(),
        "the unused local must actually be checked"
    );
    let application = root.join("app/main.jai");
    std::fs::write(
        &application,
        "#import \"internal/tools/utility\"; main :: () {}",
    )
    .unwrap();
    let result = run(&options(&root, application));
    assert!(
        result
            .incomplete
            .iter()
            .any(|(_, error)| error.contains("private")),
        "{:?}",
        result.incomplete
    );
}

#[test]
fn required_bootstrap_parameters_reuse_the_configured_instance() {
    let root = fixture("configured");
    let result = run(&options(
        &root,
        root.join("stdlib/Runtime_Support/module.jai"),
    ));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn lazy_bootstrap_dependencies_keep_their_real_parameter_bindings() {
    let root = fixture("lazy-configured");
    let helper = root.join("stdlib/internal/tools/configured.jai");
    std::fs::write(
        &helper,
        "#module_parameters(MODE:int); #assert(MODE==7); read :: () -> int #no_context { return MODE; }",
    )
    .unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"),"#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #scope_file H :: #import \"internal/tools/configured\"(MODE=7); #program_export \"bootstrap_marker\" marker :: () -> int #c_call { return H.read(); }").unwrap();
    let result = run(&options(&root, helper));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn module_context_extensions_precede_bootstrap_body_layout() {
    let root = fixture("context-extension");
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #program_export \"marker\" marker :: () {}").unwrap();
    let module = root.join("stdlib/Probe/module.jai");
    std::fs::create_dir_all(module.parent().unwrap()).unwrap();
    std::fs::write(
        &module,
        "#add_context independent_probe:bool; set :: () { context.independent_probe=true; }",
    )
    .unwrap();
    let result = run(&options(&root, module));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn independent_context_producer_lints_with_integrated_bootstrap() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let module =
        root.join("tests/fixtures/profiler-context/library-roots/modules/Context_Probe/module.jai");
    let result = run(&options(&root, module));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    assert!(result.lints.is_empty(), "{:?}", result.lints);
}

#[test]
fn configured_context_extensions_precede_bootstrap_compile_time_execution() {
    let root = fixture("configured-context-extension");
    let helper = root.join("stdlib/internal/tools/context.jai");
    std::fs::write(&helper, "#module_parameters(MODE:int); #assert MODE==7; #add_context configured_probe:bool; set :: () { context.configured_probe=true; }").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #scope_file H :: #import \"internal/tools/context\"(MODE=7); #run { #assert size_of(#Context)==1; } #program_export \"marker\" marker :: () { H.set(); }").unwrap();
    let result = run(&options(&root, helper));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn module_import_precedes_bootstrap_context_query() {
    let root = fixture("context-query-order");
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #run { #assert size_of(#Context)==1; }").unwrap();
    let module = root.join("stdlib/Probe/module.jai");
    std::fs::create_dir_all(module.parent().unwrap()).unwrap();
    std::fs::write(
        &module,
        "#add_context independent_probe:bool; set :: () { context.independent_probe=true; }",
    )
    .unwrap();
    let result = run(&options(&root, module));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn supplemental_configured_import_is_loaded_before_root_fallback_and_execution() {
    let root = fixture("supplemental-context");
    let module = root.join("stdlib/Probe/module.jai");
    std::fs::create_dir_all(module.parent().unwrap()).unwrap();
    std::fs::write(&module, "#module_parameters(MODE:int); #assert MODE==7; #add_context supplemental_probe:bool; set :: () { context.supplemental_probe=true; }").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"), "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #run { #assert size_of(#Context)==1; }").unwrap();
    let mut opts = options(&root, module.clone());
    opts.add_sources.push("#import \"Probe\"(MODE=7);".into());
    let result = run(&opts);
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    std::fs::write(&module, "#module_parameters(MODE:int); #assert MODE==7; #add_context supplemental_probe:bool; broken :: () { missing_in_context_module(); }").unwrap();
    let result = run(&opts);
    assert!(
        result
            .incomplete
            .iter()
            .any(|(_, error)| error.contains("missing_in_context_module")),
        "{:?}",
        result.incomplete
    );
}

#[test]
fn incomplete_requested_body_is_an_unconditional_cli_failure() {
    let root = fixture("failure");
    let helper = root.join("stdlib/internal/tools/broken.jai");
    std::fs::write(&helper, "bad :: () { unavailable_call(); }").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .env("JAIC_STDLIB", root.join("stdlib"))
        .arg(helper)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("unavailable_call"));
}

#[test]
fn call_rules_follow_declaration_notes_through_generic_aliases() {
    let root = fixture("call-alias");
    std::fs::create_dir_all(root.join("stdlib/Basic")).unwrap();
    std::fs::write(root.join("stdlib/Basic/module.jai"),"min :: Impl.smallest; max :: Impl.largest; #scope_file Impl :: #import \"internal/tools/extrema\";").unwrap();
    std::fs::write(root.join("stdlib/internal/tools/extrema.jai"),"smallest :: (a:$T,b:T) -> T { if a<b return a; return b; } @ReturnsSmaller largest :: (a:$T,b:T) -> T { if a>b return a; return b; } @ReturnsLarger").unwrap();
    let application = root.join("app/main.jai");
    std::fs::write(&application,"#import \"Basic\"; bad :: (n:int) -> int { return min(0,max(10,n)); } main :: () {} local :: (n:int) -> int { min :: (a:int,b:int) -> int { return a+b; } max :: (a:int,b:int) -> int { return a+b; } return min(0,max(10,n)); }").unwrap();
    let result = run(&options(&root, application));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    assert_eq!(
        result
            .lints
            .iter()
            .filter(|lint| lint.rule == "min_max")
            .count(),
        1,
        "{:?}",
        result.lints
    );
}

#[test]
fn file_in_a_library_folder_that_sets_program_parameters_lints_as_a_program() {
    let root = fixture("program-parameters");
    std::fs::create_dir_all(root.join("stdlib/Probe/tests")).unwrap();
    std::fs::write(
        root.join("stdlib/Probe/module.jai"),
        "#module_parameters()(FLAG: bool = false); #if FLAG { enabled :: () {} }",
    )
    .unwrap();
    let test = root.join("stdlib/Probe/tests/flagged.jai");
    std::fs::write(
        &test,
        "P :: #import \"Probe\"()(FLAG = true); main :: () { P.enabled(); }",
    )
    .unwrap();
    let result = run(&options(&root, test));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn stdlib_scan_skips_expected_failures_and_keeps_real_findings_fatal() {
    let root = fixture("stdlib-scan");
    let tests = root.join("stdlib/Thing/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(root.join("stdlib/Thing/module.jai"), "").unwrap();
    for (name, note) in [
        ("negative", "compile_fail"),
        ("fault", "should_fail(\"boom\")"),
        ("known", "xfail(\"broken\")"),
    ] {
        std::fs::write(
            tests.join(format!("{name}.jai")),
            format!("// test: {note}\nmain :: () {{ unavailable_call(); }}"),
        )
        .unwrap();
    }
    // A platform-only module still produces an informative note.
    std::fs::write(
        root.join("stdlib/internal/tools/platform.jai"),
        "bad :: () { unavailable_platform(); }",
    )
    .unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_jailint"))
            .env("JAIC_STDLIB", root.join("stdlib"))
            .args(["-D", "warnings", "-j", "2"])
            .arg(root.join("stdlib"))
            .output()
            .unwrap()
    };
    let output = run();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(
        stderr.contains("1 program did not compile completely"),
        "{stderr}"
    );
    std::fs::write(
        root.join("stdlib/Thing/module.jai"),
        "work :: () { lost := 1; }",
    )
    .unwrap();
    let output = run();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("unused variable `lost`"));
}

#[test]
fn module_that_sets_program_parameters_is_incomplete_like_the_compiler() {
    let root = fixture("module-program-parameters");
    std::fs::create_dir_all(root.join("stdlib/Probe")).unwrap();
    std::fs::create_dir_all(root.join("stdlib/Inner")).unwrap();
    std::fs::write(
        root.join("stdlib/Inner/module.jai"),
        "#module_parameters()(FLAG: bool = false);",
    )
    .unwrap();
    let module = root.join("stdlib/Probe/module.jai");
    std::fs::write(&module, "#import \"Inner\"()(FLAG = true);").unwrap();
    let result = run(&options(&root, module));
    assert_eq!(result.incomplete.len(), 1, "{:?}", result.incomplete);
}

#[test]
fn directory_scan_skips_the_negative_program_parameter_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = root.join("stdlib/Basic/tests/config-negative");
    let result = run(&options(&root, fixtures));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}

#[test]
fn program_marked_for_another_system_is_compiled_for_it() {
    let root = fixture("marked-target");
    std::fs::write(
        root.join("stdlib/prelude/module.jai"),
        "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }",
    )
    .unwrap();
    let marked = root.join("app/wasm-only.jai");
    std::fs::write(
        &marked,
        "// test: only(.WASM)\n#assert OS == .WASM;\nmain :: () {}\n",
    )
    .unwrap();
    let result = run(&options(&root, marked));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    // A test file in the library is compiled as a module, for the system it names too.
    std::fs::create_dir_all(root.join("stdlib/Thing/tests")).unwrap();
    let library_test = root.join("stdlib/Thing/tests/wasm-only.jai");
    std::fs::copy(root.join("app/wasm-only.jai"), &library_test).unwrap();
    let result = run(&options(&root, library_test));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    let unmarked = root.join("app/unmarked.jai");
    std::fs::write(&unmarked, "#assert OS == .WASM;\nmain :: () {}\n").unwrap();
    let result = run(&options(&root, unmarked));
    assert_eq!(result.incomplete.len(), 1, "{:?}", result.incomplete);
}

#[test]
fn module_in_a_modules_folder_sees_its_sibling_modules() {
    let root = fixture("module-siblings");
    let modules = root.join("stdlib/Thing/tests/modules");
    std::fs::create_dir_all(modules.join("Facade")).unwrap();
    std::fs::create_dir_all(modules.join("Provider")).unwrap();
    std::fs::write(
        modules.join("Provider/module.jai"),
        "answer :: () -> int { return 1; }",
    )
    .unwrap();
    let facade = modules.join("Facade/module.jai");
    std::fs::write(&facade, "P :: #import \"Provider\";").unwrap();
    let result = run(&options(&root, facade));
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}
