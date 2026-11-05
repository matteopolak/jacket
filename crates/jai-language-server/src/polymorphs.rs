//! Workspace-wide generic instances without retaining every compiler in memory.
//!
//! Each entry program keeps a summary of its instances and of the paths its compile read. An
//! edit makes a summary stale without dropping it: a lens request recompiles only the stale
//! programs that read the requested file, and leaves the others to `polymorph_background`,
//! labelling its answer incomplete until they are done.
use crate::semantic::{Analysis, Buffers, CheckedInputs};
use crate::{DocumentUri, Session};
use jaic::source::{FileId, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

type Declaration = (PathBuf, u32, String);

#[derive(Clone, Default)]
struct Row {
    end: u32,
    instances: BTreeMap<String, (String, String)>,
}

struct Snapshot {
    dirs: Vec<PathBuf>,
    trusted: bool,
    /// Explicit notification staleness supplements on-demand input validation.
    stale: bool,
    input: CheckedInputs,
    rows: BTreeMap<Declaration, Row>,
    complete: bool,
}

impl Snapshot {
    fn bytes(&self) -> usize {
        self.input.bytes
            + std::mem::size_of::<Self>()
            + self.dirs.iter().map(PathBuf::capacity).sum::<usize>()
            + self
                .rows
                .iter()
                .map(|((path, _, name), row)| {
                    path.capacity()
                        + name.capacity()
                        + std::mem::size_of::<(Declaration, Row)>()
                        + 64
                        + row
                            .instances
                            .iter()
                            .map(|(identity, (label, detail))| {
                                identity.capacity() + label.capacity() + detail.capacity() + 128
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
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
            snapshot.stale |= topology || snapshot.input.reads.concerns(path);
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
    /// Exact overlay bytes for current summary validation.
    fn polymorph_buffers(&self) -> Buffers {
        Rc::new(
            self.documents
                .iter()
                .map(|(u, d)| (PathBuf::from(u.path()), Arc::from(d.ide_text().as_bytes())))
                .collect(),
        )
    }

    /// Whether `root`'s summary is current, else whether a request for `file` needs it now.
    fn polymorph_need(
        &self,
        root: &Path,
        dirs: &[PathBuf],
        buffers: &Buffers,
        file: Option<&Path>,
    ) -> Need {
        let mut index = self.polymorph_index.borrow_mut();
        let Some(snapshot) = index.programs.get_mut(root) else {
            return Need::Now;
        };
        if snapshot.dirs != dirs || snapshot.trusted != self.trusted() {
            return Need::Now;
        }
        let env = self.environment.as_ref().expect("checked by callers");
        if !snapshot.stale
            && snapshot
                .input
                .is_current(env, buffers, self.semantic.borrow().authority())
        {
            return Need::Current;
        }
        match file {
            Some(file) if !snapshot.input.reads.concerns(file) => Need::Later,
            _ => Need::Now,
        }
    }

    fn begin_polymorph_inputs(&self, buffers: &Buffers) -> bool {
        if !self.discovery_inputs_current() {
            return false;
        }
        let Some(env) = &self.environment else {
            return false;
        };
        let mut cache = self.semantic.borrow_mut();
        cache.validate_probes(env, buffers);
        self.sync_builds(&cache);
        self.index.borrow_mut().discard_input_memos();
        *self.inputs.borrow_mut() = None;
        *self.input_proof.borrow_mut() = self.request_input_proof.borrow().clone();
        true
    }

    /// Compile `root` (or take the editor's compile of it with these texts) and summarize it.
    /// The editor's own compiles are not evicted for it.
    fn summarize(&self, root: &Path, dirs: Vec<PathBuf>, buffers: &Buffers) {
        let env = self.environment.as_ref().expect("checked by callers");
        let sources = self.sources(env);
        let trusted = self.trusted();
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let detached;
        let analysis: &Analysis = if let Some(analysis) = cache.cached(env, root, &dirs, buffers) {
            analysis
        } else {
            let Some(compiled) = cache.compile_detached(env, root, &dirs, Rc::clone(buffers))
            else {
                return;
            };
            detached = compiled;
            &detached
        };
        let mut snapshot = Snapshot {
            dirs,
            trusted,
            stale: false,
            input: CheckedInputs::capture(analysis),
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
        analysis
            .reads
            .borrow_mut()
            .disk
            .merge(&self.input_proof.borrow());
        if !analysis.reads.borrow().is_current(&*env.fs) {
            return;
        }
        snapshot.input = CheckedInputs::capture(analysis);
        self.sync_builds(&cache);
        drop(cache);
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
        let _query = self.discovery_query();
        let _capture = self.capture_discovery_inputs();
        if self.environment.is_none() {
            self.polymorph_index.borrow_mut().pending.clear();
            return false;
        }
        let buffers = self.polymorph_buffers();
        if !self.begin_polymorph_inputs(&buffers) {
            return false;
        }
        loop {
            let Some(root) = self.polymorph_index.borrow_mut().pending.pop_first() else {
                return false;
            };
            if !self.polymorph_index.borrow().programs.contains_key(&root) {
                continue;
            }
            let dirs = self.import_dirs(&root);
            if let Need::Current = self.polymorph_need(&root, &dirs, &buffers, None) {
                continue;
            }
            // Stopped by newer input, the summary stays queued for the next quiet moment.
            jaic::interrupt::arm();
            self.summarize(&root, dirs, &buffers);
            if jaic::interrupt::disarm() {
                self.polymorph_index.borrow_mut().pending.insert(root);
                return false;
            }
            let mut index = self.polymorph_index.borrow_mut();
            let mut bytes = index.programs.values().map(Snapshot::bytes).sum::<usize>();
            while bytes > 16 << 20 {
                let root = index.programs.keys().next().cloned().unwrap();
                bytes -= index.programs.remove(&root).unwrap().bytes();
            }
            return true;
        }
    }

    /// All current workspace programs contribute, regardless of the declaration's location.
    pub(crate) fn workspace_polymorphs(
        &self,
        uri: &DocumentUri,
    ) -> Vec<(Span, String, Vec<String>, bool)> {
        let _query = self.discovery_query();
        let _capture = self.capture_discovery_inputs();
        let Some(env) = &self.environment else {
            return Vec::new();
        };
        let buffers = self.polymorph_buffers();
        if !self.begin_polymorph_inputs(&buffers) {
            return Vec::new();
        }
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
        let path = self.sources(env).key(Path::new(uri.path()));
        let file = jaic::paths::file_identity(
            &self
                .config_filesystem()
                .expect("environment owns filesystem"),
            Path::new(uri.path()),
        );
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
            match self.polymorph_need(root, &dirs, &buffers, Some(&file)) {
                Need::Current => {}
                Need::Now if self.may_compile() => self.summarize(root, dirs, &buffers),
                Need::Now | Need::Later => {
                    // Its summary has no rows for this file; an edit may have made it load the
                    // file, which only its compile can tell. A request leaves that compile to
                    // `background_work`.
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
            if !snapshot
                .input
                .is_current(env, &buffers, self.semantic.borrow().authority())
            {
                complete = false;
                continue;
            }
            complete &= snapshot.complete;
            for (key, row) in &snapshot.rows {
                if key.0 == path {
                    let at = combined.entry(key.clone()).or_default();
                    at.end = row.end;
                    at.instances.extend(row.instances.clone());
                }
            }
        }
        let mut bytes = index.programs.values().map(Snapshot::bytes).sum::<usize>();
        while bytes > 16 << 20 {
            let root = index.programs.keys().next().cloned().unwrap();
            bytes -= index.programs.remove(&root).unwrap().bytes();
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
