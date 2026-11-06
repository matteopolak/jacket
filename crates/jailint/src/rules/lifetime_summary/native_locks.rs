//! Native held obligations are anchored to checked calls and captured objects.
use super::address_objects::{Base, Object};
use super::{BodyState, Event, HashSet, LifetimeBody, Prepared, ProcId, Resolver, Span, Summaries};
use crate::rules::lifetime_movement::facts::Site;
use std::collections::VecDeque;

const MAX_STEPS: usize = 8192;
const MAX_CANDIDATES: usize = 256;
type CallEffect = Option<(Option<Object>, super::locks::Balance)>;

/// Findings must agree across all checked instances sharing a written body.
/// Missing, obsolete, cyclic or canceled bodies establish no diagnostic.
pub(in crate::rules) fn findings(cx: &crate::syntax::Cx<'_>, instances: &[ProcId]) -> Vec<Span> {
    if instances.is_empty()
        || !cx.run.lock_primitives.get_or_init(|| {
            cx.compiler
                .pr
                .procs
                .iter()
                .enumerate()
                .any(|(i, _)| super::locks::native(cx.compiler, ProcId(i as u32)).is_some())
        })
    {
        return Vec::new();
    }
    let all = cx.run.lifetimes.get_or_init(|| {
        Summaries::build_for(
            cx.compiler,
            &cx.run.lifetime_files,
            cx.run.lifetime_entry_module,
        )
    });
    let Some(facts) = cx.compiler.ed.facts() else {
        return Vec::new();
    };
    let mut chosen = None;
    for id in instances {
        if jaic::interrupt::should_stop() {
            return Vec::new();
        }
        let Some(body) = facts.lifetime_bodies.get(id) else {
            return Vec::new();
        };
        let proc = cx.compiler.proc(*id);
        if proc.body_state != BodyState::Done
            || !std::sync::Arc::ptr_eq(&proc.lit, &body.syntax)
            || cyclic(body)
        {
            return Vec::new();
        }
        if body.blocks.iter().map(|b| b.events.len()).sum::<usize>() > MAX_STEPS {
            return Vec::new();
        }
        // Avoid preparing unrelated bodies when the checked direct targets have
        // no exported acquisition. An indirect call still requires resolution.
        if !body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .any(|event| match event {
                Event::Call {
                    callee,
                    expands: false,
                    ..
                } => all
                    .procs
                    .get(callee)
                    .and_then(|s| s.locks.as_ref())
                    .is_some_and(|t| t.balance.required == 0 && t.balance.delta > 0),
                Event::IndirectCall {
                    ..
                } => true,
                _ => false,
            })
        {
            if chosen
                .as_ref()
                .is_some_and(|prior: &Vec<Span>| !prior.is_empty())
            {
                return Vec::new();
            }
            chosen = Some(Vec::new());
            continue;
        }
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(cx.compiler)
                .body((body).procedure)
                .unwrap(),
            all,
        );
        let resolver = Resolver::for_prepared(&prepared, all);
        // Materialize each immutable checked call at its own event exactly once.
        // These temporary transfers are discarded with this body analysis.
        let effects: Vec<Vec<CallEffect>> = body
            .blocks
            .iter()
            .enumerate()
            .map(|(block, b)| {
                b.events
                    .iter()
                    .enumerate()
                    .map(|(event, e)| {
                        effect(
                            &resolver,
                            e,
                            Site {
                                block,
                                event,
                            },
                            all,
                        )
                    })
                    .collect()
            })
            .collect();
        let mut candidates = Vec::new();
        for (block, b) in body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                if let Some((Some(object), balance)) = effects[block][event]
                    && balance.required == 0
                    && balance.delta > 0
                    && let Event::Call {
                        span, ..
                    } = e
                {
                    candidates.push((
                        Site {
                            block,
                            event,
                        },
                        object,
                        balance.delta,
                        *span,
                    ));
                    if candidates.len() > MAX_CANDIDATES {
                        return Vec::new();
                    }
                }
            }
        }
        let mut current = Vec::new();
        let mut budget = MAX_STEPS;
        for (site, object, depth, span) in candidates {
            let Some(leak) = leaked(body, &effects, site, object, depth, &mut budget) else {
                return Vec::new();
            };
            if leak {
                current.push(span);
            }
        }
        current.sort_by_key(|s| (s.file.0, s.start, s.end));
        current.dedup();
        if chosen.as_ref().is_some_and(|prior| prior != &current) {
            return Vec::new();
        }
        chosen = Some(current);
    }
    if jaic::interrupt::should_stop() {
        return Vec::new();
    }
    chosen.unwrap_or_default()
}

fn effect(resolver: &Resolver<'_>, event: &Event, site: Site, all: &Summaries) -> CallEffect {
    if let Event::Call {
        callee,
        expands: true,
        ..
    } = event
    {
        return all
            .procs
            .get(callee)
            .filter(|s| s.trusted.is_none())
            .map(|_| (None, super::locks::Balance::default()));
    }
    let (Event::Call {
        args, ..
    }
    | Event::IndirectCall {
        args, ..
    }) = event
    else {
        return Some((None, super::locks::Balance::default()));
    };
    let targets = resolver.call_targets(event);
    if targets.unknown || targets.procedures.is_empty() {
        return None;
    }
    let mut chosen = None;
    for id in targets.ordered() {
        let transfer = all.procs.get(&id)?.locks.as_ref()?;
        let object = match transfer.object {
            Some(object) => Some(resolver.instantiate_lock_object(object, args, site, &all.procs)?),
            None if transfer.target.is_none() => None,
            None => return None,
        };
        let current = (object, transfer.balance);
        if chosen.as_ref().is_some_and(|prior| *prior != current) {
            return None;
        }
        chosen = Some(current);
    }
    chosen
}

fn leaked(
    body: &LifetimeBody,
    effects: &[Vec<CallEffect>],
    acquired: Site,
    object: Object,
    depth: i16,
    budget: &mut usize,
) -> Option<bool> {
    let mut queue = VecDeque::from([(acquired.block, acquired.event + 1, depth)]);
    let mut visited = HashSet::default();
    let mut leak = false;
    while let Some((block, from, mut depth)) = queue.pop_front() {
        if !visited.insert((block, from, depth)) {
            continue;
        }
        if visited.len() > 256 {
            return None;
        }
        let mut known = true;
        for (ordinal, event) in body.blocks[block].events.iter().enumerate().skip(from) {
            *budget = budget.checked_sub(1)?;
            if jaic::interrupt::should_stop() {
                return None;
            }
            if matches!(event, Event::Barrier | Event::Grow { .. }) {
                known = false;
                break;
            }
            if matches!(
                event,
                Event::Call {
                    expands: false,
                    ..
                } | Event::IndirectCall { .. }
            ) {
                let Some((next, balance)) = effects[block][ordinal] else {
                    known = false;
                    break;
                };
                if let Some(next) = next {
                    if next == object {
                        if depth < i16::from(balance.required) {
                            known = false;
                            break;
                        }
                        let next_depth = depth.checked_add(balance.delta)?;
                        if next_depth.abs() > 32 {
                            return None;
                        }
                        depth = next_depth;
                        // This acquisition's obligation is discharged. A later
                        // acquisition is a distinct candidate, even for this object.
                        if depth == 0 {
                            known = false;
                            break;
                        }
                    } else if matches!(next.base, Base::Entry(_))
                        || matches!(object.base, Base::Entry(_))
                    {
                        known = false;
                        break;
                    }
                }
            }
        }
        if !known {
            continue;
        }
        if body.blocks[block].normal_return && depth > 0 {
            leak = true;
        }
        for &next in &body.blocks[block].successors {
            queue.push_back((next, 0, depth));
        }
    }
    Some(leak)
}

/// Repeated acquisition sites require generative event identities. Until that
/// recency domain is present, a backedge is unknown rather than a stale pairing.
fn cyclic(body: &LifetimeBody) -> bool {
    if body.blocks.is_empty() || body.blocks.len() > MAX_STEPS {
        return true;
    }
    let mut reachable = HashSet::default();
    let mut stack = vec![0];
    while let Some(block) = stack.pop() {
        if reachable.insert(block) {
            stack.extend(&body.blocks[block].successors);
        }
    }
    let mut incoming = vec![0usize; body.blocks.len()];
    for &block in &reachable {
        for &next in &body.blocks[block].successors {
            incoming[next] += 1;
        }
    }
    let mut queue: VecDeque<_> = reachable
        .iter()
        .copied()
        .filter(|b| incoming[*b] == 0)
        .collect();
    let mut removed = 0usize;
    while let Some(block) = queue.pop_front() {
        removed += 1;
        for &next in &body.blocks[block].successors {
            incoming[next] -= 1;
            if incoming[next] == 0 {
                queue.push_back(next);
            }
        }
    }
    removed != reachable.len()
}
