//! Finding the modules an import can name: the files a program loads, the standard library's
//! modules (ranked, cached) and the project's own.
use super::index_fs::IndexFs;
use super::{Index, MAX_LOADED, Module, ModuleCache, Modules, Sources};
use crate::exports::Visibility;
use crate::project;
use jaic::fxhash::HashMap;
use jaic::sema::ModuleRanking;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
                            dir.join(&import.path).join("module.jai"),
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
    ) -> Vec<Rc<Module>> {
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
            modules.push(Rc::new(self.read_module(
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
        Rc::new(
            modules
                .iter()
                .map(|m| {
                    if m.files.iter().any(|f| stale.contains(f)) {
                        Rc::new(self.read_module(
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
        let mut modules: Vec<Rc<Module>> = self
            .modules_in(index, dir, "")
            .into_iter()
            .filter(|m| !jaic::sema::COMPILER_INTERNAL_MODULES.contains(&m.name.as_str()))
            .collect();
        let extensions = jaic::STDLIB_EXTENSIONS_DIR;
        modules.extend(self.modules_in(index, &dir.join(extensions), &format!("{extensions}/")));
        // The stdlib's files are only read through its modules: drop their scans.
        index.files.retain(|path, _| !path.starts_with(dir));
        let modules = Rc::new(modules);
        index.stdlib = Some((
            dir.to_path_buf(),
            ModuleCache {
                modules: modules.clone(),
                stale: BTreeSet::new(),
            },
        ));
        modules
    }

    /// Which stdlib modules to suggest first: those most other stdlib modules import. Worked out
    /// again only when the module list changes.
    pub(super) fn stdlib_ranking(&self, index: &mut Index, modules: &Modules) -> ModuleRanking {
        if let Some((from, ranking)) = &index.ranking
            && Rc::ptr_eq(from, modules)
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
        let modules = Rc::new(self.modules_in(index, dir, ""));
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
