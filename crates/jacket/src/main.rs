//! `jacket`: one front door to `jaic`, `jailint`, `jaifmt` and `jailsp`. See
//! `docs/tools/jacket.md`.
//!
//! The tools stay separate programs, found next to this executable (or on `PATH`). A command
//! that belongs to one of them is handed to it before clap sees the arguments, so the tool's own
//! flags work exactly as they do when it is run alone.
#![forbid(unsafe_code)]
mod cli;
mod commands;
mod compile;
mod fmt;
mod project;
mod status;
mod test;
mod tools;
mod upgrade;

use clap::Parser;
use cli::{Cli, Command, TOOL_COMMANDS};
use jacket_config::Disk;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Ok(cwd) = std::env::current_dir() else {
        eprintln!("jacket: cannot read the current directory");
        return ExitCode::from(2);
    };
    if let Some(command) = args.first().and_then(|a| a.to_str())
        && TOOL_COMMANDS.contains(&command)
    {
        return tool_command(command, &args[1..], &cwd);
    }
    let testing = matches!(
        args.first().and_then(|a| a.to_str()),
        Some("test" | "bench")
    );
    let args = if testing {
        cli::test_args(args)
    } else {
        args
    };
    let cli = Cli::parse_from(std::iter::once(OsString::from("jacket")).chain(args));
    match cli.command {
        Command::Test(test) => test::run(&test, &cwd, None),
        Command::Bench(bench) => test::run(&bench.test, &cwd, Some(&bench)),
        Command::New {
            name,
            lib,
        } => commands::new(&name, lib),
        Command::Init {
            lib,
        } => commands::init(lib),
        Command::Config {
            action,
        } => commands::config(action, &cwd),
        Command::Clean => commands::clean(&cwd),
        Command::Version => commands::version(),
        Command::Upgrade {
            version,
            check,
            yes,
        } => upgrade::run(&upgrade::Options {
            version,
            check,
            yes,
        }),
        Command::External(external) => tools::external(&external[0], &external[1..]),
        // Handled above, before clap saw them.
        _ => ExitCode::from(2),
    }
}

/// `command` is one of [`TOOL_COMMANDS`]; `args` are everything after it.
fn tool_command(command: &str, args: &[OsString], cwd: &Path) -> ExitCode {
    let raw = match command {
        "jaic" => Some(tools::JAIC),
        "jailint" => Some(tools::JAILINT),
        "jailsp" => Some(tools::JAILSP),
        "jaifmt" => Some(tools::JAIFMT),
        _ => None,
    };
    if let Some(tool) = raw {
        return match tools::require(tool) {
            Some(program) => tools::exec(program.as_os_str(), args),
            None => ExitCode::from(127),
        };
    }
    // `jailint` reads the same file and prints its own notes.
    let loaded = match project::load_with(cwd, command != "lint") {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("jacket: error: {error}");
            return ExitCode::from(2);
        }
    };
    if let Some(loaded) = &loaded {
        project::check_toolchain(loaded);
    }
    let fs = Disk;
    let ctx = compile::Context {
        loaded: loaded.as_ref(),
        cwd,
        fs: &fs,
    };
    match command {
        "build" => run_jaic(false, args, &ctx),
        "check" => run_jaic(true, args, &ctx),
        "run" => run_program(args, &ctx),
        "lint" => run_tool(tools::JAILINT, args),
        "fmt" => fmt::run(&fmt::plan(args, &ctx)),
        _ => run_tool(tools::JAILSP, args),
    }
}

/// Starts `tool` with `args`, or says it cannot be found.
fn run_tool(tool: &str, args: &[OsString]) -> ExitCode {
    match tools::require(tool) {
        Some(program) => tools::exec(program.as_os_str(), args),
        None => ExitCode::from(127),
    }
}

fn run_jaic(check: bool, args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let Some(jaic) = tools::require(tools::JAIC) else {
        return ExitCode::from(127);
    };
    let argv = compile::jaic_argv(check, args, ctx);
    tools::exec_with_env(jaic.as_os_str(), &argv, &project_name_env(ctx))
}

/// `JACKET_PROJECT`: the name jaic's build status gives the project (its directory's).
fn project_name_env(ctx: &compile::Context) -> Vec<(&'static str, OsString)> {
    ctx.loaded
        .and_then(|loaded| loaded.dir().file_name())
        .map(|name| ("JACKET_PROJECT", name.to_os_string()))
        .into_iter()
        .collect()
}

/// `jacket run`: build with jaic, then start the executable with the arguments after `--`.
fn run_program(args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let (jaic_args, program_args) = compile::split_run(args);
    let argv = compile::jaic_argv(false, jaic_args, ctx);
    let Some(jaic) = tools::require(tools::JAIC) else {
        return ExitCode::from(127);
    };
    let runs_itself = argv
        .iter()
        .any(|a| a == "-interp" || a == "-check" || a == "-help" || a == "-version");
    let mut env = project_name_env(ctx);
    if runs_itself {
        return tools::exec_with_env(jaic.as_os_str(), &argv, &env);
    }
    // The build may be a metaprogram that names its own executable, so ask jaic what it wrote.
    let report = std::env::temp_dir().join(format!("jacket-exe-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&report);
    env.push(("JAIC_REPORT_EXE", report.clone().into_os_string()));
    let started = tools::status(jaic.as_os_str(), &argv, &env);
    let reported = std::fs::read_to_string(&report).ok();
    let _ = std::fs::remove_file(&report);
    match started {
        Ok(0) => {}
        Ok(code) => return ExitCode::from(code),
        Err(error) => {
            eprintln!("jacket: could not start `jaic`: {error}");
            return ExitCode::from(127);
        }
    }
    let reported = reported
        .as_deref()
        .and_then(|text| text.lines().last())
        .map(PathBuf::from);
    let Some(exe) = reported
        .or_else(|| compile::executable(&argv, ctx.cwd))
        .filter(|p| p.is_file())
    else {
        eprintln!(
            "jacket: error: jaic built nothing to run\nhelp: pass the executable's location with -exe, or run it yourself"
        );
        return ExitCode::from(2);
    };
    if let Some(color) = status::lines(&argv) {
        let shown = exe.strip_prefix(ctx.cwd).unwrap_or(&exe);
        status::print("Running", &format!("`{}`", shown.display()), color);
    }
    tools::exec(exe.as_os_str(), program_args)
}
