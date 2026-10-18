//! `jailint.toml`: rule levels and excluded paths.
//!
//! ```toml
//! exclude = ["tests/corpus/**"]
//!
//! [rules]
//! float_equality = "warn"
//! lossy_xx = "allow"
//! ```
//!
//! Only this small subset of TOML is read: top-level `exclude` (an array of glob patterns,
//! relative to the file's directory), `max_nesting` (an integer, for `excessive_nesting`) and a
//! `[rules]` table of `name = "allow" | "warn" |
//! "deny"`. Anything else is an error, so a typo does not silently keep a rule on.
use jaic::project::{self, ProjectConfig};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = jaic::project::LINT_TOML;

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
    /// Directory of the configuration file (or the working directory).
    pub root: PathBuf,
    /// The file the settings came from, if any.
    pub source: Option<PathBuf>,
}

/// The nesting depth `excessive_nesting` allows when nothing sets it.
pub const DEFAULT_MAX_NESTING: usize = 6;

impl Config {
    /// The deepest block nesting `excessive_nesting` accepts.
    pub fn max_nesting(&self) -> usize {
        self.max_nesting.unwrap_or(DEFAULT_MAX_NESTING)
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

    /// Settings from the text of a `jailint.toml` in `root`.
    pub fn parse(text: &str, root: &Path) -> Result<Config, String> {
        let parsed = ProjectConfig::parse_lint(text, check_rule)?;
        let mut config = Config {
            root: root.to_path_buf(),
            exclude: parsed.exclude,
            max_nesting: parsed.max_nesting,
            ..Config::default()
        };
        for (rule, level) in parsed.rules {
            let level = Level::parse(&level).expect("`check_rule` returns a level");
            config.levels.insert(rule, level);
        }
        Ok(config)
    }

    /// The nearest `jailint.toml` in `start` or a directory above it.
    pub fn find(start: &Path) -> Option<PathBuf> {
        project::find_config_dir(start, FILE_NAME, Path::is_file).map(|d| d.join(FILE_NAME))
    }

    /// Settings of the file at `path`.
    pub fn load(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            format!(
                "could not read config `{}`: {}",
                path.display(),
                jaic::io_reason(&e)
            )
        })?;
        let root = path.parent().unwrap_or(Path::new("."));
        let mut config =
            Config::parse(&text, root).map_err(|e| format!("in `{}`, {e}", path.display()))?;
        config.source = Some(path.to_path_buf());
        Ok(config)
    }

    /// Whether `path` matches an `exclude` pattern.
    pub fn excluded(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.root).unwrap_or(path);
        let text = relative.to_string_lossy().replace('\\', "/");
        self.exclude.iter().any(|p| glob(p, &text))
    }
}

/// Whether `rule` is a rule and `value` a level for it; the level it names.
fn check_rule(rule: &str, value: &str, line: usize) -> Result<String, String> {
    if !crate::RULES.iter().any(|r| r.name == rule) {
        let help = match jaic::suggest::closest(rule, crate::RULES.iter().map(|r| r.name)) {
            Some(near) => {
                format!("did you mean `{near}`? `jailint --list` lists the rules")
            }
            None => "`jailint --list` lists the rules".to_string(),
        };
        return Err(format!("line {line}: unknown rule `{rule}`\nhelp: {help}"));
    }
    project::toml_string(value)
        .filter(|v| Level::parse(v).is_some())
        .ok_or_else(|| {
            format!(
                "line {line}: unknown level {value} for `{rule}`\nhelp: use \"allow\", \"warn\" or \"deny\""
            )
        })
}

/// Glob match: `*` is any run of characters but `/`, `**` any run including `/`, `?` one
/// character. A pattern naming a directory also matches what is inside it.
pub fn glob(pattern: &str, path: &str) -> bool {
    fn go(p: &[u8], s: &[u8]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some(b'*') if p.get(1) == Some(&b'*') => {
                let rest = p[2..].strip_prefix(b"/").unwrap_or(&p[2..]);
                (0..=s.len()).any(|i| go(rest, &s[i..]))
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

    #[test]
    fn parses_rules_and_excludes() {
        let text = "# settings\nexclude = [\n  \"tests/**\", # fixtures\n  \"a.jai\",\n]\n\n[rules]\nfloat_equality = \"deny\" # strict\n";
        let config = Config::parse(text, Path::new("/r")).unwrap();
        assert_eq!(config.exclude, ["tests/**", "a.jai"]);
        assert_eq!(config.level("float_equality", Level::Allow), Level::Deny);
        assert_eq!(config.level("lossy_xx", Level::Warn), Level::Warn);
    }

    #[test]
    fn rejects_unknown_rules_and_levels() {
        assert!(Config::parse("[rules]\nno_such_rule = \"warn\"", Path::new("/")).is_err());
        assert!(Config::parse("[rules]\nlossy_xx = \"loud\"", Path::new("/")).is_err());
        assert!(Config::parse("[lint]\n", Path::new("/")).is_err());
    }

    #[test]
    fn globs() {
        assert!(glob("tests/**", "tests/corpus/a.jai"));
        assert!(glob("tests/corpus", "tests/corpus/a.jai"));
        assert!(glob("*.jai", "a.jai"));
        assert!(!glob("*.jai", "dir/a.jai"));
        assert!(glob("**/gen_*.jai", "x/y/gen_a.jai"));
        assert!(!glob("tests/**", "stdlib/a.jai"));
    }
}
