//! Command-line errors, what a command line asks for, and how the options read their values.
use argdef::{Error, Shell};
use jaic::target::{TargetCpu, TargetOs};

use super::cli::{Cli, CodegenBackend};

/// A build's exit status when the program has no `main` (its work is all `#run`s), so
/// tools can tell "nothing to build" from a failed compile.
pub const NO_MAIN_STATUS: u8 = 3;

/// A command-line mistake: the message, then help lines.
pub struct CliError {
    /// What went wrong.
    pub message: String,
    /// Extra lines printed after the message, one hint each.
    pub help: Vec<String>,
}

impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        CliError {
            message: error.message,
            help: error.help,
        }
    }
}

impl CliError {
    /// A mistake with no help lines.
    pub fn new(message: impl Into<String>) -> Self {
        CliError {
            message: message.into(),
            help: Vec::new(),
        }
    }

    /// Adds a hint line after the message.
    #[must_use]
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

/// What the command line asks for.
pub enum Request {
    /// Compile, check or interpret a program with these settings.
    Compile(Box<Cli>),
    /// Print the help page.
    Help,
    /// Print the version.
    Version,
    /// Print the standard library folder in use.
    Stdlib,
    /// Print a shell completion script.
    Completions(Shell),
    /// Print the doc comments of the library in a folder.
    DocComments(String, Vec<std::path::PathBuf>),
    /// Print which public declarations of the library in a folder have docs and an example.
    DocGaps(String, Vec<std::path::PathBuf>),
    /// Print the documentation model of the library in a folder as JSON.
    DocModel(String, Vec<std::path::PathBuf>),
    /// Print the procedures of the library folders (a path list) for a target.
    Procedures(String, Option<TargetOs>, Option<TargetCpu>),
}

/// The message for two build flavors given at once.
pub const FLAVOR_CONFLICT: &str = "`-o`, `-od` and `-very_debug` each pick a build flavor; use one";

/// `-backend llvm` or `-backend cranelift`.
pub fn read_backend(name: &str, text: &str) -> Result<CodegenBackend, Error> {
    match text {
        "llvm" => Ok(CodegenBackend::Llvm),
        "cranelift" => Ok(CodegenBackend::Cranelift),
        _ => Err(
            Error::invalid(format!("`{name}` takes llvm or cranelift, not `{text}`")).help(
                format!("`{name} cranelift` builds with the experimental Cranelift code generator"),
            ),
        ),
    }
}

/// `-A`, `-W` and `-D`: which level the option name stands for.
pub fn store_level(cli: &mut Cli, option: &str, code: &str) -> Result<(), Error> {
    let level = match option {
        "-A" => jaic::diagnostics::Level::Allow,
        "-W" => jaic::diagnostics::Level::Warn,
        _ => jaic::diagnostics::Level::Deny,
    };

    cli.level_flags.push((code.to_string(), level));
    Ok(())
}

/// Stores a `-sanitize` list, a comma-separated set of `address`, `undefined` and `fuzzer`;
/// any other name is refused.
pub fn add_sanitizer(cli: &mut Cli, option: &str, list: &str) -> Result<(), Error> {
    if let Some(bad) = list
        .split(',')
        .map(str::trim)
        .find(|s| !matches!(*s, "address" | "undefined" | "fuzzer"))
    {
        return Err(
            Error::invalid(format!("unknown sanitizer `{bad}` for `{option}`"))
                .help("use address, undefined, fuzzer or a list: `-sanitize address,fuzzer`"),
        );
    }

    cli.sanitize.push(list.to_string());
    Ok(())
}
