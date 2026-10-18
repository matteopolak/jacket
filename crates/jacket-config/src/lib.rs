//! `jacket.toml`: the one configuration file of the Jai tools.
//!
//! - [`schema`]: the types a `jacket.toml` deserializes into.
//! - [`locate`] / [`load`]: the nearest config directory above a path, and its settings. A
//!   directory with a `jacket.toml` uses only that; a directory with only the legacy files
//!   (`jai.toml`, `jailint.toml`, `jaifmt.toml`) is read through the same migration code that
//!   `jacket config migrate` writes with, so both always agree.
//! - [`migrate`]: legacy files to the text of a `jacket.toml`, comments kept.
//! - [`infer_entries`]: which file a project builds, shared by `jaic` and `jailsp`.
//!
//! Every Rust tool reads the file here; `jaifmt` (written in Jai) reads its `[format]` section
//! itself. See `docs/tools/jacket.md`.
#![forbid(unsafe_code)]
use std::path::{Path, PathBuf};

pub mod migrate;
pub mod schema;

pub use schema::{Config, Experimental, Format, Lint, Project, Toolchain};

/// The configuration file name.
pub const FILE_NAME: &str = "jacket.toml";

/// Legacy files: `jai.toml` (project), `jailint.toml` and `jaifmt.toml`.
pub const LEGACY_PROJECT: &str = "jai.toml";

pub const LEGACY_LINT: &str = "jailint.toml";
pub const LEGACY_FORMAT: &str = "jaifmt.toml";
pub const LEGACY_FILES: [&str; 3] = [LEGACY_PROJECT, LEGACY_LINT, LEGACY_FORMAT];

/// Entry files tried, in order, when `[project] entry` names none: a build metaprogram, a
/// program, then a library's root.
pub const INFERRED_ENTRIES: &[&str] = &[
    "build.jai",
    "first.jai",
    "main.jai",
    "src/main.jai",
    "module.jai",
];

/// The files a lookup sees: the disk, or an editor's open buffers over it.
pub trait Files {
    fn is_file(&self, path: &Path) -> bool;
    fn read(&self, path: &Path) -> Option<String>;
}

pub struct Disk;

impl Files for Disk {
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn read(&self, path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }
}

/// The nearest directory holding a config file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub dir: PathBuf,
    /// `dir` has a `jacket.toml`.
    pub jacket: bool,
    /// The legacy file names `dir` has (all of them are ignored when `jacket` is set).
    pub legacy: Vec<&'static str>,
}

impl Location {
    pub fn file(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }
}

/// The nearest directory at or above `start` (a directory) with a `jacket.toml` or a legacy file.
pub fn locate(fs: &dyn Files, start: &Path) -> Option<Location> {
    start.ancestors().find_map(|dir| {
        let jacket = fs.is_file(&dir.join(FILE_NAME));
        let legacy: Vec<&'static str> = LEGACY_FILES
            .into_iter()
            .filter(|n| fs.is_file(&dir.join(n)))
            .collect();
        (jacket || !legacy.is_empty()).then(|| Location {
            dir: dir.to_path_buf(),
            jacket,
            legacy,
        })
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
    /// Legacy files the settings were read from (empty with a `jacket.toml`).
    pub from_legacy: Vec<PathBuf>,
    /// One-line notes for the user: a legacy file ignored beside a `jacket.toml`, a legacy file
    /// that should be migrated, keys the schema does not know.
    pub notes: Vec<String>,
}

impl Loaded {
    pub fn dir(&self) -> &Path {
        &self.location.dir
    }

    /// The file `section` (`project`, `lint`, `format`, ...) was read from.
    pub fn source_of(&self, section: &str) -> PathBuf {
        if self.location.jacket {
            return self.location.file();
        }
        let legacy = match section {
            "project" => LEGACY_PROJECT,
            "lint" => LEGACY_LINT,
            "format" => LEGACY_FORMAT,
            _ => return self.location.file(),
        };
        self.location.dir.join(legacy)
    }
}

/// The settings of the nearest config directory above `start`; `Ok(None)` when there is none.
pub fn load(fs: &dyn Files, start: &Path) -> Result<Option<Loaded>, ConfigError> {
    match locate(fs, start) {
        Some(location) => load_at(fs, location).map(Some),
        None => Ok(None),
    }
}

/// The settings of a located directory.
pub fn load_at(fs: &dyn Files, location: Location) -> Result<Loaded, ConfigError> {
    let mut notes = Vec::new();
    let (config, from_legacy) =
        if location.jacket {
            let path = location.file();
            let text = fs.read(&path).unwrap_or_default();
            let config = parse(&text).map_err(|message| ConfigError {
                path: path.clone(),
                message,
            })?;
            // The old files stay after a migration, so a file `jacket config migrate` wrote is not
            // worth a note on every command; one written by hand next to them is.
            if !location.legacy.is_empty() && !text.starts_with(migrate::HEADER_PREFIX) {
                notes.push(format!(
                    "note: {} has {FILE_NAME}; ignoring {}",
                    location.dir.display(),
                    location.legacy.join(", ")
                ));
            }
            (config, Vec::new())
        } else {
            let plan = migrate::plan(fs, &location.dir)?;
            notes.push(format!(
            "note: {} in {} {} the legacy config; run `jacket config migrate` to write {FILE_NAME}",
            location.legacy.join(", "),
            location.dir.display(),
            if location.legacy.len() == 1 { "is" } else { "are" }
        ));
            notes.extend(plan.warnings.iter().cloned());
            let config = parse(&plan.text).map_err(|message| {
                // The file whose keys alone already do not fit the schema.
                let culprit = location.legacy.iter().find(|name| {
                    migrate::plan_only(fs, &location.dir, Some(name))
                        .is_ok_and(|p| parse(&p.text).is_err())
                });
                ConfigError {
                    path: location.dir.join(culprit.unwrap_or(&location.legacy[0])),
                    message,
                }
            })?;
            (config, plan.sources)
        };
    for key in config.unknown_keys() {
        notes.push(format!("warning: unknown key `{key}` in {FILE_NAME}"));
    }
    Ok(Loaded {
        config,
        location,
        from_legacy,
        notes,
    })
}

/// The settings in the text of a `jacket.toml`, or why it is invalid (with its line).
pub fn parse(text: &str) -> Result<Config, String> {
    toml::from_str(text).map_err(|e| e.to_string().trim_end().to_string())
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
