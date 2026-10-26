//! Each file uses the nearest `jacket.toml` above it, whichever roots are listed.
use jailint::{
    config::Config,
    driver::{Options, run},
};
use std::path::{Path, PathBuf};

const PROGRAM: &str = "main :: () { total := 12345 + 6789; }\n";

fn tree(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    for dir in ["stdlib/prelude", "stdlib/Runtime_Support", "top", "nested"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(
        root.join("stdlib/Runtime_Support/module.jai"),
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);",
    )
    .unwrap();
    std::fs::write(
        root.join("jacket.toml"),
        "[lint.rules]\nmagic_number = \"warn\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("nested/jacket.toml"),
        "[lint.rules]\nmagic_number = \"allow\"\n",
    )
    .unwrap();
    std::fs::write(root.join("top/main.jai"), PROGRAM).unwrap();
    std::fs::write(root.join("nested/main.jai"), PROGRAM).unwrap();
    root.canonicalize().unwrap()
}

/// The files with a `magic_number` finding when linting `paths` from the first one's config.
fn flagged(root: &Path, paths: &[&str]) -> Vec<String> {
    let paths: Vec<PathBuf> = paths.iter().map(|p| root.join(p)).collect();
    let mut config = Config::find_on_disk(&paths[0]).unwrap().unwrap();
    config.discover = true;
    let outcome = run(&Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths,
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        stdlib: root.join("stdlib"),
        config,
        jobs: 1,
        verbose: false,
    });
    assert!(outcome.incomplete.is_empty(), "{:?}", outcome.incomplete);
    assert!(outcome.config_errors.is_empty());
    let mut files: Vec<String> = outcome
        .lints
        .iter()
        .filter(|l| l.rule == "magic_number")
        .map(|l| {
            Path::new(&l.path)
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    files.sort();
    files.dedup();
    files
}

#[test]
fn a_nested_config_applies_when_its_root_is_listed_after_another() {
    let root = tree("nested-config-two-roots");
    assert_eq!(flagged(&root, &["top", "nested"]), ["top/main.jai"]);
    assert_eq!(flagged(&root, &["nested", "top"]), ["top/main.jai"]);
}

#[test]
fn a_nested_config_applies_when_it_is_under_a_listed_root() {
    let root = tree("nested-config-one-root");
    assert_eq!(flagged(&root, &["."]), ["top/main.jai"]);
}

#[test]
fn a_nested_config_applies_when_listed_alone() {
    let root = tree("nested-config-alone");
    assert!(flagged(&root, &["nested"]).is_empty());
}
