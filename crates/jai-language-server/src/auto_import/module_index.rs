//! Finding the modules an import can name: the files a program loads, the standard library's
//! modules (ranked, cached) and the project's own.
use super::index_fs::IndexFs;
use super::{AskedOptions, Index, MAX_LOADED, Module, ModuleCache, Modules, Recorded, Sources};
use crate::exports::Visibility;
use crate::project;
use jaic::fxhash::HashMap;
use jaic::sema::ModuleRanking;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

impl Sources<'_> {
    /// `root` and every file its `#load`s reach (and the files a build metaprogram compiles).
    pub(super) fn closure(&self, index: &mut Index, root: &Path) -> BTreeSet<PathBuf> {
        project::load_closure(&IndexFs::new(self, index), root, MAX_LOADED)
    }

    /// `closure` of `root`, plus every file of the modules in `module_dirs` (the project's own,
    /// not the stdlib's) that those files `#import`, and what those modules import in turn.
    pub(super) fn closure_with_modules(
        &self,
        index: &mut Index,
        root: &Path,
        module_dirs: &[PathBuf],
    ) -> BTreeSet<PathBuf> {
        let mut seen = BTreeSet::new();
        let mut pending = vec![self.key(root)];
        while let Some(entry) = pending.pop() {
            let entry = self.key(&entry);
            if seen.len() >= MAX_LOADED || seen.contains(&entry) {
                continue;
            }
            let files = self.closure(index, &entry);
            for file in &files {
                for import in &self.scan(index, file).imports {
                    let found = module_dirs.iter().find_map(|dir| {
                        [
                            jaic::paths::module_entry(&dir.join(&import.path)),
                            dir.join(format!("{}.jai", import.path)),
                        ]
                        .into_iter()
                        .find(|f| self.is_file(f))
                    });
                    if let Some(found) = found {
                        pending.push(project::normalize(&found));
                    }
                }
            }
            seen.extend(files);
        }
        seen
    }

    /// The modules in `dir`: `Name.jai` and `Name/module.jai`, imported as `prefix` + `Name`.
    pub(super) fn modules_in(
        &self,
        index: &mut Index,
        dir: &Path,
        prefix: &str,
    ) -> Vec<Arc<Module>> {
        let mut modules = Vec::new();
        for (entry, is_dir) in self.list_dir(dir) {
            if entry.starts_with('.') {
                continue;
            }
            let (name, file) = if is_dir {
                let file = dir.join(&entry).join("module.jai");
                if !self.is_file(&file) {
                    continue;
                }
                (entry, file)
            } else if let Some(stem) = entry.strip_suffix(".jai") {
                (stem.to_string(), dir.join(&entry))
            } else {
                continue;
            };
            modules.push(Arc::new(self.read_module(
                index,
                dir,
                prefix,
                format!("{prefix}{name}"),
                file,
            )));
        }
        modules
    }

    /// The module `name` whose file is `file`, found in `dir`.
    pub(super) fn read_module(
        &self,
        index: &mut Index,
        dir: &Path,
        prefix: &str,
        name: String,
        file: PathBuf,
    ) -> Module {
        let only_os = self.scan(index, &file).only_os.clone();
        let mut decls = Vec::new();
        let mut imports = BTreeSet::new();
        let files = self.closure(index, &file);
        for part in &files {
            let scan = self.scan(index, part);
            imports.extend(scan.imports.iter().map(|i| i.path.clone()));
            for decl in scan
                .decls
                .iter()
                .filter(|d| d.visibility == Visibility::Export)
            {
                decls.extend(self.alias_documentation(index, &file, part, decl, dir));
            }
        }
        Module {
            name,
            only_os,
            decls,
            imports,
            file,
            files,
            dir: dir.to_path_buf(),
            prefix: prefix.to_string(),
        }
    }

    /// `modules` with the ones read from a file in `stale` read again.
    pub(super) fn refreshed(
        &self,
        index: &mut Index,
        modules: &Modules,
        stale: &BTreeSet<PathBuf>,
    ) -> Modules {
        if !modules
            .iter()
            .any(|m| m.files.iter().any(|f| stale.contains(f)))
        {
            return modules.clone();
        }
        Arc::new(
            modules
                .iter()
                .map(|m| {
                    if m.files.iter().any(|f| stale.contains(f)) {
                        Arc::new(self.read_module(
                            index,
                            &m.dir,
                            &m.prefix,
                            m.name.clone(),
                            m.file.clone(),
                        ))
                    } else {
                        m.clone()
                    }
                })
                .collect(),
        )
    }

    pub(super) fn stdlib(&self, index: &mut Index, dir: &Path) -> Modules {
        if let Some((cached, cache)) = &index.stdlib
            && cached == dir
        {
            if cache.stale.is_empty() {
                return cache.modules.clone();
            }
            let (modules, stale) = (cache.modules.clone(), cache.stale.clone());
            let modules = self.refreshed(index, &modules, &stale);
            index.stdlib = Some((
                dir.to_path_buf(),
                ModuleCache {
                    modules: modules.clone(),
                    stale: BTreeSet::new(),
                },
            ));
            return modules;
        }
        let modules = self.shared_stdlib(index, dir);
        index.stdlib = Some((
            dir.to_path_buf(),
            ModuleCache {
                modules: modules.clone(),
                stale: BTreeSet::new(),
            },
        ));
        modules
    }

    /// The stdlib's modules as another session of this process read them, if nothing they were
    /// read from has changed since and no open document stands in for one of those files;
    /// otherwise read here, and kept for other sessions when only the file system went in.
    fn shared_stdlib(&self, index: &mut Index, dir: &Path) -> Modules {
        static KEPT_STDLIBS: Mutex<Vec<(PathBuf, Arc<SharedStdlib>)>> = Mutex::new(Vec::new());
        let candidates: Vec<Arc<SharedStdlib>> = KEPT_STDLIBS
            .lock()
            .map(|kept| {
                kept.iter()
                    .rev()
                    .filter(|(root, _)| root == dir)
                    .map(|(_, shared)| shared.clone())
                    .collect()
            })
            .unwrap_or_default();
        if let Some(found) = candidates
            .into_iter()
            .find(|shared| shared.is_current(self))
        {
            return found.modules.clone();
        }
        // Scans kept from earlier would not be recorded as read: read them again.
        index.files.retain(|path, _| !path.starts_with(dir));
        let outer = self.recording.replace(Some(Recorded::default()));
        let modules = self.read_stdlib(index, dir);
        let recorded = self.recording.replace(outer).unwrap_or_default();
        // The stdlib's files are only read through its modules: drop their scans.
        index.files.retain(|path, _| !path.starts_with(dir));
        if let Some(shared) = SharedStdlib::new(modules.clone(), recorded)
            && let Ok(mut kept) = KEPT_STDLIBS.lock()
        {
            kept.retain(|(root, old)| root != dir || old.is_current(self));
            kept.push((dir.to_path_buf(), Arc::new(shared)));
            if kept.len() > KEPT {
                kept.remove(0);
            }
        }
        modules
    }

    fn read_stdlib(&self, index: &mut Index, dir: &Path) -> Modules {
        let preload = self
            .options(&dir.join(".prelude-index.jai"))
            .preload
            .map(|path| self.key(&path));
        let mut modules: Vec<Arc<Module>> = self
            .modules_in(index, dir, "")
            .into_iter()
            .filter(|m| {
                !jaic::lang_items::modules::INTERNAL.contains(&m.name.as_str())
                    && preload
                        .as_ref()
                        .is_none_or(|preload| *preload != self.key(&m.file))
            })
            .collect();
        let extensions = jaic::STDLIB_EXTENSIONS_DIR;
        modules.extend(self.modules_in(index, &dir.join(extensions), &format!("{extensions}/")));
        Arc::new(modules)
    }

    /// Which stdlib modules to suggest first: those most other stdlib modules import. Worked out
    /// again only when the module list changes.
    pub(super) fn stdlib_ranking(&self, index: &mut Index, modules: &Modules) -> ModuleRanking {
        if let Some((from, ranking)) = &index.ranking
            && Arc::ptr_eq(from, modules)
        {
            return ranking.clone();
        }
        let importers = jaic::sema::count_importers(
            modules
                .iter()
                .map(|m| (m.name.as_str(), m.imports.iter().map(String::as_str))),
        );
        let ranking = ModuleRanking::new(HashMap::default(), importers);
        index.ranking = Some((modules.clone(), ranking.clone()));
        ranking
    }

    pub(super) fn user_modules(&self, index: &mut Index, dir: &Path) -> Modules {
        if let Some(cache) = index.module_dirs.get(dir) {
            if cache.stale.is_empty() {
                return cache.modules.clone();
            }
            let (modules, stale) = (cache.modules.clone(), cache.stale.clone());
            let modules = self.refreshed(index, &modules, &stale);
            index.module_dirs.insert(
                dir.to_path_buf(),
                ModuleCache {
                    modules: modules.clone(),
                    stale: BTreeSet::new(),
                },
            );
            return modules;
        }
        let modules = Arc::new(self.modules_in(index, dir, ""));
        index.module_dirs.insert(
            dir.to_path_buf(),
            ModuleCache {
                modules: modules.clone(),
                stale: BTreeSet::new(),
            },
        );
        modules
    }
}

/// Stdlib module lists kept for the sessions of this process.
const KEPT: usize = 4;

/// A stdlib module list and what it was read from: each path's stamp and the options asked for.
struct SharedStdlib {
    modules: Modules,
    stamps: Vec<(PathBuf, u64)>,
    paths: BTreeSet<PathBuf>,
    options: Vec<(PathBuf, AskedOptions)>,
}

impl SharedStdlib {
    /// `None` when an open document or a build went in, or a path has no stamp.
    fn new(modules: Modules, recorded: Recorded) -> Option<Self> {
        if recorded.own {
            return None;
        }
        let stamps = recorded
            .stamps
            .into_iter()
            .map(|(path, stamp)| Some((path, stamp?)))
            .collect::<Option<Vec<_>>>()?;
        Some(SharedStdlib {
            modules,
            paths: stamps.iter().map(|(path, _)| path.clone()).collect(),
            stamps,
            options: recorded.options.into_iter().collect(),
        })
    }

    /// Whether `sources` would read the same modules: no path it was read from changed or is
    /// an open document (or a folder holding one), and the options are the same.
    fn is_current(&self, sources: &Sources) -> bool {
        sources
            .open
            .keys()
            .all(|open| !open.ancestors().any(|path| self.paths.contains(path)))
            && self
                .stamps
                .iter()
                .all(|(path, stamp)| sources.env.fs.stamp(path) == Some(*stamp))
            && self.options.iter().all(|(root, (import_paths, preload))| {
                let options = (sources.env.options)(root);
                options.import_paths == *import_paths && options.preload == *preload
            })
    }
}
