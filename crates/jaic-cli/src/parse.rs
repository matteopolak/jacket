//! Parsing arguments into a request and checking the input file.
use super::cli::{Cli, Mode, Plugin};
use super::options::{Action, CliError, CliOption, Request};
use super::usage::{no_file_error, program_args_error, unknown_option};
use std::path::Path;
use std::process::ExitCode;

pub(super) fn parse(args: &[String]) -> Result<Request, CliError> {
    if args.is_empty() {
        return Err(no_file_error());
    }
    let mut cli = Cli::default();
    let mut file: Option<String> = None;
    let mut extra: Option<String> = None;
    let (mut check, mut interp) = (false, false);
    // The first option that only builds take, for the `-check`/`-interp` error.
    let mut build_only: Option<String> = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
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
                    cli.command_line.push(arg.clone());
                }
                cli.program_args.extend(rest.by_ref().cloned());
                continue;
            }
            "--" => {
                if !interp {
                    return Err(program_args_error(file.as_deref().unwrap_or("file.jai")));
                }
                // argv[0] is the source file, like a built program's executable path.
                cli.program_args.push(file.clone().unwrap_or_default());
                cli.program_args.extend(rest.by_ref().cloned());
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
            plugin.options.push(arg.clone());
            continue;
        }
        // `-os=linux` and the like.
        let (a, inline) = match arg.split_once('=') {
            Some((option, value)) if CliOption::find_value(option).is_some() => {
                (option, Some(value.to_string()))
            }
            _ => (arg.as_str(), None),
        };
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
        if option.build_only {
            build_only.get_or_insert_with(|| a.to_string());
        }
        match option.action {
            Action::Help => return Ok(Request::Help),
            Action::Version => return Ok(Request::Version),
            Action::Check => check = true,
            Action::Interp => interp = true,
            Action::Switch(switch) => *switch(&mut cli) = true,
            Action::Ignore => {}
            Action::Value {
                what,
                store,
            } => {
                let Some(value) = inline.or_else(|| rest.next().cloned()) else {
                    return Err(CliError::new(format!("`{a}` needs {what}")));
                };
                store(&mut cli, a, value)?;
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
    cli.file = file;
    if check && interp {
        return Err(CliError::new("`-check` and `-interp` cannot be combined")
            .help("`-check` only compiles the program; `-interp` runs it in the interpreter"));
    }
    cli.mode = match (check, interp) {
        (true, _) => Mode::Check,
        (_, true) => Mode::Interp,
        _ => Mode::Build,
    };
    cli.finish(extra, build_only)?;
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
    let report = jaic::render::Report::from_text(jaic::render::Severity::Error, message);
    eprint!("{}", report.render());
}

/// Print a command-line mistake and return the usage exit status.
pub(super) fn report_cli_error(error: CliError) -> ExitCode {
    print_cli_error(error);
    ExitCode::from(2)
}

pub(super) fn print_cli_error(error: CliError) {
    let mut report = jaic::render::Report::new(jaic::render::Severity::Error, error.message);
    for help in error.help {
        report = report.help(help);
    }
    eprint!("{}", report.render());
}

/// Whether `-no_color` is among jaic's own arguments, read before parsing so that even
/// command-line mistakes are printed without colour. Plugin, metaprogram and program
/// arguments (from the first `+Name`, `-` or `--`) are not jaic's.
pub(super) fn wants_no_color(args: &[String]) -> bool {
    args.iter()
        .take_while(|a| *a != "-" && *a != "--" && !a.starts_with('+'))
        .any(|a| a == "-no_color")
}
