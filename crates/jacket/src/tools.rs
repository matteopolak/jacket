//! Finding and starting the tools `jacket` fronts. They stay separate programs: `jacket` looks
//! for each next to its own executable (a release archive or a Homebrew keg), then on `PATH`,
//! and starts it with the arguments it was given. Exit codes and standard streams pass through.
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

pub const JAIC: &str = "jaic";
pub const JAILINT: &str = "jailint";
pub const JAILSP: &str = "jailsp";
pub const JAIFMT: &str = "jaifmt";
pub const ALL: [&str; 4] = [JAIC, JAILINT, JAILSP, JAIFMT];

fn file_name(tool: &str) -> String {
    format!("{tool}{}", std::env::consts::EXE_SUFFIX)
}

/// The program to start for `tool`: next to this executable if it is there, else the bare name
/// (looked up on `PATH` when it starts).
pub fn find(tool: &str) -> PathBuf {
    if let Ok(me) = std::env::current_exe()
        && let Some(dir) = me.parent()
    {
        let beside = dir.join(file_name(tool));
        if beside.is_file() {
            return beside;
        }
    }
    PathBuf::from(file_name(tool))
}

/// Whether `tool` can be found at all.
pub fn exists(tool: &str) -> bool {
    let program = find(tool);
    program.is_absolute() || on_path(&program)
}

fn on_path(program: &std::path::Path) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
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
    let mut command = Command::new(program);
    command.args(args);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        missing(program, &error)
    }
    #[cfg(not(unix))]
    {
        run(program, args)
    }
}

/// Starts `program`, waits, and returns its exit code.
#[cfg(not(unix))]
fn run(program: &OsStr, args: &[OsString]) -> ExitCode {
    match status(program, args) {
        Ok(code) => ExitCode::from(code),
        Err(error) => missing(program, &error),
    }
}

/// Starts `program` with inherited streams and waits; its exit status (255 when a signal ended
/// it).
pub fn status(program: &OsStr, args: &[OsString]) -> std::io::Result<u8> {
    let status = Command::new(program).args(args).status()?;
    Ok(status.code().map_or(255, |c| c.clamp(0, 255) as u8))
}

/// Starts `jacket-<name>` from `PATH`, like `cargo foo` starts `cargo-foo`.
pub fn external(name: &OsStr, args: &[OsString]) -> ExitCode {
    let mut program = OsString::from("jacket-");
    program.push(name);
    program.push(std::env::consts::EXE_SUFFIX);
    if !on_path(std::path::Path::new(&program)) {
        eprintln!(
            "jacket: no such command: `{}`\nhelp: `jacket --help` lists the commands; other commands are programs called `jacket-<name>` on PATH",
            name.to_string_lossy()
        );
        return ExitCode::from(2);
    }
    exec(&program, args)
}
