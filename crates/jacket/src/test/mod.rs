//! `jacket test`: builds a project's tests once, then runs each in its own process, in parallel.
//! See docs/tools/jacket-test.md.
//!
//! The pieces: [`discover`] finds the test roots and their files, [`build`] compiles a root's
//! unit tests with the Testing module's Discovery plugin, [`exec`] runs a test program and reads
//! its records, [`golden`] checks golden-output and compile-fail tests, and [`report`] prints
//! results as text, JSON lines or `JUnit` XML. This file plans the run and schedules it.
mod build;
mod diff;
mod discover;
mod doctest;
mod exec;
mod golden;
mod jobs;
mod options;
mod report;

use crate::cli::TestArgs;
use crate::{project, tools};
use build::{Built, Toolchain};
use exec::{Failure, Subtest};
use options::{Target, TestOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT: u64 = 120;
const DEFAULT_SLOW: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Test,
    Golden,
    CompileFail,
    /// An example in a doc comment.
    Doc,
    Bench,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Test => "test",
            Kind::Golden => "golden",
            Kind::CompileFail => "compile_fail",
            Kind::Doc => "doc",
            Kind::Bench => "bench",
        }
    }
}

/// One test of the run.
#[derive(Clone, Debug)]
pub struct Case {
    /// Its name in output and filters: `name`, `Root::name` when several roots run.
    pub name: String,
    /// Its name in its program (unit tests), or the file's stem.
    pub local: String,
    pub kind: Kind,
    pub root: usize,
    pub file: PathBuf,
    pub line: u32,
    pub notes: Vec<String>,
    pub options: TestOptions,
    /// What `--run` is given: `local`, or `local/subtest` when a filter names a subtest.
    pub run_as: String,
    /// A doc test's block, in its root's `docs`.
    pub doc: Option<usize>,
}

impl Case {
    /// Whether it only compiles, so it runs on any target (compile-fail tests, `no_run` and
    /// `compile_fail` doc tests).
    fn compiles_only(&self, roots: &[discover::Root]) -> bool {
        match self.doc {
            Some(doc) => !matches!(
                roots[self.root].docs[doc].mode,
                doctest::Mode::Run | doctest::Mode::ShouldFail
            ),
            None => self.kind == Kind::CompileFail,
        }
    }
}

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
    /// The test's temporary folder, kept because it failed.
    pub kept_temp: Option<PathBuf>,
}

impl Outcome {
    fn new(case: usize, status: Status) -> Outcome {
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
            kept_temp: None,
        }
    }
}

/// The settings of a run, after jacket.toml, the profile and the flags.
struct Settings {
    jobs: usize,
    timeout: u64,
    slow: u64,
    retries: u32,
    fail_fast: bool,
    flaky_fails: bool,
    needs: Vec<String>,
    keep_temp: bool,
    accept: bool,
    isolate: bool,
    capture: bool,
}

/// Everything the workers share.
struct Plan {
    cases: Vec<Case>,
    roots: Vec<discover::Root>,
    root_flags: Vec<Vec<String>>,
    built: Vec<Option<Built>>,
    toolchain: Toolchain,
    settings: Settings,
    work: PathBuf,
    stdlib: PathBuf,
    temp_counter: AtomicUsize,
    stop: AtomicBool,
}

/// What workers tell the reporter.
pub enum Message {
    Started(usize),
    Slow(usize, Duration),
    Done(Outcome),
}

fn fail(message: impl std::fmt::Display) -> ExitCode {
    eprintln!("jacket: error: {message}");
    ExitCode::from(2)
}

fn host_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        _ => "linux",
    }
}

fn host_cpu() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        _ => "x64",
    }
}

/// The standard library jaic uses, to write it `$STDLIB` in compile-fail output: as jaic finds
/// it (`JAIC_STDLIB`, else beside the compiler, else this repository's).
fn stdlib_dir(jaic: &Path) -> PathBuf {
    if let Some(dir) = std::env::var_os("JAIC_STDLIB") {
        return dir.into();
    }
    let real = jaic.canonicalize().ok();
    [Some(jaic.to_path_buf()), real]
        .into_iter()
        .flatten()
        .filter_map(|exe| Some(exe.parent()?.join("stdlib")))
        .find(|dir| dir.is_dir())
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"))
}

/// `hash:K/N` or `count:K/N`: whether the `index`-th test named `name` is in this part.
fn partition(spec: &str) -> Result<impl Fn(usize, &str) -> bool + use<>, String> {
    let bad = || format!("`--partition {spec}` is not `hash:K/N` or `count:K/N` with 1 <= K <= N");
    let (how, part) = spec.split_once(':').ok_or_else(bad)?;
    let (k, n) = part.split_once('/').ok_or_else(bad)?;
    let (k, n): (u64, u64) = (k.parse().map_err(|_| bad())?, n.parse().map_err(|_| bad())?);
    if k == 0 || k > n || !matches!(how, "hash" | "count") {
        return Err(bad());
    }
    let by_hash = how == "hash";
    Ok(move |index: usize, name: &str| {
        let key = if by_hash {
            // FNV-1a: stable across runs and machines.
            name.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
            })
        } else {
            index as u64
        };
        key % n == k - 1
    })
}

/// The filter a case matches, and what to run it as: `None` when no filter selects it.
fn select(case: &Case, filters: &[String], exact: bool) -> Option<String> {
    if filters.is_empty() {
        return Some(case.local.clone());
    }
    let matches = |pattern: &str, name: &str| {
        if exact {
            name == pattern
        } else {
            name.contains(pattern)
        }
    };
    if filters.iter().any(|f| matches(f, &case.name)) {
        return Some(case.local.clone());
    }
    if case.kind != Kind::Test {
        return None;
    }
    // `test/subtest`: the test, asked to run that subtest only.
    filters.iter().find_map(|f| {
        let (top, sub) = f.split_once('/')?;
        matches(top, &case.name).then(|| format!("{}/{sub}", case.local))
    })
}

pub fn run(args: &TestArgs, cwd: &Path, bench: bool) -> ExitCode {
    let loaded = match project::load(cwd) {
        Ok(loaded) => loaded,
        Err(error) => return fail(error),
    };
    let config = loaded.as_ref().map(|l| &l.config.test);
    let config_dir = loaded.as_ref().map(jacket_config::Loaded::dir);
    let (settings, runner) = match settings(args, config, cwd, config_dir) {
        Ok(found) => found,
        Err(error) => return fail(error),
    };
    let os = args.os.clone().unwrap_or_else(|| host_os().into());
    let cpu = args.cpu.clone().unwrap_or_else(|| host_cpu().into());
    let target = Target {
        os: os.clone(),
        cpu: cpu.clone(),
        interp: args.interp,
    };
    let mut flags = Vec::new();
    if let Some(os) = &args.os {
        flags.extend(["-os".to_string(), os.clone()]);
    }
    if let Some(cpu) = &args.cpu {
        flags.extend(["-cpu".to_string(), cpu.clone()]);
    }
    flags.extend(args.jaic_args.iter().cloned());
    let jaic = tools::find(tools::JAIC);
    let stdlib = stdlib_dir(&jaic);
    let toolchain = Toolchain {
        jaic: jaic.clone(),
        flags,
        interp: args.interp,
        runner,
        os: os.clone(),
    };
    let cross = !args.interp && (os != host_os() || cpu != host_cpu());
    let cross_skip = (cross && toolchain.runner.is_empty())
        .then(|| format!("built for {os} {cpu}; `--runner <command>` runs it"));

    let project_dir = loaded.as_ref().map_or(cwd, |l| l.dir()).to_path_buf();
    let configured: Vec<PathBuf> = config
        .map(|c| c.paths.0.iter().map(|p| project_dir.join(p)).collect())
        .unwrap_or_default();
    let roots: Vec<discover::Root> = discover::roots(&args.path, &configured, &project_dir, cwd)
        .into_iter()
        .map(|dir| discover::Root::read(dir, &stdlib))
        .filter(|r| !r.is_empty())
        .collect();
    if roots.is_empty() {
        return fail(format!(
            "no tests found in {}\nhelp: tests are `.jai` files in a `tests/` folder; `[test] paths` in jacket.toml lists other folders",
            project_dir.display()
        ));
    }
    let work = std::env::temp_dir().join(format!("jacket-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let root_flags: Vec<Vec<String>> = roots
        .iter()
        .map(|r| build::root_flags(&r.dir, r.library))
        .collect();
    let list = args.list || bench;
    let built = build_all(
        &roots,
        &root_flags,
        &work,
        &toolchain,
        list,
        settings.jobs.min(4),
    );
    let mut broken = false;
    let built: Vec<Option<Built>> = built
        .into_iter()
        .zip(&roots)
        .map(|(result, root)| match result {
            Ok(built) => built,
            Err(output) => {
                eprintln!(
                    "jacket: error: the tests in {} do not compile\n{output}",
                    root.dir.join("tests").display()
                );
                broken = true;
                None
            }
        })
        .collect();
    if broken {
        let _ = std::fs::remove_dir_all(&work);
        return ExitCode::from(2);
    }

    let cases = cases(&roots, &built);
    let in_part = match args.partition.as_deref().map(partition).transpose() {
        Ok(p) => p,
        Err(error) => return fail(error),
    };
    let mut selected = Vec::new();
    for case in cases {
        let is_bench = case.kind == Kind::Bench;
        if is_bench != bench && (bench || !args.list) {
            continue;
        }
        let is_doc = case.kind == Kind::Doc;
        if (args.doc && !is_doc) || (args.no_doc && is_doc) {
            continue;
        }
        if let Some(run_as) = select(&case, &args.filters, args.exact) {
            selected.push(Case {
                run_as,
                ..case
            });
        }
    }
    if let Some(in_part) = &in_part {
        selected = selected
            .into_iter()
            .enumerate()
            .filter(|(i, c)| in_part(*i, &c.name))
            .map(|(_, c)| c)
            .collect();
    }
    let mut reporter = report::Reporter::new(
        args.format,
        args.junit.clone(),
        cwd,
        args.interp,
        settings.capture,
    );
    if list {
        reporter.list(&selected);
        let _ = std::fs::remove_dir_all(&work);
        if bench && !args.list {
            eprintln!(
                "jacket: `jacket bench` only lists benchmarks for now (`--list`); running them is not supported yet"
            );
            return ExitCode::from(2);
        }
        return ExitCode::SUCCESS;
    }
    if selected.is_empty() {
        let _ = std::fs::remove_dir_all(&work);
        return fail("no tests match");
    }

    let plan = Plan {
        cases: selected,
        roots,
        root_flags,
        built,
        stdlib,
        toolchain,
        settings,
        work,
        temp_counter: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
    };
    execute(&plan, &target, cross_skip.as_deref(), &mut reporter)
}

/// The run's settings from the flags over `[test]` (and the chosen profile), and the runner.
fn settings(
    args: &TestArgs,
    config: Option<&jacket_config::Test>,
    cwd: &Path,
    config_dir: Option<&Path>,
) -> Result<(Settings, Vec<String>), String> {
    let mut run_settings = config.map(jacket_config::Test::run).unwrap_or_default();
    if let Some(name) = &args.profile {
        match config.and_then(|c| c.profile.get(name)) {
            Some(profile) => run_settings = run_settings.overlaid(profile),
            None => return Err(format!("no `[test.profile.{name}]` in jacket.toml")),
        }
    }
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let capture = !args.no_capture;
    let settings = Settings {
        jobs: if capture {
            args.jobs.or(run_settings.jobs).unwrap_or(threads).max(1)
        } else {
            1
        },
        timeout: args
            .timeout
            .or(run_settings.timeout)
            .unwrap_or(DEFAULT_TIMEOUT),
        slow: run_settings.slow.unwrap_or(DEFAULT_SLOW),
        retries: args.retries.or(run_settings.retries).unwrap_or(0),
        fail_fast: if args.no_fail_fast {
            false
        } else {
            args.fail_fast || run_settings.fail_fast.unwrap_or(false)
        },
        flaky_fails: run_settings.flaky_fails.unwrap_or(false),
        needs: if args.needs.is_empty() {
            run_settings.needs.clone()
        } else {
            args.needs.clone()
        },
        keep_temp: args.keep_temp,
        accept: args.accept,
        isolate: !args.no_isolation,
        capture,
    };
    // Tests run in their own folders, so a file the runner names is resolved where it was
    // written: `--runner` from here, `[test] runner` beside jacket.toml.
    let runner = match (&args.runner, run_settings.runner) {
        (Some(runner), _) => runner_words(runner, cwd),
        (None, Some(runner)) => runner_words(&runner, config_dir.unwrap_or(cwd)),
        (None, None) => Vec::new(),
    };
    Ok((settings, runner))
}

/// The words of a runner command, with each that names a file under `base` made absolute.
fn runner_words(runner: &str, base: &Path) -> Vec<String> {
    runner
        .split_whitespace()
        .map(|word| {
            let path = base.join(word);
            if Path::new(word).is_relative() && path.is_file() {
                path.to_string_lossy().into_owned()
            } else {
                word.to_string()
            }
        })
        .collect()
}

/// Every test of the roots: their unit tests, golden tests and compile-fail tests. Problems
/// with notes are printed as warnings.
fn cases(roots: &[discover::Root], built: &[Option<Built>]) -> Vec<Case> {
    let several = roots.len() > 1;
    let mut cases = Vec::new();
    let mut headers: BTreeMap<PathBuf, TestOptions> = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut header_of = |file: &Path, warnings: &mut Vec<String>| -> TestOptions {
        headers
            .entry(file.to_path_buf())
            .or_insert_with(|| {
                let notes =
                    options::header_notes(&std::fs::read_to_string(file).unwrap_or_default());
                let (options, problems) = TestOptions::default().with_notes(&notes);
                warnings.extend(
                    problems
                        .into_iter()
                        .map(|p| format!("{}: {p}", file.display())),
                );
                options
            })
            .clone()
    };
    for (index, root) in roots.iter().enumerate() {
        let name = |local: &str| {
            if several {
                format!("{}::{local}", root.label)
            } else {
                local.to_string()
            }
        };
        if let Some(built) = &built[index] {
            for entry in &built.entries {
                let header = header_of(&entry.file, &mut warnings);
                let (options, problems) = header.with_notes(&entry.notes);
                warnings.extend(
                    problems
                        .into_iter()
                        .map(|p| format!("{}:{}: {p}", entry.file.display(), entry.line)),
                );
                cases.push(Case {
                    name: name(&entry.name),
                    local: entry.name.clone(),
                    kind: if entry.kind == "bench" {
                        Kind::Bench
                    } else {
                        Kind::Test
                    },
                    root: index,
                    file: entry.file.clone(),
                    line: entry.line,
                    notes: entry.notes.clone(),
                    options,
                    run_as: entry.name.clone(),
                    doc: None,
                });
            }
        }
        for (files, kind, prefix) in [
            (&root.golden, Kind::Golden, ""),
            (&root.compile_fail, Kind::CompileFail, "compile_fail/"),
        ] {
            for file in files {
                let stem = file
                    .file_stem()
                    .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
                let local = format!("{prefix}{stem}");
                cases.push(Case {
                    name: name(&local),
                    local: local.clone(),
                    kind,
                    root: index,
                    file: file.clone(),
                    line: 1,
                    notes: options::header_notes(
                        &std::fs::read_to_string(file).unwrap_or_default(),
                    ),
                    options: header_of(file, &mut warnings),
                    run_as: local,
                    doc: None,
                });
            }
        }
        // Doc tests are named for their module, so they need no root prefix.
        for (doc, block) in root.docs.iter().enumerate() {
            for word in &block.unknown {
                warnings.push(format!(
                    "{}:{}: unknown doc test option `{word}` (ignored)",
                    block.file.display(),
                    block.line
                ));
            }
            let local = format!("doc: {}", block.name);
            cases.push(Case {
                name: local.clone(),
                local: local.clone(),
                kind: Kind::Doc,
                root: index,
                file: block.file.clone(),
                line: block.line,
                notes: Vec::new(),
                options: TestOptions {
                    ignore: block.ignore,
                    ..TestOptions::default()
                },
                run_as: local,
                doc: Some(doc),
            });
        }
    }
    for warning in &warnings {
        eprintln!("jacket: warning: {warning}");
    }
    cases
}

/// Runs the plan's tests, reports them, and gives the exit status.
fn execute(
    plan: &Plan,
    target: &Target,
    cross_skip: Option<&str>,
    reporter: &mut report::Reporter,
) -> ExitCode {
    let started = Instant::now();
    reporter.run_started(plan.cases.len());
    let mut outcomes = Vec::new();
    // Skips are decided before anything runs.
    let mut to_run = Vec::new();
    for (index, case) in plan.cases.iter().enumerate() {
        let reason = case
            .options
            .skip_reason(target, &plan.settings.needs)
            .or_else(|| {
                (!case.compiles_only(&plan.roots))
                    .then(|| cross_skip.map(String::from))
                    .flatten()
            });
        match reason {
            Some(reason) => {
                let mut outcome = Outcome::new(index, Status::Skip);
                outcome.skip_reason = Some(reason);
                reporter.finished(&plan.cases[index], &outcome);
                outcomes.push(outcome);
            }
            None => to_run.push(index),
        }
    }
    let (parallel, serial) = jobs::jobs(plan, &to_run);
    for (jobs, threads) in [(parallel, plan.settings.jobs), (serial, 1)] {
        jobs::run_jobs(plan, jobs, threads, &mut |message| match message {
            Message::Started(case) => reporter.started(&plan.cases[case]),
            Message::Slow(case, elapsed) => reporter.slow(&plan.cases[case], elapsed),
            Message::Done(outcome) => {
                reporter.finished(&plan.cases[outcome.case], &outcome);
                outcomes.push(outcome);
            }
        });
    }
    let not_run = plan.cases.len() - outcomes.len();
    let failed = outcomes
        .iter()
        .any(|o| o.status.fails(plan.settings.flaky_fails));
    reporter.summary(
        &plan.cases,
        &outcomes,
        not_run,
        started.elapsed(),
        plan.settings.flaky_fails,
    );
    if outcomes.iter().all(|o| o.kept_temp.is_none()) {
        let _ = std::fs::remove_dir_all(&plan.work);
    } else {
        // Keep the failed tests' folders; the builds can go.
        for entry in std::fs::read_dir(&plan.work)
            .into_iter()
            .flatten()
            .flatten()
        {
            if entry.file_name() != "temp" {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// A root's build: nothing to build, the built program, or the compile errors.
type BuildResult = Result<Option<Built>, String>;

/// Builds every root's unit tests, `parallel` at a time.
fn build_all(
    roots: &[discover::Root],
    root_flags: &[Vec<String>],
    work: &Path,
    toolchain: &Toolchain,
    list: bool,
    parallel: usize,
) -> Vec<BuildResult> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<BuildResult>>> =
        Mutex::new((0..roots.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..parallel.max(1).min(roots.len()) {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(root) = roots.get(i) else {
                        break;
                    };
                    let result = if root.unit.is_empty() {
                        Ok(None)
                    } else {
                        build::unit(
                            &root.unit,
                            &root_flags[i],
                            &work.join(format!("build-{i}")),
                            toolchain,
                            list,
                        )
                        .map(Some)
                    };
                    results.lock().expect("lock")[i] = Some(result);
                }
            });
        }
    });
    results
        .into_inner()
        .expect("lock")
        .into_iter()
        .map(|r| r.unwrap_or(Ok(None)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn case(name: &str, kind: Kind) -> Case {
        Case {
            name: name.into(),
            local: name.rsplit("::").next().unwrap_or(name).into(),
            kind,
            root: 0,
            file: PathBuf::from("/t.jai"),
            line: 1,
            notes: Vec::new(),
            options: TestOptions::default(),
            run_as: String::new(),
            doc: None,
        }
    }

    #[test]
    fn filters_select_tests_and_subtests() {
        let table = case("A::table", Kind::Test);
        let none: [String; 0] = [];
        assert_eq!(select(&table, &none, false).as_deref(), Some("table"));
        assert_eq!(
            select(&table, &["tab".into()], false).as_deref(),
            Some("table")
        );
        assert_eq!(select(&table, &["tab".into()], true), None);
        assert_eq!(
            select(&table, &["A::table/two".into()], true).as_deref(),
            Some("table/two")
        );
        assert_eq!(select(&table, &["other".into()], false), None);
        let golden = case("A::compile_fail/bad", Kind::CompileFail);
        assert!(select(&golden, &["compile_fail/bad".into()], false).is_some());
    }

    #[test]
    fn partitions_cover_every_test_once() {
        let names = ["a", "b", "c", "d", "e"];
        for how in ["hash", "count"] {
            let parts: Vec<_> = (1..=3)
                .map(|k| partition(&format!("{how}:{k}/3")).unwrap())
                .collect();
            for (i, name) in names.iter().enumerate() {
                assert_eq!(
                    parts.iter().filter(|p| p(i, name)).count(),
                    1,
                    "{how} {name}"
                );
            }
        }
        assert!(partition("hash:0/3").is_err());
        assert!(partition("count:4/3").is_err());
        assert!(partition("mod:1/2").is_err());
    }
}
