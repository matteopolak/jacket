//! Opening, changing and closing documents, and their diagnostics.
use super::{Document, Session};
use crate::analysis::Span;
use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DocumentUri, Error, TextChange,
    analysis::Analysis, position::LineIndex,
};
use std::cell::OnceCell;
use std::path::Path;
use std::rc::Rc;

impl Session {
    pub fn open(&mut self, uri: DocumentUri, version: i32, text: String) -> Result<(), Error> {
        if self.document_uri(&uri).is_some() {
            return Err(Error::AlreadyOpen);
        }
        if self.documents.len() + self.lint_configs.len() >= self.limits.documents {
            return Err(Error::Limit("open document count exceeded"));
        }
        self.admit(&uri, text.len(), 0)?;
        self.file_changed(Path::new(uri.path()), true);
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
        for change in changes {
            let index = LineIndex::new(&text);
            let (start, end) = if let Some(range) = change.range {
                if range.start > range.end {
                    return Err(Error::InvalidEdit("edit range is reversed"));
                }
                let start = index.byte(&text, range.start)?;
                let end = index.byte(&text, range.end)?;
                if let Some(length) = change.range_length
                    && jaic::source::utf16_len(&text[start..end]) != length
                {
                    return Err(Error::InvalidEdit(
                        "rangeLength disagrees with UTF-16 range",
                    ));
                }
                (start, end)
            } else {
                if change.range_length.is_some() {
                    return Err(Error::InvalidEdit(
                        "full replacement cannot have rangeLength",
                    ));
                }
                (0, text.len())
            };
            let bytes = text
                .len()
                .checked_sub(end - start)
                .and_then(|n| n.checked_add(change.text.len()))
                .ok_or(Error::Limit("edit size overflow"))?;
            self.admit(uri, bytes, old_bytes)?;
            text.replace_range(start..end, &change.text);
        }
        let lexical_damage =
            crate::lexical_recovery::update(&document.text, &text, &document.lexical_damage);
        let config = self.lint_configs.contains_key(uri);
        if !config {
            self.file_changed(Path::new(uri.path()), false);
        }
        let map = if config {
            &mut self.lint_configs
        } else {
            &mut self.documents
        };
        let current = map.get_mut(uri).expect("checked document");
        *current = Document {
            version,
            index: LineIndex::new(&text),
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
        match &self.environment {
            Some(env) => env.fs.read(path).and_then(|b| String::from_utf8(b).ok()),
            None => std::fs::read_to_string(path).ok(),
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
        self.file_changed(Path::new(uri.path()), true);
        if self.lint_configs.remove(uri).is_some() {
            self.saved_configs.remove(uri);
            self.semantic.borrow_mut().forget_lints();
            self.rebuild();
            return Ok(());
        }
        self.documents.remove(uri).ok_or(Error::MissingDocument)?;
        self.rebuild();
        Ok(())
    }

    pub fn publications(&self) -> Vec<(String, i32, Vec<Diagnostic>)> {
        self.documents
            .iter()
            .map(|(uri, doc)| {
                let mut diagnostics = self
                    .analysis_for(uri)
                    .expect("opened document has analysis")
                    .diagnostics
                    .clone();
                diagnostics.extend(self.check_diagnostics(uri));
                diagnostics.extend(self.unreachable_diagnostics(uri));
                diagnostics.extend(self.lint_diagnostics(uri));
                (uri.as_str().into(), doc.version, diagnostics)
            })
            .collect()
    }

    /// The type checker's errors, when the program `uri` belongs to fails to compile and the
    /// error (or one of its notes) is in `uri`. Syntax errors already reported by the
    /// syntax layer are deduplicated; independent semantic errors remain visible.
    pub(crate) fn check_diagnostics(&self, uri: &DocumentUri) -> Vec<Diagnostic> {
        let Ok(doc) = self.document(uri) else {
            return Vec::new();
        };
        let (errors, warnings) = self
            .with_semantic(uri, &doc.text, |analysis, path| {
                Some((analysis.check_errors(path), analysis.check_warnings(path)))
            })
            .unwrap_or_default();
        let tagged = |found: Vec<(usize, usize, String)>, severity| {
            found
                .into_iter()
                .map(move |(start, end, message)| (start, end, message, severity))
                .collect::<Vec<_>>()
        };
        let mut all = tagged(errors, DiagnosticSeverity::Error);
        all.extend(tagged(warnings, DiagnosticSeverity::Warning));
        all.into_iter()
            .filter_map(|(start, end, message, severity)| {
                let range = self.range_of(uri, Span::new(start, end))?;
                if self
                    .analysis_for(uri)
                    .expect("opened document has analysis")
                    .diagnostics
                    .iter()
                    .any(|d| {
                        matches!(d.code, DiagnosticCode::Lexer | DiagnosticCode::Parser)
                            && d.range.start == range.start
                            && d.message == message
                    })
                {
                    return None;
                }
                Some(Diagnostic {
                    range,
                    severity,
                    code: DiagnosticCode::Check,
                    message,
                })
            })
            .collect()
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
                })
            })
            .collect()
    }

    /// Syntax and source diagnostics of `uri`, then its type error and lints.
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<Vec<Diagnostic>, Error> {
        self.document(uri)?;
        let mut diagnostics = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics
            .clone();
        diagnostics.extend(self.check_diagnostics(uri));
        diagnostics.extend(self.unreachable_diagnostics(uri));
        diagnostics.extend(self.lint_diagnostics(uri));
        Ok(diagnostics)
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
