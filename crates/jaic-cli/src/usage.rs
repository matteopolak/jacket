//! Help text and command line error messages.
use super::options::{CliError, NO_MAIN_STATUS, Section};
use super::tuning;

/// The `-help` lines of the options in `section`.
fn option_help(section: Section) -> String {
    let mut lines = String::new();
    for help in tuning::all_options().filter_map(|option| option.help) {
        if help.section != section {
            continue;
        }
        let mut text = help.text.lines();
        let first = text.next().unwrap_or_default();
        lines.push_str(&format!("  {:<26}{first}\n", help.synopsis));
        for more in text {
            lines.push_str(&format!("{:28}{more}\n", ""));
        }
    }
    lines
}

/// `jaic -help`.
pub(super) fn usage_text() -> String {
    format!(
        "jaic {version}: compile, check and run Jai programs

usage: jaic [options] file.jai [options] [+Plugin [plugin options...]]... [- metaprogram args...] [-- program args...]

By default jaic builds a native executable. Options may come before or after the file.

modes:
{modes}
program:
{program}
output (building only):
{output}
plugins:
  +Name                     run the metaprogram plugin module Name. Every argument after it, up
                            to the next +Name, `-` or `--`, is that plugin's own option.
                            Module parameters go in the name: '+Check(CHECK_BINDINGS=false)'
                            (quote it in a shell). Plugins do not combine with -interp.
  -                         the rest is for the metaprogram (`compiler_get_command_line`)
  --                        the rest is for the program (-interp only)

diagnostics and output:
{diagnostics}
environment:
  JAIC_MEMORY_LIMIT=<bytes|nK|nM|nG>  stop with exit status {limit} once that much is allocated
  JAIC_DIAGNOSTICS=plain|ascii|unicode  diagnostic layout (default: by terminal)
  NO_COLOR, FORCE_COLOR, CLICOLOR_FORCE  turn colour off or on

exit status: 0 success, 1 the program failed to compile, an input file cannot be read
or a runtime error stopped it (`-interp` otherwise exits with the program's own status),
2 a command-line mistake, {no_main} a build found no `main` to make an executable from,
{limit} the memory limit, {crash} native code crashed under the interpreter, 101 an
internal compiler error",
        version = env!("CARGO_PKG_VERSION"),
        modes = option_help(Section::Modes),
        program = option_help(Section::Program),
        output = option_help(Section::Output),
        diagnostics = option_help(Section::Diagnostics),
        no_main = NO_MAIN_STATUS,
        limit = jaic::memory_limit::EXIT_CODE,
        crash = jaic::interp::CRASH_STATUS,
    )
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
    let known = tuning::all_options().flat_map(|option| option.names.iter().copied());
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
