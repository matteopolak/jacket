//! The option table and the actions that store option values.
use super::cli::{Added, Cli, Flavor};
use super::tuning;
use jaic::builtin_keys::BitcodeOptimization;
use jaic::sema::TargetCpu;
use std::path::PathBuf;

/// A build's exit status when the program has no `main` (its work is all `#run`s), so
/// tools can tell "nothing to build" from a failed compile.
pub(super) const NO_MAIN_STATUS: u8 = 3;

/// A command-line mistake: the message, then help lines.
pub(super) struct CliError {
    pub(super) message: String,
    pub(super) help: Vec<String>,
}

impl CliError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        CliError {
            message: message.into(),
            help: Vec::new(),
        }
    }

    pub(super) fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

/// What the command line asks for.
pub(super) enum Request {
    Compile(Box<Cli>),
    Help,
    Version,
}

/// Where `jaic -help` lists an option.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Section {
    Modes,
    Program,
    Output,
    Diagnostics,
}

/// An option's line in `jaic -help`.
#[derive(Clone, Copy)]
pub(super) struct Help {
    pub(super) section: Section,
    /// The option as the help spells it, with its value (`-exe <name>`).
    pub(super) synopsis: &'static str,
    /// What it does. Each further line is indented under the first.
    pub(super) text: &'static str,
}

/// What an option does to the command line being parsed.
#[derive(Clone, Copy)]
pub(super) enum Action {
    Help,
    Version,
    Check,
    Interp,
    Flavor(Flavor),
    OptLevel(BitcodeOptimization),
    /// Turns a `Cli` switch on.
    Switch(fn(&mut Cli) -> &mut bool),
    /// Accepted and ignored.
    Ignore,
    /// Takes the next argument (or the text after `=`): `what` says what it is for the
    /// "needs a value" error, `store` checks it and puts it in the `Cli`. `store` gets the
    /// option as it was spelt.
    Value {
        what: &'static str,
        store: fn(&mut Cli, &str, String) -> Result<(), CliError>,
    },
}

/// One option of jaic's own. `OPTIONS` drives parsing, "did you mean" suggestions, the
/// build-only check and `jaic -help`.
pub(super) struct CliOption {
    /// Its spellings; a mistyped option suggests the nearest of these.
    pub(super) names: &'static [&'static str],
    /// Spellings it also accepts but never suggests (`-h`, `--help`).
    aliases: &'static [&'static str],
    /// Only a build takes it: `-check` and `-interp` write no output.
    pub(super) build_only: bool,
    /// Its `-help` line; `None` when another option's line covers it.
    pub(super) help: Option<Help>,
    pub(super) action: Action,
}

impl CliOption {
    pub(super) const fn new(names: &'static [&'static str], action: Action) -> Self {
        CliOption {
            names,
            aliases: &[],
            build_only: false,
            help: None,
            action,
        }
    }

    const fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    pub(super) const fn build_only(mut self) -> Self {
        self.build_only = true;
        self
    }

    pub(super) const fn help(
        mut self,
        section: Section,
        synopsis: &'static str,
        text: &'static str,
    ) -> Self {
        self.help = Some(Help {
            section,
            synopsis,
            text,
        });
        self
    }

    /// The option spelt `name`.
    pub(super) fn find(name: &str) -> Option<&'static CliOption> {
        tuning::all_options()
            .find(|option| option.names.contains(&name) || option.aliases.contains(&name))
    }

    /// The option spelt `name` if it takes a value.
    pub(super) fn find_value(name: &str) -> Option<&'static CliOption> {
        Self::find(name).filter(|option| matches!(option.action, Action::Value { .. }))
    }
}

/// `Action::Value` for an option that only stores its value.
macro_rules! store {
    ($what:literal, |$cli:ident, $value:ident| $store:expr) => {
        Action::Value {
            what: $what,
            store: |$cli, _, $value| {
                $store;
                Ok(())
            },
        }
    };
}

pub(crate) use store;

pub(super) const OPTIONS: &[CliOption] = {
    use Section::{Diagnostics, Modes, Output, Program};
    &[
        CliOption::new(&["-check"], Action::Check).help(
            Modes,
            "-check",
            "compile the program without running or writing anything",
        ),
        CliOption::new(&["-interp"], Action::Interp).help(
            Modes,
            "-interp",
            "run `main` in the interpreter (arguments after `--` go to the program)",
        ),
        CliOption::new(
            &["-add"],
            store!("the code to add to the program", |cli, code| cli
                .added
                .push(Added::Code(code))),
        )
        .help(
            Program,
            "-add \"code\"",
            "add the code to the target program (repeatable)",
        ),
        CliOption::new(
            &["-run"],
            store!("the code to run at compile time", |cli, code| cli
                .added
                .push(Added::Run(code))),
        )
        .help(
            Program,
            "-run \"code\"",
            "run the code at compile time, as a `#run` (repeatable)",
        ),
        CliOption::new(
            &["-I", "-import_dir"],
            store!("a directory", |cli, dir| cli
                .imports
                .push(PathBuf::from(dir))),
        )
        .help(
            Program,
            "-import_dir <dir>",
            "also look for modules in <dir> (repeatable; short form -I)",
        ),
        CliOption::new(
            &["-os"],
            Action::Value {
                what: "an OS name: linux, windows, macos or wasm",
                store: store_os,
            },
        )
        .help(
            Program,
            "-os <os>",
            "target OS: linux, windows, macos or wasm",
        ),
        CliOption::new(
            &["-cpu"],
            Action::Value {
                what: "a CPU name: x64 or arm64",
                store: store_cpu,
            },
        )
        .help(Program, "-cpu <cpu>", "target CPU: x64 or arm64"),
        CliOption::new(
            &["-target"],
            store!(
                "an LLVM target triple, such as x86_64-pc-windows-gnu",
                |cli, triple| cli.target = Some(triple)
            ),
        )
        .help(
            Program,
            "-target <triple>",
            "build for an LLVM target triple",
        ),
        CliOption::new(&["-no_dce"], Action::Switch(|cli| &mut cli.no_dce)).help(
            Program,
            "-no_dce",
            "type-check unreferenced module code too",
        ),
        // Check is not run unless asked for, so there is nothing to turn off.
        CliOption::new(&["-no_check"], Action::Ignore).help(
            Program,
            "-no_check",
            "accepted and ignored: jaic does not run the Check plugin unless asked",
        ),
        CliOption::new(
            &["-no_check_bindings"],
            Action::Switch(|cli| &mut cli.no_check_bindings),
        )
        .help(
            Program,
            "-no_check_bindings",
            "set CHECK_BINDINGS=false on a +Check plugin, if there is one",
        ),
        CliOption::new(
            &["-exe"],
            store!("an executable name", |cli, name| cli.exe = Some(name)),
        )
        .build_only()
        .help(
            Output,
            "-exe <name>",
            "the executable's name; it may include a directory and extension\n\
             (`-exe build/app.exe`). A bare name goes next to the source file",
        ),
        CliOption::new(
            &["-output_path"],
            store!("a directory", |cli, dir| cli.output_path =
                Some(PathBuf::from(dir))),
        )
        .build_only()
        .help(
            Output,
            "-output_path <dir>",
            "the directory the executable is written to",
        ),
        CliOption::new(&["-o", "-optimized"], Action::Flavor(Flavor::Optimized))
            .build_only()
            .help(
                Output,
                "-o, -optimized",
                "optimized build (LLVM O2) without stack traces or runtime checks",
            ),
        CliOption::new(
            &["-od", "-optimized_debug"],
            Action::Flavor(Flavor::OptimizedDebug),
        )
        .build_only()
        .help(
            Output,
            "-od, -optimized_debug",
            "lightly optimized build (LLVM O1) that keeps stack traces and checks",
        ),
        CliOption::new(&["-very_debug"], Action::Flavor(Flavor::VeryDebug))
            .build_only()
            .help(
                Output,
                "-very_debug",
                "unoptimized build with every runtime check on",
            ),
        CliOption::new(&["-O0"], Action::OptLevel(BitcodeOptimization::O0))
            .build_only()
            .help(
                Output,
                "-O0, -O1, -O2, -O3",
                "pick the LLVM optimization level directly",
            ),
        CliOption::new(&["-O1"], Action::OptLevel(BitcodeOptimization::O1)).build_only(),
        CliOption::new(&["-O2"], Action::OptLevel(BitcodeOptimization::O2)).build_only(),
        CliOption::new(&["-O3"], Action::OptLevel(BitcodeOptimization::O3)).build_only(),
        CliOption::new(&["-no_inline"], Action::Switch(|cli| &mut cli.no_inline))
            .build_only()
            .help(Output, "-no_inline", "do not inline procedures"),
        CliOption::new(&["-no_split"], Action::Switch(|cli| &mut cli.no_split))
            .build_only()
            .help(
                Output,
                "-no_split",
                "generate code as one module instead of several",
            ),
        CliOption::new(
            &["-no_backtrace_on_crash"],
            Action::Switch(|cli| &mut cli.no_backtrace_on_crash),
        )
        .help(
            Output,
            "-no_backtrace_on_crash",
            "leave the crash handler out of the program",
        ),
        CliOption::new(
            &["-emit_ir"],
            store!("a file to write the LLVM IR to", |cli, file| cli.emit_ir =
                Some(PathBuf::from(file))),
        )
        .build_only()
        .help(Output, "-emit_ir <file.ll>", "also write the LLVM IR"),
        CliOption::new(
            &["-no_debug_info"],
            Action::Switch(|cli| &mut cli.no_debug_info),
        )
        .build_only()
        .help(
            Output,
            "-no_debug_info",
            "leave out native debug information",
        ),
        CliOption::new(
            &["-sanitize"],
            Action::Value {
                what: "address, undefined and/or fuzzer, comma-separated",
                store: store_sanitize,
            },
        )
        .build_only()
        .help(
            Output,
            "-sanitize <list>",
            "address, undefined and/or fuzzer, comma-separated",
        ),
        CliOption::new(&["-coverage"], Action::Switch(|cli| &mut cli.coverage))
            .build_only()
            .help(
                Output,
                "-coverage",
                "record line and function coverage (docs/tools/coverage.md)",
            ),
        CliOption::new(
            &["-no_workspace_output"],
            Action::Switch(|cli| &mut cli.no_workspace_output),
        )
        .help(
            Output,
            "-no_workspace_output",
            "(-interp only) do not write what a metaprogram's workspaces ask for",
        ),
        CliOption::new(&["-no_color"], Action::Switch(|cli| &mut cli.no_color)).help(
            Diagnostics,
            "-no_color",
            "no colour in diagnostics",
        ),
        CliOption::new(
            &["-msvc_format"],
            Action::Switch(|cli| &mut cli.msvc_format),
        )
        .help(
            Diagnostics,
            "-msvc_format",
            "one `path(line,col): error: message` line per diagnostic",
        ),
        CliOption::new(&["-quiet"], Action::Switch(|cli| &mut cli.quiet)).help(
            Diagnostics,
            "-quiet",
            "print nothing but errors",
        ),
        CliOption::new(&["-verbose"], Action::Switch(|cli| &mut cli.verbose)).help(
            Diagnostics,
            "-verbose",
            "say what is being done",
        ),
        CliOption::new(&["-timings"], Action::Switch(|cli| &mut cli.timings)).help(
            Diagnostics,
            "-timings",
            "print the wall time of each phase on stderr",
        ),
        CliOption::new(&["-help"], Action::Help)
            .aliases(&["-h", "--help"])
            .help(Diagnostics, "-h, -help, --help", "print this help"),
        CliOption::new(&["-version"], Action::Version)
            .aliases(&["-V", "--version"])
            .help(Diagnostics, "-V, -version, --version", "print the version"),
    ]
};

fn store_os(cli: &mut Cli, _: &str, os: String) -> Result<(), CliError> {
    let Some(parsed) = jaic::target::parse_os(&os) else {
        let mut error = CliError::new(format!("unknown OS `{os}` for `-os`"));
        if let Some(near) = jaic::suggest::closest(&os, jaic::target::OS_NAMES) {
            error = error.help(format!("did you mean `-os {near}`?"));
        }
        return Err(error.help("the OS names are linux, windows, macos and wasm"));
    };
    cli.os = Some(parsed);
    Ok(())
}

fn store_cpu(cli: &mut Cli, _: &str, cpu: String) -> Result<(), CliError> {
    match jaic::target::parse_cpu(&cpu) {
        Some(parsed) if parsed != TargetCpu::Wasm => {
            cli.cpu = Some(parsed);
            Ok(())
        }
        _ => Err(CliError::new(format!("unknown CPU `{cpu}` for `-cpu`"))
            .help("the CPU names are x64 (or x86_64) and arm64 (or aarch64)")),
    }
}

fn store_sanitize(cli: &mut Cli, option: &str, list: String) -> Result<(), CliError> {
    if let Some(bad) = list
        .split(',')
        .map(str::trim)
        .find(|s| !matches!(*s, "address" | "undefined" | "fuzzer"))
    {
        return Err(
            CliError::new(format!("unknown sanitizer `{bad}` for `{option}`"))
                .help("use address, undefined, fuzzer or a list: `-sanitize address,fuzzer`"),
        );
    }
    cli.sanitize.push(list);
    Ok(())
}
