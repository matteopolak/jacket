//! Each rule against its cases in `tests/lint/<rule>/`:
//!
//! - `bad.jai` must produce exactly the diagnostics in `bad.expected` (rendered without
//!   color), with only that rule enabled.
//! - `bad.fixed.jai`, when present, is `bad.jai` with the machine-applicable fixes applied.
//! - `good.jai` must produce nothing for that rule.
//!
//! `tests/lint/suppressed/` holds code every rule would fire on, silenced in source: it must
//! produce nothing with every rule enabled.
//!
//! Set `JAILINT_BLESS=1` to rewrite the expected files from the current output.
use jailint::config::{Config, Level};
use jailint::render::{Style, render};
use jailint::{Lint, RULES};
use std::path::{Path, PathBuf};

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
    let shown = file
        .strip_prefix(repo())
        .unwrap_or(file)
        .to_string_lossy()
        .into_owned();
    let mut out = String::new();
    let lines = jaic::source::LineIndex::new(text);
    for l in lints {
        let default = RULES
            .iter()
            .find(|r| r.name == l.rule)
            .map_or(Level::Warn, |r| r.default);
        out.push_str(&render(l, text, &lines, &shown, default, &Style::PLAIN));
        out.push('\n');
    }
    out
}

/// Compare `actual` with the file at `path` (or write it when blessing).
fn expect(path: &Path, actual: &str) {
    if std::env::var_os("JAILINT_BLESS").is_some() {
        std::fs::write(path, actual).expect("write the expected file");
        return;
    }
    let expected = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("missing {} (run with JAILINT_BLESS=1)", path.display()));
    assert_eq!(actual, expected, "{} differs", path.display());
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
    // Each rule has a row in the table and its own section, whose anchor editors link to
    // (`codeDescription` of the language server's diagnostics).
    let doc = std::fs::read_to_string(repo().join("docs/tools/jailint.md"))
        .expect("docs/tools/jailint.md is readable");
    for r in RULES {
        assert!(
            doc.contains(&format!("\n| `{}` | {} |", r.name, r.default.as_str())),
            "docs/tools/jailint.md: the rules table needs a row for `{}` with its default",
            r.name
        );
        assert!(
            doc.contains(&format!("\n### {}\n", r.name)),
            "docs/tools/jailint.md needs a `### {}` section",
            r.name
        );
    }
}

#[test]
fn rules_match_their_cases() {
    let dir = repo().join("tests/lint");
    for r in RULES {
        let level = match r.default {
            Level::Allow => Level::Warn,
            other => other,
        };
        let case = dir.join(r.name);
        let bad = case.join("bad.jai");
        let (lints, text) = lint(&bad, &[(r.name, level)]);
        assert!(
            !lints.is_empty(),
            "{} finds nothing in its bad case",
            r.name
        );
        expect(&case.join("bad.expected"), &rendered(&lints, &text, &bad));
        let refs: Vec<&Lint> = lints.iter().collect();
        let (fixed, applied) = jailint::fix::apply(&text, &refs);
        let fixed_path = case.join("bad.fixed.jai");
        if applied > 0 {
            expect(&fixed_path, &fixed);
        } else {
            assert!(!fixed_path.exists(), "{} has no fixes to apply", r.name);
        }
        let good = case.join("good.jai");
        let (lints, text) = lint(&good, &[(r.name, level)]);
        assert!(
            lints.is_empty(),
            "{} fires on its good case:\n{}",
            r.name,
            rendered(&lints, &text, &good)
        );
    }
}

#[test]
fn suppressions_silence_every_rule() {
    let file = repo().join("tests/lint/suppressed/main.jai");
    let levels: Vec<(&str, Level)> = RULES.iter().map(|r| (r.name, Level::Warn)).collect();
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
fn fixed_cases_are_clean() {
    // Applying the fixes must leave code the rule no longer fires on (and that compiles).
    let dir = repo().join("tests/lint");
    for r in RULES {
        let fixed = dir.join(r.name).join("bad.fixed.jai");
        if !fixed.exists() {
            continue;
        }
        let (lints, text) = lint(&fixed, &[(r.name, Level::Warn)]);
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
    expect(
        &dir.join("linux/window.expected"),
        &rendered(&lints, &text, &linux),
    );
}

#[test]
fn runs_in_one_process_share_no_state() {
    // `a/win.jai` has an unused parameter that `b/win.jai` (not loaded) also takes: a variant,
    // so no lint. Rewriting `b/win.jai` to other parameters and linting `a/win.jai` again
    // must see the new file, not what the first run read.
    let root = std::env::temp_dir().join(format!("jailint-runs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
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
    let _ = std::fs::remove_dir_all(&root);
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
        ["37", "8640", "45", "1.37", "-13", "3600", "0.3", "47"]
    );
    // An allowed number is allowed negated too; whole numbers below the minimum pass.
    let tuned = found(Config {
        magic_number_in_tests: Some(true),
        magic_number_allow: vec![13.0, 1.37, 3600.0],
        magic_number_min: Some(50.0),
        ..Config::default()
    });
    assert_eq!(tuned, ["8640", "0.3"]);
}
