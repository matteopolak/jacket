//! Compiling one program under several symbol-number scrambles.
//!
//! Symbol numbers depend on which parse thread met a spelling first, so output that follows them
//! changes from run to run. `JAIC_SYM_SCRAMBLE=<seed>` with `-serial_parse` spreads the numbers
//! out the same way every time, which turns that dependence into a difference between seeds.

use std::path::Path;
use std::process::{Command, Output};

/// The seeds [`assert_stable`] tries besides running without one.
pub const SEEDS: &[u64] = &[1, 2, 3];

fn run(jaic: &Path, program: &Path, args: &[&str], seed: Option<u64>) -> Output {
    let mut command = Command::new(jaic);
    command.arg(program).args(args).arg("-serial_parse");
    match seed {
        Some(seed) => command.env("JAIC_SYM_SCRAMBLE", seed.to_string()),
        None => command.env_remove("JAIC_SYM_SCRAMBLE"),
    };
    command
        .output()
        .unwrap_or_else(|error| panic!("cannot run {command:?}: {error}"))
}

/// Runs `jaic program args` with and without each of [`SEEDS`] and fails unless the exit status,
/// standard output and standard error are the same every time. Returns the unscrambled run.
///
/// # Panics
/// When two runs differ, naming the seed and showing both outputs.
pub fn assert_stable(jaic: &Path, program: &Path, args: &[&str]) -> Output {
    let (base, others) = std::thread::scope(|scope| {
        let runs: Vec<_> = SEEDS
            .iter()
            .map(|&seed| scope.spawn(move || (seed, run(jaic, program, args, Some(seed)))))
            .collect();
        let base = run(jaic, program, args, None);
        let others: Vec<_> = runs.into_iter().map(|run| run.join().unwrap()).collect();
        (base, others)
    });
    for (seed, other) in others {
        assert!(
            base.status.code() == other.status.code()
                && base.stdout == other.stdout
                && base.stderr == other.stderr,
            "{} differs under JAIC_SYM_SCRAMBLE={seed}\n--- unscrambled\n{}\n--- scrambled\n{}",
            program.display(),
            super::process::describe(&base),
            super::process::describe(&other),
        );
    }
    base
}
