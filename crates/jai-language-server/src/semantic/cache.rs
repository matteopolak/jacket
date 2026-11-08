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

pub struct Cache {
    entries: Vec<Analysis>,
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
    /// The last good run of each build metaprogram (by its canonical root path). A run that
    /// fails or runs out of budget leaves these alone.
    pub(crate) recorded: BTreeMap<PathBuf, BuildFacts>,
    /// Moves on whenever `recorded` changes.
    pub(crate) recorded_version: u64,
    /// Limits the compiles since the last `take_limit_hits` ran into.
    limit_hits: Vec<super::queries::LimitHit>,
}

impl Default for Cache {
    fn default() -> Self {
        Cache {
            entries: Vec::new(),
            canonical: jaic::paths::CanonCache::default(),
            compiles: 0,
            parses: shared_parses(),
            untrusted: false,
            limits: CheckLimits::default(),
            recorded: BTreeMap::new(),
            recorded_version: 0,
            limit_hits: Vec::new(),
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
/// changed buffer is unrelated, and opened or closed inputs still have their checked bytes.
fn unaffected(
    analysis: &Analysis,
    files: &BTreeMap<PathBuf, Arc<[u8]>>,
    base: &dyn jaic::sema::FileSystem,
) -> bool {
    let reads = analysis.reads.borrow();
    let same = |old: &Arc<[u8]>, new: &Arc<[u8]>| Arc::ptr_eq(old, new) || old == new;
    analysis.files.iter().all(|(path, old)| {
        !reads.concerns(path)
            || match files.get(path) {
                Some(new) => same(old, new),
                None => base
                    .read(path)
                    .is_some_and(|bytes| bytes.as_slice() == old.as_ref()),
            }
    }) && files.iter().all(|(path, text)| {
        analysis.files.contains_key(path)
            || !reads.concerns(path)
            || std::iter::once(&analysis.compiler)
                .chain(analysis.others.iter().map(|p| &p.compiler))
                .any(|compiler| {
                    super::compile::find_file(compiler, path).is_some_and(|file| {
                        compiler.ide_records_source(file)
                            && compiler.src.sources.get(file).text.as_bytes() == text.as_ref()
                    })
                })
            || reads.only_checked_present(path)
                && base.is_file(path)
                && base
                    .read(path)
                    .is_some_and(|bytes| bytes.as_slice() == text.as_ref())
    })
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
    /// Whether compile-time code may run. A change drops every compile kept, which were made
    /// under the other setting.
    pub fn set_trusted(&mut self, trusted: bool) {
        if self.untrusted != trusted {
            return;
        }
        *self = Self {
            untrusted: !trusted,
            ..Self::default()
        };
    }

    /// The limits compiles run under. A change drops every compile kept: its errors and its
    /// stopping point were those of the other limits.
    pub fn set_limits(&mut self, limits: CheckLimits) {
        if self.limits != limits {
            self.entries.clear();
            self.limits = limits;
        }
    }

    /// The limits compiles ran into since the last call.
    pub(crate) fn take_limit_hits(&mut self) -> Vec<super::queries::LimitHit> {
        std::mem::take(&mut self.limit_hits)
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
    }

    /// Disk or buffer contents changed; opening identical disk text does not call this.
    pub fn invalidate(&mut self, path: &Path, topology: bool) {
        self.entries
            .retain(|a| !topology && !a.reads.borrow().concerns(path));
    }

    /// Drop every compile (answers they were made without have arrived); recorded build runs
    /// stay.
    pub fn forget_compiles(&mut self) {
        self.entries.clear();
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
        let small = |a: &Analysis| a.cost <= SMALL_COST;
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
                        .max_by_key(|&i| self.entries[i].cost)
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
        let canonical: BTreeMap<PathBuf, Arc<[u8]>> = files
            .iter()
            .map(|(p, t)| (jaic::paths::file_identity(&*env.fs, p), t.clone()))
            .collect();
        let path = jaic::paths::file_identity(&*env.fs, path);
        let at = self.entries.iter().rposition(|a| {
            a.usable
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
        let at = match self.lookup(env, root, dirs, &files, os, true) {
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
                let analysis = self.build(env, root, dirs, files, inputs, os)?;
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
        let at = self.lookup(env, root, dirs, files, None, false).ok()?;
        Some(&mut self.entries[at])
    }

    /// The kept analysis of `root` that was used last, whatever text it compiled: an answer
    /// from it is about older text, for the caller to move to the text now.
    pub fn latest(&mut self, root: &Path, dirs: &[PathBuf]) -> Option<&mut Analysis> {
        self.entries
            .iter_mut()
            .rev()
            .find(|a| a.usable && a.os.is_none() && a.root == root && a.dirs == dirs)
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
                .rposition(|a| a.usable && a.os.is_none() && a.root == root);
            let mut at = 0;
            self.entries.retain(|a| {
                let keep = a.os.is_some() || at == active || Some(at) == checked;
                at += 1;
                keep
            });
        }
        self.build(env, root, dirs, files, inputs, None)
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
    /// compile. When `remember`, the analysis found takes the inputs as its own.
    fn lookup(
        &mut self,
        env: &Environment,
        root: &Path,
        dirs: &[PathBuf],
        given: &Buffers,
        os: Option<TargetOs>,
        remember: bool,
    ) -> Result<usize, Inputs> {
        // The same buffers as last time (the session hands out its inputs again until a
        // document changes): no need to hash megabytes of text to know it is the same program.
        let quick = quick_key(root, dirs, given, os);
        if let Some(at) = self
            .entries
            .iter()
            .position(|a| a.quick.as_ref().is_some_and(|(k, _)| *k == quick))
        {
            return Ok(at);
        }
        let inputs = self.inputs(env, root, dirs, given, os);
        // The very same texts, or texts that differ only in files the compile never read: a
        // buffer it did not load (another program's file) is not one of its inputs.
        let at = self
            .entries
            .iter()
            .position(|a| a.key == inputs.key)
            .or_else(|| {
                self.entries.iter().rposition(|a| {
                    a.os == os
                        && a.root == root
                        && a.dirs == dirs
                        && unaffected(a, &inputs.files, &*env.fs)
                })
            });
        let Some(at) = at else {
            return Err(inputs);
        };
        if remember {
            let entry = &mut self.entries[at];
            entry.quick = Some((inputs.quick, given.clone()));
            if entry.key != inputs.key {
                entry.key = inputs.key;
                entry.files = inputs.files;
            }
        }
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
        os: Option<TargetOs>,
    ) -> Option<Analysis> {
        self.compiles += 1;
        let Inputs {
            quick,
            key,
            files,
        } = inputs;
        let snapshot = files.clone();
        // Folders a past run of this metaprogram added are not handed back to it: it adds
        // them again if it still does, and a removed `import_path` line must disappear.
        let past = self
            .recorded
            .get(&jaic::paths::file_identity(&*env.fs, root));
        let given_dirs: Vec<PathBuf> = dirs
            .iter()
            .filter(|d| !past.is_some_and(|f| f.programs.iter().any(|p| p.import_dirs.contains(d))))
            .cloned()
            .collect();
        let Compiled {
            compiler,
            errors,
            others,
            facts,
            stopped,
            reads,
        } = compile(
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
        )?;
        // Stopped for a newer request: the compile is partial, and nothing of it is kept.
        if jaic::interrupt::fired() {
            return None;
        }
        // Another target's compile says nothing new about what the metaprogram did.
        if let Some(mut facts) = facts.filter(|_| os.is_none()) {
            facts.checked = self.compiles;
            let key = jaic::paths::file_identity(&*env.fs, root);
            if self.recorded.get(&key) != Some(&facts) {
                self.recorded.insert(key, facts);
                self.recorded_version += 1;
            }
        }
        self.limit_hits
            .extend(super::queries::limit_hits(&compiler, &errors));
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
        Some(Analysis {
            compiler,
            entry: Some(root.to_path_buf()),
            key,
            quick: Some((quick, given)),
            os,
            root: root.to_path_buf(),
            dirs: dirs.to_vec(),
            files: snapshot,
            reads,
            usable,
            cost,
            complete: errors.is_empty() && !stopped,
            errors,
            lints: BTreeMap::new(),
            enriched: None,
            found: RefCell::default(),
            others,
        })
    }
}
