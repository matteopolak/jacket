//! Running the tests: jobs of one root's cases in one process (or one each), on a pool of
//! workers, and how each run's records, exit and timeouts become an outcome.
use super::build::Toolchain;
use super::exec::{self, Ending, Live};
use super::options::TestOptions;
use super::{Case, Kind, Message, Outcome, Plan, Status, build, coverage, doctest, golden};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Mutex, mpsc};
use std::time::{Duration, Instant};

/// A piece of work for one worker: cases of one root run in one process (or one each).
pub(super) struct Job {
    cases: Vec<usize>,
}

/// The jobs that run in parallel and those that run alone, `@slow` ones first.
pub(super) fn jobs(plan: &Plan, to_run: &[usize]) -> (Vec<Job>, Vec<Job>) {
    let mut order = to_run.to_vec();
    order.sort_by_key(|&i| !plan.cases[i].options.slow);
    let batched = plan.toolchain.interp || !plan.settings.isolate;
    let mut parallel = Vec::new();
    let mut serial = Vec::new();
    // Per root: unit tests to batch, the parallel ones and the serial ones apart.
    let mut batches: BTreeMap<(usize, bool), Vec<usize>> = BTreeMap::new();
    for &i in &order {
        let case = &plan.cases[i];
        let alone = case.options.serial;
        if case.kind == Kind::Test && batched {
            batches.entry((case.root, alone)).or_default().push(i);
            continue;
        }
        let job = Job {
            cases: vec![i],
        };
        if alone {
            serial.push(job);
        } else {
            parallel.push(job);
        }
    }
    for ((_, alone), cases) in batches {
        if alone {
            serial.push(Job {
                cases,
            });
            continue;
        }
        // One process per worker slot: each compiles the program again under `-interp`.
        let count = plan.settings.jobs.min(cases.len()).max(1);
        let mut split: Vec<Vec<usize>> = vec![Vec::new(); count];
        for (n, case) in cases.into_iter().enumerate() {
            split[n % count].push(case);
        }
        parallel.extend(split.into_iter().map(|cases| Job {
            cases,
        }));
    }
    (parallel, serial)
}

/// Runs `jobs` on `threads` workers, passing their messages to `report` on this thread.
pub(super) fn run_jobs(
    plan: &Plan,
    jobs: Vec<Job>,
    threads: usize,
    report: &mut dyn FnMut(Message),
) {
    if jobs.is_empty() {
        return;
    }
    let queue = Mutex::new(VecDeque::from(jobs));
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|scope| {
        for _ in 0..threads.max(1) {
            let tx = tx.clone();
            let queue = &queue;
            scope.spawn(move || {
                loop {
                    if plan.stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let Some(job) = queue.lock().expect("lock").pop_front() else {
                        break;
                    };
                    run_job(plan, &job, &tx);
                }
            });
        }
        drop(tx);
        for message in rx {
            report(message);
        }
    });
}

fn send_done(plan: &Plan, tx: &mpsc::Sender<Message>, outcome: Outcome) {
    if plan.settings.fail_fast && outcome.status.fails(plan.settings.flaky_fails) {
        plan.stop.store(true, Ordering::Relaxed);
    }
    let _ = tx.send(Message::Done(outcome));
}

fn run_job(plan: &Plan, job: &Job, tx: &mpsc::Sender<Message>) {
    let first = &plan.cases[job.cases[0]];
    if first.kind != Kind::Test {
        tx.send(Message::Started(job.cases[0])).ok();
        let outcome = with_retries(
            plan,
            job.cases[0],
            run_file_case(plan, job.cases[0]),
            &|| run_file_case(plan, job.cases[0]),
        );
        send_done(plan, tx, outcome);
        return;
    }
    let mut live = |message: Live| live_message(plan, &job.cases, tx, message);
    let outcomes = run_unit(plan, &job.cases, &mut live);
    for outcome in outcomes {
        let case = outcome.case;
        let outcome = with_retries(plan, case, outcome, &|| {
            let mut quiet = |_: Live| {};
            run_unit(plan, &[case], &mut quiet)
                .pop()
                .expect("one outcome")
        });
        send_done(plan, tx, outcome);
    }
}

fn live_message(plan: &Plan, cases: &[usize], tx: &mpsc::Sender<Message>, message: Live) {
    let find = |name: &str| {
        cases
            .iter()
            .copied()
            .find(|&i| plan.cases[i].local == exec::top_name(name))
    };
    match message {
        Live::Started(name) => {
            if let Some(case) = find(name) {
                tx.send(Message::Started(case)).ok();
            }
        }
        Live::Slow(name, elapsed) => {
            if let Some(case) = find(name).filter(|&c| !plan.cases[c].options.slow) {
                tx.send(Message::Slow(case, elapsed)).ok();
            }
        }
        Live::Output(bytes, out) => {
            use std::io::Write;
            if out {
                let _ = std::io::stdout().write_all(bytes);
            } else {
                let _ = std::io::stderr().write_all(bytes);
            }
        }
    }
}

/// `first`, or, when it failed and retries are left, the first retry that passes (FLAKY) or the
/// last failure.
fn with_retries(plan: &Plan, case: usize, first: Outcome, again: &dyn Fn() -> Outcome) -> Outcome {
    let retries = plan.cases[case]
        .options
        .retries
        .unwrap_or(plan.settings.retries);
    let mut outcome = first;
    let mut attempts = 1;
    while matches!(outcome.status, Status::Fail | Status::Timeout) && attempts <= retries {
        attempts += 1;
        let retry = again();
        if retry.status == Status::Pass {
            outcome.status = Status::Flaky;
            break;
        }
        outcome = retry;
    }
    outcome.attempts = attempts;
    outcome
}

fn timeout_of(plan: &Plan, options: &TestOptions) -> Option<Duration> {
    let seconds = options.timeout.unwrap_or(plan.settings.timeout);
    (seconds > 0).then(|| Duration::from_secs(seconds))
}

/// A fresh folder for a process's tests' temporary folders.
fn temp_base(plan: &Plan, count: usize) -> PathBuf {
    let n = plan.temp_counter.fetch_add(1, Ordering::Relaxed);
    let base = plan.work.join("temp").join(n.to_string());
    for i in 0..count {
        let _ = std::fs::create_dir_all(base.join(i.to_string()));
    }
    base
}

/// Runs unit tests of one root, starting processes until every one of them ran.
fn run_unit(plan: &Plan, cases: &[usize], live: &mut dyn FnMut(Live)) -> Vec<Outcome> {
    let root = plan.cases[cases[0]].root;
    let built = plan.built[root]
        .as_ref()
        .expect("a root with unit tests is built");
    let cwd = &plan.roots[root].dir;
    let mut outcomes = Vec::new();
    let mut remaining: Vec<usize> = cases.to_vec();
    while !remaining.is_empty() {
        let names: Vec<String> = remaining
            .iter()
            .map(|&i| plan.cases[i].run_as.clone())
            .collect();
        let temp = temp_base(plan, names.len());
        let mut invocation = build::invocation(
            built,
            &plan.root_flags[root],
            &plan.toolchain,
            &names,
            &temp,
            cwd,
            &build::RunFlags {
                seed: plan.settings.seed,
                accept: plan.settings.accept,
            },
        );
        let run_dir = plan.coverage.as_ref().map(coverage::Coverage::run_dir);
        if let Some(dir) = &run_dir {
            invocation.env.push((
                "JAIC_COVERAGE_DIR".into(),
                dir.to_string_lossy().into_owned(),
            ));
        }
        let timeout = |name: &str| {
            remaining
                .iter()
                .find(|&&i| plan.cases[i].local == name)
                .and_then(|&i| timeout_of(plan, &plan.cases[i].options))
        };
        let limits = exec::Limits {
            timeout: &timeout,
            slow: (plan.settings.slow > 0).then(|| Duration::from_secs(plan.settings.slow)),
            stream: !plan.settings.capture,
        };
        let run_flags = build::RunFlags {
            seed: plan.settings.seed,
            accept: plan.settings.accept,
        };
        // The first run of a root under `--interp` is the process that found its tests.
        let warm = if run_dir.is_none() {
            build::warm_start(built, &names, &temp, &run_flags)
        } else {
            None
        };
        let run = match warm {
            Some(process) => exec::run_child(Ok(process), &invocation, &limits, live),
            None => exec::run(&invocation, &limits, live),
        };
        if let (Some(coverage), Some(dir)) = (&plan.coverage, &run_dir) {
            coverage.collect(dir);
        }
        let mut done = 0;
        for (n, raw) in run.tests.iter().enumerate() {
            let Some(position) = remaining
                .iter()
                .position(|&i| plan.cases[i].local == raw.name)
            else {
                continue;
            };
            let case = remaining[position];
            let last = n + 1 == run.tests.len();
            let ending = (last && raw.end.is_none()).then_some(&run.ending);
            let mut outcome = resolve(&plan.cases[case], case, raw, ending);
            let folder = temp.join(position.to_string());
            if keep_folder(plan, outcome.status, &folder) {
                outcome.kept_temp = Some(folder);
            } else {
                let _ = std::fs::remove_dir_all(&folder);
            }
            outcomes.push(outcome);
            done = done.max(position + 1);
        }
        for name in &run.missing {
            if let Some(&case) = remaining.iter().find(|&&i| plan.cases[i].run_as == *name) {
                let mut outcome = Outcome::new(case, Status::Fail);
                outcome
                    .problems
                    .push(format!("the test program has no test `{name}`"));
                outcomes.push(outcome);
                done = done.max(remaining.iter().position(|&i| i == case).unwrap_or(0) + 1);
            }
        }
        if done == 0 {
            // The process stopped before its first test: every one left fails with it.
            let why = match &run.ending {
                Ending::NotStarted(why) => why.clone(),
                ending => format!(
                    "the test program {} before any test started",
                    ending_text(ending)
                ),
            };
            for &case in &remaining {
                let mut outcome = Outcome::new(case, Status::Fail);
                outcome.problems.push(why.clone());
                outcome.stdout.clone_from(&run.preamble_stdout);
                outcome.stderr.clone_from(&run.preamble);
                outcomes.push(outcome);
            }
            break;
        }
        let _ = std::fs::remove_dir(&temp);
        remaining.drain(..done);
        remaining.retain(|i| !outcomes.iter().any(|o| o.case == *i));
    }
    outcomes
}

fn keep_folder(plan: &Plan, status: Status, folder: &Path) -> bool {
    let used = std::fs::read_dir(folder).is_ok_and(|mut d| d.next().is_some());
    used && (plan.settings.keep_temp || status.fails(plan.settings.flaky_fails))
}

pub fn ending_text(ending: &Ending) -> String {
    match ending {
        Ending::Exited(code) => format!("exited with status {code}"),
        Ending::Signal(signal) => format!("was killed by signal {signal}{}", signal_name(*signal)),
        Ending::TimedOut => "timed out".into(),
        Ending::NotStarted(why) => why.clone(),
    }
}

fn signal_name(signal: i32) -> &'static str {
    match signal {
        4 => " (SIGILL)",
        5 => " (SIGTRAP)",
        6 => " (SIGABRT)",
        8 => " (SIGFPE)",
        9 => " (SIGKILL)",
        10 => " (SIGBUS)",
        11 => " (SIGSEGV)",
        _ => "",
    }
}

/// A unit test's outcome from what its process reported and, when the process stopped in it,
/// how it ended.
fn resolve(case: &Case, index: usize, raw: &exec::Raw, ending: Option<&Ending>) -> Outcome {
    let mut outcome = Outcome::new(index, Status::Pass);
    outcome.duration = raw.duration;
    outcome.failures.clone_from(&raw.failures);
    outcome.stdout.clone_from(&raw.stdout);
    outcome.stderr.clone_from(&raw.stderr);
    outcome.subtests.clone_from(&raw.subtests);
    outcome.skip_reason.clone_from(&raw.skip_reason);
    let want_exit = case.options.exit;
    outcome.status = match (raw.end.as_deref(), ending) {
        (Some("skip"), _) => Status::Skip,
        (Some("fail"), _) => Status::Fail,
        (Some(_), _) if want_exit.is_some() => {
            outcome.problems.push(format!(
                "the test returned; @exit wants the program to exit with status {}",
                want_exit.unwrap_or(0)
            ));
            Status::Fail
        }
        (Some(_), _) => Status::Pass,
        (None, Some(Ending::TimedOut)) => {
            outcome.problems.push(match case.options.timeout {
                Some(seconds) => format!("it ran past its @timeout({seconds})"),
                None => "it ran past the timeout (`--timeout`, `[test] timeout`)".into(),
            });
            Status::Timeout
        }
        (None, Some(Ending::Exited(code)))
            if Some(*code) == want_exit && raw.failures.is_empty() =>
        {
            Status::Pass
        }
        (None, Some(ending)) => {
            outcome.problems.push(format!(
                "the test program {} during the test",
                ending_text(ending)
            ));
            Status::Fail
        }
        (None, None) => {
            outcome.problems.push("the test did not finish".into());
            Status::Fail
        }
    };
    expectations(case, &mut outcome);
    outcome
}

/// `@should_fail` and `@xfail` turn the outcome around.
fn expectations(case: &Case, outcome: &mut Outcome) {
    if let Some(text) = &case.options.should_fail {
        outcome.status = match outcome.status {
            Status::Fail | Status::Timeout => {
                let found = text.is_empty()
                    || outcome
                        .failures
                        .iter()
                        .any(|f| f.message.contains(text.as_str()))
                    || outcome.problems.iter().any(|p| p.contains(text.as_str()))
                    || String::from_utf8_lossy(&outcome.stderr).contains(text.as_str());
                if found {
                    Status::Pass
                } else {
                    outcome.problems.push(format!(
                        "it failed, but no failure message or standard error contains `{text}`"
                    ));
                    Status::Fail
                }
            }
            Status::Pass => {
                outcome
                    .problems
                    .push("it passed; @should_fail expects it to fail".into());
                Status::Fail
            }
            other => other,
        };
    } else if let Some(reason) = &case.options.xfail {
        outcome.status = match outcome.status {
            Status::Fail | Status::Timeout => Status::XFail,
            Status::Pass => {
                let why = if reason.is_empty() {
                    String::new()
                } else {
                    format!(" ({reason})")
                };
                outcome
                    .problems
                    .push(format!("it passed; remove its @xfail{why}"));
                Status::XPass
            }
            other => other,
        };
    }
}

/// Runs a golden or compile-fail case.
fn run_file_case(plan: &Plan, index: usize) -> Outcome {
    let case = &plan.cases[index];
    let root = &plan.roots[case.root];
    let started = Instant::now();
    // A program built to run once reports what it ran in a folder of its own.
    let run_dir = plan.coverage.as_ref().map(coverage::Coverage::run_dir);
    let toolchain = Toolchain {
        coverage_dir: run_dir.clone(),
        ..plan.toolchain.clone()
    };
    let checked = if let Some(doc) = case.doc {
        let work = plan.work.join(format!("doc-{index}"));
        let _ = std::fs::create_dir_all(&work);
        let mut quiet = |_: Live| {};
        doctest::check(
            &root.docs[doc],
            &root.module,
            &plan.root_flags[case.root],
            &toolchain,
            &work,
            &root.dir,
            timeout_of(plan, &case.options),
            &mut quiet,
        )
    } else if case.kind == Kind::Golden {
        let work = plan.work.join(format!("golden-{index}"));
        let _ = std::fs::create_dir_all(&work);
        let mut quiet = |_: Live| {};
        golden::golden(
            &case.file,
            &case.options,
            &plan.root_flags[case.root],
            &toolchain,
            &work,
            case.file.parent().unwrap_or(&root.dir),
            timeout_of(plan, &case.options),
            plan.settings.accept,
            &mut quiet,
        )
    } else {
        golden::compile_fail(
            &case.file,
            &case.options,
            &plan.root_flags[case.root],
            &plan.toolchain,
            &plan.stdlib,
            plan.settings.accept,
        )
    };
    if let (Some(coverage), Some(dir)) = (&plan.coverage, &run_dir) {
        coverage.collect_once(dir);
        // The program and its debug info are large and have been read.
        for kind in ["doc", "golden"] {
            let _ = std::fs::remove_dir_all(plan.work.join(format!("{kind}-{index}")));
        }
    }
    let status = if checked.timed_out {
        Status::Timeout
    } else if checked.passed {
        Status::Pass
    } else {
        Status::Fail
    };
    let mut outcome = Outcome::new(index, status);
    outcome.duration = started.elapsed();
    outcome.problems = checked.problems;
    outcome.stdout = checked.stdout;
    outcome.stderr = checked.stderr;
    expectations(case, &mut outcome);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::tests::case;
    use exec::Failure;

    fn raw(end: Option<&str>) -> exec::Raw {
        exec::Raw {
            name: "t".into(),
            end: end.map(String::from),
            ..exec::Raw::default()
        }
    }

    #[test]
    fn exits_crashes_and_expectations_decide_the_status() {
        let mut plain = case("t", Kind::Test);
        assert_eq!(
            resolve(&plain, 0, &raw(Some("pass")), None).status,
            Status::Pass
        );
        let crashed = resolve(&plain, 0, &raw(None), Some(&Ending::Signal(11)));
        assert_eq!(crashed.status, Status::Fail);
        assert!(crashed.problems[0].contains("SIGSEGV"));
        assert_eq!(
            resolve(&plain, 0, &raw(None), Some(&Ending::TimedOut)).status,
            Status::Timeout
        );

        plain.options.exit = Some(3);
        assert_eq!(
            resolve(&plain, 0, &raw(None), Some(&Ending::Exited(3))).status,
            Status::Pass
        );
        assert_eq!(
            resolve(&plain, 0, &raw(None), Some(&Ending::Exited(1))).status,
            Status::Fail
        );
        assert_eq!(
            resolve(&plain, 0, &raw(Some("pass")), None).status,
            Status::Fail
        );

        let mut should = case("t", Kind::Test);
        should.options.should_fail = Some("bounds".into());
        let mut failing = raw(Some("fail"));
        failing.failures.push(Failure {
            test: "t".into(),
            file: "/t.jai".into(),
            line: 3,
            message: "index out of bounds".into(),
        });
        assert_eq!(resolve(&should, 0, &failing, None).status, Status::Pass);
        assert_eq!(
            resolve(&should, 0, &raw(Some("pass")), None).status,
            Status::Fail
        );
        should.options.should_fail = Some("other".into());
        assert_eq!(resolve(&should, 0, &failing, None).status, Status::Fail);

        let mut known = case("t", Kind::Test);
        known.options.xfail = Some(String::new());
        assert_eq!(resolve(&known, 0, &failing, None).status, Status::XFail);
        assert_eq!(
            resolve(&known, 0, &raw(Some("pass")), None).status,
            Status::XPass
        );
    }
}
