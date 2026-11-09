//! Build project analyses from their entries before individual documents need them.
use super::Session;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

/// How far each indexing phase has got: `(finished, total)` entries, then diagnostic roots.
#[derive(Clone, Copy, Default)]
pub(super) struct IndexCounts {
    semantic: (usize, usize),
    diagnostics: (usize, usize),
}

/// The workspace's entry programs, with what they were worked out from.
#[derive(Default)]
pub(super) struct EntryMemo {
    key: Option<(u64, u64, Vec<PathBuf>)>,
    entries: Rc<BTreeSet<PathBuf>>,
    /// How many times they were worked out (see `Session::entry_scans`).
    scans: u64,
}

#[derive(Default)]
pub(super) struct WorkspaceIndex {
    generation: Option<u64>,
    entries: Vec<PathBuf>,
    next: usize,
}

impl Session {
    /// The programs the workspace is made of: the entries of the workspace folders and of the
    /// open documents' projects (`Sources::workspace_entries`). Worked out again only when
    /// what it reads changed: the generation (open documents, edits, notified file changes),
    /// the index's epoch (disk changes, build facts recorded by a compile) or the folders. A
    /// file changed on disk without a notification is noticed when one of those moves.
    pub(crate) fn workspace_entry_set(
        &self,
        env: &crate::semantic::Environment,
    ) -> Rc<BTreeSet<PathBuf>> {
        let key = (
            self.generation.get(),
            self.index.borrow().epoch(),
            self.workspace_folders.clone(),
        );
        let mut memo = self.entries.borrow_mut();
        if memo.key.as_ref() == Some(&key) {
            return memo.entries.clone();
        }
        let sources = self.sources(env);
        let open = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect::<Vec<_>>();
        let entries =
            sources.workspace_entries(&mut self.index.borrow_mut(), &self.workspace_folders, &open);
        memo.scans += 1;
        memo.entries = Rc::new(entries);
        // Working them out reads `epoch`: take it again, as the walk can have recorded edits.
        memo.key = Some((key.0, self.index.borrow().epoch(), key.2));
        memo.entries.clone()
    }

    /// How many times `workspace_entry_set` worked the entries out rather than reusing them.
    pub fn entry_scans(&self) -> u64 {
        self.entries.borrow().scans
    }

    /// One combined `(finished, total)` over both indexing phases: the entries analysed, then
    /// the diagnostic roots checked. The diagnostics total is the entry total until that phase
    /// reports its own, so the count does not start over when the phases hand over. `(0, 0)`
    /// until a step has run.
    pub fn indexing_counts(&self) -> (usize, usize) {
        let counts = self.index_counts.get();
        let (done, total) = counts.semantic;
        let (checked, roots) = counts.diagnostics;
        let roots = if roots == 0 {
            total
        } else {
            roots
        };
        (done + checked, total + roots)
    }

    /// Forget the counts when indexing finishes, so the next run starts from zero.
    pub fn reset_indexing_counts(&self) {
        self.index_counts.set(IndexCounts::default());
    }

    pub(super) fn record_diagnostic_counts(&self, counts: (usize, usize)) {
        let mut all = self.index_counts.get();
        all.semantic.0 = all.semantic.1;
        all.diagnostics = counts;
        self.index_counts.set(all);
    }

    /// Whether workspace entries still need indexing for the current source snapshot.
    pub fn indexing_pending(&self) -> bool {
        self.semantic_indexing_pending() || self.workspace_diagnostics_pending()
    }

    fn semantic_indexing_pending(&self) -> bool {
        let Some(env) = self.environment.as_ref() else {
            return false;
        };
        if !self.trusted() || self.workspace_folders.is_empty() && self.documents.is_empty() {
            return false;
        }
        let generation = self.generation.get();
        let state = self.workspace_index.borrow();
        if state.generation == Some(generation) && state.next == state.entries.len() {
            return false;
        }
        let done = if state.generation == Some(generation) {
            state.entries[..state.next].to_vec()
        } else {
            Vec::new()
        };
        drop(state);
        let entries = self.workspace_entry_set(env);
        let index = self.index.borrow();
        let pending = entries.iter().any(|entry| {
            !done.contains(entry)
                && (!index.builds.contains_key(entry)
                    || index.stale_source_graphs.contains_key(entry))
        });
        if !pending {
            *self.workspace_index.borrow_mut() = WorkspaceIndex {
                generation: Some(generation),
                next: entries.len(),
                entries: entries.iter().cloned().collect(),
            };
        }
        pending
    }

    /// Index one entry, including the child workspaces its build metaprogram creates.
    /// Returns `(finished entries, total entries)`; newer input leaves this entry pending.
    /// A native transport calls this while idle, then checks its input again.
    pub fn index_workspace(&self) -> Option<(usize, usize)> {
        if !self.semantic_indexing_pending() {
            return self.index_workspace_diagnostics();
        }
        let counts = self.index_workspace_entry()?;
        let mut all = self.index_counts.get();
        all.semantic = counts;
        self.index_counts.set(all);
        Some(counts)
    }

    fn index_workspace_entry(&self) -> Option<(usize, usize)> {
        let env = self.environment.as_ref()?;
        let generation = self.generation.get();
        let entries = self.workspace_entry_set(env);
        if self.workspace_index.borrow().generation != Some(generation) {
            *self.workspace_index.borrow_mut() = WorkspaceIndex {
                generation: Some(generation),
                entries: entries.iter().cloned().collect(),
                next: 0,
            };
        }
        let root = self
            .workspace_index
            .borrow()
            .entries
            .get(self.workspace_index.borrow().next)
            .cloned();
        if let Some(root) = root {
            if !entries.contains(&root) {
                let mut state = self.workspace_index.borrow_mut();
                state.next += 1;
                return Some((state.next, state.entries.len()));
            }
            let buffers = self
                .documents
                .iter()
                .map(|(u, d)| {
                    (
                        PathBuf::from(u.path()),
                        Arc::<[u8]>::from(d.ide_text().as_bytes()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            let dirs = self.import_dirs(&root);
            let _background = self.background();
            jaic::interrupt::arm();
            {
                let mut cache = self.semantic.borrow_mut();
                cache.analyze(env, &root, &dirs, Rc::new(buffers));
                self.sync_builds(&cache);
            }
            if !jaic::interrupt::disarm() {
                self.workspace_index.borrow_mut().next += 1;
            }
        }
        let state = self.workspace_index.borrow();
        Some((state.next, state.entries.len()))
    }
}
