//! Auto-import completion: names the document cannot see yet, each with the `#import` or
//! `#load` that brings it in as an additional edit (TypeScript-style auto-imports).
//!
//! Three sources, all matched case-insensitively by prefix:
//! - **Standard-library modules** (and `Extensions/<Name>`): their exported top-level names.
//!   Indexed once per stdlib folder, on first use.
//! - **The project's modules**: the `modules/` folder next to the entry file and next to the
//!   project root, `[project] import_path`, and the environment's import paths. A module
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
use crate::project::{self};
use crate::semantic::Environment;
use crate::session::Session;
use jaic::sema::{ModuleRanking, TargetOs};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

mod aliases;
mod index_fs;
mod items;
mod module_index;
mod names;
mod programs;

/// The most auto-import items one completion offers.
pub const MAX_ITEMS: usize = 50;

/// The most overload signatures one item's documentation shows.
const MAX_OVERLOADS: usize = 4;

/// Characters to type before auto-import items are offered.
pub const MIN_PREFIX: usize = 2;

/// The most files one load graph (a program's or a module's) follows.
const MAX_LOADED: usize = 4000;

use jaic::project::TREE_SKIPPED as SKIPPED_FOLDERS;

/// A module and the names it exports.
pub(crate) struct Module {
    /// As imported: `Basic`, `Extensions/Long_Double`.
    name: String,
    only_os: Option<Vec<String>>,
    decls: Vec<Decl>,
    /// What its files `#import`, as the paths written.
    imports: BTreeSet<String>,
    /// The module file (`dir/Name.jai` or `dir/Name/module.jai`) and the files its `#load`s
    /// reach, which this was read from: an edit to any of them makes it stale.
    file: PathBuf,
    files: BTreeSet<PathBuf>,
    /// The folder it was found in and the prefix its name has there (`Extensions/`).
    dir: PathBuf,
    prefix: String,
}

/// The modules of a folder. Each is shared, so one that an edit made stale is read again
/// without reading the rest. The stdlib's are shared between sessions too
/// (`module_index::SharedStdlib`).
type Modules = Arc<Vec<Arc<Module>>>;

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
    /// How the stdlib modules rank for suggestion, for the module list it was worked out from.
    ranking: Option<(Modules, ModuleRanking)>,
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
    /// `[project] entry` that exist, when no entry's loads reach the document: one of
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
    /// While the stdlib's modules are read for sharing: what they were read from.
    recording: RefCell<Option<Recorded>>,
}

/// The paths a module list was read from (keys, with their stamps), the options asked for
/// (import paths and Preload by root), and whether anything else went into it.
/// The options a module list depends on: import paths and Preload.
type AskedOptions = (Vec<PathBuf>, Option<PathBuf>);

#[derive(Default)]
struct Recorded {
    stamps: BTreeMap<PathBuf, Option<u64>>,
    options: BTreeMap<PathBuf, AskedOptions>,
    /// An open document or a build's facts were read: the list is this session's own.
    own: bool,
}

impl Session {
    /// The open documents and lint configurations over the file system of `env`.
    pub(crate) fn sources<'a>(&'a self, env: &'a Environment) -> Sources<'a> {
        let keys = jaic::paths::CanonCache::new();
        let open = self
            .documents
            .iter()
            .map(|(u, d)| (u, d.text.as_str()))
            .chain(
                self.saved_configs
                    .iter()
                    .map(|(u, text)| (u, text.as_str())),
            )
            .map(|(u, text)| (memo_key(&keys, env, Path::new(u.path())), text))
            .collect();
        Sources {
            env,
            open,
            keys,
            recording: RefCell::default(),
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

    /// While recording, `key` was read from the file system (`open`: from an open document).
    fn note(&self, key: &Path, open: bool) {
        if let Some(recorded) = self.recording.borrow_mut().as_mut() {
            if open {
                recorded.own = true;
            } else if !recorded.stamps.contains_key(key) {
                recorded
                    .stamps
                    .insert(key.to_path_buf(), self.env.fs.stamp(key));
            }
        }
    }

    /// While recording, something besides files and options went in.
    fn note_own(&self) {
        if let Some(recorded) = self.recording.borrow_mut().as_mut() {
            recorded.own = true;
        }
    }

    /// The compiler options for `root`.
    pub(crate) fn options(&self, root: &Path) -> jaic::sema::Options {
        let options = (self.env.options)(root);
        if let Some(recorded) = self.recording.borrow_mut().as_mut() {
            recorded.options.insert(
                root.to_path_buf(),
                (options.import_paths.clone(), options.preload.clone()),
            );
        }
        options
    }

    pub(crate) fn read(&self, path: &Path) -> Option<String> {
        let key = self.key(path);
        let open = self.open.get(&key);
        self.note(&key, open.is_some());
        match open {
            Some(text) => Some((*text).to_string()),
            None => self
                .env
                .fs
                .read(path)
                .map(|b| String::from_utf8_lossy(&b).into_owned()),
        }
    }

    pub(crate) fn is_file(&self, path: &Path) -> bool {
        let key = self.key(path);
        let open = self.open.contains_key(&key);
        self.note(&key, open);
        open || self.env.fs.is_file(path)
    }

    /// Whether an open document lies under `key`.
    fn open_under(&self, key: &Path) -> bool {
        self.open.keys().any(|p| p.starts_with(key) && *p != key)
    }

    pub(crate) fn is_dir(&self, path: &Path) -> bool {
        let key = self.key(path);
        let open = self.open_under(&key);
        self.note(&key, open);
        open || self.env.fs.is_dir(path)
    }

    /// Entries of `dir` (name, is a folder): on disk and open documents below it.
    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        let key = self.key(dir);
        self.note(&key, self.open_under(&key));
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
        self.note(path, self.open.contains_key(path));
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
}
