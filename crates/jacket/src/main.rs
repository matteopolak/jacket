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
mod project;
mod tools;

use clap::Parser;
use cli::{Cli, Command, TOOL_COMMANDS};
use jacket_config::Disk;
use std::ffi::{OsStr, OsString};
use std::path::Path;
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
    match Cli::parse().command {
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
        Command::Clean => {
            project::migrate_on_first_use(&cwd);
            commands::clean(&cwd)
        }
        Command::Version => commands::version(),
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
        return tools::exec(tools::find(tool).as_os_str(), args);
    }
    project::migrate_on_first_use(cwd);
    let loaded = match project::load(cwd) {
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
        "lint" => tools::exec(tools::find(tools::JAILINT).as_os_str(), args),
        "fmt" => tools::exec(
            tools::find(tools::JAIFMT).as_os_str(),
            &fmt_args(args, &ctx),
        ),
        _ => tools::exec(tools::find(tools::JAILSP).as_os_str(), args),
    }
}

fn run_jaic(check: bool, args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let argv = compile::jaic_argv(check, args, ctx);
    tools::exec(tools::find(tools::JAIC).as_os_str(), &argv)
}

/// `jacket run`: build with jaic, then start the executable with the arguments after `--`.
fn run_program(args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let (jaic_args, program_args) = compile::split_run(args);
    let argv = compile::jaic_argv(false, jaic_args, ctx);
    let jaic = tools::find(tools::JAIC);
    let runs_itself = argv
        .iter()
        .any(|a| a == "-interp" || a == "-check" || a == "-help" || a == "-version");
    if runs_itself {
        return tools::exec(jaic.as_os_str(), &argv);
    }
    match tools::status(jaic.as_os_str(), &argv) {
        Ok(0) => {}
        Ok(code) => return ExitCode::from(code),
        Err(error) => {
            eprintln!("jacket: could not start `jaic`: {error}");
            return ExitCode::from(127);
        }
    }
    let Some(exe) = compile::executable(&argv, ctx.cwd).filter(|p| p.is_file()) else {
        eprintln!(
            "jacket: error: jaic built nothing to run\nhelp: pass the executable's location with -exe, or run it yourself"
        );
        return ExitCode::from(2);
    };
    tools::exec(exe.as_os_str(), program_args)
}

/// `jaifmt` takes paths and has no default: the project is the default.
fn fmt_args(args: &[OsString], ctx: &compile::Context) -> Vec<OsString> {
    let mut out = args.to_vec();
    let value_flags: [&OsStr; 1] = [OsStr::new("--config")];
    let mut skip = false;
    let mut positional = false;
    let mut informational = false;
    for arg in args {
        if skip {
            skip = false;
        } else if value_flags.contains(&arg.as_os_str()) {
            skip = true;
        } else if matches!(
            arg.to_str(),
            Some("--stdin" | "-h" | "--help" | "--version" | "-V")
        ) {
            informational = true;
        } else if !arg.to_string_lossy().starts_with('-') {
            positional = true;
        }
    }
    if !positional && !informational {
        let root = ctx.loaded.map_or(ctx.cwd, |l| l.dir());
        out.push(if root == ctx.cwd {
            ".".into()
        } else {
            root.as_os_str().to_owned()
        });
    }
    out
}
