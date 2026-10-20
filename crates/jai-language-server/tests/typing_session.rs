//! A fixed-seed slice of the typing-session checker (see docs/tools/lsp-fuzzing.md). The example
//! `typing_fuzz` runs the same checker over many more files and seeds.
#[path = "../examples/support/typing_files.rs"]
mod typing_files;
#[path = "../examples/support/typing_session.rs"]
mod typing_session;

use std::path::{Path, PathBuf};
use typing_session::{Config, Input, Seeded};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Run `file` for each seed; the checker's report on every failure, with its minimised edits.
fn check(file: &str, seeds: std::ops::Range<u64>, config: &Config) -> Vec<String> {
    let root = repository();
    let stdlib = root.join("stdlib");
    let environment = || typing_files::native_environment(stdlib.clone());
    let opened = typing_files::read_file_input(&root.join(file)).expect("readable fixture");
    let input = Input {
        setup: opened.setup,
        docs: opened.docs,
        environment: &environment,
    };
    let mut problems = Vec::new();
    for seed in seeds {
        let report = typing_session::run(&input, config.clone(), Seeded(seed));
        assert!(
            report.versions > 20,
            "{file} seed {seed}: {} versions",
            report.versions
        );
        for failure in &report.failures {
            let mut text = format!("{file} seed {seed}: {failure}");
            for (doc, history) in &report.histories {
                if let Some(small) = typing_session::minimise_history(&input, *doc, history, config)
                {
                    text.push_str("\nminimised edits:");
                    for step in &small.steps {
                        text.push_str(&format!("\n  {:?}", step.text));
                    }
                }
            }
            problems.push(text);
        }
    }
    problems
}

/// The checker needs the stack `jailsp`'s compiler thread has.
fn on_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1 << 29)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap()
}

fn quick() -> Config {
    Config {
        scenarios: 2,
        full_probes: 40,
        ..Config::default()
    }
}

#[test]
fn typing_in_a_program_that_loads_another_file_keeps_every_answer() {
    let problems = on_big_stack(|| check("tests/lsp/typing/main.jai", 0..2, &quick()));
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn typing_in_the_loaded_file_keeps_every_answer() {
    let problems = on_big_stack(|| check("tests/lsp/typing/lib.jai", 0..1, &quick()));
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

#[test]
fn typing_in_partial_fixtures_keeps_every_answer() {
    let problems = on_big_stack(|| {
        ["standalone.jai", "pending.jai"]
            .iter()
            .flat_map(|name| check(&format!("tests/lsp/partial/{name}"), 0..1, &quick()))
            .collect::<Vec<_>>()
    });
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}

/// A statement typed at the end of a body has the body checked again, with fresh locals for
/// its parameters: hovering one still shows its layout.
#[test]
fn typing_after_a_switch_keeps_parameter_hovers() {
    let config = Config {
        scenarios: 1,
        ..Config::default()
    };
    let problems =
        on_big_stack(move || check("stdlib/Extensions/Args/tests/help.jai", 0..1, &config));
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}
