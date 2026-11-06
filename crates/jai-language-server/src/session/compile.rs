//! Rebuilding analyses and running semantic compiles for a document.
use super::{CompileInputs, Session};
use crate::analysis::Span;
use crate::semantic::{self, QueryAnalysis, QuerySelection};
use crate::{DiagnosticCode, DiagnosticSeverity, DocumentUri, analysis::Analysis};
use jaic::sema::FileSystem;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

enum SemanticOutcome<T> {
    Accepted(Option<T>),
    Unavailable,
}

enum BuildAuthority {
    Proven,
    Advisory,
    Unavailable,
}

impl Session {
    pub(super) fn rebuild(&mut self) {
        // Opening, closing or editing one document leaves the others as they were: an
        // analysis of the same text is kept. One with `#load`s is built again, because the
        // warnings about files that are not open depend on which documents are.
        let mut analyses: BTreeMap<DocumentUri, Rc<Analysis>> = BTreeMap::new();
        for (uri, document) in &self.documents {
            let kept = self.analysis_for(uri).filter(|a| {
                a.loads.is_empty()
                    && a.key == crate::semantic::text_key(&document.text)
                    && *a.source == document.text
            });
            if let Some(kept) = kept {
                analyses.insert(uri.clone(), kept.clone());
                continue;
            }
            let mut analysis = Analysis::build(&document.ide_text(), uri, self.limits);
            analysis.key = crate::semantic::text_key(&document.text);
            analysis.source = document.text.as_str().into();
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
        self.root_with(uri, None)
    }

    /// Choose an exact open-load owner using actual observed native identities.
    /// Example: a physical loaded file can belong to a loader opened through an alias.
    pub(crate) fn projected_root(&self, uri: &DocumentUri) -> DocumentUri {
        self.root_with(uri, self.config_filesystem().as_ref())
    }

    fn root_with(
        &self,
        uri: &DocumentUri,
        fs: Option<&crate::semantic::ObservedFs<'_>>,
    ) -> DocumentUri {
        let resolve = |uri: &DocumentUri| {
            match fs {
                Some(fs) => self.projected_uri(fs, Path::new(uri.path())),
                None => self.document_uri(uri),
            }
            .unwrap_or_else(|| uri.clone())
        };
        let actual = resolve(uri);
        let uri = &actual;
        let loaded: BTreeSet<DocumentUri> = self
            .analyses
            .values()
            .flat_map(|analysis| analysis.loads.iter().map(|(target, _)| resolve(target)))
            .collect();
        if !loaded.contains(uri) {
            return uri.clone();
        }
        self.documents
            .keys()
            .find(|document| {
                !loaded.contains(*document) && self.reachable_with(document, fs).contains(&uri)
            })
            .unwrap_or(uri)
            .clone()
    }

    /// What a compile of the program containing `uri` needs, with `uri`'s text replaced by
    /// `text`: the open files, the program's root file and the module folders searched.
    pub(crate) fn compile_inputs(&self, uri: &DocumentUri, text: &str) -> Option<CompileInputs> {
        let _query = self.discovery_query();
        // The outer scope already validated the retained discovery proof. New observations
        // remain sticky; misses validate before memo publication, and semantic consumers
        // validate their complete accumulated context before accepting an answer.
        if self.documents.values().any(|doc| doc.identity_pending)
            || !self.input_proof.borrow().is_consistent()
            || self
                .environment
                .as_ref()
                .is_some_and(|env| env.fs.input_epoch_valid() == Some(false))
        {
            return None;
        }
        self.sync_builds(&self.semantic.borrow());
        let key = self.projected_uri(&self.config_filesystem()?, Path::new(uri.path()))?;
        let uri = &key;
        // The document's own text (as most requests pass it) needs no comparing: any change to
        // a document moves the generation on.
        let own = self.is_own_text(uri, text);
        let memo = self.inputs.borrow();
        if let Some((generation, last, last_text, inputs, actual)) = &*memo
            && *generation == self.generation.get()
            && last == uri
            && match last_text {
                None => own,
                Some(last_text) => last_text == text,
            }
        {
            let restored = self.restore_input_epoch(actual.as_ref());
            // This matching memo's originals and the outer discovery scope already own
            // the sealed input view. New URI observations must still fit that same view.
            let ready = self.input_epoch.as_ref().is_some_and(|epoch| epoch.ready())
                && self
                    .environment
                    .as_ref()
                    .is_some_and(|env| env.fs.input_epoch_valid() == Some(true));
            let accepted = restored
                && if ready {
                    self.input_proof.borrow().is_consistent()
                        && !self.configuration_input_failed.get()
                        && !jaic::interrupt::fired()
                } else {
                    self.discovery_inputs_current()
                };
            let answer = accepted.then(|| inputs.clone());
            drop(memo);
            if !accepted {
                // A selected old memo cannot be rediscovered under a contradictory history
                // in this same transaction. The next request may capture fresh inputs.
                *self.inputs.borrow_mut() = None;
                *self.memo_input_proof.borrow_mut() = semantic::Observations::default();
            }
            return answer;
        }
        drop(memo);
        if !self
            .semantic
            .borrow()
            .has_accepted_source(Path::new(uri.path()))
            && !self.begin_cold_input_capture()
        {
            return None;
        }
        self.index.borrow_mut().discard_input_memos();
        *self.input_proof.borrow_mut() = self.request_input_proof.borrow().clone();
        let _capture = self.capture_discovery_inputs();
        let inputs = self.work_out_inputs(uri, text)?;
        if !self.discovery_inputs_current() {
            return None;
        }
        if self.builds_skipped.replace(false) {
            return Some(inputs);
        }
        *self.memo_input_proof.borrow_mut() = self.input_proof.borrow().clone();
        *self.inputs.borrow_mut() = Some((
            self.generation.get(),
            uri.clone(),
            (!own).then(|| text.to_string()),
            inputs.clone(),
            self.input_epoch.as_ref().map(|epoch| epoch.actual_inputs()),
        ));
        Some(inputs)
    }

    // A genuinely absent compiler context can capture new operations before selection.
    // A failed selected transaction or coverage miss must never reach this reset.
    pub(super) fn begin_cold_input_capture(&self) -> bool {
        if !self.input_proof.borrow().is_consistent()
            || self.configuration_input_failed.get()
            || jaic::interrupt::fired()
        {
            return false;
        }
        let Some(epoch) = &self.input_epoch else {
            return true;
        };
        if !epoch.ready() {
            return epoch.input_epoch_valid() != Some(false);
        }
        let original = epoch.actual_inputs();
        epoch.begin_request_capture();
        epoch.reuse_actual(&original)
    }

    /// Whether `text` is the open document's own buffer, not a copy or an edited probe.
    fn is_own_text(&self, uri: &DocumentUri, text: &str) -> bool {
        self.document(uri)
            .is_ok_and(|d| std::ptr::eq(d.text.as_str(), text))
    }

    fn work_out_inputs(&self, uri: &DocumentUri, text: &str) -> Option<CompileInputs> {
        let _query = self.discovery_query();
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
        let open_root = self.projected_root(uri);
        {
            let buffers = Rc::new(files.clone());
            let mut cache = self.semantic.borrow_mut();
            cache.validate_probes(environment, &buffers);
            self.sync_builds(&cache);
        }
        match self.run_unrecorded_builds(&open_root, &files) {
            BuildAuthority::Proven => {}
            BuildAuthority::Advisory => {
                let dirs = self.last_good_import_dirs(uri);
                if !self.index.borrow().advisory_builds.is_empty() {
                    *self.advisory_inputs.borrow_mut() =
                        Some((Rc::new(files), PathBuf::from(uri.path()), dirs));
                }
                return None;
            }
            BuildAuthority::Unavailable => return None,
        }
        let root = match self.project_root(&open_root) {
            Some(entry) if self.config_filesystem()?.is_file(&entry) => entry,
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
        let _query = self.discovery_query();
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let mut analysis = cache.query(
            environment,
            &root,
            &dirs,
            files,
            QuerySelection::Exact {
                compile: true,
                os: None,
            },
        )?;
        let path = Path::new(uri.path());
        analysis.focus(path);
        let answer = self.semantic_query(analysis, path, query);
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
        let _query = self.discovery_query();
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let mut analysis = cache.query(
            environment,
            &root,
            &dirs,
            files,
            QuerySelection::Exact {
                compile: false,
                os: None,
            },
        )?;
        let path = Path::new(uri.path());
        analysis.focus(path);
        Some(self.semantic_query(analysis, path, query))
    }

    /// Best-effort hints may use last-good import discovery, never an exact compiler entry.
    pub(crate) fn with_best_effort_semantic<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        query: impl Fn(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let _scope = self.discovery_query();
        if let Some(answer) = self.with_semantic(uri, text, &query) {
            return Some(answer);
        }
        if !self.may_compile() || !self.discovery_inputs_current() {
            return None;
        }
        let (files, root, dirs) = self.advisory_inputs.borrow().clone()?;
        let env = self.environment.as_ref()?;
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let mut analysis = cache.advisory_analysis(env, &root, &dirs, files)?;
        self.stale_used.set(true);
        let analysis = QueryAnalysis::validate(&mut analysis, env, &self.input_proof.borrow())?;
        self.semantic_query(analysis, Path::new(uri.path()), query)
    }

    /// Project offset-sensitive output before the selected Analysis's final guard.
    /// Example: definition locations apply the selected older-text shift inside this callback.
    pub(crate) fn with_best_effort_semantic_at_projected<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        query: impl Fn(&mut semantic::Analysis, &Path, usize, &semantic::Shift) -> Option<T>,
    ) -> Option<(T, semantic::Shift)> {
        let _scope = self.discovery_query();
        let exact = if self.may_compile() {
            self.compiled_at(uri, text, byte, |a, path| {
                query(a, path, byte, &semantic::Shift::none())
            })
            .map(|answer| (answer, semantic::Shift::none()))
        } else {
            self.on_older_compile(uri, text, |a, path, shift| {
                let at = shift.to_old(byte)?;
                Some((query(a, path, at, shift)?, *shift))
            })
        };
        if exact.is_some() {
            return exact;
        }
        if !self.may_compile() || !self.discovery_inputs_current() {
            return None;
        }
        let (files, root, dirs) = self.advisory_inputs.borrow().clone()?;
        let env = self.environment.as_ref()?;
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let mut analysis = cache.advisory_analysis(env, &root, &dirs, files)?;
        self.stale_used.set(true);
        let analysis = QueryAnalysis::validate(&mut analysis, env, &self.input_proof.borrow())?;
        self.semantic_query(analysis, Path::new(uri.path()), |a, path| {
            query(a, path, byte, &semantic::Shift::none())
        })
        .map(|answer| (answer, semantic::Shift::none()))
    }

    /// Every callback may resolve existing entities or add facts without changing a counter.
    /// Freeze resulting primary-program views only after the query completes without cancellation.
    fn semantic_query<T>(
        &self,
        analysis: QueryAnalysis<'_>,
        path: &Path,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        match self.semantic_query_outcome(analysis, path, query) {
            SemanticOutcome::Accepted(answer) => answer,
            SemanticOutcome::Unavailable => None,
        }
    }

    /// Distinguish accepted emptiness from a failed transaction before alternate selection.
    /// Example: a rejected native proof must not trigger a same-request recheck.
    fn semantic_query_outcome<T>(
        &self,
        analysis: QueryAnalysis<'_>,
        path: &Path,
        query: impl FnOnce(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> SemanticOutcome<T> {
        let analysis = analysis.into_analysis();
        let _hints = analysis.hint_query();
        let Some(environment) = self.environment.as_ref() else {
            return SemanticOutcome::Unavailable;
        };
        let fs = &*environment.fs;
        let capture_identity = self
            .input_epoch
            .as_ref()
            .map(|epoch| epoch.capture_identity());
        if let Some(epoch) = &self.input_epoch {
            analysis.input_epoch = Some(Rc::downgrade(epoch));
            if analysis
                .actual_inputs
                .as_ref()
                .is_some_and(|original| !epoch.reuse_actual(original))
            {
                analysis.invalidate_reference_views(true);
                return SemanticOutcome::Unavailable;
            }
        }
        analysis
            .reads
            .borrow_mut()
            .disk
            .merge(&self.input_proof.borrow());
        if jaic::interrupt::fired() || !analysis.reads.borrow().disk.is_consistent() {
            analysis.invalidate_reference_views(true);
            return SemanticOutcome::Unavailable;
        }
        // Input equality does not prove equality of compiler facts after an IDE query.
        // Retire only this Analysis owner, preserving unrelated and evicted contexts.
        let graph_owners = self.call_graph_owners(&analysis.facts_accepted);
        analysis.facts_accepted.set(false);
        analysis.facts_accepted = Rc::new(std::cell::Cell::new(false));
        analysis.invalidate_reference_views(false);
        let answer = query(analysis, path);
        self.refresh_call_graphs(&graph_owners, analysis);
        analysis.focus(path);
        analysis.invalidate_reference_views(false);
        analysis
            .reads
            .borrow_mut()
            .disk
            .merge(&self.input_proof.borrow());
        let references = self.capture_reference_records(analysis);
        if !analysis.freeze_actual_inputs() {
            analysis.invalidate_reference_views(true);
            return SemanticOutcome::Unavailable;
        }
        let cancelled = jaic::interrupt::fired()
            || self.configuration_input_failed.get()
            || !analysis.reads.borrow().is_current(fs);
        let hints_current = !cancelled && analysis.hints_current(fs);
        let unchanged = self
            .input_epoch
            .as_ref()
            .map(|epoch| epoch.capture_identity())
            == capture_identity;
        // Optional output failure cannot poison an otherwise accepted semantic epoch.
        let epoch_current = cancelled || (unchanged && self.accept_input_epoch());
        let cancelled = cancelled || !epoch_current || jaic::interrupt::fired();
        analysis.facts_accepted.set(!cancelled);
        if cancelled {
            analysis.invalidate_reference_views(true);
            self.call_graphs
                .borrow_mut()
                .retain(|_, graph| graph.accepted());
            return SemanticOutcome::Unavailable;
        }
        if !hints_current {
            return SemanticOutcome::Unavailable;
        }
        self.publish_reference_records(references);
        SemanticOutcome::Accepted(answer)
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
    fn run_unrecorded_builds(
        &self,
        uri: &DocumentUri,
        files: &BTreeMap<PathBuf, Arc<[u8]>>,
    ) -> BuildAuthority {
        let Some(environment) = self.environment.as_ref() else {
            return BuildAuthority::Unavailable;
        };
        if !self.trusted() {
            return BuildAuthority::Proven;
        }
        let initial = self.unrecorded_builds(uri);
        let buffers: semantic::Buffers = Rc::new(files.clone());
        if !initial.pending.is_empty() && !self.may_compile() {
            self.builds_skipped.set(true);
            return BuildAuthority::Unavailable;
        }
        for entry in initial.pending {
            let dirs = self.import_dirs(&entry);
            let mut cache = self.semantic.borrow_mut();
            cache.set_input_proof(self.input_proof.borrow().clone());
            if cache
                .probe(environment, &entry, &dirs, buffers.clone())
                .is_none()
            {
                return BuildAuthority::Unavailable;
            }
            self.sync_builds(&cache);
        }
        let current = self.unrecorded_builds(uri);
        let proofs = self.semantic.borrow().probe_reads(&current.entries);
        for reads in proofs {
            self.input_proof.borrow_mut().merge(&reads.disk);
            self.request_input_proof.borrow_mut().merge(&reads.disk);
        }
        if !current.pending.is_empty() || !self.discovery_inputs_current() {
            BuildAuthority::Unavailable
        } else if current.failed {
            BuildAuthority::Advisory
        } else {
            BuildAuthority::Proven
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
        index.probed = cache.probe_statuses();
        index.advisory_builds = cache.advisory_builds();
        index.discard_input_memos();
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
        let _query = self.discovery_query();
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let path = Path::new(uri.path());
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        if let Some(mut exact) = cache.query(
            environment,
            &root,
            &dirs,
            files.clone(),
            QuerySelection::Exact {
                compile: false,
                os: None,
            },
        ) {
            exact.focus(path);
            return self.semantic_query(exact, path, |a, path| {
                query(a, path, &semantic::Shift::none())
            });
        }
        self.want_compile(uri, text);
        let mut analysis = cache.query(environment, &root, &dirs, files, QuerySelection::Latest)?;
        let old = analysis.compiled_text(path)?;
        let shift = semantic::Shift::between(old.as_bytes(), text.as_bytes());
        analysis.focus(path);
        self.semantic_query(analysis, path, |a, path| query(a, path, &shift))
    }

    fn compiled_at<T>(
        &self,
        uri: &DocumentUri,
        text: &str,
        byte: usize,
        query: impl Fn(&mut semantic::Analysis, &Path) -> Option<T>,
    ) -> Option<T> {
        let _query = self.discovery_query();
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let path = Path::new(uri.path());
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let mut analysis = cache.query(
            environment,
            &root,
            &dirs,
            files.clone(),
            QuerySelection::Exact {
                compile: true,
                os: None,
            },
        )?;
        analysis.focus(path);
        let first = self.semantic_query_outcome(analysis, path, &query);
        self.sync_builds(&cache);
        match first {
            SemanticOutcome::Accepted(Some(found)) => return Some(found),
            SemanticOutcome::Accepted(None) => {}
            SemanticOutcome::Unavailable => return None,
        }
        let analysis = cache.analyze(environment, &root, &dirs, files.clone())?;
        if !analysis.skipped(path, byte) {
            return None;
        }
        let own = (environment.options)(&self.config_filesystem()?, &root).os;
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
                        .is_some_and(|(input, spans)| {
                            input.root == root
                                && input.dirs == dirs
                                && input.os == Some(os)
                                && input.is_current(environment, &files, cache.authority())
                                && spans.iter().any(|&(a, b)| a <= byte && byte <= b)
                        })
            })
            .collect();
        for os in order {
            if !cache.has_accepted_context(&root, &dirs, Some(os))
                && !self.begin_cold_input_capture()
            {
                return None;
            }
            let mut analysis = cache.query(
                environment,
                &root,
                &dirs,
                files.clone(),
                QuerySelection::Exact {
                    compile: true,
                    os: Some(os),
                },
            )?;
            analysis.focus(path);
            if let Some(span) = analysis.skipped_span(path, byte) {
                unreached.1.insert(
                    (path.to_path_buf(), os as u8),
                    (semantic::CheckedInputs::capture(&analysis), vec![span]),
                );
                while unreached
                    .1
                    .iter()
                    .map(|((path, _), (input, spans))| {
                        path.capacity()
                            + input.bytes
                            + spans.capacity() * std::mem::size_of::<(usize, usize)>()
                            + 128
                    })
                    .sum::<usize>()
                    > 16 << 20
                {
                    unreached.1.pop_first();
                }
            } else {
                return self.semantic_query(analysis, path, &query);
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
        let _query = self.discovery_query();
        let environment = self.environment.as_ref()?;
        let (files, root, dirs) = self.compile_inputs(uri, text)?;
        let mut cache = self.semantic.borrow_mut();
        cache.set_input_proof(self.input_proof.borrow().clone());
        let path = Path::new(uri.path());
        let analysis = cache.query(
            environment,
            &root,
            &dirs,
            files,
            QuerySelection::Prefix {
                path,
                prefix: &text.as_bytes()[..at],
            },
        )?;
        self.semantic_query(analysis, path, query)
    }
}

/// Compiles are allowed while it lives (see `Session::background`).
pub(crate) struct Background<'a>(&'a Session);

impl Drop for Background<'_> {
    fn drop(&mut self) {
        self.0.background.set(self.0.background.get() - 1);
    }
}

#[cfg(test)]
#[path = "compile_tests.rs"]
mod tests;
