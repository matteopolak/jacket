//! `jailint`: report (and fix) common mistakes and unidiomatic code in Jai programs.
use jailint::config::{Config, Level};
use jailint::render::render;
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
  -os <os>           check linux, windows, macos or wasm (default: host)
  -cpu <cpu>         check x64/x86_64 or arm64/aarch64 (default: target/host)
  -target <triple>   select a target triple (overrides -os/-cpu; includes WASI)
  --manifest <file> write JSON source-check coverage and actual target contexts
  --context <file>  check an enclosing program; report only the requested paths
  --checked-files <file> write the actual checked paths, one per line
  --add <source>     add source in the program scope (also -add)
  -j <n>              compile n programs at once (default 1)
  --list              list the rules and their default levels
  --color <when>      auto, always or never
  -v, --verbose       show each program as it is compiled
  -h, --help          show this help

Exit status: 0 when nothing at level `deny` was found, 1 when something was, 2 on a usage
or configuration error.";

fn main() -> ExitCode {
    // Errors about the command line or configuration go to stderr in its own style.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let color = jaic::render::ColorChoice::from_args(&args, &["--color"]);
    jaic::render::set_style(jaic::render::detect(color));
    match run() {
        Ok(code) => code,
        Err(message) => {
            let report = jaic::render::Report::from_text(jaic::render::Severity::Error, &message);
            eprint!("{}", report.render());
            ExitCode::from(2)
        }
    }
}

fn color_value(when: &str) -> Result<jaic::render::ColorChoice, String> {
    jaic::render::ColorChoice::parse(when)
        .ok_or_else(|| format!("unknown `--color` value `{when}`\nhelp: use auto, always or never"))
}

struct Args {
    context: Option<PathBuf>,
    manifest: Option<PathBuf>,
    checked_files: Option<PathBuf>,
    add_sources: Vec<String>,
    target: jailint::target::Selection,
    paths: Vec<PathBuf>,
    fix: bool,
    config: Option<PathBuf>,
    levels: Vec<(String, Level)>,
    imports: Vec<PathBuf>,
    jobs: usize,
    color: jaic::render::ColorChoice,
    verbose: bool,
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut args = Args {
        context: None,
        manifest: None,
        checked_files: None,
        add_sources: Vec::new(),
        target: Default::default(),
        paths: Vec::new(),
        fix: false,
        config: None,
        levels: Vec::new(),
        imports: Vec::new(),
        jobs: 1,
        color: jaic::render::ColorChoice::Auto,
        verbose: false,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("`{name}` needs a value"));
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
            "--manifest" => args.manifest = Some(PathBuf::from(value(&a)?)),
            "--checked-files" => args.checked_files = Some(PathBuf::from(value(&a)?)),
            "--add" | "-add" => args.add_sources.push(value(&a)?),
            "--context" => args.context = Some(PathBuf::from(value(&a)?)),
            "-I" | "-import_dir" | "--import-dir" => args.imports.push(value(&a)?.into()),
            "-os" | "--os" => args.target.os = Some(jailint::target::parse_os(&value(&a)?)?),
            "-cpu" | "--cpu" => args.target.cpu = Some(jailint::target::parse_cpu(&value(&a)?)?),
            "-target" => args.target.triple = Some(value(&a)?),
            "-j" | "--jobs" => {
                args.jobs = value(&a)?
                    .parse()
                    .map_err(|_| format!("`{a}` takes a number of jobs, such as `{a} 4`"))?
            }
            "--color" => {
                let when = value("--color")?;
                args.color = color_value(&when)?;
            }
            flag if flag.starts_with("--color=") => {
                args.color = color_value(&flag["--color=".len()..])?;
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
                let known = [
                    "--fix",
                    "--config",
                    "--manifest",
                    "--checked-files",
                    "--add",
                    "--os",
                    "--cpu",
                    "--context",
                    "-A",
                    "-W",
                    "-D",
                    "-I",
                    "-import_dir",
                    "-os",
                    "-cpu",
                    "-target",
                    "-j",
                    "--jobs",
                    "--list",
                    "--color",
                    "--verbose",
                    "--help",
                ];
                let help = match jaic::suggest::closest(flag, known) {
                    Some(near) => {
                        format!("did you mean `{near}`? `jailint --help` lists the options")
                    }
                    None => "`jailint --help` lists the options".to_string(),
                };
                return Err(format!("unknown option `{flag}`\nhelp: {help}"));
            }
            path => args.paths.push(path.into()),
        }
    }
    if args.paths.is_empty() {
        args.paths.push(".".into());
    }
    Ok(Some(args))
}

/// Passes `--fix` makes at most: lint, apply what can go together, lint again.
const MAX_FIX_PASSES: usize = 5;

fn run() -> Result<ExitCode, String> {
    let Some(args) = parse_args()? else {
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
            let help = match jaic::suggest::closest(rule, RULES.iter().map(|r| r.name)) {
                Some(near) => format!("did you mean `{near}`? `jailint --list` lists the rules"),
                None => "`jailint --list` lists the rules".to_string(),
            };
            return Err(format!("unknown rule `{rule}`\nhelp: {help}"));
        }
    }
    let stdlib = jaic::stdlib_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"));
    if let Some(message) = jaic::missing_stdlib(&stdlib) {
        return Err(message);
    }
    let stdlib = stdlib.canonicalize().unwrap_or(stdlib);
    let mut compiler_options = jaic::sema::Options::host();
    args.target.apply(&mut compiler_options)?;
    let options = jailint::driver::Options {
        compiler_options,
        source: args.context,
        paths: args.paths.clone(),
        import_dirs: args.imports.clone(),
        stdlib,
        config,
        jobs: args.jobs,
        verbose: args.verbose,
        add_sources: args.add_sources,
    };
    let outcome = jailint::driver::run(&options);
    if let Some(path) = &args.checked_files {
        let mut paths = outcome.texts.keys().cloned().collect::<Vec<_>>().join("\n");
        paths.push('\n');
        std::fs::write(path, paths).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    if let Some(path) = &args.manifest {
        write_manifest(path, &options, args.target.triple.as_deref(), &outcome)?;
    }
    // Lints go to stdout: its terminal decides the layout.
    let style = jaic::render::detect_with(args.color, std::io::stdout().is_terminal(), |key| {
        std::env::var(key).ok()
    });
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
    for (root, error) in &outcome.incomplete {
        eprintln!(
            "jailint: {} did not compile completely ({error}); bodies that failed were not linted",
            shown(&root.to_string_lossy())
        );
    }
    if args.fix {
        // Fixes that wrap or overlap another are left for the next pass, which lints the
        // fixed text again (a few passes at most; most runs need one).
        let mut total = 0;
        let mut later: Option<jailint::driver::Outcome> = None;
        for pass in 0..MAX_FIX_PASSES {
            let current = later.as_ref().unwrap_or(&outcome);
            let mut by_file: std::collections::BTreeMap<&str, Vec<&Lint>> = Default::default();
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
    Ok(if errors > 0 || !outcome.incomplete.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn write_manifest(
    path: &std::path::Path,
    options: &jailint::driver::Options,
    triple: Option<&str>,
    outcome: &jailint::driver::Outcome,
) -> Result<(), String> {
    use jailint::target::{cpu_name, os_name};
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
