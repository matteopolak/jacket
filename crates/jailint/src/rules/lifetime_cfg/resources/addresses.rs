//! Exposure proofs use typed argument addresses, preserving distinct offset-zero fields.
use super::{
    Cx, EffectKind, Event, HashSet, LifetimeBody, LifetimeOperand, Places, Prepared, Resolver,
    Root, Summaries,
};

pub(super) fn exposed(
    cx: &Cx,
    body: &LifetimeBody,
    prepared: &Prepared<'_>,
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
    let mut resolver = Resolver::for_prepared(cx.compiler, body, prepared, summaries);
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

/// Only the expansion root shares its caller lifetime; explicit nested blocks do not.
pub(super) fn storage_scope(
    body: &jaic::ide::lifetimes::LifetimeBody,
    mut scope: jaic::sema::ScopeId,
) -> jaic::sema::ScopeId {
    for _ in 0..64 {
        let Some((_, caller)) = body.macro_owners.iter().find(|(root, _)| *root == scope) else {
            break;
        };
        scope = *caller;
    }
    scope
}

// A dynamic element load can select a different allocation. Preserve a dead
// possibility for strict mode without promoting an unknown element to a must alias.
pub(super) fn loaded_state(model: &super::Model, source: usize, value: u8) -> u8 {
    if value & super::DEAD != 0
        && let super::Cell::Place(place) = model.cells.values[source]
        && model.places.paths[place].1.iter().any(|p| {
            matches!(
                p,
                crate::rules::lifetime_places::Projection::DynamicElement(_)
            )
        })
    {
        value | super::UNKNOWN
    } else {
        value
    }
}

pub(super) fn seed_state(op: &super::CfgOp) -> u8 {
    if matches!(
        op,
        super::CfgOp::Resource(super::Op::Seed {
            possible: true,
            ..
        })
    ) {
        super::LIVE | super::UNKNOWN
    } else {
        super::LIVE
    }
}
