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
use jaic::sema::{FileSystem, ModuleRanking};
use jaic::target::TargetOs;
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
#[derive(Clone)]
struct ModuleCache {
    modules: Modules,
    stale: BTreeSet<PathBuf>,
}

/// What the module folders of a program depend on: the root, the workspace folders that bound
/// the search, and the project configuration in force. A change of any of them is a different
/// key, so no entry has to be cleared by hand when they change.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ImportKey {
    pub(crate) root: PathBuf,
    pub(crate) folders: Vec<PathBuf>,
    pub(crate) config: Option<PathBuf>,
    pub(crate) advisory: bool,
}

#[derive(Clone, Default)]
pub(crate) struct Index {
    /// Module folders found for a program root (`modules.rs`), with the files they were read from,
    /// by everything the answer depends on besides those files.
    pub(crate) import_dirs: BTreeMap<ImportKey, (Vec<PathBuf>, Vec<PathBuf>)>,
    /// Proven positive memberships from completed producers, including checked children
    /// of a workspace with an unrelated child failure. Current proof is owned by `Cache`.
    pub(crate) builds: BTreeMap<PathBuf, Rc<crate::semantic::BuildFacts>>,
    /// Last-good hints; exact discovery never consumes this map.
    pub(crate) advisory_builds: BTreeMap<PathBuf, Rc<crate::semantic::BuildFacts>>,
    /// Completed probe status projections; failure never establishes a no-child result.
    pub(crate) probed: BTreeMap<PathBuf, crate::semantic::ProbeStatus>,
    /// Module lists. A file that changes makes stale only the modules read from it, which are
    /// made again when the list is next asked for.
    stdlib: Option<(PathBuf, ModuleCache)>,
    /// Complete modules retained across interrupted warm-ups. Partial lists never escape.
    warming: BTreeMap<(PathBuf, String), Arc<Module>>,
    /// Scans left by a stopped warm-up belong to this session, not the shared stdlib cache.
    resume_warm: bool,
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
    logged: BTreeSet<String>,
}

impl Index {
    /// Helper caches are not semantic input authority. Recompute them on an input memo miss.
    pub(crate) fn discard_input_memos(&mut self) {
        self.import_dirs.clear();
        self.files.clear();
        self.open.clear();
        self.graphs.clear();
        self.loads_seen.clear();
        self.walked.clear();
        self.graph_epoch += 1;
    }

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

    /// A log line for the client, sent every time.
    pub(crate) fn log(&mut self, line: &str) {
        self.logs.push(line.to_string());
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
        self.warming
            .retain(|_, module| !module.files.contains(path));
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
        self.warming.clear();
        self.resume_warm = false;
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

struct AdvisoryInputs<'a> {
    session: &'a Session,
    selected_generation: u64,
    valid: std::cell::Cell<bool>,
}

/// Files as the server sees them: open documents over the environment's file system.
/// Made for one request: the file system is taken not to change while it lives.
pub(crate) struct Sources<'a> {
    /// Only transactional background scans may return partial results.
    stoppable: bool,
    pub(crate) env: &'a Environment,
    base: Rc<dyn FileSystem>,
    role: Option<AdvisoryInputs<'a>>,
    pub(crate) open: BTreeMap<PathBuf, &'a str>,
    /// `canonical_key` of the paths asked about, each a few file system calls.
    keys: jaic::paths::CanonCache,
    /// While the stdlib's modules are read for sharing: what they were read from.
    recording: RefCell<Option<Recorded>>,
    input_proof: Option<&'a RefCell<crate::semantic::Observations>>,
    pub(crate) advisory: bool,
    request_input_proof: Option<&'a RefCell<crate::semantic::Observations>>,
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
        self.sources_with_provider(env, env.fs.clone(), None)
    }

    /// Advisory catalogue rows keep their existing notification-governed freshness.
    /// They never borrow semantic or strict diagnostic-hint input authority.
    fn advisory_sources<'a>(&'a self, env: &'a Environment) -> Sources<'a> {
        let base = self
            .input_epoch
            .as_ref()
            .map_or_else(|| env.fs.clone(), |epoch| epoch.advisory_provider());
        let role = AdvisoryInputs {
            session: self,
            selected_generation: self.semantic_revision(),
            valid: std::cell::Cell::new(true),
        };
        self.sources_with_provider(env, base, Some(role))
    }

    fn sources_with_provider<'a>(
        &'a self,
        env: &'a Environment,
        base: Rc<dyn FileSystem>,
        role: Option<AdvisoryInputs<'a>>,
    ) -> Sources<'a> {
        let capture = role.is_none() && self.capture_inputs.get() > 0;
        let keys = jaic::paths::CanonCache::new();
        let observed = crate::semantic::ObservedFs {
            base: &*base,
            proof: capture.then_some(&self.input_proof),
            secondary: capture.then_some(&self.request_input_proof),
        };
        let open = self
            .documents
            .iter()
            .map(|(u, d)| (u, d.text.as_str()))
            .chain(self.saved_configs.keys().filter_map(|u| {
                let saved = if let Some(role) = &role {
                    let saved = self.saved_configs.get(u)?;
                    if saved.unavailable() {
                        role.valid.set(false);
                        return None;
                    }
                    saved
                } else {
                    self.saved_config_for_use(u)?
                };
                saved.text().map(|text| (u, text))
            }))
            .map(|(u, text)| {
                let key = keys.get_or_insert_with(Path::new(u.path()), |path| {
                    filesystem_key(&observed, path)
                });
                (key, text)
            })
            .collect();
        Sources {
            stoppable: false,
            env,
            base,
            role,
            open,
            keys,
            recording: RefCell::default(),
            input_proof: capture.then_some(&self.input_proof),
            advisory: !capture,
            request_input_proof: capture.then_some(&self.request_input_proof),
        }
    }
}


pub(crate) fn hash(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = jaic::fxhash::FxHasher::default();
    text.hash(&mut h);
    h.finish()
}

/// Filesystem identity for index keys, including unsaved or not-yet-created files.
/// Existing ancestors establish alias identity; spelling and case are otherwise preserved.
pub(crate) fn canonical_key(env: &Environment, path: &Path) -> PathBuf {
    filesystem_key(&*env.fs, path)
}

pub(crate) fn filesystem_key(fs: &dyn jaic::sema::FileSystem, path: &Path) -> PathBuf {
    let canonical = fs.canonical(path);
    if canonical != project::normalize(path) || fs.is_file(path) || fs.is_dir(path) {
        return jaic::paths::identity_spelling(&canonical);
    }
    for parent in path.ancestors().skip(1) {
        if fs.is_dir(parent) {
            let suffix = path.strip_prefix(parent).expect("ancestor prefix");
            return jaic::paths::identity_spelling(
                &jaic::paths::file_identity(fs, parent).join(suffix),
            );
        }
    }
    jaic::paths::identity_spelling(&canonical)
}

impl Sources<'_> {
    pub(crate) fn use_advisory(&mut self) {
        self.advisory = true;
    }

    pub(crate) fn known_build<'a>(
        &self,
        index: &'a Index,
        entry: &Path,
    ) -> Option<&'a crate::semantic::BuildFacts> {
        index
            .builds
            .get(entry)
            .or_else(|| {
                self.advisory
                    .then(|| index.advisory_builds.get(entry))
                    .flatten()
            })
            .map(Rc::as_ref)
    }

    fn disk_fs(&self) -> crate::semantic::ObservedFs<'_> {
        crate::semantic::ObservedFs {
            base: &*self.base,
            proof: self.input_proof,
            secondary: self.request_input_proof,
        }
    }

    fn snapshot_current(&self) -> bool {
        self.role.as_ref().is_none_or(|role| {
            role.valid.get() && role.session.semantic_revision() == role.selected_generation
        })
    }

    fn stopped(&self) -> bool {
        !self.snapshot_current() || (self.stoppable && jaic::interrupt::should_stop())
    }

    fn reject_advisory(&self) {
        if let Some(role) = &self.role {
            role.valid.set(false);
        }
    }

    fn accept_advisory(&self) -> bool {
        !self.stopped() && (self.role.is_none() || !jaic::interrupt::fired())
    }

    pub(crate) fn key(&self, path: &Path) -> PathBuf {
        self.keys
            .get_or_insert_with(path, |path| filesystem_key(&self.disk_fs(), path))
    }

    /// While recording, `key` was read from the file system (`open`: from an open document).
    fn note(&self, key: &Path, open: bool) {
        if let Some(recorded) = self.recording.borrow_mut().as_mut() {
            if open {
                recorded.own = true;
            } else if !recorded.stamps.contains_key(key) {
                recorded
                    .stamps
                    .insert(key.to_path_buf(), self.base.stamp(key));
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
        let options = (self.env.options)(&self.disk_fs(), root);
        if let Some(recorded) = self.recording.borrow_mut().as_mut() {
            recorded.options.insert(
                root.to_path_buf(),
                (options.import_paths.clone(), options.preload.clone()),
            );
        }
        options
    }

    pub(crate) fn read(&self, path: &Path) -> Option<String> {
        if self.stopped() {
            return None;
        }
        let key = self.key(path);
        let open = self.open.get(&key);
        self.note(&key, open.is_some());
        match open {
            Some(text) => Some((*text).to_string()),
            None => {
                if let Some(bytes) = self.disk_fs().read(path) {
                    Some(String::from_utf8_lossy(&bytes).into_owned())
                } else {
                    // Discovery probes absent entry candidates deliberately. Only certified
                    // absence is a usable negative; unknown I/O cannot complete a catalogue.
                    if self.role.is_some() && self.disk_fs().missing_file_key(path).is_none() {
                        self.reject_advisory();
                    }
                    None
                }
            }
        }
    }

    pub(crate) fn is_file(&self, path: &Path) -> bool {
        let key = self.key(path);
        let open = self.open.contains_key(&key);
        self.note(&key, open);
        open || self.disk_fs().is_file(path)
    }

    /// Whether an open document lies under `key`.
    fn open_under(&self, key: &Path) -> bool {
        self.open.keys().any(|p| p.starts_with(key) && *p != key)
    }

    pub(crate) fn is_dir(&self, path: &Path) -> bool {
        let key = self.key(path);
        let open = self.open_under(&key);
        self.note(&key, open);
        open || self.disk_fs().is_dir(path)
    }

    /// Entries of `dir` (name, is a folder): on disk and open documents below it.
    fn list_dir(&self, dir: &Path) -> Vec<(String, bool)> {
        if self.stopped() {
            return Vec::new();
        }
        let key = self.key(dir);
        self.note(&key, self.open_under(&key));
        let mut entries: BTreeSet<(String, bool)> =
            self.disk_fs().list_dir(dir).into_iter().collect();
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
        if self.stopped() {
            return Rc::default();
        }
        self.note(path, self.open.contains_key(path));
        if let Some(text) = self.open.get(path) {
            let h = hash(text);
            if self.advisory
                && let Some((seen, scan)) = index.open.get(path)
                && *seen == h
            {
                return scan.clone();
            }
            let scan = Rc::new(exports::scan(text));
            index.note_loads(path, &scan);
            if self.advisory {
                index.open.insert(path.to_path_buf(), (h, scan.clone()));
            }
            return scan;
        }
        if let Some(scan) = index.files.get(path) {
            return scan.clone();
        }
        let text = self.read(path);
        if self.stopped() {
            return Rc::default();
        }
        let scan = Rc::new(text.map(|text| exports::scan(&text)).unwrap_or_default());
        index.note_loads(path, &scan);
        index.files.insert(path.to_path_buf(), scan.clone());
        scan
    }
}

#[cfg(test)]
mod exact_input_tests {
    use super::*;

    #[test]
    fn captured_input_scan_cannot_inherit_an_advisory_hash_collision() {
        let mut session = Session::with_environment(
            crate::Limits::default(),
            Environment {
                fs: Rc::new(jaic::sema::NativeFs),
                options: Box::new(|_, _| jaic::sema::Options::host()),
                services: None,
            },
        );
        let uri = crate::DocumentUri::parse("file:///scan-proof/main.jai").unwrap();
        session.open(uri.clone(), 1, "new :: 2;".into()).unwrap();
        let _capture = session.capture_discovery_inputs();
        let sources = session.sources(session.environment.as_ref().unwrap());
        let path = sources.key(Path::new(uri.path()));
        let mut index = Index::default();
        index.open.insert(
            path.clone(),
            (hash("new :: 2;"), Rc::new(exports::scan("old :: 1;"))),
        );
        let scan = sources.scan(&mut index, &path);
        assert_eq!(scan.decls[0].name, "new");
    }
}

#[cfg(test)]
mod advisory_tests;
