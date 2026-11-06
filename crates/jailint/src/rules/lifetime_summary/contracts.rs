//! Explicit contracts seed inference; checked return origins refine allocation notes.
use super::{
    Compiler, Effect, HashMap, HashSet, Kind, LifetimeBody, Path, ProcId, Root, Step, Summaries,
    Summary,
};

mod unpassed;
pub(in crate::rules) use unpassed::UnpassedStorage;

pub(super) fn seed(
    compiler: &Compiler,
    valid: &HashMap<ProcId, &LifetimeBody>,
    needed: Option<&HashSet<ProcId>>,
) -> Summaries {
    let mut all = Summaries::default();
    let mut ids: Vec<_> = needed.map_or_else(
        || {
            (0..compiler.pr.procs.len())
                .map(|i| ProcId(i as u32))
                .collect()
        },
        |ids| ids.iter().copied().collect(),
    );
    ids.sort_unstable_by_key(|p| p.0);
    for id in ids {
        let i = id.0 as usize;
        let mut summary = Summary {
            observable: super::observable::primitive(compiler, id),
            pushes_context: super::templates::pushes_context(&compiler.pr.procs[i].lit),
            ordering: super::ordering::scalar(&compiler.pr.procs[i].lit),
            ..Default::default()
        };
        if compiler.pr.procs[i].lit.body.is_none() {
            summary.ordering = if compiler.proc_has_note(id, "ReturnsSmaller") {
                Some(super::ordering::Ordering::Minimum)
            } else if compiler.proc_has_note(id, "ReturnsLarger") {
                Some(super::ordering::Ordering::Maximum)
            } else {
                None
            };
            summary.observable.output |= compiler.proc_has_note(id, "WritesOutput");
            summary.observable.exit |= compiler.proc_has_note(id, "Exits");
        }
        super::primitives::seed(compiler, id, &mut summary);
        summary.locks = super::locks::native(compiler, id);
        if valid.contains_key(&id) || summary.observable.exit {
            summary.normal_return = Some(false);
        }
        if compiler.proc_has_note(id, "ValidUntilNextCall") {
            summary.returns.insert(
                0,
                (
                    Path {
                        root: Root::Until(id),
                        steps: Vec::new(),
                    },
                    None,
                ),
            );
            let effect = Effect {
                target: Path {
                    root: Root::Until(id),
                    steps: Vec::new(),
                },
                kind: Kind::Advance,
                guard: None,
            };
            summary
                .may
                .insert(effect.clone(), vec![compiler.pr.procs[i].span]);
            summary
                .guaranteed
                .insert(effect, vec![compiler.pr.procs[i].span]);
        }
        for (formal, mutation) in super::lifetime_movement::contracts(compiler, id) {
            let effect = Effect {
                target: Path {
                    root: Root::Parameter(formal),
                    steps: vec![Step::Storage],
                },
                kind: Kind::Mutation(mutation),
                guard: None,
            };
            summary
                .may
                .insert(effect.clone(), vec![compiler.pr.procs[i].span]);
            if mutation.guaranteed() {
                summary
                    .guaranteed
                    .insert(effect, vec![compiler.pr.procs[i].span]);
            }
        }
        if compiler.proc_has_note(id, "GrowsArray") {
            let effect = Effect {
                target: Path {
                    root: Root::Parameter(0),
                    steps: vec![Step::Storage],
                },
                kind: Kind::Resize,
                guard: None,
            };
            summary
                .may
                .insert(effect.clone(), vec![compiler.pr.procs[i].span]);
            summary
                .guaranteed
                .insert(effect, vec![compiler.pr.procs[i].span]);
        }
        if let Some(formal) = super::lifetime_movement::released_formal(compiler, id) {
            let effect = Effect {
                target: Path {
                    root: Root::Parameter(formal),
                    steps: Vec::new(),
                },
                kind: Kind::Free,
                guard: None,
            };
            summary
                .may
                .insert(effect.clone(), vec![compiler.pr.procs[i].span]);
            summary
                .guaranteed
                .insert(effect, vec![compiler.pr.procs[i].span]);
        }
        for (formal, key, result) in super::lifetime_movement::borrowed(compiler, id) {
            let path = Path {
                root: Root::Parameter(formal),
                steps: vec![Step::Storage],
            };
            summary.returns.insert(result, (path.clone(), None));
            summary.elements.insert(result, (path, key));
        }
        for (note, root) in [
            ("Allocates", Root::Heap),
            ("TempAllocates", Root::Temporary),
        ] {
            if note == "Allocates" && compiler.pr.procs[i].lit.body.is_some() {
                continue;
            }
            let indices = super::lifetime_movement::result_indices(compiler, id, note);
            if compiler.pr.procs[i].lit.body.is_none() && !indices.is_empty() {
                summary.allocation_effects.insert((root.clone(), None));
            }
            for index in indices {
                summary.returns.insert(
                    index,
                    (
                        Path {
                            root: root.clone(),
                            steps: Vec::new(),
                        },
                        None,
                    ),
                );
            }
        }
        if compiler.proc_has_note(id, "TemporaryStorageMark") {
            summary.returns.insert(
                0,
                (
                    Path {
                        root: Root::Watermark,
                        steps: Vec::new(),
                    },
                    None,
                ),
            );
        }
        for (note, kind, root) in [
            (
                "ResetsTemporaryStorage",
                Kind::ResetTemporary,
                Root::Temporary,
            ),
            (
                "RestoresTemporaryStorage",
                Kind::RestoreTemporary,
                Root::Parameter(0),
            ),
        ] {
            if compiler.proc_has_note(id, note) {
                let e = Effect {
                    target: Path {
                        root,
                        steps: Vec::new(),
                    },
                    kind,
                    guard: None,
                };
                summary
                    .may
                    .insert(e.clone(), vec![compiler.pr.procs[i].span]);
                summary
                    .guaranteed
                    .insert(e, vec![compiler.pr.procs[i].span]);
            }
        }
        all.procs.insert(id, summary);
    }
    for body in valid.values() {
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            if let super::Event::Call {
                callee,
                memory: Some(memory),
                ..
            } = event
            {
                let summary = all.procs.entry(*callee).or_default();
                summary.memory = Some(*memory);
                for formal in 0..8 {
                    if memory.writes & (1 << formal) != 0 {
                        summary.may.insert(
                            Effect {
                                target: Path {
                                    root: Root::Parameter(formal),
                                    steps: Vec::new(),
                                },
                                kind: Kind::Write,
                                guard: None,
                            },
                            vec![compiler.pr.procs[callee.0 as usize].span],
                        );
                    }
                    if (memory.reads | memory.writes) & (1 << formal) != 0 {
                        summary.may_reads.insert(Path {
                            root: Root::Parameter(formal),
                            steps: Vec::new(),
                        });
                    }
                }
            }
        }
    }
    // Freeze after every semantic seed pass, including selected call contracts.
    for (&id, summary) in &mut all.procs {
        summary.trusted = Trusted::capture(compiler, id, summary).map(Box::new);
        summary.external_preservation =
            UnpassedStorage::capture(compiler, id, summary).map(std::rc::Rc::new);
        if jacket_config::env::LIFETIME_STATS.is_set()
            && let Some(proof) = &summary.external_preservation
        {
            eprintln!(
                "lifetime-external-preservation proc={} payload_bytes={} auth_work={}",
                id.0,
                proof.payload_bytes(),
                proof.authentication_work()
            );
        }
    }
    all
}


/// Independent declaration/native constraints. None on Summary means known
/// absence, not purity. Opaque is absorbing and never accepts a partial prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Trusted {
    Effects(Vec<Effect>),
    Opaque,
}

impl Trusted {
    const MAX_EFFECTS: usize = 32;

    fn capture(c: &Compiler, id: ProcId, s: &Summary) -> Option<Self> {
        // These semantic seeds cannot be represented as a memory-only frame.
        if !s.returns.is_empty()
            || !s.elements.is_empty()
            || !s.allocation_effects.is_empty()
            || s.native_allocation.is_some()
            || s.locks.is_some()
            || s.observable.output
            || s.observable.exit
            || super::locks::declared(c, id, "Locks").is_some()
            || super::locks::declared(c, id, "Unlocks").is_some()
            || c.proc_has_note(id, "Opens")
        {
            return Some(Self::Opaque);
        }
        Self::effects(s.may.keys().chain(s.guaranteed.keys()))
    }

    /// Bound count, path shape and total cloning work before collecting anything.
    /// Only seed roots with bounded inline payloads are admitted.
    pub(super) fn effects<'a>(effects: impl Iterator<Item = &'a Effect> + Clone) -> Option<Self> {
        let mut count = 0usize;
        let mut work = 0usize;
        for e in effects.clone() {
            count = count.saturating_add(1);
            work = work.saturating_add(e.target.steps.len() + 1);
            if count > Self::MAX_EFFECTS
                || work > Self::MAX_EFFECTS * (super::MAX_PATH + 1)
                || e.target.steps.len() > super::MAX_PATH
                || !matches!(
                    e.target.root,
                    Root::Parameter(_) | Root::Temporary | Root::Until(_)
                )
            {
                return Some(Self::Opaque);
            }
        }
        if count == 0 {
            return None;
        }
        let mut out = Vec::with_capacity(count);
        for e in effects {
            if !out.contains(e) {
                out.push(e.clone());
            }
        }
        Some(Self::Effects(out))
    }

    pub(super) fn work(&self) -> usize {
        match self {
            Self::Effects(effects) => effects.len(),
            Self::Opaque => 1,
        }
    }

    /// The append witness covers only its own unguarded backing Resize.
    pub(super) fn covered_by_append(&self, receiver: usize) -> bool {
        matches!(self, Self::Effects(effects) if effects.iter().all(|e|
            e.kind == Kind::Resize && e.guard.is_none()
            && e.target.root == Root::Parameter(receiver)
            && e.target.steps == [Step::Storage]))
    }
}
