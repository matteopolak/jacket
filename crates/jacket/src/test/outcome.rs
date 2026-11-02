//! What happened to a case: its status and what it reported.
use super::exec::{Failure, Subtest};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Skip,
    Timeout,
    /// Failed as its `@xfail` says it does.
    XFail,
    /// Passed despite its `@xfail`: an error.
    XPass,
    /// Failed, then passed on a retry.
    Flaky,
}

impl Status {
    /// Whether the run fails because of it (`flaky_fails` decides about FLAKY).
    pub fn fails(self, flaky_fails: bool) -> bool {
        matches!(self, Status::Fail | Status::Timeout | Status::XPass)
            || (flaky_fails && self == Status::Flaky)
    }
}

/// What happened to a case.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub case: usize,
    pub status: Status,
    pub duration: Duration,
    pub failures: Vec<Failure>,
    /// Failures that are not a check's: a crash, a diff, compile errors.
    pub problems: Vec<String>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub subtests: Vec<Subtest>,
    pub skip_reason: Option<String>,
    pub attempts: u32,
    pub crashed: bool, // the process died of a fault; its output tails are shown
    /// The test's temporary folder, kept because it failed.
    pub kept_temp: Option<PathBuf>,
    /// Expected-output files written because they did not exist yet.
    pub created: Vec<PathBuf>,
}

impl Outcome {
    pub(super) fn new(case: usize, status: Status) -> Outcome {
        Outcome {
            case,
            status,
            duration: Duration::ZERO,
            failures: Vec::new(),
            problems: Vec::new(),
            stdout: Vec::new(),
            stderr: Vec::new(),
            subtests: Vec::new(),
            skip_reason: None,
            attempts: 1,
            crashed: false,
            kept_temp: None,
            created: Vec::new(),
        }
    }
}
