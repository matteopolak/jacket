//! The linter's settings: the `[lint]` section of `jacket.toml` (or a legacy `jailint.toml`,
//! migrated on the fly by `jacket-config`).
//!
//! ```toml
//! [lint]
//! exclude = ["tests/corpus/**"]
//! max_nesting = 6
//!
//! [lint.rules]
//! float_equality = "warn"
//! lossy_xx = "allow"
//! ```
//!
//! Anything the schema does not know under `[lint]` is an error, so a typo does not silently
//! keep a rule on.
use jacket_config::{Disk, Files, Lint};
use jaic::fxhash::HashMap;
use std::path::{Path, PathBuf};

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

#[derive(Clone, Debug, Default)]
pub struct Config {
    /// Levels set for rules; others keep their default.
    pub levels: HashMap<String, Level>,
    /// Warnings become errors (`-D warnings`).
    pub deny_warnings: bool,
    /// Glob patterns of paths not to lint, relative to `root`.
    pub exclude: Vec<String>,
    /// How many blocks deep code may nest before `excessive_nesting` fires.
    pub max_nesting: Option<usize>,
    /// Numbers `magic_number` lets through, besides its built-in ones.
    pub magic_number_allow: Vec<f64>,
    /// Whole numbers below this magnitude are not magic.
    pub magic_number_min: Option<f64>,
    /// `magic_number` also checks files in `tests` folders.
    pub magic_number_in_tests: Option<bool>,
    /// Directory of the configuration file (or the working directory).
    pub root: PathBuf,
    /// The file the settings came from, if any.
    pub source: Option<PathBuf>,
    /// One-line notes about the configuration (a legacy file in use or ignored).
    pub notes: Vec<String>,
}

/// The nesting depth `excessive_nesting` allows when nothing sets it.
pub const DEFAULT_MAX_NESTING: usize = 6;

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

    /// The level of `rule`, whose default is `default`.
    pub fn level(&self, rule: &str, default: Level) -> Level {
        let level = self.levels.get(rule).copied().unwrap_or(default);
        if self.deny_warnings && level == Level::Warn {
            Level::Deny
        } else {
            level
        }
    }

    /// Settings from a `[lint]` section whose paths are relative to `root`.
    pub fn from_lint(lint: &Lint, root: &Path) -> Result<Config, String> {
        let mut config = Config {
            root: root.to_path_buf(),
            exclude: lint.exclude.clone(),
            max_nesting: lint.max_nesting,
            magic_number_allow: lint.magic_number_allow.clone(),
            magic_number_min: lint.magic_number_min,
            magic_number_in_tests: lint.magic_number_in_tests,
            ..Config::default()
        };
        for (rule, level) in &lint.rules {
            check_rule(rule, level)?;
            let level = Level::parse(level).expect("`check_rule` accepts a level");
            config.levels.insert(rule.clone(), level);
        }
        Ok(config)
    }

    /// The settings of the nearest config directory above `start`; `Ok(None)` when there is
    /// none. `files` is the disk, or an editor's buffers over it.
    pub fn find(files: &dyn Files, start: &Path) -> Result<Option<Config>, String> {
        match jacket_config::load(files, start) {
            Ok(Some(loaded)) => Self::from_loaded(loaded).map(Some),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Settings of a `jacket.toml` or legacy `jailint.toml` at `path`.
    pub fn load(path: &Path) -> Result<Config, String> {
        // Paths checked against `exclude` are canonical, so the root must be too.
        let dir = match path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir,
            _ => Path::new("."),
        };
        let dir = &dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "could not read config `{}`: {}",
                path.display(),
                jaic::io_reason(&e)
            )
        })?;
        let mut config = if name == jacket_config::LEGACY_LINT {
            // One legacy file, named: read it alone, as migration would.
            let fs = One(path.to_path_buf(), text);
            let loaded = jacket_config::load_at(
                &fs,
                jacket_config::Location {
                    dir: dir.clone(),
                    jacket: false,
                    legacy: vec![jacket_config::LEGACY_LINT],
                },
            )
            .map_err(|e| e.to_string())?;
            Self::from_loaded(loaded)?
        } else {
            let (parsed, unknown) = jacket_config::parse_noting(&text)
                .map_err(|e| format!("in `{}`, {e}", path.display()))?;
            let mut config = Self::from_lint(&parsed.lint, dir)
                .map_err(|e| format!("in `{}`, {e}", path.display()))?;
            config.notes = unknown
                .iter()
                .map(|key| jacket_config::unknown_note(key))
                .collect();
            config
        };
        config.source = Some(path.to_path_buf());
        Ok(config)
    }

    fn from_loaded(loaded: jacket_config::Loaded) -> Result<Config, String> {
        let source = loaded.source_of("lint");
        let mut config = Config::from_lint(&loaded.config.lint, loaded.dir())
            .map_err(|e| format!("in `{}`, {e}", source.display()))?;
        config.source = Some(source);
        config.notes = loaded.notes;
        Ok(config)
    }

    /// The nearest settings on disk above `start`.
    pub fn find_on_disk(start: &Path) -> Result<Option<Config>, String> {
        Self::find(&Disk, start)
    }

    /// Whether `path` matches an `exclude` pattern.
    pub fn excluded(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        let text = relative.to_string_lossy().replace('\\', "/");
        self.exclude.iter().any(|p| glob(p, &text))
    }
}

/// Whether `rule` is a rule and `value` a level for it.
fn check_rule(rule: &str, value: &str) -> Result<(), String> {
    if !crate::RULES.iter().any(|r| r.name == rule) {
        let help = match jaic::suggest::closest(rule, crate::RULES.iter().map(|r| r.name)) {
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

/// One file in memory, for reading a named legacy config.
struct One(PathBuf, String);

impl Files for One {
    fn is_file(&self, path: &Path) -> bool {
        path == self.0
    }

    fn read(&self, path: &Path) -> Option<String> {
        (path == self.0).then(|| self.1.clone())
    }
}

/// Glob match: `*` is any run of characters but `/`, `**` any run including `/`, `?` one
/// character. A pattern naming a directory also matches what is inside it.
pub fn glob(pattern: &str, path: &str) -> bool {
    fn go(p: &[u8], s: &[u8]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some(b'*') if p.get(1) == Some(&b'*') => {
                // `**/x` matches `x` at the top or after any `/`, never mid-name.
                let rest = &p[2..];
                rest.strip_prefix(b"/").is_some_and(|top| go(top, s))
                    || (0..=s.len()).any(|i| go(rest, &s[i..]))
            }
            Some(b'*') => {
                let mut i = 0;
                loop {
                    if go(&p[1..], &s[i..]) {
                        return true;
                    }
                    if i == s.len() || s[i] == b'/' {
                        return false;
                    }
                    i += 1;
                }
            }
            Some(b'?') => !s.is_empty() && s[0] != b'/' && go(&p[1..], &s[1..]),
            Some(c) => s.first() == Some(c) && go(&p[1..], &s[1..]),
        }
    }
    let pattern = pattern.trim_end_matches('/');
    go(pattern.as_bytes(), path.as_bytes())
        || path
            .char_indices()
            .filter(|&(_, c)| c == '/')
            .any(|(i, _)| go(pattern.as_bytes(), &path.as_bytes()[..i]))
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
        assert_eq!(config.exclude, ["tests/**", "a.jai"]);
        assert_eq!(config.level("float_equality", Level::Allow), Level::Deny);
        assert_eq!(config.level("lossy_xx", Level::Warn), Level::Warn);
    }

    #[test]
    fn rejects_unknown_rules_and_levels() {
        assert!(parse("[lint.rules]\nno_such_rule = \"warn\"").is_err());
        assert!(parse("[lint.rules]\nlossy_xx = \"loud\"").is_err());
        assert!(parse("[lint]\nverbose = true\n").is_ok());
    }

    #[test]
    fn globs() {
        assert!(glob("tests/**", "tests/corpus/a.jai"));
        assert!(glob("tests/corpus", "tests/corpus/a.jai"));
        assert!(glob("*.jai", "a.jai"));
        assert!(!glob("*.jai", "dir/a.jai"));
        assert!(glob("**/gen_*.jai", "x/y/gen_a.jai"));
        assert!(glob("**/gen_*.jai", "gen_a.jai"));
        assert!(!glob("**/gen_*.jai", "x/my_gen_a.jai"));
        assert!(!glob("**/test.jai", "latest.jai"));
        assert!(glob("a/**/b", "a/x/y/b"));
        assert!(glob("a/**/b", "a/b"));
        assert!(!glob("a/**/b", "a/xb"));
        assert!(!glob("tests/**", "stdlib/a.jai"));
    }
}
