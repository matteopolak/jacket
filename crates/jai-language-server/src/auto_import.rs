//! Auto-import completion: names the document cannot see yet, each with the `#import` or
//! `#load` that brings it in as an additional edit (TypeScript-style auto-imports).
//!
//! Three sources, all matched case-insensitively by prefix:
//! - **Standard-library modules** (and `Extensions/<Name>`): their exported top-level names.
//!   Indexed once per stdlib folder, on first use.
//! - **The project's modules**: the `modules/` folder next to the entry file and next to the
//!   project root, `jai.toml`'s `import_path`, and the environment's import paths. A module
//!   there hides a stdlib module of the same name, as for the compiler.
//! - **The project's other files**: declarations of `.jai` files under the project folder that
//!   the program does not load yet get `#load "relative/path.jai";` into the document; files
//!   it already loads offer their names with no edit.
//!
//! Every file is scanned from tokens (`exports.rs`) and cached by path: files on disk until a
//! watched-file or save notification names them, open documents by a hash of their text.
//! Folder listings are cached until a file is created or deleted under them.
use crate::exports::{self, Decl, FileScan, Visibility};
use crate::model::CompletionKind;
use crate::project::{self, ProjectConfig};
use crate::semantic::Environment;
use crate::session::Session;
use jaic::sema::TargetOs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod aliases;
mod index_fs;
mod items;
mod names;

/// The most auto-import items one completion offers.
pub const MAX_ITEMS: usize = 50;

/// The most overload signatures one item's documentation shows.
const MAX_OVERLOADS: usize = 4;

/// Characters to type before auto-import items are offered.
pub const MIN_PREFIX: usize = 2;

use jaic::project::{MAX_TREE_DEPTH as MAX_DEPTH, MAX_TREE_FILES as MAX_PROJECT_FILES};

/// The most files one load graph (a program's or a module's) follows.
const MAX_LOADED: usize = 4000;

use self::index_fs::IndexFs;
use jaic::project::TREE_SKIPPED as SKIPPED_FOLDERS;

/// A module and the names it exports.
pub(crate) struct Module {
    /// As imported: `Basic`, `Extensions/Long_Double`.
    name: String,
    only_os: Option<Vec<String>>,
    decls: Vec<Decl>,
    /// The module file (`dir/Name.jai` or `dir/Name/module.jai`) and the files its `#load`s
    /// reach, which this was read from: an edit to any of them makes it stale.
    file: PathBuf,
    files: BTreeSet<PathBuf>,
    /// The folder it was found in and the prefix its name has there (`Extensions/`).
    dir: PathBuf,
    prefix: String,
}

/// The modules of a folder. Each is shared, so one that an edit made stale is read again
/// without reading the rest.
type Modules = Rc<Vec<Rc<Module>>>;

/// The modules of a folder, and the files that changed since they were read.
struct ModuleCache {
    modules: Modules,
    stale: BTreeSet<PathBuf>,
}

#[derive(Default)]
pub(crate) struct Index {
    /// Module folders found for a program root (`modules.rs`), with the files they were read from.
    pub(crate) import_dirs: BTreeMap<PathBuf, (Vec<PathBuf>, Vec<PathBuf>)>,
    /// What each build metaprogram did when it last ran to the end (`semantic::BuildFacts`),
    /// by its canonical path. Where a metaprogram has an entry here, discovery follows it
    /// instead of reading the metaprogram's text.
    pub(crate) builds: BTreeMap<PathBuf, crate::semantic::BuildFacts>,
    /// The build files run to see what they build, with a hash of everything they were run
    /// from: a metaprogram is run again only after that changes.
    pub(crate) probed: BTreeMap<PathBuf, u64>,
    /// Module lists. A file that changes makes stale only the modules read from it, which are
    /// made again when the list is next asked for.
    stdlib: Option<(PathBuf, ModuleCache)>,
    module_dirs: BTreeMap<PathBuf, ModuleCache>,
    /// Direct loaders of each file under a folder, as of `graph_epoch` and the files given.
    graphs: BTreeMap<PathBuf, (u64, Vec<PathBuf>, project::LoaderGraph)>,
    /// Moves on whenever a file's `#load`s are not what the last scan of it saw.
    graph_epoch: u64,
    /// The `#load` paths each file had when it was last scanned.
    loads_seen: BTreeMap<PathBuf, Vec<String>>,
    /// Scans of files read from the environment.
    files: BTreeMap<PathBuf, Rc<FileScan>>,
    /// Scans of open documents, with the hash of the text scanned.
    open: BTreeMap<PathBuf, (u64, Rc<FileScan>)>,
    /// The `.jai` files on disk under each walked folder.
    walked: BTreeMap<PathBuf, Rc<Vec<PathBuf>>>,
    /// Settings files that did not parse, with the problem last reported for each.
    reported: BTreeMap<PathBuf, String>,
    /// Messages for the client not sent yet.
    pending: Vec<String>,
    /// Log lines for the client not sent yet, and the ones sent (each goes out once).
    logs: Vec<String>,
    /// The settings read for each config directory, with a hash of the text they were read from.
    pub(crate) configs: BTreeMap<PathBuf, (u64, jacket_config::Loaded)>,
    logged: BTreeSet<String>,
}

impl Index {
    /// The settings file `path` does not parse: tell the client once per distinct problem.
    pub(crate) fn bad_config(&mut self, path: &Path, problem: &str) {
        if self.reported.get(path).is_some_and(|p| p == problem) {
            return;
        }
        self.reported
            .insert(path.to_path_buf(), problem.to_string());
        self.pending.push(format!(
            "{}: {problem}. The defaults are used until it is fixed.",
            path.display()
        ));
    }

    /// The settings file `path` parses (again): a later problem is news.
    pub(crate) fn good_config(&mut self, path: &Path) {
        self.reported.remove(path);
    }

    /// A log line for the client (`window/logMessage`), sent once however often it is found.
    pub(crate) fn log_once(&mut self, line: &str) {
        if self.logged.insert(line.to_string()) {
            self.logs.push(line.to_string());
        }
    }

    /// Log lines for the client, once.
    pub(crate) fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.logs)
    }

    /// A message for the client, sent once.
    pub(crate) fn notify(&mut self, message: String) {
        self.pending.push(message);
    }

    /// Messages for the client, once.
    pub(crate) fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending)
    }

    /// `scan` is the latest of `path`: when its `#load`s differ from the scan before, the loader
    /// graphs are out of date.
    fn note_loads(&mut self, path: &Path, scan: &FileScan) {
        let same = self.loads_seen.get(path).is_some_and(|seen| {
            seen.len() == scan.loads.len()
                && seen.iter().zip(&scan.loads).all(|(a, b)| *a == b.path)
        });
        if !same {
            self.loads_seen.insert(
                path.to_path_buf(),
                scan.loads.iter().map(|l| l.path.clone()).collect(),
            );
            self.graph_epoch += 1;
        }
    }

    /// `path` changed: its scan and the module index it is part of are stale.
    pub(crate) fn changed(&mut self, path: &Path) {
        self.files.remove(path);
        self.import_dirs
            .retain(|_, (_, deps)| !deps.iter().any(|d| d == path));
        if let Some((dir, cache)) = &mut self.stdlib
            && path.starts_with(dir)
        {
            cache.stale.insert(path.to_path_buf());
        }
        for (dir, cache) in &mut self.module_dirs {
            if path.starts_with(dir) {
                cache.stale.insert(path.to_path_buf());
            }
        }
    }

    /// `path` was created or deleted: listings of the folders above it are stale too.
    pub(crate) fn created_or_deleted(&mut self, path: &Path) {
        self.changed(path);
        // A folder of modules may have appeared or gone.
        self.import_dirs.clear();
        self.walked.retain(|dir, _| !path.starts_with(dir));
        // A module may have appeared or gone in the folders above it; the other files' keys
        // and the loaders found among the files may be different too.
        self.module_dirs.retain(|dir, _| !path.starts_with(dir));
        if self
            .stdlib
            .as_ref()
            .is_some_and(|(dir, _)| path.starts_with(dir))
        {
            self.stdlib = None;
        }
        self.graphs.clear();
        self.graph_epoch += 1;
    }
}

/// Where a name comes from.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Source {
    /// A file the program loads already: no edit.
    Loaded(PathBuf),
    /// A project file to `#load`.
    Load(PathBuf),
    /// A module to `#import`.
    Import(String),
}

/// What the document's program is made of.
struct Program {
    /// The entry file the program is compiled from.
    root: PathBuf,
    /// Every file the entry loads (the document included).
    closure: BTreeSet<PathBuf>,
    /// The folder whose other files may be loaded.
    dir: PathBuf,
    /// The project's entry files: never offered for `#load`.
    entries: BTreeSet<PathBuf>,
    /// `jai.toml`'s `build_files` that exist, when no entry's loads reach the document: one of
    /// them may build it in a way only running it shows (a computed path).
    unattributed: Vec<PathBuf>,
    /// Folders of user modules, searched in order.
    module_dirs: Vec<PathBuf>,
    os: TargetOs,
    stdlib: Option<PathBuf>,
}

/// Files as the server sees them: open documents over the environment's file system.
/// Made for one request: the file system is taken not to change while it lives.
pub(crate) struct Sources<'a> {
    pub(crate) env: &'a Environment,
    pub(crate) open: BTreeMap<PathBuf, &'a str>,
    /// `canonical_key` of the paths asked about, each a few file system calls.
    keys: jaic::paths::CanonCache,
}

impl Session {
    /// The open documents and lint configurations over the file system of `env`.
    pub(crate) fn sources<'a>(&'a self, env: &'a Environment) -> Sources<'a> {
        let keys = jaic::paths::CanonCache::new();
        let open = self
            .documents
            .iter()
            .chain(&self.lint_configs)
            .map(|(u, d)| (memo_key(&keys, env, Path::new(u.path())), d.text.as_str()))
            .collect();
        Sources {
            env,
            open,
            keys,
        }
    }
}

fn memo_key(keys: &jaic::paths::CanonCache, env: &Environment, path: &Path) -> PathBuf {
    keys.get_or_insert_with(path, |path| canonical_key(env, path))
}

pub(crate) fn hash(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
    text.hash(&mut h);
    h.finish()
}

/// Filesystem identity for index keys, including unsaved or not-yet-created files.
/// Existing ancestors establish alias identity; spelling and case are otherwise preserved.
pub(crate) fn canonical_key(env: &Environment, path: &Path) -> PathBuf {
    let canonical = env.fs.canonical(path);
    if canonical != project::normalize(path) || env.fs.is_file(path) || env.fs.is_dir(path) {
        return crate::document::path_key(&canonical);
    }
    for parent in path.ancestors().skip(1) {
        if env.fs.is_dir(parent) {
            let suffix = path.strip_prefix(parent).expect("ancestor prefix");
            return crate::document::path_key(
                &crate::document::file_key(&*env.fs, parent).join(suffix),
            );
        }
    }
    crate::document::path_key(&canonical)
}

impl Sources<'_> {
    pub(crate) fn key(&self, path: &Path) -> PathBuf {
        memo_key(&self.keys, self.env, path)
    }

    pub(crate) fn read(&self, path: &Path) -> Option<String> {
        match self.open.get(&self.key(path)) {
            Some(text) => Some((*text).to_string()),
            None => self
                .env
                .fs
                .read(path)
                .map(|b| String::from_utf8_lossy(&b).into_owned()),
        }
    }

    pub(crate) fn is_file(&self, path: &Path) -> bool {
        self.open.contains_key(&self.key(path)) || self.env.fs.is_file(path)
    }

    pub(crate) fn is_dir(&self, path: &Path) -> bool {
        let key = self.key(path);
        self.open.keys().any(|p| p.starts_with(&key) && *p != key) || self.env.fs.is_dir(path)
    }

    /// Entries of `dir` (name, is a folder): on disk and open documents below it.
    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        let key = self.key(dir);
        let mut entries: BTreeSet<(String, bool)> = self.env.fs.list_dir(dir).into_iter().collect();
        for open in self.open.keys() {
            if let Ok(rest) = open.strip_prefix(&key) {
                let mut parts = rest.iter();
                if let Some(first) = parts.next() {
                    entries.insert((first.to_string_lossy().into_owned(), parts.next().is_some()));
                }
            }
        }
        entries.into_iter().collect()
    }

    pub(crate) fn scan(&self, index: &mut Index, path: &Path) -> Rc<FileScan> {
        self.scan_keyed(index, &self.key(path))
    }

    /// `scan` of a path that is already a key (what `project_files` lists), which costs no
    /// file system calls to know.
    fn scan_keyed(&self, index: &mut Index, path: &Path) -> Rc<FileScan> {
        if let Some(text) = self.open.get(path) {
            let h = hash(text);
            if let Some((seen, scan)) = index.open.get(path)
                && *seen == h
            {
                return scan.clone();
            }
            let scan = Rc::new(exports::scan(text));
            index.note_loads(path, &scan);
            index.open.insert(path.to_path_buf(), (h, scan.clone()));
            return scan;
        }
        if let Some(scan) = index.files.get(path) {
            return scan.clone();
        }
        let scan = Rc::new(
            self.read(path)
                .map(|text| exports::scan(&text))
                .unwrap_or_default(),
        );
        index.note_loads(path, &scan);
        index.files.insert(path.to_path_buf(), scan.clone());
        scan
    }

    /// `root` and every file its `#load`s reach (and the files a build metaprogram compiles).
    fn closure(&self, index: &mut Index, root: &Path) -> BTreeSet<PathBuf> {
        project::load_closure(&IndexFs::new(self, index), root, MAX_LOADED)
    }

    /// `closure` of `root`, plus every file of the modules in `module_dirs` (the project's own,
    /// not the stdlib's) that those files `#import`, and what those modules import in turn.
    fn closure_with_modules(
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
    fn modules_in(&self, index: &mut Index, dir: &Path, prefix: &str) -> Vec<Rc<Module>> {
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
    fn read_module(
        &self,
        index: &mut Index,
        dir: &Path,
        prefix: &str,
        name: String,
        file: PathBuf,
    ) -> Module {
        let only_os = self.scan(index, &file).only_os.clone();
        let mut decls = Vec::new();
        let files = self.closure(index, &file);
        for part in &files {
            let scan = self.scan(index, part);
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
            file,
            files,
            dir: dir.to_path_buf(),
            prefix: prefix.to_string(),
        }
    }

    /// `modules` with the ones read from a file in `stale` read again.
    fn refreshed(
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

    fn stdlib(&self, index: &mut Index, dir: &Path) -> Modules {
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

    fn user_modules(&self, index: &mut Index, dir: &Path) -> Modules {
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

    /// The `.jai` files under `dir` (cached walk of the disk, plus open documents).
    fn project_files(&self, index: &mut Index, dir: &Path) -> Vec<PathBuf> {
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
    fn loader_chain(&self, index: &mut Index, file: &Path, base: Option<&Path>) -> PathBuf {
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

    fn program(&self, index: &mut Index, file: &Path, folders: &[PathBuf]) -> Program {
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
