//! Original reduction: module-mode lint must discover Context additions before final layout.
use jailint::{
    config::Config,
    driver::{Options, run},
};
use std::path::PathBuf;

#[test]
fn module_context_additions_precede_program_finalization() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("generator-context-lint-root");
    let directory = root.join("Context_Module");
    std::fs::create_dir_all(&directory).unwrap();
    let entry = directory.join("module.jai");
    std::fs::write(&entry,
        "#add_context original_counter: int; read :: () -> int { return context.original_counter; }\n"
    ).unwrap();
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let options = Options {
        paths: vec![entry],
        import_dirs: vec![root],
        stdlib: workspace.join("stdlib"),
        config: Config::default(),
        jobs: 1,
        verbose: false,
        compiler_options: jaic::sema::Options::host(),
        source: None,
        add_sources: Vec::new(),
    };
    let result = run(&options);
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
    assert!(result.lints.is_empty(), "{:?}", result.lints);
}

#[test]
fn context_fix_preserves_lazy_configured_bootstrap_dependencies() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("generator-configured-lint-root");
    for directory in [
        "stdlib/prelude",
        "stdlib/Runtime_Support",
        "stdlib/internal/original",
    ] {
        std::fs::create_dir_all(root.join(directory)).unwrap();
    }
    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    let helper = root.join("stdlib/internal/original/configured.jai");
    std::fs::write(&helper, "#module_parameters(MODE:int); #assert MODE == 7; read :: () -> int #no_context { return MODE; }").unwrap();
    std::fs::write(root.join("stdlib/Runtime_Support/module.jai"),
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool); #scope_file H :: #import \"internal/original/configured\"(MODE=7); #program_export \"original_marker\" marker :: () -> int #c_call { return H.read(); }").unwrap();
    let options = Options {
        paths: vec![helper],
        import_dirs: Vec::new(),
        stdlib: root.join("stdlib"),
        config: Config::default(),
        jobs: 1,
        verbose: false,
        compiler_options: jaic::sema::Options::host(),
        source: None,
        add_sources: Vec::new(),
    };
    let result = run(&options);
    assert!(result.incomplete.is_empty(), "{:?}", result.incomplete);
}
