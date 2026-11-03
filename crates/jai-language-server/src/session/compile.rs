//! Rebuilding analyses and running semantic compiles for a document.
use super::{CompileInputs, Session};
use crate::analysis::Span;
use crate::semantic::{self};
use crate::{DiagnosticCode, DiagnosticSeverity, DocumentUri, analysis::Analysis};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

impl Session {
    pub(super) fn rebuild(&mut self) {
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
        if self.builds_skipped.replace(false) {
            return Some(inputs);
        }
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
            .is_ok_and(|d| std::ptr::eq(d.text.as_str(), text))
    }

    fn work_out_inputs(&self, uri: &DocumentUri, text: &str) -> Option<CompileInputs> {
        let environment = self.environment.as_ref()?;
        let files: BTreeMap<PathBuf, Arc<[u8]>> = self
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
                (PathBuf::from(u.path()), Arc::from(view.as_bytes()))
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
        if !self.may_compile() {
            return self
                .with_cached_semantic(uri, text, query)
                .unwrap_or_else(|| {
                    self.want_compile(uri, text);
                    None
                });
        }
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        let analysis = cache.analyze(environment, &root, &dirs, files)?;
        let path = Path::new(uri.path());
        analysis.focus(path);
        let answer = query(analysis, path);
        self.sync_builds(&cache);
        answer
    }

    /// Like `with_semantic`, but only on a compile that is already kept: `None` when there is
    /// none (nothing is compiled), else what `query` answered.
    pub(crate) fn with_cached_semantic<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<Option<T>> {
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        let analysis = cache.cached(environment, &root, &dirs, &files)?;
        let path = Path::new(uri.path());
        analysis.focus(path);
        Some(query(analysis, path))
    }

    /// Whether a compile may be made now: always, until `defer_compiles` turns it off for
    /// requests (the diagnostics and `warm_pending` still compile, under `background`).
    pub(crate) fn may_compile(&self) -> bool {
        !self.defer_compiles.get() || self.background.get() > 0
    }

    /// Lets the compiles made until the guard drops through while requests may not compile.
    pub(crate) fn background(&self) -> Background<'_> {
        self.background.set(self.background.get() + 1);
        Background(self)
    }

    /// Remembers that `text` of `uri` wanted a compile a request did not make, and that the
    /// answer is not what a compile of it gives. The document's own text is compiled for its
    /// diagnostics anyway, so it only fills an empty slot.
    pub(crate) fn want_compile(&self, uri: &DocumentUri, text: &str) {
        self.stale_used.set(true);
        let mut pending = self.compile_pending.borrow_mut();
        if pending.is_none() || !self.is_own_text(uri, text) {
            *pending = Some((uri.clone(), text.to_string()));
        }
    }

    /// Whether the answer just made came from a compile of older text, which it forgets.
    pub(crate) fn take_stale(&self) -> bool {
        self.stale_used.replace(false)
    }

    /// Runs the `[project] entry` files that could build `uri` but have not been run (or were
    /// changed since), once each, so their recorded facts decide which program `uri` is in
    /// instead of a scan of their text. A run that fails is not repeated until the
    /// metaprogram, what it loads or `jacket.toml` changes.
    fn run_unrecorded_builds(&self, uri: &DocumentUri, files: &BTreeMap<PathBuf, Arc<[u8]>>) {
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
        if !self.may_compile() {
            self.builds_skipped.set(true);
            return;
        }
        let buffers: semantic::Buffers = Rc::new(files.clone());
        for (entry, hash) in pending {
            self.index.borrow_mut().probed.insert(entry.clone(), hash);
            let dirs = self.import_dirs(&entry);
            let mut cache = self.semantic.borrow_mut();
            if cache
                .analyze(environment, &entry, &dirs, buffers.clone())
                .is_none()
            {
                return;
            }
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

    /// Like `with_semantic` for a question about `byte` of `uri`, `query` given the offset in
    /// the text the answer comes from, and the answer the `Shift` from that text to `text`
    /// (none unless requests may not compile: then an older compile answers, if `byte` is
    /// outside what was edited since). When the program's compile skipped that code (an
    /// `#if OS == ...` branch, or a file only another target loads) and has no answer,
    /// compiles for the other targets are asked in `OTHER_TARGETS` order, the first that
    /// checked it answering.
    pub(crate) fn with_semantic_at<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        query: impl Fn(&mut semantic::Analysis, &Path, usize) -> Option<T>,
    ) -> Option<(T, semantic::Shift)> {
        if !self.may_compile() {
            return self.on_older_compile(uri, text, |a, path, shift| {
                let at = shift.to_old(byte)?;
                Some((query(a, path, at)?, *shift))
            });
        }
        let found = self.compiled_at(uri, text, byte, |a, path| query(a, path, byte))?;
        Some((found, semantic::Shift::none()))
    }

    /// Runs `query` on the compile of `uri`'s program that answers for `text` exactly, else on
    /// the last one made, with the `Shift` from its text to `text`; none is compiled and the
    /// text is queued for `warm_pending`. Answers made from an older compile mark themselves
    /// with `take_stale`.
    pub(crate) fn on_older_compile<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl FnOnce(&mut semantic::Analysis, &Path, &semantic::Shift) -> Option<T>,
    ) -> Option<T> {
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let path = Path::new(uri.path());
        let mut cache = self.semantic.borrow_mut();
        if let Some(exact) = cache.cached(environment, &root, &dirs, &files) {
            exact.focus(path);
            return query(exact, path, &semantic::Shift::none());
        }
        self.want_compile(uri, text);
        let analysis = cache.latest(&root, &dirs)?;
        let old = analysis.compiled_text(path)?;
        let shift = semantic::Shift::between(old.as_bytes(), text.as_bytes());
        analysis.focus(path);
        query(analysis, path, &shift)
    }

    /// Like `with_semantic` for a query that asks about no offset of `text`, so the last
    /// compile answers when none fits `text` and requests may not compile.
    pub(crate) fn with_semantic_or_older<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        if self.may_compile() {
            self.with_semantic(uri, text, query)
        } else {
            self.on_older_compile(uri, text, |a, path, _| query(a, path))
        }
    }

    fn compiled_at<T>(
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
        let analysis = cache.analyze(environment, &root, &dirs, files.clone())?;
        analysis.focus(path);
        let first = query(analysis, path);
        self.sync_builds(&cache);
        if let Some(found) = first {
            return Some(found);
        }
        let analysis = cache.analyze(environment, &root, &dirs, files.clone())?;
        if !analysis.skipped(path, byte) {
            return None;
        }
        let own = (environment.options)(&root).os;
        let mut unreached = self.unreached.borrow_mut();
        if unreached.0 != self.generation.get() {
            *unreached = (self.generation.get(), BTreeMap::default());
        }
        // Always in the same order, so the answer does not depend on which target answered
        // an earlier question: code several targets check is described as the first sees it.
        let order: Vec<jaic::target::TargetOs> = semantic::OTHER_TARGETS
            .into_iter()
            .filter(|&os| {
                os != own
                    && !unreached
                        .1
                        .get(&(path.to_path_buf(), os as u8))
                        .is_some_and(|spans| spans.iter().any(|&(a, b)| a <= byte && byte <= b))
            })
            .collect();
        for os in order {
            let analysis = cache.analyze_for(environment, &root, &dirs, files.clone(), Some(os))?;
            analysis.focus(path);
            if let Some(span) = analysis.skipped_span(path, byte) {
                unreached
                    .1
                    .entry((path.to_path_buf(), os as u8))
                    .or_default()
                    .push(span);
            } else {
                return query(analysis, path);
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
}

/// Compiles are allowed while it lives (see `Session::background`).
pub(crate) struct Background<'a>(&'a Session);

impl Drop for Background<'_> {
    fn drop(&mut self) {
        self.0.background.set(self.0.background.get() - 1);
    }
}
