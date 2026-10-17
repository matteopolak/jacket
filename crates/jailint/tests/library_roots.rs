//! Private-library lint coverage and application privacy regressions.
use jailint::{
    config::Config,
    driver::{Options, run},
};
use std::{path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("library-roots")
        .join(name);
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
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
fn library_call_rules_follow_exported_procedure_identity_through_generic_aliases() {
    let root = fixture("call-alias");
    std::fs::create_dir_all(root.join("stdlib/Basic")).unwrap();
    std::fs::write(root.join("stdlib/Basic/module.jai"),"min :: Impl.smallest; max :: Impl.largest; #scope_file Impl :: #import \"internal/tools/extrema\";").unwrap();
    std::fs::write(root.join("stdlib/internal/tools/extrema.jai"),"smallest :: (a:$T,b:T) -> T { if a<b return a; return b; } largest :: (a:$T,b:T) -> T { if a>b return a; return b; }").unwrap();
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
