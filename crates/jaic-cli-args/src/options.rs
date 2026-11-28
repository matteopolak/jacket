//! Command-line errors, what a command line asks for, and how the options read their values.
use std::path::PathBuf;

use argdef::{Error, Shell};
use jaic::flags::{self, FlagError, Shared, SharedFlag};
use jaic::target::{TargetCpu, TargetOs};

use super::cli::{Added, Cli, CodegenBackend};

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

impl From<FlagError> for CliError {
    fn from(error: FlagError) -> Self {
        CliError {
            message: error.message,
            help: error.help,
        }
    }
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

fn flag_error(error: FlagError) -> Error {
    error
        .help
        .into_iter()
        .fold(Error::invalid(error.message), Error::help)
}

/// A value of a flag the tools share, read by the flag's own rules.
fn shared(flag: &SharedFlag, name: &str, text: &str) -> Result<Shared, Error> {
    flag.parse(name, text).map_err(flag_error)
}

/// Reads the `-os` value; the error lists the valid names.
pub fn read_os(name: &str, text: &str) -> Result<TargetOs, Error> {
    match shared(&flags::OS, name, text)? {
        Shared::Os(os) => Ok(os),
        _ => unreachable!("`-os` reads an OS"),
    }
}

/// Reads the `-cpu` value, `x64` or `arm64` (or their long names).
pub fn read_cpu(name: &str, text: &str) -> Result<TargetCpu, Error> {
    match shared(&flags::CPU, name, text)? {
        Shared::Cpu(cpu) => Ok(cpu),
        _ => unreachable!("`-cpu` reads a CPU"),
    }
}

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

/// Reads a `-package_import_dir` value, `<package>=<dir>`, into the package folder and the
/// folder to search for it; both halves must be non-empty.
pub fn read_package_import(name: &str, text: &str) -> Result<(PathBuf, PathBuf), Error> {
    match shared(&flags::PACKAGE_IMPORT_DIR, name, text)? {
        Shared::PackageImportDir(package, dir) => Ok((package, dir)),
        _ => unreachable!("`-package_import_dir` reads two folders"),
    }
}

/// How many jobs `-j` asks for (`0` is as no flag: one per core).
pub fn read_jobs(name: &str, text: &str) -> Result<Option<usize>, Error> {
    match shared(&flags::JOBS, name, text)? {
        Shared::Jobs(n) => Ok((n > 0).then_some(n)),
        _ => unreachable!("`-j` reads a number"),
    }
}

/// Stores `--color`: turns colour off for `never` and on for `auto` and `always`.
pub fn set_color(cli: &mut Cli, name: &str, text: &str) -> Result<(), Error> {
    match shared(&flags::COLOR, name, text)? {
        Shared::Color(choice) => cli.no_color = choice == jaic::render::ColorChoice::Never,
        _ => unreachable!("`--color` reads a choice"),
    }

    Ok(())
}

/// Appends a `-add` string to `cli.added`.
pub fn add_code(cli: &mut Cli, _: &str, code: &str) -> Result<(), Error> {
    cli.added.push(Added::Code(code.to_string()));
    Ok(())
}

/// Appends a `-run` string to `cli.added`.
pub fn add_run(cli: &mut Cli, _: &str, code: &str) -> Result<(), Error> {
    cli.added.push(Added::Run(code.to_string()));
    Ok(())
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
