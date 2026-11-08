//! Which program a file belongs to: the project's files, its `jacket.toml`, the file a fragment is
//! analysed from, and the folders and entries that make up one program.
use super::index_fs::IndexFs;
use super::{Index, Program, SKIPPED_FOLDERS, Sources};
use crate::project::{self, ProjectConfig, ProjectFs};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The sources as a bare file system: the shared tree walk of [`ProjectFs::project_files`]
/// over their directory listings, without the index that [`IndexFs`] caches the walk in.
struct Listing<'a, 's>(&'a Sources<'s>);

impl ProjectFs for Listing<'_, '_> {
    fn key(&self, path: &Path) -> PathBuf {
        self.0.key(path)
    }

    fn read(&self, path: &Path) -> Option<String> {
        self.0.read(path)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.0.is_file(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.0.is_dir(path)
    }

    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        self.0.list_dir(dir)
    }
}

impl Sources<'_> {
    /// Validate compiler-observed ownership only for sources whose buffers or disk state changed.
    fn refresh_source_graphs(&self, index: &mut Index) {
        for path in std::mem::take(&mut index.source_graphs_dirty) {
            let stamp = self
                .read(&path)
                .map(|text| crate::semantic::text_hash(text.as_bytes()));
            for (root, facts) in &index.builds {
                if facts.dependencies.concerns(&path) {
                    let stale = index.stale_source_graphs.entry(root.clone()).or_default();
                    if stamp.is_some() && stamp == facts.sources.get(&path).copied() {
                        stale.remove(&path);
                    } else {
                        stale.insert(path.clone());
                    }
                }
            }
        }
        index
            .stale_source_graphs
            .retain(|_, paths| !paths.is_empty());
    }

    /// The `.jai` files under `dir` (cached walk of the disk, plus open documents).
    pub(super) fn project_files(&self, index: &mut Index, dir: &Path) -> Vec<PathBuf> {
        let key = self.key(dir);
        let dir = key.as_path();
        let walked = if let Some(files) = index.walked.get(dir) {
            files.clone()
        } else {
            let files = Listing(self).project_files(dir);
            let files = Rc::new(files);
            index.walked.insert(dir.to_path_buf(), files.clone());
            files
        };
        let mut all: BTreeSet<PathBuf> = walked.iter().cloned().collect();
        all.extend(
            self.open
                .keys()
                .filter(|p| {
                    p.starts_with(dir)
                        && p.extension().is_some_and(|e| e == "jai")
                        && !p
                            .strip_prefix(dir)
                            .unwrap_or(p)
                            .parent()
                            .is_some_and(|rest| {
                                rest.iter().any(|c| {
                                    let c = c.to_string_lossy();
                                    c.starts_with('.') || SKIPPED_FOLDERS.contains(&&*c)
                                })
                            })
                })
                .cloned(),
        );
        all.into_iter().collect()
    }

    /// The nearest config directory at or above `file` that configures a project (has a
    /// `[project]` section): its folder and settings (the defaults when the file does not
    /// parse). A `jacket.toml` that only holds `[lint]` or `[format]` does not make its folder
    /// a project, so a repository-wide one does not make every file in it part of one program.
    pub(crate) fn config(
        &self,
        index: &mut Index,
        file: &Path,
    ) -> Option<(PathBuf, ProjectConfig)> {
        let mut dir = file.parent()?.to_path_buf();
        loop {
            let loaded = crate::config::load(self, index, &dir)?;
            if !loaded.config.project.is_empty() {
                return Some((loaded.location.dir, loaded.config.project));
            }
            dir = loaded.location.dir.parent()?.to_path_buf();
        }
    }

    /// The file a fragment is analysed from: `file`, then whoever `#load`s it, and whoever loads
    /// that, until a file nothing loads. Loaders are looked for among the files of the library
    /// tree holding `file` (the standard library, whose `Keymap/module.jai` loads
    /// `internal/input/keymap-text.jai`) or of the project folder, found through the cached
    /// scans of those files. Where several files load one, the `module.jai` goes first, then
    /// the shallowest path, then the first by name, so the same file always gets the same owner.
    pub(super) fn loader_chain(
        &self,
        index: &mut Index,
        file: &Path,
        base: Option<&Path>,
    ) -> PathBuf {
        let stdlib = (self.env.options)(file).stdlib_root().map(|p| self.key(p));
        let search = match stdlib {
            Some(s) if file.starts_with(&s) => Some(s),
            _ => base.map(Path::to_path_buf),
        };
        // A loader next to or above the file also counts when there is no folder to search.
        let options = project::RootOptions::language_server(search);
        let fs = IndexFs::new(self, index);
        match project::enclosing_program(
            &fs,
            file,
            &options.search,
            options.tie_break,
            &mut project::LoadCache::default(),
        ) {
            Some(root) => root,
            None => file.to_path_buf(),
        }
    }

    pub(crate) fn workspace_entries(
        &self,
        index: &mut Index,
        folders: &[PathBuf],
        open: &[PathBuf],
    ) -> BTreeSet<PathBuf> {
        self.refresh_source_graphs(index);
        let mut entries = BTreeSet::new();
        let mut declared = BTreeSet::new();
        for folder in folders {
            let config = self.config(index, &folder.join("main.jai"));
            let base = config
                .as_ref()
                .map_or(folder.as_path(), |(dir, _)| dir.as_path());
            let found = project::entry_files(base, config.as_ref().map(|(_, c)| c));
            for path in found.paths.into_iter().filter(|p| self.is_file(p)) {
                let path = self.key(&path);
                if found.declared {
                    declared.insert(path.clone());
                }
                entries.insert(path);
            }
        }
        let mut fallback = BTreeSet::new();
        for path in open {
            let program = self.program(index, path, folders);
            if let Some((dir, config)) = self.config(index, path) {
                let found = project::entry_files(&dir, Some(&config));
                for path in found.paths.into_iter().filter(|p| self.is_file(p)) {
                    let path = self.key(&path);
                    if found.declared {
                        declared.insert(path.clone());
                    }
                    entries.insert(path);
                }
            }
            if !jaic::paths::is_module_entry(&program.root) {
                fallback.insert(program.root);
            }
        }
        if entries.is_empty() {
            entries = fallback;
        }
        // An inferred source already checked by an earlier entry is part of that program.
        // Explicit entries remain independent even when their source graphs overlap.
        let candidates = entries.clone();
        entries.retain(|entry| {
            declared.contains(entry)
                || !candidates.range(..entry.clone()).any(|owner| {
                    !index.stale_source_graphs.contains_key(owner)
                        && index
                            .builds
                            .get(owner)
                            .is_some_and(|facts| facts.sources.contains_key(entry))
                })
        });
        entries
    }

    pub(super) fn program(&self, index: &mut Index, file: &Path, folders: &[PathBuf]) -> Program {
        self.refresh_source_graphs(index);
        let key = self.key(file);
        let file = key.as_path();
        let folder_keys: Vec<_> = folders.iter().map(|p| self.key(p)).collect();
        let folders = folder_keys.as_slice();
        let config = self.config(index, file);
        let base =
            project::project_base(config.as_ref().map(|(dir, _)| dir.as_path()), folders, file);
        let entries: BTreeSet<PathBuf> = match &base {
            Some(base) => {
                let found = project::entry_files(base, config.as_ref().map(|(_, c)| c));
                found
                    .paths
                    .iter()
                    .map(|f| self.key(f))
                    .filter(|f| found.declared || self.is_file(f))
                    .collect()
            }
            None => BTreeSet::new(),
        };
        let mut found = None;
        for entry in &entries {
            let closure = self.closure(index, entry);
            if closure.contains(file) {
                found = Some((entry.clone(), closure));
                break;
            }
        }
        if found.is_none() {
            for entry in &entries {
                if !index.stale_source_graphs.contains_key(entry)
                    && index
                        .builds
                        .get(entry)
                        .is_some_and(|facts| facts.sources.contains_key(file))
                {
                    // Ownership includes imports; the lexical load closure still determines
                    // which declarations are visible without an import.
                    found = Some((entry.clone(), self.closure(index, entry)));
                    break;
                }
            }
        }
        // A file of a module the project keeps in its own folders (`modules/Simp/font.jai`)
        // belongs to the program that imports it: checked there, it sees what the program
        // gives it (`#module_parameters`, names a metaprogram pokes in).
        if found.is_none()
            && let Some(base) = &base
        {
            let configured = config
                .as_ref()
                .map(|(_, config)| config.import_dirs(base))
                .unwrap_or_default();
            let mut dirs = project::existing_dirs(
                &Listing(self),
                &project::import_paths(base, &configured, &[], Path::new("")),
            );
            dirs.retain(|d| file.starts_with(d));
            if !dirs.is_empty() {
                for entry in &entries {
                    let mut entry_dirs = vec![entry.parent().unwrap_or(base).join("modules")];
                    entry_dirs.extend(dirs.iter().cloned());
                    if self
                        .closure_with_modules(index, entry, &entry_dirs)
                        .contains(file)
                    {
                        found = Some((entry.clone(), self.closure(index, entry)));
                        break;
                    }
                }
            }
        }
        let unattributed: Vec<PathBuf> = match &found {
            None => entries
                .iter()
                .filter(|e| self.is_file(e))
                .cloned()
                .collect(),
            _ => Vec::new(),
        };
        let (root, closure) = if let Some(found) = found {
            found
        } else {
            // Up the `#load`s: a file another file loads belongs to that file's program, so
            // it sees the names its loader declares or imports.
            let root = self.loader_chain(index, file, base.as_deref());
            let closure = self.closure(index, &root);
            (root, closure)
        };
        let root_dir = root.parent().unwrap_or(Path::new("/")).to_path_buf();
        let dir = match &base {
            Some(base) if entries.contains(&root) => base.clone(),
            _ => root_dir.clone(),
        };
        let mut options = (self.env.options)(&root);
        let found = self.import_dirs(index, &root, folders).0;
        jaic::project::add_import_dirs(&mut options.import_paths, &found);
        let stdlib = options.stdlib_root().map(Path::to_path_buf);
        let configured = match (&base, &config) {
            (Some(base), Some((_, config))) => config.import_dirs(base),
            _ => Vec::new(),
        };
        // The project's own `modules`, then what the environment and the build added.
        let extra: Vec<PathBuf> = base
            .iter()
            .map(|base| base.join("modules"))
            .chain(options.import_paths)
            .collect();
        let module_dirs = project::existing_dirs(
            &Listing(self),
            &project::import_paths(&root_dir, &configured, &extra, Path::new("")),
        );
        let module_dirs: Vec<PathBuf> = module_dirs
            .into_iter()
            .filter(|d| Some(d) != stdlib.as_ref().map(|s| self.key(s)).as_ref())
            .collect();
        Program {
            root,
            closure,
            dir,
            entries,
            unattributed,
            module_dirs,
            os: options.os,
            stdlib,
        }
    }
}
