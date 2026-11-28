//! `jacket test`: builds a project's tests once, then runs each in its own process, in parallel.
//! See docs/tools/jacket-test.md.
//!
//! The pieces: [`discover`] finds the test roots and their files, [`build`] compiles a root's
//! unit tests with the Testing module's Discovery plugin, [`exec`] runs a test program and reads
//! its records, [`golden`] checks golden-output and compile-fail tests, and [`report`] prints
//! results as text, JSON lines or `JUnit` XML. This file plans the run and schedules it.
mod bench;
mod bench_stats;
mod build;
mod coverage;
mod coverage_plan;
mod deps;
mod deps_store;
mod diff;
mod discover;
pub(crate) mod doctest;
pub(crate) mod exec;
mod filter;
mod golden;
mod jobs;
mod outcome;
mod partition;
pub use outcome::{Outcome, Status};
pub(crate) mod options;
mod prerequisites;
mod report;
mod review;
pub(crate) mod stdlib;
mod watch;

use crate::cli::{BenchArgs, TestArgs};
use crate::{project, tools};
use build::{Built, Toolchain};
use coverage_plan::coverage_for;
use options::{RunMode, Target, TestOptions};

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize};
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
    /// The cases of a `@cases` test, known when only listing.
    pub cases: Vec<String>,
}

impl Case {
    /// Why a board skips it: it builds a host program to run (golden tests and doc examples that
    /// run), which a board cannot start. Unit tests, benchmarks and the cases that only compile
    /// need no host program.
    fn board_skip(&self, roots: &[discover::Root], check_only: bool) -> Option<&'static str> {
        (!matches!(self.kind, Kind::Test | Kind::Bench) && !self.compiles_only(roots, check_only))
            .then_some("it runs a host program; a board target runs unit tests only")
    }

    /// Whether it only compiles, so it runs on any target (compile-fail tests, `no_run` and
    /// `compile_fail` doc tests).
    fn compiles_only(&self, roots: &[discover::Root], check_only: bool) -> bool {
        match self.doc {
            Some(doc) => {
                check_only
                    || !matches!(
                        roots[self.root].docs[doc].mode,
                        doctest::Mode::Run | doctest::Mode::ShouldFail
                    )
            }
            None => check_only || self.kind == Kind::CompileFail,
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
    /// The seed of property tests: `--seed`, else drawn once for the run.
    seed: u64,
    isolate: bool,
    capture: bool,
    /// `--allow-missing-prerequisites`: a missing test-driver probe skips its tests.
    allow_missing_prerequisites: bool,
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
    /// The folder with `jacket.toml`, written `$ROOT` in compile-fail output.
    project: PathBuf,
    temp_counter: AtomicUsize,
    stop: AtomicBool,
    coverage: Option<coverage::Coverage>,
    /// The target is a board (`--target` names one): cases that need a host program are skipped.
    on_board: bool,
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

/// `args` with every path the user wrote made absolute from `cwd`: the one place where the
/// current directory is applied to them (`--path`, `--junit` and the paths among the jaic flags).
fn resolve_paths(args: &TestArgs, cwd: &Path) -> TestArgs {
    let mut args = args.clone();
    let outputs = args
        .junit
        .iter_mut()
        .chain(args.coverage_out.iter_mut())
        .chain(args.coverage_baseline.iter_mut())
        .chain(args.coverage_record.iter_mut())
        .chain(args.coverage_summary.iter_mut())
        .chain(args.partition_times.iter_mut());
    for path in args
        .path
        .iter_mut()
        .chain(args.coverage_include.iter_mut())
        .chain(args.changed.iter_mut())
        .chain(outputs)
    {
        *path = cwd.join(&*path);
    }
    crate::compile::absolute_flag_paths(&mut args.jaic_args, cwd);
    args
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

/// The entries of [`filter::KINDS`] a case has.
fn kinds_of(case: &Case, snapshot: bool) -> Vec<&'static str> {
    let has_note = |name: &str| case.notes.iter().any(|n| n.split('(').next() == Some(name));
    match case.kind {
        Kind::Test => {
            let mut kinds = Vec::new();
            let property = has_note("property");
            let compile_time = has_note("compile_time");
            if !property && !compile_time {
                kinds.push("unit");
            }
            if property {
                kinds.push("property");
            }
            if compile_time {
                kinds.push("compile_time");
            }
            if snapshot && uses_snapshots(&case.file, case.line) {
                kinds.push("snapshot");
            }
            kinds
        }
        Kind::Golden => vec!["golden"],
        Kind::CompileFail => vec!["compile_fail"],
        Kind::Doc => vec!["doc"],
        Kind::Bench => vec!["bench"],
    }
}

/// Whether the procedure that starts at `line` of `file` calls `expect_snapshot`: its text runs
/// to the first line that starts with `}`.
fn uses_snapshots(file: &Path, line: u32) -> bool {
    let source = std::fs::read_to_string(file).unwrap_or_default();
    source
        .lines()
        .skip(line.saturating_sub(1) as usize)
        .take_while(|l| !l.starts_with('}'))
        .any(|l| l.contains("expect_snapshot("))
}

/// Whether `expr` selects `case`.
fn expr_selects(expr: &filter::FilterExpr, case: &Case, root: &discover::Root, cwd: &Path) -> bool {
    let kinds = kinds_of(case, expr.uses_snapshot_kind());
    let files: Vec<String> = [&root.dir, cwd]
        .iter()
        .filter_map(|base| case.file.strip_prefix(base).ok())
        .map(jacket_config::slashed)
        .chain([jacket_config::slashed(&case.file)])
        .collect();
    expr.matches(&filter::Subject {
        names: [&case.name, &case.local],
        modules: [&root.module, &root.label],
        files: &files,
        kinds: &kinds,
    })
}

pub fn run(args: &TestArgs, cwd: &Path, bench: Option<&BenchArgs>) -> ExitCode {
    if !args.watch {
        return run_once(args, cwd, bench, &mut watch::Watched::default(), None);
    }
    if bench.is_some() {
        return fail("`jacket bench` has no --watch");
    }
    watch::run(args, cwd)
}

/// Whether the flags select a case of `kind`: benchmarks for `jacket bench`, and doc examples
/// for `--doc` (never for `--no-doc`).
fn kind_selected(kind: Kind, args: &TestArgs, bench: bool) -> bool {
    let is_bench = kind == Kind::Bench;
    if is_bench != bench && (bench || !args.list) {
        return false;
    }
    let is_doc = kind == Kind::Doc;
    !((args.doc && !is_doc) || (args.no_doc && is_doc))
}

/// Moves the `-interp` flag among the arguments for jaic into `args`, and rejects the modes that
/// do not run tests (`-check`, `-deps`) or that conflict.
fn take_forwarded_modes(args: &mut TestArgs) -> Result<(), String> {
    let forwarded = jaic_cli_args::arguments::target_flags(&args.jaic_args)?;
    jaic_cli_args::cli::Mode::from_flags(forwarded.check, forwarded.interp, forwarded.deps)
        .map_err(|error| error.message)?;
    if forwarded.deps {
        return Err("use `jacket test --list --deps` to list test dependencies".into());
    }
    if forwarded.check && !args.check_only {
        return Err(
            "`-check` does not run tests; use `--check-only` to type-check tests and doc examples"
                .into(),
        );
    }
    args.interp |= forwarded.interp;
    args.jaic_args = forwarded.without_modes(std::mem::take(&mut args.jaic_args));
    Ok(())
}

/// Builds the test-driver probe libraries `files` need and adds their folder to the library
/// search path of every compile `toolchain` makes.
fn link_probes(
    toolchain: &mut Toolchain,
    files: &[PathBuf],
    stdlib: &Path,
    allow_missing: bool,
) -> Result<(), String> {
    if let Some(probes) = prerequisites::prepare_probes(files, stdlib, allow_missing)? {
        let path = serde_json::to_string(&probes.to_string_lossy()).expect("serialize path");
        toolchain.flags.extend(["-add".into(), format!("#run {{ C :: #import \"Compiler\"; C.compiler_add_library_search_directory({path}); }}")]);
        toolchain.environment_flags = 2;
    }
    Ok(())
}

/// One run of the selected tests. `watched` is told which folders and test files the run
/// used; `only` narrows the run to the cases of these test files.
fn run_once(
    args: &TestArgs,
    cwd: &Path,
    bench_args: Option<&BenchArgs>,
    watched: &mut watch::Watched,
    only: Option<&std::collections::BTreeSet<PathBuf>>,
) -> ExitCode {
    watched.clear();
    let bench = bench_args.is_some();
    let mut resolved = resolve_paths(args, cwd);
    if let Err(error) = take_forwarded_modes(&mut resolved) {
        return fail(error);
    }
    let args = &resolved;
    let loaded = match project::load(cwd) {
        Ok(loaded) => loaded,
        Err(error) => return fail(error),
    };
    let config = loaded.as_ref().map(|l| &l.config.test);
    let config_dir = loaded.as_ref().map(jacket_config::Loaded::dir);
    let (target_flags, target_runner) = match args
        .target
        .as_deref()
        .map(|name| crate::target::for_tests(name, loaded.as_ref().map(|l| &l.config)))
    {
        Some(Ok(found)) => found,
        Some(Err(error)) => return fail(error),
        None => (Vec::new(), None),
    };
    let (settings, runner) =
        match settings(args, config, cwd, config_dir).and_then(|(settings, runner)| {
            let runner = choose_runner(args, runner, target_runner, config_dir.unwrap_or(cwd));
            crate::prerequisite::runner(&runner, "jacket test").map(|()| (settings, runner))
        }) {
            Ok(found) => found,
            Err(error) => return fail(error),
        };
    let project_dir = loaded.as_ref().map_or(cwd, |l| l.dir()).to_path_buf();
    let ignore = loaded.as_ref().map_or_else(Vec::new, |l| {
        jacket_config::test_roots::ignore_patterns(&l.config)
    });
    let configured = match config.map_or(Ok(Vec::new()), |c| {
        discover::expand(&c.paths.0, &project_dir, &ignore)
    }) {
        Ok(found) => found,
        Err(error) => return fail(format!("in `[test] paths`, {error}")),
    };
    if args.roots {
        for root in discover::roots(&args.path, &configured, &project_dir, cwd) {
            println!("{}", root.display());
        }
        return ExitCode::SUCCESS;
    }
    if args.review {
        use std::io::IsTerminal;
        let roots = discover::roots(&args.path, &configured, &project_dir, cwd);
        return review::run(
            &roots,
            cwd,
            std::io::stdin().is_terminal(),
            &mut std::io::stdin().lock(),
            &mut std::io::stdout(),
        );
    }
    let filter_expr = match args.filter_expr.as_deref().map(filter::FilterExpr::parse) {
        Some(Ok(expr)) => Some(expr),
        Some(Err(error)) => return fail(error),
        None => None,
    };
    // On a board only the unit tests run; the cases that need a host program are skipped
    // (`Case::board_skip`).
    let on_board = target_flags.iter().any(|flag| flag.starts_with('+'));
    let mut flags = target_flags;
    if let Some(os) = &args.os {
        flags.extend(["-os".to_string(), os.clone()]);
    }
    if let Some(cpu) = &args.cpu {
        flags.extend(["-cpu".to_string(), cpu.clone()]);
    }
    // The project's import folders and `[experimental]` settings reach tests as they reach
    // `jacket build`, and the user's own flags come last.
    let user_flags: Vec<std::ffi::OsString> = args.jaic_args.iter().map(Into::into).collect();
    match crate::compile::project_flags(&user_flags, loaded.as_ref()) {
        Ok(project) => flags.extend(
            project
                .into_iter()
                .map(|flag| flag.to_string_lossy().into_owned()),
        ),
        Err(error) => return fail(error),
    }
    flags.extend(args.jaic_args.iter().cloned());
    let parsed = match jaic_cli_args::arguments::target_flags(&flags) {
        Ok(parsed) => parsed,
        Err(error) => return fail(error),
    };
    let mut effective = jaic::sema::Options::host();
    // Metadata is also needed by compile-only cases. The compiler checks native link
    // eligibility only when a case actually builds a program; valid modes share OS/CPU defaults.
    if let Err(error) = parsed.selection.apply_for_mode(&mut effective, false) {
        return fail(error);
    }
    let os = jaic::target::os_name(effective.os).to_string();
    let cpu = jaic::target::cpu_name(effective.cpu).to_string();
    let target = Target {
        os: os.clone(),
        cpu: cpu.clone(),
        mode: RunMode::of(args.check_only, args.interp),
    };

    if !args.interp && !bench && matches!(os.as_str(), "windows" | "linux" | "macos") {
        flags.insert(0, "-backtrace_on_crash".into()); // the only targets with a crash handler
    }
    if bench && !args.list && (args.interp || os == "wasm") {
        return fail("`jacket bench` measures native builds: drop `--interp` and `-os wasm`");
    }
    // Benchmarks are optimized, without runtime checks, unless a flag after `--` chooses.
    if bench && !args.list && !args.jaic_args.iter().any(|flag| sets_optimization(flag)) {
        flags.extend(["-o".to_string(), "-no_debug_info".to_string()]);
    }
    let Some(jaic) = tools::require(tools::JAIC, "jacket test") else {
        return ExitCode::from(127);
    };
    let stdlib = match stdlib::stdlib_dir(&jaic, &project_dir, cwd) {
        Ok(dir) => dir,
        Err(error) => return fail(error),
    };
    let mut doc_files = std::collections::BTreeSet::new();
    let deps = match deps_store::Store::open_shared(&jaic) {
        Ok(deps) => deps,
        Err(error) => return fail(error),
    };
    let mut toolchain = Toolchain {
        jaic: jaic.clone(),
        stdlib: stdlib.clone(),
        flags,
        interp: args.interp,
        runner,
        os: os.clone(),
        coverage: args.coverage,
        coverage_dir: None,
        check_only: args.check_only,
        deps,
        environment_flags: 0,
    };
    let cross = !args.interp && (os != host_os() || cpu != host_cpu());
    let cross_skip = (cross && toolchain.runner.is_empty())
        .then(|| format!("built for {os} {cpu}; `--runner <command>` runs it"));
    // A target named on the command line is one the user wants the tests run on: with no runner
    // for it, fail now (as `jacket run` does) instead of building and skipping every test.
    if let (Some(name), Some(_)) = (&args.target, &cross_skip)
        && !args.list
        && !args.check_only
    {
        return fail(format!(
            "`{name}` is not this machine's target ({os} {cpu}), so jacket cannot run the tests it builds\nhelp: name a runner with `--runner COMMAND` or `runner` under `[target.NAME]` in jacket.toml, or run with `--interp`"
        ));
    }

    let mut roots: Vec<discover::Root> = Vec::new();
    let import_dirs = jaic_cli_args::arguments::import_dirs(&toolchain.flags);
    for dir in discover::roots(&args.path, &configured, &project_dir, cwd) {
        match discover::Root::read(dir, &stdlib, &jaic, &import_dirs, &mut doc_files, !bench) {
            Ok(root) if !root.is_empty() => roots.push(root),
            Ok(_) => {}
            Err(error) => return fail(error),
        }
    }
    if roots.is_empty() {
        return fail(format!(
            "no tests found in {}\nhelp: tests are `.jai` files in a `tests/` folder; `[test] paths` in jacket.toml names other folders or patterns such as `stdlib/**`",
            project_dir.display()
        ));
    }
    let work = std::env::temp_dir().join(format!("jacket-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let root_flags: Vec<_> = roots
        .iter()
        .map(|r| build::root_flags(&r.dir, r.library))
        .collect();
    if args.watch {
        watched.set_folders(&roots, &toolchain.flags, &root_flags);
    }
    let list = args.list;
    let coverage = match coverage_for(args, cross, &roots, &toolchain, &work, &project_dir) {
        Ok(coverage) => coverage,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&work);
            return fail(error);
        }
    };
    if os == host_os() && cpu == host_cpu() && !bench && !args.check_only && !list {
        let files: Vec<_> = roots
            .iter()
            .flat_map(|root| root.unit.iter().chain(&root.golden))
            .cloned()
            .collect();
        if let Err(error) = link_probes(
            &mut toolchain,
            &files,
            &stdlib,
            args.allow_missing_prerequisites,
        ) {
            let _ = std::fs::remove_dir_all(&work);
            return fail(error);
        }
    }
    // Native compilation runs #run code and generators too. Their interpreter coverage belongs
    // in this report, alongside the blocks the built test programs execute later.
    let compile_coverage = coverage.as_ref().map(coverage::Coverage::run_dir);
    toolchain.coverage_dir.clone_from(&compile_coverage);
    let listing = if list && args.list_target {
        Some(build::Listing::Target)
    } else if list || args.check_only {
        Some(build::Listing::Host)
    } else {
        None
    };
    let discovery_toolchain = toolchain.for_discovery(listing);
    let built = build_all(
        &roots,
        &root_flags,
        &work,
        &discovery_toolchain,
        listing,
        bench,
        Workers {
            builds: settings.jobs.min(4),
            warm: settings.jobs,
        },
    );
    if let (Some(coverage), Some(dir)) = (&coverage, &compile_coverage) {
        coverage.collect_once(dir);
    }
    toolchain.coverage_dir = None;
    // Public host listings use their host graph; actual runs/checks use their selected target.
    let dependency_toolchain = if list {
        &discovery_toolchain
    } else {
        &toolchain
    };
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
    if args.watch {
        watched.set_tests(cases.iter().map(|c| c.file.as_path()));
    }
    let in_expr = |case: &Case| {
        filter_expr
            .as_ref()
            .is_none_or(|expr| expr_selects(expr, case, &roots[case.root], cwd))
    };
    let (in_part, time_share) =
        match partition::parse(args.partition.as_deref(), args.partition_times.as_deref()) {
            Ok(split) => split,
            Err(error) => return fail(error),
        };
    let mut selected = Vec::new();
    for case in cases {
        if !kind_selected(case.kind, args, bench) {
            continue;
        }
        if !in_expr(&case)
            || only.is_some_and(|files| !files.contains(&watch::canonical(&case.file)))
        {
            continue;
        }
        if let Some(run_as) = select(&case, &args.filters, args.exact) {
            selected.push(Case {
                run_as,
                ..case
            });
        }
    }
    if !args.changed.is_empty() {
        let ctx = deps::Context {
            roots: &roots,
            root_flags: &root_flags,
            work: &work,
            toolchain: dependency_toolchain,
            bench,
            jobs: settings.jobs,
        };
        selected = match deps::narrow(selected, &args.changed, &ctx) {
            Ok(selected) => selected,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&work);
                return fail(error);
            }
        };
    }
    if let Some(in_part) = &in_part {
        selected = selected
            .into_iter()
            .enumerate()
            .filter(|(i, c)| in_part(*i, &c.name))
            .map(|(_, c)| c)
            .collect();
    }
    if let Some(share) = &time_share {
        share.keep(&mut selected);
    }
    let mut reporter = report::Reporter::new(
        args.format,
        args.junit.clone(),
        cwd,
        args.interp,
        settings.capture,
        args.quiet,
    );
    if list {
        // Which folders were searched depends on the current directory, so say which.
        for root in &roots {
            eprintln!(
                "jacket: root {}{}",
                jaic::display_path_from(cwd, &root.dir),
                if root.library {
                    " (library)"
                } else {
                    ""
                }
            );
        }
        if args.deps {
            let listing = deps::Listing {
                root_flags: &root_flags,
                work: &work,
                toolchain: &discovery_toolchain,
                bench,
                jobs: settings.jobs,
            };
            if let Err(error) = deps::print(&selected, &roots, &listing) {
                let _ = std::fs::remove_dir_all(&work);
                return fail(error);
            }
        } else {
            reporter.list(&selected);
        }
        let _ = std::fs::remove_dir_all(&work);
        return ExitCode::SUCCESS;
    }
    // The files of the selected tests only, so a filter or a partition shares the check too.
    if args.check_only && !bench && !check_roots(&roots, &selected, &root_flags, &work, &toolchain)
    {
        let _ = std::fs::remove_dir_all(&work);
        return ExitCode::from(2);
    }
    if selected.is_empty() {
        let _ = std::fs::remove_dir_all(&work);
        return fail(if bench {
            "no benchmarks match"
        } else {
            "no tests match"
        });
    }
    if let Some(bench_args) = bench_args {
        let code = bench::run(&bench::Run {
            cases: &selected,
            roots: &roots,
            built: &built,
            toolchain: &toolchain,
            settings: &settings,
            target: &target,
            needs: &settings.needs,
            project_dir: &project_dir,
            args: bench_args,
        });
        let _ = std::fs::remove_dir_all(&work);
        return code;
    }

    let plan = Plan {
        cases: selected,
        roots,
        root_flags,
        built,
        stdlib,
        project: project_dir.clone(),
        toolchain,
        settings,
        work,
        temp_counter: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
        coverage,
        on_board,
    };
    execute(&plan, &target, cross_skip.as_deref(), &mut reporter)
}

/// Whether a jaic flag chooses how much the build is optimized.
fn sets_optimization(flag: &str) -> bool {
    matches!(
        flag,
        "-o" | "-optimized" | "-od" | "-optimized_debug" | "-very_debug"
    ) || flag
        .strip_prefix("-O")
        .is_some_and(|level| level.len() == 1 && level.bytes().all(|b| b.is_ascii_digit()))
}

/// A seed for a run that was not given one: the clock mixed with the process id.
fn random_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mixed = (nanos as u64) ^ (u64::from(std::process::id()) << 32);
    // Keep it short enough to type: a decimal number of at most nine digits.
    mixed.wrapping_mul(0x9e37_79b9_7f4a_7c15) % 1_000_000_000
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
        seed: args.seed.unwrap_or_else(random_seed),
        isolate: !args.no_isolation,
        capture,
        allow_missing_prerequisites: args.allow_missing_prerequisites,
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
/// The runner to use: `--runner`, else the target's, else the settings'.
fn choose_runner(
    args: &TestArgs,
    runner: Vec<String>,
    target_runner: Option<String>,
    dir: &Path,
) -> Vec<String> {
    match target_runner {
        Some(words) if args.runner.is_none() => runner_words(&words, dir),
        _ => runner,
    }
}

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
                    cases: built
                        .cases
                        .iter()
                        .filter(|(test, _)| *test == entry.name)
                        .map(|(_, case)| case.clone())
                        .collect(),
                });
            }
        }
        for (files, kind) in [
            (&root.golden, Kind::Golden),
            (&root.compile_fail, Kind::CompileFail),
        ] {
            for file in files {
                // The path below `tests/`, so `compile_fail/bad` and `faults/overflow`.
                let local = file
                    .strip_prefix(root.dir.join("tests"))
                    .unwrap_or(file)
                    .with_extension("")
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
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
                    cases: Vec::new(),
                });
            }
        }
        // Doc tests are named for their module, so they need no root prefix.
        for (doc, block) in root.docs.iter().enumerate() {
            for problem in &block.problems {
                warnings.push(format!(
                    "{}:{}: {problem}",
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
                options: block.options.clone(),
                run_as: local,
                doc: Some(doc),
                cases: Vec::new(),
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
                prerequisites::optional_skip(&case.options, &jacket_config::env::for_program)
            })
            .or_else(|| {
                plan.on_board
                    .then(|| {
                        case.board_skip(&plan.roots, plan.toolchain.check_only)
                            .map(String::from)
                    })
                    .flatten()
            })
            .or_else(|| {
                (!case.compiles_only(&plan.roots, plan.toolchain.check_only))
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
    let files: Vec<PathBuf> = to_run
        .iter()
        .map(|&index| plan.cases[index].file.clone())
        .collect();
    if let Err(error) = prerequisites::prepare(&files, &plan.stdlib) {
        return fail(error);
    }
    let (parallel, serial) = jobs::jobs(plan, &to_run);
    for (jobs, threads) in [(parallel, plan.settings.jobs), (serial, 1)] {
        jobs::run_jobs(plan, jobs, threads, &mut |message| match message {
            Message::Started(case) => reporter.started(&plan.cases[case]),
            Message::Slow(case, elapsed) => reporter.slow(&plan.cases[case], elapsed),
            Message::Done(mut outcome) => {
                if plan.cases[outcome.case].kind != Kind::CompileFail {
                    prerequisites::skip_missing_probe(
                        &mut outcome,
                        plan.settings.allow_missing_prerequisites,
                    );
                }
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
    let mut coverage_failed = false;
    let mut coverage_dropped = false;
    if let Some(coverage) = &plan.coverage {
        match coverage.report(reporter.is_human()) {
            Ok(verdict) => coverage_dropped = verdict == coverage::Verdict::Dropped,
            Err(error) => {
                eprintln!("jacket: error: {error}");
                coverage_failed = true;
            }
        }
    }
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
    if failed || coverage_dropped {
        ExitCode::from(1)
    } else if coverage_failed {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

/// Type-checks the unit test files of the `selected` cases, a root at a time, for the target of
/// `toolchain`, printing each root that does not. Whether all of them passed.
fn check_roots(
    roots: &[discover::Root],
    selected: &[Case],
    root_flags: &[Vec<String>],
    work: &Path,
    toolchain: &Toolchain,
) -> bool {
    let mut passed = true;
    for (i, root) in roots.iter().enumerate() {
        let files: Vec<PathBuf> = root
            .unit
            .iter()
            .filter(|file| {
                selected
                    .iter()
                    .any(|c| c.root == i && c.kind == Kind::Test && &c.file == *file)
            })
            .cloned()
            .collect();
        if files.is_empty() {
            continue;
        }
        let work = work.join(format!("check-{i}"));
        if let Err(output) = build::check_unit(&files, &root_flags[i], &work, toolchain) {
            eprintln!(
                "jacket: error: the tests in {} do not compile for {}\n{output}",
                root.dir.join("tests").display(),
                toolchain.os
            );
            passed = false;
        }
    }
    passed
}

/// A root's build: nothing to build, the built program, or the compile errors.
type BuildResult = Result<Option<Built>, String>;

/// How many roots build at once, and how many keep a waiting compiler under `--interp`.
struct Workers {
    builds: usize,
    warm: usize,
}

/// Builds every root's unit tests, `workers.builds` at a time. Under `--interp` the first
/// `workers.warm` roots (the test workers that run at once) keep their compiler waiting for the
/// run; the others compile again when they run, so no more processes sit idle than can work.
fn build_all(
    roots: &[discover::Root],
    root_flags: &[Vec<String>],
    work: &Path,
    toolchain: &Toolchain,
    listing: Option<build::Listing>,
    bench: bool,
    workers: Workers,
) -> Vec<BuildResult> {
    crate::parallel::map(roots, workers.builds, |i, root| {
        let files = if bench {
            &root.bench
        } else {
            &root.unit
        };
        if files.is_empty() {
            return Ok(None);
        }
        build::unit(
            files,
            &root_flags[i],
            &work.join(format!("build-{i}")),
            toolchain,
            listing,
            i < workers.warm,
        )
        .map(Some)
    })
}

#[cfg(test)]
mod tests;
