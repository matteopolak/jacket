//! The project a document belongs to: its program's entry file (what `jaic build` is given),
//! the files that entry `#load`s, and the folders modules are imported from. Auto-import
//! completion (`auto_import.rs`) uses it to tell a name the program already sees from one a
//! `#load` or `#import` would bring in.
//!
//! Where it comes from, first match wins:
//! 1. `jai.toml`, the nearest one at or above the document (an open one before the one on
//!    disk): `build_files` are the entry files, `import_path` adds module folders; paths are
//!    relative to its folder.
//! 2. Otherwise the workspace folder holding the document, with the entry files inferred: the
//!    first of `build.jai`, `first.jai`, `main.jai` and `src/main.jai` that exist (all of them
//!    are tried as entries).
//! 3. A document no entry reaches belongs to the program of the file that `#load`s it, walking
//!    up the loads among the files of its folder; a file nothing loads is its own entry.
use std::path::{Component, Path, PathBuf};

pub const CONFIG_FILE: &str = "jai.toml";

/// Entry files tried, in order, when no `jai.toml` names them.
pub const INFERRED_ENTRIES: &[&str] = &["build.jai", "first.jai", "main.jai", "src/main.jai"];

/// The settings of a `jai.toml`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    /// `build_files = ["first.jai"]`: the files given to `jaic build` (or `add_build_file`).
    pub build_files: Vec<String>,
    /// `import_path = ["vendor"]`: more module folders, as `-import_dir` (and
    /// `Build_Options.import_path`) add.
    pub import_path: Vec<String>,
}

impl ProjectConfig {
    /// The settings in the text of a `jai.toml`, or why it is invalid (with its line).
    pub fn parse(text: &str) -> Result<ProjectConfig, String> {
        let mut config = ProjectConfig::default();
        let mut pending: Option<(usize, String, String)> = None;
        for (number, raw) in text.lines().enumerate() {
            let line = strip_comment(raw).trim().to_string();
            if let Some((at, key, mut value)) = pending.take() {
                value.push(' ');
                value.push_str(&line);
                if value.trim_end().ends_with(']') {
                    config.set(&key, &value, at)?;
                } else {
                    pending = Some((at, key, value));
                }
                continue;
            }
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') && !line.contains('=') {
                return Err(format!(
                    "line {}: `jai.toml` has no tables; its settings are `build_files` and `import_path`",
                    number + 1
                ));
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!(
                    "line {}: expected `key = value`, found `{line}`",
                    number + 1
                ));
            };
            let (key, value) = (key.trim().to_string(), value.trim().to_string());
            if value.starts_with('[') && !value.ends_with(']') {
                pending = Some((number + 1, key, value));
                continue;
            }
            config.set(&key, &value, number + 1)?;
        }
        if let Some((at, ..)) = pending {
            return Err(format!("line {at}: this array is never closed"));
        }
        Ok(config)
    }

    fn set(&mut self, key: &str, value: &str, line: usize) -> Result<(), String> {
        let list = match key {
            "build_files" => &mut self.build_files,
            "import_path" => &mut self.import_path,
            other => {
                return Err(format!(
                    "line {line}: unknown setting `{other}`; the settings are `build_files` and `import_path`"
                ));
            }
        };
        // A single string is a list of one.
        if let Some(one) = string(value) {
            list.push(one);
            return Ok(());
        }
        let inner = value
            .strip_prefix('[')
            .and_then(|v| v.strip_suffix(']'))
            .ok_or_else(|| format!("line {line}: `{key}` takes an array of strings"))?;
        for item in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            list.push(
                string(item).ok_or_else(|| format!("line {line}: expected a quoted string"))?,
            );
        }
        Ok(())
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

/// The contents of a `"string"`.
fn string(text: &str) -> Option<String> {
    let inner = text.trim().strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then(|| inner.to_string())
}

/// `path` with `.` and `..` resolved without touching the disk.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
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

/// `target` relative to the folder `from`, with `/` separators (`../util/strings.jai`).
pub fn relative(from: &Path, target: &Path) -> String {
    let from: Vec<Component> = from.components().collect();
    let to: Vec<Component> = target.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec!["..".into(); from.len() - common];
    parts.extend(
        to[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_build_files_and_import_path() {
        let config = ProjectConfig::parse(
            "# The editor's settings\nbuild_files = [\"first.jai\", # metaprogram\n  \"src/main.jai\"]\nimport_path = \"vendor\"\n",
        )
        .unwrap();
        assert_eq!(config.build_files, ["first.jai", "src/main.jai"]);
        assert_eq!(config.import_path, ["vendor"]);
        assert!(
            ProjectConfig::parse("roots = [\"a.jai\"]")
                .unwrap_err()
                .contains("unknown setting `roots`")
        );
        assert!(ProjectConfig::parse("[project]").is_err());
    }

    #[test]
    fn relative_paths() {
        assert_eq!(
            relative(Path::new("/p/src"), Path::new("/p/src/ui/draw.jai")),
            "ui/draw.jai"
        );
        assert_eq!(
            relative(Path::new("/p/src/ui"), Path::new("/p/util.jai")),
            "../../util.jai"
        );
        assert_eq!(
            normalize(Path::new("/p/src/../a/./b.jai")),
            Path::new("/p/a/b.jai")
        );
    }
}
