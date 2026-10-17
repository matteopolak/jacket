use crate::analysis::{DIRECTIVES, KEYWORDS, Span, Token, TokenKind};
use crate::hover::{Block, HoverText, Inline};
use crate::semantic::{self, Environment};
use crate::{
    CompletionData, CompletionItem, CompletionKind, CompletionList, Diagnostic, DiagnosticCode,
    DiagnosticSeverity, DocumentSymbol, DocumentUri, Error, Hover, Limits, Location, MarkupContent,
    MarkupKind, Position, SemanticToken, SemanticTokenKind, SymbolKind, TextChange, VirtualSources,
    analysis::{Analysis, SymbolRow},
    position::LineIndex,
};
use jaic::intern::Sym;
use jaic::sema::ide::{IdeKind, IdeLayout, IdeName};
use jaic::sema::ide_meta::IdeClass;
use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) struct Document {
    pub(crate) version: i32,
    pub(crate) text: String,
    pub(crate) index: LineIndex,
    /// Whether `text` parses, worked out on first use.
    pub(crate) parses: OnceCell<bool>,
    /// Whether `text` has an `#asm` block anywhere, worked out on first use.
    pub(crate) asm: OnceCell<bool>,
    /// The links in `text`'s doc comments, found on first use.
    pub(crate) sites: OnceCell<Vec<jaic::docs::Site>>,
    pub(crate) lexical_damage: Vec<Span>,
}

impl Document {
    pub(crate) fn ide_text(&self) -> Cow<'_, str> {
        crate::lexical_recovery::mask(&self.text, &self.lexical_damage)
            .map_or_else(|| Cow::Borrowed(self.text.as_str()), Cow::Owned)
    }

    pub(crate) fn same_positions(&self, text: &str) -> bool {
        self.text == text || (!self.lexical_damage.is_empty() && self.ide_text().as_ref() == text)
    }

    pub(crate) fn has_asm(&self) -> bool {
        *self.asm.get_or_init(|| self.text.contains("#asm"))
    }

    pub(crate) fn sites(&self) -> &[jaic::docs::Site] {
        self.sites.get_or_init(|| jaic::docs::sites(&self.text))
    }
}

/// The tokens whose span holds `byte` (touching it at either end), in order, found by binary
/// search: `tokens` are in source order.
pub(crate) fn tokens_at(tokens: &[Token], byte: usize) -> impl Iterator<Item = (usize, &Token)> {
    let first = tokens.partition_point(|t| t.span.end < byte);
    tokens[first..]
        .iter()
        .enumerate()
        .map(move |(i, t)| (first + i, t))
        .take_while(move |(_, t)| t.span.start <= byte)
        .filter(move |(_, t)| byte <= t.span.end)
}

/// Byte ranges each other target skipped, by file and target.
type Unreached = BTreeMap<(PathBuf, u8), Vec<(usize, usize)>>;

pub struct Session {
    pub(crate) limits: Limits,
    pub(crate) documents: BTreeMap<DocumentUri, Document>,
    pub(crate) analyses: BTreeMap<DocumentUri, Rc<Analysis>>,
    /// Last analysis of each document that parsed: completion keeps offering its names while
    /// the text being typed does not parse.
    pub(crate) parsed: BTreeMap<DocumentUri, Rc<Analysis>>,
    /// Type-checked answers (absent: syntax only).
    pub(crate) environment: Option<Environment>,
    pub(crate) semantic: RefCell<semantic::Cache>,
    /// `Cache::recorded_version` as of the last time discovery was given the recorded builds.
    synced_builds: std::cell::Cell<u64>,
    /// Open `jailint.toml` files: settings for the lints of documents under their directory.
    /// They are not Jai documents, so they get no analysis or diagnostics of their own.
    pub(crate) lint_configs: BTreeMap<DocumentUri, Document>,
    /// Auto-import completion is on (`jai.completion.autoImport`).
    pub(crate) auto_import: bool,
    /// The client's workspace folders: projects without a `jai.toml` are inferred in them.
    pub(crate) workspace_folders: Vec<PathBuf>,
    /// What modules and project files declare, for auto-import completion.
    pub(crate) index: RefCell<crate::auto_import::Index>,
    /// Bumped by anything that can change a compile's inputs: an open document, a file on
    /// disk, the workspace folders.
    generation: std::cell::Cell<u64>,
    /// The last `compile_inputs` answer, with the generation, document and text it was for:
    /// working them out reads folders and copies every open document, which a burst of
    /// requests on one version would otherwise repeat.
    /// The text is `None` when it was the document's own.
    inputs: RefCell<Option<InputsMemo>>,
    /// Filesystem-proven identities of open documents, invalidated with every input change.
    open_file_identities: RefCell<Option<Vec<(PathBuf, DocumentUri)>>>,
    /// The code each other target's compile skipped too (by file and target, byte ranges), for
    /// the generation it was found in: not compiled for again. Code no target checks (a file
    /// nothing loads, a branch on yet another CPU) would otherwise compile every target on
    /// every hover, more of them than the cache keeps.
    unreached: RefCell<(u64, Unreached)>,
}

/// The open files, the root file and the module folders of a compile.
type CompileInputs = (semantic::Buffers, PathBuf, Vec<PathBuf>);

/// `compile_inputs`' last answer: generation, document, its text (`None` for the document's
/// own) and the inputs.
type InputsMemo = (u64, DocumentUri, Option<String>, CompileInputs);

/// What completion asks the type checker: the word being typed, the names it could be, and the
/// path of each file they are declared in (by file number).
type Completions = (String, Vec<IdeName>, BTreeMap<u32, String>);

impl Session {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            documents: BTreeMap::new(),
            analyses: BTreeMap::new(),
            parsed: BTreeMap::new(),
            environment: None,
            semantic: RefCell::default(),
            synced_builds: std::cell::Cell::new(0),
            lint_configs: BTreeMap::new(),
            auto_import: true,
            workspace_folders: Vec::new(),
            index: RefCell::default(),
            generation: std::cell::Cell::new(0),
            inputs: RefCell::default(),
            open_file_identities: RefCell::default(),
            unreached: RefCell::default(),
        }
    }

    /// Turn auto-import completion on or off.
    pub fn set_auto_import(&mut self, on: bool) {
        self.auto_import = on;
    }

    /// The client's workspace folders (absolute paths).
    /// Messages for the user the server has found since the last call (a settings file that
    /// does not parse), each to be sent once.
    pub fn take_messages(&self) -> Vec<String> {
        self.index.borrow_mut().take_messages()
    }

    /// How many programs the type checker has compiled for this session, and how many of them
    /// are kept (for tests and the benchmark: an edit that needs no compile leaves the first
    /// number alone, and the second stays bounded).
    pub fn compile_counts(&self) -> (usize, usize) {
        self.semantic.borrow().counts()
    }

    /// Coverage for the current open buffer and its real analysis context. A lack
    /// of file-local diagnostics is not proof that the compiler reached the file.
    pub fn semantic_coverage(
        &self,
        uri: &DocumentUri,
    ) -> Result<Option<semantic::SemanticCoverage>, Error> {
        let doc = self.document(uri)?;
        let Some(probe) = self.probe(uri, &doc.text, None) else {
            return Ok(None);
        };
        let base = self.with_semantic(uri, &probe, |analysis, path| Some(analysis.coverage(path)));
        if base.as_ref().is_some_and(|coverage| coverage.source_loaded) {
            return Ok(base);
        }
        Ok(self
            .with_semantic_at(uri, &probe, 0, |analysis, path| {
                let coverage = analysis.coverage(path);
                coverage.source_loaded.then_some(coverage)
            })
            .or(base))
    }

    /// Queue a message for the user (`window/showMessage`), sent once.
    pub fn notify(&self, message: String) {
        self.index.borrow_mut().notify(message);
    }

    /// Drop everything cached (compiles, scans) after a failure: it is rebuilt on demand.
    pub fn reset_caches(&self) {
        // What the metaprograms did is not lost with the compiles that recorded it.
        let trusted = self.semantic.borrow().trusted();
        let recorded = std::mem::take(&mut self.semantic.borrow_mut().recorded);
        let version = self.semantic.borrow().recorded_version;
        *self.semantic.borrow_mut() = Default::default();
        {
            let mut cache = self.semantic.borrow_mut();
            cache.set_trusted(trusted);
            cache.recorded = recorded;
            cache.recorded_version = version + 1;
        }
        *self.index.borrow_mut() = Default::default();
        self.synced_builds.set(0);
        self.touch();
    }

    /// Allow or withhold running the workspace's compile-time code (`#run`, build
    /// metaprograms, computed `#insert`). Trusted by default. A change drops the compiles made
    /// under the other setting; the caller republishes diagnostics.
    pub fn set_trusted(&mut self, trusted: bool) {
        if self.semantic.borrow().trusted() != trusted {
            self.semantic.borrow_mut().set_trusted(trusted);
            // Without trust nothing runs, so discovery falls back to the static scan; what
            // earlier runs recorded goes with the cache.
            let mut index = self.index.borrow_mut();
            index.builds.clear();
            index.probed.clear();
            index.import_dirs.clear();
            self.synced_builds.set(0);
            self.touch();
        }
    }

    pub fn trusted(&self) -> bool {
        self.semantic.borrow().trusted()
    }

    /// The inputs of a compile may have changed.
    fn touch(&self) {
        self.generation.set(self.generation.get() + 1);
        *self.inputs.borrow_mut() = None;
        *self.open_file_identities.borrow_mut() = None;
    }

    pub fn set_workspace_folders(&mut self, folders: Vec<PathBuf>) {
        self.workspace_folders = folders
            .iter()
            .map(|p| crate::document::path_key(p))
            .collect();
        self.index.borrow_mut().import_dirs.clear();
        self.touch();
    }

    /// A file changed on disk (`created_or_deleted` when it appeared or went away): what was
    /// read from it is read again.
    pub fn file_changed(&mut self, path: &Path, created_or_deleted: bool) {
        self.touch();
        if created_or_deleted {
            self.semantic.borrow_mut().forget_paths();
        }
        let path = self.environment.as_ref().map_or_else(
            || crate::document::path_key(path),
            |env| crate::auto_import::canonical_key(env, path),
        );
        let path = path.as_path();
        let mut index = self.index.borrow_mut();
        if created_or_deleted {
            index.created_or_deleted(path);
        } else {
            index.changed(path);
        }
    }

    /// A session that also type-checks the open documents against `environment`'s modules.
    pub fn with_environment(limits: Limits, environment: Environment) -> Self {
        Self {
            environment: Some(environment),
            ..Self::new(limits)
        }
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    pub fn source_snapshot(&self) -> VirtualSources {
        VirtualSources {
            files: self
                .documents
                .iter()
                .map(|(uri, doc)| (uri.path().to_owned(), doc.text.clone()))
                .collect(),
        }
    }

    pub fn document_text(&self, uri: &DocumentUri) -> Result<&str, Error> {
        Ok(self.document(uri)?.text.as_str())
    }

    pub fn version(&self, uri: &DocumentUri) -> Result<i32, Error> {
        Ok(self.document(uri)?.version)
    }

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
                parses: OnceCell::new(),
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
        let mut text = document.text.to_owned();
        for change in changes {
            let index = LineIndex::new(&text);
            let (start, end) = if let Some(range) = change.range {
                if range.start > range.end {
                    return Err(Error::InvalidEdit("edit range is reversed"));
                }
                let start = index.byte(&text, range.start)?;
                let end = index.byte(&text, range.end)?;
                if let Some(length) = change.range_length
                    && text[start..end].encode_utf16().count() != length as usize
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
        self.file_changed(Path::new(uri.path()), false);
        let config = self.lint_configs.contains_key(uri);
        if config {
            self.semantic.borrow_mut().forget_lints();
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
            parses: OnceCell::new(),
            asm: OnceCell::new(),
            sites: OnceCell::new(),
            lexical_damage,
        };
        self.rebuild();
        Ok(())
    }

    pub fn close(&mut self, uri: &DocumentUri) -> Result<(), Error> {
        let key = self.document_uri(uri).ok_or(Error::MissingDocument)?;
        let uri = &key;
        self.file_changed(Path::new(uri.path()), true);
        if self.lint_configs.remove(uri).is_some() {
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

    /// Syntax and source diagnostics of `uri`, then its type error and lints.
    pub fn diagnostics(&self, uri: &DocumentUri) -> Result<Vec<Diagnostic>, Error> {
        self.document(uri)?;
        let mut diagnostics = self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .diagnostics
            .clone();
        diagnostics.extend(self.check_diagnostics(uri));
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

    fn rebuild(&mut self) {
        // Opening, closing or editing one document leaves the others as they were: an
        // analysis of the same text is kept. One with `#load`s is built again, because the
        // warnings about files that are not open depend on which documents are.
        let mut analyses: BTreeMap<DocumentUri, Rc<Analysis>> = BTreeMap::new();
        for (uri, document) in &self.documents {
            let kept = self.analysis_for(uri).filter(|a| {
                a.loads.is_empty() && a.key == crate::semantic::text_key(&document.text)
            });
            if let Some(kept) = kept {
                analyses.insert(uri.clone(), kept.clone());
                continue;
            }
            let mut analysis = Analysis::build(&document.ide_text(), uri, self.limits);
            analysis.key = crate::semantic::text_key(&document.text);
            for span in &document.lexical_damage {
                let end = document.text[span.start..span.end]
                    .find('\n')
                    .map_or(span.end, |n| span.start + n);
                analysis.diagnostic(&document.index, &document.text, Span::new(span.start, end), DiagnosticSeverity::Error, DiagnosticCode::Lexer,
                    "Incomplete lexical edit; this literal or comment is omitted from IDE analysis.");
            }
            for (target, span) in analysis.loads.clone() {
                // A file the environment can read is loaded by the compile; only when nothing
                // can read it (no open document, no file system) does the syntax-only view warn.
                if self.document(&target).is_err() && self.source(&target).is_none() {
                    analysis.diagnostic(
                        &document.index,
                        &document.text,
                        span,
                        DiagnosticSeverity::Warning,
                        DiagnosticCode::Source,
                        "The #load file is not open in this session; \
                         no filesystem fallback is permitted.",
                    );
                }
            }
            analyses.insert(uri.clone(), Rc::new(analysis));
        }
        for (uri, analysis) in &analyses {
            if analysis.complete {
                self.parsed.insert(uri.clone(), analysis.clone());
            }
        }
        self.parsed
            .retain(|uri, _| self.documents.contains_key(uri));
        self.analyses = analyses;
    }

    /// The document a check of `uri` starts from: an open document that `#load`s it (directly or
    /// not) and is loaded by none, else `uri` itself.
    pub(crate) fn root(&self, uri: &DocumentUri) -> DocumentUri {
        let actual = self.document_uri(uri).unwrap_or_else(|| uri.clone());
        let uri = &actual;
        let loaded: BTreeSet<DocumentUri> = self
            .analyses
            .values()
            .flat_map(|a| {
                a.loads
                    .iter()
                    .map(|(target, _)| self.document_uri(target).unwrap_or_else(|| target.clone()))
            })
            .collect();
        if !loaded.contains(uri) {
            return uri.clone();
        }
        self.documents
            .keys()
            .find(|d| !loaded.contains(*d) && self.reachable(d).contains(&uri))
            .unwrap_or(uri)
            .clone()
    }

    /// What a compile of the program containing `uri` needs, with `uri`'s text replaced by
    /// `text`: the open files, the program's root file and the module folders searched.
    fn compile_inputs(&self, uri: &DocumentUri, text: &str) -> Option<CompileInputs> {
        let key = self.document_uri(uri)?;
        let uri = &key;
        // The document's own text (as most requests pass it) needs no comparing: any change to
        // a document moves the generation on.
        let own = self.is_own_text(uri, text);
        if let Some((generation, last, last_text, inputs)) = &*self.inputs.borrow()
            && *generation == self.generation.get()
            && last == uri
            && match last_text {
                None => own,
                Some(last_text) => last_text == text,
            }
        {
            return Some(inputs.clone());
        }
        let inputs = self.work_out_inputs(uri, text)?;
        *self.inputs.borrow_mut() = Some((
            self.generation.get(),
            uri.clone(),
            (!own).then(|| text.to_string()),
            inputs.clone(),
        ));
        Some(inputs)
    }

    /// Whether `text` is the open document's own buffer, not a copy or an edited probe.
    fn is_own_text(&self, uri: &DocumentUri, text: &str) -> bool {
        self.document(uri)
            .ok()
            .is_some_and(|d| std::ptr::eq(d.text.as_str(), text))
    }

    /// `repair` for text of `uri`, which for the document's own text that parses is that text
    /// itself, known without parsing it again.
    pub(crate) fn probe<'a>(
        &self,
        uri: &DocumentUri,
        text: &'a str,
        first: Option<usize>,
    ) -> Option<Cow<'a, str>> {
        if let Ok(doc) = self.document(uri)
            && std::ptr::eq(doc.text.as_str(), text)
            && *doc
                .parses
                .get_or_init(|| semantic::parse_error(text).is_none())
        {
            return Some(Cow::Borrowed(text));
        }
        repair(text, first).map(Cow::Owned)
    }

    fn work_out_inputs(&self, uri: &DocumentUri, text: &str) -> Option<CompileInputs> {
        let environment = self.environment.as_ref()?;
        let files: BTreeMap<PathBuf, Rc<[u8]>> = self
            .documents
            .iter()
            .map(|(u, d)| {
                let view = if u == uri {
                    let damage = if text == d.text {
                        d.lexical_damage.clone()
                    } else {
                        crate::lexical_recovery::update(&d.text, text, &d.lexical_damage)
                    };
                    crate::lexical_recovery::mask(text, &damage)
                        .map_or_else(|| Cow::Borrowed(text), Cow::Owned)
                } else {
                    d.ide_text()
                };
                (PathBuf::from(u.path()), Rc::from(view.as_bytes()))
            })
            .collect();
        // The open document no other open document loads, compiled from the project's entry
        // file (which may load it, or build it from a metaprogram), so the files of the program
        // that are not open are checked too.
        let open_root = self.root(uri);
        self.run_unrecorded_builds(&open_root, &files);
        let root = match self.project_root(&open_root) {
            Some(entry) if environment.fs.is_file(&entry) => entry,
            _ => PathBuf::from(open_root.path()),
        };
        // Untrusted: the metaprogram that would build the open file does not run, so the file
        // is its own program.
        let root = if !self.trusted() && self.is_build_metaprogram(&root) {
            PathBuf::from(open_root.path())
        } else {
            root
        };
        let dirs = self.import_dirs(&root);
        Some((Rc::new(files), root, dirs))
    }

    /// Run `query` on the type-checked program containing `uri`, its text replaced by `text`.
    pub(crate) fn with_semantic<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        let analysis = cache.analyze(environment, &root, &dirs, files);
        let path = Path::new(uri.path());
        analysis.focus(path);
        let answer = query(analysis, path);
        self.sync_builds(&cache);
        answer
    }

    /// Runs the `jai.toml` build files that could build `uri` but have not been run (or were
    /// changed since), once each, so their recorded facts decide which program `uri` is in
    /// instead of a scan of their text. A run that fails is not repeated until the
    /// metaprogram, what it loads or `jai.toml` changes.
    fn run_unrecorded_builds(&self, uri: &DocumentUri, files: &BTreeMap<PathBuf, Rc<[u8]>>) {
        let Some(environment) = self.environment.as_ref() else {
            return;
        };
        if !self.trusted() {
            return;
        }
        let pending = self.unrecorded_builds(uri);
        if pending.is_empty() {
            return;
        }
        let buffers: semantic::Buffers = Rc::new(files.clone());
        for (entry, hash) in pending {
            self.index.borrow_mut().probed.insert(entry.clone(), hash);
            let dirs = self.import_dirs(&entry);
            let mut cache = self.semantic.borrow_mut();
            cache.analyze(environment, &entry, &dirs, buffers.clone());
            self.sync_builds(&cache);
        }
    }

    /// Hands what the build metaprograms did (`Cache::recorded`) to discovery, which then finds
    /// the files and module folders they really added; what was worked out before is dropped.
    pub(crate) fn sync_builds(&self, cache: &semantic::Cache) {
        if cache.recorded_version == self.synced_builds.get() {
            return;
        }
        self.synced_builds.set(cache.recorded_version);
        let mut index = self.index.borrow_mut();
        index.builds = cache.recorded.clone();
        index.import_dirs.clear();
        *self.inputs.borrow_mut() = None;
    }

    /// Like `with_semantic` for a question about `byte` of `uri`. When the program's compile
    /// skipped that code (an `#if OS == ...` branch, or a file only another target loads) and
    /// has no answer, compiles for the other targets are asked, the first that checked it
    /// answering.
    pub(crate) fn with_semantic_at<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        query: impl Fn(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let path = Path::new(uri.path());
        let mut cache = self.semantic.borrow_mut();
        let analysis = cache.analyze(environment, &root, &dirs, files.clone());
        analysis.focus(path);
        let first = query(analysis, path);
        self.sync_builds(&cache);
        if let Some(found) = first {
            return Some(found);
        }
        let analysis = cache.analyze(environment, &root, &dirs, files.clone());
        if !analysis.skipped(path, byte) {
            return None;
        }
        let own = (environment.options)(&root).os;
        let mut unreached = self.unreached.borrow_mut();
        if unreached.0 != self.generation.get() {
            *unreached = (self.generation.get(), Default::default());
        }
        // The target that answered last, then those compiled already, then the rest.
        let last = cache.answered.get(path).copied();
        let mut order: Vec<jaic::sema::TargetOs> = semantic::OTHER_TARGETS
            .into_iter()
            .filter(|&os| {
                os != own
                    && !unreached
                        .1
                        .get(&(path.to_path_buf(), os as u8))
                        .is_some_and(|spans| spans.iter().any(|&(a, b)| a <= byte && byte <= b))
            })
            .collect();
        order.sort_by_key(|&os| (Some(os) != last, !cache.has(&root, &dirs, &files, Some(os))));
        for os in order {
            let analysis = cache.analyze_for(environment, &root, &dirs, files.clone(), Some(os));
            analysis.focus(path);
            match analysis.skipped_span(path, byte) {
                Some(span) => unreached
                    .1
                    .entry((path.to_path_buf(), os as u8))
                    .or_default()
                    .push(span),
                None => {
                    let found = query(analysis, path);
                    cache.answered.insert(path.to_path_buf(), os);
                    return found;
                }
            }
        }
        None
    }

    /// Like `with_semantic`, but on a compile that already exists and knows the first `at` bytes
    /// of `uri` as they are in `text` (the other open files unchanged), so it never compiles.
    /// `None` when no compile fits.
    pub(crate) fn with_compiled_before<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        at: usize,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        let path = Path::new(uri.path());
        let analysis = cache.compiled_before(
            environment,
            &root,
            &dirs,
            files,
            path,
            &text.as_bytes()[..at],
        )?;
        query(analysis, path)
    }

    pub(crate) fn source_detail<'a>(doc: &'a Document, row: &SymbolRow) -> &'a str {
        let text = &doc.text;
        let span = row.location;
        // Rows of an older parse may point past the current text.
        let start = text.floor_char_boundary(span.start.min(text.len()));
        let end =
            text.floor_char_boundary(span.end.min(span.start.saturating_add(256)).min(text.len()));
        text[start..end.max(start)].trim()
    }

    pub fn document_symbols(&self, uri: &DocumentUri) -> Result<Vec<DocumentSymbol>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let rows = &self
            .analysis_for(uri)
            .expect("opened document has analysis")
            .rows;
        // Children by parent, so the tree is built in one pass over the rows.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
        for (id, row) in rows.iter().enumerate() {
            if let Some(parent) = row.parent {
                children[parent].push(id);
            }
        }
        fn tree(
            id: usize,
            rows: &[SymbolRow],
            children: &[Vec<usize>],
            doc: &Document,
        ) -> Result<DocumentSymbol, Error> {
            let row = &rows[id];
            Ok(DocumentSymbol {
                name: row.name.as_str().into(),
                detail: Session::source_detail(doc, row).into(),
                kind: row.kind,
                range: doc.index.range(&doc.text, row.location)?,
                selection_range: doc.index.range(&doc.text, row.selection)?,
                children: children[id]
                    .iter()
                    .map(|&child| tree(child, rows, children, doc))
                    .collect::<Result<_, _>>()?,
            })
        }
        rows.iter()
            .enumerate()
            .filter(|(_, row)| row.parent.is_none())
            .map(|(id, _)| tree(id, rows, &children, doc))
            .collect()
    }

    pub(crate) fn reachable(&self, uri: &DocumentUri) -> Vec<&DocumentUri> {
        let mut pending = vec![uri.clone()];
        let mut seen = BTreeSet::new();
        let mut out = vec![];
        while let Some(next) = pending.pop() {
            let next = self.document_uri(&next).unwrap_or(next);
            if !seen.insert(next.clone()) {
                continue;
            }
            if let Some((key, analysis)) = self.analyses.get_key_value(&next) {
                out.push(key);
                pending.extend(analysis.loads.iter().map(|(uri, _)| uri.clone()));
            }
        }
        out
    }

    pub(crate) fn word(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Option<(usize, Token)>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        Ok(tokens_at(
            &self
                .analysis_for(uri)
                .expect("opened document has analysis")
                .tokens,
            byte,
        )
        .find(|(_, token)| matches!(token.kind, TokenKind::Ident | TokenKind::Keyword))
        .map(|(at, token)| (at, *token)))
    }

    fn bindings(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<(&DocumentUri, &SymbolRow)>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let Some((at, token)) = self.word(uri, position)? else {
            return Ok(vec![]);
        };
        let analysis = &self
            .analysis_for(uri)
            .expect("opened document has analysis");
        if let Some(row) = analysis
            .rows
            .iter()
            .find(|row| contains(row.selection, byte))
        {
            return Ok(vec![(uri, row)]);
        }
        if at > 0 && analysis.tokens[at - 1].kind == TokenKind::Dot {
            return Ok(vec![]);
        }
        if analysis
            .opaque_scopes
            .iter()
            .any(|span| contains(*span, byte))
        {
            return Ok(vec![]);
        }
        let name = Sym::intern(token.spelling(&doc.text));
        let mut locals = analysis
            .rows
            .iter()
            .filter(|row| {
                row.local
                    && row.name == name
                    && contains(row.scope, byte)
                    && row.selection.start <= byte
            })
            .collect::<Vec<_>>();
        locals.sort_by_key(|row| {
            (
                row.scope.end - row.scope.start,
                std::cmp::Reverse(row.selection.start),
            )
        });
        if let Some(row) = locals.first() {
            return Ok(vec![(uri, *row)]);
        }
        let mut out = vec![];
        for candidate in self.reachable(uri) {
            for row in &self.analyses[candidate].rows {
                if row.parent.is_none()
                    && !row.local
                    && row.name == name
                    && (candidate == uri || !row.file_private)
                {
                    out.push((candidate, row));
                }
            }
        }
        Ok(out)
    }

    pub fn definition(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<Vec<Location>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some(location) = self.link_definition(uri, byte) {
            return Ok(vec![location]);
        }
        if let Some(locations) = self.doc_link_definition(uri, byte) {
            return Ok(locations);
        }
        if let Some(probe) = self.probe(uri, &doc.text, None) {
            let found = self
                .with_semantic_at(uri, &probe, byte, |a, path| {
                    Some(a.definition(path, byte)).filter(|d| !d.is_empty())
                })
                .unwrap_or_default();
            let locations: Vec<Location> = found
                .into_iter()
                .filter_map(|(path, text, start, end)| {
                    let target = DocumentUri::from_path(Path::new(&path)).ok()?;
                    // An open document's own text and index, else the compiled file's.
                    let range = match self.documents.get(&target) {
                        Some(open) if open.same_positions(&text) => open
                            .index
                            .range(
                                &open.text,
                                Span {
                                    start,
                                    end,
                                },
                            )
                            .ok()?,
                        _ => LineIndex::new(&text)
                            .range(
                                &text,
                                Span {
                                    start,
                                    end,
                                },
                            )
                            .ok()?,
                    };
                    Some(Location {
                        uri: target.as_str().into(),
                        range,
                    })
                })
                .collect();
            if !locations.is_empty() {
                return Ok(locations);
            }
        }
        self.bindings(uri, position)?
            .iter()
            .map(|(target, row)| {
                Ok(Location {
                    uri: target.as_str().into(),
                    range: self
                        .document(target)?
                        .index
                        .range(&self.document(target)?.text, row.selection)?,
                })
            })
            .collect()
    }

    /// Text of a file the editor may not have open (a definition in a module or the stdlib):
    /// the open document's text, else the environment's file system.
    pub fn source(&self, uri: &DocumentUri) -> Option<String> {
        if let Some(key) = self.document_uri(uri)
            && let Some(doc) = self
                .documents
                .get(&key)
                .or_else(|| self.lint_configs.get(&key))
        {
            return Some(doc.text.clone());
        }
        let bytes = self.environment.as_ref()?.fs.read(Path::new(uri.path()))?;
        String::from_utf8(bytes).ok()
    }

    /// Hover as plain text.
    pub fn hover(&self, uri: &DocumentUri, position: Position) -> Result<Option<Hover>, Error> {
        self.hover_as(uri, position, MarkupKind::PlainText)
    }

    /// Hover written as `kind`: Markdown puts code in fenced `jai` blocks and starts each
    /// section (what a macro expands to, what `#run` printed) with a thematic break.
    pub fn hover_as(
        &self,
        uri: &DocumentUri,
        position: Position,
        kind: MarkupKind,
    ) -> Result<Option<Hover>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        let found = |start: usize, end: usize, text: HoverText| -> Result<Option<Hover>, Error> {
            Ok(Some(Hover {
                contents: MarkupContent {
                    kind,
                    value: text.render(kind),
                },
                range: doc.index.range(
                    &doc.text,
                    Span {
                        start,
                        end,
                    },
                )?,
            }))
        };
        if let Some((start, end, text)) = self.link_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.format_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.directive_hover(uri, byte) {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.cast_flag_hover(&doc.text, byte) {
            return found(start, end, text);
        }
        if doc.has_asm()
            && let Some((start, end, text)) = self.asm_hover(&doc.text, byte)
        {
            return found(start, end, text);
        }
        if let Some((start, end, text)) = self.doc_link_hover(uri, byte, kind) {
            return found(start, end, text);
        }
        let Some((_, token)) = self.word(uri, position)? else {
            return match self.expansion_hover(uri, byte) {
                Some((start, end, text)) => found(start, end, text),
                None => Ok(None),
            };
        };
        if let Some(facts) = self.semantic_hover(uri, &doc.text, byte) {
            let (start, end) = (facts.start, facts.end);
            let text = self.hover_with_docs(
                uri,
                facts.text,
                facts.layout.map(layout_line),
                &facts.decls,
                kind,
            );
            let text = self.with_macro_expansion(uri, start, text);
            return found(start, end, text);
        }
        let rows = self.bindings(uri, position)?;
        let text = if rows.len() == 1 {
            let mut blocks = vec![Block::Code(
                Self::source_detail(self.document(rows[0].0)?, rows[0].1).into(),
            )];
            let at = rows[0].1.selection.start;
            if let Some(raw) = crate::docs::declaration(&self.documents[rows[0].0].text, at)
                && let Some(doc) = self
                    .render_docs(uri, rows[0].0.path(), at, &[raw], kind)
                    .pop()
            {
                blocks.push(Block::Doc(doc));
            }
            blocks.push(Block::Note(vec![Inline::Text(
                "Source syntax declaration. \
                 Type evaluation and compile-time execution are disabled during editing."
                    .into(),
            )]));
            HoverText::new(blocks)
        } else if token.kind == TokenKind::Keyword {
            HoverText::new(vec![Block::Para(vec![
                Inline::Text("keyword ".into()),
                Inline::Code(token.spelling(&doc.text).into()),
            ])])
        } else {
            return Ok(None);
        };
        found(token.span.start, token.span.end, text)
    }

    /// Hover from the type checker: the text as typed if it parses, else with the cursor's line
    /// blanked (offsets elsewhere stay the same).
    pub(crate) fn semantic_hover(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<semantic::HoverFacts> {
        let probe = self.probe(uri, text, None)?;
        let facts = self.with_semantic_at(uri, &probe, byte, |a, path| a.hover(path, byte))?;
        (facts.end <= text.len()
            && text.is_char_boundary(facts.start)
            && text.is_char_boundary(facts.end))
        .then_some(facts)
    }

    /// Completion from the type checker. The word being typed (and `a.b.` before it) is cut
    /// out, so the probe text is the same while a word is typed and its compile is reused.
    fn semantic_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<Completions> {
        let bytes = text.as_bytes();
        let ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80;
        let mut start = byte;
        while start > 0 && ident(bytes[start - 1]) {
            start -= 1;
        }
        let prefix = text[start..byte].to_owned();
        let mut chain = Vec::new();
        let mut at = start;
        // A `.` that is the second of `..` is a range, not member access: `0..ta`, `a..b.c`.
        let member_dot =
            |at: usize| at > 0 && bytes[at - 1] == b'.' && !(at > 1 && bytes[at - 2] == b'.');
        while member_dot(at) {
            let end = at - 1;
            let mut s = end;
            while s > 0 && ident(bytes[s - 1]) {
                s -= 1;
            }
            if s == end
                && chain.is_empty()
                && !(end > 0 && matches!(bytes[end - 1], b')' | b']' | b'}' | b'"' | b'\''))
            {
                // `.Member` with an inferred type: the enum its context expects.
                return self.inferred_completion(uri, text, byte, end, prefix);
            }
            if s == end || bytes[s].is_ascii_digit() {
                // `f().x`, a number: nothing to offer.
                return Some((prefix, Vec::new(), BTreeMap::new()));
            }
            chain.push(&text[s..end]);
            at = s;
        }
        chain.reverse();
        // The program as last compiled knows everything before the expression being typed.
        // Declarations below the cursor are where the compiled text had them; their docs are
        // read from the document as it is now.
        let ask = |a: &mut semantic::Analysis, path: &Path| {
            let mut names = a.complete(path, at, &chain)?;
            a.shift_to(path, text, &mut names);
            let files = a.decl_files(&names);
            Some((names, files))
        };
        if let Some((names, files)) = self.with_compiled_before(uri, text, at, ask) {
            return Some((prefix, names, files));
        }
        let probe = self.probe(uri, text, None)?;
        let (names, files) = self.with_semantic(uri, &probe, ask)?;
        Some((prefix, names, files))
    }

    /// Members for `.NAME` typed with `dot` the offset of the `.`: the probe holds a
    /// placeholder name in place of the word (so the compiler records the type its context
    /// expects), and the line is closed with whichever of a few endings makes the file parse.
    fn inferred_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        dot: usize,
        prefix: String,
    ) -> Option<Completions> {
        const PLACEHOLDER: &str = "__jailsp_member";
        const ENDINGS: [&str; 10] = ["", ";", ")", ");", "))", ")));", "]", "];", " {}", ") {}"];
        let ident = |c: char| c.is_alphanumeric() || c == '_';
        let tail = byte
            + text[byte..]
                .find(|c: char| !ident(c))
                .unwrap_or(text.len() - byte);
        let line_end = text[tail..].find('\n').map_or(text.len(), |i| tail + i);
        for ending in ENDINGS {
            let candidate = format!(
                "{}{PLACEHOLDER}{}{ending}{}",
                &text[..=dot],
                &text[tail..line_end],
                &text[line_end..]
            );
            let Some(probe) = repair(&candidate, None) else {
                continue;
            };
            if !probe.contains(PLACEHOLDER) {
                continue;
            }
            let found = self.with_semantic(uri, &probe, |a, path| {
                let mut names = a.complete_inferred(path, dot + 1)?;
                a.shift_to(path, text, &mut names);
                let files = a.decl_files(&names);
                Some((names, files))
            });
            let (names, files) = found.unwrap_or_default();
            if names.is_empty() {
                continue;
            }
            return Some((prefix, names, files));
        }
        Some((prefix, Vec::new(), BTreeMap::new()))
    }

    /// Inside the string of `#load "..."` (files and folders relative to the document) or
    /// `#import "..."` (modules on the import path): the entries of the folder typed so far.
    fn path_completion(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
    ) -> Option<CompletionList> {
        let line = &text[text[..byte].rfind('\n').map_or(0, |n| n + 1)..byte];
        let quote = line.rfind('"')?;
        let before = line[..quote].trim_end();
        // The string must be open: an even number of quotes before it on the line.
        if !line[..quote].matches('"').count().is_multiple_of(2) {
            return None;
        }
        let directive = before
            .rsplit(|c: char| c.is_whitespace() || c == '(')
            .next()?;
        let load = directive.starts_with("#load");
        if !load && !directive.starts_with("#import") {
            return None;
        }
        let typed = &line[quote + 1..];
        let (folder, partial) = typed.rsplit_once('/').unwrap_or(("", typed));
        let here = Path::new(uri.path()).parent().unwrap_or(Path::new("/"));
        let roots: Vec<PathBuf> = if load {
            vec![here.join(folder)]
        } else {
            let options = self.options_for(&PathBuf::from(self.root(uri).path()));
            let mut roots: Vec<PathBuf> = options
                .map(|o| o.import_paths)
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.join(folder))
                .collect();
            roots.push(here.join("modules").join(folder));
            roots
        };
        // (name, is_folder): the environment's files and the open documents under each root.
        let mut entries = BTreeSet::new();
        for root in &roots {
            if let Some(environment) = &self.environment {
                entries.extend(environment.fs.list_dir(root));
            }
            for open in self.documents.keys() {
                let Ok(rest) = Path::new(open.path()).strip_prefix(root) else {
                    continue;
                };
                let mut parts = rest.iter();
                let Some(first) = parts.next() else {
                    continue;
                };
                entries.insert((first.to_string_lossy().into_owned(), parts.next().is_some()));
            }
        }
        // jaic's own modules, imported as `Extensions/Name` (stdlib/Extensions).
        let extensions = jaic::STDLIB_EXTENSIONS_DIR;
        let module_detail = if folder == extensions {
            "jaic extension module"
        } else {
            "module"
        };
        let items = entries
            .into_iter()
            .filter(|(name, _)| name.starts_with(partial) && !name.starts_with('.'))
            .filter_map(|(name, is_folder)| {
                let jai = name.strip_suffix(".jai");
                match (is_folder, jai) {
                    (true, _) if !load && folder.is_empty() && name == extensions => {
                        Some(CompletionItem {
                            label: name,
                            kind: CompletionKind::Folder,
                            detail: "jaic extension modules (not official Jai)".into(),
                            ..CompletionItem::default()
                        })
                    }
                    (true, _) => Some(CompletionItem {
                        label: if load {
                            format!("{name}/")
                        } else {
                            name
                        },
                        kind: if load {
                            CompletionKind::Folder
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "folder"
                        } else {
                            module_detail
                        }
                        .into(),
                        ..CompletionItem::default()
                    }),
                    (false, Some(stem)) => Some(CompletionItem {
                        label: if load {
                            name.clone()
                        } else {
                            stem.into()
                        },
                        kind: if load {
                            CompletionKind::File
                        } else {
                            CompletionKind::Module
                        },
                        detail: if load {
                            "file"
                        } else {
                            module_detail
                        }
                        .into(),
                        ..CompletionItem::default()
                    }),
                    _ => None,
                }
            })
            // Loading the file being edited would include it twice.
            .filter(|item| !load || here.join(folder).join(&item.label) != Path::new(uri.path()))
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        Some(CompletionList {
            is_incomplete: false,
            items: items
                .into_iter()
                .filter(|i| seen.insert(i.label.clone()))
                .collect(),
        })
    }

    pub fn completion(
        &self,
        uri: &DocumentUri,
        position: Position,
    ) -> Result<CompletionList, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let byte = doc.index.byte(&doc.text, position)?;
        if let Some(list) = self.asm_completion(uri, &doc.text, byte) {
            return Ok(list);
        }
        let mut list = self.completion_items(uri, byte)?;
        let parameters = self.named_argument_completions(uri, byte);
        if !parameters.is_empty() {
            // Both alternatives remain useful when a local expression shares a parameter name.
            list.items.extend(parameters);
            if list.items.len() > self.limits.symbols {
                list.items.truncate(self.limits.symbols);
                list.is_incomplete = true;
            }
        }
        // Accepting an item replaces the whole word around the cursor, the letters after it
        // included, so confirming in the middle of a word does not leave its tail behind.
        let text = &doc.text;
        let word = |c: char| c.is_alphanumeric() || c == '_';
        let mut start = text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !word(c))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let mut end = byte
            + text[byte..]
                .find(|c: char| !word(c))
                .unwrap_or(text.len() - byte);
        if start > 0
            && text.as_bytes()[start - 1] == b'#'
            && list.items.iter().any(|i| i.label.starts_with('#'))
        {
            // A directive's label includes its `#`.
            start -= 1;
        }
        if let Some(quote) = self.string_path_start(text, byte) {
            // Inside `#load "..."` / `#import "..."`: the entry after the last `/`.
            start = quote;
            end = byte
                + text[byte..]
                    .find(|c: char| c == '"' || c == '/' || c.is_whitespace())
                    .unwrap_or(text.len() - byte);
        }
        if let Ok(range) = doc.index.range(text, Span::new(start, end)) {
            for item in &mut list.items {
                if item.replace.is_none() {
                    item.replace = Some(range);
                }
            }
        }
        Ok(list)
    }

    /// Where the partial path being typed in an open `#load` / `#import` string starts.
    fn string_path_start(&self, text: &str, byte: usize) -> Option<usize> {
        let line_start = text[..byte].rfind('\n').map_or(0, |n| n + 1);
        let line = &text[line_start..byte];
        let quote = line.rfind('"')?;
        if !line[..quote].matches('"').count().is_multiple_of(2) {
            return None;
        }
        let directive = line[..quote]
            .trim_end()
            .rsplit(|c: char| c.is_whitespace() || c == '(')
            .next()?;
        if !(directive.starts_with("#load") || directive.starts_with("#import")) {
            return None;
        }
        let typed = &line[quote + 1..];
        Some(line_start + quote + 1 + typed.rfind('/').map_or(0, |s| s + 1))
    }

    fn completion_items(&self, uri: &DocumentUri, byte: usize) -> Result<CompletionList, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let position = doc.index.position(&doc.text, byte)?;
        // `#` and the start of a directive: offer directives (labels include the `#`).
        // Past the whole separator: it may be multi-byte (a no-break space).
        let word_start = doc.text[..byte]
            .char_indices()
            .rfind(|&(_, c)| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |(i, c)| i + c.len_utf8());
        if doc.text[..word_start].ends_with('#') {
            let typed = doc.text[word_start..byte].to_lowercase();
            return Ok(CompletionList {
                is_incomplete: false,
                items: DIRECTIVES
                    .iter()
                    .filter(|(name, _)| name.starts_with(&typed))
                    .map(|(name, detail)| CompletionItem {
                        label: format!("#{name}"),
                        kind: CompletionKind::Keyword,
                        detail: (*detail).into(),
                        ..CompletionItem::default()
                    })
                    .collect(),
            });
        }
        if let Some(list) = self.path_completion(uri, &doc.text, byte) {
            return Ok(list);
        }
        if let Some((prefix, names, files)) = self.semantic_completion(uri, &doc.text, byte) {
            let lower = prefix.to_lowercase();
            let mut items = BTreeMap::new();
            let mut incomplete = false;
            let keywords = KEYWORDS.iter().map(|k| IdeName {
                name: (*k).into(),
                kind: IdeKind::Constant,
                detail: "keyword".into(),
                decl: jaic::source::Span::NONE,
            });
            let member = {
                let before = &doc.text.as_bytes()[..byte - prefix.len()];
                before.ends_with(b".") && !before.ends_with(b"..")
            };
            let all = names.into_iter().map(|n| (n, false));
            let all: Vec<(IdeName, bool)> = if member {
                all.collect()
            } else {
                all.chain(keywords.map(|k| (k, true))).collect()
            };
            for (name, keyword) in all {
                if !name.name.to_lowercase().starts_with(&lower) {
                    continue;
                }
                if items.len() >= self.limits.symbols {
                    incomplete = true;
                    break;
                }
                let kind = match name.kind {
                    _ if keyword => CompletionKind::Keyword,
                    IdeKind::Variable => CompletionKind::Variable,
                    IdeKind::Constant => CompletionKind::Constant,
                    IdeKind::Function => CompletionKind::Function,
                    IdeKind::Type => CompletionKind::Struct,
                    IdeKind::Module => CompletionKind::Module,
                    IdeKind::Field => CompletionKind::Field,
                    IdeKind::EnumMember => CompletionKind::EnumMember,
                };
                let data = files.get(&name.decl.file.0).map(|path| CompletionData {
                    document: uri.as_str().to_owned(),
                    path: path.clone(),
                    offset: name.decl.start as usize,
                });
                items.entry(name.name.clone()).or_insert(CompletionItem {
                    label: name.name,
                    kind,
                    detail: name.detail,
                    data,
                    ..CompletionItem::default()
                });
            }
            let mut items: Vec<CompletionItem> = items.into_values().collect();
            // A short list carries its documentation; a long one waits for the client to
            // ask about the item it highlights.
            if items.len() <= crate::doc_view::EAGER_DOCS {
                for item in &mut items {
                    if let Some(data) = &item.data {
                        item.documentation = self.completion_documentation(data);
                    }
                }
            }
            if !member {
                incomplete |= self.add_auto_imports(uri, &doc.text, &prefix, &mut items);
            }
            return Ok(CompletionList {
                is_incomplete: incomplete,
                items,
            });
        }
        let analysis = match self.analysis_for(uri) {
            Some(a) if a.complete => a,
            _ => self.parsed_for(uri).unwrap_or(
                self.analysis_for(uri)
                    .expect("opened document has analysis"),
            ),
        };
        let prefix = self
            .word(uri, position)?
            .filter(|(_, token)| token.kind == TokenKind::Ident)
            .map_or("", |(_, token)| {
                &doc.text[token.span.start..byte.min(token.span.end)]
            });
        if self.word(uri, position)?.is_some_and(|(at, _)| {
            at > 0
                && self
                    .analysis_for(uri)
                    .expect("opened document has analysis")
                    .tokens[at - 1]
                    .kind
                    == TokenKind::Dot
        }) {
            return Ok(CompletionList {
                is_incomplete: true,
                items: vec![],
            });
        }
        let mut items = BTreeMap::new();
        let mut incomplete = !analysis.complete;
        for candidate in self.reachable(uri) {
            let rows = if candidate == uri {
                &analysis.rows
            } else {
                &self.analyses[candidate].rows
            };
            for row in rows {
                let visible = if row.local {
                    candidate == uri && contains(row.scope, byte) && row.selection.start <= byte
                } else {
                    row.parent.is_none() && (candidate == uri || !row.file_private)
                };
                let name = row.name.as_str();
                if visible && name.starts_with(prefix) && items.len() < self.limits.symbols {
                    let kind = match row.kind {
                        SymbolKind::Function => CompletionKind::Function,
                        SymbolKind::Struct => CompletionKind::Struct,
                        SymbolKind::Enum => CompletionKind::Enum,
                        SymbolKind::Constant => CompletionKind::Constant,
                        SymbolKind::TypeAlias => CompletionKind::TypeAlias,
                        _ => CompletionKind::Variable,
                    };
                    items.entry(name.to_owned()).or_insert(CompletionItem {
                        label: name.into(),
                        kind,
                        detail: Self::source_detail(&self.documents[candidate], row).into(),
                        ..CompletionItem::default()
                    });
                } else if visible && items.len() >= self.limits.symbols {
                    incomplete = true;
                }
            }
        }
        for name in KEYWORDS {
            if name.starts_with(prefix) && items.len() < self.limits.symbols {
                items.entry((*name).into()).or_insert(CompletionItem {
                    label: (*name).into(),
                    kind: CompletionKind::Keyword,
                    detail: "keyword".into(),
                    ..CompletionItem::default()
                });
            }
        }
        let mut items: Vec<CompletionItem> = items.into_values().collect();
        incomplete |= self.add_auto_imports(uri, &doc.text, prefix, &mut items);
        Ok(CompletionList {
            is_incomplete: incomplete,
            items,
        })
    }

    /// Append the auto-import items for `prefix` (names not among `items`); whether the client
    /// should ask again as the word grows.
    fn add_auto_imports(
        &self,
        uri: &DocumentUri,
        text: &str,
        prefix: &str,
        items: &mut Vec<CompletionItem>,
    ) -> bool {
        let visible: BTreeSet<String> = items.iter().map(|i| i.label.clone()).collect();
        let (auto, incomplete) = self.auto_import_items(uri, text, prefix, &visible);
        items.extend(auto);
        incomplete
    }

    /// Tokens classified by the syntax layer, refined by the type checker when the session has
    /// an environment: uses of types, procedures, macros, modules, constants and enum members,
    /// `$T` parameters, notes, and the `%` directives of format strings.
    pub fn semantic_tokens(&self, uri: &DocumentUri) -> Result<Vec<SemanticToken>, Error> {
        let (uri, doc) = self.document_entry(uri)?;
        let text = doc.text.as_str();
        let analysis = &self
            .analysis_for(uri)
            .expect("opened document has analysis");
        let classes: BTreeMap<usize, (usize, IdeClass)> = if self.environment.is_some() {
            self.identifier_classes(uri)
                .into_iter()
                .map(|(span, class)| (span.start, (span.end, class)))
                .collect()
        } else {
            BTreeMap::new()
        };
        // `$T` / `$$T`: the name is a type parameter throughout its procedure.
        let mut poly: HashMap<&str, Vec<Span>> = HashMap::new();
        for pair in analysis.tokens.windows(2) {
            let (sigil, name) = (pair[0], pair[1]);
            if matches!(sigil.spelling(text), "$" | "$$")
                && name.kind == TokenKind::Ident
                && sigil.span.end == name.span.start
                && let Some(row) = analysis
                    .rows
                    .iter()
                    .filter(|r| {
                        r.kind == SymbolKind::Function && contains(r.location, name.span.start)
                    })
                    .min_by_key(|r| r.location.end - r.location.start)
            {
                poly.entry(name.spelling(text))
                    .or_default()
                    .push(row.location);
            }
        }
        let format_calls = self.format_calls(uri);
        let mut specs: Vec<Span> = format_calls
            .iter()
            .flat_map(|c| c.specs.iter().map(|s| s.span))
            .collect();
        specs.sort_by_key(|s| s.start);
        // The first row declared at each token (a row per token would be a scan per token).
        let mut declared: HashMap<Span, usize> = HashMap::with_capacity(analysis.rows.len());
        for (at, row) in analysis.rows.iter().enumerate() {
            declared.entry(row.selection).or_insert(at);
        }
        // (span, kind, declaration, readonly, expand)
        let mut pieces: Vec<(Span, SemanticTokenKind, bool, bool, bool)> = Vec::new();
        for (i, token) in analysis.tokens.iter().enumerate() {
            let row = declared.get(&token.span).map(|&at| &analysis.rows[at]);
            let declaration = row.is_some();
            let readonly = row.is_some_and(|row| row.readonly);
            let piece = |kind| (token.span, kind, declaration, readonly, false);
            match token.kind {
                // `#string WGSL`: the editor highlights the body as that language.
                TokenKind::String if crate::here_string::names_language(token.spelling(text)) => {}
                TokenKind::String => {
                    let mut at = token.span.start;
                    let first = specs.partition_point(|s| s.start < token.span.start);
                    for spec in specs[first..]
                        .iter()
                        .take_while(|s| s.start < token.span.end)
                        .filter(|s| s.end <= token.span.end)
                    {
                        if spec.start > at {
                            pieces.push((
                                Span::new(at, spec.start),
                                SemanticTokenKind::String,
                                false,
                                false,
                                false,
                            ));
                        }
                        pieces.push((
                            *spec,
                            SemanticTokenKind::FormatSpecifier,
                            false,
                            false,
                            false,
                        ));
                        at = spec.end;
                    }
                    if at < token.span.end {
                        pieces.push((
                            Span::new(at, token.span.end),
                            SemanticTokenKind::String,
                            false,
                            false,
                            false,
                        ));
                    }
                }
                TokenKind::Keyword => pieces.push(piece(SemanticTokenKind::Keyword)),
                TokenKind::Number => pieces.push(piece(SemanticTokenKind::Number)),
                TokenKind::Directive if token.spelling(text).starts_with('@') => {
                    pieces.push(piece(SemanticTokenKind::Decorator))
                }
                TokenKind::Directive => pieces.push(piece(SemanticTokenKind::Macro)),
                TokenKind::Dot | TokenKind::Punctuation => {
                    pieces.push(piece(SemanticTokenKind::Operator))
                }
                TokenKind::Ident => {
                    let spelling = token.spelling(text);
                    if poly.get(spelling).is_some_and(|scopes| {
                        scopes
                            .iter()
                            .any(|scope| contains(*scope, token.span.start))
                    }) {
                        let sigil = i > 0
                            && matches!(analysis.tokens[i - 1].spelling(text), "$" | "$$")
                            && analysis.tokens[i - 1].span.end == token.span.start;
                        pieces.push((
                            token.span,
                            SemanticTokenKind::TypeParameter,
                            sigil,
                            true,
                            false,
                        ));
                        continue;
                    }
                    let base = match row.map(|row| row.kind) {
                        Some(SymbolKind::Function) => SemanticTokenKind::Function,
                        Some(SymbolKind::Struct | SymbolKind::Enum | SymbolKind::TypeAlias) => {
                            SemanticTokenKind::Type
                        }
                        Some(SymbolKind::Property) => SemanticTokenKind::Property,
                        Some(SymbolKind::Namespace) => SemanticTokenKind::Namespace,
                        Some(SymbolKind::EnumMember) => SemanticTokenKind::EnumMember,
                        Some(SymbolKind::Variable)
                            if row.is_some_and(|row| row.parent.is_some() && row.local) =>
                        {
                            SemanticTokenKind::Parameter
                        }
                        _ => SemanticTokenKind::Variable,
                    };
                    let (kind, constant, expand) = match classes.get(&token.span.start) {
                        Some(&(end, class)) if end == token.span.end => match class {
                            IdeClass::Variable if base == SemanticTokenKind::Parameter => {
                                (base, false, false)
                            }
                            IdeClass::Variable => (SemanticTokenKind::Variable, false, false),
                            IdeClass::Constant => (SemanticTokenKind::Variable, true, false),
                            IdeClass::Function => (SemanticTokenKind::Function, false, false),
                            IdeClass::Macro => (SemanticTokenKind::Function, false, true),
                            IdeClass::Type => (SemanticTokenKind::Type, false, false),
                            IdeClass::Module => (SemanticTokenKind::Namespace, false, false),
                            IdeClass::Field => (SemanticTokenKind::Property, false, false),
                            IdeClass::EnumMember => (SemanticTokenKind::EnumMember, true, false),
                        },
                        _ => (base, false, false),
                    };
                    pieces.push((token.span, kind, declaration, readonly || constant, expand));
                }
            }
        }
        // Code in the Jai code blocks of comments.
        let fences = Self::fence_tokens(text);
        if !fences.is_empty() {
            pieces.extend(
                fences
                    .into_iter()
                    .map(|(span, kind)| (span, kind, false, false, false)),
            );
            pieces.sort_by_key(|p| p.0.start);
        }
        // Links in doc comments, colored like what they name (a comment has no token of its own).
        if self.environment.is_some() {
            let links = self.doc_link_tokens(uri);
            if !links.is_empty() {
                pieces.extend(
                    links
                        .into_iter()
                        .map(|(span, kind, readonly)| (span, kind, false, readonly, false)),
                );
                pieces.sort_by_key(|p| p.0.start);
            }
        }
        let mut output = vec![];
        for (span, kind, declaration, readonly, expand) in pieces {
            let first = doc.index.position(text, span.start)?.line as usize;
            let last = doc.index.position(text, span.end)?.line as usize;
            for (_, (start, end)) in doc
                .index
                .lines()
                .enumerate()
                .skip(first)
                .take(last - first + 1)
            {
                let start = start.max(span.start);
                let end = end.min(span.end);
                if start >= end {
                    continue;
                }
                if output.len() >= self.limits.tokens {
                    return Err(Error::Limit("semantic token line budget exceeded"));
                }
                let position = doc.index.position(text, start)?;
                let length = text[start..end].encode_utf16().count() as u32;
                output.push(SemanticToken {
                    position,
                    length,
                    kind,
                    declaration,
                    readonly,
                    expand,
                });
            }
        }
        Ok(output)
    }
}

/// Current-source IDE probe. Statement recovery happens inside the compiler parser;
/// no lines are deleted and every unaffected byte keeps its original source position.
pub(crate) fn repair(text: &str, first: Option<usize>) -> Option<String> {
    let _ = first;
    // The IDE parser recovers at grammar boundaries, retaining current source spans.
    Some(text.to_owned())
}

pub(crate) fn contains(span: Span, byte: usize) -> bool {
    span.start <= byte && byte <= span.end
}

/// A hover's memory layout line: `size 24, align 8 (4 bytes of padding)` for a type,
/// `offset 8, size 4, align 4 (4 bytes of padding before)` for a field.
fn layout_line(layout: IdeLayout) -> String {
    let bytes = |n: u64| {
        if n == 1 {
            "1 byte".to_owned()
        } else {
            format!("{n} bytes")
        }
    };
    let mut line = match layout.offset {
        Some(offset) => format!(
            "offset {offset}, size {}, align {}",
            layout.size, layout.align
        ),
        None => format!("size {}, align {}", layout.size, layout.align),
    };
    if layout.padding > 0 {
        let before = if layout.offset.is_some() {
            " before"
        } else {
            ""
        };
        line.push_str(&format!(" ({} of padding{before})", bytes(layout.padding)));
    }
    line
}

#[cfg(test)]
#[path = "uri_alias_tests.rs"]
mod uri_alias_tests;
