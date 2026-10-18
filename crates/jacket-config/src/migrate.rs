//! Legacy config files to a `jacket.toml`.
//!
//! The mapping (see `docs/tools/jacket.md`):
//!
//! | legacy | `jacket.toml` |
//! | --- | --- |
//! | `jai.toml` `build_files` | `[project] entry` |
//! | `jai.toml` `import_path` | `[project] import_path` |
//! | `jailint.toml` top-level keys, `[rules]` | `[lint]`, `[lint.rules]` |
//! | `jaifmt.toml` keys | `[format]` |
//!
//! Keys and tables are moved as parsed `toml_edit` items, so the comments and layout around
//! them come along. A key the legacy file's own tool did not know is kept in the section and
//! reported in [`Plan::warnings`].
use crate::{ConfigError, Files, LEGACY_FORMAT, LEGACY_LINT, LEGACY_PROJECT, Location};
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Key, Table};

/// A legacy file: its name, the section it becomes, and the keys renamed on the way in.
type Legacy<'a> = (&'a str, &'a str, &'a [(&'a str, &'a str)]);

/// What migrating one directory would write.
#[derive(Clone, Debug)]
pub struct Plan {
    /// The text of the `jacket.toml`.
    pub text: String,
    /// Keys kept that no schema section knows.
    pub warnings: Vec<String>,
    /// The legacy files merged, in the order of their sections.
    pub sources: Vec<PathBuf>,
}

/// How far a migration got.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `jacket.toml` already there; nothing to do.
    AlreadyMigrated(PathBuf),
    /// No legacy file in the directory.
    NothingToMigrate,
    /// Written (or, for a dry run, would be).
    Wrote { path: PathBuf, written: bool },
}

/// The `jacket.toml` the legacy files in `dir` make.
pub fn plan(fs: &dyn Files, dir: &Path) -> Result<Plan, ConfigError> {
    plan_only(fs, dir, None)
}

/// [`plan`] for the legacy file called `only`, or all of them.
pub fn plan_only(fs: &dyn Files, dir: &Path, only: Option<&str>) -> Result<Plan, ConfigError> {
    let mut out = DocumentMut::new();
    let mut warnings = Vec::new();
    let mut sources = Vec::new();
    let files: [Legacy; 3] = [
        (
            LEGACY_PROJECT,
            "project",
            &[("build_files", "entry"), ("import_path", "import_path")],
        ),
        (
            LEGACY_LINT,
            "lint",
            &[
                ("exclude", "exclude"),
                ("max_nesting", "max_nesting"),
                ("rules", "rules"),
            ],
        ),
        (LEGACY_FORMAT, "format", &[]),
    ];
    for (name, section, renames) in files {
        let path = dir.join(name);
        if !fs.is_file(&path) || only.is_some_and(|o| o != name) {
            continue;
        }
        let text = fs.read(&path).unwrap_or_default();
        let doc: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| ConfigError {
                path: path.clone(),
                message: e.to_string().trim_end().to_string(),
            })?;
        move_into(&mut out, &doc, section, name, renames, &mut warnings);
        sources.push(path);
    }
    let names: Vec<&str> = sources
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
        .collect();
    let mut text = format!(
        "# Written by `jacket config migrate` from {}.\n\n",
        names.join(", ")
    );
    text.push_str(&out.to_string());
    Ok(Plan {
        text,
        warnings,
        sources,
    })
}

fn move_into(
    out: &mut DocumentMut,
    doc: &DocumentMut,
    section: &str,
    file: &str,
    renames: &[(&str, &str)],
    warnings: &mut Vec<String>,
) {
    let mut target = Table::new();
    for (key, item) in doc.as_table() {
        let (name, known) = match renames.iter().find(|(from, _)| *from == key) {
            Some((_, to)) => (*to, true),
            // The formatter's keys are all `[format]` keys; the schema checks them on load.
            None => (key, section == "format"),
        };
        if !known {
            warnings.push(format!(
                "warning: unknown key `{key}` in {file}, kept in [{section}]"
            ));
        }
        let mut item = item.clone();
        if let Item::Table(table) = &mut item {
            // An empty `[rules]` says nothing.
            if table.is_empty()
                && table
                    .decor()
                    .prefix()
                    .is_none_or(|p| p.as_str().is_none_or(|t| t.trim().is_empty()))
            {
                continue;
            }
            table.set_position(None);
        }
        let mut new_key = Key::new(name);
        if let Some((old, _)) = doc.as_table().get_key_value(key) {
            *new_key.leaf_decor_mut() = old.leaf_decor().clone();
            *new_key.dotted_decor_mut() = old.dotted_decor().clone();
        }
        target.insert_formatted(&new_key, item);
    }
    let only_tables = target.iter().all(|(_, v)| v.is_table());
    target.set_implicit(only_tables);
    target.decor_mut().set_prefix("\n");
    if out.as_table().is_empty() {
        target.decor_mut().set_prefix("");
    }
    out.insert(section, Item::Table(target));
    let trailing = doc.trailing().as_str().unwrap_or("");
    if !trailing.trim().is_empty() {
        let mut keep = out.trailing().as_str().unwrap_or("").to_string();
        keep.push_str(trailing);
        out.set_trailing(keep);
    }
}

/// Writes the `jacket.toml` for the legacy files in `location`'s directory. Never touches the
/// legacy files. With `dry_run`, only reports.
pub fn run(
    fs: &dyn Files,
    location: &Location,
    dry_run: bool,
) -> Result<(Outcome, Option<Plan>), ConfigError> {
    let target = location.file();
    if location.jacket {
        return Ok((Outcome::AlreadyMigrated(target), None));
    }
    if location.legacy.is_empty() {
        return Ok((Outcome::NothingToMigrate, None));
    }
    let plan = plan(fs, &location.dir)?;
    let mut written = false;
    if !dry_run {
        std::fs::write(&target, &plan.text).map_err(|e| ConfigError {
            path: target.clone(),
            message: format!("could not write: {e}"),
        })?;
        written = true;
    }
    Ok((
        Outcome::Wrote {
            path: target,
            written,
        },
        Some(plan),
    ))
}

/// A one-line description of the written file, for the notice after an automatic migration.
pub fn summary(plan: &Plan, target: &Path) -> String {
    let from: Vec<String> = plan
        .sources
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    format!(
        "jacket: wrote {} from {} (the old files are kept)",
        target.display(),
        from.join(", ")
    )
}
