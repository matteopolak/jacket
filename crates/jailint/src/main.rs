//! `jailint`: report (and fix) common mistakes and unidiomatic code in Jai programs.
#![forbid(unsafe_code)]
mod cli;

use jailint::config::{Category, Config, Level};
use jailint::render::render;
use jailint::{Lint, RULES, RuleInfo};
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Errors about the command line or configuration go to stderr in its own style.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let color = jaic::render::ColorChoice::from_args(&args, jaic::flags::COLOR.names);
    jaic::render::set_style(jaic::render::detect(color));
    match run() {
        Ok(code) => code,
        Err(message) => {
            let report =
                jaic::render::Report::from_text(jaic::diagnostics::Code::CliFailure, &message);
            eprint!("{}", report.render());
            ExitCode::from(2)
        }
    }
}

/// Passes `--fix` makes at most: lint, apply what can go together, lint again.
const MAX_FIX_PASSES: usize = 5;

fn run() -> Result<ExitCode, String> {
    let Some(args) = cli::parse(std::env::args().skip(1))? else {
        return Ok(ExitCode::SUCCESS);
    };
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    if let Some(source) = &args.context {
        let metadata = std::fs::metadata(source).map_err(|e| {
            format!(
                "cannot read context `{}`: {}",
                source.display(),
                jaic::io_reason(&e)
            )
        })?;
        if !metadata.is_file() {
            return Err(format!(
                "context `{}` must be a program file",
                source.display()
            ));
        }
    }
    for path in &args.paths {
        if let Err(e) = std::fs::metadata(path) {
            let shown = path.display();
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("cannot read `{shown}`: {}", jaic::io_reason(&e)));
            }
            let help = match jaic::suggest::similar_sibling(path, |_| true) {
                Some(near) => format!("did you mean `{}`?", near.display()),
                None => {
                    "pass .jai files or directories that hold them (default: the current directory)"
                        .to_string()
                }
            };
            return Err(format!("`{shown}` does not exist\nhelp: {help}"));
        }
    }
    let mut config = if let Some(path) = &args.config {
        Config::load(path)?
    } else {
        let first = jaic::canonicalize(&args.paths[0]).unwrap_or(cwd.clone());
        let start = if first.is_dir() {
            first
        } else {
            first.parent().map(PathBuf::from).unwrap_or(cwd.clone())
        };
        Config::find_on_disk(&start)?.unwrap_or_else(|| Config {
            root: cwd.clone(),
            ..Config::default()
        })
    };
    for note in &config.notes {
        eprintln!("jailint: {note}");
    }
    config.discover = args.config.is_none();
    if args.max_nesting.is_some() {
        config.max_nesting = args.max_nesting;
        config.flag_max_nesting = args.max_nesting;
    }
    for (rule, level) in &args.levels {
        if rule == "warnings" && *level == Level::Deny {
            config.deny_warnings = true;
        } else if rule == "all" {
            for r in RULES {
                config.flags.insert(r.name.to_string(), *level);
            }
        } else if let Some(category) = Category::parse(rule) {
            // Later flags win, so set each rule rather than the category under them.
            for r in RULES.iter().filter(|r| r.category == category) {
                config.flags.insert(r.name.to_string(), *level);
            }
        } else if RULES.iter().any(|r| r.name == rule) {
            config.flags.insert(rule.clone(), *level);
        } else {
            let help = match jaic::suggest::closest(rule, jailint::config::rule_names()) {
                Some(near) => format!("did you mean `{near}`? `jailint --list` lists the rules"),
                None => "`jailint --list` lists the rules".to_string(),
            };
            return Err(format!("unknown rule `{rule}`\nhelp: {help}"));
        }
    }
    let stdlib = jaic::default_stdlib_dir();
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        return Err(message);
    }
    let stdlib = jaic::canonicalize(&stdlib).unwrap_or(stdlib);
    let mut compiler_options = jaic::sema::Options::host();
    args.common.target.apply(&mut compiler_options)?;
    let limits = args.limits();
    let options = jailint::driver::Options {
        compiler_options,
        source: args.context,
        paths: args.paths.clone(),
        import_dirs: args.common.imports.clone(),
        stdlib,
        config,
        jobs: args.common.jobs.unwrap_or(0),
        verbose: args.verbose,
        add_sources: args.common.add,
        limits,
    };
    let outcome = jailint::driver::run(&options);
    for note in &outcome.config_notes {
        eprintln!("jailint: {note}");
    }
    if let Some(error) = outcome.config_errors.first() {
        return Err(error.clone());
    }
    if let Some(path) = &args.checked_files {
        let mut paths = outcome.texts.keys().cloned().collect::<Vec<_>>().join("\n");
        paths.push('\n');
        std::fs::write(path, paths).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    if let Some(path) = &args.manifest {
        write_manifest(
            path,
            &options,
            args.common.target.triple.as_deref(),
            &outcome,
        )?;
    }
    // Lints go to stdout: its terminal decides the layout.
    let style = jaic::render::detect_with(
        args.common.color.unwrap_or(jaic::render::ColorChoice::Auto),
        std::io::stdout().is_terminal(),
        jacket_config::env::lookup,
    );
    let shown = |p: &str| {
        let path = PathBuf::from(p);
        path.strip_prefix(&cwd)
            .map_or_else(|_| p.to_string(), |r| r.to_string_lossy().into_owned())
    };
    let mut errors = 0;
    let mut warnings = 0;
    // One write per report instead of one per line: a file with thousands of findings spent
    // most of its time in the line-buffered stdout's system calls.
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let mut indexes: jaic::fxhash::HashMap<&str, jaic::source::LineIndex> =
        jaic::fxhash::HashMap::default();
    for lint in &outcome.lints {
        let Some(text) = outcome.texts.get(&lint.path) else {
            continue;
        };
        let lines = indexes
            .entry(lint.path.as_str())
            .or_insert_with(|| jaic::source::LineIndex::new(text));
        let default = RULES
            .iter()
            .find(|r| r.name == lint.rule)
            .map_or(Level::Warn, RuleInfo::default);
        let _ = writeln!(
            out,
            "{}",
            render(lint, text, lines, &shown(&lint.path), default, &style)
        );
        match lint.level {
            Level::Deny => errors += 1,
            _ => warnings += 1,
        }
    }
    let _ = out.flush();
    drop(out);
    // Under --allow-incomplete these are expected (other targets' modules, files only their
    // module loads), so one count stands in for the list unless -v asks for it.
    if args.allow_incomplete && !args.verbose {
        if !outcome.incomplete.is_empty() {
            eprintln!(
                "jailint: {} did not compile completely on this target; bodies that failed were not linted (-v lists them)",
                count(outcome.incomplete.len(), "program"),
            );
        }
    } else {
        for (root, error) in &outcome.incomplete {
            eprintln!(
                "jailint: {} did not compile completely; bodies that failed were not linted\n  {error}",
                shown(&root.to_string_lossy())
            );
        }
    }
    if args.fix {
        // Fixes that wrap or overlap another are left for the next pass, which lints the
        // fixed text again (a few passes at most; most runs need one).
        let mut total = 0;
        let mut later: Option<jailint::driver::Outcome> = None;
        for pass in 0..MAX_FIX_PASSES {
            let current = later.as_ref().unwrap_or(&outcome);
            let mut by_file: std::collections::BTreeMap<&str, Vec<&Lint>> =
                std::collections::BTreeMap::default();
            for l in &current.lints {
                by_file.entry(&l.path).or_default().push(l);
            }
            let mut pending = 0;
            for (path, lints) in by_file {
                let Some(text) = current.texts.get(path) else {
                    continue;
                };
                pending += jailint::fix::pending(&lints);
                let (fixed, n) = jailint::fix::apply(text, &lints);
                if n > 0 {
                    std::fs::write(path, fixed).map_err(|e| format!("cannot write {path}: {e}"))?;
                    total += n;
                    eprintln!("jailint: fixed {n} in {}", shown(path));
                }
            }
            if pending == 0 || pass + 1 == MAX_FIX_PASSES {
                break;
            }
            later = Some(jailint::driver::run(&options));
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
    Ok(
        if errors > 0 || (!args.allow_incomplete && !outcome.incomplete.is_empty()) {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        },
    )
}

fn write_manifest(
    path: &std::path::Path,
    options: &jailint::driver::Options,
    triple: Option<&str>,
    outcome: &jailint::driver::Outcome,
) -> Result<(), String> {
    use jaic::target::{cpu_name, os_name};
    let contexts: Vec<_> = outcome
        .contexts
        .iter()
        .map(|context| {
            serde_json::json!({
                "root": context.root, "os": os_name(context.os), "cpu": cpu_name(context.cpu),
                "complete": context.complete, "visited_files": context.visited_files,
            })
        })
        .collect();
    let incomplete: Vec<_> = outcome
        .incomplete
        .iter()
        .map(|(root, error)| serde_json::json!({"root":root,"error":error}))
        .collect();
    let document = serde_json::json!({
        "version": 1,
        "kind": "source-check",
        "phase": "initial",
        "target": {"os":os_name(options.compiler_options.os), "cpu":cpu_name(options.compiler_options.cpu), "triple":triple},
        "config_source": options.config.source,
        "source": options.source,
        "roots": outcome.roots,
        "contexts": contexts,
        "unvisited_files": outcome.unvisited_files,
        "incomplete": incomplete,
    });
    let mut text = serde_json::to_vec_pretty(&document).map_err(|e| e.to_string())?;
    text.push(b'\n');
    std::fs::write(path, text).map_err(|e| format!("cannot write manifest {}: {e}", path.display()))
}

fn count(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}
