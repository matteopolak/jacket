//! Original private-library lint coverage and application privacy regressions.
use jailint::{
    config::Config,
    driver::{Options, run},
};
use std::{path::PathBuf, process::Command};

fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cleanroom-lint-library")
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
