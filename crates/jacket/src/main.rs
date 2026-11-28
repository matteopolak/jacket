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
mod doc;
mod markup;
mod migrate;
mod parallel;
mod project;
mod shown;
mod status;
mod target;
mod test;
mod toolchain;
mod tools;
mod upgrade;

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Command, TOOL_COMMANDS};
use jacket_config::Disk;
use jacket_dep::{dep, git, net, prerequisite};

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
        Command::Doc(args) => doc::run(&args, &cwd),
        Command::Config {
            action,
        } => commands::config(action, &cwd),
        Command::Migrate(args) => migrate::run(&args, &cwd),
        Command::Toolchain {
            action,
        } => commands::toolchain(action, &cwd),
        Command::Dep {
            action,
        } => dep::run(action, &cwd),
        Command::Clean => commands::clean(&cwd),
        Command::Version => commands::version(),
        Command::Completions {
            shell,
            tool,
        } => commands::completions(shell, tool),
        Command::Explain {
            code,
        } => commands::explain(code.as_deref()),
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
        return match tools::require(tool, &format!("jacket {command}")) {
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

    if let Some(loaded) = &loaded
        && let Err(error) = project::check_toolchain(loaded)
    {
        return commands::fail(error);
    }

    if matches!(command, "build" | "check" | "run")
        && let Some(loaded) = &loaded
        && let Err(error) = dep::require_checked_out(&loaded.config, loaded.dir())
    {
        eprintln!("jacket: error: {error}");
        return ExitCode::from(2);
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
        "lint" => run_tool(tools::JAILINT, "lint", args),
        "fmt" => run_tool(tools::JAIFMT, "fmt", args),
        _ => run_tool(tools::JAILSP, "lsp", args),
    }
}

/// Starts `tool` with `args`, or says it cannot be found.
fn run_tool(tool: &str, command: &str, args: &[OsString]) -> ExitCode {
    match tools::require(tool, &format!("jacket {command}")) {
        Some(program) => tools::exec(program.as_os_str(), args),
        None => ExitCode::from(127),
    }
}

fn run_jaic(check: bool, args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let Some(jaic) = tools::require(
        tools::JAIC,
        if check {
            "jacket check"
        } else {
            "jacket build"
        },
    ) else {
        return ExitCode::from(127);
    };
    let expansion = match target::expand(args, ctx.loaded.map(|loaded| &loaded.config)) {
        Ok(expansion) => expansion,
        Err(error) => return commands::fail(error),
    };
    let argv = match compile::jaic_argv(check, &expansion.args, ctx) {
        Ok(argv) => argv,
        Err(error) => return commands::fail(error),
    };

    tools::exec_with_env(jaic.as_os_str(), &argv, &project_name_env(ctx))
}

/// `JACKET_PROJECT`: the name jaic's build status gives the project (its directory's).
fn project_name_env(ctx: &compile::Context) -> Vec<(&'static str, OsString)> {
    ctx.loaded
        .and_then(|loaded| loaded.dir().file_name())
        .map(|name| (jacket_config::env::PROJECT.name, name.to_os_string()))
        .into_iter()
        .collect()
}

/// Why `jacket run` cannot start `exe`: `target` has no runner. A triple named on the command line
/// gets a `[target]` table to copy.
fn no_runner(target: &target::Target, exe: &Path, cwd: &Path) -> String {
    let at = shown::path(exe, cwd);

    if target.project || target.board.is_some() {
        let name = &target.name;

        return format!(
            "target `{name}` has no runner, so jacket cannot run what it built\nhelp: set `runner` under `[target.{name}]` in jacket.toml; the program is at {at}"
        );
    }

    let triple = target.triple.as_deref().unwrap_or(&target.name);
    let runner = if triple.starts_with("wasm") {
        "wasmtime {exe}"
    } else {
        "PROGRAM {exe}"
    };

    format!(
        "`{triple}` is not this machine's target, so jacket cannot run what it built\nhelp: name a runner for it in jacket.toml, then run with `--target NAME`:\n      [target.NAME]\n      triple = \"{triple}\"\n      runner = \"{runner}\"\nnote: the program is at {at}"
    )
}

/// `jacket run`: build with jaic, then start the executable with the arguments after `--`.
fn run_program(args: &[OsString], ctx: &compile::Context) -> ExitCode {
    let (jaic_args, program_args) = compile::split_run(args);
    let expansion = match target::expand(jaic_args, ctx.loaded.map(|loaded| &loaded.config)) {
        Ok(expansion) => expansion,
        Err(error) => return commands::fail(error),
    };
    let argv = match compile::jaic_argv(false, &expansion.args, ctx) {
        Ok(argv) => argv,
        Err(error) => return commands::fail(error),
    };
    let Some(jaic) = tools::require(tools::JAIC, "jacket run") else {
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

    env.push((
        jacket_config::env::REPORT_EXE.name,
        report.clone().into_os_string(),
    ));
    let started = tools::status(jaic.as_os_str(), &argv, &env);
    let reported = compile::reported_executable(&report);
    let _ = std::fs::remove_file(&report);

    match started {
        Ok(0) => {}
        Ok(code) => return ExitCode::from(code),
        Err(error) => {
            eprintln!("jacket: could not start `jaic`: {error}");
            return ExitCode::from(127);
        }
    }

    let exe = match reported {
        Ok(exe) => exe,
        Err(error) => return commands::fail(error),
    };

    if let Some(color) = status::lines(&argv) {
        let shown = exe.strip_prefix(ctx.cwd).unwrap_or(&exe);

        status::print("Running", &format!("`{}`", shown.display()), color);
    }

    let Some(target) = expansion.target else {
        return tools::exec(exe.as_os_str(), program_args);
    };
    let target = match target.with_sysroot() {
        Ok(target) => target,
        Err(message) => return commands::fail(message),
    };
    let Some(command) = target.command(&exe) else {
        if target.triple.is_none() && target.board.is_none() {
            return tools::exec(exe.as_os_str(), program_args);
        }

        return commands::fail(no_runner(&target, &exe, ctx.cwd));
    };

    if !program_args.is_empty() {
        return commands::fail(format!(
            "target `{}` runs the program through its runner, which passes it no arguments",
            target.name
        ));
    }

    tools::exec(&command[0], &command[1..])
}
