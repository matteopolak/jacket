//! The linter's settings: the `[lint]` section of `jacket.toml`.
//!
//! ```toml
//! [lint]
//! exclude = ["tests/corpus/**"]
//! max_nesting = 6
//!
//! [lint.rules]
//! float_equality = "warn"
//! lossy_xx = "allow"
//!
//! [[lint.overrides]]
//! paths = ["tests/**"]
//! rules = { float_equality = "allow" }
//!
//! [lint.portability]
//! targets = ["linux", "macos", "windows"]
//! portable = ["stdlib/**"]
//!
//! [lint.portability.platforms]
//! "stdlib/POSIX/**" = ["linux", "macos"]
//! ```
//!
//! Anything the schema does not know under `[lint]` is an error, so a typo does not silently
//! keep a rule on.
use jacket_config::{Disk, Files, Lint, PathPattern};
use jaic::fxhash::HashMap;
use jaic::sema::TargetOs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const FILE_NAME: &str = jacket_config::FILE_NAME;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Allow,
    Warn,
    Deny,
}

impl Level {
    pub fn parse(text: &str) -> Option<Level> {
        match text {
            "allow" => Some(Level::Allow),
            "warn" => Some(Level::Warn),
            "deny" => Some(Level::Deny),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Level::Allow => "allow",
            Level::Warn => "warn",
            Level::Deny => "deny",
        }
    }
}

/// A group of rules, named like a rule in settings, `-A`/`-W`/`-D` and `allow(...)` comments.
/// Its level is the default of every rule in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Code that is wrong: it does not do what it says.
    Correctness,
    /// Code that is most likely wrong or does nothing.
    Suspicious,
    /// Code that says something in a roundabout way.
    Complexity,
    /// Code that is fine but not written the usual way.
    Style,
    /// Code that could be faster.
    Perf,
    /// Stricter checks with more false positives: off by default.
    Pedantic,
    /// Checks that forbid things a project may choose to avoid: off by default.
    Restriction,
}

impl Category {
    pub const ALL: [Category; 7] = [
        Category::Correctness,
        Category::Suspicious,
        Category::Complexity,
        Category::Style,
        Category::Perf,
        Category::Pedantic,
        Category::Restriction,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Correctness => "correctness",
            Category::Suspicious => "suspicious",
            Category::Complexity => "complexity",
            Category::Style => "style",
            Category::Perf => "perf",
            Category::Pedantic => "pedantic",
            Category::Restriction => "restriction",
        }
    }

    pub fn parse(text: &str) -> Option<Category> {
        Category::ALL.into_iter().find(|c| c.as_str() == text)
    }

    /// The default level of the rules in this category.
    pub fn level(self) -> Level {
        match self {
            Category::Correctness => Level::Deny,
            Category::Suspicious | Category::Complexity | Category::Style | Category::Perf => {
                Level::Warn
            }
            Category::Pedantic | Category::Restriction => Level::Allow,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Levels set for rules and categories (by name); a rule's own entry wins over its
    /// category's, and anything unset keeps its default.
    pub levels: HashMap<String, Level>,
    /// Levels for the files under some paths, in file order: a later match wins, and any match
    /// wins over `levels`.
    pub overrides: Vec<Override>,
    /// Levels given on the command line (`-A`, `-W`, `-D`), by rule: they win over everything.
    pub flags: HashMap<String, Level>,
    /// Warnings become errors (`-D warnings`).
    pub deny_warnings: bool,
    /// `--max-nesting` from the command line: it wins over every `jacket.toml`.
    pub flag_max_nesting: Option<usize>,
    /// Each file uses the nearest `jacket.toml` above it (see [`Configs`]); `--config` leaves
    /// this off so that one file settles everything.
    pub discover: bool,
    /// Patterns of paths not to lint, relative to `root`.
    pub exclude: Vec<PathPattern>,
    /// How many blocks deep code may nest before `excessive_nesting` fires.
    pub max_nesting: Option<usize>,
    /// The score above which `cognitive_complexity` fires.
    pub max_complexity: Option<usize>,
    /// The parameter count above which `too_many_arguments` fires.
    pub max_arguments: Option<usize>,
    /// The `bool` parameter count above which `excessive_bools` fires.
    pub max_bools: Option<usize>,
    /// The size in bytes above which `large_stack_array` fires.
    pub max_stack_array_bytes: Option<usize>,
    /// Numbers `magic_number` lets through, besides its built-in ones.
    pub magic_number_allow: Vec<f64>,
    /// Whole numbers below this magnitude are not magic.
    pub magic_number_min: Option<f64>,
    /// `magic_number` also checks files in `tests` folders.
    pub magic_number_in_tests: Option<bool>,
    /// Directory of the configuration file (or the working directory).
    pub root: PathBuf,
    /// Which code `platform_leak` holds to every operating system.
    pub portability: Portability,
    /// The file the settings came from, if any.
    pub source: Option<PathBuf>,
    /// One-line notes about the configuration (unknown keys).
    pub notes: Vec<String>,
    /// `[project] import_path` of the file the settings came from, as absolute folders.
    pub import_dirs: Vec<PathBuf>,
}

/// The settings of every file in a run: each file uses the nearest `jacket.toml` above it,
/// whichever roots were listed, with the command-line levels on top. When the base [`Config`] is
/// not `discover`ing (`--config`), it holds for every file.
pub struct Configs {
    base: Arc<Config>,
    /// Settings by the directory they were asked for.
    found: Mutex<HashMap<PathBuf, Arc<Config>>>,
    /// Notes and errors of the nested files, for the caller to report.
    notes: Mutex<Vec<String>>,
    errors: Mutex<Vec<String>>,
}

impl Configs {
    pub fn new(base: &Config) -> Configs {
        Configs {
            base: Arc::new(base.clone()),
            found: Mutex::default(),
            notes: Mutex::default(),
            errors: Mutex::default(),
        }
    }

    /// The settings of the run's first root (or the file given with `--config`).
    pub fn base(&self) -> &Config {
        &self.base
    }

    /// The settings that hold for the file or directory at `path`.
    pub fn at(&self, path: &Path) -> Arc<Config> {
        if !self.base.discover {
            return self.base.clone();
        }
        let dir = if path.is_dir() {
            Some(path)
        } else {
            path.parent()
        };
        let Some(dir) = dir else {
            return self.base.clone();
        };
        let mut found = self.found.lock().expect("configs lock");
        if let Some(config) = found.get(dir) {
            return config.clone();
        }
        let config = self.load_near(dir, &mut found);
        found.insert(dir.to_path_buf(), config.clone());
        config
    }

    fn load_near(&self, dir: &Path, found: &mut HashMap<PathBuf, Arc<Config>>) -> Arc<Config> {
        let Some(location) = jacket_config::locate(&Disk, dir) else {
            return self.base.clone();
        };
        if location.dir == self.base.root {
            return self.base.clone();
        }
        if let Some(config) = found.get(&location.dir) {
            return config.clone();
        }
        let loaded = jacket_config::load_at_checked(&Disk, location.clone(), CHECKS)
            .map_err(|e| e.to_string())
            .and_then(Config::from_loaded);
        let config = match loaded {
            Ok(mut config) => {
                self.notes
                    .lock()
                    .expect("configs lock")
                    .append(&mut config.notes);
                self.base.invocation_over(config)
            }
            Err(e) => {
                self.errors.lock().expect("configs lock").push(e);
                self.base.clone()
            }
        };
        found.insert(location.dir, config.clone());
        config
    }

    /// Whether `path` is excluded by the run's settings or those nearest to it.
    pub fn excluded(&self, path: &Path) -> bool {
        self.base.excluded(path) || self.at(path).excluded(path)
    }

    /// Notes about the nested configuration files read so far.
    pub fn take_notes(&self) -> Vec<String> {
        std::mem::take(&mut self.notes.lock().expect("configs lock"))
    }

    /// Nested configuration files that could not be read; their files used the base settings.
    pub fn take_errors(&self) -> Vec<String> {
        std::mem::take(&mut self.errors.lock().expect("configs lock"))
    }
}

/// Rule levels for the files matching some globs (`[[lint.overrides]]`).
#[derive(Clone, Debug, Default)]
pub struct Override {
    /// Patterns relative to the config's `root`.
    pub paths: Vec<PathPattern>,
    /// Levels by rule or category name; a rule's own entry wins over its category's.
    pub levels: HashMap<String, Level>,
}

/// The operating systems `platform_leak` checks portable code on when `targets` is left out.
pub const DEFAULT_PORTABILITY_TARGETS: [TargetOs; 3] =
    [TargetOs::Linux, TargetOs::MacOS, TargetOs::Windows];

/// `[lint.portability]`: the code that must build on every operating system it is built for.
#[derive(Clone, Debug, Default)]
pub struct Portability {
    /// The systems portable code is built for; empty means [`DEFAULT_PORTABILITY_TARGETS`].
    pub targets: Vec<TargetOs>,
    /// Patterns relative to `Config::root` of the code that must build on every target.
    pub portable: Vec<PathPattern>,
    /// Patterns of files that build only on the listed systems.
    pub platforms: Vec<(PathPattern, Vec<TargetOs>)>,
}

impl Portability {
    /// The systems portable code is checked on.
    pub fn targets(&self) -> &[TargetOs] {
        if self.targets.is_empty() {
            &DEFAULT_PORTABILITY_TARGETS
        } else {
            &self.targets
        }
    }
}

/// The systems a `// jailint: platforms(linux, macos)` comment in the first lines of a file (before
/// any code) restricts it to; names that are not a system are ignored.
pub fn file_platforms(text: &str) -> Option<Vec<TargetOs>> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let comment = line.strip_prefix("//")?;
        let Some(rest) = comment.trim().strip_prefix("jailint:") else {
            continue;
        };
        let Some(list) = rest.trim().strip_prefix("platforms") else {
            continue;
        };
        let inner = list.trim().strip_prefix('(')?;
        let inner = &inner[..inner.find(')')?];
        return Some(
            inner
                .split(',')
                .filter_map(|name| platform(name.trim()))
                .collect(),
        );
    }
    None
}

/// The operating system a `platform_leak` setting names.
fn platform(name: &str) -> Option<TargetOs> {
    DEFAULT_PORTABILITY_TARGETS
        .into_iter()
        .find(|os| jaic::target::os_name(*os) == name)
}

/// The nesting depth `excessive_nesting` allows when nothing sets it.
pub const DEFAULT_MAX_NESTING: usize = 6;

/// The cognitive complexity `cognitive_complexity` allows when nothing sets it.
pub const DEFAULT_MAX_COMPLEXITY: usize = 25;

/// The parameters `too_many_arguments` allows when nothing sets it.
pub const DEFAULT_MAX_ARGUMENTS: usize = 7;

/// The `bool` parameters `excessive_bools` allows when nothing sets it.
pub const DEFAULT_MAX_BOOLS: usize = 3;

/// The bytes a local array may take for `large_stack_array` when nothing sets it (64 KiB).
pub const DEFAULT_MAX_STACK_ARRAY_BYTES: usize = 64 * 1024;

/// Whole numbers below this magnitude are not magic when nothing sets `magic_number_min`.
pub const DEFAULT_MAGIC_NUMBER_MIN: f64 = 3.0;

/// The numbers `magic_number` lets through.
#[derive(Clone, Debug)]
pub struct MagicNumbers {
    /// Allowed exactly (and negated).
    pub allow: Vec<f64>,
    /// Whole numbers below this magnitude are allowed.
    pub min: f64,
    /// Files in `tests` folders are checked too.
    pub in_tests: bool,
}

impl Default for MagicNumbers {
    fn default() -> Self {
        MagicNumbers {
            allow: Vec::new(),
            min: DEFAULT_MAGIC_NUMBER_MIN,
            in_tests: false,
        }
    }
}

impl Config {
    /// The deepest block nesting `excessive_nesting` accepts.
    pub fn max_nesting(&self) -> usize {
        self.max_nesting.unwrap_or(DEFAULT_MAX_NESTING)
    }

    /// The numbers `magic_number` lets through.
    pub fn magic_numbers(&self) -> MagicNumbers {
        MagicNumbers {
            allow: self.magic_number_allow.clone(),
            min: self.magic_number_min.unwrap_or(DEFAULT_MAGIC_NUMBER_MIN),
            in_tests: self.magic_number_in_tests.unwrap_or(false),
        }
    }

    /// The level of `rule` outside every override: a flag, else its own setting, else its
    /// category's, else the category default.
    pub fn level(&self, rule: &crate::RuleInfo) -> Level {
        self.resolve(rule, None)
    }

    /// The level of `rule` in the file at `path`: as [`Config::level`], with the last
    /// `[[lint.overrides]]` entry matching `path` that names the rule or its category taking
    /// the place of `[lint.rules]`.
    pub fn level_at(&self, rule: &crate::RuleInfo, path: &Path) -> Level {
        self.resolve(rule, Some(path))
    }

    fn resolve(&self, rule: &crate::RuleInfo, path: Option<&Path>) -> Level {
        let named = |levels: &HashMap<String, Level>| {
            levels
                .get(rule.name)
                .or_else(|| levels.get(rule.category.as_str()))
                .copied()
        };
        let overridden = path.and_then(|path| {
            let text = self.relative(path);
            self.overrides
                .iter()
                .rev()
                .filter(|o| o.paths.iter().any(|p| p.is_match(&text)))
                .find_map(|o| named(&o.levels))
        });
        let level = self
            .flags
            .get(rule.name)
            .copied()
            .or(overridden)
            .or_else(|| named(&self.levels))
            .unwrap_or_else(|| rule.default());
        if self.deny_warnings && level == Level::Warn {
            Level::Deny
        } else {
            level
        }
    }

    /// Settings from a `[lint]` section whose paths are relative to `root`.
    pub fn from_lint(lint: &Lint, root: &Path) -> Result<Config, String> {
        if let Some(i) = lint
            .exclude
            .iter()
            .position(|p| p.as_str().trim_matches('/').is_empty())
        {
            return Err(format!(
                "`exclude` entry {} is empty and would match nothing\nhelp: write a path or glob such as \"tests/**\", or remove the entry",
                i + 1
            ));
        }
        let mut config = Config {
            root: root.to_path_buf(),
            exclude: lint.exclude.clone(),
            max_nesting: lint.max_nesting,
            max_complexity: lint.max_complexity,
            max_arguments: lint.max_arguments,
            max_bools: lint.max_bools,
            max_stack_array_bytes: lint.max_stack_array_bytes,
            magic_number_allow: lint.magic_number_allow.clone(),
            magic_number_min: lint.magic_number_min,
            magic_number_in_tests: lint.magic_number_in_tests,
            ..Config::default()
        };
        config.levels = levels(&lint.rules)?;
        config.portability = portability(&lint.portability)?;
        for o in &lint.overrides {
            if o.paths.is_empty() {
                return Err("a `[[lint.overrides]]` entry has no `paths`".into());
            }
            config.overrides.push(Override {
                paths: o.paths.clone(),
                levels: levels(&o.rules)?,
            });
        }
        Ok(config)
    }

    /// Whether the file at `path` is held to every target: it matches a `portable` glob.
    pub fn is_portable(&self, path: &Path) -> bool {
        let text = self.relative(path);
        self.portability.portable.iter().any(|p| p.is_match(&text))
    }

    /// Whether the file at `path` builds on `os` as the configuration says: the `platforms` glob
    /// that matches it most specifically (the most characters that are not `*` or `?`) lists
    /// `os`, or none matches. Globs of equal specificity must all list it. (A `platforms(...)`
    /// comment in the file restricts it further; see [`file_platforms`].)
    pub fn builds_on(&self, path: &Path, os: TargetOs) -> bool {
        let text = self.relative(path);
        let matching = || {
            self.portability
                .platforms
                .iter()
                .filter(|(pattern, _)| pattern.is_match(&text))
        };
        let best = matching().map(|(pattern, _)| pattern.specificity()).max();
        matching()
            .filter(|(pattern, _)| Some(pattern.specificity()) == best)
            .all(|(_, systems)| systems.contains(&os))
    }

    /// The settings of the nearest config directory above `start`; `Ok(None)` when there is
    /// none. `files` is the disk, or an editor's buffers over it.
    pub fn find(files: &dyn Files, start: &Path) -> Result<Option<Config>, String> {
        match jacket_config::load_checked(files, start, CHECKS) {
            Ok(Some(loaded)) => Self::from_loaded(loaded).map(Some),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Settings of the `jacket.toml` (or another file in its format) at `path`.
    pub fn load(path: &Path) -> Result<Config, String> {
        // Paths checked against `exclude` are canonical, so the root must be too.
        let dir = match path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir,
            _ => Path::new("."),
        };
        let dir = &dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "could not read config `{}`: {}",
                path.display(),
                jaic::io_reason(&e)
            )
        })?;
        let (parsed, unknown) = jacket_config::parse_noting(&text)
            .map_err(|e| format!("in `{}`, {e}", path.display()))?;
        let mut config = Self::from_lint(&parsed.lint, dir)
            .map_err(|e| format!("in `{}`, {e}", path.display()))?;
        config.notes = unknown
            .iter()
            .map(|key| jacket_config::unknown_note(key))
            .collect();
        config.source = Some(path.to_path_buf());
        Ok(config)
    }

    /// `config` with this run's command-line levels on top.
    fn invocation_over(&self, mut config: Config) -> Arc<Config> {
        config.flags.clone_from(&self.flags);
        config.deny_warnings = self.deny_warnings;
        config.discover = true;
        if self.flag_max_nesting.is_some() {
            config.max_nesting = self.flag_max_nesting;
            config.flag_max_nesting = self.flag_max_nesting;
        }
        Arc::new(config)
    }

    fn from_loaded(loaded: jacket_config::Loaded) -> Result<Config, String> {
        let source = loaded.file();
        if let Some(problem) = loaded.problems.first() {
            return Err(format!("in `{}`, {problem}", source.display()));
        }
        let mut config = Config::from_lint(&loaded.config.lint, loaded.dir())
            .map_err(|e| format!("in `{}`, {e}", source.display()))?;
        config.source = Some(source);
        config.import_dirs = loaded.config.project.import_dirs(loaded.dir());
        config.notes = loaded.notes;
        Ok(config)
    }

    /// The nearest settings on disk above `start`.
    pub fn find_on_disk(start: &Path) -> Result<Option<Config>, String> {
        Self::find(&Disk, start)
    }

    /// Whether `path` matches an `exclude` pattern.
    pub fn excluded(&self, path: &Path) -> bool {
        let text = self.relative(path);
        self.exclude.iter().any(|p| p.is_match(&text))
    }

    /// `path` relative to `root`, with `/` separators, for matching globs.
    fn relative(&self, path: &Path) -> String {
        jacket_config::relative_text(path, &self.root)
    }
}

/// The check of `[lint]` that `jacket_config` runs as a file is loaded: rule, category and
/// operating-system names must exist, paths must make sense.
fn check_lint(config: &jacket_config::Config, dir: &Path) -> Result<(), String> {
    Config::from_lint(&config.lint, dir).map(|_| ())
}

/// The checks a tool that reads `[lint]` passes to `jacket_config::load_checked`.
pub const CHECKS: &[jacket_config::Check] = &[check_lint];

/// Checked `[lint.portability]` settings.
fn portability(settings: &jacket_config::Portability) -> Result<Portability, String> {
    let systems = |names: &[String], key: &str| -> Result<Vec<TargetOs>, String> {
        names
            .iter()
            .map(|name| {
                platform(name).ok_or_else(|| {
                    format!(
                        "unknown operating system \"{name}\" in `{key}`\nhelp: use \"linux\", \"macos\" or \"windows\""
                    )
                })
            })
            .collect()
    };
    let mut out = Portability {
        targets: systems(&settings.targets, "lint.portability.targets")?,
        portable: settings.portable.clone(),
        platforms: Vec::new(),
    };
    for (pattern, names) in &settings.platforms {
        out.platforms.push((
            pattern.clone(),
            systems(names, &format!("lint.portability.platforms.\"{pattern}\""))?,
        ));
    }
    Ok(out)
}

/// Rule and category names, for suggestions.
pub fn rule_names() -> impl Iterator<Item = &'static str> {
    crate::RULES
        .iter()
        .map(|r| r.name)
        .chain(Category::ALL.map(Category::as_str))
}

/// Checked levels of a `rules` table.
fn levels(
    rules: &std::collections::BTreeMap<String, String>,
) -> Result<HashMap<String, Level>, String> {
    let mut out = HashMap::default();
    for (rule, level) in rules {
        check_rule(rule, level)?;
        out.insert(
            rule.clone(),
            Level::parse(level).expect("`check_rule` accepts a level"),
        );
    }
    Ok(out)
}

/// Whether `rule` is a rule or category and `value` a level for it.
fn check_rule(rule: &str, value: &str) -> Result<(), String> {
    if !rule_names().any(|r| r == rule) {
        let help = match jaic::suggest::closest(rule, rule_names()) {
            Some(near) => {
                format!("did you mean `{near}`? `jailint --list` lists the rules")
            }
            None => "`jailint --list` lists the rules".to_string(),
        };
        return Err(format!("unknown rule `{rule}`\nhelp: {help}"));
    }
    Level::parse(value).map(drop).ok_or_else(|| {
        format!("unknown level \"{value}\" for `{rule}`\nhelp: use \"allow\", \"warn\" or \"deny\"")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Config, String> {
        let parsed = jacket_config::parse(text)?;
        Config::from_lint(&parsed.lint, Path::new("/r"))
    }

    #[test]
    fn parses_rules_and_excludes() {
        let text = "[lint]\n# settings\nexclude = [\n  \"tests/**\", # fixtures\n  \"a.jai\",\n]\n\n[lint.rules]\nfloat_equality = \"deny\" # strict\n";
        let config = parse(text).unwrap();
        assert_eq!(
            config
                .exclude
                .iter()
                .map(PathPattern::as_str)
                .collect::<Vec<_>>(),
            ["tests/**", "a.jai"]
        );
        assert_eq!(config.level(rule("float_equality")), Level::Deny);
        assert_eq!(config.level(rule("lossy_xx")), Level::Allow);
    }

    fn rule(name: &str) -> &'static crate::RuleInfo {
        crate::RULES.iter().find(|r| r.name == name).unwrap()
    }

    #[test]
    fn categories_set_their_rules_unless_a_rule_says_otherwise() {
        let text =
            "[lint.rules]\npedantic = \"warn\"\nlossy_xx = \"allow\"\ncorrectness = \"warn\"\n";
        let config = parse(text).unwrap();
        assert_eq!(config.level(rule("float_equality")), Level::Warn);
        assert_eq!(config.level(rule("lossy_xx")), Level::Allow);
        assert_eq!(config.level(rule("double_free")), Level::Warn);
        assert_eq!(config.level(rule("unused_variable")), Level::Warn);
        let defaults = parse("").unwrap();
        assert_eq!(defaults.level(rule("double_free")), Level::Deny);
        assert!(parse("[lint.rules]\npedantc = \"warn\"").is_err_and(|e| e.contains("pedantic")));
    }

    #[test]
    fn overrides_set_levels_under_their_paths() {
        let text = "[lint.rules]\nfloat_equality = \"warn\"\n\n[[lint.overrides]]\npaths = [\"tests/**\"]\nrules = { float_equality = \"allow\", pedantic = \"warn\" }\n\n[[lint.overrides]]\npaths = [\"tests/strict\"]\nrules = { float_equality = \"deny\" }\n";
        let mut config = parse(text).unwrap();
        let at = |c: &Config, r: &str, p: &str| c.level_at(rule(r), Path::new(p));
        assert_eq!(
            at(&config, "float_equality", "/r/stdlib/a.jai"),
            Level::Warn
        );
        assert_eq!(
            at(&config, "float_equality", "/r/tests/a.jai"),
            Level::Allow
        );
        assert_eq!(
            at(&config, "float_equality", "/r/tests/strict/a.jai"),
            Level::Deny
        );
        assert_eq!(at(&config, "lossy_xx", "/r/tests/a.jai"), Level::Warn);
        assert_eq!(at(&config, "lossy_xx", "/r/stdlib/a.jai"), Level::Allow);
        // Command-line levels win everywhere.
        config.flags.insert("float_equality".into(), Level::Deny);
        assert_eq!(at(&config, "float_equality", "/r/tests/a.jai"), Level::Deny);
        assert!(parse("[[lint.overrides]]\nrules = { lossy_xx = \"warn\" }\n").is_err());
        assert!(
            parse("[[lint.overrides]]\npaths = [\"a\"]\nrules = { lossy = \"warn\" }\n").is_err()
        );
    }

    #[test]
    fn portability_settings() {
        let text = "[lint.portability]\nportable = [\"lib/**\"]\n[lint.portability.platforms]\n\"lib/posix/**\" = [\"linux\", \"macos\"]\n\"lib/posix/mac/**\" = [\"macos\"]\n\"lib/posix/win*.jai\" = [\"windows\"]\n";
        let config = parse(text).unwrap();
        assert_eq!(config.portability.targets(), DEFAULT_PORTABILITY_TARGETS);
        assert!(config.is_portable(Path::new("/r/lib/a.jai")));
        assert!(!config.is_portable(Path::new("/r/app/a.jai")));
        let on = |p: &str, os| config.builds_on(Path::new(p), os);
        assert!(on("/r/lib/a.jai", TargetOs::Windows));
        assert!(on("/r/lib/posix/a.jai", TargetOs::Linux));
        assert!(!on("/r/lib/posix/a.jai", TargetOs::Windows));
        assert!(!on("/r/lib/posix/mac/a.jai", TargetOs::Linux));
        assert!(on("/r/lib/posix/mac/a.jai", TargetOs::MacOS));
        // The more specific glob decides, whatever order they are written in.
        assert!(on("/r/lib/posix/win32.jai", TargetOs::Windows));
        assert!(!on("/r/lib/posix/win32.jai", TargetOs::Linux));
        assert!(
            parse("[lint.portability]\ntargets = [\"beos\"]\n").is_err_and(|e| e.contains("beos"))
        );
        assert!(parse("[lint.portability.platforms]\n\"a\" = [\"wasm\"]\n").is_err());
    }

    #[test]
    fn platforms_comment_must_come_before_code() {
        assert_eq!(
            file_platforms("// notes\n// jailint: platforms(linux, macos, beos)\nx :: 1;\n"),
            Some(vec![TargetOs::Linux, TargetOs::MacOS])
        );
        assert_eq!(file_platforms("// jailint: platforms()\n"), Some(vec![]));
        assert_eq!(
            file_platforms("x :: 1;\n// jailint: platforms(linux)\n"),
            None
        );
        assert_eq!(file_platforms("// jailint: allow(a)\n"), None);
    }

    #[test]
    fn rejects_unknown_rules_and_levels() {
        assert!(parse("[lint.rules]\nno_such_rule = \"warn\"").is_err());
        assert!(parse("[lint.rules]\nlossy_xx = \"loud\"").is_err());
        assert!(parse("[lint]\nverbose = true\n").is_ok());
    }

    #[test]
    fn rejects_empty_excludes() {
        assert!(
            parse("[lint]\nexclude = [\"a.jai\", \"\"]\n").is_err_and(|e| e.contains("entry 2"))
        );
        assert!(parse("[lint]\nexclude = [\"/\"]\n").is_err());
    }

    #[test]
    fn rejects_invalid_patterns_with_their_line() {
        let error = parse("[lint]\nmax_nesting = 3\nexclude = [\"a[\"]\n").unwrap_err();
        assert!(
            error.contains("line 3") && error.contains("`a[`"),
            "{error}"
        );
        assert!(parse("[[lint.overrides]]\npaths = [\"{a\"]\nrules = {}\n").is_err());
        let config = parse(
            "[[lint.overrides]]\npaths = [\"src/*.{jai,c}\"]\nrules = { lossy_xx = \"deny\" }\n",
        )
        .unwrap();
        let at = |p: &str| config.level_at(rule("lossy_xx"), Path::new(p));
        assert_eq!(at("/r/src/a.c"), Level::Deny);
        assert_eq!(at("/r/src/a.h"), Level::Allow);
    }
}
