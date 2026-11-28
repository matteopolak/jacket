//! jailint from the command line: which files it compiles as what, and mistakes on its command
//! line and in `jacket.toml`, each of which says what is wrong and how to fix it, and exits
//! with status 2.
use std::path::Path;
use std::process::Command;
use test_support::TempDir;

fn scratch(name: &str) -> TempDir {
    let dir = TempDir::new(name);
    std::fs::write(dir.join("game.jai"), "main :: () {}\n").unwrap();
    dir
}

/// jailint's stderr and exit status for `args`, run in `dir`.
fn jailint(dir: &Path, args: &[&str]) -> (String, Option<i32>) {
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(args)
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .env_remove("FORCE_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code(),
    )
}

#[test]
fn command_line_mistakes_explain_themselves() {
    let dir = scratch("args");
    let cases: &[(&[&str], &str)] = &[
        (
            &["--fx"],
            "error[cli-failure]: unknown option `--fx`\nhelp: did you mean `--fix`?",
        ),
        (
            &["-D", "float_equalty"],
            "error[cli-failure]: unknown rule `float_equalty`\nhelp: did you mean `float_equality`?",
        ),
        (
            &["--color", "sometimes"],
            "error[cli-failure]: unknown `--color` value `sometimes`\nhelp: use auto, always or never",
        ),
        (
            &["-j", "x"],
            "error[cli-failure]: `-j` takes a number of jobs, such as `-j 4`",
        ),
        (&["--config"], "error[cli-failure]: `--config` needs a file"),
        (
            &["gmae.jai"],
            "error[cli-failure]: `gmae.jai` does not exist\nhelp: did you mean `game.jai`?",
        ),
        (
            &["nothing_like_it.jai"],
            "error[cli-failure]: `nothing_like_it.jai` does not exist\nhelp: pass .jai files",
        ),
    ];
    for (args, expected) in cases {
        let (text, code) = jailint(&dir, args);
        assert_eq!(code, Some(2), "{args:?}: {text}");
        assert!(text.contains(expected), "{args:?}: {text}");
    }
}

#[test]
fn config_mistakes_name_the_line_and_the_fix() {
    let dir = scratch("config");
    let cases = [
        ("[lint]\nexclude = [\n", "TOML parse error at line 2"),
        (
            "[lint.rules]\nfloat_equalty = \"deny\"\n",
            "unknown rule `float_equalty`\nhelp: did you mean `float_equality`?",
        ),
        (
            "[lint.rules]\nfloat_equality = \"loud\"\n",
            "unknown level \"loud\" for `float_equality`\nhelp: use \"allow\", \"warn\" or \"deny\"",
        ),
        ("[lint]\nmax-nesting = \"deep\"\n", "invalid type: string"),
    ];
    for (config, expected) in cases {
        std::fs::write(dir.join("jacket.toml"), config).unwrap();
        let (text, code) = jailint(&dir, &["game.jai"]);
        assert_eq!(code, Some(2), "{config:?}: {text}");
        assert!(
            text.starts_with("error[cli-failure]: in `"),
            "{config:?}: {text}"
        );
        assert!(text.contains(expected), "{config:?}: {text}");
    }
    std::fs::remove_file(dir.join("jacket.toml")).unwrap();
    let (text, code) = jailint(&dir, &["--config", "missing.toml", "game.jai"]);
    assert_eq!(code, Some(2));
    assert!(
        text.contains(
            "error[cli-failure]: could not read config `missing.toml`: no such file or directory"
        ),
        "{text}"
    );
}

#[test]
fn a_file_a_module_loads_is_checked_as_part_of_the_module() {
    let dir = scratch("module-part");
    let module = dir.join("Thing");
    std::fs::create_dir_all(&module).unwrap();
    std::fs::write(module.join("module.jai"), "#load \"part.jai\";\n").unwrap();
    // `get` is exported: its parameter belongs to the module's interface even when unused.
    std::fs::write(
        module.join("part.jai"),
        "get :: (w: int = -1) -> int { return 4; }\n\
         twice :: () -> int { unused := 1; return get() * 2; }\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(["-D", "warnings", "Thing/part.jai"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(!text.contains("unused parameter"), "{text}");
    assert!(text.contains("unused variable `unused`"), "{text}");
    assert_eq!(output.status.code(), Some(1), "{text}");
}

#[test]
fn a_file_a_nearby_program_loads_is_checked_as_part_of_that_program() {
    let dir = scratch("loader-root");
    let part = dir.join("tour");
    std::fs::create_dir_all(part.join("basics")).unwrap();
    // `basics.jai` has no entry point of its own: the program that loads it has to be the root.
    std::fs::write(
        part.join("main.jai"),
        "#load \"basics/basics.jai\";\nmain :: () { run_basics(); }\n",
    )
    .unwrap();
    std::fs::write(
        part.join("basics/basics.jai"),
        "run_basics :: () { unused := helper(1); }\nhelper :: (x: int) -> int { return x; }\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
            output.status.code(),
        )
    };
    let (out, err, code) = run(&["-D", "warnings", "-v", "tour/basics/basics.jai"]);
    assert!(err.contains("main.jai"), "root should be main.jai: {err}");
    assert!(!err.contains("did not compile completely"), "{err}");
    assert!(out.contains("unused variable `unused`"), "{out}");
    assert_eq!(code, Some(1), "{out}{err}");
    // Only the listed file is reported, not the loader's other findings.
    std::fs::write(
        part.join("main.jai"),
        "#load \"basics/basics.jai\";\nmain :: () { run_basics(); stray := 2; }\n",
    )
    .unwrap();
    let (out, _, _) = run(&["-D", "warnings", "tour/basics/basics.jai"]);
    assert!(!out.contains("stray"), "{out}");
}

#[test]
fn fix_does_not_let_nested_index_loops_capture_each_others_it() {
    let dir = scratch("fix-nested");
    let before = "\
#import \"Basic\";
main :: () {
    str := \"hello\";
    to_find := \"lo\";
    for i: 0..str.count - 1 {
        for j: 0..to_find.count - 1 {
            if str[i] == to_find[j] print(\"%\\n\", i);
        }
    }
}
";
    // Only the inner loop is rewritten: the outer one would read the inner `it`.
    let after = "\
#import \"Basic\";
main :: () {
    str := \"hello\";
    to_find := \"lo\";
    for i: 0..str.count - 1 {
        for to_find {
            if str[i] == it print(\"%\\n\", i);
        }
    }
}
";
    let file = dir.join("game.jai");
    std::fs::write(&file, before).unwrap();
    let (text, _) = jailint(&dir, &["--fix", "game.jai"]);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), after, "{text}");
    // A second run finds nothing more to change.
    let (text, _) = jailint(&dir, &["--fix", "game.jai"]);
    assert!(!text.contains("fixed"), "{text}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), after);
}

#[test]
fn fix_leaves_imports_written_inside_expressions_alone() {
    let dir = scratch("fix-expression-import");
    // The directive is part of the statement around it: removing it would leave `!` behind.
    let before = "!#import \"Basic\";\nG :#no_aoc: #import \"GetRect\";\nmain :: () {}\n";
    let file = dir.join("game.jai");
    std::fs::write(&file, before).unwrap();
    let (text, _) = jailint(&dir, &["--fix", "--allow-incomplete", "game.jai"]);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before, "{text}");
}

#[test]
fn fix_removes_the_notes_of_a_removed_statement() {
    let dir = scratch("fix-notes");
    // A note belongs to the statement before it; left behind it would start the file.
    let before = "#import \"File\"; @first\n@second\n#import \"Basic\";\nmain :: () {}\n";
    let file = dir.join("game.jai");
    std::fs::write(&file, before).unwrap();
    let (text, _) = jailint(&dir, &["--fix", "game.jai"]);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "main :: () {}\n",
        "{text}"
    );
}

#[test]
fn a_program_that_does_not_compile_fails_the_run_unless_incomplete_is_allowed() {
    let dir = scratch("incomplete");
    std::fs::write(
        dir.join("broken.jai"),
        "main :: () { missing_procedure(); }\n",
    )
    .unwrap();
    let (err, code) = jailint(&dir, &["-D", "warnings", "broken.jai"]);
    // A diagnostic of its own, like the findings and the errors.
    assert!(
        err.contains("warning[lint-incomplete]: broken.jai did not compile completely"),
        "{err}"
    );
    assert_eq!(code, Some(1), "{err}");
    let (err, code) = jailint(
        &dir,
        &["-D", "warnings", "--allow-incomplete", "broken.jai"],
    );
    assert!(
        err.contains("warning[lint-incomplete]: 1 program did not compile completely"),
        "{err}"
    );
    assert!(err.contains("help: -v lists them"), "{err}");
    assert_eq!(code, Some(0), "{err}");
}

#[test]
fn exclude_applies_with_a_relative_config_path() {
    let dir = scratch("relative-config");
    std::fs::create_dir_all(dir.join("skip")).unwrap();
    std::fs::write(dir.join("skip/bad.jai"), "main :: () { x := 1; }\n").unwrap();
    std::fs::write(dir.join("jacket.toml"), "[lint]\nexclude = [\"skip/**\"]\n").unwrap();
    let (text, _) = jailint(&dir, &["skip/bad.jai"]);
    assert!(!text.contains("1 warning"), "{text}");
    let (text, _) = jailint(&dir, &["--config", "jacket.toml", "skip/bad.jai"]);
    assert!(!text.contains("1 warning"), "{text}");
    std::fs::remove_file(dir.join("jacket.toml")).unwrap();
    let (text, _) = jailint(&dir, &["skip/bad.jai"]);
    assert!(text.contains("1 warning"), "{text}");
}

#[test]
fn an_unknown_setting_is_a_configuration_error() {
    let dir = scratch("config-note");
    std::fs::write(dir.join("jacket.toml"), "[lint]\nmax-nestng = 3\n").unwrap();
    let (err, code) = jailint(&dir, &["game.jai"]);
    assert!(
        err.contains("unknown setting `lint.max-nestng` in jacket.toml (line 2)"),
        "{err}"
    );
    assert!(
        err.contains("help: did you mean `lint.max-nesting`?"),
        "{err}"
    );
    assert_eq!(code, Some(2), "{err}");
}

#[test]
fn test_notes_that_do_not_parse_are_warned_about_once() {
    let dir = scratch("bad-test-note");
    std::fs::write(
        dir.join("game.jai"),
        "// test: only(.NOWHERE) skip_on(.WASM)\nmain :: () {}\n",
    )
    .unwrap();
    let (err, code) = jailint(&dir, &["game.jai"]);
    assert!(
        err.contains(
            "warning[lint-test-note]: game.jai: `@only` does not know the target `nowhere`"
        ),
        "{err}"
    );
    assert_eq!(err.matches("lint-test-note").count(), 1, "{err}");
    assert_eq!(code, Some(0), "{err}");
    let (err, _) = jailint(&dir, &["."]);
    assert_eq!(err.matches("lint-test-note").count(), 1, "{err}");
}

#[test]
fn test_notes_that_do_not_parse_are_warned_about_in_loaded_files() {
    let dir = scratch("bad-test-note-loaded");
    std::fs::write(
        dir.join("game.jai"),
        "#load \"part.jai\";\n#load \"other.jai\";\nmain :: () { part(); other(); }\n",
    )
    .unwrap();
    for name in ["part", "other"] {
        std::fs::write(
            dir.join(format!("{name}.jai")),
            format!("// test: only(.NOWHERE)\n{name} :: () {{}}\n"),
        )
        .unwrap();
    }
    for args in [&["game.jai"][..], &["."][..]] {
        let (err, _) = jailint(&dir, args);
        for name in ["part", "other"] {
            let line = format!(
                "warning[lint-test-note]: {name}.jai: `@only` does not know the target `nowhere`"
            );
            assert_eq!(err.matches(&line).count(), 1, "{args:?}: {err}");
        }
        assert_eq!(err.matches("lint-test-note").count(), 2, "{args:?}: {err}");
    }
}

#[test]
fn a_directory_scan_leaves_out_tests_compile_fail_fixtures() {
    let dir = scratch("compile-fail-folder");
    std::fs::create_dir_all(dir.join("tests/compile_fail")).unwrap();
    std::fs::write(dir.join("tests/ok.jai"), "main :: () {}\n").unwrap();
    // No `// test: compile_fail` note: jacket test knows it from the folder.
    std::fs::write(
        dir.join("tests/compile_fail/bad.jai"),
        "main :: () { missing_procedure(); }\n",
    )
    .unwrap();
    let (err, code) = jailint(&dir, &["."]);
    assert!(!err.contains("lint-incomplete"), "{err}");
    assert_eq!(code, Some(0), "{err}");
    // Named on its own, the fixture is still linted.
    let (err, code) = jailint(&dir, &["tests/compile_fail/bad.jai"]);
    assert!(err.contains("lint-incomplete"), "{err}");
    assert_eq!(code, Some(1), "{err}");
}

#[test]
fn compiler_warnings_are_reported_and_take_the_level_flags() {
    let dir = scratch("compiler-warnings");
    std::fs::write(
        dir.join("main.jai"),
        "main :: () {\n    a1: [..] u8 = .{};\n    _ = a1;\n}\n",
    )
    .unwrap();
    let run = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
            .args(args)
            .arg("main.jai")
            .current_dir(&dir)
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        (
            String::from_utf8_lossy(&output.stdout).into_owned(),
            output.status.code(),
        )
    };
    let (out, code) = run(&[]);
    assert!(
        out.contains("warning[struct-literal-as-dynamic-array]"),
        "{out}"
    );
    assert!(out.contains("main.jai:2:"), "{out}");
    // The linter's own notes are for its rules, not the compiler's.
    assert!(!out.contains("by default"), "{out}");
    assert_eq!(code, Some(0), "{out}");
    let (out, code) = run(&["-D", "warnings"]);
    assert!(
        out.contains("error[struct-literal-as-dynamic-array]"),
        "{out}"
    );
    assert_eq!(code, Some(1), "{out}");
    let (out, code) = run(&["-A", "struct-literal-as-dynamic-array"]);
    assert!(!out.contains("struct-literal-as-dynamic-array"), "{out}");
    assert_eq!(code, Some(0), "{out}");
    let (out, code) = run(&["-D", "struct-literal-as-dynamic-array"]);
    assert!(
        out.contains("error[struct-literal-as-dynamic-array]"),
        "{out}"
    );
    assert_eq!(code, Some(1), "{out}");
}

#[test]
fn fix_applies_the_compilers_safe_suggestions() {
    let dir = scratch("fix-compiler-suggestion");
    let file = dir.join("game.jai");
    std::fs::write(&file, "main :: () {\n    x := 1e9;\n    _ = x;\n}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jailint"))
        .args(["-A", "all", "game.jai"])
        .current_dir(&dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let out = String::from_utf8_lossy(&output.stdout);
    assert!(
        out.contains("warning[float-exponent-without-fraction]"),
        "{out}"
    );
    assert!(out.contains("help: write `1.0e9`"), "{out}");
    let (text, code) = jailint(&dir, &["--fix", "-A", "all", "game.jai"]);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "main :: () {\n    x := 1.0e9;\n    _ = x;\n}\n",
        "{text}"
    );
    assert_eq!(code, Some(0), "{text}");
}

#[test]
fn fix_writes_the_float32_spelling_of_a_literal_float32_cannot_hold() {
    let dir = scratch("fix-float32-literal");
    let file = dir.join("game.jai");
    std::fs::write(
        &file,
        "A: float32 : 16777.2171234;\nmain :: () {\n    pi: float32 = 31415.92653;\n    pi = -98765.43210987;\n    _ = pi + A;\n}\n",
    )
    .unwrap();
    let (text, code) = jailint(&dir, &["--fix", "-A", "all", "game.jai"]);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "A: float32 : 16777.217;\nmain :: () {\n    pi: float32 = 31415.926;\n    pi = -98765.43;\n    _ = pi + A;\n}\n",
        "{text}"
    );
    assert_eq!(code, Some(0), "{text}");
}

#[test]
fn a_compile_time_run_over_the_step_budget_is_reported_as_incomplete() {
    // With no wall-clock limit, the evaluation budget is the only bound on a `#run`.
    let dir = scratch("step-budget");
    std::fs::write(
        dir.join("spin.jai"),
        "spin :: () -> int {\n    total := 0;\n    for 0..100000 total += it;\n    return total;\n}\n\nvalue :: #run spin();\n\nmain :: () {}\n",
    )
    .unwrap();
    let (err, code) = jailint(
        &dir,
        &["-D", "warnings", "--compile-time-budget", "1", "spin.jai"],
    );
    assert!(
        err.contains("warning[lint-incomplete]: spin.jai did not compile completely"),
        "{err}"
    );
    assert_eq!(code, Some(1), "{err}");
    // Within the budget the same program compiles completely.
    let (err, code) = jailint(&dir, &["-D", "warnings", "spin.jai"]);
    assert!(!err.contains("lint-incomplete"), "{err}");
    assert_eq!(code, Some(0), "{err}");
}

#[test]
fn a_librarys_tests_import_it_by_its_folder_name_as_jacket_test_does() {
    let dir = scratch("library-tests");
    std::fs::create_dir_all(dir.join("slugs/tests")).unwrap();
    std::fs::write(
        dir.join("slugs/module.jai"),
        "twice :: (x: int) -> int { return x * 2; }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("slugs/tests/cases.jai"),
        "#import \"Basic\";\n#import \"slugs\";\nmain :: () {\n    assert(twice(2) == 4);\n}\n",
    )
    .unwrap();
    let (text, code) = jailint(&dir, &["slugs/tests/cases.jai"]);
    assert!(!text.contains("lint-incomplete"), "{text}");
    assert!(!text.contains("not found"), "{text}");
    assert_eq!(code, Some(0), "{text}");
}
