//! A compile error is reported wherever it sits in an `ifx`, whichever branch the condition
//! picks: `jacket migrate` plans from these errors.

use jailint::{
    config::Config,
    driver::{Options, run},
};
use test_support::TempDir;

/// The unknown names the driver reports as errors for `source`.
fn unknown_names(name: &str, source: &str) -> Vec<String> {
    let dir = TempDir::new(name);

    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.jai");

    std::fs::write(&file, source).unwrap();
    let file = std::fs::canonicalize(file).unwrap();
    let found = run(&Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![file],
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: std::collections::BTreeMap::new(),
        stdlib: test_support::repo::stdlib(),
        config: Config::default(),
        jobs: 1,
        verbose: false,
    });
    let mut names: Vec<String> = found
        .errors
        .iter()
        .filter(|e| e.code == "unknown-identifier")
        .map(|e| e.message.clone())
        .collect();

    names.sort();
    names
}

#[test]
fn a_plain_unknown_name_is_reported() {
    let names = unknown_names(
        "branch-errors-control",
        "main :: () {\n    a := plain_missing;\n}\n",
    );

    assert_eq!(names, ["unknown identifier `plain_missing`"]);
}

#[test]
fn an_error_in_either_ifx_branch_is_reported() {
    let names = unknown_names(
        "branch-errors-ifx",
        "main :: () {\n    n := 3;\n    a := ifx n > 1 then 1 else missing_else;\n    b := ifx n > 1 then missing_then else 2;\n}\n",
    );

    assert_eq!(
        names,
        [
            "unknown identifier `missing_else`",
            "unknown identifier `missing_then`"
        ]
    );
}

#[test]
fn an_error_in_an_ifx_else_branch_is_reported_when_the_value_is_unused() {
    let names = unknown_names(
        "branch-errors-ifx-unused",
        "f :: (n: int) -> int {\n    return ifx n > 1 then n else missing_else;\n}\nmain :: () {}\n",
    );

    assert_eq!(names, ["unknown identifier `missing_else`"]);
}
