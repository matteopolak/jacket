//! The cache of finished analyses.
use super::compile::compile;
use super::parses::parse_error;
use super::{ALTERNATES, Analysis, Buffers, BuildFacts, CACHED, Environment};
use jaic::sema::TargetOs;
use jaic::source::FileId;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Default)]
pub struct Cache {
    entries: Vec<Analysis>,
    /// Canonical forms of the paths compiled, so a request does not ask the file system again.
    canonical: jaic::paths::CanonCache,
    /// Programs compiled so far.
    compiles: usize,
    /// Parsed files compiles share: most files are the same from one compile to the next.
    parses: Rc<RefCell<jaic::parser::ParseReuse>>,
    /// Workspace trust is withheld: compiles run no compile-time code (see
    /// `IdeFacts::untrusted`). Survives `reset`; change it with `set_trusted`.
    untrusted: bool,
    /// The last good run of each build metaprogram (by its canonical root path). A run that
    /// fails or runs out of budget leaves these alone.
    pub(crate) recorded: BTreeMap<PathBuf, BuildFacts>,
    /// Moves on whenever `recorded` changes.
    pub(crate) recorded_version: u64,
}

pub(super) fn hash(
    root: &Path,
    dirs: &[PathBuf],
    files: &BTreeMap<PathBuf, Rc<[u8]>>,
    os: Option<TargetOs>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = crate::fast_hash::FastHasher::default();
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
    let mut h = crate::fast_hash::FastHasher::default();
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

    /// Drop the lints found so far (their settings changed); the compiles stay.
    pub fn forget_lints(&mut self) {
        for entry in &mut self.entries {
            entry.lints.clear();
        }
    }

    /// Drop compiles until one fewer than `CACHED` is left. The one kept is the newest that
    /// checked of `root` (what completion falls back to while its text does not parse), else
    /// the costliest that checked (a project's compile, not a stray file's that takes
    /// milliseconds to redo), else the newest. A compile for another target makes room only
    /// among those (one is kept), so asking about a skipped branch does not cost the program's
    /// own compile.
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
        while self.entries.iter().filter(|a| a.os.is_none()).count() >= CACHED {
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
            let victim = (0..self.entries.len())
                .find(|&i| Some(i) != keep && self.entries[i].os.is_none())
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
        let canonical: BTreeMap<PathBuf, Rc<[u8]>> = files
            .iter()
            .map(|(p, t)| (crate::document::file_key(&*env.fs, p), t.clone()))
            .collect();
        let path = crate::document::file_key(&*env.fs, path);
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
    ) -> &mut Analysis {
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
    ) -> &mut Analysis {
        // The same buffers as last time (the session hands out its inputs again until a
        // document changes): no need to hash megabytes of text to know it is the same program.
        let quick = quick_key(root, dirs, &files, os);
        if let Some(at) = self
            .entries
            .iter()
            .position(|a| a.quick.as_ref().is_some_and(|(k, _)| *k == quick))
        {
            let entry = self.entries.remove(at);
            self.entries.push(entry);
            return self.entries.last_mut().expect("just pushed");
        }
        let given = files.clone();
        let files: BTreeMap<PathBuf, Rc<[u8]>> = files
            .iter()
            .map(|(p, t)| {
                let canonical = self
                    .canonical
                    .get_or_insert_with(p, |p| crate::document::file_key(&*env.fs, p));
                (canonical, t.clone())
            })
            .collect();
        let key = hash(root, dirs, &files, os);
        if let Some(at) = self.entries.iter().position(|a| a.key == key) {
            let mut entry = self.entries.remove(at);
            entry.quick = Some((quick, given));
            self.entries.push(entry);
        } else {
            // A compile of the same program from text that parses will check, and everything the
            // older one can answer (completion after an unchanged prefix) it answers too, so the
            // older one goes now instead of staying alive through the whole new compile.
            if files.values().all(|bytes| {
                std::str::from_utf8(bytes).is_ok_and(|text| parse_error(text).is_none())
            }) {
                self.entries
                    .retain(|a| !(a.root == root && a.dirs == dirs && a.os == os));
            }
            self.make_room(os, root);
            self.compiles += 1;
            let snapshot = files.clone();
            // Folders a past run of this metaprogram added are not handed back to it: it adds
            // them again if it still does, and a removed `import_path` line must disappear.
            let past = self
                .recorded
                .get(&crate::document::path_key(&env.fs.canonical(root)));
            let given_dirs: Vec<PathBuf> = dirs
                .iter()
                .filter(|d| {
                    !past.is_some_and(|f| f.programs.iter().any(|p| p.import_dirs.contains(d)))
                })
                .cloned()
                .collect();
            let (compiler, errors, others, facts) = compile(
                env,
                root,
                &given_dirs,
                files,
                os,
                &self.parses,
                self.untrusted,
            );
            // Another target's compile says nothing new about what the metaprogram did.
            if let Some(facts) = facts.filter(|f| os.is_none() && !f.programs.is_empty()) {
                let key = crate::document::path_key(&env.fs.canonical(root));
                if self.recorded.get(&key) != Some(&facts) {
                    self.recorded.insert(key, facts);
                    self.recorded_version += 1;
                }
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
            self.entries.push(Analysis {
                compiler,
                entry: Some(root.to_path_buf()),
                key,
                quick: Some((quick, given)),
                os,
                root: root.to_path_buf(),
                dirs: dirs.to_vec(),
                files: snapshot,
                usable,
                cost,
                complete: errors.is_empty(),
                errors,
                lints: BTreeMap::new(),
                imports: None,
                found: RefCell::default(),
                others,
            });
        }
        self.entries.last_mut().expect("just pushed")
    }
}
