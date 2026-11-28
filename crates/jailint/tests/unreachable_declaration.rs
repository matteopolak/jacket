//! `unreachable_declaration` across files and roots: what counts as a use.
use jailint::config::{Config, Level};
use jailint::driver::{Options, run};
use std::path::{Path, PathBuf};
use test_support::TempDir;

const RULE: &str = "unreachable_declaration";

fn project(name: &str, files: &[(&str, &str)]) -> TempDir {
    let root = TempDir::new(name);
    for dir in ["stdlib/prelude", "stdlib/Runtime_Support", "app"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("stdlib/prelude/module.jai"), "").unwrap();
    std::fs::write(
        root.join("stdlib/Runtime_Support/module.jai"),
        "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool, DEFINE_INITIALIZATION:bool, ENABLE_BACKTRACE_ON_CRASH:bool);",
    )
    .unwrap();
    for (path, text) in files {
        let path = root.join("app").join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    root
}

/// The names the rule reports in the project's `app` folder, sorted.
fn dead(root: &Path) -> Vec<String> {
    let mut config = Config::default();
    config.levels.insert(RULE.into(), Level::Warn);
    let options = Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![root.join("app")],
        import_dirs: Vec::new(),
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
        overlay: std::collections::BTreeMap::new(),
        stdlib: root.join("stdlib"),
        config,
        jobs: 1,
        verbose: false,
    };
    let outcome = run(&options);
    assert!(outcome.incomplete.is_empty(), "{:?}", outcome.incomplete);
    let mut names: Vec<String> = outcome
        .lints
        .iter()
        .filter(|l| l.rule == RULE)
        .map(|l| {
            let text = &outcome.texts[&l.path];
            let file = PathBuf::from(&l.path);
            format!(
                "{}:{}",
                file.file_name().unwrap().to_string_lossy(),
                &text[l.start..l.end]
            )
        })
        .collect();
    names.sort();
    names
}

#[test]
fn an_unused_procedure_is_reported_and_a_called_one_is_not() {
    let root = project(
        "plain",
        &[(
            "main.jai",
            "used :: () {}\nunused :: () {}\nmain :: () { used(); }\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:unused"]);
}

#[test]
fn what_only_dead_code_uses_is_dead_too() {
    let root = project(
        "chain",
        &[(
            "main.jai",
            "leaf :: () {}\nmiddle :: () { leaf(); }\ntop :: () { middle(); }\nmain :: () {}\n",
        )],
    );
    assert_eq!(
        dead(&root),
        ["main.jai:leaf", "main.jai:middle", "main.jai:top"]
    );
}

#[test]
fn recursion_does_not_keep_a_procedure_alive() {
    let root = project(
        "recursion",
        &[(
            "main.jai",
            "ping :: (n: int) { if n > 0 pong(n - 1); }\npong :: (n: int) { if n > 0 ping(n - 1); }\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:ping", "main.jai:pong"]);
}

#[test]
fn run_directives_are_roots() {
    let root = project(
        "run",
        &[(
            "main.jai",
            "helper :: () -> int { return 1; }\nonly_at_compile_time :: () -> int { return helper(); }\nTABLE :: #run only_at_compile_time();\nstray :: () {}\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:stray"]);
}

#[test]
fn a_bare_run_statement_keeps_what_it_calls() {
    let root = project(
        "bare-run",
        &[(
            "main.jai",
            "setup :: () {}\nunused :: () {}\n#run setup();\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:unused"]);
}

#[test]
fn an_overload_nothing_selects_is_dead() {
    let root = project(
        "overloads",
        &[(
            "main.jai",
            "show :: (n: int) {}\nshow :: (f: float) {}\nmain :: () { show(1); }\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:show"]);
}

#[test]
fn a_struct_reached_through_using_is_used() {
    let root = project(
        "using",
        &[(
            "main.jai",
            "Base :: struct { id: int; }\nDerived :: struct { using base: Base; }\nOrphan :: struct { id: int; }\nmain :: () { d: Derived; d.id = 1; }\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:Orphan"]);
}

#[test]
fn types_constants_enums_and_variables_are_judged_too() {
    let root = project(
        "kinds",
        &[(
            "main.jai",
            "Mode :: enum { A; B; }\nLIMIT :: 4;\ncounter: int;\nUnused_Mode :: enum { A; }\nUNUSED_LIMIT :: 9;\nunused_counter: int;\nmain :: () { m := Mode.A; x := LIMIT + counter; }\n",
        )],
    );
    assert_eq!(
        dead(&root),
        [
            "main.jai:UNUSED_LIMIT",
            "main.jai:Unused_Mode",
            "main.jai:unused_counter"
        ]
    );
}

#[test]
fn a_constant_read_only_by_an_if_condition_is_used() {
    let root = project(
        "if-condition",
        &[(
            "main.jai",
            "ENABLED :: true;\nunused :: () {}\n#if ENABLED { visible :: () {} }\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:unused"]);
}

#[test]
fn a_procedure_the_program_exports_or_the_compiler_calls_is_a_root() {
    let root = project(
        "exported",
        &[(
            "main.jai",
            "exported :: () #program_export {}\nnoted :: () {} @Keep\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), Vec::<String>::new());
}

#[test]
fn loaded_files_are_followed_and_judged() {
    let root = project(
        "loaded",
        &[
            ("main.jai", "#load \"util.jai\";\nmain :: () { used(); }\n"),
            ("util.jai", "used :: () {}\nunused :: () {}\n"),
        ],
    );
    assert_eq!(dead(&root), ["util.jai:unused"]);
}

#[test]
fn a_shared_file_is_judged_by_every_program_that_loads_it() {
    let root = project(
        "shared",
        &[
            ("a.jai", "#load \"shared.jai\";\nmain :: () { first(); }\n"),
            ("b.jai", "#load \"shared.jai\";\nmain :: () { second(); }\n"),
            (
                "shared.jai",
                "first :: () {}\nsecond :: () {}\nneither :: () {}\n",
            ),
        ],
    );
    assert_eq!(dead(&root), ["shared.jai:neither"]);
}

#[test]
fn imports_of_a_local_module_count_as_uses() {
    let root = project(
        "import",
        &[
            (
                "main.jai",
                "#import \"tools\";\nmain :: () { exported(); }\n",
            ),
            (
                "modules/tools/module.jai",
                "exported :: () { inner(); }\ninner :: () {}\n",
            ),
        ],
    );
    assert_eq!(dead(&root), Vec::<String>::new());
}

#[test]
fn a_name_in_code_the_compiler_skipped_is_used() {
    let root = project(
        "skipped",
        &[(
            "main.jai",
            "ONLY_ELSEWHERE :: 3;\nunused :: () {}\nmain :: () {\n    #if false { x := ONLY_ELSEWHERE; }\n}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:unused"]);
}

#[test]
fn a_name_in_the_note_of_live_code_is_used() {
    let root = project(
        "note-argument",
        &[(
            "main.jai",
            "CASES :: int.[1, 2];\nunused :: () {}\ncheck :: (n: int) {} @test @cases(CASES)\nmain :: () {}\n",
        )],
    );
    assert_eq!(dead(&root), ["main.jai:unused"]);
}

#[test]
fn an_alias_of_a_module_is_used_through_its_members() {
    let root = project(
        "alias",
        &[
            (
                "main.jai",
                "T :: #import \"tools\";\nAlias :: T;\nUnused :: T;\nmain :: () { Alias.exported(); }\n",
            ),
            ("modules/tools/module.jai", "exported :: () {}\n"),
        ],
    );
    assert_eq!(dead(&root), ["main.jai:Unused"]);
}
