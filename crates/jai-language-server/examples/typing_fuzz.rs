//! Seeded typing sessions over source files; see docs/tools/lsp-fuzzing.md.
//!
//! ```text
//! cargo run --release -p jai-language-server --example typing_fuzz -- \
//!     [--seeds A..B] [--scenarios N] [--diff-every N] [--stdlib DIR] FILE...
//! ```
//!
//! Each file runs once per seed; failures print the file, seed and the edits that led to them, and
//! the exit status is 1 when there are any.
#[path = "support/typing_files.rs"]
mod typing_files;
#[path = "support/typing_session.rs"]
mod typing_session;

use std::path::PathBuf;
use typing_session::{Config, Input, Seeded};

fn main() {
    // Match jailsp's compiler thread; the parser's depth budget needs a large stack.
    let code = std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(run)
        .expect("spawn")
        .join()
        .expect("typing_fuzz panicked");
    std::process::exit(code);
}

fn run() -> i32 {
    let mut seeds = 0..4u64;
    let mut config = Config::default();
    let mut stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| panic!("{arg} needs a value"));
        match arg.as_str() {
            "--seeds" => {
                let range = value();
                let (a, b) = range.split_once("..").expect("--seeds A..B");
                seeds = a.parse().expect("seed")..b.parse().expect("seed");
            }
            "--scenarios" => config.scenarios = value().parse().expect("count"),
            "--diff-every" => config.diff_every = value().parse().expect("count"),
            "--stdlib" => stdlib = PathBuf::from(value()),
            "--list" => {
                let list = std::fs::read_to_string(value()).expect("--list FILE");
                files.extend(list.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
            }
            _ => files.push(PathBuf::from(arg)),
        }
    }
    let stdlib = stdlib.canonicalize().expect("stdlib directory");
    let environment = || typing_files::native_environment(stdlib.clone());
    let mut failed = false;
    for file in &files {
        let Some(opened) = typing_files::read_file_input(file) else {
            eprintln!("skipped {} (unreadable)", file.display());
            continue;
        };
        let input = Input {
            setup: opened.setup,
            docs: opened.docs,
            environment: &environment,
        };
        for seed in seeds.clone() {
            let report = typing_session::run(&input, config.clone(), Seeded(seed));
            println!(
                "{} seed {seed}: {} versions, {} fresh, {} baseline, {} outside, {} failures",
                file.display(),
                report.versions,
                report.fresh_comparisons,
                report.baseline_comparisons,
                report.outside_comparisons,
                report.failures.len()
            );
            for failure in &report.failures {
                failed = true;
                println!("FAIL {} seed {seed}: {failure}", file.display());
            }
            for (doc, history) in &report.histories {
                let Some(small) = typing_session::minimise_history(&input, *doc, history, &config)
                else {
                    println!(
                        "replaying the {} steps does not show it",
                        history.steps.len()
                    );
                    continue;
                };
                println!(
                    "MINIMISED to {} of {} steps:",
                    small.steps.len(),
                    history.steps.len()
                );
                for step in &small.steps {
                    println!("    {:?},  // {} requests", step.text, step.queries.len());
                }
            }
        }
    }
    i32::from(failed)
}
