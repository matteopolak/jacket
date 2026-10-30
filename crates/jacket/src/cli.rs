//! The command line, as clap sees it. `build`, `check`, `run`, `lint`, `fmt`, `lsp` and the raw
//! tool commands are listed here only so `--help` shows them: `main` hands their arguments to
//! the tool before clap parses anything, so no flag of `jaic`, `jailint`, `jaifmt` or `jailsp`
//! is ever interpreted, translated or reordered.
use clap::{Parser, Subcommand};
use std::ffi::OsString;

/// The commands whose arguments belong to a tool.
pub const TOOL_COMMANDS: [&str; 10] = [
    "build", "check", "run", "lint", "fmt", "lsp", "jaic", "jailint", "jailsp", "jaifmt",
];

const ABOUT: &str =
    "One front door to the Jai toolchain: compiler, linter, formatter and language server.";

const AFTER_HELP: &str = "\
`jacket` is a drop-in for the tools it fronts. `build`, `check` and `run` take the flags of `jaic`
(single-dash, as in `jacket build -o -exe app main.jai`) and pass them through untouched; `lint`,
`fmt` and `lsp` take those of `jailint`, `jaifmt` and `jailsp`. `jacket build -help` prints the
compiler's own help. What `jacket` adds is the project: with no .jai file named, `build` and
`check` use `[project] entry` from the nearest jacket.toml (else build.jai, first.jai, main.jai,
src/main.jai or module.jai), and `[experimental]` settings become jaic flags unless you pass them.

A command that is none of these runs `jacket-<command>` from PATH, as `cargo` does.

Configuration is the nearest jacket.toml (docs/tools/jacket.md).";

#[derive(Parser, Debug)]
#[command(name = "jacket", version, about = ABOUT, after_help = AFTER_HELP, disable_help_subcommand = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Compile a program: `jaic <args>`, flags untouched
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Build(Passthrough),
    /// Type-check without writing anything: `jaic -check <args>`
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Check(Passthrough),
    /// Build, then run the program; arguments after `--` go to it (`-interp` runs it in jaic)
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Run(Passthrough),
    /// Lint with `jailint` (`--fix`, `-D warnings`, paths)
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Lint(Passthrough),
    /// Format with `jaifmt` (`--check`, paths; the project by default)
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Fmt(Passthrough),
    /// Run the language server over stdio (`jailsp`)
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Lsp(Passthrough),
    /// Create a project in a new directory
    New {
        /// Directory and project name
        name: String,
        /// A library (module.jai) instead of a program
        #[arg(long)]
        lib: bool,
    },
    /// Create a project in the current directory
    Init {
        /// A library (module.jai) instead of a program
        #[arg(long)]
        lib: bool,
    },
    /// Build the tests once, then run each in its own process (`-os`/`-cpu` as in jaic; flags
    /// after `--` go to jaic)
    Test(TestArgs),
    /// Build and measure the `@bench` procedures (optimized by default; the options of `test`
    /// select and build them)
    Bench(BenchArgs),
    /// Show the configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Remove the executable `jacket build` writes
    Clean,
    /// Print the version of jacket and of every tool it fronts
    Version,
    /// Install the latest release over this installation
    Upgrade {
        /// Install this release (`0.22.0` or `v0.22.0`) instead of the latest
        #[arg(long = "version", value_name = "TAG")]
        version: Option<String>,
        /// Only report whether an update exists
        #[arg(long)]
        check: bool,
        /// Do not ask for confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Run `jaic` with exactly these arguments
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Jaic(Passthrough),
    /// Run `jailint` with exactly these arguments
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Jailint(Passthrough),
    /// Run `jailsp` with exactly these arguments
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Jailsp(Passthrough),
    /// Run `jaifmt` with exactly these arguments
    #[command(disable_help_flag = true, trailing_var_arg = true)]
    Jaifmt(Passthrough),
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    /// One JSON object per event on standard output
    Json,
}

/// What `--coverage` writes to `--coverage-out`.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageFormat {
    /// lcov's text format, with line counts and procedures
    Lcov,
    /// One self-contained page: modules, files and the source with the count of each line
    Html,
}

/// What a row of the coverage table is.
#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageBy {
    File,
    /// The nearest folder with a `module.jai`
    Module,
}

#[derive(clap::Args, Clone, Debug)]
pub struct TestArgs {
    /// Run the tests whose name contains one of these; `test/subtest` runs one subtest
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,
    /// Filters are whole names, not parts of names
    #[arg(long)]
    pub exact: bool,
    /// Run the tests a filter expression selects, as `module(Sort) and not kind(property)`; with
    /// FILTERs, the tests both select (predicates: test, module, file, kind, all, none)
    #[arg(short = 'E', long, value_name = "EXPR")]
    pub filter_expr: Option<String>,
    /// Print the tests (and benchmarks) instead of running them
    #[arg(long)]
    pub list: bool,
    /// With `--list`: print the source files the test programs read (what `jaic -deps` prints for
    /// each), instead of the tests
    #[arg(long, requires = "list")]
    pub deps: bool,
    /// Report which lines of the code the tests ran (unit tests, golden programs and doc
    /// examples), and write them as lcov
    #[arg(long, conflicts_with_all = ["list", "review"])]
    pub coverage: bool,
    /// Where `--coverage` writes its report [default: target/lcov.info, or target/coverage.html]
    #[arg(long, value_name = "FILE", requires = "coverage")]
    pub coverage_out: Option<std::path::PathBuf>,
    /// The report's format
    #[arg(long, value_enum, default_value_t = CoverageFormat::Lcov, requires = "coverage")]
    pub coverage_format: CoverageFormat,
    /// What a row of the printed table is
    #[arg(long, value_enum, default_value_t = CoverageBy::File, requires = "coverage")]
    pub coverage_by: CoverageBy,
    /// Fail when a module (or the total) covers fewer lines than this recorded baseline by more
    /// than `--coverage-margin`
    #[arg(long, value_name = "FILE", requires = "coverage")]
    pub coverage_baseline: Option<std::path::PathBuf>,
    /// Percentage points a module may lose against `--coverage-baseline`
    #[arg(
        long,
        value_name = "POINTS",
        default_value_t = 0.5,
        requires = "coverage"
    )]
    pub coverage_margin: f64,
    /// Record this run's lines per module as a baseline
    #[arg(long, value_name = "FILE", requires = "coverage")]
    pub coverage_record: Option<std::path::PathBuf>,
    /// Add a markdown table of the modules to this file (`$GITHUB_STEP_SUMMARY`)
    #[arg(long, value_name = "FILE", requires = "coverage")]
    pub coverage_summary: Option<std::path::PathBuf>,
    /// Also report the code below this folder (repeatable); by default the test roots and the
    /// folders of `-import_dir`
    #[arg(long, value_name = "DIR", requires = "coverage")]
    pub coverage_include: Vec<std::path::PathBuf>,
    /// Run again whenever a `.jai` file of the tests or the code they import changes
    #[arg(long, conflicts_with_all = ["list", "review"])]
    pub watch: bool,
    /// Tests run at once, as jaic's -j [default: the machine's threads]
    #[arg(short = 'j', long, value_name = "N")]
    pub jobs: Option<usize>,
    /// Seconds before a test without `@timeout` is killed; 0 for no limit [default: 120]
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,
    /// Run a failed test again up to N times; one that then passes is FLAKY
    #[arg(long, value_name = "N")]
    pub retries: Option<u32>,
    /// Start no more tests after the first failure
    #[arg(long)]
    pub fail_fast: bool,
    /// Run every test even when a profile sets `fail_fast`
    #[arg(long, conflicts_with = "fail_fast")]
    pub no_fail_fast: bool,
    /// Show the tests' output as it is written (runs one test at a time)
    #[arg(long)]
    pub no_capture: bool,
    /// Run a root's tests in one process instead of one process each
    #[arg(long)]
    pub no_isolation: bool,
    /// Print only the tests that fail, time out or are flaky, then the summary
    #[arg(short, long)]
    pub quiet: bool,
    /// `human` text, or `json`: one event per line on standard output
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub format: OutputFormat,
    /// Also write the results as `JUnit` XML
    #[arg(long, value_name = "FILE")]
    pub junit: Option<std::path::PathBuf>,
    /// Run one part of the tests: `hash:K/N` (by name) or `count:K/N` (by position)
    #[arg(long, value_name = "SPEC")]
    pub partition: Option<String>,
    /// Run the tests in jaic's interpreter
    #[arg(long)]
    pub interp: bool,
    /// Target OS, as jaic's -os (also spelt `-os`)
    #[arg(long, value_name = "OS", value_parser = ["linux", "windows", "macos", "wasm"])]
    pub os: Option<String>,
    /// Target CPU, as jaic's -cpu (also spelt `-cpu`)
    #[arg(long, value_name = "CPU", value_parser = parse_cpu)]
    pub cpu: Option<String>,
    /// Command that runs test programs built for another target (`node tools/wasi_run.mjs`)
    #[arg(long, value_name = "COMMAND")]
    pub runner: Option<String>,
    /// Capabilities this machine has, for tests with `@needs` (`network,gpu`)
    #[arg(long, value_delimiter = ',', value_name = "LIST")]
    pub needs: Vec<String>,
    /// Use `[test.profile.NAME]` from jacket.toml
    #[arg(long, value_name = "NAME")]
    pub profile: Option<String>,
    /// Test the `tests/` of these folders instead of the project's
    #[arg(long, value_name = "DIR")]
    pub path: Vec<std::path::PathBuf>,
    /// Run only the doc tests: the `jai` code blocks in a library's doc comments
    #[arg(long, conflicts_with = "no_doc")]
    pub doc: bool,
    /// Leave the doc tests out
    #[arg(long)]
    pub no_doc: bool,
    /// Keep every test's temporary folder (failed tests keep theirs anyway)
    #[arg(long)]
    pub keep_temp: bool,
    /// Write what golden and compile-fail tests printed into their expected files, and keep
    /// the snapshots that differ or are new
    #[arg(long)]
    pub accept: bool,
    /// Go through the pending `.snap.new` snapshots one by one (only lists them without a terminal)
    #[arg(long, conflicts_with_all = ["list", "accept"])]
    pub review: bool,
    /// The seed property tests start from [default: random for each run, shown with a failure]
    #[arg(long, value_name = "N")]
    pub seed: Option<u64>,
    /// Flags for jaic, after `--`
    #[arg(last = true, value_name = "FLAGS")]
    pub jaic_args: Vec<String>,
}

/// `jacket bench`: the selection and build options of `jacket test`, and how to measure.
#[derive(clap::Args, Clone, Debug)]
pub struct BenchArgs {
    #[command(flatten)]
    pub test: TestArgs,
    /// Samples per benchmark, over `@samples` [default: 50]
    #[arg(long, value_name = "N", value_parser = clap::value_parser!(u32).range(2..))]
    pub samples: Option<u32>,
    /// Warm-up per benchmark in milliseconds, at least; it goes on until the time per iteration
    /// is steady [default: 100]
    #[arg(long, value_name = "MS", value_parser = clap::value_parser!(u32).range(1..))]
    pub warmup: Option<u32>,
    /// Time to spend sampling each benchmark in milliseconds, over `@min_time` [default: 1000]
    #[arg(long, value_name = "MS", value_parser = clap::value_parser!(u32).range(1..))]
    pub measure_ms: Option<u32>,
    /// Bootstrap resamples for the confidence intervals and the comparison
    #[arg(long, value_name = "N", default_value_t = 100_000, value_parser = clap::value_parser!(u32).range(100..))]
    pub resamples: u32,
    /// A change smaller than this many percent is noise
    #[arg(long, value_name = "PCT", default_value_t = 2.0)]
    pub noise_threshold: f64,
    /// A change counts when its p-value is below this
    #[arg(long, value_name = "P", default_value_t = 0.05)]
    pub significance: f64,
    /// Write the results to `target/bench-baselines/NAME.json`
    #[arg(long, value_name = "NAME")]
    pub save_baseline: Option<String>,
    /// Compare with the results saved as NAME and show the change per benchmark
    #[arg(long, value_name = "NAME")]
    pub baseline: Option<String>,
    /// Exit 1 when a benchmark regressed (the change is significant and beyond the noise
    /// threshold) by more than PCT percent
    #[arg(long, value_name = "PCT", requires = "baseline")]
    pub max_regression: Option<f64>,
}

/// A CPU name as jaic's `-cpu` takes it, in the spelling the tests use (`x64`, `arm64`).
fn parse_cpu(name: &str) -> Result<String, String> {
    match name {
        "x64" | "x86_64" => Ok("x64".into()),
        "arm64" | "aarch64" => Ok("arm64".into()),
        _ => Err("expected x64 or arm64".into()),
    }
}

/// jaic's single-dash `-os` and `-cpu` (before `--`, also as `-os=linux`) as the `--os` and
/// `--cpu` clap knows.
pub fn test_args(args: Vec<OsString>) -> Vec<OsString> {
    let mut seen_end = false;
    args.into_iter()
        .map(|arg| {
            seen_end |= arg == "--";
            let Some(text) = arg.to_str().filter(|_| !seen_end) else {
                return arg;
            };
            for name in ["os", "cpu"] {
                if text == format!("-{name}") {
                    return format!("--{name}").into();
                }
                if let Some(value) = text.strip_prefix(&format!("-{name}=")) {
                    return format!("--{name}={value}").into();
                }
            }
            arg
        })
        .collect()
}

/// Arguments for a tool; clap never reads them.
#[derive(clap::Args, Debug)]
pub struct Passthrough {
    #[arg(allow_hyphen_values = true, num_args = 0.., value_name = "ARGS")]
    pub args: Vec<OsString>,
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Print the effective configuration
    Show,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaic_spellings_of_the_target_reach_clap() {
        let args = |words: &[&str]| -> Vec<OsString> { words.iter().map(OsString::from).collect() };
        let got = test_args(args(&[
            "test",
            "-os",
            "wasm",
            "-cpu=arm64",
            "--",
            "-os",
            "x",
        ]));
        assert_eq!(
            got,
            args(&["test", "--os", "wasm", "--cpu=arm64", "--", "-os", "x"])
        );
    }

    #[test]
    fn cpu_aliases_are_normalised() {
        assert_eq!(parse_cpu("aarch64").as_deref(), Ok("arm64"));
        assert_eq!(parse_cpu("x86_64").as_deref(), Ok("x64"));
        assert!(parse_cpu("riscv").is_err());
    }
}
