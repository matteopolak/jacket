//! Contracts belong to the actual lexical callable, never to an equal ABI type.
use super::{Effect, Event, HashMap, Kind, LifetimeBody, Path, Root, Summary};

pub(super) fn collect(body: &LifetimeBody) -> HashMap<*const Event, Summary> {
    let mut out = HashMap::default();
    for pair in body.blocks.iter().flat_map(|b| b.events.windows(2)) {
        let Event::CallContract {
            callee,
            declaration,
        } = &pair[0]
        else {
            continue;
        };
        if !matches!(&pair[1], Event::IndirectCall {callee:target,..} if target==callee) {
            continue;
        }
        let mut summary = Summary::default();
        let mut recognized = false;
        for note in &declaration.notes {
            if note.is("NoLifetimeEffects") {
                recognized = true;
            }
            let Some(args) = super::lifetime_movement::parameters(&note.text, "FreesArgument")
                .or_else(|| super::lifetime_movement::parameters(&note.text, "Closes"))
            else {
                continue;
            };
            let formal = if args.is_empty() {
                0
            } else {
                let Ok(n) = args[0].parse() else {
                    continue;
                };
                n
            };
            let formals = declaration
                .params
                .iter()
                .filter(|p| !p.baked && (!declaration.uses_c_abi() || !p.variadic))
                .count();
            if formal >= formals {
                continue;
            }
            recognized = true;
            let effect = Effect {
                target: Path {
                    root: Root::Parameter(formal),
                    steps: Vec::new(),
                },
                kind: Kind::Free,
                guard: None,
            };
            summary.may.insert(effect.clone(), vec![note.span]);
            summary.guaranteed.insert(effect, vec![note.span]);
        }
        if recognized {
            out.insert(std::ptr::from_ref(&pair[1]), summary);
        }
    }
    out
}
