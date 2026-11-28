//! Parsing arguments into a request and checking the input file.
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use argdef::{Args, ErrorKind};
use jaic_cli_args::cli::{Cli, Mode};
use jaic_cli_args::options::{CliError, Request};

use super::usage::{no_file_error, program_args_error, unknown_option};

pub(super) fn parse(args: &[OsString]) -> Result<Request, CliError> {
    if args.is_empty() {
        return Err(no_file_error());
    }

    let mut cli = Cli::parse(args).map_err(|error| match error.kind {
        ErrorKind::Unknown => unknown_option(&error.subject),
        _ => CliError::from(error),
    })?;

    if cli.help {
        return Ok(Request::Help);
    }

    if cli.version {
        return Ok(Request::Version);
    }

    if cli.print_stdlib {
        return Ok(Request::Stdlib);
    }

    if let Some(shell) = cli.completions {
        return Ok(Request::Completions(shell));
    }

    let mut files = std::mem::take(&mut cli.positional).into_iter();
    let (file, extra) = (files.next(), files.next());

    // Arguments after `--` belong to the program, which only `-interp` runs; its argv[0] is the
    // source file, like a built program's executable path.
    if let Some(rest) = cli.after_dashes.take() {
        if !cli.interp {
            return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
        }

        cli.program_args.push(file.clone().unwrap_or_default());
        cli.program_args.extend(rest);
    }

    cli.command_line = cli.after_dash.take().unwrap_or_default();
    let Some(file) = file else {
        return Err(no_file_error());
    };

    if cli.doc_comments {
        return Ok(Request::DocComments(file, cli.shared.imports));
    }

    if cli.doc_gaps {
        return Ok(Request::DocGaps(file, cli.shared.imports));
    }

    if cli.doc_model {
        return Ok(Request::DocModel(file, cli.shared.imports));
    }

    if cli.procedures {
        return Ok(Request::Procedures(file, cli.shared.os, cli.shared.cpu));
    }

    cli.file = file;
    cli.levels =
        jaic::diagnostics::Levels::parse(&cli.level_flags).map_err(|message| {
            match message.split_once("\nhelp: ") {
                Some((message, help)) => CliError::new(message).help(help),
                None => CliError::new(message),
            }
        })?;
    cli.mode = Mode::from_flags(cli.check, cli.interp, cli.deps)?;
    cli.finish(extra)?;
    Ok(Request::Compile(Box::new(cli)))
}

/// Why `file` cannot be compiled, if it cannot: missing, a directory, unreadable.
pub(super) fn check_input(file: &str) -> Result<(), CliError> {
    let path = Path::new(file);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    // The `.jai` files of a directory, as paths joined to `dir` the way the user wrote it.
    let jai_files = |dir: &Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".jai"))
            .collect();

        names.sort();
        names
    };

    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut error = CliError::new(format!("file `{file}` does not exist"));

            if let Some(near) = jaic::suggest::similar_sibling(path, |n| n.ends_with(".jai")) {
                error = error.help(format!("did you mean `{}`?", near.display()));
            } else if !parent.is_dir() {
                error = error.help(format!(
                    "the directory `{}` does not exist either",
                    parent.display()
                ));
            } else if let Ok(cwd) = std::env::current_dir()
                && path.is_relative()
            {
                error = error.help(format!(
                    "relative paths start from the current directory, {}",
                    cwd.display()
                ));
            }

            Err(error)
        }
        Err(e) => Err(CliError::new(format!(
            "cannot read `{file}`: {}",
            jaic::io_reason(&e)
        ))),
        Ok(meta) if meta.is_dir() => {
            let mut error = CliError::new(format!("`{file}` is a directory, not a .jai file"));
            let files = jai_files(path);
            let entry = jacket_config::project_entry(&jacket_config::Disk, path);

            match (entry, files.len()) {
                (Some(entry), _) => {
                    error = error.help(format!("did you mean `{}`?", entry.display()));
                }
                (None, 0) => error = error.help("it holds no .jai files"),
                (None, _) => {
                    let list: Vec<String> =
                        files.iter().take(5).map(|f| format!("`{f}`")).collect();

                    error = error.help(format!("pass one of its files: {}", list.join(", ")));
                }
            }

            Err(error)
        }
        Ok(_) => match std::fs::File::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(CliError::new(format!(
                "cannot read `{file}`: {}",
                jaic::io_reason(&e)
            ))),
        },
    }
}

/// `path` as the user would write it: relative to where jaic was started when inside it.
pub(super) fn shown(path: &Path) -> String {
    jaic::display_path(path)
}

/// Print an error given as text, its `help: `/`note: ` lines rendered as such.
pub(super) fn print_error(message: &str) {
    let report = jaic::render::Report::from_text(jaic::diagnostics::Code::CliFailure, message);

    eprint!("{}", report.render());
}

/// Print a command-line mistake and return the usage exit status.
pub(super) fn report_cli_error(error: CliError) -> ExitCode {
    print_cli_error(error);
    ExitCode::from(2)
}

pub(super) fn print_cli_error(error: CliError) {
    let mut report = jaic::render::Report::new(jaic::diagnostics::Code::CliUsage, error.message);

    for help in error.help {
        report = report.help(help);
    }

    eprint!("{}", report.render());
}

/// The colour choice among jaic's own arguments (`-no_color`, `--color <when>`), read before
/// parsing so that even command-line mistakes are printed in it. Plugin, metaprogram and
/// program arguments (from the first `+Name`, `-` or `--`) are not jaic's.
pub(super) fn color_choice(args: &[OsString]) -> jaic::render::ColorChoice {
    // Only the colour flags matter here, and they are UTF-8.
    let own: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .take_while(|a| a != "-" && a != "--" && !a.starts_with('+'))
        .collect();

    if own.iter().any(|a| a == "-no_color") {
        return jaic::render::ColorChoice::Never;
    }

    jaic::render::ColorChoice::from_args(&own, jaic_cli_args::shared::COLOR_NAMES)
}
