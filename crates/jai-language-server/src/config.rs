//! The nearest `jacket.toml` of a path, read through the session's saved snapshots (the client sends
//! initial settings with `didOpen` and commits edits with `didSave`, also in the browser).
//!
//! `[lint]` is checked here too, so a rule typo is one problem reported once, whichever feature
//! reads the file first. A file with a problem gives no settings (`Broken`): what depends on them
//! is left out until it is fixed, never run with defaults in their place.
use crate::auto_import::Index;
use crate::session::Session;
use jacket_config::{Files, Loaded};
use std::path::Path;

/// A `jacket.toml` that does not read, or whose checks found a problem. The problems are the
/// file's diagnostics (`Session::manifest_diagnostics`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Broken;

/// The settings above `start`: `Ok(None)` when no `jacket.toml` is there.
pub(crate) fn load(
    files: &dyn Files,
    index: &mut Index,
    start: &Path,
) -> Result<Option<Loaded>, Broken> {
    let Some(location) = jacket_config::locate(files, start) else {
        return Ok(None);
    };
    let stamp = crate::auto_import::hash(&files.read(&location.file()).unwrap_or_default());
    if let Some((seen, loaded)) = index.configs.get(&location.dir)
        && *seen == stamp
    {
        return loaded.clone().map(Some).ok_or(Broken);
    }
    let dir = location.dir.clone();
    let loaded = read(files, index, location);
    index.configs.insert(dir, (stamp, loaded.clone()));
    loaded.map(Some).ok_or(Broken)
}

fn read(files: &dyn Files, index: &mut Index, location: jacket_config::Location) -> Option<Loaded> {
    let file = location.file();
    let loaded = match jacket_config::load_at_checked(files, location, jailint::config::CHECKS) {
        Ok(loaded) => loaded,
        Err(error) => {
            index.bad_config(&error.path, &error.message);
            return None;
        }
    };
    if let Some(problem) = loaded.problems.first() {
        index.bad_config(&file, problem);
        return None;
    }
    index.good_config(&file);
    Some(loaded)
}

/// Whether the settings above `start` are broken.
pub(crate) fn is_broken(files: &dyn Files, index: &mut Index, start: &Path) -> bool {
    load(files, index, start).is_err()
}

/// Saved configuration snapshots and source buffers over the disk.
pub(crate) struct SessionFiles<'a>(pub &'a Session);

impl Files for SessionFiles<'_> {
    fn is_file(&self, path: &Path) -> bool {
        self.0.open_file_uri(path).is_some_and(|u| {
            self.0.saved_configs.contains_key(&u) || self.0.documents.contains_key(&u)
        }) || self
            .0
            .environment
            .as_ref()
            .map_or_else(|| path.is_file(), |env| env.fs.is_file(path))
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        if let Some(uri) = self.0.open_file_uri(path) {
            if let Some(text) = self.0.saved_configs.get(&uri) {
                return Ok(text.clone());
            }
            if let Some(doc) = self.0.documents.get(&uri) {
                return Ok(doc.text.clone());
            }
        }
        self.0.read_config_on_disk(path)
    }
}

impl Files for crate::auto_import::Sources<'_> {
    fn is_file(&self, path: &Path) -> bool {
        crate::auto_import::Sources::is_file(self, path)
    }

    fn read(&self, path: &Path) -> std::io::Result<String> {
        crate::auto_import::Sources::read(self, path)
            .ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}
