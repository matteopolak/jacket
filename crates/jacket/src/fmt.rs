//! `jacket fmt`: `jaifmt` with the project's `[format]` settings resolved here.
//!
//! `jacket-config` is the one reader of `jacket.toml`. For each file to format this finds the
//! nearest `jacket.toml`, validates its `[format]` table, drops files its `ignore` patterns
//! match, and starts `jaifmt` with `--no-config --set key=value ...` (every setting, defaults
//! filled in) and the remaining files. Files that resolve to the same settings go in one
//! `jaifmt` run (split by command line length); the exit status is the worst of the runs.
//!
//! - no path: the project directory (else the working directory) is formatted;
//! - `--stdin`: the settings come from the working directory's `jacket.toml`;
//! - `--config FILE`: that file's `[format]` applies to everything;
//! - `--is-ignored FILE`: answered here (exit 0 ignored, 1 not, 2 error);
//! - `--help`, `--version`, and `--set`/`--no-config` given by the user go to `jaifmt` untouched.
use crate::compile::Context;
use jacket_config::{Config, FILE_NAME, Files, PathPattern, locate, relative_text};
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

/// Bytes of file names one `jaifmt` command line carries (Windows allows 32767 characters in all).
const COMMAND_LINE_BUDGET: usize = if cfg!(windows) {
    24 * 1024
} else {
    128 * 1024
};

/// What `jacket fmt` does.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// The `jaifmt` command lines to run, in order.
    pub runs: Vec<Vec<OsString>>,
    /// Exit with this status without running anything (an answered `--is-ignored`).
    pub answer: Option<u8>,
    /// A configuration or path was reported as an error: the exit status is 2 at best.
    pub failed: bool,
}

/// The settings of one `jacket.toml`, ready to hand to `jaifmt`.
struct Resolved {
    /// The directory the `ignore` patterns are relative to.
    root: PathBuf,
    /// `key=value`, every setting.
    settings: Vec<String>,
    ignore: Vec<PathPattern>,
}

impl Resolved {
    fn new(config: &Config, file: &Path) -> Result<Resolved, String> {
        let at = |message: String| format!("in `{}`, {message}", file.display());
        config.format.validate().map_err(at)?;
        let ignore = config
            .format
            .ignore
            .iter()
            .map(|pattern| PathPattern::new(pattern).map_err(|e| at(e.to_string())))
            .collect::<Result<_, _>>()?;
        Ok(Resolved {
            root: file.parent().unwrap_or(Path::new("")).to_path_buf(),
            settings: config
                .format
                .resolved()
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
            ignore,
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let relative = relative_text(path, &self.root);
        self.ignore
            .iter()
            .any(|pattern| pattern.is_match(&relative))
    }
}

/// Finds the settings that apply to a directory, reading each `jacket.toml` once.
struct Resolver<'a> {
    fs: &'a dyn Files,
    /// `--config FILE`: applies everywhere.
    explicit: Option<Rc<Resolved>>,
    defaults: Rc<Resolved>,
    by_dir: BTreeMap<PathBuf, Option<Rc<Resolved>>>,
    by_config: BTreeMap<PathBuf, Option<Rc<Resolved>>>,
    /// The directory whose `jacket.toml` the command already printed notes for.
    noted: Option<PathBuf>,
    failed: bool,
}

impl Resolver<'_> {
    /// The settings for files in `dir`; `None` when the config is broken (already reported).
    fn for_dir(&mut self, dir: &Path) -> Option<Rc<Resolved>> {
        if let Some(explicit) = &self.explicit {
            return Some(explicit.clone());
        }
        if let Some(found) = self.by_dir.get(dir) {
            return found.clone();
        }
        let found = match locate(self.fs, dir) {
            None => Some(self.defaults.clone()),
            Some(location) => {
                let file = location.file();
                if let Some(found) = self.by_config.get(&location.dir) {
                    found.clone()
                } else {
                    let found = self.load(&file, location.dir.clone());
                    self.by_config.insert(location.dir, found.clone());
                    found
                }
            }
        };
        self.by_dir.insert(dir.to_path_buf(), found.clone());
        found
    }

    fn load(&mut self, file: &Path, dir: PathBuf) -> Option<Rc<Resolved>> {
        let loaded = jacket_config::load_at(
            self.fs,
            jacket_config::Location {
                dir: dir.clone(),
            },
        );
        match loaded {
            Err(error) => self.report(&error.to_string()),
            Ok(loaded) => {
                if self.noted.as_deref() != Some(dir.as_path()) {
                    for note in &loaded.notes {
                        eprintln!("jacket: {note}");
                    }
                }
                match Resolved::new(&loaded.config, file) {
                    Ok(resolved) => Some(Rc::new(resolved)),
                    Err(message) => self.report(&message),
                }
            }
        }
    }

    fn report<T>(&mut self, message: &str) -> Option<T> {
        eprintln!("jacket: error: {message}");
        self.failed = true;
        None
    }
}

fn file_dir(file: &Path) -> PathBuf {
    file.parent().unwrap_or(Path::new("")).to_path_buf()
}

/// `path` made absolute against `cwd`, without `.` and `x/..` components.
fn clean(path: &Path, cwd: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in cwd.join(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// The `jaifmt` flags this understands, sorted out of the command line.
#[derive(Default)]
struct Arguments {
    /// Flags (and their values) `jaifmt` gets as written.
    pass: Vec<OsString>,
    config: Option<PathBuf>,
    is_ignored: Option<PathBuf>,
    stdin: bool,
    paths: Vec<PathBuf>,
    /// `jaifmt` alone can answer: help, version, or settings the user gave it directly.
    raw: bool,
}

fn parse(args: &[OsString]) -> Arguments {
    let mut parsed = Arguments::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let text = arg.to_str().unwrap_or("");
        match text {
            "--config" => match iter.next() {
                Some(value) => parsed.config = Some(PathBuf::from(value)),
                None => parsed.raw = true,
            },
            "--is-ignored" => match iter.next() {
                Some(value) => parsed.is_ignored = Some(PathBuf::from(value)),
                None => parsed.raw = true,
            },
            "--color" | "--set" => {
                parsed.raw |= text == "--set";
                parsed.pass.push(arg.clone());
                parsed.pass.extend(iter.next().cloned());
            }
            "--stdin" => {
                parsed.stdin = true;
                parsed.pass.push(arg.clone());
            }
            "--no-config" | "-h" | "--help" | "--version" | "-V" => {
                parsed.raw = true;
                parsed.pass.push(arg.clone());
            }
            _ if text.starts_with('-') && text.len() > 1 => parsed.pass.push(arg.clone()),
            _ => parsed.paths.push(PathBuf::from(arg)),
        }
    }
    parsed
}

/// What to run for `jacket fmt <args>`.
pub fn plan(args: &[OsString], ctx: &Context) -> Plan {
    plan_within(args, ctx, COMMAND_LINE_BUDGET)
}

/// [`plan`] with `budget` bytes of file names per `jaifmt` command line.
fn plan_within(args: &[OsString], ctx: &Context, budget: usize) -> Plan {
    let parsed = parse(args);
    if parsed.raw {
        return Plan {
            runs: vec![args.to_vec()],
            ..Plan::default()
        };
    }
    let mut resolver = Resolver {
        fs: ctx.fs,
        explicit: None,
        defaults: Rc::new(
            Resolved::new(&Config::default(), Path::new(FILE_NAME)).expect("defaults are valid"),
        ),
        by_dir: BTreeMap::new(),
        by_config: BTreeMap::new(),
        noted: ctx.loaded.map(|loaded| loaded.dir().to_path_buf()),
        failed: false,
    };
    if let Some(file) = &parsed.config {
        let file = clean(file, ctx.cwd);
        resolver.explicit = explicit_settings(&mut resolver, &file);
        if resolver.explicit.is_none() {
            return Plan {
                answer: Some(2),
                ..Plan::default()
            };
        }
    }

    if let Some(file) = &parsed.is_ignored {
        let file = clean(file, ctx.cwd);
        let Some(settings) = resolver.for_dir(&file_dir(&file)) else {
            return Plan {
                answer: Some(2),
                ..Plan::default()
            };
        };
        return Plan {
            answer: Some(u8::from(!settings.is_ignored(&file))),
            ..Plan::default()
        };
    }

    let mut plan = Plan::default();
    if parsed.stdin {
        match resolver.for_dir(ctx.cwd) {
            Some(settings) => plan.runs.push(command(&settings, &parsed.pass, &[])),
            None => plan.failed = true,
        }
        return plan;
    }

    let roots: Vec<PathBuf> = if parsed.paths.is_empty() {
        vec![ctx.loaded.map_or(ctx.cwd, |l| l.dir()).to_path_buf()]
    } else {
        parsed.paths.iter().map(|p| clean(p, ctx.cwd)).collect()
    };
    // Files by the settings they get, so equal settings share a run.
    let mut groups: BTreeMap<Vec<String>, (Rc<Resolved>, Vec<PathBuf>)> = BTreeMap::new();
    for root in &roots {
        if !root.exists() {
            eprintln!(
                "jacket: error: `{}` does not exist\nhelp: relative paths start from the current directory",
                root.display()
            );
            plan.failed = true;
        } else if root.is_dir() {
            collect(root, &mut resolver, &mut groups);
        } else if let Some(settings) = resolver.for_dir(&file_dir(root))
            && !settings.is_ignored(root)
        {
            add(&mut groups, settings, root.clone());
        }
    }
    plan.failed |= resolver.failed;
    for (settings, mut files) in groups.into_values() {
        files.sort();
        files.dedup();
        let mut chunk: Vec<OsString> = Vec::new();
        let mut size = 0;
        for file in files {
            let file = file.into_os_string();
            if size + file.len() > budget && !chunk.is_empty() {
                plan.runs.push(command(&settings, &parsed.pass, &chunk));
                chunk.clear();
                size = 0;
            }
            size += file.len() + 1;
            chunk.push(file);
        }
        if !chunk.is_empty() {
            plan.runs.push(command(&settings, &parsed.pass, &chunk));
        }
    }
    plan
}

/// The `[format]` of the `--config` file, wherever it is named.
fn explicit_settings(resolver: &mut Resolver, file: &Path) -> Option<Rc<Resolved>> {
    let text = match resolver.fs.read(file) {
        Ok(text) => text,
        Err(error) => {
            return resolver.report(&format!(
                "could not read config `{}`: {error}",
                file.display()
            ));
        }
    };
    match jacket_config::parse_noting(&text) {
        Err(message) => resolver.report(&format!("in `{}`, {message}", file.display())),
        Ok((config, unknown)) => {
            for key in unknown {
                eprintln!("jacket: {}", jacket_config::unknown_note(&key));
            }
            match Resolved::new(&config, file) {
                Ok(resolved) => Some(Rc::new(resolved)),
                Err(message) => resolver.report(&message),
            }
        }
    }
}

fn add(
    groups: &mut BTreeMap<Vec<String>, (Rc<Resolved>, Vec<PathBuf>)>,
    settings: Rc<Resolved>,
    file: PathBuf,
) {
    groups
        .entry(settings.settings.clone())
        .or_insert_with(|| (settings, Vec::new()))
        .1
        .push(file);
}

/// The `*.jai` files under `dir`, apart from dot-directories, symlinked directories and what the
/// `ignore` patterns of the config above each one match.
fn collect(
    dir: &Path,
    resolver: &mut Resolver,
    groups: &mut BTreeMap<Vec<String>, (Rc<Resolved>, Vec<PathBuf>)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("jacket: error: could not read `{}`", dir.display());
        resolver.failed = true;
        return;
    };
    let Some(settings) = resolver.for_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            if !settings.is_ignored(&path) {
                collect(&path, resolver, groups);
            }
        } else if path.extension() == Some(OsStr::new("jai"))
            && (kind.is_file() || path.is_file())
            && !settings.is_ignored(&path)
        {
            add(groups, settings.clone(), path);
        }
    }
}

fn command(settings: &Resolved, pass: &[OsString], files: &[OsString]) -> Vec<OsString> {
    let mut out: Vec<OsString> = vec!["--no-config".into()];
    for setting in &settings.settings {
        out.push("--set".into());
        out.push(setting.into());
    }
    out.extend(pass.iter().cloned());
    out.extend(files.iter().cloned());
    out
}

/// Runs the plan with `jaifmt` and returns the exit status.
pub fn run(plan: &Plan) -> std::process::ExitCode {
    use std::process::{Command, ExitCode};
    if let Some(code) = plan.answer {
        return ExitCode::from(code);
    }
    let Some(jaifmt) = crate::tools::require(crate::tools::JAIFMT) else {
        return ExitCode::from(127);
    };
    let mut worst = if plan.failed {
        2
    } else {
        0
    };
    for args in &plan.runs {
        match Command::new(&jaifmt).args(args).status() {
            Ok(status) => {
                let code = status.code().unwrap_or(2);
                // 2 (errors) outranks 1 (files would change).
                worst = if worst == 2 || code == 2 {
                    2
                } else {
                    worst.max(u8::try_from(code).unwrap_or(2))
                };
            }
            Err(error) => {
                eprintln!(
                    "jacket: could not start `{}`: {error}\nhelp: jacket looks next to its own executable, then on PATH; install the toolchain archive or put the tool on PATH",
                    jaifmt.display()
                );
                return ExitCode::from(127);
            }
        }
    }
    ExitCode::from(worst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jacket_config::Disk;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("jacket-fmt-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn plan_in(cwd: &Path, args: &[&str]) -> Plan {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        let ctx = Context {
            loaded: None,
            cwd,
            fs: &Disk,
        };
        plan(&args, &ctx)
    }

    fn texts(run: &[OsString]) -> Vec<String> {
        run.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    fn files_of(run: &[OsString], root: &Path) -> Vec<String> {
        texts(run)
            .into_iter()
            .filter(|a| a.starts_with(root.to_str().unwrap()))
            .map(|a| a[root.to_str().unwrap().len() + 1..].to_string())
            .collect()
    }

    #[test]
    fn settings_are_resolved_and_ignored_files_dropped() {
        let root = scratch("ignore");
        write(
            &root.join("jacket.toml"),
            "[format]\nindent_width = 2\nignore = [\"gen/**\", \"skip.jai\"]\n",
        );
        for file in [
            "a.jai",
            "skip.jai",
            "gen/b.jai",
            "src/c.jai",
            ".hidden/d.jai",
        ] {
            write(&root.join(file), "");
        }
        write(&root.join("notes.txt"), "");
        let plan = plan_in(&root, &["--check", "."]);
        assert!(!plan.failed);
        assert_eq!(plan.runs.len(), 1);
        let run = texts(&plan.runs[0]);
        assert_eq!(run[0], "--no-config");
        assert!(run.windows(2).any(|w| w == ["--set", "indent_width=2"]));
        assert!(run.windows(2).any(|w| w == ["--set", "case_indent=2"]));
        assert!(run.contains(&"--check".to_string()));
        assert_eq!(files_of(&plan.runs[0], &root), ["a.jai", "src/c.jai"]);
    }

    #[test]
    fn nested_configs_group_files_by_their_settings() {
        let root = scratch("nested");
        write(&root.join("jacket.toml"), "[format]\nindent_width = 2\n");
        write(
            &root.join("sub/jacket.toml"),
            "[format]\nindent_width = 8\n",
        );
        write(&root.join("a.jai"), "");
        write(&root.join("sub/b.jai"), "");
        write(&root.join("sub/deep/c.jai"), "");
        let plan = plan_in(&root, &[]);
        assert_eq!(plan.runs.len(), 2);
        let mut by_indent: Vec<(String, Vec<String>)> = plan
            .runs
            .iter()
            .map(|run| {
                let indent = texts(run)
                    .into_iter()
                    .find(|a| a.starts_with("indent_width="))
                    .unwrap();
                (indent, files_of(run, &root))
            })
            .collect();
        by_indent.sort();
        assert_eq!(
            by_indent[0],
            ("indent_width=2".into(), vec!["a.jai".to_string()])
        );
        assert_eq!(
            by_indent[1],
            (
                "indent_width=8".into(),
                vec!["sub/b.jai".to_string(), "sub/deep/c.jai".to_string()]
            )
        );
    }

    #[test]
    fn a_bad_value_fails_the_run() {
        let root = scratch("bad");
        write(&root.join("jacket.toml"), "[format]\nindent_width = 99\n");
        write(&root.join("a.jai"), "");
        let plan = plan_in(&root, &["."]);
        assert!(plan.failed);
        assert!(plan.runs.is_empty());
    }

    #[test]
    fn an_explicit_file_is_checked_against_ignore() {
        let root = scratch("explicit");
        write(
            &root.join("jacket.toml"),
            "[format]\nignore = [\"x.jai\"]\n",
        );
        write(&root.join("x.jai"), "");
        write(&root.join("y.jai"), "");
        assert!(plan_in(&root, &["x.jai"]).runs.is_empty());
        assert_eq!(plan_in(&root, &["y.jai"]).runs.len(), 1);
        assert_eq!(plan_in(&root, &["--is-ignored", "x.jai"]).answer, Some(0));
        assert_eq!(plan_in(&root, &["--is-ignored", "y.jai"]).answer, Some(1));
    }

    #[test]
    fn stdin_uses_the_working_directory_settings() {
        let root = scratch("stdin");
        write(
            &root.join("jacket.toml"),
            "[format]\nbrace_style = \"preserve\"\n",
        );
        let plan = plan_in(&root, &["--stdin"]);
        let run = texts(&plan.runs[0]);
        assert!(
            run.windows(2)
                .any(|w| w == ["--set", "brace_style=\"preserve\""])
        );
        assert_eq!(run.last().unwrap(), "--stdin");
    }

    #[test]
    fn flag_values_are_not_paths_and_raw_flags_pass_through() {
        let root = scratch("flags");
        write(&root.join("a.jai"), "");
        let plan = plan_in(&root, &["--color", "never", "--check"]);
        let run = texts(&plan.runs[0]);
        assert!(run.windows(2).any(|w| w == ["--color", "never"]));
        assert_eq!(files_of(&plan.runs[0], &root), ["a.jai"]);
        let raw = plan_in(&root, &["--version"]);
        assert_eq!(texts(&raw.runs[0]), ["--version"]);
        let raw = plan_in(&root, &["--no-config", "--stdin"]);
        assert_eq!(texts(&raw.runs[0]), ["--no-config", "--stdin"]);
    }

    #[test]
    fn missing_paths_are_reported() {
        let root = scratch("missing");
        let plan = plan_in(&root, &["nope.jai"]);
        assert!(plan.failed);
        assert!(plan.runs.is_empty());
    }

    #[test]
    fn long_file_lists_split_into_runs() {
        let root = scratch("long");
        for n in 0..400 {
            write(
                &root.join(format!(
                    "module_with_a_long_name_{n:04}_{}.jai",
                    "x".repeat(60)
                )),
                "",
            );
        }
        let ctx = Context {
            loaded: None,
            cwd: &root,
            fs: &Disk,
        };
        let plan = plan_within(&[".".into()], &ctx, 8 * 1024);
        assert!(plan.runs.len() > 1);
        let total: usize = plan.runs.iter().map(|r| files_of(r, &root).len()).sum();
        assert_eq!(total, 400);
    }
}
