//! Help text and command line error messages.
use argdef::{Args, Row};
use jaic_cli_args::cli::Cli;
use jaic_cli_args::options::{CliError, NO_MAIN_STATUS};

/// `jaic -help`, with bold headings and flags when `color`.
pub(super) fn usage_text(color: bool) -> String {
    let rows = |rows: &[(&str, String)]| {
        rows.iter()
            .map(|(label, text)| Row {
                label: (*label).to_string(),
                text: text.clone(),
            })
            .collect()
    };
    let environment = rows(&[
        (
            "JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>",
            format!(
                "stop with exit status {} once that much is allocated",
                jaic::memory_limit::EXIT_CODE
            ),
        ),
        (
            "JAIC_DIAGNOSTICS=plain|ascii|unicode",
            "diagnostic layout (default: by terminal)".into(),
        ),
        (
            "NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE",
            "turn colour off or on".into(),
        ),
    ]);

    Cli::spec()
        .help()
        .section("Environment", environment)
        .footer(format!(
            "Exit status: 0 success, 1 the program failed to compile, an input file cannot be read
or a runtime error stopped it (`-interp` otherwise exits with the program's own status),
2 a command-line mistake, {no_main} a build found no `main` to make an executable from,
{limit} the memory limit, {crash} native code crashed under the interpreter, 101 an
internal compiler error",
            no_main = NO_MAIN_STATUS,
            limit = jaic::memory_limit::EXIT_CODE,
            crash = jaic::interp::CRASH_STATUS,
        ))
        .render(color)
}

/// What is wrong with `--`, which only `-interp` takes.
pub(super) fn program_args_error(file: &str) -> CliError {
    CliError::new("`--` hands arguments to the program, which only `-interp` runs")
        .help(format!("use `jaic {file} -interp -- ...`"))
        .help("arguments for the metaprogram go after a lone `-`")
}

/// The error for an option jaic does not know: a bad `-O` level, or a near miss.
pub(super) fn unknown_option(option: &str) -> CliError {
    if option.starts_with("-O") {
        return CliError::new(format!("unknown optimization level `{option}`"))
            .help("use -O0, -O1, -O2 or -O3");
    }

    let mut error = CliError::new(format!("unknown option `{option}`"));
    let spec = Cli::spec();
    let known = spec
        .opts
        .iter()
        .filter(|opt| matches!(opt.kind, argdef::Kind::Flag | argdef::Kind::Value))
        .flat_map(|opt| opt.names.iter().copied());

    if let Some(near) = jaic::suggest::closest(option, known) {
        error = error.help(format!("did you mean `{near}`?"));
    }

    error.help("options after a `+Plugin` belong to that plugin; `jaic -help` lists jaic's own")
}

pub(super) fn no_file_error() -> CliError {
    CliError::new("no file to compile")
        .help("usage: jaic [options] file.jai [options]")
        .help("`jaic -help` lists the options")
}
