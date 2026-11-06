//! Exposure proofs use typed argument addresses, preserving distinct offset-zero fields.
use super::{
    CfgOp, Cx, EffectKind, Event, HashSet, LifetimeBody, LifetimeOperand, Places, Prepared,
    Resolver, Root, Summaries,
};

pub(super) fn exposed(
    cx: &Cx,
    body: &LifetimeBody,
    prepared: &Prepared<'_>,
    summaries: &Summaries,
    places: &mut Places,
    borrows: &crate::rules::lifetime_summary::no_growth::calls::Calls,
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
    let scoped_stores: Vec<_> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|event| {
            let Event::Store {
                place,
                value,
                span,
                ..
            } = *event
            else {
                return None;
            };
            body.globals
                .iter()
                .any(|(v, global)| {
                    *v == place && summaries.targets.scoped_pointers.contains(global)
                })
                .then_some((value, span))
        })
        .collect();
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        if let Event::Address {
            place,
            span,
            ..
        } = *event
            && scoped_stores.iter().any(|(value, store)| {
                place == *value
                    && store.file == span.file
                    && store.start <= span.start
                    && span.end <= store.end
            })
        {
            safe_addresses.insert((place, span));
        }
    }
    for (block, b) in body.blocks.iter().enumerate() {
        for (ordinal, e) in b.events.iter().enumerate() {
            if let Event::Call {
                args, ..
            } = e
            {
                for arg in args {
                    if borrows.preserves(
                        crate::rules::lifetime_movement::facts::Site {
                            block,
                            event: ordinal,
                        },
                        arg,
                    ) && let LifetimeOperand::Value(v) = arg.operand
                    {
                        safe_addresses.insert((v, arg.span));
                    }
                }
            }
        }
    }
    let mut preserved_addresses = HashSet::default();
    let mut uncertain_addresses = HashSet::default();
    let mut resolver = Resolver::for_prepared(prepared, summaries);
    for (at, event) in body.blocks.iter().enumerate().flat_map(|(block, b)| {
        b.events.iter().enumerate().map(move |(event, e)| {
            (
                crate::rules::lifetime_movement::facts::Site {
                    block,
                    event,
                },
                e,
            )
        })
    }) {
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
            let preserved = borrows.preserves(at, arg)
                || !targets.unknown
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
                    preserved_addresses.insert(p);
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
                (!preserved_addresses.contains(&p) || uncertain_addresses.contains(&p)).then_some(p)
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


pub(super) fn define(state: &mut [u8], cell: Option<usize>) {
    if let Some(cell) = cell {
        state[cell] = super::UNKNOWN;
    }
}


impl super::Builder<'_> {
    /// One body-local budget for optional exact store-location captures. A failed
    /// scope or query cannot change ordinary resource transfer or eligibility.
    pub(super) fn capture_location(
        &self,
        r: &Resolver<'_>,
        value: jaic::ir::Val,
        ty: jaic::types::TypeId,
    ) -> Option<Box<(jaic::ir::Val, crate::rules::lifetime_facts::ByteRange)>> {
        let mut work = self.capture_work.get();
        if self.capture_scope.get().is_none() {
            self.capture_scope
                .set(Some(r.retention_capture_scope(&mut work).is_some()));
        }
        let result = if self.capture_scope.get() == Some(true) {
            if self.capture_index.borrow().is_none() {
                let index = r.capture_index(&mut work);
                *self.capture_index.borrow_mut() = Some(index);
            }
            self.capture_index
                .borrow()
                .as_ref()
                .and_then(Option::as_ref)
                .and_then(|index| index.write_location(r, value, ty, &mut work, 0))
                .map(Box::new)
        } else {
            None
        };
        self.capture_work.set(work);
        result
    }
}


/// Exact load-epoch equality can fill an unknown peer, never erase a known
/// invalidation. Conflicting known alternatives remain a conservative union.
pub(super) fn captured_state(memory: u8, captured: Option<u8>) -> u8 {
    match captured {
        None => memory,
        Some(value) if value == super::UNKNOWN => memory,
        Some(value) if memory == super::UNKNOWN => value,
        Some(value) => value | memory,
    }
}


/// Resource relevance includes both peers of a proven load equality. These are
/// dependency edges, not a claim that a mutable slot keeps that value later.
pub(super) fn value_edges(cells: &super::Cells, blocks: &[super::Block]) -> Vec<Vec<usize>> {
    let mut forward = vec![Vec::new(); cells.ids.len()];
    for block in blocks {
        match &block.op {
            CfgOp::Copy(pairs) => {
                for &(target, source) in pairs {
                    forward[source].push(target);
                }
            }
            CfgOp::PointerLoad(target, source, _, _, captured) => {
                forward[*source].push(*target);
                if let Some(captured) = captured {
                    forward[*captured].push(*target);
                    forward[*target].push(*captured);
                }
            }
            _ => {}
        }
    }
    forward
}

/// Transfer one pointer load and its optional exact SSA peer at this event.
/// For example, a later slot overwrite cannot alter an already captured value.
pub(super) fn load_pointer(
    model: &super::Model,
    (target, source, captured): (usize, usize, Option<usize>),
    state: &mut [u8],
    index: &impl Fn(usize) -> Option<usize>,
    initial: bool,
    born: usize,
) {
    if let Some(target) = index(target) {
        let peer = captured.and_then(&index);
        let value = loaded_state(
            model,
            source,
            captured_state(
                index(source).map_or(super::UNKNOWN, |s| state[s]),
                peer.map(|s| state[s]),
            ),
        );
        state[target] = if initial && value == super::UNKNOWN {
            super::LIVE
        } else {
            value
        };
        if initial || peer.is_some() {
            if let Some(source) = index(source) {
                state[source] = state[target];
            }
            if let Some(peer) = peer {
                state[peer] = state[target];
            }
        }
        if initial {
            state[born] = super::LIVE;
        }
    }
}

#[cfg(test)]
mod capture_tests {
    use super::captured_state;
    use crate::rules::lifetime_cfg::{DEAD, LIVE, UNKNOWN};

    #[test]
    fn equality_at_a_load_never_revives_a_known_dead_pointer() {
        assert_eq!(captured_state(DEAD, Some(UNKNOWN)), DEAD);
        assert_eq!(captured_state(UNKNOWN, Some(DEAD)), DEAD);
        assert_eq!(captured_state(LIVE, Some(DEAD)), LIVE | DEAD);
        assert_eq!(
            captured_state(DEAD | UNKNOWN, Some(LIVE)),
            LIVE | DEAD | UNKNOWN
        );
        assert_eq!(captured_state(UNKNOWN, None), UNKNOWN);
    }
}
