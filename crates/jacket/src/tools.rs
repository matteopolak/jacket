//! Finding and starting the tools `jacket` fronts. They stay separate programs: `jacket` looks
//! for each next to its own executable (a release archive or a Homebrew keg), then on `PATH`,
//! and starts it with the arguments it was given. Exit codes and standard streams pass through.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub const JAIC: &str = "jaic";
pub const JAILINT: &str = "jailint";
pub const JAILSP: &str = "jailsp";
pub const JAIFMT: &str = "jaifmt";
pub const ALL: [&str; 4] = [JAIC, JAILINT, JAILSP, JAIFMT];

fn file_name(tool: &str) -> String {
    format!("{tool}{}", std::env::consts::EXE_SUFFIX)
}

/// The file to start for `tool`: next to this executable, else the first match on `PATH`.
pub fn locate(tool: &str) -> Result<PathBuf, String> {
    let name = file_name(tool);
    // Through symlinks (`~/.local/bin/jacket` -> the installed release) to where the tools are.
    if let Ok(me) = std::env::current_exe().and_then(|me| me.canonicalize())
        && let Some(dir) = me.parent()
    {
        let beside = dir.join(&name);
        if beside.is_file() {
            return Ok(beside);
        }
    }
    on_path(Path::new(&name)).ok_or_else(|| {
        format!(
            "cannot find `{tool}`\nhelp: jacket looks next to its own executable, then on PATH; install the toolchain archive or put `{tool}` on PATH"
        )
    })
}

/// [`locate`], printing why it failed.
pub fn require(tool: &str) -> Option<PathBuf> {
    match locate(tool) {
        Ok(path) => Some(path),
        Err(message) => {
            eprintln!("jacket: error: {message}");
            None
        }
    }
}

/// The file `program` names in a folder of `PATH`.
fn on_path(program: &Path) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

fn missing(program: &OsStr, error: &std::io::Error) -> ExitCode {
    eprintln!(
        "jacket: could not start `{}`: {error}\nhelp: jacket looks next to its own executable, then on PATH; install the toolchain archive or put the tool on PATH",
        program.to_string_lossy()
    );
    ExitCode::from(127)
}

/// Replaces this process with `program` (on Windows: starts it and waits), passing its exit
/// code on.
pub fn exec(program: &OsStr, args: &[OsString]) -> ExitCode {
    exec_with_env(program, args, &[])
}

/// `exec` with `env` added to the environment.
pub fn exec_with_env(program: &OsStr, args: &[OsString], env: &[(&str, OsString)]) -> ExitCode {
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
pub fn status(program: &OsStr, args: &[OsString], env: &[(&str, OsString)]) -> std::io::Result<u8> {
    let status = Command::new(program)
        .args(args)
        .envs(env.iter().map(|(k, v)| (k, v)))
        .status()?;
    Ok(status.code().map_or(255, |c| c.clamp(0, 255) as u8))
}

/// Starts `jacket-<name>` from `PATH`, like `cargo foo` starts `cargo-foo`.
pub fn external(name: &OsStr, args: &[OsString]) -> ExitCode {
    let mut program = OsString::from("jacket-");
    program.push(name);
    program.push(std::env::consts::EXE_SUFFIX);
    if on_path(Path::new(&program)).is_none() {
        eprintln!(
            "jacket: no such command: `{}`\nhelp: `jacket --help` lists the commands; other commands are programs called `jacket-<name>` on PATH",
            name.to_string_lossy()
        );
        return ExitCode::from(2);
    }
    exec(&program, args)
}
