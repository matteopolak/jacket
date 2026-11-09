//! Opening, changing and closing documents, and their diagnostics.
use super::{Document, Session};
use crate::analysis::Span;
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Error, RelatedInformation,
    TextChange, analysis::Analysis, position::LineIndex,
};
use std::cell::OnceCell;
use std::path::Path;
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
        if self.document_uri(&uri).is_some() {
            return Err(Error::AlreadyOpen);
        }
        if self.documents.len() + self.lint_configs.len() >= self.limits.documents {
            return Err(Error::Limit("open document count exceeded"));
        }
        self.admit(&uri, text.len(), 0)?;
        let unchanged = !crate::lints::is_config(&uri)
            && self.environment.as_ref().is_some_and(|env| {
                env.fs
                    .read(Path::new(uri.path()))
                    .is_some_and(|bytes| String::from_utf8_lossy(&bytes) == text)
            });
        if unchanged {
            self.touch();
        } else {
            self.buffer_changed(Path::new(uri.path()), true);
        }
        let map = if crate::lints::is_config(&uri) {
            let saved = self
                .config_on_disk(Path::new(uri.path()))
                .unwrap_or_else(|| text.clone());
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
            self.buffer_changed(Path::new(uri.path()), false);
        }
        let map = if config {
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        let current = map.get_mut(uri).expect("checked document");
        *current = Document {
            version,
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

    /// Commit an open settings buffer; ordinary source saves need no additional analysis.
    pub fn save(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        if let Some(config) = self.lint_configs.get(&key) {
            self.saved_configs.insert(key.clone(), config.text.clone());
            self.file_changed(Path::new(key.path()), false);
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
        }
        Ok(())
    }

    pub(crate) fn config_on_disk(&self, path: &Path) -> Option<String> {
        self.read_config_on_disk(path).ok()
    }

    /// [`Self::config_on_disk`], with the reason a file could not be read.
    pub(crate) fn read_config_on_disk(&self, path: &Path) -> std::io::Result<String> {
        match &self.environment {
            Some(env) => {
                let bytes = env
                    .fs
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
        if let Some(uri) = self.open_file_uri(path)
            && self.saved_configs.contains_key(&uri)
        {
            if let Some(text) = self.config_on_disk(path) {
                self.saved_configs.insert(uri, text);
            } else {
                self.saved_configs.remove(&uri);
            }
            self.semantic.borrow_mut().forget_lints();
        }
        self.file_changed(path, created_or_deleted);
    }

    pub fn close(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        let uri = &key;
        let unchanged = self.documents.get(uri).is_some_and(|doc| {
            self.environment.as_ref().is_some_and(|env| {
                env.fs
                    .read(Path::new(uri.path()))
                    .is_some_and(|bytes| String::from_utf8_lossy(&bytes) == doc.text)
            })
        });
        if unchanged {
            self.touch();
        } else {
            self.buffer_changed(Path::new(uri.path()), true);
        }
        if self.lint_configs.remove(uri).is_some() {
            self.saved_configs.remove(uri);
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
            return Ok(());
        }
        self.documents.remove(uri).ok_or(Error::MissingDocument)?;
        self.call_graphs.borrow_mut().remove(uri);
        self.rebuild();
        Ok(())
    }

    /// `publications`, unless the transport asked for the check to stop (see
    /// `jaic::interrupt::request`): then `None`, nothing is kept of the partial check, and the
    /// requests that came in are free to run before it starts again.
    pub fn publications_unless_interrupted(&self) -> Option<Vec<(String, i32, Vec<Diagnostic>)>> {
        jaic::interrupt::arm();
        let all = self.publications();
        (!jaic::interrupt::disarm()).then_some(all)
    }

    /// The diagnostics of every open document. One whose check did not finish is left out, so
    /// the client keeps what it shows until the next publication.
    pub fn publications(&self) -> Vec<(String, i32, Vec<Diagnostic>)> {
        let _background = self.background();
        self.documents
            .iter()
            .filter_map(|(uri, doc)| {
                let mut diagnostics = self
                    .analysis_for(uri)
                    .expect("opened document has analysis")
                    .diagnostics
                    .clone();
                diagnostics.extend(
                    self.try_check_findings(uri)
                        .ok()?
                        .into_iter()
                        .map(|f| f.diagnostic),
                );
                diagnostics.extend(self.unreachable_diagnostics(uri));
                diagnostics.extend(self.lint_diagnostics(uri));
                self.add_other_program_diagnostics(uri, &mut diagnostics)
                    .ok()?;
                Some((uri.as_str().into(), doc.version, diagnostics))
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
        self.try_check_findings(uri).unwrap_or_default()
    }

    /// `check_findings`, or `Error::CheckUnavailable` when the check was stopped, failed or
    /// deferred. A session with nothing to check against has no findings, not an error.
    pub(crate) fn try_check_findings(&self, uri: &DocumentUri) -> Result<Vec<Finding>, Error> {
        let Ok(doc) = self.document(uri) else {
            return Ok(Vec::new());
        };
        let found = match self.run_semantic(uri, &doc.text, |analysis, path| {
            let findings = analysis.check_findings(path);
            self.remember_call_graphs(uri, analysis);
            analysis.focus(path);
            Some(findings)
        }) {
            Ok(found) => found.unwrap_or_default(),
            Err(crate::session::compile::Unchecked::NoEnvironment) => Vec::new(),
            Err(_) => return Err(Error::CheckUnavailable),
        };
        let syntax = &self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics;
        Ok(found
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
                Some(self.check_finding_at(f, range))
            })
            .collect())
    }

    pub(super) fn check_finding_at(
        &self,
        f: crate::semantic::CheckFinding,
        range: crate::Range,
    ) -> Finding {
        let related = f
            .related
            .iter()
            .filter_map(|r| {
                let at = self.location_in(&r.path, &r.text, r.start, r.end)?;
                Some(RelatedInformation {
                    uri: at.uri,
                    range: at.range,
                    message: r.message.clone(),
                })
            })
            .collect();
        Finding {
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
        }
    }

    /// Hints that fade the code of `uri` the compile found can never run.
    pub(crate) fn unreachable_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let found = self
            .with_semantic(uri, &doc.text, |analysis, path| {
                Some(analysis.unreachable(path))
            })
            .unwrap_or_default();
        found
            .into_iter()
            .filter_map(|(start, end)| {
                Some(Diagnostic {
                    range: self.range_of(uri, Span::new(start, end))?,
                    severity: DiagnosticSeverity::Hint,
                    code: DiagnosticCode::Unreachable,
                    message: "unreachable code".into(),
                    related: Vec::new(),
                })
            })
            .collect()
    }

    /// Syntax and source diagnostics of `uri`, then its type error and lints.
    /// The diagnostics a pull answers with. `Error::CheckUnavailable` when the check did not
    /// finish (stopped, failed, or deferred): an empty list would wipe what the client shows.
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<Vec<Diagnostic>, Error> {
        if let Some(cached) = self.cached_workspace_diagnostics(uri) {
            return Ok(cached);
        }
        // A pull is a request for the check itself: it is the one request that compiles.
        let _background = self.background();
        self.document(uri)?;
        let mut diagnostics = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics
            .clone();
        diagnostics.extend(
            self.try_check_findings(uri)?
                .into_iter()
                .map(|f| f.diagnostic),
        );
        diagnostics.extend(self.unreachable_diagnostics(uri));
        diagnostics.extend(self.lint_diagnostics(uri));
        self.add_other_program_diagnostics(uri, &mut diagnostics)?;
        Ok(diagnostics)
    }

    /// Add to the diagnostics of the program that answers for `uri` what the other programs it
    /// is in find (what it has already is not added twice), or the note that it is in none.
    fn add_other_program_diagnostics(
        &self,
        uri: &DocumentUri,
        all: &mut Vec<Diagnostic>,
    ) -> Result<(), Error> {
        let others = self.sibling_program_diagnostics(uri)?;
        super::workspace_diagnostics::merge(all, &others);
        if self.is_unreached(uri) {
            all.push(super::workspace_diagnostics::not_in_program(
                &self.document(uri)?.text,
            ));
        }
        Ok(())
    }

    fn admit(&self, uri: &DocumentUri, bytes: usize, old: usize) -> Result<(), Error> {
        if bytes > self.limits.document_bytes {
            return Err(Error::Limit("document byte budget exceeded"));
        }
        let current = self
            .documents
            .iter()
            .chain(&self.lint_configs)
            .try_fold(0usize, |n, (name, doc)| {
                n.checked_add(name.as_str().len() + doc.text.len())
            })
            .ok_or(Error::Limit("workspace byte overflow"))?;
        let total = current
            .checked_sub(old)
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

    pub(crate) fn open_file_uri(&self, path: &Path) -> Option<DocumentUri> {
        if let Ok(uri) = DocumentUri::from_path(path)
            && (self.documents.contains_key(&uri) || self.lint_configs.contains_key(&uri))
        {
            return Some(uri);
        }
        let env = self.environment.as_ref()?;
        let want = crate::auto_import::canonical_key(env, path);
        let mut identities = self.open_file_identities.borrow_mut();
        let known = identities.get_or_insert_with(|| {
            self.documents
                .keys()
                .chain(self.lint_configs.keys())
                .map(|uri| {
                    (
                        crate::auto_import::canonical_key(env, Path::new(uri.path())),
                        uri.clone(),
                    )
                })
                .collect()
        });
        known
            .iter()
            .find(|(path, _)| *path == want)
            .map(|(_, uri)| uri.clone())
    }

    pub(crate) fn analysis_for(&self, uri: &DocumentUri) -> Option<&Rc<Analysis>> {
        self.analyses.get(&self.document_uri(uri)?)
    }

    pub(crate) fn parsed_for(&self, uri: &DocumentUri) -> Option<&Rc<Analysis>> {
        self.parsed.get(&self.document_uri(uri)?)
    }
}
