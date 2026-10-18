//! Help text and command line error messages.
use super::options::{CliError, CliOption, NO_MAIN_STATUS, Section};
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

/// Spellings from before 0.17.0 and what replaces them.
const RENAMED: &[(&str, &str)] = &[
    ("--emit-ir", "-emit_ir <file.ll>"),
    ("--no-debug-info", "-no_debug_info"),
    ("--timings", "-timings"),
    ("--sanitize", "-sanitize <list>"),
    ("--target", "-target <triple>"),
    (
        "--color",
        "-no_color to turn colour off; FORCE_COLOR=1 turns it on",
    ),
    (
        "-color",
        "-no_color to turn colour off; FORCE_COLOR=1 turns it on",
    ),
    ("-plug", "+Name after the file, as in `jaic file.jai +Name`"),
    (
        "-plugin",
        "+Name after the file, as in `jaic file.jai +Name`",
    ),
];

/// `arg` as a shell would need it written.
fn shell_word(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+,@%".contains(c));
    if plain {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// The error for `jaic run|check|build ...`: the same command in the single-command form, with
/// the other arguments carried over (renamed options included).
pub(super) fn subcommand_error(args: &[String]) -> CliError {
    let old = args[0].as_str();
    let mode_flag = match old {
        "run" => Some("-interp"),
        "check" => Some("-check"),
        _ => None,
    };
    let mut file = None;
    let mut options: Vec<String> = Vec::new();
    let mut plugins: Vec<String> = Vec::new();
    let mut plugin_options: Vec<String> = Vec::new();
    let mut tail: Vec<String> = Vec::new();
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        let arg = arg.as_str();
        match arg {
            "-" | "--" => {
                tail.push(arg.to_string());
                tail.extend(rest.by_ref().cloned());
            }
            "-plug" | "-plugin" => plugins.extend(rest.next().map(|name| format!("+{name}"))),
            "--color" | "-color" => {
                if rest.next().is_some_and(|when| when == "never") {
                    options.push("-no_color".into());
                }
            }
            "-o" => {
                if let Some(path) = rest.next() {
                    options.extend(["-exe".into(), path.clone()]);
                }
            }
            "--emit-ir" | "--sanitize" | "--target" => {
                options.push(arg[1..].replace("emit-ir", "emit_ir"));
                options.extend(rest.next().cloned());
            }
            "--no-debug-info" | "--timings" => {
                options.push(arg[1..].replace("no-debug-info", "no_debug_info"));
            }
            _ if CliOption::find_value(arg).is_some() => {
                options.push(arg.to_string());
                options.extend(rest.next().cloned());
            }
            _ if arg.starts_with('-') || !plugin_options.is_empty() => {
                plugin_options.push(arg.to_string());
            }
            _ if file.is_none() => file = Some(arg.to_string()),
            _ => plugin_options.push(arg.to_string()),
        }
    }
    let mut words = vec![file.unwrap_or_else(|| "file.jai".into())];
    words.extend(options);
    words.extend(mode_flag.map(String::from));
    words.extend(plugins);
    words.extend(plugin_options);
    words.extend(tail);
    let words: Vec<String> = words.iter().map(|w| shell_word(w)).collect();
    CliError::new("jaic has no subcommands since 0.17.0")
        .help(format!("use `jaic {}`", words.join(" ")))
        .help(
            "`-check` only compiles, `-interp` runs the program in the interpreter, and with \
             neither jaic builds an executable",
        )
}

/// What is wrong with `--`, which only `-interp` takes.
pub(super) fn program_args_error(file: &str) -> CliError {
    CliError::new("`--` hands arguments to the program, which only `-interp` runs")
        .help(format!("use `jaic {file} -interp -- ...`"))
        .help("arguments for the metaprogram go after a lone `-`")
}

/// The error for an option jaic does not know: renamed, a bad `-O` level, or a near miss.
pub(super) fn unknown_option(option: &str) -> CliError {
    let name = option.split_once('=').map_or(option, |(name, _)| name);
    if let Some((_, instead)) = RENAMED.iter().find(|(old, _)| *old == name) {
        return CliError::new(format!("`{name}` is spelled differently since 0.17.0"))
            .help(format!("use {instead}"));
    }
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
