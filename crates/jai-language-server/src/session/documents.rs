//! Opening, changing and closing documents, and their diagnostics.
use super::saved_configs::{ConfigCapture, ConfigFailure, SavedConfigState};
use super::{Document, Session};
use crate::analysis::Span;
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Error, RelatedInformation,
    TextChange, analysis::Analysis, position::LineIndex,
};
use jaic::sema::FileSystem;
use std::cell::OnceCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A compile diagnostic with where it is (bytes) and what can fix it.
pub(crate) struct Finding {
    pub diagnostic: Diagnostic,
    pub start: usize,
    pub end: usize,
    pub suggestions: Vec<jaic::source::Suggestion>,
}

impl Session {
    pub fn open(&mut self, uri: DocumentUri, version: i32, text: String) -> Result<(), Error> {
        if self.documents.contains_key(&uri) || self.lint_configs.contains_key(&uri) {
            return Err(Error::AlreadyOpen);
        }
        if self.documents.len() + self.lint_configs.len() >= self.limits.documents {
            return Err(Error::Limit("open document count exceeded"));
        }
        self.admit(&uri, text.len(), 0)?;
        let config = crate::lints::is_config(&uri);
        let mut identity_pending = !config && !self.begin_cold_input_capture();
        if config {
            self.document_changed(Path::new(uri.path()), true);
        }
        if !identity_pending {
            let existing = self.document_uri(&uri);
            identity_pending = self
                .environment
                .as_ref()
                .is_some_and(|env| env.fs.input_epoch_valid() == Some(false));
            if !identity_pending && existing.is_some() {
                return Err(Error::AlreadyOpen);
            }
        }
        if !config {
            self.source_buffer_changed(Path::new(uri.path()));
        }
        let map = if crate::lints::is_config(&uri) {
            let saved = self.capture_saved_config(&uri, ConfigCapture::Initial(&text))?;
            self.admit_saved_config(&uri, &saved, uri.as_str().len() + text.len())?;
            self.saved_configs.insert(uri.clone(), saved);
            // Lints computed with the old settings are stale.
            self.semantic.borrow_mut().forget_lints();
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        map.insert(
            uri,
            Document {
                version,
                identity_pending,
                identity_recovery_token: None,
                index: LineIndex::new(&text),
                text,
                asm: OnceCell::new(),
                sites: OnceCell::new(),
                lexical_damage: Vec::new(),
            },
        );
        self.rebuild();
        Ok(())
    }

    pub fn change(
        &mut self,
        uri: &DocumentUri,
        version: i32,
        changes: &[TextChange],
    ) -> Result<(), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        let uri = &key;
        let document = match self.lint_configs.get(uri) {
            Some(config) => config,
            None => self.document(uri)?,
        };
        if version <= document.version {
            return Err(Error::StaleVersion);
        }
        if changes.len() > self.limits.edits {
            return Err(Error::Limit("incremental edit count exceeded"));
        }
        let old_bytes = document.text.len() + uri.as_str().len();
        let mut text = document.text.clone();
        let mut index = document.index.clone();
        for change in changes {
            let (start, end) = if let Some(range) = change.range {
                if range.start > range.end {
                    return Err(Error::InvalidEdit("edit range is reversed"));
                }
                let start = index.byte(&text, range.start);
                // `rangeLength` is deprecated and clients count it differently: the range decides.
                (start, index.byte(&text, range.end))
            } else {
                (0, text.len())
            };
            let bytes = text
                .len()
                .checked_sub(end - start)
                .and_then(|n| n.checked_add(change.text.len()))
                .ok_or(Error::Limit("edit size overflow"))?;
            self.admit(uri, bytes, old_bytes)?;
            text.replace_range(start..end, &change.text);
            index.update(&text, start);
        }
        let lexical_damage =
            crate::lexical_recovery::update(&document.text, &text, &document.lexical_damage);
        let config = self.lint_configs.contains_key(uri);
        if !config {
            self.buffer_changed(Path::new(uri.path()));
        }
        let map = if config {
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        let current = map.get_mut(uri).expect("checked document");
        let identity_pending = current.identity_pending;
        let identity_recovery_token = current.identity_recovery_token;
        *current = Document {
            version,
            identity_pending,
            identity_recovery_token,
            index,
            text,
            asm: OnceCell::new(),
            sites: OnceCell::new(),
            lexical_damage,
        };
        if !config {
            self.rebuild();
        }
        Ok(())
    }

    /// Commit settings edits, or invalidate closed native inputs after a source save.
    /// For example, the next diagnostics query may observe a changed closed dependency.
    pub fn save(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        if let Some(config) = self.lint_configs.get(&key) {
            let text = config.text.clone();
            self.document_changed(Path::new(key.path()), false);
            let required = std::cell::Cell::new(usize::MAX);
            let candidate = self.capture_saved_config(&key, ConfigCapture::Buffer(&text));
            let saved = match candidate.and_then(|saved| {
                required.set(saved.storage(&key));
                self.admit_saved_config(&key, &saved, 0)?;
                Ok(saved)
            }) {
                Ok(saved) => saved,
                Err(error) => {
                    let reason = if matches!(error, Error::Limit(_)) {
                        ConfigFailure::Budget {
                            required: required.get(),
                        }
                    } else {
                        ConfigFailure::Input
                    };
                    self.unavailable_saved_config(&key, reason);
                    return Err(error);
                }
            };
            self.saved_configs.insert(key.clone(), saved);
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
        } else {
            self.file_changed(Path::new(key.path()), false);
        }
        Ok(())
    }

    /// Read native configuration with the reason a file could not be read.
    pub(crate) fn read_config_on_disk(&self, path: &Path) -> std::io::Result<String> {
        match &self.environment {
            Some(_) => {
                let bytes = self
                    .config_filesystem()
                    .expect("environment")
                    .read(path)
                    .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;

                String::from_utf8(bytes).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "stream did not contain valid UTF-8",
                    )
                })
            }
            None => std::fs::read_to_string(path),
        }
    }

    /// Watch notifications describe disk state, never the unfinished editor buffer.
    pub(crate) fn saved_file_changed(&mut self, path: &Path, created_or_deleted: bool) {
        let raw = jaic::paths::identity_spelling(path);
        let mut affected: std::collections::BTreeSet<_> = self
            .saved_configs
            .iter()
            .filter(|(uri, saved)| Path::new(uri.path()) == path || saved.identity == raw)
            .map(|(uri, _)| uri.clone())
            .collect();
        self.native_file_changed(path, created_or_deleted);
        let key = self.environment.as_ref().map_or_else(
            || raw.clone(),
            |env| crate::auto_import::canonical_key(env, path),
        );
        for uri in self.lint_configs.keys() {
            let identity = self.environment.as_ref().map_or_else(
                || jaic::paths::identity_spelling(Path::new(uri.path())),
                |env| crate::auto_import::canonical_key(env, Path::new(uri.path())),
            );
            if identity == key || Path::new(uri.path()) == path {
                affected.insert(uri.clone());
            }
        }
        for uri in affected {
            let required = std::cell::Cell::new(usize::MAX);
            let candidate = self
                .capture_saved_config(&uri, ConfigCapture::Disk)
                .and_then(|mut saved| {
                    if matches!(saved.state, SavedConfigState::Missing)
                        && let Some(previous) = self.saved_configs.get(&uri)
                    {
                        saved.identity.clone_from(&previous.identity);
                    }
                    required.set(saved.storage(&uri));
                    self.admit_saved_config(&uri, &saved, 0)?;
                    Ok(saved)
                });
            match candidate {
                Ok(saved) => {
                    self.saved_configs.insert(uri, saved);
                }
                Err(error) => {
                    let reason = if matches!(error, Error::Limit(_)) {
                        ConfigFailure::Budget {
                            required: required.get(),
                        }
                    } else {
                        ConfigFailure::Input
                    };
                    self.unavailable_saved_config(&uri, reason);
                    if matches!(error, Error::Limit(_)) {
                        self.notify("jailsp could not retain saved settings: workspace byte budget exceeded".into());
                    }
                }
            }
            self.semantic.borrow_mut().forget_lints();
        }
    }

    pub fn close(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        let raw = self.documents.contains_key(uri) || self.lint_configs.contains_key(uri);
        if !raw && !self.begin_cold_input_capture() {
            return Err(Error::DiagnosticsUnavailable);
        }
        let key = if raw {
            uri.clone()
        } else {
            let key = self.document_uri(uri);
            if self
                .environment
                .as_ref()
                .is_some_and(|env| env.fs.input_epoch_valid() == Some(false))
            {
                return Err(Error::DiagnosticsUnavailable);
            }
            key.ok_or(Error::MissingDocument)?
        };
        let uri = &key;
        if self.lint_configs.contains_key(uri) {
            self.document_changed(Path::new(uri.path()), true);
        } else {
            // The raw client key already identifies its owned buffer. Closing it cannot
            // depend on whether unrelated settings or semantic input is available.
            self.source_buffer_changed(Path::new(uri.path()));
        }
        if self.lint_configs.remove(uri).is_some() {
            self.saved_configs.remove(uri);
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
            return Ok(());
        }
        self.documents.remove(uri).ok_or(Error::MissingDocument)?;
        self.call_graphs.borrow_mut().remove(uri);
        self.reference_records.borrow_mut().remove(uri);
        self.rebuild();
        Ok(())
    }

    /// Resolve pending raw registrations using sealed responses only.
    /// Missing operations queue ordinary hydration; conflicting aliases stay unavailable.
    pub(super) fn resolve_buffer_identities(&mut self) -> bool {
        let Some(epoch) = &self.input_epoch else {
            return false;
        };
        if !self.documents.values().any(|doc| doc.identity_pending) {
            return false;
        }
        let retry = self.documents.values().any(|doc| {
            doc.identity_pending && doc.identity_recovery_token != Some(epoch.capture_identity())
        });
        if !epoch.prepare_buffer_identity_hydration(retry) {
            return false;
        }
        for doc in self
            .documents
            .values_mut()
            .filter(|doc| doc.identity_pending)
        {
            doc.identity_recovery_token = Some(epoch.capture_identity());
        }
        let fs = &**epoch;
        let mut keys = std::collections::BTreeMap::<PathBuf, Vec<DocumentUri>>::new();
        let mut complete = true;
        for uri in self.documents.keys().chain(self.lint_configs.keys()) {
            let path = Path::new(uri.path());
            if fs.canonical_exact(path).is_none() && fs.missing_file_key(path).is_none() {
                complete = false;
            }
            keys.entry(crate::auto_import::filesystem_key(fs, path))
                .or_default()
                .push(uri.clone());
        }
        if !complete || !epoch.buffer_identity_inputs_ready() || jaic::interrupt::fired() {
            return false;
        }
        let mut changed = false;
        for uris in keys.values().filter(|uris| uris.len() == 1) {
            if let Some(doc) = self.documents.get_mut(&uris[0]) {
                changed |= std::mem::replace(&mut doc.identity_pending, false);
            }
        }
        if !self.documents.values().any(|doc| doc.identity_pending) {
            epoch.finish_buffer_identity_hydration();
        }
        changed
    }

    /// `publications`, unless the transport asked for the check to stop (see
    /// `jaic::interrupt::request`): then `None`, nothing is kept of the partial check, and the
    /// requests that came in are free to run before it starts again.
    pub fn publications_unless_interrupted(&self) -> Option<Vec<(String, i32, Vec<Diagnostic>)>> {
        jaic::interrupt::arm();
        let all = self.publications();
        (!jaic::interrupt::disarm()).then_some(all)
    }

    /// Ready diagnostic replacements. An unavailable document is omitted so clients keep
    /// its last accepted diagnostics until a new trigger can produce a valid replacement.
    pub fn publications(&self) -> Vec<(String, i32, Vec<Diagnostic>)> {
        let _query = self.discovery_query();
        let _background = self.background();
        self.documents
            .iter()
            .filter_map(|(uri, doc)| {
                self.diagnostics(uri)
                    .ok()
                    .map(|items| (uri.as_str().into(), doc.version, items))
            })
            .collect()
    }

    /// Collect a workspace pull without representing an unavailable document as clean.
    /// Unlike push, a failed batch returns an error instead of silently omitting a report.
    pub(crate) fn diagnostic_reports(&self) -> Result<Vec<(String, i32, Vec<Diagnostic>)>, Error> {
        let _query = self.discovery_query();
        let _background = self.background();
        self.documents
            .iter()
            .map(|(uri, doc)| {
                self.diagnostics(uri)
                    .map(|items| (uri.as_str().into(), doc.version, items))
            })
            .collect()
    }

    /// The type checker's errors and warnings, when the program `uri` belongs to fails to
    /// compile (or warns) and the diagnostic (or one of its notes) is in `uri`. Syntax errors
    /// already reported by the syntax layer are deduplicated; independent semantic errors
    /// remain visible. Notes with a place are related information; the rest, and the help,
    /// are appended to the message.
    pub(crate) fn check_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        self.check_findings(uri)
            .into_iter()
            .map(|f| f.diagnostic)
            .collect()
    }

    /// `check_diagnostics` with the byte range of each and what can fix it.
    pub(crate) fn check_findings(&self, uri: &DocumentUri) -> Vec<Finding> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        self.with_semantic(uri, &doc.text, |analysis, path| {
            let findings = analysis.staged_check_findings(path);
            let _ = self.remember_call_graphs(uri, analysis);
            analysis.focus(path);
            Some(self.findings_from(&self.config_filesystem()?, uri, findings))
        })
        .unwrap_or_default()
    }

    /// Project accepted checker rows into the document's protocol ranges and related locations.
    fn findings_from(
        &self,
        fs: &crate::semantic::ObservedFs<'_>,
        uri: &DocumentUri,
        found: Vec<crate::semantic::CheckFinding>,
    ) -> Vec<Finding> {
        let syntax = &self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics;
        found
            .into_iter()
            .filter_map(|f| {
                let range = self.range_of(uri, Span::new(f.start, f.end))?;
                if syntax.iter().any(|d| {
                    matches!(d.code, DiagnosticCode::Lexer | DiagnosticCode::Parser)
                        && d.range.start == range.start
                        && d.message == f.message
                }) {
                    return None;
                }
                let related = f
                    .related
                    .iter()
                    .filter_map(|r| {
                        let at = self.projected_location(fs, &r.path, &r.text, r.start, r.end)?;
                        Some(RelatedInformation {
                            uri: at.uri,
                            range: at.range,
                            message: r.message.clone(),
                        })
                    })
                    .collect();
                Some(Finding {
                    diagnostic: Diagnostic {
                        range,
                        severity: if f.warning {
                            DiagnosticSeverity::Warning
                        } else {
                            DiagnosticSeverity::Error
                        },
                        code: DiagnosticCode::Check,
                        message: f.display,
                        related,
                    },
                    start: f.start,
                    end: f.end,
                    suggestions: f.suggestions,
                })
            })
            .collect()
    }

    /// Syntax and source diagnostics of `uri`, then its type error and lints.
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<Vec<Diagnostic>, Error> {
        let _query = self.discovery_query();
        // A pull is a request for the check itself: it is the one request that compiles.
        let _background = self.background();
        let fs = self.config_filesystem();
        let (uri, _) = if let Some(fs) = &fs {
            self.projected_document_entry(fs, uri)?
        } else {
            self.document_entry(uri)?
        };
        let mut diagnostics = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics
            .clone();
        // No configured semantic provider is an explicitly syntax-only session.
        if self.environment.is_none() {
            return Ok(diagnostics);
        }
        let doc = self.document(uri)?;
        let config = if crate::semantic::parse_error(&doc.text).is_none() {
            self.lint_config(Path::new(uri.path()))
        } else {
            None
        };
        let (checks, unreachable, lints) = self
            .with_semantic(uri, &doc.text, |analysis, path| {
                let checks = analysis.staged_check_findings(path);
                let _ = self.remember_call_graphs(uri, analysis);
                analysis.focus(path);
                let unreachable = analysis.unreachable(path);
                let lints = match config.as_ref() {
                    Some(config) => analysis.lints(path, config)?,
                    None => Vec::new(),
                };
                let fs = self.config_filesystem()?;
                let checks = self.findings_from(&fs, uri, checks);
                let lints = self.projected_lint_diagnostics_from(&fs, uri, &lints);
                Some((checks, unreachable, lints))
            })
            .ok_or(Error::DiagnosticsUnavailable)?;
        diagnostics.extend(checks.into_iter().map(|f| f.diagnostic));
        diagnostics.extend(unreachable.into_iter().filter_map(|(start, end)| {
            Some(Diagnostic {
                range: self.range_of(uri, Span::new(start, end))?,
                severity: DiagnosticSeverity::Hint,
                code: DiagnosticCode::Unreachable,
                message: "unreachable code".into(),
                related: Vec::new(),
            })
        }));
        diagnostics.extend(lints);
        Ok(diagnostics)
    }

    fn admit(&self, uri: &DocumentUri, bytes: usize, old: usize) -> Result<(), Error> {
        if bytes > self.limits.document_bytes {
            return Err(Error::Limit("document byte budget exceeded"));
        }
        let current = self.buffer_storage()?;
        let total = current
            .checked_add(self.saved_config_storage()?)
            .and_then(|sum| sum.checked_sub(old))
            .and_then(|n| n.checked_add(uri.as_str().len() + bytes))
            .ok_or(Error::Limit("workspace byte overflow"))?;
        if total > self.limits.workspace_bytes {
            return Err(Error::Limit("workspace byte budget exceeded"));
        }
        Ok(())
    }

    pub(crate) fn document(&self, uri: &DocumentUri) -> Result<&Document, Error> {
        self.document_entry(uri).map(|(_, doc)| doc)
    }

    /// Resolve a request spelling to the actual stored key and its current buffer together.
    /// Only the filesystem can establish equivalence between distinct native paths.
    pub(crate) fn document_entry(
        &self,
        uri: &DocumentUri,
    ) -> Result<(&DocumentUri, &Document), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        self.documents
            .get_key_value(&key)
            .ok_or(Error::MissingDocument)
    }

    /// Select the stored client key under observation, then validate that mapping after use.
    /// Example: a physical request URI can address a buffer opened through a symlink.
    pub(crate) fn projected_document_query<T: Default>(
        &self,
        uri: &DocumentUri,
        query: impl FnOnce(&DocumentUri) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let _query = self.discovery_query();
        let mapping = std::cell::RefCell::new(crate::semantic::Observations::default());
        let base = self.config_filesystem();
        let fs = base.as_ref().map(|base| crate::semantic::ObservedFs {
            base,
            proof: Some(&mapping),
            secondary: None,
        });
        let (key, _) = match &fs {
            Some(fs) => self.projected_document_entry(fs, uri)?,
            None => self.document_entry(uri)?,
        };
        if jaic::interrupt::fired() {
            return Ok(T::default());
        }
        let answer = query(key)?;
        if jaic::interrupt::fired()
            || self
                .environment
                .as_ref()
                .is_some_and(|env| !mapping.borrow().matches(&*env.fs))
            || !self.accept_input_epoch()
            || jaic::interrupt::fired()
        {
            return Ok(T::default());
        }
        Ok(answer)
    }

    /// Resolve an exact request spelling inside its existing discovery transaction.
    /// Example: a physical URI may select the buffer opened through a symlink.
    pub(crate) fn projected_document_entry(
        &self,
        fs: &crate::semantic::ObservedFs<'_>,
        uri: &DocumentUri,
    ) -> Result<(&DocumentUri, &Document), Error> {
        let key = self
            .projected_uri(fs, Path::new(uri.path()))
            .ok_or(Error::MissingDocument)?;
        self.documents
            .get_key_value(&key)
            .ok_or(Error::MissingDocument)
    }

    pub(crate) fn document_uri(&self, uri: &DocumentUri) -> Option<DocumentUri> {
        if self.documents.contains_key(uri) || self.lint_configs.contains_key(uri) {
            return Some(uri.clone());
        }
        self.open_file_uri(Path::new(uri.path()))
    }

    /// The URI to report for `path`: an open document's own spelling when `path` is that file
    /// under another name (a symlinked folder such as macOS `/var` -> `/private/var`), so the
    /// client recognizes the document it opened rather than seeing a second, unopened one.
    pub(crate) fn client_uri(&self, path: &Path) -> Option<DocumentUri> {
        self.open_file_uri(path)
            .or_else(|| DocumentUri::from_path(path).ok())
    }

    /// Stage a client spelling through actual observed responses before its output guard.
    /// The caller must include this observer's history in the accepted result's final proof.
    pub(crate) fn projected_uri(
        &self,
        fs: &crate::semantic::ObservedFs<'_>,
        path: &Path,
    ) -> Option<DocumentUri> {
        if let Ok(uri) = DocumentUri::from_path(path)
            && (self.documents.contains_key(&uri) || self.lint_configs.contains_key(&uri))
        {
            return Some(uri);
        }
        let want = crate::auto_import::filesystem_key(fs, path);
        self.documents
            .keys()
            .chain(self.lint_configs.keys())
            .find(|uri| crate::auto_import::filesystem_key(fs, Path::new(uri.path())) == want)
            .cloned()
            .or_else(|| DocumentUri::from_path(path).ok())
    }

    pub(crate) fn open_file_uri(&self, path: &Path) -> Option<DocumentUri> {
        if let Ok(uri) = DocumentUri::from_path(path)
            && (self.documents.contains_key(&uri) || self.lint_configs.contains_key(&uri))
        {
            return Some(uri);
        }
        let env = self.environment.as_ref()?;
        let want = crate::auto_import::canonical_key(env, path);
        self.open_identities()?
            .iter()
            .find(|(path, _)| *path == want)
            .map(|(_, uri)| uri.clone())
    }

    /// Current canonical buffer identities, shared by navigation and snapshot validation.
    pub(crate) fn open_identities(&self) -> Option<std::cell::Ref<'_, [(PathBuf, DocumentUri)]>> {
        let env = self.environment.as_ref()?;
        if env.fs.tracks_native_inputs() {
            *self.open_file_identities.borrow_mut() = None;
        }
        if self.open_file_identities.borrow().is_none() {
            *self.open_file_identities.borrow_mut() = Some(
                self.documents
                    .keys()
                    .chain(self.lint_configs.keys())
                    .map(|uri| {
                        (
                            crate::auto_import::canonical_key(env, Path::new(uri.path())),
                            uri.clone(),
                        )
                    })
                    .collect(),
            );
        }
        Some(std::cell::Ref::map(
            self.open_file_identities.borrow(),
            |identities| identities.as_ref().unwrap().as_slice(),
        ))
    }

    pub(crate) fn analysis_for(&self, uri: &DocumentUri) -> Option<&Rc<Analysis>> {
        self.analyses.get(&self.document_uri(uri)?)
    }

    pub(crate) fn parsed_for(&self, uri: &DocumentUri) -> Option<&Rc<Analysis>> {
        self.parsed.get(&self.document_uri(uri)?)
    }
}
