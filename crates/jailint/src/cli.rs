//! `jailint`'s command line: its own flags are a `clap` derive, and the flags it shares with the
//! other tools (`-os`, `-I`, `-j`, ...; single-dash words clap cannot spell) are read from
//! `jaic::flags` before clap sees the rest.
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use jaic::flags::{self, Common, FlagError, Shared};
use jailint::RULES;
use jailint::config::Level;
use std::path::PathBuf;

/// What the command line asked for.
#[derive(Default)]
pub struct Args {
    pub context: Option<PathBuf>,
    pub manifest: Option<PathBuf>,
    pub checked_files: Option<PathBuf>,
    pub allow_incomplete: bool,
    pub paths: Vec<PathBuf>,
    pub fix: bool,
    pub config: Option<PathBuf>,
    pub levels: Vec<(String, Level)>,
    pub max_nesting: Option<usize>,
    pub verbose: bool,
    /// `--compile-time-budget`: blocks one compile-time evaluation may run (0: no limit).
    pub compile_time_budget: Option<u64>,
    /// `--check-time-limit`: seconds one program's check may take (0: no limit).
    pub check_time_limit: Option<f64>,
    /// The flags shared with `jaic`: target, import folders, added source, jobs, colour.
    pub common: Common,
}

impl Args {
    /// The limits the flags set, over the defaults.
    pub fn limits(&self) -> jaic::ide::lint_compile::CheckLimits {
        use jaic::ide::lint_compile::CheckLimits;
        CheckLimits::new(
            self.compile_time_budget
                .unwrap_or(CheckLimits::DEFAULT_EVAL_STEPS),
            self.check_time_limit
                .unwrap_or_else(CheckLimits::default_seconds),
        )
    }
}

/// A parser for a numeric flag whose error names the flag and shows a value that works.
fn number<T: std::str::FromStr>(
    flag: &'static str,
    kind: &'static str,
    example: &'static str,
) -> impl Fn(&str) -> Result<T, String> + Clone + Send + Sync + 'static {
    move |text| {
        text.parse()
            .map_err(|_| format!("`{flag}` takes {kind}, such as `{flag} {example}`"))
    }
}

/// jailint's own flags.
#[derive(Parser)]
#[command(
    name = "jailint",
    version,
    about = "Lints the Jai files under each path (default: the current directory).",
    color = clap::ColorChoice::Never,
    disable_help_subcommand = true
)]
struct Own {
    /// Files, or folders that hold them (default: the current directory)
    #[arg(value_name = "paths")]
    paths: Vec<PathBuf>,
    /// Apply the fixes that are safe to apply automatically
    #[arg(long)]
    fix: bool,
    /// Settings file (default: the nearest jacket.toml)
    #[arg(long, value_name = "file")]
    config: Option<PathBuf>,
    /// Allow a rule (repeatable; `all` for every rule)
    #[arg(short = 'A', value_name = "rule")]
    allow: Vec<String>,
    /// Warn on a rule
    #[arg(short = 'W', value_name = "rule")]
    warn: Vec<String>,
    /// Deny a rule; `-D warnings` makes every warning an error
    #[arg(short = 'D', value_name = "rule")]
    deny: Vec<String>,
    /// Blocks `excessive_nesting` allows (default: `[lint] max_nesting` in `jacket.toml`, else 6)
    #[arg(long, value_name = "n", value_parser = number::<usize>("--max-nesting", "a whole number", "6"))]
    max_nesting: Option<usize>,
    /// Steps (loop iterations and calls) one compile-time evaluation may run before it fails (default: 100000000; 0:
    /// no limit)
    #[arg(long, value_name = "steps", value_parser = number::<u64>("--compile-time-budget", "a whole number", "50000000"))]
    compile_time_budget: Option<u64>,
    /// Seconds each program may take to check before its check stops with partial results;
    /// every program gets the whole limit, and the run as a whole has none (default: 10; 0: no
    /// limit)
    #[arg(long, value_name = "seconds", value_parser = number::<f64>("--check-time-limit", "a number of seconds", "30"))]
    check_time_limit: Option<f64>,
    /// Write JSON source-check coverage and actual target contexts
    #[arg(long, value_name = "file")]
    manifest: Option<PathBuf>,
    /// Check an enclosing program; report only the requested paths
    #[arg(long, value_name = "program file")]
    context: Option<PathBuf>,
    /// Write the actual checked paths, one per line
    #[arg(long, value_name = "file")]
    checked_files: Option<PathBuf>,
    /// Count programs that do not compile on this target without failing the run
    #[arg(long)]
    allow_incomplete: bool,
    /// List the rules and their default levels
    #[arg(long)]
    list: bool,
    /// Show each program as it is compiled
    #[arg(short, long)]
    verbose: bool,
}

/// The help text after the options: the flags shared with `jaic`, and the exit status.
fn trailer() -> String {
    let mut text = String::from("Options shared with jaic:\n");
    for flag in flags::ALL {
        text.push_str(&format!("  {:<30}{}\n", flag.synopsis, flag.help));
    }
    text.push_str(
        "\nExit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a\n\
         usage or configuration error.",
    );
    text
}

fn command() -> clap::Command {
    // An option takes the next word whatever it looks like, as the hand-written parser did.
    Own::command().after_help(trailer()).mut_args(|arg| {
        if arg.is_positional() || !arg.get_action().takes_values() {
            arg
        } else {
            arg.allow_hyphen_values(true)
        }
    })
}

/// The flags of jailint's own that take a value, as typed (`-A`, `--config`): the token after
/// one belongs to it, even when it is spelt like a shared flag.
fn value_flags(command: &clap::Command) -> Vec<String> {
    let mut spellings = Vec::new();
    for arg in command
        .get_arguments()
        .filter(|a| a.get_action().takes_values())
    {
        spellings.extend(arg.get_short().map(|c| format!("-{c}")));
        spellings.extend(arg.get_long().map(|l| format!("--{l}")));
    }
    spellings
}

/// The flags from `jaic::flags` out of `argv`, and everything else in order.
fn take_shared(
    argv: impl IntoIterator<Item = String>,
    value_flags: &[String],
) -> Result<(Common, Vec<String>), String> {
    let mut common = Common::default();
    let mut rest = Vec::new();
    let mut it = argv.into_iter();
    let mut value_next = false;
    while let Some(a) = it.next() {
        if std::mem::take(&mut value_next) {
            rest.push(a);
        } else if a == "--" {
            rest.push(a);
            rest.extend(it.by_ref());
        } else if let Some((flag, inline)) = flags::find(flags::ALL, &a) {
            let shared: Shared = flag
                .take(&a, inline, || it.next())
                .map_err(FlagError::into_text)?;
            common.take(shared);
        } else {
            value_next = value_flags.contains(&a);
            rest.push(a);
        }
    }
    Ok((common, rest))
}

fn unknown(flag: &str, command: &clap::Command) -> FlagError {
    let own = command.get_arguments().flat_map(|a| {
        let short = a.get_short().map(|c| format!("-{c}"));
        let long = a.get_long().map(|l| format!("--{l}"));
        short.into_iter().chain(long)
    });
    let known: Vec<String> = own
        .chain(flags::spellings(flags::ALL).map(String::from))
        .collect();
    let help = match jaic::suggest::closest(flag, known.iter().map(String::as_str)) {
        Some(near) => format!("did you mean `{near}`? `jailint --help` lists the options"),
        None => "`jailint --help` lists the options".to_string(),
    };
    FlagError::new(format!("unknown option `{flag}`")).help(help)
}

/// The text of a clap error in this tool's style: clap's first line, without its usage block.
fn describe(error: &clap::Error, command: &clap::Command) -> String {
    let argument = match error.get(ContextKind::InvalidArg) {
        Some(ContextValue::String(arg)) => Some(arg.clone()),
        _ => None,
    };
    if error.kind() == ErrorKind::ValueValidation
        && let Some(reason) = std::error::Error::source(error)
    {
        return reason.to_string();
    }
    match (error.kind(), argument) {
        (ErrorKind::UnknownArgument, Some(arg)) => return unknown(&arg, command).into_text(),
        (ErrorKind::InvalidValue, Some(arg))
            if error.to_string().contains("a value is required") =>
        {
            let (flag, value) = arg.split_once(" <").unwrap_or((&arg, ""));
            let what = match value.trim_end_matches('>') {
                "n" => "a whole number",
                "blocks" => "a number of blocks",
                "seconds" => "a number of seconds",
                other => &format!("a {other}"),
            };
            return format!("`{flag}` needs {what}");
        }
        _ => {}
    }
    let text = error.render().to_string();
    let first = text.lines().next().unwrap_or("");
    let message = first.strip_prefix("error: ").unwrap_or(first);
    format!("{message}\nhelp: `jailint --help` lists the options")
}

/// The levels the `-A`, `-W` and `-D` flags set, in the order they were written (a later flag
/// wins).
fn levels(matches: &ArgMatches) -> Vec<(String, Level)> {
    let mut found = Vec::new();
    for (id, level) in [
        ("allow", Level::Allow),
        ("warn", Level::Warn),
        ("deny", Level::Deny),
    ] {
        let rules = matches.get_many::<String>(id).into_iter().flatten();
        let positions = matches.indices_of(id).into_iter().flatten();
        for (position, rule) in positions.zip(rules) {
            found.push((position, rule.clone(), level));
        }
    }
    found.sort_by_key(|(position, ..)| *position);
    found
        .into_iter()
        .map(|(_, rule, level)| (rule, level))
        .collect()
}

/// Parse `argv` (without the program name). `None` when a flag answered by itself.
pub fn parse(argv: impl IntoIterator<Item = String>) -> Result<Option<Args>, String> {
    let command = command();
    let (common, rest) = take_shared(argv, &value_flags(&command))?;
    let argv = std::iter::once("jailint".to_string()).chain(rest);
    let matches = match command.clone().try_get_matches_from(argv) {
        Ok(matches) => matches,
        Err(error) if error.use_stderr() => return Err(describe(&error, &command)),
        Err(error) => {
            // `--help` and `--version` answer on stdout.
            print!("{}", error.render());
            return Ok(None);
        }
    };
    let own = Own::from_arg_matches(&matches).map_err(|e| describe(&e, &command))?;
    if own.list {
        for r in RULES {
            println!(
                "{:<26} {:<11} {:<5}  {}",
                r.name,
                r.category.as_str(),
                r.default().as_str(),
                r.summary
            );
        }
        return Ok(None);
    }
    let mut paths = own.paths;
    if paths.is_empty() {
        paths.push(".".into());
    }
    Ok(Some(Args {
        context: own.context,
        manifest: own.manifest,
        checked_files: own.checked_files,
        allow_incomplete: own.allow_incomplete,
        paths,
        fix: own.fix,
        config: own.config,
        levels: levels(&matches),
        max_nesting: own.max_nesting,
        verbose: own.verbose,
        compile_time_budget: own.compile_time_budget,
        check_time_limit: own.check_time_limit,
        common,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(args: &[&str]) -> Result<Option<Args>, String> {
        parse(args.iter().map(ToString::to_string))
    }

    #[test]
    fn shared_and_own_flags_fill_the_arguments() {
        let args = parse_all(&[
            "-os",
            "linux",
            "-j",
            "3",
            "--color=never",
            "-I",
            "libs",
            "-D",
            "warnings",
            "--fix",
            "a.jai",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(args.common.target.os, Some(jaic::target::TargetOs::Linux));
        assert_eq!(args.common.jobs, Some(3));
        assert_eq!(args.common.color, Some(jaic::render::ColorChoice::Never));
        assert_eq!(args.common.imports, [PathBuf::from("libs")]);
        assert_eq!(args.levels, [("warnings".to_string(), Level::Deny)]);
        assert!(args.fix);
        assert_eq!(args.paths, [PathBuf::from("a.jai")]);
    }

    #[test]
    fn a_missing_value_and_a_mistyped_flag_are_reported() {
        let error = parse_all(&["-os"]).err().unwrap();
        assert!(error.contains("needs an OS name"), "{error}");
        let error = parse_all(&["--contxt", "x"]).err().unwrap();
        assert!(error.contains("did you mean `--context`"), "{error}");
        let error = parse_all(&["--manifest"]).err().unwrap();
        assert!(error.contains("`--manifest` needs a file"), "{error}");
    }

    #[test]
    fn levels_keep_the_order_they_were_written_in() {
        let args = parse_all(&["-D", "warnings", "-A", "magic_number", "-Wpedantic"])
            .unwrap()
            .unwrap();
        assert_eq!(
            args.levels,
            [
                ("warnings".to_string(), Level::Deny),
                ("magic_number".to_string(), Level::Allow),
                ("pedantic".to_string(), Level::Warn),
            ]
        );
    }

    #[test]
    fn a_value_that_looks_like_a_shared_flag_belongs_to_its_flag() {
        let args = parse_all(&["--config", "-j", "a.jai"]).unwrap().unwrap();
        assert_eq!(args.config, Some(PathBuf::from("-j")));
        assert_eq!(args.common.jobs, None);
    }

    #[test]
    fn the_help_lists_the_shared_flags() {
        let text = command().render_long_help().to_string();
        for flag in flags::ALL {
            assert!(text.contains(flag.synopsis), "{}", flag.synopsis);
        }
        assert!(text.contains("--compile-time-budget"), "{text}");
    }
}
