//! Invocation-local native backing applicability, using existing CFG reachability.
use super::{Addresses, Backing, ByteRange, Event, HashSet, Resolver, Site, Summaries, Val};
use crate::rules::lifetime_summary::flow;

fn charge(work: &mut usize, amount: usize) -> Option<()> {
    *work = work.checked_add(amount)?;
    (*work <= super::MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
}

/// Authenticate, prove successful control flow, then frame the interval to use.
pub(super) fn backing(
    r: &Resolver<'_>,
    capture_index: &mut Option<Option<crate::rules::lifetime_summary::captured_values::QueryIndex>>,
    addresses: &Addresses,
    all: &Summaries,
    value: Val,
    at: Site,
    work: &mut usize,
) -> Option<Backing> {
    let body = r.body;
    let total = body.blocks.iter().try_fold(0usize, |n, b| {
        n.checked_add(b.events.len() + b.successors.len() + 1)
    })?;
    if total > 128 || body.comparisons.len() > 128 {
        return None;
    }
    charge(work, total.checked_add(body.comparisons.len())?)?;
    let result_count =
        body.blocks
            .iter()
            .flat_map(|b| &b.events)
            .try_fold(0usize, |n, e| match e {
                Event::Call {
                    results, ..
                }
                | Event::IndirectCall {
                    results, ..
                } => n.checked_add(results.len()),
                _ => Some(n),
            })?;
    if total.checked_add(result_count).is_none_or(|n| n > 128)
        || body.comparisons.len() > 128
        || body
            .blocks
            .iter()
            .any(|b| b.successors.iter().any(|&s| s >= body.blocks.len()))
    {
        return None;
    }
    charge(
        work,
        total
            .checked_add(result_count)?
            .checked_mul(3)?
            .checked_add(body.comparisons.len())?,
    )?;
    if capture_index.is_none() {
        let mut remaining = super::MAX_WORK.checked_sub(*work)?;
        let index = r.capture_index(&mut remaining);
        *work = super::MAX_WORK - remaining;
        *capture_index = Some(index);
    }
    let index = capture_index.as_ref()?.as_ref()?;
    let mut remaining = super::MAX_WORK.checked_sub(*work)?;
    let leaf = index.pointer_leaf(r, value, &mut remaining, 0);
    *work = super::MAX_WORK - remaining;
    let (root, _) = leaf?;
    let mut selected = None;
    for (event, e) in body.blocks.first()?.events.iter().enumerate() {
        let Event::Call {
            callee,
            args,
            results,
            expands: false,
            ..
        } = e
        else {
            continue;
        };
        if !results.iter().any(|(_, v, _)| *v == root) {
            continue;
        }
        let contract = all.procs.get(callee)?.native_allocation.as_ref()?;
        let signature = r.compiler.pr.procs.get(callee.0 as usize)?.sig.as_ref()?;
        if args.len() != signature.params.len()
            || args.len() > 8
            || results.len() != 1
            || results[0].0 != 0
            || results[0].2 != signature.returns[0]
            || selected.is_some()
        {
            return None;
        }
        charge(work, args.len().checked_mul(args.len() + 3)?)?;
        let mut inputs = Vec::with_capacity(args.len());
        for (formal, param) in signature.params.iter().enumerate() {
            let mut found = args.iter().filter(|arg| arg.formal == formal);
            let arg = found.next()?;
            if found.next().is_some() || arg.spread || arg.ty != param.ty {
                return None;
            }
            inputs.push(arg.constant);
        }
        let range = contract.apply_nonnull(r.compiler, &all.procs, (*callee, &inputs), work);
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-applicability proc={} callee={} inputs={:?} range={:?} work={}",
                body.procedure.0, callee.0, inputs, range, *work
            );
        }
        let range = range?;
        selected = Some(Backing {
            root,
            bytes: range.bytes,
            alignment: range.alignment,
            acquisition: Site {
                block: 0,
                event,
            },
        });
    }
    let backing = selected?;
    let mut guard = None;
    for (event, e) in body.blocks[0]
        .events
        .iter()
        .enumerate()
        .skip(backing.acquisition.event + 1)
    {
        let Event::Branch {
            condition,
            then_block,
            else_block,
            ..
        } = e
        else {
            continue;
        };
        if *then_block == *else_block
            || body.blocks[0].successors.len() != 2
            || !body.blocks[0].successors.contains(then_block)
            || !body.blocks[0].successors.contains(else_block)
        {
            return None;
        }
        let mut remaining = super::MAX_WORK.checked_sub(*work)?;
        let when = index.null_test(
            r,
            &r.core().storage_versions,
            *condition,
            root,
            &mut remaining,
            0,
        );
        *work = super::MAX_WORK - remaining;
        let when = when?;
        guard = Some((
            event,
            if when {
                *then_block
            } else {
                *else_block
            },
        ));
        break;
    }
    let (branch, success) = guard?;
    if body.blocks[0].events[..branch]
        .iter()
        .any(|e| matches!(e, Event::Branch { .. }))
        || at.block == 0
    {
        return None;
    }
    charge(work, total)?;
    let mut reachable = false;
    flow::states_except(
        body,
        &flow::Live::all(body),
        (),
        &HashSet::from_iter([(0, success)]),
        |(), ()| false,
        |(), site, _| {
            reachable |= site == at;
        },
    )?;
    if reachable {
        return None;
    }
    for (ordinal, event) in body.blocks[0]
        .events
        .iter()
        .enumerate()
        .take(branch)
        .skip(backing.acquisition.event + 1)
    {
        harmless(
            r,
            addresses,
            Site {
                block: 0,
                event: ordinal,
            },
            event,
            work,
        )?;
    }
    let mut seen = HashSet::default();
    let mut previous = 0;
    let mut block = success;
    loop {
        charge(work, body.blocks.len() + 1)?;
        if !seen.insert(block) {
            return None;
        }
        charge(work, total)?;
        let mut incoming = body
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(i, b)| b.successors.iter().map(move |&s| (i, s)))
            .filter(|&(_, s)| s == block);
        if incoming.next() != Some((previous, block)) || incoming.next().is_some() {
            return None;
        }
        let bb = body.blocks.get(block)?;
        let end = if block == at.block {
            at.event
        } else {
            bb.events.len()
        };
        if end > bb.events.len() {
            return None;
        }
        for (ordinal, event) in bb.events[..end].iter().enumerate() {
            harmless(
                r,
                addresses,
                Site {
                    block,
                    event: ordinal,
                },
                event,
                work,
            )?;
        }
        if block == at.block {
            return Some(backing);
        }
        let [next] = bb.successors.as_slice() else {
            return None;
        };
        previous = block;
        block = *next;
    }
}

/// Only original stack memory may be accessed between native acquisition and use.
/// This is native-object noninterference, not noescape or a general pure frame.
fn harmless(
    r: &Resolver<'_>,
    addresses: &Addresses,
    site: Site,
    event: &Event,
    work: &mut usize,
) -> Option<()> {
    charge(work, 1)?;
    let access = |place, bytes| {
        let address = addresses.get(place)?;
        (ByteRange {
            offset: address.offset,
            width: bytes,
        })
        .within(addresses.roots.extent(address.root).flatten())
        .then_some(())
    };
    match event {
        Event::Store {
            place,
            ty,
            ..
        }
        | Event::Load {
            place,
            ty,
            ..
        }
        | Event::Indirect {
            place,
            ty,
        } => {
            r.compiler.ir_ty(*ty)?;
            access(*place, r.compiler.ty.types.size_of(*ty))
        }
        Event::Initialize {
            place,
            ty,
            ..
        } => {
            matches!(r.body.blocks[site.block].events.get(site.event+1),Some(Event::DefaultImage {place:p,ty:t,..}) if p==place && t==ty).then_some(())?;
            access(*place, r.compiler.ty.types.size_of(*ty))
        }
        Event::DefaultImage {
            place,
            ty,
            bytes,
            ..
        } => {
            matches!(site.event.checked_sub(1).and_then(|i|r.body.blocks[site.block].events.get(i)),Some(Event::Initialize {place:p,ty:t,..}) if p==place && t==ty).then_some(())?;
            access(*place, *bytes)
        }
        Event::ReadMemory {
            value, ..
        } => access(*value, 0),
        Event::ContextSet {
            ..
        }
        | Event::ContextRead {
            ..
        }
        | Event::ContextAlias {
            ..
        }
        | Event::Expression {
            ..
        }
        | Event::Parameter {
            ..
        }
        | Event::Use {
            ..
        }
        | Event::Address {
            ..
        }
        | Event::Copy {
            ..
        }
        | Event::Offset {
            ..
        }
        | Event::IntegerBinary {
            ..
        }
        | Event::Boolean {
            ..
        }
        | Event::Local {
            ..
        }
        | Event::ScopeEnd {
            ..
        }
        | Event::StatementEnd {
            ..
        }
        | Event::StaticChoice {
            ..
        }
        | Event::Expansion {
            ..
        }
        | Event::Cleanup {
            ..
        }
        | Event::Field {
            ..
        }
        | Event::FieldProjection {
            ..
        }
        | Event::View {
            ..
        } => Some(()),
        _ => None,
    }
}
