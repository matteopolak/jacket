//! The cache of finished analyses.
use super::compile::{Compiled, Mode, compile};
use super::parses::parse_error;
use super::{
    ALTERNATES, Analysis, Buffers, BuildFacts, CACHED, Environment, SMALL_COST, SMALL_KEPT,
};
use jaic::ide::lint_compile::CheckLimits;
use jaic::source::FileId;
use jaic::target::TargetOs;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// A selected analysis whose captured inputs were just validated by this cache.
/// It is borrow-scoped and consumed once by the semantic query transaction.
pub(crate) struct QueryAnalysis<'a>(&'a mut Analysis);

impl std::ops::Deref for QueryAnalysis<'_> {
    type Target = Analysis;

    fn deref(&self) -> &Analysis {
        self.0
    }
}

impl<'a> QueryAnalysis<'a> {
    pub(crate) fn focus(&mut self, path: &Path) {
        self.0.focus(path);
    }

    pub(crate) fn into_analysis(self) -> &'a mut Analysis {
        self.0
    }

    /// Detached/advisory work enters through an explicit complete validation.
    pub(crate) fn validate(
        analysis: &'a mut Analysis,
        env: &Environment,
        proof: &super::Observations,
    ) -> Option<Self> {
        analysis.reads.borrow_mut().disk.merge(proof);
        if !analysis.query_valid
            || jaic::interrupt::fired()
            || !analysis.reads.borrow().is_current(&*env.fs)
        {
            analysis.invalidate_reference_views(true);
            return None;
        }
        Some(Self(analysis))
    }
}

pub(crate) enum QuerySelection<'a> {
    Exact { compile: bool, os: Option<TargetOs> },
    Latest,
    Prefix { path: &'a Path, prefix: &'a [u8] },
}

enum BuildKind {
    Exact(Option<TargetOs>),
    Advisory,
}

pub struct Cache {
    authority: Rc<()>,
    entries: Vec<Analysis>,
    input_proof: super::Observations,
    /// Canonical forms of the paths compiled, so a request does not ask the file system again.
    canonical: jaic::paths::CanonCache,
    /// Programs compiled so far.
    compiles: usize,
    /// Parsed files compiles share: most files are the same from one compile to the next.
    /// Every session on a thread shares one (`shared_parses`).
    parses: Rc<RefCell<jaic::parser::ParseReuse>>,
    /// Workspace trust is withheld: compiles run no compile-time code (see
    /// `IdeFacts::untrusted`). Survives `reset`; change it with `set_trusted`.
    untrusted: bool,
    /// What bounds a compile's compile-time code (the client's settings); change it with
    /// `set_limits`.
    limits: CheckLimits,
    /// Currently proven positive build memberships, by canonical producer path.
    /// Last-good advisory metadata is retained separately by `probes`.
    pub(crate) recorded: BTreeMap<PathBuf, Rc<BuildFacts>>,
    probes: super::probes::Probes,
    /// Moves on when published probe memberships or statuses change.
    pub(crate) recorded_version: u64,
    /// Limits the compiles since the last `take_limit_hits` ran into.
    limit_hits: super::notices::Notices,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            authority: Rc::new(()),
            entries: Vec::new(),
            input_proof: super::Observations::default(),
            canonical: jaic::paths::CanonCache::default(),
            compiles: 0,
            parses: shared_parses(),
            untrusted: false,
            limits: CheckLimits::default(),
            recorded: BTreeMap::new(),
            probes: super::probes::Probes::default(),
            recorded_version: 0,
            limit_hits: super::notices::Notices::default(),
        }
    }
}

/// The parses of this thread's sessions. A kept parse is only taken for the very same text
/// under the same path and file id, and a tree is never changed after parsing, so a session
/// gets what it would have parsed itself; the stdlib is then parsed once for all of them
/// instead of once per session.
fn shared_parses() -> Rc<RefCell<jaic::parser::ParseReuse>> {
    thread_local! {
        static PARSES: Rc<RefCell<jaic::parser::ParseReuse>> = Rc::default();
    }
    PARSES.with(Rc::clone)
}

/// A compile's open texts by canonical path, with the keys they are known by.
struct Inputs {
    quick: u64,
    key: u64,
    files: BTreeMap<PathBuf, Arc<[u8]>>,
}

/// Whether `analysis` answers for `files` as well as for the texts it was compiled from: every
/// buffer that changed, appeared or went away is one it never read, tested or searched a
/// folder for.
fn unaffected(analysis: &Analysis, files: &BTreeMap<PathBuf, Arc<[u8]>>) -> bool {
    let reads = analysis.reads.borrow();
    let same = |old: &Arc<[u8]>, new: &Arc<[u8]>| Arc::ptr_eq(old, new) || old == new;
    analysis.files.iter().all(|(path, old)| {
        files.get(path).is_some_and(|new| same(old, new)) || !reads.concerns(path)
    }) && files
        .keys()
        .all(|path| analysis.files.contains_key(path) || !reads.concerns(path))
}

pub(super) fn hash(
    root: &Path,
    dirs: &[PathBuf],
    files: &BTreeMap<PathBuf, Arc<[u8]>>,
    os: Option<TargetOs>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = jaic::fxhash::FxHasher::default();
    format!("{os:?}").hash(&mut h);
    root.hash(&mut h);
    dirs.hash(&mut h);
    files.hash(&mut h);
    h.finish()
}

/// A key for inputs by the map of texts they share rather than its contents: equal only for
/// the very same map (or one at the same address, which `Analysis::quick` keeps alive).
fn quick_key(root: &Path, dirs: &[PathBuf], files: &Buffers, os: Option<TargetOs>) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = jaic::fxhash::FxHasher::default();
    os.map(|os| os as u8).hash(&mut h);
    h.write(root.as_os_str().as_encoded_bytes());
    for dir in dirs {
        h.write(dir.as_os_str().as_encoded_bytes());
    }
    (Rc::as_ptr(files) as usize).hash(&mut h);
    h.finish()
}

impl Cache {
    /// Select and validate immediately before a semantic callback.
    /// Example: an exact deferred request uses `Exact { compile: false, os: None }`.
    pub(crate) fn query(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
        selection: QuerySelection<'_>,
    ) -> Option<QueryAnalysis<'_>> {
        match selection {
            QuerySelection::Exact {
                compile,
                os,
            } => {
                let analysis = if compile {
                    self.analyze_for(env, root, dirs, files, os)?
                } else {
                    let at = self.lookup(env, root, dirs, &files, os).ok()?;
                    &mut self.entries[at]
                };
                Some(QueryAnalysis(analysis))
            }
            QuerySelection::Latest => self.latest(env, root, dirs).map(QueryAnalysis),
            QuerySelection::Prefix {
                path,
                prefix,
            } => self
                .compiled_before(env, root, dirs, files, path, prefix)
                .map(QueryAnalysis),
        }
    }

    pub(crate) fn authority(&self) -> &Rc<()> {
        &self.authority
    }

    pub(crate) fn set_input_proof(&mut self, proof: super::Observations) {
        self.input_proof = proof;
    }

    fn refresh_probe_views(&mut self) {
        self.recorded = self.probes.facts();
        self.recorded_version += 1;
    }

    pub(crate) fn restore_advisory(&mut self, facts: BTreeMap<PathBuf, Rc<BuildFacts>>) {
        self.probes.restore_advisory(facts);
        self.refresh_probe_views();
    }

    pub(crate) fn probe_statuses(&self) -> BTreeMap<PathBuf, super::ProbeStatus> {
        self.probes.statuses()
    }

    pub(crate) fn advisory_builds(&self) -> BTreeMap<PathBuf, Rc<BuildFacts>> {
        self.probes.advisory()
    }

    pub(crate) fn probe_counts(&self) -> (usize, usize, usize) {
        self.probes.counts()
    }

    pub(crate) fn probe_reads(&self, entries: &[PathBuf]) -> Vec<Rc<super::Reads>> {
        self.probes.inputs_for(entries)
    }

    pub(crate) fn validate_probes(&mut self, env: &Environment, files: &Buffers) -> bool {
        let changed = self.probes.validate(env, files, &self.authority);
        if changed {
            self.refresh_probe_views();
        }
        changed
    }

    pub(crate) fn end_request(&mut self) -> bool {
        let changed = self.probes.end_request();
        if changed {
            self.refresh_probe_views();
        }
        changed
    }

    pub(crate) fn forget_probe_authority(&mut self) {
        self.authority = Rc::new(());
        self.entries.clear();
        self.probes.clear_exact();
        self.refresh_probe_views();
    }

    #[cfg(test)]
    pub(crate) fn set_probe_budget(&mut self, budget: usize) {
        self.probes.set_budget(budget);
        self.refresh_probe_views();
    }

    pub(crate) fn probe(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
    ) -> Option<super::ProbeStatus> {
        if self.untrusted {
            return None;
        }
        let analysis = self.analyze(env, root, dirs, files)?;
        let history = RefCell::new(super::Observations::default());
        let fs = super::ObservedFs {
            base: &*env.fs,
            proof: Some(&history),
            secondary: None,
        };
        let key = jaic::paths::file_identity(&fs, root);
        analysis
            .reads
            .borrow_mut()
            .disk
            .merge(&history.into_inner());
        if jaic::interrupt::fired()
            || !analysis.reads.borrow().is_current(&*env.fs)
            || jaic::interrupt::fired()
        {
            analysis.query_valid = false;
            return None;
        }
        let status = if analysis.build_complete {
            super::ProbeStatus::Complete
        } else {
            super::ProbeStatus::Failed
        };
        // Clone only immutable proof/source bytes; no compiler or compiler ID escapes.
        let proof = super::probes::ProbeCapture::new(analysis);
        self.probes.remember_capture(key, proof);
        self.refresh_probe_views();
        Some(status)
    }

    /// Whether compile-time code may run. A change drops every compile kept, which were made
    /// under the other setting.
    pub fn set_trusted(&mut self, trusted: bool) {
        if self.untrusted != trusted {
            return;
        }
        let notices = self.limit_hits.enabled();
        *self = Self {
            untrusted: !trusted,
            limits: self.limits,
            ..Self::default()
        };
        self.limit_hits.set_enabled(notices);
    }

    /// The limits compiles run under. A change drops every compile kept: its errors and its
    /// stopping point were those of the other limits.
    pub fn set_limits(&mut self, limits: CheckLimits) {
        if self.limits != limits {
            self.entries.clear();
            self.forget_probe_authority();
            self.limits = limits;
        }
    }

    pub(crate) fn limit_notice_counts(&self) -> (usize, usize, usize) {
        self.limit_hits.counts()
    }

    pub(crate) fn set_limit_notices(&mut self, enabled: bool) {
        self.limit_hits.set_enabled(enabled);
    }

    /// Historical accepted-build events since the last drain; no source text is retained.
    pub(crate) fn take_limit_hits(
        &mut self,
    ) -> std::collections::VecDeque<super::queries::LimitHit> {
        self.limit_hits.take()
    }

    pub fn trusted(&self) -> bool {
        !self.untrusted
    }

    /// Programs compiled so far, and compiles alive now.
    pub fn counts(&self) -> (usize, usize) {
        (self.compiles, self.entries.len())
    }

    /// Files appeared or went away: canonical paths are asked for again.
    pub fn forget_paths(&mut self) {
        self.canonical.clear();
        for analysis in &self.entries {
            analysis.overlay.forget_paths();
        }
    }

    /// Drop every compile (answers they were made without have arrived); recorded build runs
    /// stay.
    pub fn forget_compiles(&mut self) {
        self.entries.clear();
    }

    /// A previously proven path identity, useful when a watched alias has just been deleted.
    pub(crate) fn known_path(&self, path: &Path) -> Option<PathBuf> {
        self.canonical.get(path)
    }

    /// Retire analyses that observed a watched path, including failed file lookups and
    /// directory searches. Unrelated disk edits leave exact checked programs reusable.
    pub fn invalidate_path(&mut self, path: &Path) {
        self.entries
            .retain(|analysis| !analysis.reads.borrow().concerns_tree(path));
    }

    /// Drop the lints found so far (their settings changed); the compiles stay.
    pub fn forget_lints(&mut self) {
        for entry in &mut self.entries {
            entry.lints.clear();
        }
    }

    /// Drop compiles until one fewer than `CACHED` is left, not counting up to `SMALL_KEPT`
    /// small ones. The one kept is the newest that checked of `root` (what completion falls
    /// back to while its text does not parse), else the costliest that checked (a project's
    /// compile, not a stray file's that takes milliseconds to redo), else the newest. A compile
    /// for another target makes room only among those (one is kept), so asking about a skipped
    /// branch does not cost the program's own compile.
    fn make_room(&mut self, os: Option<TargetOs>, root: &Path) {
        if os.is_some() {
            // The oldest compile for another target goes; the newest stays, so hovers that go
            // back and forth between a Windows and a Linux branch do not recompile each time.
            while self.entries.iter().filter(|a| a.os.is_some()).count() >= ALTERNATES {
                let oldest = self.entries.iter().position(|a| a.os.is_some());
                self.entries.remove(oldest.expect("counted"));
            }
            return;
        }
        let small = |a: &Analysis| a.retained_cost() <= SMALL_COST;
        loop {
            let (light, heavy): (Vec<usize>, Vec<usize>) = (0..self.entries.len())
                .filter(|&i| self.entries[i].os.is_none())
                .partition(|&i| small(&self.entries[i]));
            let over_light = light.len() > SMALL_KEPT;
            if heavy.len() + light.len().saturating_sub(SMALL_KEPT) < CACHED {
                break;
            }
            let own = |a: &Analysis| a.usable && a.os.is_none();
            let keep = self
                .entries
                .iter()
                .rposition(|a| own(a) && a.root == root)
                .or_else(|| {
                    (0..self.entries.len())
                        .filter(|&i| own(&self.entries[i]))
                        .max_by_key(|&i| self.entries[i].retained_cost())
                })
                .or_else(|| self.entries.iter().rposition(|a| a.os.is_none()));
            // The oldest of the kind there are too many of goes first.
            let class = if over_light {
                &light
            } else {
                &heavy
            };
            let victim = class
                .iter()
                .copied()
                .find(|&i| Some(i) != keep)
                .or_else(|| {
                    (0..self.entries.len())
                        .find(|&i| Some(i) != keep && self.entries[i].os.is_none())
                })
                .unwrap_or(0);
            self.entries.remove(victim);
        }
    }

    /// A compile of `root` that already knows the code before `prefix.len()` of `path`: the
    /// other open files are as they were compiled and `path` starts with `prefix` (what is
    /// typed after that point is not in its facts, and does not need to be). Completion uses it
    /// instead of compiling again for every keystroke; the names declared by edits since that
    /// compile show up once the next one has run.
    pub fn compiled_before(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
        path: &Path,
        prefix: &[u8],
    ) -> Option<&mut Analysis> {
        self.retire_changed(env, root, dirs);
        let canonical: BTreeMap<PathBuf, Arc<[u8]>> = files
            .iter()
            .map(|(p, t)| (jaic::paths::file_identity(&*env.fs, p), t.clone()))
            .collect();
        let path = jaic::paths::file_identity(&*env.fs, path);
        let at = self.entries.iter().rposition(|a| {
            a.usable
                && a.query_valid
                && a.os.is_none()
                && a.root == root
                && a.dirs == dirs
                && a.files.len() == canonical.len()
                && a.files.iter().all(|(p, bytes)| match canonical.get(p) {
                    Some(new) if *p == path => bytes.starts_with(prefix) && new.starts_with(prefix),
                    Some(new) => bytes == new,
                    None => false,
                })
        })?;
        let analysis = &mut self.entries[at];
        analysis.focus(&path);
        Some(analysis)
    }

    /// The analysis of `root` with `files` open (compiled now unless cached).
    pub fn analyze(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
    ) -> Option<&mut Analysis> {
        self.analyze_for(env, root, dirs, files, None)
    }

    /// Like `analyze`, compiled for `os` instead of the environment's target when it is `Some`.
    pub fn analyze_for(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
        os: Option<TargetOs>,
    ) -> Option<&mut Analysis> {
        let at = match self.lookup(env, root, dirs, &files, os) {
            Ok(at) => at,
            Err(inputs) => {
                // A compile of the same program from text that parses will check, and
                // everything the older one can answer (completion after an unchanged prefix) it
                // answers too, so the older one goes now instead of staying alive through the
                // whole new compile. A check that may be stopped (see `jaic::interrupt`) keeps
                // it until the new one is made: stopping must not cost the requests that come
                // next a compile.
                let replaces = inputs.files.values().all(|bytes| {
                    std::str::from_utf8(bytes).is_ok_and(|text| parse_error(text).is_none())
                });
                if replaces && !jaic::interrupt::armed() {
                    self.entries
                        .retain(|a| !(a.root == root && a.dirs == dirs && a.os == os));
                }
                self.make_room(os, root);
                let analysis = self.build(env, root, dirs, files, inputs, BuildKind::Exact(os))?;
                if replaces {
                    self.entries
                        .retain(|a| !(a.root == root && a.dirs == dirs && a.os == os));
                }
                self.entries.push(analysis);
                self.entries.len() - 1
            }
        };
        let entry = self.entries.remove(at);
        self.entries.push(entry);
        self.entries.last_mut()
    }

    /// A kept analysis of `root` with `files` open, if one answers for them, without compiling
    /// and without making it the most recently used.
    pub fn cached(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: &Buffers,
    ) -> Option<&mut Analysis> {
        let at = self.lookup(env, root, dirs, files, None).ok()?;
        Some(&mut self.entries[at])
    }

    /// Freeze a published view only from the exact quick candidate normal lookup selects.
    /// No compiler query, focus switch or filesystem access occurs here.
    pub(crate) fn published_reference_inputs(
        &self,
        root: &Path,
        dirs: &[PathBuf],
        files: &Buffers,
        owner: &Rc<()>,
        index: &Rc<jaic::ide::meta::ReferenceIndex>,
    ) -> Option<super::CheckedInputs> {
        let at = self.published_reference_candidate(root, dirs, files)?;
        let analysis = &self.entries[at];
        if !analysis.reference_records_allowed()
            || !analysis.facts_accepted.get()
            || !Rc::ptr_eq(&analysis.reference_owner, owner)
            || !analysis
                .references
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(current, index))
        {
            return None;
        }
        Some(super::CheckedInputs::capture(analysis))
    }

    fn published_reference_candidate(
        &self,
        root: &Path,
        dirs: &[PathBuf],
        files: &Buffers,
    ) -> Option<usize> {
        let quick = quick_key(root, dirs, files, None);
        self.entries.iter().position(|analysis| {
            analysis.query_valid
                && analysis.os.is_none()
                && analysis.root == root
                && analysis.dirs == dirs
                && analysis
                    .quick
                    .as_ref()
                    .is_some_and(|(key, raw)| *key == quick && Rc::ptr_eq(raw, files))
        })
    }

    /// Retire only the admitted candidate after a failed immutable-answer guard.
    /// The next request can rebuild; historical records retain their original authority.
    pub(crate) fn retire_published_reference_view(
        &mut self,
        root: &Path,
        dirs: &[PathBuf],
        files: &Buffers,
        owner: &Rc<()>,
        index: &Rc<jaic::ide::meta::ReferenceIndex>,
    ) {
        if let Some(at) = self.published_reference_candidate(root, dirs, files) {
            let analysis = &mut self.entries[at];
            if Rc::ptr_eq(&analysis.reference_owner, owner)
                && analysis
                    .references
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(current, index))
            {
                analysis.query_valid = false;
            }
        }
    }

    fn retire_changed(&mut self, env: &Environment, root: &Path, dirs: &[PathBuf]) {
        let before = self.entries.len();
        let proof = &self.input_proof;
        self.entries.retain(|a| {
            if !a.query_valid {
                return false;
            }
            if a.root != root || a.dirs != dirs || a.os.is_some() {
                return true;
            }
            let mut reads = a.reads.borrow_mut();
            reads.disk.merge(proof);
            reads.is_current(&*env.fs)
        });
        if self.entries.len() != before {
            self.forget_paths();
        }
    }

    /// The kept analysis of `root` that was used last, whatever text it compiled: an answer
    /// from it is about older text, for the caller to move to the text now.
    pub fn latest(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
    ) -> Option<&mut Analysis> {
        self.retire_changed(env, root, dirs);
        self.entries.iter_mut().rev().find(|a| {
            a.usable && a.query_valid && a.os.is_none() && a.root == root && a.dirs == dirs
        })
    }

    /// An analysis of `root` that the cache does not keep, for work beside the editor's (the
    /// workspace's polymorph summaries): the compile the open document's requests use stays,
    /// with the one that checked it last, and room is made among the others only.
    pub fn compile_detached(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
    ) -> Option<Analysis> {
        let inputs = self.inputs(env, root, dirs, &files, None);
        if let Some(active) = self.entries.iter().rposition(|a| a.os.is_none()) {
            let root = self.entries[active].root.clone();
            let checked = self
                .entries
                .iter()
                .rposition(|a| a.usable && a.query_valid && a.os.is_none() && a.root == root);
            let mut at = 0;
            self.entries.retain(|a| {
                let keep = a.os.is_some() || at == active || Some(at) == checked;
                at += 1;
                keep
            });
        }
        self.build(env, root, dirs, files, inputs, BuildKind::Exact(None))
    }

    pub(crate) fn advisory_analysis(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        files: Buffers,
    ) -> Option<Analysis> {
        let inputs = self.inputs(env, root, dirs, &files, None);
        self.build(env, root, dirs, files, inputs, BuildKind::Advisory)
    }

    /// The canonical texts and keys of a compile's inputs.
    fn inputs(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        given: &Buffers,
        os: Option<TargetOs>,
    ) -> Inputs {
        if env.fs.is_native() {
            self.canonical.clear();
        }
        let files: BTreeMap<PathBuf, Arc<[u8]>> = given
            .iter()
            .map(|(p, t)| {
                let canonical = self
                    .canonical
                    .get_or_insert_with(p, |p| jaic::paths::file_identity(&*env.fs, p));
                (canonical, t.clone())
            })
            .collect();
        Inputs {
            quick: quick_key(root, dirs, given, os),
            key: hash(root, dirs, &files, os),
            files,
        }
    }

    /// Where a kept analysis answers for these inputs, else the inputs worked out for a new
    /// compile. Reuse updates the compiler's live overlay and its manifest together.
    fn lookup(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        given: &Buffers,
        os: Option<TargetOs>,
    ) -> Result<usize, Inputs> {
        // The same buffers as last time (the session hands out its inputs again until a
        // document changes): no need to hash megabytes of text to know it is the same program.
        // Other programs remain bounded candidates, validated only when selected.
        self.entries.retain(|a| a.query_valid);
        let quick = quick_key(root, dirs, given, os);
        if let Some(at) = self.entries.iter().position(|a| {
            a.query_valid
                && a.os == os
                && a.root == root
                && a.dirs == dirs
                && a.quick
                    .as_ref()
                    .is_some_and(|(k, kept)| *k == quick && Rc::ptr_eq(kept, given))
        }) {
            let entry = &mut self.entries[at];
            entry.reads.borrow_mut().disk.merge(&self.input_proof);
            if entry.reads.borrow().is_current(&*env.fs) {
                return Ok(at);
            }
            entry.query_valid = false;
        }
        let inputs = self.inputs(env, root, dirs, given, os);
        // The very same texts, or texts that differ only in files the compile never read: a
        // buffer it did not load (another program's file) is not one of its inputs.
        let at = self
            .entries
            .iter()
            .position(|a| {
                a.query_valid
                    && a.key == inputs.key
                    && a.os == os
                    && a.root == root
                    && a.dirs == dirs
                    && a.files == inputs.files
            })
            .or_else(|| {
                self.entries.iter().rposition(|a| {
                    a.query_valid
                        && a.os == os
                        && a.root == root
                        && a.dirs == dirs
                        && unaffected(a, &inputs.files)
                })
            });
        let Some(at) = at else {
            return Err(inputs);
        };
        let entry = &mut self.entries[at];
        entry.reads.borrow_mut().disk.merge(&self.input_proof);
        if env.fs.is_native() {
            let mut observed = entry.reads.borrow_mut();
            for path in given.keys() {
                observed.observe_canonical(
                    path.clone(),
                    self.canonical.get(path).expect("canonical reuse input"),
                );
            }
            if !observed.is_current(&*env.fs) {
                entry.query_valid = false;
                return Err(inputs);
            }
        }
        if entry.files != inputs.files {
            entry.overlay.replace_files(inputs.files.clone());
        }
        entry.quick = Some((inputs.quick, given.clone()));
        entry.key = inputs.key;
        entry.files = inputs.files;
        Ok(at)
    }

    /// Compile `root` (counted, and its metaprogram's run recorded).
    fn build(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        given: Buffers,
        inputs: Inputs,
        kind: BuildKind,
    ) -> Option<Analysis> {
        let (os, advisory) = match kind {
            BuildKind::Exact(os) => (os, false),
            BuildKind::Advisory => (None, true),
        };
        // Reject contradictory discovery/input identities before starting compile work.
        // These are already observed responses from input assembly, not a fresh replay.
        let mut accepted_inputs = super::Reads::default();
        accepted_inputs.disk.merge(&self.input_proof);
        if env.fs.is_native() {
            for path in given.keys() {
                accepted_inputs.observe_canonical(
                    path.clone(),
                    self.canonical
                        .get(path)
                        .expect("canonical input captured before compiling"),
                );
            }
        }
        let registry_key = {
            let history = RefCell::new(super::Observations::default());
            let fs = super::ObservedFs {
                base: &*env.fs,
                proof: Some(&history),
                secondary: None,
            };
            let key = jaic::paths::file_identity(&fs, root);
            accepted_inputs.disk.merge(&history.into_inner());
            key
        };
        if !accepted_inputs.disk.is_consistent() {
            return None;
        }
        self.compiles += 1;
        let trace_job = crate::trace::global().and_then(crate::trace::Trace::job_start);
        let Inputs {
            quick,
            key,
            files,
        } = inputs;
        let snapshot = files.clone();
        // Folders a past run of this metaprogram added are not handed back to it: it adds
        // them again if it still does, and a removed `import_path` line must disappear.
        let past = self.recorded.get(&registry_key);
        let given_dirs: Vec<PathBuf> = dirs
            .iter()
            .filter(|d| !past.is_some_and(|f| f.programs.iter().any(|p| p.import_dirs.contains(d))))
            .cloned()
            .collect();
        let compiled = compile(
            env,
            root,
            &given_dirs,
            files,
            os,
            &self.parses,
            Mode {
                untrusted: self.untrusted,
                limits: self.limits,
            },
        );
        let Some(compiled) = compiled else {
            if let Some(job) = trace_job {
                job.finish(crate::trace::Outcome::Abandoned);
            }
            return None;
        };
        let Compiled {
            compiler,
            errors,
            others,
            facts,
            stopped,
            reads,
            overlay,
        } = compiled;
        reads.borrow_mut().merge(&accepted_inputs);
        // The same observed identity selects prior import directories and names publication.
        let producer_key = (!advisory
            && os.is_none()
            && facts
                .as_ref()
                .is_some_and(|facts| !facts.programs.is_empty()))
        .then_some(registry_key);
        // Stopped or mixed disk inputs: none of the partial context is accepted.
        if jaic::interrupt::fired()
            || !reads.borrow().is_current(&*env.fs)
            || jaic::interrupt::fired()
        {
            if let Some(job) = trace_job {
                job.finish(crate::trace::Outcome::Abandoned);
            }
            return None;
        }
        if self.limit_hits.enabled() {
            self.limit_hits
                .extend(super::queries::limit_hits(&compiler, &errors));
        }
        let cost = std::iter::once(&compiler)
            .chain(others.iter().map(|p| &p.compiler))
            .flat_map(|c| {
                (0..c.src.sources.len() as u32).map(|f| c.src.sources.get(FileId(f)).text.len())
            })
            .sum();
        let usable = errors.is_empty()
            || compiler
                .ed
                .facts()
                .is_some_and(|ide| !ide.scopes.is_empty());
        let analysis = Analysis {
            authority: self.authority.clone(),
            facts_accepted: Rc::new(std::cell::Cell::new(true)),
            compiler,
            build_complete: facts.is_some()
                && errors.is_empty()
                && facts
                    .as_ref()
                    .is_none_or(|f| f.programs.iter().all(|p| p.checked))
                && !stopped,
            build_facts: facts,
            advisory,
            query_valid: true,
            entry: Some(root.to_path_buf()),
            key,
            quick: Some((quick, given)),
            os,
            root: root.to_path_buf(),
            dirs: dirs.to_vec(),
            files: snapshot,
            reads,
            overlay,
            usable,
            cost,
            complete: errors.is_empty() && !stopped,
            errors,
            lints: BTreeMap::new(),
            references: None,
            reference_owner: Rc::new(()),
            hints: Rc::default(),
            found: RefCell::default(),
            others,
        };
        if let Some(key) = producer_key {
            self.probes.remember(key, &analysis);
            self.refresh_probe_views();
        }
        if let Some(job) = trace_job {
            job.finish(crate::trace::Outcome::Accepted);
        }
        Some(analysis)
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
