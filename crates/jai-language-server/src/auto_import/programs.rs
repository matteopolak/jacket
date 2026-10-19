//! Which program a file belongs to: the project's files, its `jai.toml`, the file a fragment is
//! analysed from, and the folders and entries that make up one program.
use super::index_fs::IndexFs;
use super::{Index, MAX_DEPTH, MAX_PROJECT_FILES, Program, SKIPPED_FOLDERS, Sources};
use crate::project::{self, ProjectConfig};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

impl Sources<'_> {
    /// The `.jai` files under `dir` (cached walk of the disk, plus open documents).
    pub(super) fn project_files(&self, index: &mut Index, dir: &Path) -> Vec<PathBuf> {
        let key = self.key(dir);
        let dir = key.as_path();
        let walked = if let Some(files) = index.walked.get(dir) {
            files.clone()
        } else {
            let mut files = Vec::new();
            let mut pending = vec![(dir.to_path_buf(), 0)];
            while let Some((folder, depth)) = pending.pop() {
                for (entry, is_dir) in self.env.fs.list_dir(&folder) {
                    if files.len() >= MAX_PROJECT_FILES {
                        break;
                    }
                    let path = folder.join(&entry);
                    if is_dir {
                        if depth < MAX_DEPTH
                            && !entry.starts_with('.')
                            && !SKIPPED_FOLDERS.contains(&entry.as_str())
                        {
                            pending.push((path, depth + 1));
                        }
                    } else if entry.ends_with(".jai") {
                        files.push(self.key(&path));
                    }
                }
            }
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

    pub(super) fn program(&self, index: &mut Index, file: &Path, folders: &[PathBuf]) -> Program {
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
        // A file of a module the project keeps in its own folders (`modules/Simp/font.jai`)
        // belongs to the program that imports it: checked there, it sees what the program
        // gives it (`#module_parameters`, names a metaprogram pokes in).
        if found.is_none()
            && let Some(base) = &base
        {
            let mut dirs = vec![base.join("modules")];
            if let Some((_, config)) = &config {
                dirs.extend(config.import_path.0.iter().map(|p| self.key(&base.join(p))));
            }
            dirs.retain(|d| file.starts_with(d) && self.is_dir(d));
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
        let unattributed: Vec<PathBuf> = match (&found, &config) {
            (None, Some((_, config))) if !config.entry.0.is_empty() => entries
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
        crate::modules::splice(&mut options.import_paths, &found);
        let stdlib = options.stdlib_root().map(Path::to_path_buf);
        let mut module_dirs = vec![root_dir.join("modules")];
        if let Some(base) = &base {
            module_dirs.push(base.join("modules"));
            if let Some((_, config)) = &config {
                module_dirs.extend(config.import_path.0.iter().map(|p| self.key(&base.join(p))));
            }
        }
        module_dirs.extend(
            options.import_paths.into_iter().filter(|p| {
                stdlib.as_deref().map(project::normalize) != Some(project::normalize(p))
            }),
        );
        let mut seen = BTreeSet::new();
        module_dirs = module_dirs.iter().map(|d| self.key(d)).collect();
        module_dirs.retain(|d| seen.insert(d.clone()) && self.is_dir(d));
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
