//! Bounded exact build-discovery outcomes, independent of full compiler ownership.
use super::{Analysis, Buffers, BuildFacts, CheckedInputs, Environment, Reads};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::rc::Rc;

const BUDGET: usize = 16 << 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProbeStatus {
    Complete,
    Failed,
}


struct Probe {
    entry: PathBuf,
    input: CheckedInputs,
    /// `None` is a completed failed attempt, never a proof that no child is built.
    facts: Option<Rc<BuildFacts>>,
    complete: bool,
}

impl Probe {
    fn status(&self) -> ProbeStatus {
        if self.complete {
            ProbeStatus::Complete
        } else {
            ProbeStatus::Failed
        }
    }
}

pub(crate) struct ProbeCapture {
    input: CheckedInputs,
    facts: Option<Rc<BuildFacts>>,
    complete: bool,
}

impl ProbeCapture {
    pub(crate) fn new(analysis: &Analysis) -> Self {
        Self {
            input: CheckedInputs::capture(analysis),
            facts: analysis.build_facts.clone().map(Rc::new),
            complete: analysis.build_complete,
        }
    }
}

fn facts_bytes(facts: &BuildFacts) -> usize {
    std::mem::size_of::<BuildFacts>()
        + facts.programs.capacity() * std::mem::size_of::<super::BuiltProgram>()
        + facts
            .programs
            .iter()
            .map(|program| {
                (program.files.capacity() + program.import_dirs.capacity())
                    * std::mem::size_of::<PathBuf>()
                    + program
                        .files
                        .iter()
                        .chain(&program.import_dirs)
                        .map(PathBuf::capacity)
                        .sum::<usize>()
            })
            .sum::<usize>()
}

pub(crate) struct Probes {
    budget: usize,
    retained: VecDeque<Probe>,
    /// At most one oversized current result. The session drops it at request exit.
    ephemeral: Option<Probe>,
    /// Last-good metadata is advisory only; no old input proof becomes current authority.
    advisory: VecDeque<(PathBuf, Rc<BuildFacts>)>,
}

impl Default for Probes {
    fn default() -> Self {
        Self {
            budget: BUDGET,
            retained: VecDeque::new(),
            ephemeral: None,
            advisory: VecDeque::new(),
        }
    }
}

fn probe_storage(probe: &Probe) -> usize {
    // Includes the persistent current membership/status projections in Cache and Index.
    std::mem::size_of::<Probe>() + 6 * probe.entry.capacity() + 512 + probe.input.bytes
}

fn advisory_storage(entry: &PathBuf) -> usize {
    std::mem::size_of::<(PathBuf, Rc<BuildFacts>)>() + 4 * entry.capacity() + 256
}

impl Probes {
    pub(crate) fn bytes(&self) -> usize {
        let mut facts_seen = BTreeSet::new();
        let facts_size = |facts: &Rc<BuildFacts>, seen: &mut BTreeSet<usize>| {
            if seen.insert(Rc::as_ptr(facts) as usize) {
                facts_bytes(facts)
            } else {
                0
            }
        };
        self.retained
            .iter()
            .map(|probe| {
                probe_storage(probe)
                    + probe
                        .facts
                        .as_ref()
                        .map_or(0, |facts| facts_size(facts, &mut facts_seen))
            })
            .sum::<usize>()
            + self
                .advisory
                .iter()
                .map(|(entry, facts)| advisory_storage(entry) + facts_size(facts, &mut facts_seen))
                .sum::<usize>()
    }

    fn trim(&mut self) {
        let mut bytes = self.bytes();
        if bytes <= self.budget {
            return;
        }
        let mut owners: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
        for facts in self
            .retained
            .iter()
            .filter_map(|p| p.facts.as_ref())
            .chain(self.advisory.iter().map(|(_, f)| f))
        {
            let entry = owners
                .entry(Rc::as_ptr(facts) as usize)
                .or_insert_with(|| (0, facts_bytes(facts)));
            entry.0 += 1;
        }
        let release = |facts: &Rc<BuildFacts>, owners: &mut BTreeMap<usize, (usize, usize)>| {
            let entry = owners
                .get_mut(&(Rc::as_ptr(facts) as usize))
                .expect("counted owner");
            entry.0 -= 1;
            if entry.0 == 0 {
                entry.1
            } else {
                0
            }
        };
        while bytes > self.budget {
            if let Some(probe) = self.retained.pop_front() {
                bytes -= probe_storage(&probe)
                    + probe
                        .facts
                        .as_ref()
                        .map_or(0, |facts| release(facts, &mut owners));
            } else if let Some((entry, facts)) = self.advisory.pop_front() {
                bytes -= advisory_storage(&entry) + release(&facts, &mut owners);
            } else {
                break;
            }
        }
        debug_assert_eq!(bytes, self.bytes());
    }

    pub(crate) fn counts(&self) -> (usize, usize, usize) {
        (
            self.retained.len(),
            self.bytes(),
            self.ephemeral.as_ref().map_or(0, |p| {
                probe_storage(p) + p.facts.as_ref().map_or(0, |f| facts_bytes(f))
            }),
        )
    }

    #[cfg(test)]
    pub(crate) fn set_budget(&mut self, budget: usize) {
        self.budget = budget;
        self.trim();
    }

    pub(crate) fn restore_advisory(&mut self, facts: BTreeMap<PathBuf, Rc<BuildFacts>>) {
        self.advisory = facts.into_iter().collect();
        self.trim();
    }

    pub(crate) fn clear_exact(&mut self) {
        self.retained.clear();
        self.ephemeral = None;
    }

    /// Known events revoke exact captured histories; last-good metadata stays advisory.
    pub(crate) fn invalidate_path(
        &mut self,
        path: &std::path::Path,
        current: Option<&super::input_epoch::ActualInputs>,
    ) -> bool {
        let before = self.retained.len();
        self.retained
            .retain(|probe| !probe.input.concerns_tree_since(path, current));
        let ephemeral_changed = self
            .ephemeral
            .as_ref()
            .is_some_and(|probe| probe.input.concerns_tree_since(path, current));
        if ephemeral_changed {
            self.ephemeral = None;
        }
        self.retained.len() != before || ephemeral_changed
    }

    pub(crate) fn end_request(&mut self) -> bool {
        self.ephemeral.take().is_some()
    }

    pub(crate) fn validate(
        &mut self,
        env: &Environment,
        files: &Buffers,
        authority: &Rc<()>,
    ) -> bool {
        let before = self.retained.len();
        self.retained
            .retain(|probe| probe.input.is_current(env, files, authority));
        let ephemeral_invalid = self
            .ephemeral
            .as_ref()
            .is_some_and(|p| !p.input.is_current(env, files, authority));
        if ephemeral_invalid {
            self.ephemeral = None;
        }
        self.retained.len() != before || ephemeral_invalid
    }

    pub(crate) fn remember(&mut self, entry: PathBuf, analysis: &Analysis) {
        self.remember_capture(entry, ProbeCapture::new(analysis));
    }

    pub(crate) fn remember_capture(&mut self, entry: PathBuf, capture: ProbeCapture) {
        let ProbeCapture {
            input,
            facts,
            complete,
        } = capture;
        if let Some(facts) = facts
            .as_ref()
            .filter(|facts| complete && !facts.programs.is_empty())
        {
            self.advisory.retain(|(path, _)| path != &entry);
            self.advisory.push_back((entry.clone(), facts.clone()));
        }
        self.retained.retain(|probe| probe.entry != entry);
        if self.ephemeral.as_ref().is_some_and(|p| p.entry == entry) {
            self.ephemeral = None;
        }
        let probe = Probe {
            entry,
            input,
            facts,
            complete,
        };
        if probe_storage(&probe) + probe.facts.as_ref().map_or(0, |f| facts_bytes(f)) > self.budget
        {
            self.ephemeral = Some(probe);
        } else {
            self.retained.push_back(probe);
        }
        self.trim();
    }

    fn all(&self) -> impl Iterator<Item = &Probe> {
        self.retained.iter().chain(self.ephemeral.iter())
    }

    pub(crate) fn facts(&self) -> BTreeMap<PathBuf, Rc<BuildFacts>> {
        self.all()
            .filter_map(|probe| {
                let facts = probe.facts.as_ref()?;
                (!facts.programs.is_empty()).then(|| (probe.entry.clone(), facts.clone()))
            })
            .collect()
    }

    pub(crate) fn statuses(&self) -> BTreeMap<PathBuf, ProbeStatus> {
        self.all()
            .map(|probe| (probe.entry.clone(), probe.status()))
            .collect()
    }

    pub(crate) fn advisory(&self) -> BTreeMap<PathBuf, Rc<BuildFacts>> {
        self.advisory
            .iter()
            .map(|(entry, facts)| (entry.clone(), facts.clone()))
            .collect()
    }

    pub(crate) fn inputs_for(&self, entries: &[PathBuf]) -> Vec<Rc<Reads>> {
        self.all()
            .filter(|probe| entries.contains(&probe.entry))
            .map(|probe| probe.input.reads.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn environment() -> Environment {
        Environment {
            fs: Rc::new(jaic::sema::NativeFs),
            services: None,
            options: Box::new(|_, _| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        }
    }

    #[test]
    fn watched_paths_retire_only_affected_retained_and_ephemeral_probe_authority() {
        let temp = test_support::TempDir::new("probe-event-retirement");
        let first = temp.path().join("first/build.jai");
        let second = temp.path().join("second/build.jai");
        for root in [&first, &second] {
            std::fs::create_dir_all(root.parent().unwrap()).unwrap();
            std::fs::write(root, "#run {}").unwrap();
        }
        let files: Buffers = Rc::new(
            [first.clone(), second.clone()]
                .into_iter()
                .map(|path| (path, Arc::from(&b"#run {}"[..])))
                .collect(),
        );
        let env = environment();
        let mut cache = super::super::Cache::default();
        for root in [&first, &second] {
            assert_eq!(
                cache.probe(&env, root, &[], files.clone()),
                Some(ProbeStatus::Complete)
            );
        }
        let second_key = jaic::paths::file_identity(&*env.fs, &second);
        std::fs::write(&first, "#run { changed := 1; }").unwrap();
        cache.invalidate_path(first.parent().unwrap());
        assert_eq!(
            cache.probe_statuses(),
            [(second_key.clone(), ProbeStatus::Complete)].into(),
            "ancestor events retire affected proof, not independent probe ownership"
        );
        assert!(
            !cache.validate_probes(&env, &files),
            "the surviving probe is actually current"
        );
        let before = cache.counts().0;
        assert_eq!(
            cache.probe(&env, &second, &[], files.clone()),
            Some(ProbeStatus::Complete)
        );
        assert_eq!(
            cache.counts().0,
            before,
            "the unaffected producer remains useful without recompiling"
        );
        cache.set_probe_budget(0);
        assert_eq!(
            cache.probe(&env, &first, &[], files),
            Some(ProbeStatus::Complete)
        );
        assert!(cache.probe_counts().2 > 0);
        cache.invalidate_path(&second);
        assert!(
            cache.probe_counts().2 > 0,
            "an unrelated event preserves the ephemeral current request"
        );
        cache.invalidate_path(first.parent().unwrap());
        assert_eq!(cache.probe_counts(), (0, 0, 0));
    }

    #[test]
    fn negative_pressure_retirement_and_oversized_request_are_distinct() {
        let temp = test_support::TempDir::new("probe-pressure");
        let root = temp.path().join("build.jai");
        let files: Buffers = Rc::new([(root.clone(), Arc::from(&b"#run {}"[..]))].into());
        let env = environment();
        let mut cache = super::super::Cache::default();
        assert_eq!(
            cache.probe(&env, &root, &[], files.clone()),
            Some(ProbeStatus::Complete)
        );
        assert_eq!(cache.probe_counts().0, 1);
        cache.set_probe_budget(0);
        assert!(cache.probe_statuses().is_empty());
        assert!(cache.recorded.is_empty());
        assert_eq!(cache.probe_counts().1, 0);
        assert_eq!(
            cache.probe(&env, &root, &[], files),
            Some(ProbeStatus::Complete)
        );
        assert_eq!(
            cache.probe_counts().0,
            0,
            "oversized result is not retained"
        );
        assert!(
            cache.probe_counts().2 > 0,
            "current request may consume accepted oversized proof"
        );
        assert!(cache.end_request());
        assert!(cache.probe_statuses().is_empty());
        assert_eq!(cache.probe_counts(), (0, 0, 0));
    }

    #[test]
    fn advisory_metadata_shares_the_probe_budget() {
        let mut probes = Probes::default();
        probes.set_budget(512);
        let huge = PathBuf::from("x".repeat(4096));
        probes.restore_advisory(
            [(
                PathBuf::from("build.jai"),
                Rc::new(BuildFacts {
                    programs: vec![super::super::BuiltProgram {
                        files: vec![huge],
                        ..Default::default()
                    }],
                }),
            )]
            .into(),
        );
        assert!(probes.advisory().is_empty());
        assert!(probes.bytes() <= 512);
    }

    #[test]
    fn accepted_proof_cannot_cross_environment_authority_changes() {
        let temp = test_support::TempDir::new("probe-authority");
        let root = temp.path().join("main.jai");
        let files: Buffers = Rc::new([(root.clone(), Arc::from(&b"main :: () {}"[..]))].into());
        let env = environment();
        let mut cache = super::super::Cache::default();
        let checked =
            CheckedInputs::capture(cache.analyze(&env, &root, &[], files.clone()).unwrap());
        assert!(checked.is_current(&env, &files, cache.authority()));
        cache.forget_compiles();
        assert!(
            checked.is_current(&env, &files, cache.authority()),
            "ordinary eviction does not change context"
        );
        cache.set_limits(jaic::ide::lint_compile::CheckLimits {
            hint_blocks: 1,
            ..Default::default()
        });
        assert!(!checked.is_current(&env, &files, cache.authority()));
        let checked =
            CheckedInputs::capture(cache.analyze(&env, &root, &[], files.clone()).unwrap());
        cache.set_trusted(false);
        assert!(!checked.is_current(&env, &files, cache.authority()));
    }
}
