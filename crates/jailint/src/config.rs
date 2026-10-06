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
//! relative to the file's directory) and a `[rules]` table of `name = "allow" | "warn" |
//! "deny"`. Anything else is an error, so a typo does not silently keep a rule on.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "jailint.toml";

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
    /// Directory of the configuration file (or the working directory).
    pub root: PathBuf,
    /// The file the settings came from, if any.
    pub source: Option<PathBuf>,
}

impl Config {
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
        let mut config = Config {
            root: root.to_path_buf(),
            ..Config::default()
        };
        let mut table = String::new();
        let mut pending: Option<(usize, String, String)> = None;
        for (number, raw) in text.lines().enumerate() {
            let line = strip_comment(raw).trim().to_string();
            // An array continued over several lines.
            if let Some((at, key, mut value)) = pending.take() {
                value.push(' ');
                value.push_str(&line);
                if value.trim_end().ends_with(']') {
                    config.set(&table, &key, &value, at)?;
                } else {
                    pending = Some((at, key, value));
                }
                continue;
            }
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                table = name.trim().to_string();
                if table != "rules" {
                    return Err(format!("line {}: unknown table [{table}]", number + 1));
                }
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {}: expected `key = value`", number + 1));
            };
            let (key, value) = (key.trim().to_string(), value.trim().to_string());
            if value.starts_with('[') && !value.ends_with(']') {
                pending = Some((number + 1, key, value));
                continue;
            }
            config.set(&table, &key, &value, number + 1)?;
        }
        if let Some((at, ..)) = pending {
            return Err(format!("line {at}: unterminated array"));
        }
        Ok(config)
    }

    fn set(&mut self, table: &str, key: &str, value: &str, line: usize) -> Result<(), String> {
        match (table, key) {
            ("", "exclude") => {
                let inner = value
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| format!("line {line}: `exclude` takes an array of strings"))?;
                for item in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    self.exclude.push(
                        string(item)
                            .ok_or_else(|| format!("line {line}: expected a quoted string"))?,
                    );
                }
                Ok(())
            }
            ("", other) => Err(format!("line {line}: unknown setting `{other}`")),
            (_, rule) => {
                let rule = rule.trim_matches('"');
                if !crate::RULES.iter().any(|r| r.name == rule) {
                    return Err(format!("line {line}: unknown rule `{rule}`"));
                }
                let level = string(value)
                    .and_then(|v| Level::parse(&v))
                    .ok_or_else(|| {
                        format!("line {line}: `{rule}` must be \"allow\", \"warn\" or \"deny\"")
                    })?;
                self.levels.insert(rule.to_string(), level);
                Ok(())
            }
        }
    }

    /// The nearest `jailint.toml` in `start` or a directory above it.
    pub fn find(start: &Path) -> Option<PathBuf> {
        let mut dir = Some(start);
        while let Some(d) = dir {
            let candidate = d.join(FILE_NAME);
            if candidate.is_file() {
                return Some(candidate);
            }
            dir = d.parent();
        }
        None
    }

    /// Settings of the file at `path`.
    pub fn load(path: &Path) -> Result<Config, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let root = path.parent().unwrap_or(Path::new("."));
        let mut config =
            Config::parse(&text, root).map_err(|e| format!("{}: {e}", path.display()))?;
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

/// The line without a `#` comment (a `#` inside a string stays).
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

/// The contents of a `"string"` (no escapes are needed for paths and levels).
fn string(text: &str) -> Option<String> {
    let inner = text.trim().strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then(|| inner.to_string())
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
