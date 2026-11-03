//! Exposure proofs use typed argument addresses, preserving distinct offset-zero fields.
use super::{
    Cx, EffectKind, Event, HashSet, LifetimeBody, LifetimeOperand, Places, Resolver, Root,
    Summaries,
};

pub(super) fn exposed(
    cx: &Cx,
    body: &LifetimeBody,
    summaries: &Summaries,
    places: &mut Places,
) -> HashSet<usize> {
    let mut safe_addresses = HashSet::default();
    for e in body.blocks.iter().flat_map(|b| &b.events) {
        if let Event::Call {
            callee,
            args,
            ..
        } = e
            && let Some(summary) = summaries.procs.get(callee)
        {
            for arg in args {
                if !summary.unknown_write.contains(&arg.formal)
                    && !summary.escapes.contains(&arg.formal)
                    && summary.guaranteed.keys().any(|e| {
                        e.kind == EffectKind::Free && e.target.root == Root::Parameter(arg.formal)
                    })
                    && let LifetimeOperand::Value(value) = arg.operand
                {
                    safe_addresses.insert((value, arg.span));
                }
            }
        }
    }
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        if let Event::IndirectCall {
            callee,
            args,
            allocator: false,
            ..
        } = event
        {
            let targets = summaries.targets.get(body.procedure, *callee);
            for arg in args {
                if (!targets.unknown || cx.lifetime_mode == jacket_config::LifetimeMode::Strict)
                    && let LifetimeOperand::Value(v) = arg.operand
                {
                    safe_addresses.insert((v, arg.span));
                }
            }
        }
        if cx.lifetime_mode == jacket_config::LifetimeMode::Strict
            && let Event::Call {
                args, ..
            } = event
        {
            for arg in args {
                if let LifetimeOperand::Value(v) = arg.operand {
                    safe_addresses.insert((v, arg.span));
                }
            }
        }
    }
    let mut read_only_addresses = HashSet::default();
    let mut uncertain_addresses = HashSet::default();
    let mut resolver = Resolver::new(cx.compiler, body, &summaries.procs, &summaries.targets);
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        let (Event::Call {
            args,
            expands: false,
            ..
        }
        | Event::IndirectCall {
            args, ..
        }) = event
        else {
            continue;
        };
        let targets = resolver.call_targets(event);
        for arg in args {
            let LifetimeOperand::Value(v) = arg.operand else {
                continue;
            };
            let preserved = !targets.unknown
                && targets.procedures.iter().all(|p| {
                    summaries
                        .procs
                        .get(p)
                        .is_some_and(|s| s.preserves_address(arg.formal))
                });
            let mut origins = Vec::new();
            origins.extend(resolver.object(arg));
            origins.extend(resolver.stored_address(v, &mut HashSet::default()));
            for p in origins {
                let p = places.import(&resolver.places, p);
                if preserved {
                    read_only_addresses.insert(p);
                } else {
                    uncertain_addresses.insert(p);
                }
            }
        }
    }
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| match e {
            Event::Address {
                place,
                ty,
                span,
            } if !safe_addresses.contains(&(*place, *span)) => {
                let p = places.place(*place, *ty);
                (!read_only_addresses.contains(&p) || uncertain_addresses.contains(&p)).then_some(p)
            }
            _ => None,
        })
        .collect()
}
