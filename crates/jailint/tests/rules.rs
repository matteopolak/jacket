//! Each rule against its cases in `tests/lint/<rule>/`:
//!
//! - `bad.jai` must produce exactly the diagnostics in the snapshot `bad.snap` (rendered
//!   without color), with only that rule enabled.
//! - `bad.fixed.jai`, when present, is `bad.jai` with the machine-applicable fixes applied.
//!   It stays a Jai file because `fixed_cases_are_clean` lints it again.
//! - `good.jai` must produce nothing for that rule.
//!
//! `tests/lint/suppressed/` holds code every rule would fire on, silenced in source: it must
//! produce nothing with every rule enabled.
//!
//! Run with `INSTA_UPDATE=always` to rewrite the snapshots and `bad.fixed.jai` files from the
//! current output.
use jailint::config::{Config, Level};
use jailint::render::{Style, render};
use jailint::{Lint, RULES};
use std::path::{Path, PathBuf};
use test_support::TempDir;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists")
}

/// Lint `file` with `levels` on top of every rule allowed.
fn lint(file: &Path, levels: &[(&str, Level)]) -> (Vec<Lint>, String) {
    lint_with(
        file,
        levels,
        Config {
            // The cases live in `tests/lint`, which `magic_number` skips by default.
            magic_number_in_tests: Some(true),
            ..Config::default()
        },
    )
}

/// [`lint`] with other settings in `config`.
fn lint_with(file: &Path, levels: &[(&str, Level)], mut config: Config) -> (Vec<Lint>, String) {
    config.root = repo();
    // `platform_leak`'s cases: their folder is held to every system, and POSIX is Unix only.
    config.portability = jailint::config::Portability {
        targets: Vec::new(),
        portable: vec!["tests/lint/platform_leak/**".parse().unwrap()],
        platforms: vec![(
            "stdlib/POSIX/**".parse().unwrap(),
            vec![jaic::target::TargetOs::Linux, jaic::target::TargetOs::MacOS],
        )],
    };
    for r in RULES {
        config.levels.insert(r.name.to_string(), Level::Allow);
    }
    for (rule, level) in levels {
        config.levels.insert(rule.to_string(), *level);
    }
    let options = jailint::driver::Options {
        compiler_options: jaic::sema::Options::host(),
        source: None,
        paths: vec![file.to_path_buf()],
        import_dirs: Vec::new(),
        stdlib: jaic::stdlib_dir(repo().join("stdlib")),
        config,
        jobs: 1,
        verbose: false,
        add_sources: Vec::new(),
        limits: jaic::ide::lint_compile::CheckLimits::default(),
    };
    let outcome = jailint::driver::run(&options);
    assert!(
        outcome.incomplete.is_empty(),
        "{} must compile: {:?}",
        file.display(),
        outcome.incomplete
    );
    let text = std::fs::read_to_string(file).expect("the case is readable");
    (outcome.lints, text)
}

fn rendered(lints: &[Lint], text: &str, file: &Path) -> String {
    let shown = test_support::snapshot::slashed(file.strip_prefix(repo()).unwrap_or(file));
    let mut out = String::new();
    let lines = jaic::source::LineIndex::new(text);
    for l in lints {
        let default = RULES
            .iter()
            .find(|r| r.name == l.rule)
            .map_or(Level::Warn, jailint::RuleInfo::default);
        out.push_str(&render(l, text, &lines, &shown, default, &Style::PLAIN));
        out.push('\n');
    }
    out
}

/// Compare `actual` with the Jai file at `path` (or rewrite it when updating snapshots).
fn expect_file(path: &Path, actual: &str) {
    test_support::snapshot::assert_generated_file(path, actual, "");
}

/// Rules a case needs on besides its own: a suppression is only unused when the rules it
/// names ran.
fn companions(rule: &str) -> &'static [&'static str] {
    match rule {
        "unused_suppression" => &[
            "unused_variable",
            "unused_parameter",
            "float_equality",
            "todo_marker",
        ],
        _ => &[],
    }
}

/// The levels for the case of `rule`: the rule at `level`, and its companions at `warn`.
fn case_levels(rule: &str, level: Level) -> Vec<(&str, Level)> {
    std::iter::once((rule, level))
        .chain(companions(rule).iter().map(|c| (*c, Level::Warn)))
        .collect()
}

#[test]
fn every_rule_has_cases() {
    let dir = repo().join("tests/lint");
    for r in RULES {
        assert!(
            dir.join(r.name).join("bad.jai").is_file()
                && dir.join(r.name).join("good.jai").is_file(),
            "tests/lint/{}/ needs bad.jai and good.jai",
            r.name
        );
    }
}

#[test]
fn every_rule_is_documented() {
    // Each rule has its own section, whose anchor editors link to (`codeDescription` of the
    // language server's diagnostics). The table of rules is generated: docs/reference/lint-rules.md.
    let doc = std::fs::read_to_string(repo().join("docs/tools/jailint.md"))
        .expect("docs/tools/jailint.md is readable");
    for r in RULES {
        assert!(
            doc.contains(&format!("\n### {}\n", r.name)),
            "docs/tools/jailint.md needs a `### {}` section",
            r.name
        );
    }
}

/// How many tests share the rules' cases (each takes every `RULE_PARTS`-th rule), so they run side by side.
const RULE_PARTS: usize = 8;

fn rules_match_their_cases(part: usize) {
    let dir = repo().join("tests/lint");
    for r in RULES.iter().skip(part).step_by(RULE_PARTS) {
        let level = match r.default() {
            Level::Allow => Level::Warn,
            other => other,
        };
        let case = dir.join(r.name);
        let bad = case.join("bad.jai");
        let (lints, text) = lint(&bad, &case_levels(r.name, level));
        assert!(
            !lints.is_empty(),
            "{} finds nothing in its bad case",
            r.name
        );
        test_support::assert_snapshot_in!(&case, "bad", rendered(&lints, &text, &bad));
        let refs: Vec<&Lint> = lints.iter().collect();
        let (fixed, applied) = jailint::fix::apply(&text, &refs);
        let fixed_path = case.join("bad.fixed.jai");
        if applied > 0 {
            expect_file(&fixed_path, &fixed);
        } else {
            assert!(!fixed_path.exists(), "{} has no fixes to apply", r.name);
        }
        let good = case.join("good.jai");
        let (lints, text) = lint(&good, &case_levels(r.name, level));
        assert!(
            lints.is_empty(),
            "{} fires on its good case:\n{}",
            r.name,
            rendered(&lints, &text, &good)
        );
    }
}

macro_rules! rule_case_parts {
    ($($name:ident: $part:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                rules_match_their_cases($part);
            }
        )*
    };
}

rule_case_parts!(
    rules_match_their_cases_part_1: 0,
    rules_match_their_cases_part_2: 1,
    rules_match_their_cases_part_3: 2,
    rules_match_their_cases_part_4: 3,
    rules_match_their_cases_part_5: 4,
    rules_match_their_cases_part_6: 5,
    rules_match_their_cases_part_7: 6,
    rules_match_their_cases_part_8: 7,
);

#[test]
fn suppressions_silence_every_rule() {
    let file = repo().join("tests/lint/suppressed/main.jai");
    let levels: Vec<(&str, Level)> = RULES.iter().map(|r| (r.name, Level::Warn)).collect();
    let (lints, text) = lint(&file, &levels);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn programs_may_print_and_exit() {
    // `print_in_module` and `exit_in_module` look only at files that belong to a module.
    let file = repo().join("tests/lint/module_scope/main.jai");
    let levels = [
        ("print_in_module", Level::Warn),
        ("exit_in_module", Level::Warn),
    ];
    let (lints, text) = lint(&file, &levels);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
    // So may the programs in a `tests` folder inside a module's directory.
    let file = repo().join("tests/lint/print_in_module/tests/program.jai");
    let (lints, text) = lint(&file, &levels);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn generated_files_are_skipped_by_unused_rules() {
    let file = repo().join("tests/lint/unused_import/generated.jai");
    let levels: Vec<(&str, Level)> = [
        "unused_import",
        "unused_parameter",
        "unused_variable",
        "unused_result",
    ]
    .into_iter()
    .map(|r| (r, Level::Warn))
    .collect();
    let (lints, text) = lint(&file, &levels);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn private_declarations_in_files_with_insert_are_left_alone() {
    let file = repo().join("tests/lint/unused_private_declaration/insert.jai");
    let (lints, text) = lint(&file, &[("unused_private_declaration", Level::Warn)]);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn files_generated_with_a_tool_are_skipped_by_enum_member_prefix() {
    let file = repo().join("tests/lint/enum_member_prefix/generated.jai");
    let (lints, text) = lint(&file, &[("enum_member_prefix", Level::Warn)]);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &file));
}

#[test]
fn fixed_cases_are_clean() {
    // Applying the fixes must leave code the rule no longer fires on (and that compiles).
    let dir = repo().join("tests/lint");
    for r in RULES {
        let fixed = dir.join(r.name).join("bad.fixed.jai");
        if !fixed.exists() {
            continue;
        }
        let (lints, text) = lint(&fixed, &case_levels(r.name, Level::Warn));
        // The linter skips a file that does not parse, so a broken fix would pass as clean.
        if let Err(error) = jaic::parser::parse_file(jaic::source::FileId(0), &text) {
            panic!("{} does not parse: {error:?}", fixed.display());
        }
        // What is left carries no fix to apply (a nested loop that keeps `index_only_loop`
        // from using `it`).
        let lints: Vec<Lint> = lints
            .into_iter()
            .filter(|l| l.fix.as_ref().is_some_and(|f| f.machine_applicable))
            .collect();
        assert!(
            lints.is_empty(),
            "{} still has fixes to apply after its fixes:\n{}",
            r.name,
            rendered(&lints, &text, &fixed)
        );
    }
}

#[test]
fn unused_parameter_variants_one_directory_over() {
    // `platform/macos/window.jai` and `platform/windows/window.jai` define `set_title` with the
    // same parameter names; the build loads one of them. `platform/linux/window.jai` takes other
    // parameters, so it is no variant and its unused one is reported.
    let dir = repo().join("tests/lint/unused_parameter/platform");
    let levels = [("unused_parameter", Level::Warn)];
    let macos = dir.join("macos/window.jai");
    let (lints, text) = lint(&macos, &levels);
    assert!(lints.is_empty(), "{}", rendered(&lints, &text, &macos));
    let linux = dir.join("linux/window.jai");
    let (lints, text) = lint(&linux, &levels);
    assert_eq!(lints.len(), 1);
    test_support::assert_snapshot_in!(
        &dir.join("linux"),
        "window",
        rendered(&lints, &text, &linux)
    );
}

#[test]
fn runs_in_one_process_share_no_state() {
    // `a/win.jai` has an unused parameter that `b/win.jai` (not loaded) also takes: a variant,
    // so no lint. Rewriting `b/win.jai` to other parameters and linting `a/win.jai` again
    // must see the new file, not what the first run read.
    let root = TempDir::new("jailint-runs");
    std::fs::create_dir_all(root.join("a")).unwrap();
    std::fs::create_dir_all(root.join("b")).unwrap();
    let a = root.join("a/win.jai");
    let b = root.join("b/win.jai");
    std::fs::write(
        &a,
        "set_title :: (w: int, title: string) -> int {\n    return w;\n}\n\nmain :: () {\n    x := set_title(1, \"t\");\n    if x == 99 return;\n}\n",
    )
    .unwrap();
    std::fs::write(
        &b,
        "set_title :: (w: int, title: string) -> int {\n    return title.count + w;\n}\n",
    )
    .unwrap();
    let levels = [("unused_parameter", Level::Warn)];
    let (first, text) = lint(&a, &levels);
    assert!(first.is_empty(), "{}", rendered(&first, &text, &a));
    std::fs::write(
        &b,
        "set_title :: (w: int, name: string, extra: int) -> int {\n    return w + extra + name.count;\n}\n",
    )
    .unwrap();
    let (second, _) = lint(&a, &levels);
    assert_eq!(second.len(), 1, "{second:?}");
    assert!(second[0].message.contains("title"));
}

#[test]
fn magic_number_settings() {
    let file = repo().join("tests/lint/magic_number/bad.jai");
    let levels = [("magic_number", Level::Warn)];
    let found = |config: Config| -> Vec<String> {
        let (lints, text) = lint_with(&file, &levels, config);
        lints
            .iter()
            .map(|l| text[l.start..l.end].to_string())
            .collect()
    };
    // Files in a `tests` folder are skipped unless asked for.
    assert!(found(Config::default()).is_empty());
    let all = found(Config {
        magic_number_in_tests: Some(true),
        ..Config::default()
    });
    assert_eq!(
        all,
        [
            "37", "8640", "45", "1.37", "-13", "3600", "0.3", "47", "97", "34", "92", "9", "65",
            "88"
        ]
    );
    // An allowed number is allowed negated too; whole numbers below the minimum pass.
    let tuned = found(Config {
        magic_number_in_tests: Some(true),
        magic_number_allow: vec![13.0, 1.37, 3600.0],
        magic_number_min: Some(50.0),
        ..Config::default()
    });
    assert_eq!(tuned, ["8640", "0.3", "97", "92", "65", "88"]);
}

#[test]
fn cognitive_complexity_limit_comes_from_the_settings() {
    let file = repo().join("tests/lint/cognitive_complexity/good.jai");
    let levels = [("cognitive_complexity", Level::Warn)];
    let config = |limit| Config {
        max_complexity: Some(limit),
        ..Config::default()
    };
    let (lints, _) = lint_with(&file, &levels, config(20));
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(
        lints[0]
            .message
            .contains("complexity of 21, past the limit of 20")
    );
    let (lints, _) = lint_with(&file, &levels, config(21));
    assert!(lints.is_empty(), "{lints:?}");
}

#[test]
fn parameter_limits_come_from_the_settings() {
    let args = repo().join("tests/lint/too_many_arguments/good.jai");
    let levels = [("too_many_arguments", Level::Warn)];
    let config = |limit| Config {
        max_arguments: Some(limit),
        ..Config::default()
    };
    // `seven` takes seven parameters and `draw` two.
    let (lints, _) = lint_with(&args, &levels, config(6));
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(
        lints[0]
            .message
            .contains("takes 7 parameters, past the limit of 6")
    );
    let (lints, _) = lint_with(&args, &levels, config(7));
    assert!(lints.is_empty(), "{lints:?}");

    let bools = repo().join("tests/lint/excessive_bools/good.jai");
    let levels = [("excessive_bools", Level::Warn)];
    let config = |limit| Config {
        max_bools: Some(limit),
        ..Config::default()
    };
    let (lints, _) = lint_with(&bools, &levels, config(2));
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(
        lints[0]
            .message
            .contains("takes 3 `bool` parameters, past the limit of 2")
    );
    let (lints, _) = lint_with(&bools, &levels, config(3));
    assert!(lints.is_empty(), "{lints:?}");
}

#[test]
fn stack_array_limit_comes_from_the_settings() {
    let file = repo().join("tests/lint/large_stack_array/bad.jai");
    let levels = [("large_stack_array", Level::Warn)];
    let config = |limit| Config {
        max_stack_array_bytes: Some(limit),
        ..Config::default()
    };
    // `scratch` takes a MiB and `pixels` 40,000 bytes.
    let (lints, _) = lint_with(&file, &levels, config(30_000));
    assert_eq!(lints.len(), 2, "{lints:?}");
    let (lints, _) = lint_with(&file, &levels, config(65_536));
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(
        lints[0]
            .message
            .contains("is 1 MiB on the stack, past the limit of 64 KiB")
    );
    let (lints, _) = lint_with(&file, &levels, config(1 << 20));
    assert!(lints.is_empty(), "{lints:?}");
}
