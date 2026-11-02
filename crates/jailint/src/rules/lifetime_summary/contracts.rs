//! Explicit contracts seed inference; stronger return contracts take precedence over bodies.
use super::{
    Compiler, Effect, HashMap, Kind, LifetimeBody, Path, ProcId, Root, Step, Summaries, Summary,
};

pub(super) fn seed(compiler: &Compiler, valid: &HashMap<ProcId, &LifetimeBody>) -> Summaries {
    let mut all = Summaries::default();
    for (i, _) in compiler.pr.procs.iter().enumerate() {
        let id = ProcId(i as u32);
        let mut summary = Summary::default();
        if valid.contains_key(&id) {
            summary.normal_return = Some(false);
        }
        for (note, kind, storage) in [
            ("GrowsArray", Kind::Resize, true),
            ("FreesArgument", Kind::Free, false),
            ("Closes", Kind::Free, false),
        ] {
            if compiler.proc_has_note(id, note) {
                let effect = Effect {
                    target: Path {
                        root: Root::Parameter(0),
                        steps: if storage {
                            vec![Step::Storage]
                        } else {
                            Vec::new()
                        },
                    },
                    kind,
                    guard: None,
                };
                summary
                    .may
                    .insert(effect.clone(), vec![compiler.pr.procs[i].span]);
                summary
                    .guaranteed
                    .insert(effect, vec![compiler.pr.procs[i].span]);
            }
        }
        for (note, root) in [
            ("Allocates", Root::Heap),
            ("TempAllocates", Root::Temporary),
        ] {
            if compiler.proc_has_note(id, note) {
                summary.returns.insert(
                    0,
                    (
                        Path {
                            root,
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
    all
}
