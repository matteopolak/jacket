//! Parsing arguments into a request and checking the input file.
use super::usage::{no_file_error, program_args_error, unknown_option};
use jaic_cli_args::cli::{Cli, Mode, Plugin};
use jaic_cli_args::options::{Action, CliError, CliOption, Request};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::ExitCode;

/// The text of a command-line argument, or the error for one that is not UTF-8. Only the values
/// of `Action::Passthrough` options may be any OS string.
fn text(arg: &OsStr) -> Result<&str, CliError> {
    arg.to_str().ok_or_else(|| {
        CliError::new(format!("argument `{}` is not valid UTF-8", arg.display())).help(
            "only the values of -link_arg, -linker, -sysroot and -compiler_rt may be any bytes",
        )
    })
}

/// The next argument as text, for an option whose value was not written inline.
fn next_text<'a>(
    rest: &mut impl Iterator<Item = &'a OsString>,
) -> Result<Option<String>, CliError> {
    rest.next()
        .map(|arg| text(arg).map(str::to_owned))
        .transpose()
}

/// `-name=value` with a value that is not UTF-8: the option, when it is a passthrough one, and
/// the value as given.
fn passthrough_inline(arg: &OsStr) -> Option<(&'static CliOption, OsString)> {
    let (name, value) = jacket_config::os_text::split_once(arg, b'=')?;
    let option = CliOption::find(name.to_str()?)?;
    matches!(option.action, Action::Passthrough { .. }).then_some((option, value))
}

pub(super) fn parse(args: &[OsString]) -> Result<Request, CliError> {
    if args.is_empty() {
        return Err(no_file_error());
    }
    let mut cli = Cli::default();
    let mut file: Option<String> = None;
    let mut extra: Option<String> = None;
    let (mut check, mut interp, mut doc_comments, mut doc_gaps, mut doc_model, mut procedures) =
        (false, false, false, false, false, false);
    let mut rest = args.iter();
    while let Some(os_arg) = rest.next() {
        if os_arg.to_str().is_none()
            && let Some((option, value)) = passthrough_inline(os_arg)
            && let Action::Passthrough {
                store, ..
            } = option.action
        {
            store(&mut cli, value);
            continue;
        }
        let arg = text(os_arg)?;
        match arg {
            "-" => {
                // Metaprogram arguments run up to a `--` (if any).
                for arg in rest.by_ref() {
                    if arg == "--" {
                        if !interp {
                            return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
                        }
                        cli.program_args.push(file.clone().unwrap_or_default());
                        break;
                    }
                    cli.command_line.push(text(arg)?.to_owned());
                }
                for arg in rest.by_ref() {
                    cli.program_args.push(text(arg)?.to_owned());
                }
                continue;
            }
            "--" => {
                if !interp {
                    return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
                }
                // argv[0] is the source file, like a built program's executable path.
                cli.program_args.push(file.clone().unwrap_or_default());
                for arg in rest.by_ref() {
                    cli.program_args.push(text(arg)?.to_owned());
                }
                continue;
            }
            _ => {}
        }
        // A plugin takes every argument up to the next `+Name`, `-` or `--`.
        if let Some(name) = arg.strip_prefix('+').filter(|name| !name.is_empty()) {
            cli.plugins.push(Plugin::parse(name));
            continue;
        }
        if let Some(plugin) = cli.plugins.last_mut() {
            plugin.options.push(arg.to_owned());
            continue;
        }
        let (a, inline) = jaic_cli_args::arguments::split_option(arg);
        let Some(option) = CliOption::find(a) else {
            if a.starts_with('-') {
                return Err(unknown_option(a));
            }
            if file.is_none() {
                file = Some(a.to_string());
            } else {
                extra.get_or_insert_with(|| a.to_string());
            }
            continue;
        };
        match option.action {
            Action::Help => return Ok(Request::Help),
            Action::Version => return Ok(Request::Version),
            Action::PrintStdlib => return Ok(Request::Stdlib),
            Action::DocComments => doc_comments = true,
            Action::DocGaps => doc_gaps = true,
            Action::DocModel => doc_model = true,
            Action::Procedures => procedures = true,
            Action::Check => check = true,
            Action::Interp => interp = true,
            Action::Deps => cli.deps = true,
            Action::Switch(switch) => *switch(&mut cli) = true,
            Action::Ignore => {}
            Action::Shared(flag) => {
                let next = if inline.is_none() {
                    next_text(&mut rest)?
                } else {
                    None
                };
                let shared = flag.take(a, inline, || next)?;
                cli.take_shared(shared);
            }
            Action::Value {
                what,
                store,
            } => {
                let value = match inline {
                    Some(value) => Some(value),
                    None => next_text(&mut rest)?,
                };
                let Some(value) = value else {
                    return Err(CliError::new(format!("`{a}` needs {what}")));
                };
                store(&mut cli, a, value)?;
            }
            Action::Passthrough {
                what,
                store,
            } => {
                let Some(value) = inline.map(OsString::from).or_else(|| rest.next().cloned())
                else {
                    return Err(CliError::new(format!("`{a}` needs {what}")));
                };
                store(&mut cli, value);
            }
            Action::OptLevel(level) => cli.opt_level = Some(level),
            Action::Flavor(flavor) => {
                if cli.flavor.is_some_and(|f| f != flavor) {
                    return Err(CliError::new(
                        "`-o`, `-od` and `-very_debug` each pick a build flavor; use one",
                    ));
                }
                cli.flavor = Some(flavor);
            }
        }
    }
    let Some(file) = file else {
        return Err(no_file_error());
    };
    if doc_comments {
        return Ok(Request::DocComments(
            file,
            jaic_cli_args::arguments::import_dirs(args),
        ));
    }
    if doc_gaps {
        return Ok(Request::DocGaps(
            file,
            jaic_cli_args::arguments::import_dirs(args),
        ));
    }
    if doc_model {
        return Ok(Request::DocModel(
            file,
            jaic_cli_args::arguments::import_dirs(args),
        ));
    }
    if procedures {
        return Ok(Request::Procedures(file, cli.os, cli.cpu));
    }
    cli.file = file;
    cli.levels =
        jaic::diagnostics::Levels::parse(&cli.level_flags).map_err(|message| {
            match message.split_once("\nhelp: ") {
                Some((message, help)) => CliError::new(message).help(help),
                None => CliError::new(message),
            }
        })?;
    cli.mode = Mode::from_flags(check, interp, cli.deps)?;
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
    // An argument that is not UTF-8 is never a colour option; it stays in place, as `?`s, so a
    // spaced option value after it is still read as one.
    let own: Vec<String> = args
        .iter()
        .map(|a| a.to_string_lossy().into_owned())
        .take_while(|a| a != "-" && a != "--" && !a.starts_with('+'))
        .collect();
    if own.iter().any(|a| a == "-no_color") {
        return jaic::render::ColorChoice::Never;
    }
    jaic::render::ColorChoice::from_args(&own, jaic::flags::COLOR.names)
}
