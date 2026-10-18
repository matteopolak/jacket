//! The nearest `jacket.toml` of a path, read through the session's open buffers (the client sends
//! a settings file it has open with `didOpen`, which is also how the browser build gets one).
//!
//! `[lint]` is checked here too, so a rule typo is one problem reported once, whichever feature
//! reads the file first. A legacy file beside a `jacket.toml` is ignored, and the server logs a
//! line saying so (`window/logMessage`, once).
use crate::auto_import::Index;
use crate::session::Session;
use jacket_config::{Files, Loaded};
use std::path::Path;

/// The settings above `start`, with problems and notes sent to the client. A file that does not
/// parse gives the default settings (and a message) until it is fixed.
pub(crate) fn load(files: &dyn Files, index: &mut Index, start: &Path) -> Option<Loaded> {
    let location = jacket_config::locate(files, start)?;
    let mut stamp = String::new();
    let names = std::iter::once(jacket_config::FILE_NAME).chain(jacket_config::LEGACY_FILES);
    for name in names {
        let path = location.dir.join(name);
        stamp.push_str(name);
        stamp.push('\0');
        stamp.push_str(&files.read(&path).unwrap_or_default());
        stamp.push('\0');
    }
    let stamp = crate::auto_import::hash(&stamp);
    if let Some((seen, loaded)) = index.configs.get(&location.dir)
        && *seen == stamp
    {
        return Some(loaded.clone());
    }
    let loaded = read(files, index, location)?;
    index
        .configs
        .insert(loaded.location.dir.clone(), (stamp, loaded.clone()));
    Some(loaded)
}

fn read(files: &dyn Files, index: &mut Index, location: jacket_config::Location) -> Option<Loaded> {
    let mut loaded = match jacket_config::load_at(files, location.clone()) {
        Ok(loaded) => loaded,
        Err(error) => {
            index.bad_config(&error.path, &error.message);
            return Some(Loaded {
                config: jacket_config::Config::default(),
                location,
                from_legacy: Vec::new(),
                notes: Vec::new(),
            });
        }
    };
    let source = loaded.source_of("lint");
    match jailint::config::Config::from_lint(&loaded.config.lint, loaded.dir()) {
        Ok(_) => {
            index.good_config(&loaded.location.file());
            index.good_config(&source);
        }
        Err(problem) => {
            index.bad_config(&source, &problem);
            loaded.config.lint = jacket_config::Lint::default();
        }
    }
    for note in &loaded.notes {
        index.log_once(note);
    }
    Some(loaded)
}

/// The open buffers over the disk.
pub(crate) struct SessionFiles<'a>(pub &'a Session);

impl Files for SessionFiles<'_> {
    fn is_file(&self, path: &Path) -> bool {
        self.0.open_file_uri(path).is_some() || path.is_file()
    }

    fn read(&self, path: &Path) -> Option<String> {
        if let Some(uri) = self.0.open_file_uri(path)
            && let Some(doc) = self
                .0
                .documents
                .get(&uri)
                .or_else(|| self.0.lint_configs.get(&uri))
        {
            return Some(doc.text.clone());
        }
        std::fs::read_to_string(path).ok()
    }
}

impl Files for crate::auto_import::Sources<'_> {
    fn is_file(&self, path: &Path) -> bool {
        crate::auto_import::Sources::is_file(self, path)
    }

    fn read(&self, path: &Path) -> Option<String> {
        crate::auto_import::Sources::read(self, path)
    }
}
