//! `jailint`: report (and fix) common mistakes and unidiomatic code in Jai programs.
use jailint::config::{Config, Level};
use jailint::render::{Style, render};
use jailint::{Lint, RULES};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
usage: jailint [options] [paths...]

Lints the Jai files under each path (default: the current directory).

options:
  --fix               apply the fixes that are safe to apply automatically
  --config <file>     settings file (default: the nearest jailint.toml)
  -A <rule>           allow a rule (repeatable; `all` for every rule)
  -W <rule>           warn on a rule
  -D <rule>           deny a rule; `-D warnings` makes every warning an error
  -I <dir>            add an import directory (also -import_dir)
  -j <n>              compile n programs at once (default 1)
  --list              list the rules and their default levels
  --color <when>      auto, always or never
  -v, --verbose       show each program as it is compiled
  -h, --help          show this help

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage
or configuration error.";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("jailint: {message}");
            ExitCode::from(2)
        }
    }
}

struct Args {
    paths: Vec<PathBuf>,
    fix: bool,
    config: Option<PathBuf>,
    levels: Vec<(String, Level)>,
    imports: Vec<PathBuf>,
    jobs: usize,
    color: Option<bool>,
    verbose: bool,
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut args = Args {
        paths: Vec::new(),
        fix: false,
        config: None,
        levels: Vec::new(),
        imports: Vec::new(),
        jobs: 1,
        color: None,
        verbose: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "--list" => {
                for r in RULES {
                    println!("{:<26} {:<5}  {}", r.name, r.default.as_str(), r.summary);
                }
                return Ok(None);
            }
            "--fix" => args.fix = true,
            "-v" | "--verbose" => args.verbose = true,
            "--config" => args.config = Some(PathBuf::from(value("--config")?)),
            "-I" | "-import_dir" | "--import-dir" => args.imports.push(value(&a)?.into()),
            "-j" | "--jobs" => {
                args.jobs = value(&a)?
                    .parse()
                    .map_err(|_| format!("{a} takes a number"))?
            }
            "--color" => {
                args.color = match value("--color")?.as_str() {
                    "always" => Some(true),
                    "never" => Some(false),
                    "auto" => None,
                    other => return Err(format!("--color: unknown value `{other}`")),
                }
            }
            "-A" | "-W" | "-D" => {
                let level = match a.as_str() {
                    "-A" => Level::Allow,
                    "-W" => Level::Warn,
                    _ => Level::Deny,
                };
                args.levels.push((value(&a)?, level));
            }
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(format!("unknown option `{flag}` (see --help)"));
            }
            path => args.paths.push(path.into()),
        }
    }
    if args.paths.is_empty() {
        args.paths.push(".".into());
    }
    Ok(Some(args))
}

fn run() -> Result<ExitCode, String> {
    let Some(args) = parse_args()? else {
        return Ok(ExitCode::SUCCESS);
    };
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let config_path = match &args.config {
        Some(p) => Some(p.clone()),
        None => {
            let first = args.paths[0].canonicalize().unwrap_or(cwd.clone());
            let start = if first.is_dir() {
                first
            } else {
                first.parent().map(PathBuf::from).unwrap_or(cwd.clone())
            };
            Config::find(&start)
        }
    };
    let mut config = match config_path {
        Some(p) => Config::load(&p)?,
        None => Config {
            root: cwd.clone(),
            ..Config::default()
        },
    };
    for (rule, level) in &args.levels {
        if rule == "warnings" && *level == Level::Deny {
            config.deny_warnings = true;
        } else if rule == "all" {
            for r in RULES {
                config.levels.insert(r.name.to_string(), *level);
            }
        } else if RULES.iter().any(|r| r.name == rule) {
            config.levels.insert(rule.clone(), *level);
        } else {
            return Err(format!("unknown rule `{rule}` (see --list)"));
        }
    }
    let stdlib = jaic::stdlib_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        return Err(message);
    }
    let stdlib = stdlib.canonicalize().unwrap_or(stdlib);
    let options = jailint::driver::Options {
        paths: args.paths.clone(),
        import_dirs: args.imports.clone(),
        stdlib,
        config,
        jobs: args.jobs,
        verbose: args.verbose,
    };
    let outcome = jailint::driver::run(&options);
    let style = Style {
        color: args.color.unwrap_or_else(|| {
            std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
        }),
    };
    let shown = |p: &str| {
        let path = PathBuf::from(p);
        path.strip_prefix(&cwd)
            .map(|r| r.to_string_lossy().into_owned())
            .unwrap_or_else(|_| p.to_string())
    };
    let mut errors = 0;
    let mut warnings = 0;
    for lint in &outcome.lints {
        let Some(text) = outcome.texts.get(&lint.path) else {
            continue;
        };
        let default = RULES
            .iter()
            .find(|r| r.name == lint.rule)
            .map_or(Level::Warn, |r| r.default);
        println!(
            "{}",
            render(lint, text, &shown(&lint.path), default, &style)
        );
        match lint.level {
            Level::Deny => errors += 1,
            _ => warnings += 1,
        }
    }
    if args.verbose {
        for (root, error) in &outcome.incomplete {
            eprintln!(
                "jailint: {} did not compile completely ({error}); bodies that failed were not linted",
                shown(&root.to_string_lossy())
            );
        }
    }
    if args.fix {
        let mut by_file: std::collections::BTreeMap<&str, Vec<&Lint>> = Default::default();
        for l in &outcome.lints {
            by_file.entry(&l.path).or_default().push(l);
        }
        let mut total = 0;
        for (path, lints) in by_file {
            let Some(text) = outcome.texts.get(path) else {
                continue;
            };
            let (fixed, n) = jailint::fix::apply(text, &lints);
            if n > 0 {
                std::fs::write(path, fixed).map_err(|e| format!("cannot write {path}: {e}"))?;
                total += n;
                eprintln!("jailint: fixed {n} in {}", shown(path));
            }
        }
        if total > 0 {
            eprintln!("jailint: applied {total} fixes; run again to see what remains");
        }
    }
    let files: std::collections::BTreeSet<&str> =
        outcome.lints.iter().map(|l| l.path.as_str()).collect();
    if errors + warnings > 0 {
        eprintln!(
            "jailint: {} and {} in {} ({} checked)",
            count(errors, "error"),
            count(warnings, "warning"),
            count(files.len(), "file"),
            count(outcome.roots, "program"),
        );
    }
    Ok(if errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn count(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}
