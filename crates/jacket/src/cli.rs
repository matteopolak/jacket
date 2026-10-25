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
