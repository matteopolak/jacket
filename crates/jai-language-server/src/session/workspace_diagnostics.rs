//! Completed workspace diagnostic snapshots, independent of which files the editor opens.
use super::Session;
use crate::analysis::{Analysis, Span};
use crate::semantic::{Reads, text_hash};
use crate::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, position::LineIndex};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

pub(crate) type WorkspaceReport = (String, Option<i32>, Vec<Diagnostic>);

struct FileDiagnostics {
    stamp: u64,
    items: Vec<Diagnostic>,
}

struct Snapshot {
    dependencies: Reads,
    stale: bool,
    files: BTreeMap<PathBuf, FileDiagnostics>,
}

#[derive(Default)]
pub(super) struct WorkspaceDiagnostics {
    active: bool,
    roots: BTreeMap<PathBuf, Snapshot>,
}

impl WorkspaceDiagnostics {
    pub(super) fn invalidate(&mut self, path: &Path, topology: bool) {
        for snapshot in self.roots.values_mut() {
            snapshot.files.remove(path);
            snapshot.stale |= topology
                || path.extension().is_some_and(|e| e == "toml")
                || snapshot.dependencies.concerns(path);
        }
    }

    pub(super) fn invalidate_all(&mut self) {
        for snapshot in self.roots.values_mut() {
            snapshot.stale = true;
        }
    }

    pub(super) fn clear(&mut self) {
        self.roots.clear();
    }
}

impl Session {
    /// Start closed-file diagnostics after the LSP client's initialized notification.
    pub(crate) fn enable_workspace_diagnostics(&self) {
        self.workspace_diagnostics.borrow_mut().active = true;
    }

    fn diagnostic_entries(&self) -> Option<Vec<PathBuf>> {
        if !self.workspace_diagnostics.borrow().active || !self.trusted() {
            return None;
        }
        let env = self.environment.as_ref()?;
        let sources = self.sources(env);
        let open = self
            .documents
            .keys()
            .map(|u| PathBuf::from(u.path()))
            .collect::<Vec<_>>();
        Some(
            sources
                .workspace_entries(&mut self.index.borrow_mut(), &self.workspace_folders, &open)
                .into_iter()
                .collect(),
        )
    }

    pub(super) fn workspace_diagnostics_pending(&self) -> bool {
        let Some(entries) = self.diagnostic_entries() else {
            return false;
        };
        let state = self.workspace_diagnostics.borrow();
        state.roots.keys().any(|root| !entries.contains(root))
            || entries
                .iter()
                .any(|root| state.roots.get(root).is_none_or(|s| s.stale))
    }

    /// Check one program's diagnostics as a batch. Interrupted results never replace a
    /// completed snapshot; a client fetches the new snapshot after indexing finishes.
    pub(super) fn index_workspace_diagnostics(&self) -> Option<(usize, usize)> {
        let entries = self.diagnostic_entries()?;
        let removed = {
            let mut state = self.workspace_diagnostics.borrow_mut();
            let old = state.roots.len();
            state.roots.retain(|root, _| entries.contains(root));
            old != state.roots.len()
        };
        let root = entries
            .iter()
            .find(|root| {
                self.workspace_diagnostics
                    .borrow()
                    .roots
                    .get(*root)
                    .is_none_or(|s| s.stale)
            })
            .cloned();
        let Some(root) = root else {
            let counts = (entries.len(), entries.len());
            self.record_diagnostic_counts(counts);
            return removed.then_some(counts);
        };
        let env = self.environment.as_ref()?;
        let buffers = Rc::new(
            self.documents
                .iter()
                .map(|(uri, doc)| {
                    (
                        PathBuf::from(uri.path()),
                        Arc::<[u8]>::from(doc.ide_text().as_bytes()),
                    )
                })
                .collect(),
        );
        let dirs = self.import_dirs(&root);
        let _background = self.background();
        jaic::interrupt::arm();
        let snapshot = (|| {
            let mut cache = self.semantic.borrow_mut();
            let analysis = cache.analyze(env, &root, &dirs, buffers)?;
            let scope: Vec<_> = if self.workspace_folders.is_empty() {
                root.parent().into_iter().map(Path::to_path_buf).collect()
            } else {
                self.workspace_folders
                    .iter()
                    .map(|dir| jaic::paths::file_identity(&*env.fs, dir))
                    .collect()
            };
            let sources: BTreeMap<_, _> = analysis
                .workspace_sources()
                .into_iter()
                .filter(|(path, _)| scope.iter().any(|dir| path.starts_with(dir)))
                .collect();
            let mut groups: BTreeMap<PathBuf, (jailint::config::Config, Vec<PathBuf>)> =
                BTreeMap::new();
            for path in sources.keys() {
                if let Some(config) = self.lint_config(path) {
                    groups
                        .entry(config.root.clone())
                        .or_insert_with(|| (config, Vec::new()))
                        .1
                        .push(path.clone());
                }
            }
            if !analysis.warm_lints(&groups.into_values().collect::<Vec<_>>()) {
                return None;
            }
            let mut files = BTreeMap::new();
            for (path, compiled_text) in sources {
                if jaic::interrupt::should_stop() {
                    return None;
                }
                let uri = self.client_uri(&path)?;
                let text = self
                    .document(&uri)
                    .map_or_else(|_| compiled_text.to_string(), |doc| doc.text.clone());
                let lines = LineIndex::new(&text);
                let syntax = self
                    .analysis_for(&uri)
                    .cloned()
                    .unwrap_or_else(|| Rc::new(Analysis::build(&text, &uri, self.limits)));
                let mut items = syntax.diagnostics.clone();
                analysis.focus(&path);
                for finding in analysis.check_findings(&path) {
                    let Ok(range) = lines.range(&text, Span::new(finding.start, finding.end))
                    else {
                        continue;
                    };
                    if syntax.diagnostics.iter().any(|d| {
                        matches!(d.code, DiagnosticCode::Lexer | DiagnosticCode::Parser)
                            && d.range.start == range.start
                            && d.message == finding.message
                    }) {
                        continue;
                    }
                    items.push(self.check_finding_at(finding, range).diagnostic);
                }
                for (start, end) in analysis.unreachable(&path) {
                    if let Ok(range) = lines.range(&text, Span::new(start, end)) {
                        items.push(Diagnostic {
                            range,
                            severity: DiagnosticSeverity::Hint,
                            code: DiagnosticCode::Unreachable,
                            message: "unreachable code".into(),
                            related: Vec::new(),
                        });
                    }
                }
                if let Some(config) = self.lint_config(&path) {
                    for lint in analysis.lints(&path, &config)? {
                        if let Ok(range) = lines.range(&text, Span::new(lint.start, lint.end)) {
                            items.push(self.lint_diagnostic_at(&lint, range));
                        }
                    }
                }
                files.insert(
                    path,
                    FileDiagnostics {
                        stamp: text_hash(text.as_bytes()),
                        items,
                    },
                );
            }
            let (dependencies, _) = analysis.inputs_read();
            self.sync_builds(&cache);
            Some(Snapshot {
                dependencies,
                stale: false,
                files,
            })
        })();
        let interrupted = jaic::interrupt::disarm();
        if !interrupted && let Some(snapshot) = snapshot {
            self.workspace_diagnostics
                .borrow_mut()
                .roots
                .insert(root, snapshot);
        }
        let done = self
            .workspace_diagnostics
            .borrow()
            .roots
            .values()
            .filter(|s| !s.stale)
            .count();
        self.record_diagnostic_counts((done, entries.len()));
        Some((done, entries.len()))
    }

    pub(super) fn cached_workspace_diagnostics(
        &self,
        uri: &DocumentUri,
    ) -> Option<Vec<Diagnostic>> {
        if self.workspace_folders.is_empty() && self.documents.is_empty() {
            return None;
        }
        let env = self.environment.as_ref()?;
        let path = jaic::paths::file_identity(&*env.fs, Path::new(uri.path()));
        let state = self.workspace_diagnostics.borrow();
        for snapshot in state.roots.values().filter(|s| !s.stale) {
            if let Some(file) = snapshot.files.get(&path) {
                if let Ok(doc) = self.document(uri)
                    && text_hash(doc.text.as_bytes()) != file.stamp
                {
                    continue;
                }
                return Some(file.items.clone());
            }
        }
        None
    }

    pub(crate) fn closed_workspace_diagnostics(
        &self,
        uri: &DocumentUri,
    ) -> Option<Vec<Diagnostic>> {
        if self.workspace_folders.is_empty() && self.documents.is_empty() {
            return None;
        }
        let env = self.environment.as_ref()?;
        let path = jaic::paths::file_identity(&*env.fs, Path::new(uri.path()));
        self.workspace_diagnostics
            .borrow()
            .roots
            .values()
            .find_map(|snapshot| snapshot.files.get(&path).map(|file| file.items.clone()))
    }

    /// Cached reports for all included workspace files, without starting foreground lint work.
    pub(crate) fn workspace_reports(&self) -> Option<Vec<WorkspaceReport>> {
        let state = self.workspace_diagnostics.borrow();
        if !state.active {
            return None;
        }
        let mut files = BTreeMap::new();
        for snapshot in state.roots.values() {
            for (path, file) in &snapshot.files {
                files.entry(path).or_insert(file);
            }
        }
        Some(
            files
                .into_iter()
                .filter_map(|(path, file)| {
                    let uri = self.client_uri(path)?;
                    let version = match self.document(&uri) {
                        Ok(doc) if text_hash(doc.text.as_bytes()) == file.stamp => {
                            Some(doc.version)
                        }
                        Ok(_) => return None,
                        Err(_) => None,
                    };
                    Some((uri.as_str().into(), version, file.items.clone()))
                })
                .collect(),
        )
    }
}
