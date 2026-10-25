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
    /// List benchmarks (`--list`); running them is reserved for a later release
    Bench(TestArgs),
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

#[derive(clap::Args, Debug)]
pub struct TestArgs {
    /// Run the tests whose name contains one of these; `test/subtest` runs one subtest
    #[arg(value_name = "FILTER")]
    pub filters: Vec<String>,
    /// Filters are whole names, not parts of names
    #[arg(long)]
    pub exact: bool,
    /// Print the tests (and benchmarks) instead of running them
    #[arg(long)]
    pub list: bool,
    /// Tests run at once [default: the machine's threads]
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
    /// Target OS, as jaic's -os
    #[arg(long, value_name = "OS")]
    pub os: Option<String>,
    /// Target CPU, as jaic's -cpu
    #[arg(long, value_name = "CPU")]
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
    /// Write what golden and compile-fail tests printed into their expected files
    #[arg(long)]
    pub accept: bool,
    /// Flags for jaic, after `--`
    #[arg(last = true, value_name = "JAIC_FLAGS")]
    pub jaic_args: Vec<String>,
}

/// jaic's single-dash `-os` and `-cpu` (before `--`) as the `--os` and `--cpu` clap knows.
pub fn test_args(args: Vec<OsString>) -> Vec<OsString> {
    let mut seen_end = false;
    args.into_iter()
        .map(|arg| {
            seen_end |= arg == "--";
            match arg.to_str() {
                Some("-os") if !seen_end => "--os".into(),
                Some("-cpu") if !seen_end => "--cpu".into(),
                _ => arg,
            }
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
