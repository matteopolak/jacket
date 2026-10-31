//! `jacket.toml`: the one configuration file of the Jai tools.
//!
//! - [`env`]: every environment variable the tools read, declared once.
//! - [`schema`]: the types a `jacket.toml` deserializes into.
//! - [`locate`] / [`load`]: the nearest `jacket.toml` above a path, and its settings.
//! - [`infer_entries`]: which file a project builds, shared by `jaic` and `jailsp`.
//!
//! Every Rust tool reads the file here; `jacket fmt` resolves `[format]` and hands it to `jaifmt`,
//! whose own reader (standalone use, the browser) a test keeps in step. See `docs/tools/jacket.md`.
#![forbid(unsafe_code)]
use std::path::{Path, PathBuf};

pub mod env;
pub mod llvm;
pub mod pattern;
pub mod schema;

pub use pattern::{PathPattern, PatternError, relative_text};
pub use schema::{
    Config, Experimental, FORMAT_SETTINGS, Fallback, Format, Kind, Lint, LintOverride, Portability,
    Project, Setting, Test, TestRun, Toolchain,
};

/// The configuration file name.
pub const FILE_NAME: &str = "jacket.toml";

/// Entry files tried, in order, when `[project] entry` names none: a build metaprogram, a
/// program, then a library's root.
pub const INFERRED_ENTRIES: &[&str] = &[
    "build.jai",
    "first.jai",
    "main.jai",
    "src/main.jai",
    "module.jai",
];

/// `path` with every `\` written `/`: the spelling paths take in program text, module keys,
/// golden output and generated sources, so they read the same on every host.
pub fn to_slash(path: &str) -> String {
    path.replace('\\', "/")
}

/// [`to_slash`] of a path, lossily.
pub fn slashed(path: &Path) -> String {
    to_slash(&path.to_string_lossy())
}

/// Whether `path` names a settings file (`jacket.toml`).
pub fn is_config_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| name == FILE_NAME)
}

/// A tool's check of the settings it owns, run as the file is loaded: given the settings and the
/// folder holding the file, why they are invalid, if they are (`jailint` knows which rule and
/// operating-system names exist; this crate does not). A failed check does not fail the load: it
/// is recorded in [`Loaded::problems`] for the tool to report as it sees fit.
pub type Check = fn(&Config, &Path) -> Result<(), String>;

/// The files a lookup sees: the disk, or an editor's open buffers over it.
pub trait Files {
    fn is_file(&self, path: &Path) -> bool;

    /// The file's text, or why it could not be read.
    fn read(&self, path: &Path) -> std::io::Result<String>;
}

pub struct Disk;

impl Files for Disk {
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }
}

/// The nearest directory holding a `jacket.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub dir: PathBuf,
}

impl Location {
    pub fn file(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }
}

/// The nearest directory at or above `start` (a directory) with a `jacket.toml`.
pub fn locate(fs: &dyn Files, start: &Path) -> Option<Location> {
    start
        .ancestors()
        .find(|dir| fs.is_file(&dir.join(FILE_NAME)))
        .map(|dir| Location {
            dir: dir.to_path_buf(),
        })
}

/// A settings file that could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub path: PathBuf,
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in `{}`, {}", self.path.display(), self.message)
    }
}

impl std::error::Error for ConfigError {
}

/// Settings and where they came from.
#[derive(Clone, Debug)]
pub struct Loaded {
    pub config: Config,
    pub location: Location,
    /// One-line notes for the user: keys the schema does not know.
    pub notes: Vec<String>,
    /// What the [`Check`]s run at load found wrong, one message each.
    pub problems: Vec<String>,
}

impl Loaded {
    pub fn dir(&self) -> &Path {
        &self.location.dir
    }

    /// The `jacket.toml` the settings were read from.
    pub fn file(&self) -> PathBuf {
        self.location.file()
    }
}

/// The text of a config file that exists; one that cannot be read (permissions, not UTF-8) is an
/// error rather than empty settings.
fn read(fs: &dyn Files, path: &Path) -> Result<String, ConfigError> {
    fs.read(path).map_err(|error| ConfigError {
        path: path.to_path_buf(),
        message: format!("the file could not be read: {error}"),
    })
}

/// The settings of the nearest config directory above `start`; `Ok(None)` when there is none.
pub fn load(fs: &dyn Files, start: &Path) -> Result<Option<Loaded>, ConfigError> {
    load_checked(fs, start, &[])
}

/// [`load`], running `checks` on what is found.
pub fn load_checked(
    fs: &dyn Files,
    start: &Path,
    checks: &[Check],
) -> Result<Option<Loaded>, ConfigError> {
    match locate(fs, start) {
        Some(location) => load_at_checked(fs, location, checks).map(Some),
        None => Ok(None),
    }
}

/// The settings of a located directory.
pub fn load_at(fs: &dyn Files, location: Location) -> Result<Loaded, ConfigError> {
    load_at_checked(fs, location, &[])
}

/// [`load_at`], running `checks` on what is found.
pub fn load_at_checked(
    fs: &dyn Files,
    location: Location,
    checks: &[Check],
) -> Result<Loaded, ConfigError> {
    let path = location.file();
    let text = read(fs, &path)?;
    let (config, unknown) = parse_noting(&text).map_err(|message| ConfigError {
        path,
        message,
    })?;
    let problems = checks
        .iter()
        .filter_map(|check| check(&config, &location.dir).err())
        .collect();
    Ok(Loaded {
        config,
        location,
        notes: unknown.iter().map(|key| unknown_note(key)).collect(),
        problems,
    })
}

/// The settings in the text of a `jacket.toml`, or why it is invalid (with its line).
pub fn parse(text: &str) -> Result<Config, String> {
    parse_noting(text).map(|(config, _)| config)
}

/// Like [`parse`], also giving the dotted path of every key the schema does not know
/// (`lint.max_nestng`). Those keys are dropped from the settings; [`unknown_note`] words the
/// warning every tool prints for one.
pub fn parse_noting(text: &str) -> Result<(Config, Vec<String>), String> {
    let invalid = |e: toml::de::Error| e.to_string().trim_end().to_string();
    let deserializer = toml::de::Deserializer::parse(text).map_err(invalid)?;
    let mut unknown = Vec::new();
    let config = serde_ignored::deserialize(deserializer, |path| unknown.push(path.to_string()))
        .map_err(invalid)?;
    Ok((config, unknown))
}

/// The warning for an unknown setting at `path`, naming the nearest known one when there is one.
pub fn unknown_note(path: &str) -> String {
    let mut note = format!("warning: unknown setting `{path}` in {FILE_NAME} (ignored)");
    let (table, key) = path.rsplit_once('.').unwrap_or(("", path));
    if table.is_empty()
        && let Some((home, meant)) = schema::misplaced_home(key)
    {
        if meant == key {
            note.push_str(&format!("; it belongs under `[{home}]`"));
        } else {
            note.push_str(&format!("; use `{meant}` under `[{home}]`"));
        }
        return note;
    }
    if let Some(near) = schema::closest(key, schema::known_keys(table)) {
        let near = if table.is_empty() {
            near.to_string()
        } else {
            format!("{table}.{near}")
        };
        note.push_str(&format!("; did you mean `{near}`?"));
    }
    note
}

/// The line (1-based) of `key = ...` in `text`, to point an error at.
pub fn line_of(text: &str, key: &str) -> Option<usize> {
    text.lines()
        .position(|l| {
            let l = l.trim_start().trim_start_matches('"');
            l.strip_prefix(key).is_some_and(|rest| {
                let rest = rest.trim_start_matches('"').trim_start();
                rest.starts_with('=')
            })
        })
        .map(|i| i + 1)
}

/// The entry files of the project based at `base`.
pub struct EntryFiles {
    pub paths: Vec<PathBuf>,
    /// They are the configured `[project] entry`; otherwise they are the [`INFERRED_ENTRIES`],
    /// of which only those that exist are entries.
    pub declared: bool,
}

/// `base`'s entry files: `[project] entry`, else the inferred ones in order of preference.
pub fn entry_files(base: &Path, project: Option<&Project>) -> EntryFiles {
    match project {
        Some(project) if !project.entry.0.is_empty() => EntryFiles {
            paths: project.entry.0.iter().map(|f| base.join(f)).collect(),
            declared: true,
        },
        _ => EntryFiles {
            paths: INFERRED_ENTRIES.iter().map(|f| base.join(f)).collect(),
            declared: false,
        },
    }
}

/// The first entry file of `base` that exists (`jaic <directory>` builds it).
pub fn first_entry(fs: &dyn Files, base: &Path, project: Option<&Project>) -> Option<PathBuf> {
    entry_files(base, project)
        .paths
        .into_iter()
        .find(|p| fs.is_file(p))
}

/// The entry file `jaic <directory>` would build in `dir`: `[project] entry` of the nearest
/// `jacket.toml`, else the first inferred one that exists.
pub fn project_entry(fs: &dyn Files, dir: &Path) -> Option<PathBuf> {
    let project = load(fs, dir)
        .ok()
        .flatten()
        .map(|loaded| loaded.config.project);
    first_entry(fs, dir, project.as_ref())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod jai_format_sync;
