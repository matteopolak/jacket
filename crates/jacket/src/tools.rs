//! Finding and starting the tools `jacket` fronts. They stay separate programs: `jacket` looks
//! for each next to its own executable (a release archive or a Homebrew keg), then on `PATH`,
//! and starts it with the arguments it was given. Exit codes and standard streams pass through.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub(crate) const JAIC: &str = "jaic";
pub(crate) const JAILINT: &str = "jailint";
pub(crate) const JAILSP: &str = "jailsp";
pub(crate) const JAIFMT: &str = "jaifmt";
pub(crate) const ALL: [&str; 4] = [JAIC, JAILINT, JAILSP, JAIFMT];

/// The file to start for `tool`: next to this executable, else the first match on `PATH`. When
/// there is none, the error says that `command` needs it.
pub(crate) fn locate(tool: &str, command: &str) -> Result<PathBuf, String> {
    jacket_config::tools::locate(tool).ok_or_else(|| not_installed(tool, command))
}

/// The error for `tool` missing when `command` needs it.
pub(crate) fn not_installed(tool: &str, command: &str) -> String {
    format!(
        "`{tool}` is not installed, and `{command}` needs it\nnote: jacket looks next to its own executable, then on PATH; install a release with install.sh or install.ps1 (docs/tools/releases.md), or put `{tool}` on PATH"
    )
}

/// [`locate`], printing why it failed.
pub(crate) fn require(tool: &str, command: &str) -> Option<PathBuf> {
    match locate(tool, command) {
        Ok(path) => Some(path),
        Err(message) => {
            eprintln!("jacket: error: {message}");
            None
        }
    }
}

fn missing(program: &OsStr, error: &std::io::Error) -> ExitCode {
    eprintln!(
        "jacket: error: could not start `{}`: {}\nnote: jacket looks next to its own executable, then on PATH; install a release with install.sh or install.ps1 (docs/tools/releases.md), or put the tool on PATH",
        program.to_string_lossy(),
        jaic::io_reason(error)
    );
    ExitCode::from(127)
}

/// Replaces this process with `program` (on Windows: starts it and waits), passing its exit
/// code on.
pub(crate) fn exec(program: &OsStr, args: &[OsString]) -> ExitCode {
    exec_with_env(program, args, &[])
}

/// `exec` with `env` added to the environment.
pub(crate) fn exec_with_env(
    program: &OsStr,
    args: &[OsString],
    env: &[(&str, OsString)],
) -> ExitCode {
    let mut command = Command::new(program);

    command.args(args).envs(env.iter().map(|(k, v)| (k, v)));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        missing(program, &error)
    }
    #[cfg(not(unix))]
    {
        run(program, args, env)
    }
}

/// Starts `program`, waits, and returns its exit code.
#[cfg(not(unix))]
fn run(program: &OsStr, args: &[OsString], env: &[(&str, OsString)]) -> ExitCode {
    match status(program, args, env) {
        Ok(code) => ExitCode::from(code),
        Err(error) => missing(program, &error),
    }
}

/// Runs `program` to the end with `env` added to its environment; its exit status, clamped to a byte.
pub(crate) fn status(
    program: &OsStr,
    args: &[OsString],
    env: &[(&str, OsString)],
) -> std::io::Result<u8> {
    let status = Command::new(program)
        .args(args)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()?;
    Ok(status.code().map_or(255, |c| c.clamp(0, 255) as u8))
}

/// Starts `jacket-<name>` from `PATH`, like `cargo foo` starts `cargo-foo`.
pub(crate) fn external(name: &OsStr, args: &[OsString]) -> ExitCode {
    let mut program = OsString::from("jacket-");

    program.push(name);
    program.push(std::env::consts::EXE_SUFFIX);

    if jacket_config::tools::on_path(Path::new(&program)).is_none() {
        eprintln!(
            "jacket: no such command: `{}`\nhelp: `jacket --help` lists the commands; other commands are programs called `jacket-<name>` on PATH",
            name.to_string_lossy()
        );
        return ExitCode::from(2);
    }

    exec(&program, args)
}
