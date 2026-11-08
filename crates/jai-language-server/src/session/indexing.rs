//! Build project analyses from their entries before individual documents need them.
use super::Session;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Default)]
pub(super) struct WorkspaceIndex {
    generation: Option<u64>,
    entries: Vec<PathBuf>,
    next: usize,
}

impl Session {
    /// Discovered workspace files; build metaprograms can expand this set while indexing.
    pub fn indexing_message(&self) -> String {
        let files = self
            .index
            .borrow()
            .builds
            .values()
            .flat_map(|facts| facts.sources.keys())
            .filter(|path| {
                self.workspace_folders
                    .iter()
                    .any(|dir| path.starts_with(dir))
            })
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        if files == 0 {
            "indexing".into()
        } else {
            format!("indexing {files} files")
        }
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
        let sources = self.sources(env);
        let open = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect::<Vec<_>>();
        let mut index = self.index.borrow_mut();
        let entries = sources.workspace_entries(&mut index, &self.workspace_folders, &open);
        let pending = entries.iter().any(|entry| {
            !done.contains(entry)
                && (!index.builds.contains_key(entry)
                    || index.stale_source_graphs.contains_key(entry))
        });
        if !pending {
            *self.workspace_index.borrow_mut() = WorkspaceIndex {
                generation: Some(generation),
                next: entries.len(),
                entries: entries.into_iter().collect(),
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
        let env = self.environment.as_ref()?;
        let generation = self.generation.get();
        let sources = self.sources(env);
        let open = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect::<Vec<_>>();
        let entries =
            sources.workspace_entries(&mut self.index.borrow_mut(), &self.workspace_folders, &open);
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
