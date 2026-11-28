//! `jailint`'s command line: its own flags are declared here, and the ones it shares with `jaic`
//! (`-os`, `-I`, `-j`, ...) come from `jaic_cli_args::shared::SharedFlags`.
use std::path::PathBuf;

use argdef::{Args as _, Error, ErrorKind};
use jaic_cli_args::shared::SharedFlags;
use jailint::RULES;
use jailint::config::Level;

/// What the command line asked for.
#[derive(Default)]
pub(crate) struct Args {
    pub default_project: bool,
    pub context: Option<PathBuf>,
    pub manifest: Option<PathBuf>,
    pub checked_files: Option<PathBuf>,
    pub allow_incomplete: bool,
    pub paths: Vec<PathBuf>,
    pub fix: bool,
    pub config: Option<PathBuf>,
    pub levels: Vec<(String, Level)>,
    pub max_nesting: Option<usize>,
    pub lifetime_mode: Option<jacket_config::LifetimeMode>,
    pub verbose: bool,
    /// `--compile-time-budget`: blocks one compile-time evaluation may run (0: no limit).
    pub compile_time_budget: Option<u64>,
    /// `--check-time-limit`: seconds one program's check may take (0: no limit).
    pub check_time_limit: Option<f64>,
    /// The flags shared with `jaic`: target, import folders, added source, jobs, colour.
    pub shared: SharedFlags,
}

impl Args {
    /// The limits the flags set, over the defaults.
    pub(crate) fn limits(&self) -> jaic::ide::lint_compile::CheckLimits {
        use jaic::ide::lint_compile::CheckLimits;
        CheckLimits::new(
            self.compile_time_budget
                .unwrap_or(CheckLimits::DEFAULT_EVAL_STEPS),
            // A lint run is a batch: a wall-clock limit would make what it reports depend on the
            // load of the machine, so only a flag or the environment sets one (the evaluation
            // budget bounds a runaway `#run` by steps).
            self.check_time_limit
                .unwrap_or_else(CheckLimits::env_seconds),
        )
    }
}

/// A numeric value whose error names the flag and shows one that works.
fn number<T: std::str::FromStr>(
    flag: &str,
    text: &str,
    kind: &str,
    example: &str,
) -> Result<T, Error> {
    text.parse()
        .map_err(|_| Error::invalid(format!("`{flag}` takes {kind}, such as `{flag} {example}`")))
}

fn read_nesting(flag: &str, text: &str) -> Result<usize, Error> {
    number(flag, text, "a whole number", "6")
}

fn read_budget(flag: &str, text: &str) -> Result<u64, Error> {
    number(flag, text, "a whole number", "50000000")
}

fn read_seconds(flag: &str, text: &str) -> Result<f64, Error> {
    number(flag, text, "a number of seconds", "30")
}

fn read_lifetime_mode(flag: &str, text: &str) -> Result<jacket_config::LifetimeMode, Error> {
    text.parse().map_err(|reason: String| {
        Error::invalid(format!("`{flag}` takes must or strict: {reason}"))
    })
}

fn allow(own: &mut Own, _: &str, rule: &str) -> Result<(), Error> {
    own.levels.push((rule.to_string(), Level::Allow));
    Ok(())
}

fn warn(own: &mut Own, _: &str, rule: &str) -> Result<(), Error> {
    own.levels.push((rule.to_string(), Level::Warn));
    Ok(())
}

fn deny(own: &mut Own, _: &str, rule: &str) -> Result<(), Error> {
    own.levels.push((rule.to_string(), Level::Deny));
    Ok(())
}

/// Lints the current project's inclusion graph, or the files under explicit paths.
// The doc comments are `-help` text: names and variables stay unquoted.
#[allow(clippy::doc_markdown, clippy::duplicated_attributes)]
#[derive(Default, argdef::Args)]
#[args(
    name = "jailint",
    version = env!("CARGO_PKG_VERSION"),
    usage = "jailint [options] [paths...]",
    sections = ["Options", "Program", "Performance", "Diagnostics and output"],
)]
struct Own {
    /// Apply the fixes that are safe to apply automatically
    #[arg(name = "--fix", section = "Options")]
    fix: bool,
    /// Settings file (default: the nearest jacket.toml)
    #[arg(name = "--config", value = "file", needs = "a file", hint = "file")]
    config: Option<PathBuf>,
    /// Allow a rule or a compiler warning (repeatable; `all` for every rule)
    #[arg(name = "-A", value = "rule", needs = "a rule", with = allow, attached)]
    #[arg(name = "-W", value = "rule", needs = "a rule", with = warn, attached, help = "warn on a rule or a compiler warning")]
    #[arg(name = "-D", value = "rule", needs = "a rule", with = deny, attached, help = "deny a rule; `-D warnings` makes every warning an error")]
    levels: Vec<(String, Level)>,
    /// Blocks `excessive_nesting` allows (default: `[lint] max-nesting` in `jacket.toml`, else 6)
    #[arg(name = "--max-nesting", value = "n", needs = "a whole number", parse = read_nesting)]
    max_nesting: Option<usize>,
    /// Lifetime proof mode (default: [lint.lifetimes] mode, else must)
    #[arg(name = "--lifetime-mode", value = "must|strict", needs = "must or strict", parse = read_lifetime_mode)]
    lifetime_mode: Option<jacket_config::LifetimeMode>,
    /// Steps (loop iterations and calls) one compile-time evaluation may run before it fails (default: 100000000; 0:
    /// no limit)
    #[arg(name = "--compile-time-budget", value = "steps", needs = "a number of steps", parse = read_budget)]
    compile_time_budget: Option<u64>,
    /// Seconds each program may take to check before its check stops with partial results;
    /// every program gets the whole limit, and the run as a whole has none (default: 10; 0: no
    /// limit)
    #[arg(name = "--check-time-limit", value = "seconds", needs = "a number of seconds", parse = read_seconds)]
    check_time_limit: Option<f64>,
    /// Write JSON source-check coverage and actual target contexts
    #[arg(name = "--manifest", value = "file", needs = "a file", hint = "file")]
    manifest: Option<PathBuf>,
    /// Check an enclosing program; report only the requested paths
    #[arg(
        name = "--context",
        value = "program file",
        needs = "a program file",
        hint = "file"
    )]
    context: Option<PathBuf>,
    /// Write the actual checked paths, one per line
    #[arg(
        name = "--checked-files",
        value = "file",
        needs = "a file",
        hint = "file"
    )]
    checked_files: Option<PathBuf>,
    /// Count programs that do not compile on this target without failing the run
    #[arg(name = "--allow-incomplete")]
    allow_incomplete: bool,
    /// List the rules and their default levels
    #[arg(name = "--list")]
    list: bool,
    /// Show each program as it is compiled
    #[arg(name = "-v", name = "--verbose", synopsis = "-v, --verbose")]
    verbose: bool,
    /// print this help
    #[arg(name = "-h", name = "--help", synopsis = "-h, --help", exit)]
    help: bool,
    /// print the version
    #[arg(name = "-V", name = "--version", synopsis = "-V, --version", exit)]
    version: bool,
    /// The options `jaic` takes as well: the target, the import folders, `-add`, `-j` and
    /// `--color`.
    #[arg(flatten)]
    shared: SharedFlags,
    /// Files or folders to scan explicitly; without paths, use the project's entrypoints
    #[arg(positional)]
    paths: Vec<String>,
}

/// The help page: the options by section, then the exit status.
fn help() -> String {
    Own::spec()
        .help()
        .footer(
            "Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a\n\
             usage or configuration error.",
        )
        .render(false)
}

/// The text of a command-line mistake in this tool's style: the message, then `help:` lines.
fn describe(error: Error) -> String {
    let mut text = error.message;
    let mut help = error.help;

    if error.kind == ErrorKind::Unknown {
        let spec = Own::spec();
        let known = spec
            .opts
            .iter()
            .filter(|opt| opt.listed())
            .flat_map(|opt| opt.names.iter().copied());

        help.push(match jaic::suggest::closest(&error.subject, known) {
            Some(near) => format!("did you mean `{near}`? `jailint --help` lists the options"),
            None => "`jailint --help` lists the options".to_string(),
        });
    }

    for line in help {
        text.push_str("\nhelp: ");
        text.push_str(&line);
    }

    text
}

/// Parse `argv` (without the program name). `None` when a flag answered by itself.
pub(crate) fn parse(argv: impl IntoIterator<Item = String>) -> Result<Option<Args>, String> {
    let argv: Vec<String> = argv.into_iter().collect();
    let own = Own::parse(&argv).map_err(describe)?;

    if own.help {
        print!("{}", help());
        return Ok(None);
    }

    if own.version {
        println!("jailint {}", env!("CARGO_PKG_VERSION"));
        return Ok(None);
    }

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

    let default_project = own.paths.is_empty() && own.context.is_none();
    let mut paths: Vec<PathBuf> = own.paths.into_iter().map(PathBuf::from).collect();

    if paths.is_empty() {
        paths.push(".".into());
    }

    Ok(Some(Args {
        default_project,
        context: own.context,
        manifest: own.manifest,
        checked_files: own.checked_files,
        allow_incomplete: own.allow_incomplete,
        paths,
        fix: own.fix,
        config: own.config,
        levels: own.levels,
        max_nesting: own.max_nesting,
        lifetime_mode: own.lifetime_mode,
        verbose: own.verbose,
        compile_time_budget: own.compile_time_budget,
        check_time_limit: own.check_time_limit,
        shared: own.shared,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(args: &[&str]) -> Result<Option<Args>, String> {
        parse(args.iter().map(ToString::to_string))
    }

    #[test]
    fn lifetime_mode_is_validated() {
        let args = parse_all(&["--lifetime-mode", "strict"]).unwrap().unwrap();

        assert_eq!(
            args.lifetime_mode,
            Some(jacket_config::LifetimeMode::Strict)
        );
        assert!(parse_all(&["--lifetime-mode", "optimistic"]).is_err());
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

        assert_eq!(args.shared.os, Some(jaic::target::TargetOs::Linux));
        assert_eq!(args.shared.jobs(), Some(3));
        assert_eq!(args.shared.color, Some(jaic::render::ColorChoice::Never));
        assert_eq!(args.shared.imports, [PathBuf::from("libs")]);
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
        assert_eq!(args.shared.jobs(), None);
    }

    #[test]
    fn a_lint_run_has_no_wall_clock_limit_unless_asked() {
        let args = parse_all(&["a.jai"]).unwrap().unwrap();

        if jacket_config::env::CHECK_TIME_LIMIT.get().is_none() {
            assert_eq!(args.limits().seconds, None);
        }

        let args = parse_all(&["--check-time-limit", "3", "a.jai"])
            .unwrap()
            .unwrap();

        assert_eq!(args.limits().seconds, Some(3.0));
    }

    #[test]
    fn the_help_lists_the_shared_flags() {
        let text = help();

        for flag in [
            "-os <os>",
            "-j, --jobs <n>",
            "--color <when>",
            "-add \"code\"",
        ] {
            assert!(text.contains(flag), "{flag}\n{text}");
        }

        assert!(text.contains("--compile-time-budget"), "{text}");
        assert!(text.contains("Exit status"), "{text}");
    }
}
