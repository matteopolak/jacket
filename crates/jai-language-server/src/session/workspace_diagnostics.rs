//! Completed workspace diagnostic snapshots, independent of which files the editor opens.
use super::Session;
use crate::analysis::{Analysis, Span};
use crate::semantic::{Reads, text_hash};
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Error, position::LineIndex,
};
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
    /// The project files no entry reaches: their syntax and the note that says so.
    unreached: BTreeMap<PathBuf, FileDiagnostics>,
    /// Whether `unreached` is what the programs and files are now.
    unreached_fresh: bool,
    /// The index epoch it was worked out at: what a build records afterwards (an entry's
    /// graph was out of date when it ran) decides it again.
    unreached_epoch: Option<u64>,
}

/// The note on a file that is part of no program, as an information on its first line.
pub(super) fn not_in_program(text: &str) -> Diagnostic {
    let end = text.find('\n').unwrap_or(text.len());
    let lines = LineIndex::new(text);
    Diagnostic {
        range: lines.range(text, Span::new(0, end)).unwrap_or_default(),
        severity: DiagnosticSeverity::Information,
        code: DiagnosticCode::NotInProgram,
        message: "No program entry loads or imports this file, so it is not checked as part \
                  of one: names it takes from other files are unknown to it, and its types \
                  and lints are not checked against the project."
            .into(),
        related: Vec::new(),
    }
}

/// Add `more` to `all`, skipping what a program already reported: a file several programs
/// include (`#load`ed by two entries) shows what any of them finds wrong with it.
pub(super) fn merge(all: &mut Vec<Diagnostic>, more: &[Diagnostic]) {
    for item in more {
        if !all.contains(item) {
            all.push(item.clone());
        }
    }
}

impl WorkspaceDiagnostics {
    pub(super) fn invalidate(&mut self, path: &Path, topology: bool) {
        let mut reach = false;
        let mut in_program = false;
        for snapshot in self.roots.values_mut() {
            in_program |= snapshot.files.remove(path).is_some();
            let hit = topology
                || path.extension().is_some_and(|e| e == "toml")
                || snapshot.dependencies.concerns(path);
            snapshot.stale |= hit;
            reach |= hit;
        }
        // A file no program holds can be one no program reaches: opening, closing or editing
        // it changes what is said of it.
        let loose = !in_program && path.extension().is_some_and(|e| e == "jai");
        if reach || loose {
            self.unreached_fresh = false;
        }
    }

    pub(super) fn invalidate_all(&mut self) {
        for snapshot in self.roots.values_mut() {
            snapshot.stale = true;
        }
        self.unreached_fresh = false;
    }

    pub(super) fn clear(&mut self) {
        self.roots.clear();
        self.unreached.clear();
        self.unreached_fresh = false;
    }
}

impl Session {
    /// What the compile in `analysis` finds in `path` (whose text is `text`): the type
    /// checker's errors and warnings, the code that never runs and the lints, leaving out
    /// the errors the `syntax` diagnostics already say. `None` when stopped.
    pub(super) fn semantic_items(
        &self,
        analysis: &mut crate::semantic::Analysis,
        path: &Path,
        text: &str,
        lines: &LineIndex,
        syntax: &[Diagnostic],
    ) -> Option<Vec<Diagnostic>> {
        let mut items = Vec::new();
        analysis.focus(path);
        for finding in analysis.check_findings(path) {
            let Ok(range) = lines.range(text, Span::new(finding.start, finding.end)) else {
                continue;
            };
            if syntax.iter().any(|d| {
                matches!(d.code, DiagnosticCode::Lexer | DiagnosticCode::Parser)
                    && d.range.start == range.start
                    && d.message == finding.message
            }) {
                continue;
            }
            items.push(self.check_finding_at(finding, range).diagnostic);
        }
        for (start, end) in analysis.unreachable(path) {
            if let Ok(range) = lines.range(text, Span::new(start, end)) {
                items.push(Diagnostic {
                    range,
                    severity: DiagnosticSeverity::Hint,
                    code: DiagnosticCode::Unreachable,
                    message: "unreachable code".into(),
                    related: Vec::new(),
                });
            }
        }
        if let Some(config) = self.lint_config(path) {
            for lint in analysis.lints(path, &config)? {
                if let Ok(range) = lines.range(text, Span::new(lint.start, lint.end)) {
                    items.push(self.lint_diagnostic_at(&lint, range));
                }
            }
        }
        Some(items)
    }

    /// What the programs of the open document `uri` other than the one requests use report
    /// for it (`sibling_roots`), checked now: the open buffers are what they compile, and an
    /// edit leaves no snapshot to take them from. `CheckUnavailable` when one is stopped.
    pub(crate) fn sibling_program_diagnostics(
        &self,
        uri: &DocumentUri,
    ) -> Result<Vec<Diagnostic>, Error> {
        let (Ok(doc), Some(env)) = (self.document(uri), self.environment.as_ref()) else {
            return Ok(Vec::new());
        };
        let roots = self.sibling_roots(uri);
        if roots.is_empty() {
            return Ok(Vec::new());
        }
        let (files, ..) = self
            .compile_inputs(uri, &doc.text)
            .ok_or(Error::CheckUnavailable)?;
        let syntax = &self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics;
        let lines = LineIndex::new(&doc.text);
        let path = Path::new(uri.path());
        let mut found = Vec::new();
        for root in roots {
            // An untrusted workspace does not run the metaprogram that would build the file.
            if !self.trusted() && self.is_build_metaprogram(&root) {
                continue;
            }
            let dirs = self.import_dirs(&root);
            let mut cache = self.semantic.borrow_mut();
            let analysis = cache
                .analyze(env, &root, &dirs, files.clone())
                .ok_or(Error::CheckUnavailable)?;
            let items = self
                .semantic_items(analysis, path, &doc.text, &lines, syntax)
                .ok_or(Error::CheckUnavailable)?;
            self.sync_builds(&cache);
            merge(&mut found, &items);
        }
        Ok(found)
    }

    /// Bring the closed files no program reaches up to date: their syntax diagnostics and the
    /// note that says they are not checked as part of a program, open or closed. `true` when that changed.
    fn refresh_unreached(&self) -> bool {
        let Some(env) = self.environment.as_ref() else {
            return false;
        };
        let wanted: Vec<PathBuf> = self.unreached_files().into_iter().collect();
        let mut old = std::mem::take(&mut self.workspace_diagnostics.borrow_mut().unreached);
        let mut now = BTreeMap::new();
        let mut changed = false;
        for path in wanted {
            let Some(uri) = self.client_uri(&path) else {
                continue;
            };
            // What the editor shows when the file is open, else what is on disk.
            let text = match self.document(&uri) {
                Ok(doc) => doc.text.clone(),
                Err(_) => match env.fs.read(&path) {
                    Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    None => continue,
                },
            };
            let stamp = text_hash(text.as_bytes());
            let file = match old.remove(&path) {
                Some(kept) if kept.stamp == stamp => kept,
                _ => {
                    changed = true;
                    let mut items = if let Some(open) = self.analysis_for(&uri) {
                        open.diagnostics.clone()
                    } else {
                        let lines = LineIndex::new(&text);
                        let mut built = Analysis::build(&text, &uri, self.limits);
                        self.warn_unreadable_loads(&mut built, &lines, &text);
                        built.diagnostics
                    };
                    items.push(not_in_program(&text));
                    FileDiagnostics {
                        stamp,
                        items,
                    }
                }
            };
            now.insert(path, file);
        }
        changed |= !old.is_empty();
        let epoch = self.index.borrow().epoch();
        let mut state = self.workspace_diagnostics.borrow_mut();
        state.unreached = now;
        state.unreached_fresh = true;
        state.unreached_epoch = Some(epoch);
        changed
    }

    /// Start closed-file diagnostics after the LSP client's initialized notification.
    pub(crate) fn enable_workspace_diagnostics(&self) {
        self.workspace_diagnostics.borrow_mut().active = true;
    }

    fn diagnostic_entries(&self) -> Option<Vec<PathBuf>> {
        if !self.workspace_diagnostics.borrow().active || !self.trusted() {
            return None;
        }
        let env = self.environment.as_ref()?;
        Some(self.workspace_entry_set(env).iter().cloned().collect())
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
            || !state.unreached_fresh
            || state.unreached_epoch != Some(self.index.borrow().epoch())
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
            let unreached = self.refresh_unreached();
            return (removed || unreached).then_some(counts);
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
                let syntax = self.analysis_for(&uri).cloned().unwrap_or_else(|| {
                    // A closed file is warned about unreadable loads like an open one.
                    let mut built = Analysis::build(&text, &uri, self.limits);
                    self.warn_unreadable_loads(&mut built, &lines, &text);
                    Rc::new(built)
                });
                let mut items = syntax.diagnostics.clone();
                items.extend(self.semantic_items(
                    analysis,
                    &path,
                    &text,
                    &lines,
                    &syntax.diagnostics,
                )?);
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
            let mut state = self.workspace_diagnostics.borrow_mut();
            state.roots.insert(root, snapshot);
            // What the programs record decides what none of them reaches.
            state.unreached_fresh = false;
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
        let current = self
            .document(uri)
            .ok()
            .map(|doc| text_hash(doc.text.as_bytes()));
        let mut found: Option<Vec<Diagnostic>> = None;
        for snapshot in state.roots.values().filter(|s| !s.stale) {
            if let Some(file) = snapshot.files.get(&path)
                && current.is_none_or(|stamp| stamp == file.stamp)
            {
                merge(found.get_or_insert_with(Vec::new), &file.items);
            }
        }
        found
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
        let state = self.workspace_diagnostics.borrow();
        let mut found: Option<Vec<Diagnostic>> = None;
        for file in state
            .roots
            .values()
            .filter_map(|s| s.files.get(&path))
            .chain(state.unreached.get(&path))
        {
            merge(found.get_or_insert_with(Vec::new), &file.items);
        }
        found
    }

    /// Cached reports for all included workspace files, without starting foreground lint work.
    pub(crate) fn workspace_reports(&self) -> Option<Vec<WorkspaceReport>> {
        let state = self.workspace_diagnostics.borrow();
        if !state.active {
            return None;
        }
        let mut files: BTreeMap<&PathBuf, Vec<&FileDiagnostics>> = BTreeMap::new();
        for snapshot in state.roots.values() {
            for (path, file) in &snapshot.files {
                files.entry(path).or_default().push(file);
            }
        }
        for (path, file) in &state.unreached {
            files.entry(path).or_default().push(file);
        }
        Some(
            files
                .into_iter()
                .filter_map(|(path, programs)| {
                    let uri = self.client_uri(path)?;
                    let (version, current) = match self.document(&uri) {
                        Ok(doc) => (Some(doc.version), Some(text_hash(doc.text.as_bytes()))),
                        Err(_) => (None, None),
                    };
                    let mut items = Vec::new();
                    let mut any = false;
                    for file in programs {
                        if current.is_none_or(|stamp| stamp == file.stamp) {
                            any = true;
                            merge(&mut items, &file.items);
                        }
                    }
                    any.then(|| (uri.as_str().into(), version, items))
                })
                .collect(),
        )
    }
}
