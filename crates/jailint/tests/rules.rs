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
    let mut config = Config {
        root: repo(),
        ..Config::default()
    };
    for r in RULES {
        config.levels.insert(r.name.to_string(), Level::Allow);
    }
    for (rule, level) in levels {
        config.levels.insert(rule.to_string(), *level);
    }
    let options = jailint::driver::Options {
        paths: vec![file.to_path_buf()],
        import_dirs: Vec::new(),
        stdlib: repo().join("stdlib"),
        config,
        jobs: 1,
        verbose: false,
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
    for l in lints {
        let default = RULES
            .iter()
            .find(|r| r.name == l.rule)
            .map_or(Level::Warn, |r| r.default);
        out.push_str(&render(
            l,
            text,
            &shown,
            default,
            &Style {
                color: false,
            },
        ));
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
fn fixed_cases_are_clean() {
    // Applying the fixes must leave code the rule no longer fires on (and that compiles).
    let dir = repo().join("tests/lint");
    for r in RULES {
        let fixed = dir.join(r.name).join("bad.fixed.jai");
        if !fixed.exists() {
            continue;
        }
        let (lints, text) = lint(&fixed, &[(r.name, Level::Warn)]);
        assert!(
            lints.is_empty(),
            "{} still fires after its fixes:\n{}",
            r.name,
            rendered(&lints, &text, &fixed)
        );
    }
}
