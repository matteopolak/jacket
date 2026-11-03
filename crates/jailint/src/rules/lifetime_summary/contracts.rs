//! Explicit contracts seed inference; stronger return contracts take precedence over bodies.
use super::{
    Compiler, Effect, HashMap, HashSet, Kind, LifetimeBody, Path, ProcId, Root, Step, Summaries,
    Summary,
};

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
        let mut summary = Summary::default();
        if valid.contains_key(&id) {
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
            for index in super::lifetime_movement::result_indices(compiler, id, note) {
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
    all
}
