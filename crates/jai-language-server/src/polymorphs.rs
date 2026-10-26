//! Workspace-wide generic instances without retaining every compiler in memory.
//!
//! Each entry program keeps a summary of its instances and of the paths its compile read. An
//! edit makes a summary stale without dropping it: a lens request recompiles only the stale
//! programs that read the requested file, and leaves the others to `polymorph_background`,
//! labelling its answer incomplete until they are done.
use crate::semantic::{Analysis, Buffers, Reads, text_hash};
use crate::{DocumentUri, Session};
use jaic::source::{FileId, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

type Declaration = (PathBuf, u32, String);

#[derive(Clone, Default)]
struct Row {
    end: u32,
    instances: BTreeMap<String, (String, String)>,
}

struct Snapshot {
    dirs: Vec<PathBuf>,
    trusted: bool,
    /// `Session::semantic_revision` when the summary was last known to be current.
    checked_revision: u64,
    /// A file on disk that the compile read changed (or files appeared or went away).
    stale: bool,
    /// The paths the compile asked about, by canonical key.
    reads: Reads,
    /// The open texts among them, hashed.
    buffers: BTreeMap<PathBuf, u64>,
    rows: BTreeMap<Declaration, Row>,
    complete: bool,
}

impl Snapshot {
    /// Whether the inputs changed since the compile, given the open texts now.
    fn outdated(&self, open: &BTreeMap<PathBuf, u64>) -> bool {
        self.stale
            || self
                .buffers
                .iter()
                .any(|(path, hash)| open.get(path) != Some(hash))
            || open
                .keys()
                .any(|path| !self.buffers.contains_key(path) && self.reads.concerns(path))
    }
}

#[derive(Default)]
pub(crate) struct Index {
    programs: BTreeMap<PathBuf, Snapshot>,
    /// Stale programs a lens request did not recompile: `polymorph_background` does.
    pending: BTreeSet<PathBuf>,
}

impl Index {
    pub(crate) fn clear(&mut self) {
        self.programs.clear();
        self.pending.clear();
    }

    /// A file on disk changed (`topology`: it appeared or went away).
    pub(crate) fn invalidate(&mut self, path: &Path, topology: bool) {
        // Settings can change which programs and dependencies exist.
        if path.extension().is_some_and(|e| e == "toml") {
            self.clear();
            return;
        }
        // A new file can satisfy an import that failed, under a spelling the compile may not
        // have asked about: every summary is stale, though none is dropped.
        for snapshot in self.programs.values_mut() {
            snapshot.stale |= topology || snapshot.reads.concerns(path);
        }
    }

    /// Stale programs are waiting for `Session::polymorph_background`.
    pub(crate) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }
}

/// What a lens request does about one program.
enum Need {
    /// Its summary is current.
    Current,
    /// Stale, but its compile never read the requested file: answered later.
    Later,
    /// Compiled now.
    Now,
}

impl Session {
    /// The open texts for a compile, and their hashes by canonical key.
    fn polymorph_buffers(&self) -> (Buffers, BTreeMap<PathBuf, u64>) {
        let env = self.environment.as_ref().expect("checked by callers");
        let buffers: Buffers = Rc::new(
            self.documents
                .iter()
                .map(|(u, d)| (PathBuf::from(u.path()), Rc::from(d.ide_text().as_bytes())))
                .collect(),
        );
        let open = buffers
            .iter()
            .map(|(path, bytes)| (jaic::paths::file_identity(&*env.fs, path), text_hash(bytes)))
            .collect();
        (buffers, open)
    }

    /// Whether `root`'s summary is current, else whether a request for `file` needs it now.
    fn polymorph_need(
        &self,
        root: &Path,
        dirs: &[PathBuf],
        open: &BTreeMap<PathBuf, u64>,
        file: Option<&Path>,
    ) -> Need {
        let mut index = self.polymorph_index.borrow_mut();
        let Some(snapshot) = index.programs.get_mut(root) else {
            return Need::Now;
        };
        if snapshot.dirs != dirs || snapshot.trusted != self.trusted() {
            return Need::Now;
        }
        if snapshot.checked_revision == self.semantic_revision() && !snapshot.stale {
            return Need::Current;
        }
        if !snapshot.outdated(open) {
            snapshot.checked_revision = self.semantic_revision();
            return Need::Current;
        }
        match file {
            Some(file) if !snapshot.reads.concerns(file) => Need::Later,
            _ => Need::Now,
        }
    }

    /// Compile `root` (or take the editor's compile of it with these texts) and summarize it.
    /// The editor's own compiles are not evicted for it.
    fn summarize(&self, root: &Path, dirs: Vec<PathBuf>, buffers: &Buffers) {
        let env = self.environment.as_ref().expect("checked by callers");
        let sources = self.sources(env);
        let trusted = self.trusted();
        let mut cache = self.semantic.borrow_mut();
        let detached;
        let analysis: &Analysis = if let Some(analysis) = cache.cached(env, root, &dirs, buffers) {
            analysis
        } else {
            detached = cache.compile_detached(env, root, &dirs, Rc::clone(buffers));
            &detached
        };
        let (reads, open) = analysis.inputs_read();
        let mut snapshot = Snapshot {
            dirs,
            trusted,
            checked_revision: self.semantic_revision(),
            stale: false,
            reads,
            buffers: open,
            rows: BTreeMap::new(),
            complete: true,
        };
        for (compiler, ok) in analysis.polymorph_compilers() {
            snapshot.complete &= ok;
            for row in compiler.ide_polymorph_summaries() {
                let Some(file) = compiler.src.sources.get(row.span.file).path.as_file() else {
                    continue;
                };
                let at = snapshot
                    .rows
                    .entry((sources.key(file), row.span.start, row.name))
                    .or_default();
                at.end = row.span.end;
                at.instances.extend(
                    row.instances
                        .into_iter()
                        .map(|i| (i.identity, (i.bindings, i.detail))),
                );
            }
        }
        self.sync_builds(&cache);
        drop(cache);
        snapshot.dirs = self.import_dirs(root);
        let mut index = self.polymorph_index.borrow_mut();
        index.pending.remove(root);
        index.programs.insert(root.to_path_buf(), snapshot);
    }

    /// Whether `background_work` has something to do.
    pub fn has_background_work(&self) -> bool {
        self.polymorph_index.borrow().has_pending()
    }

    /// Summarize one stale program a lens request left for later, for a transport to call
    /// while no input waits (one compile at most, so input is read again soon). True when a
    /// summary changed, so lenses are worth asking for again; false when nothing was waiting.
    /// The queue is rechecked first, so work an edit made pointless is dropped unstarted.
    pub fn background_work(&self) -> bool {
        if self.environment.is_none() {
            self.polymorph_index.borrow_mut().pending.clear();
            return false;
        }
        let (buffers, open) = self.polymorph_buffers();
        loop {
            let Some(root) = self.polymorph_index.borrow_mut().pending.pop_first() else {
                return false;
            };
            if !self.polymorph_index.borrow().programs.contains_key(&root) {
                continue;
            }
            let dirs = self.import_dirs(&root);
            if let Need::Current = self.polymorph_need(&root, &dirs, &open, None) {
                continue;
            }
            self.summarize(&root, dirs, &buffers);
            return true;
        }
    }

    /// All current workspace programs contribute, regardless of the declaration's location.
    pub(crate) fn workspace_polymorphs(
        &self,
        uri: &DocumentUri,
    ) -> Vec<(Span, String, Vec<String>, bool)> {
        let Some(env) = &self.environment else {
            return Vec::new();
        };
        let open_paths: Vec<_> = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect();
        let mut roots = {
            let sources = self.sources(env);
            sources.workspace_entries(
                &mut self.index.borrow_mut(),
                &self.workspace_folders,
                &open_paths,
            )
        };
        if roots.is_empty() {
            roots.insert(PathBuf::from(uri.path()));
        }
        let (buffers, open) = self.polymorph_buffers();
        let path = self.sources(env).key(Path::new(uri.path()));
        let file = jaic::paths::file_identity(&*env.fs, Path::new(uri.path()));
        let built_child = |root: &PathBuf| {
            self.index.borrow().builds.iter().any(|(build, facts)| {
                build != root
                    && roots.contains(build)
                    && facts.programs.iter().any(|p| p.files.contains(root))
            })
        };
        let mut complete = true;
        let mut used = BTreeSet::new();
        for root in &roots {
            if built_child(root) {
                continue;
            }
            used.insert(root.clone());
            let dirs = self.import_dirs(root);
            match self.polymorph_need(root, &dirs, &open, Some(&file)) {
                Need::Current => {}
                Need::Now => self.summarize(root, dirs, &buffers),
                Need::Later => {
                    // Its summary has no rows for this file; an edit may have made it load the
                    // file, which only its compile can tell.
                    complete = false;
                    self.polymorph_index
                        .borrow_mut()
                        .pending
                        .insert(root.clone());
                }
            }
        }
        // A metaprogram discovered later in the pass may own a root checked earlier.
        // Merge only after build discovery settles, so that standalone child's errors
        // and instances cannot leak into the workspace result.
        used.retain(|root| !built_child(root));
        let mut combined: BTreeMap<Declaration, Row> = BTreeMap::new();
        let mut index = self.polymorph_index.borrow_mut();
        index.programs.retain(|root, _| used.contains(root));
        index.pending.retain(|root| used.contains(root));
        for snapshot in index.programs.values() {
            complete &= snapshot.complete;
            for (key, row) in &snapshot.rows {
                if key.0 == path {
                    let at = combined.entry(key.clone()).or_default();
                    at.end = row.end;
                    at.instances.extend(row.instances.clone());
                }
            }
        }
        drop(index);
        combined
            .into_iter()
            .map(|((_, start, name), row)| {
                let mut counts = BTreeMap::<String, usize>::new();
                for (label, _) in row.instances.values() {
                    *counts.entry(label.clone()).or_default() += 1;
                }
                let mut values: Vec<_> = row
                    .instances
                    .into_values()
                    .map(|(label, detail)| {
                        if counts[&label] > 1 {
                            detail
                        } else {
                            label
                        }
                    })
                    .collect();
                values.sort();
                (
                    Span {
                        file: FileId(0),
                        start,
                        end: row.end,
                    },
                    name,
                    values,
                    complete,
                )
            })
            .collect()
    }
}
