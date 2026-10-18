//! Rebuilding analyses and running semantic compiles for a document.
use super::{CompileInputs, Session};
use crate::analysis::Span;
use crate::semantic::{self};
use crate::{DiagnosticCode, DiagnosticSeverity, DocumentUri, analysis::Analysis};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
}
