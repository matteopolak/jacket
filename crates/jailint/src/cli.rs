//! `jailint`'s command line: its own flags in one table, the flags it shares with the other tools
//! from `jaic::flags`, and the usage text and "did you mean" suggestions both feed.
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
    /// The flags shared with `jaic`: target, import folders, added source, jobs, colour.
    pub common: Common,
}

/// What parsing a flag leads to.
enum Flow {
    Continue,
    /// The flag answered by itself (`--help`); the run is over.
    Done,
}

/// A flag of jailint's own.
struct Own {
    names: &'static [&'static str],
    /// What it takes; `None` for a switch.
    what: Option<&'static str>,
    synopsis: &'static str,
    help: &'static str,
    /// Called with the spelling used and the value (empty for a switch).
    apply: fn(&mut Args, &str, &str) -> Result<Flow, String>,
}

const OWN: &[Own] = &[
    Own {
        names: &["--fix"],
        what: None,
        synopsis: "--fix",
        help: "apply the fixes that are safe to apply automatically",
        apply: |args, _, _| {
            args.fix = true;
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--config"],
        what: Some("a file"),
        synopsis: "--config <file>",
        help: "settings file (default: the nearest jacket.toml)",
        apply: |args, _, value| {
            args.config = Some(value.into());
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["-A"],
        what: Some("a rule"),
        synopsis: "-A <rule>",
        help: "allow a rule (repeatable; `all` for every rule)",
        apply: |args, _, value| level(args, Level::Allow, value),
    },
    Own {
        names: &["-W"],
        what: Some("a rule"),
        synopsis: "-W <rule>",
        help: "warn on a rule",
        apply: |args, _, value| level(args, Level::Warn, value),
    },
    Own {
        names: &["-D"],
        what: Some("a rule"),
        synopsis: "-D <rule>",
        help: "deny a rule; `-D warnings` makes every warning an error",
        apply: |args, _, value| level(args, Level::Deny, value),
    },
    Own {
        names: &["--max-nesting"],
        what: Some("a number"),
        synopsis: "--max-nesting <n>",
        help: "blocks `excessive_nesting` allows (default: [lint] max_nesting in jacket.toml, else 6)",
        apply: |args, name, value| {
            args.max_nesting = Some(
                value
                    .parse()
                    .map_err(|_| format!("`{name}` takes a whole number, such as `{name} 6`"))?,
            );
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--manifest"],
        what: Some("a file"),
        synopsis: "--manifest <file>",
        help: "write JSON source-check coverage and actual target contexts",
        apply: |args, _, value| {
            args.manifest = Some(value.into());
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--context"],
        what: Some("a program file"),
        synopsis: "--context <file>",
        help: "check an enclosing program; report only the requested paths",
        apply: |args, _, value| {
            args.context = Some(value.into());
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--checked-files"],
        what: Some("a file"),
        synopsis: "--checked-files <file>",
        help: "write the actual checked paths, one per line",
        apply: |args, _, value| {
            args.checked_files = Some(value.into());
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--allow-incomplete"],
        what: None,
        synopsis: "--allow-incomplete",
        help: "count programs that do not compile on this target without failing the run",
        apply: |args, _, _| {
            args.allow_incomplete = true;
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["--list"],
        what: None,
        synopsis: "--list",
        help: "list the rules and their default levels",
        apply: |_, _, _| {
            for r in RULES {
                println!(
                    "{:<26} {:<11} {:<5}  {}",
                    r.name,
                    r.category.as_str(),
                    r.default().as_str(),
                    r.summary
                );
            }
            Ok(Flow::Done)
        },
    },
    Own {
        names: &["-v", "--verbose"],
        what: None,
        synopsis: "-v, --verbose",
        help: "show each program as it is compiled",
        apply: |args, _, _| {
            args.verbose = true;
            Ok(Flow::Continue)
        },
    },
    Own {
        names: &["-V", "--version"],
        what: None,
        synopsis: "-V, --version",
        help: "show the version",
        apply: |_, _, _| {
            println!("jailint {}", env!("CARGO_PKG_VERSION"));
            Ok(Flow::Done)
        },
    },
    Own {
        names: &["-h", "--help"],
        what: None,
        synopsis: "-h, --help",
        help: "show this help",
        apply: |_, _, _| {
            println!("{}", usage());
            Ok(Flow::Done)
        },
    },
];

fn level(args: &mut Args, level: Level, rule: &str) -> Result<Flow, String> {
    args.levels.push((rule.to_string(), level));
    Ok(Flow::Continue)
}

/// The usage text: built from the tables, so a flag cannot be listed wrongly.
pub fn usage() -> String {
    let mut text = String::from(
        "usage: jailint [options] [paths...]\n\n\
         Lints the Jai files under each path (default: the current directory).\n\noptions:\n",
    );
    let line = |text: &mut String, synopsis: &str, help: &str| {
        text.push_str(&format!("  {synopsis:<24}{help}\n"));
    };
    for flag in OWN.iter().filter(|f| f.what.is_some()) {
        line(&mut text, flag.synopsis, flag.help);
    }
    for flag in flags::ALL {
        line(&mut text, flag.synopsis, flag.help);
    }
    for flag in OWN.iter().filter(|f| f.what.is_none()) {
        line(&mut text, flag.synopsis, flag.help);
    }
    text.push_str(
        "\nExit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a\n\
         usage or configuration error.",
    );
    text
}

/// The own flag spelt `name`, with the text after `=` when it is written `name=value`.
fn find_own(name: &str) -> Option<(&'static Own, Option<String>)> {
    let named = |n: &str| OWN.iter().find(|flag| flag.names.contains(&n));
    if let Some(flag) = named(name) {
        return Some((flag, None));
    }
    let (n, value) = name.split_once('=')?;
    named(n)
        .filter(|flag| flag.what.is_some())
        .map(|flag| (flag, Some(value.to_string())))
}

fn unknown(flag: &str) -> FlagError {
    let own = OWN.iter().flat_map(|f| f.names.iter().copied());
    let known: Vec<&str> = own.chain(flags::spellings(flags::ALL)).collect();
    let help = match jaic::suggest::closest(flag, known) {
        Some(near) => format!("did you mean `{near}`? `jailint --help` lists the options"),
        None => "`jailint --help` lists the options".to_string(),
    };
    FlagError::new(format!("unknown option `{flag}`")).help(help)
}

/// Parse `argv` (without the program name). `None` when a flag answered by itself.
pub fn parse(argv: impl IntoIterator<Item = String>) -> Result<Option<Args>, String> {
    let mut args = Args::default();
    let mut it = argv.into_iter();
    while let Some(a) = it.next() {
        if let Some((flag, inline)) = find_own(&a) {
            let value = match flag.what {
                None => String::new(),
                Some(what) => inline
                    .or_else(|| it.next())
                    .ok_or_else(|| format!("`{a}` needs {what}"))?,
            };
            match (flag.apply)(&mut args, &a, &value)? {
                Flow::Continue => {}
                Flow::Done => return Ok(None),
            }
        } else if let Some((flag, inline)) = flags::find(flags::ALL, &a) {
            let shared: Shared = flag
                .take(&a, inline, || it.next())
                .map_err(FlagError::into_text)?;
            args.common.take(shared);
        } else if a.starts_with('-') && a.len() > 1 {
            return Err(unknown(&a).into_text());
        } else {
            args.paths.push(a.into());
        }
    }
    if args.paths.is_empty() {
        args.paths.push(".".into());
    }
    Ok(Some(args))
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
        assert_eq!(args.common.target.os, Some(jaic::sema::TargetOs::Linux));
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
        assert!(error.contains("needs a file"), "{error}");
    }

    #[test]
    fn the_usage_lists_every_flag() {
        let text = usage();
        for name in OWN
            .iter()
            .map(|f| f.names[0])
            .chain(flags::ALL.iter().map(|f| f.names[0]))
        {
            assert!(text.contains(name), "{name}");
        }
    }
}
