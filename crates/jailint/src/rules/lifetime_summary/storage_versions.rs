//! A bounded, statement-ordered domain for explicitly replaced pointer fields.
//!
//! Slots start with their pre-state allocation. Stores select a checked SSA value;
//! control-flow joins retain alternatives, and opaque mutations add uncertainty. A
//! captured load keeps its own selection when the slot is subsequently replaced.
use super::{Event, HashMap, HashSet, LifetimeBody, Places, Val};
use std::collections::VecDeque;

mod request;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Source {
    Initial,
    Value(Val),
    /// A typed scalar captured by an actual full-width Store; never pointer provenance.
    ScalarValue {
        ty: jaic::types::TypeId,
        value: Val,
    },
    Constant {
        ty: jaic::types::TypeId,
        value: jaic::ide::lifetimes::LifetimeDefaultValue,
    },
    Unknown,
    Aggregate(
        Val,
        jaic::types::TypeId,
        usize,
        usize,
        crate::rules::lifetime_movement::facts::Site,
    ),
    Call(crate::rules::lifetime_movement::facts::Site),
}

pub(super) type Sources = HashSet<Source>;
type State = HashMap<usize, Sources>;
/// Reaching definitions from this borrowed checked body. This binds ownership,
/// not applicability of any selected branch cuts used by the transfer.
#[derive(Clone)]
pub(super) struct CapturedLoads<'c> {
    owner: crate::rules::lifetime_facts::CheckedBody<'c>,
    sources: HashMap<Val, Sources>,
}
impl<'c> CapturedLoads<'c> {
    pub(super) fn owner(&self) -> crate::rules::lifetime_facts::CheckedBody<'c> {
        self.owner
    }

    pub(super) fn empty(owner: crate::rules::lifetime_facts::CheckedBody<'c>) -> Self {
        Self {
            owner,
            sources: HashMap::default(),
        }
    }
    pub(super) fn matches(&self, owner: crate::rules::lifetime_facts::CheckedBody<'_>) -> bool {
        std::ptr::eq(self.owner.compiler(), owner.compiler())
            && std::ptr::eq(self.owner.body(), owner.body())
    }
}
impl std::ops::Deref for CapturedLoads<'_> {
    type Target = HashMap<Val, Sources>;
    fn deref(&self) -> &Self::Target {
        &self.sources
    }
}
impl std::fmt::Debug for CapturedLoads<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapturedLoads")
            .field("procedure", &self.owner.body().procedure)
            .field("sources", &self.sources)
            .finish()
    }
}
impl PartialEq for CapturedLoads<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.matches(other.owner) && self.sources == other.sources
    }
}
impl Eq for CapturedLoads<'_> {
}


/// Compute the finite reaching definitions of replaced pointer fields. Values are
/// body-local SSA identities, so loops converge without widening or a path cutoff.
pub(super) fn loads<'c>(
    places: &Places,
    core: &super::prepared::Core<'c>,
    probes: &super::call_slots::Probes,
) -> (CapturedLoads<'c>, super::call_slots::Snapshots) {
    loads_except(places, core, probes, &HashSet::default())
}

/// Collect exactly the ordinary pointer slots, independently of optional scalar requests.
pub(super) fn pointer_slots(
    compiler: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
) -> HashSet<usize> {
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| {
            let (Event::Store {
                place,
                ty,
                ..
            }
            | Event::Load {
                place,
                ty,
                ..
            }
            | Event::ContextRead {
                place,
                ty,
                ..
            }
            | Event::ContextSet {
                place,
                ty,
                ..
            }) = e
            else {
                return None;
            };
            let p = places.existing(*place, *ty)?;
            compiler.ty.types.is_pointer(*ty).then_some(p)
        })
        .collect()
}

/// The checked entry copy establishes an aggregate/value parameter snapshot.
/// Explicitly requested scalars instead retain their actual incoming SSA value.
fn initial_input_copy(
    compiler: &jaic::sema::Compiler,
    core: &super::prepared::Core<'_>,
    places: &Places,
    probes: &super::call_slots::Probes,
    place: Val,
    value: Val,
    ty: jaic::types::TypeId,
) -> bool {
    !compiler.ty.types.is_pointer(ty)
        && !places.existing(place, ty).is_some_and(|p| probes.scalar(p))
        && core.values.get(&value).is_some_and(|defs|defs.iter().any(|def| {
            matches!(def,super::Def::Parameter(_,target,input_ty) if *target==place && *input_ty==ty)
        }))
}

/// Physical exposure belongs to the root, independent of typed projections.
fn exposed_roots(body: &LifetimeBody, places: &Places) -> HashSet<jaic::ir::Val> {
    body.blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|event| {
            let Event::Address {
                place,
                ty,
                ..
            } = event
            else {
                return None;
            };
            let place = places.existing(*place, *ty)?;
            (!places.paths[place].1.contains(&super::Projection::Deref))
                .then(|| places.paths[place].0.0)
        })
        .collect()
}

/// The same ordered transfer under explicit structural edge exclusions. Callers
/// separately prove the cut hypotheses and complete normal-return coverage.
pub(super) fn loads_except<'c>(
    places: &Places,
    core: &super::prepared::Core<'c>,
    probes: &super::call_slots::Probes,
    skipped: &HashSet<(usize, usize)>,
) -> (CapturedLoads<'c>, super::call_slots::Snapshots) {
    execute(places, core, probes, skipped, None, None)
}

fn execute<'c>(
    places: &Places,
    core: &super::prepared::Core<'c>,
    probes: &super::call_slots::Probes,
    skipped: &HashSet<(usize, usize)>,
    mut capture: Option<&mut request::Capture<'_>>,
    query: Option<(
        &super::Resolver<'_>,
        &super::captured_values::QueryIndex,
        &CapturedLoads<'_>,
    )>,
) -> (CapturedLoads<'c>, super::call_slots::Snapshots) {
    let owner = core.checked_body();
    let compiler = owner.compiler();
    let body = owner.body();
    let owned = &core.owned;
    if capture.is_none() && !may_replace(body) {
        return (CapturedLoads::empty(owner), HashMap::default());
    }
    let mut slots = if let Some(capture) = capture.as_ref() {
        capture.tracked_slots().collect()
    } else {
        pointer_slots(compiler, body, places)
    };
    slots.extend(probes.slots());
    if slots.is_empty() {
        return (CapturedLoads::empty(owner), HashMap::default());
    }
    let exposed_roots = exposed_roots(body, places);
    let original_private = |slot: usize| {
        !places.paths[slot].1.contains(&super::Projection::Deref)
            && owned.contains(&places.root_of(slot))
            && places.selected_type(slot).is_some_and(|ty| {
                private_range(
                    compiler,
                    body,
                    places,
                    slot,
                    compiler.ty.types.size_of(ty),
                    &core.private_bytes,
                )
            })
            && !exposed_roots.contains(&places.paths[slot].0.0)
    };
    let all_private_spills = if capture.as_ref().is_some_and(|c| c.range.is_none()) {
        if capture
            .as_mut()
            .is_some_and(|c| !c.charge(slots.len().saturating_mul(8).saturating_add(1)))
        {
            return (CapturedLoads::empty(owner), HashMap::default());
        }
        if !probes.slots().all(|slot| original_private(slot)) {
            if let Some(capture) = &mut capture {
                capture.failed = true;
            }
            return (CapturedLoads::empty(owner), HashMap::default());
        }
        true
    } else {
        false
    };
    let private = |slot: usize| all_private_spills && probes.scalar(slot) || original_private(slot);
    let mut entry: State = slots
        .iter()
        .map(|p| {
            (
                *p,
                if capture.as_ref().is_some_and(|c| c.tracks(*p))
                    || private(*p)
                    || stable_incoming_slot(compiler, body, places, core, *p, &private)
                {
                    HashSet::from_iter([Source::Initial])
                } else {
                    HashSet::from_iter([Source::Initial, Source::Unknown])
                },
            )
        })
        .collect();
    if capture.as_mut().is_some_and(|c| !c.charge(slots.len() + 1)) {
        return (CapturedLoads::empty(owner), HashMap::default());
    }
    probes.normalize(&mut entry);
    let mut states = vec![None; body.blocks.len()];
    let start = capture.as_ref().and_then(|c| c.bound_entry()).unwrap_or(
        crate::rules::lifetime_movement::facts::Site {
            block: 0,
            event: 0,
        },
    );
    states[start.block] = Some(entry);
    let mut queue = VecDeque::from([start.block]);
    let mut result = CapturedLoads::empty(owner);
    let mut calls = HashMap::default();
    while let Some(index) = queue.pop_front() {
        if capture.as_ref().is_some_and(|c| !c.live.contains(index)) {
            continue;
        }
        if capture.as_mut().is_some_and(|c| {
            !c.charge(
                states[index]
                    .as_ref()
                    .map_or(0, |s| s.values().map(HashSet::len).sum::<usize>())
                    + 1,
            )
        }) {
            return (CapturedLoads::empty(owner), HashMap::default());
        }
        let mut state = states[index].clone().unwrap();
        for (ordinal, event) in
            body.blocks[index]
                .events
                .iter()
                .enumerate()
                .skip(if index == start.block {
                    start.event
                } else {
                    0
                })
        {
            let site = crate::rules::lifetime_movement::facts::Site {
                block: index,
                event: ordinal,
            };
            if capture.as_mut().is_some_and(|c| !c.observe(site, &state)) {
                return (CapturedLoads::empty(owner), HashMap::default());
            }
            if capture.as_ref().is_some_and(|c| c.ignored(site)) {
                continue;
            }
            if let Some((r, index, completed)) = query
                && capture
                    .as_mut()
                    .is_some_and(|c| !c.bound_event(r, index, completed, site, event))
            {
                return (CapturedLoads::empty(owner), HashMap::default());
            }
            if matches!(event, Event::Call { .. } | Event::Grow { .. }) {
                probes.observe(
                    crate::rules::lifetime_movement::facts::Site {
                        block: index,
                        event: ordinal,
                    },
                    &state,
                    &mut calls,
                );
            }
            if let Event::ContextRead {
                place,
                value,
                ..
            } = event
                && core.context_slot != Some(*place)
            {
                result
                    .sources
                    .entry(*value)
                    .or_default()
                    .insert(Source::Unknown);
                continue;
            }
            if let Event::ContextSet {
                place, ..
            } = event
                && core.context_slot != Some(*place)
            {
                for sources in state.values_mut() {
                    sources.extend([Source::Initial, Source::Unknown]);
                }
                continue;
            }
            let known_watch = capture.as_ref().is_some_and(|c| {
                c.bound_slots().any(|slot| {
                    state.get(&slot).is_some_and(|sources| {
                        sources.len() == 1
                            && matches!(sources.iter().next(), Some(Source::Constant { .. }))
                    })
                })
            });
            match event {
                Event::Barrier
                | Event::Grow {
                    ..
                } => {
                    for sources in state.values_mut() {
                        sources.extend([Source::Initial, Source::Unknown]);
                    }
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                }
                | Event::ContextRead {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    if let Some(slot) = places.existing(*place, *ty)
                        && (!probes.scalar(slot)
                            || probes.captures_scalar(
                                crate::rules::lifetime_movement::facts::Site {
                                    block: index,
                                    event: ordinal,
                                },
                                *value,
                                *ty,
                                slot,
                            ))
                        && let Some(sources) = state.get(&slot)
                    {
                        if capture
                            .as_mut()
                            .is_some_and(|c| !c.charge(sources.len() + 1))
                        {
                            return (CapturedLoads::empty(owner), HashMap::default());
                        }
                        result.sources.entry(*value).or_default().extend(sources);
                    }
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    constant,
                    ..
                }
                | Event::ContextSet {
                    place,
                    value,
                    ty,
                    constant,
                } => {
                    // The compiler's initial aggregate formal copy establishes
                    // the entry snapshot; it does not overwrite that snapshot with
                    // an unknown later value. Match the checked Parameter event.
                    if initial_input_copy(compiler, core, places, probes, *place, *value, *ty) {
                        continue;
                    }
                    let target = places.existing(*place, *ty);
                    let target = capture.as_ref().map_or(target, |c| c.target(site, target));
                    // Only a completed inline transfer can discharge same-root aliases.
                    let mut inline_transfer = false;
                    if let Some(target) = target.filter(|p| !places.unresolved(*p)) {
                        store_inline(
                            compiler,
                            body,
                            places,
                            &mut state,
                            target,
                            probes,
                            Stored {
                                value: *value,
                                ty: *ty,
                                constant: *constant,
                                site: crate::rules::lifetime_movement::facts::Site {
                                    block: index,
                                    event: ordinal,
                                },
                            },
                        );
                        inline_transfer = true;
                        if private(target) {
                            continue;
                        }
                    }
                    // Different receiver values can alias. Without a disjointness proof,
                    // a write through another path cannot preserve a must origin.
                    for (&slot, sources) in &mut state {
                        if private(slot)
                            || capture
                                .as_ref()
                                .is_some_and(|c| c.tracks(slot) && c.disjoint(site))
                        {
                            continue;
                        }
                        if inline_transfer
                            && !places.unresolved(slot)
                            && target.is_some_and(|target| {
                                inline_range(
                                    compiler,
                                    body,
                                    places,
                                    target,
                                    compiler.ty.types.size_of(*ty),
                                )
                                .zip(places.selected_type(slot).and_then(|ty| {
                                    inline_range(
                                        compiler,
                                        body,
                                        places,
                                        slot,
                                        compiler.ty.types.size_of(ty),
                                    )
                                }))
                                .is_some_and(|(a, b)| a.0 == b.0)
                            })
                        {
                            continue;
                        }
                        if Some(slot) == target
                            && (capture.as_ref().is_some_and(|c| c.tracks(slot))
                                || stable_incoming_slot(
                                    compiler, body, places, core, slot, &private,
                                )
                                || !places.unresolved(slot))
                        {
                            *sources = HashSet::from_iter([Source::Value(*value)]);
                        } else if !target.is_some_and(|p| places.disjoint_slots(compiler, slot, p))
                        {
                            sources.insert(Source::Unknown);
                            if compiler.ty.types.is_pointer(*ty) {
                                sources.insert(Source::Value(*value));
                            }
                        }
                    }
                }
                Event::DefaultImage {
                    ..
                }
                | Event::Initialize {
                    ..
                } => {
                    initialize(
                        compiler,
                        body,
                        places,
                        (core, probes),
                        &mut state,
                        (
                            crate::rules::lifetime_movement::facts::Site {
                                block: index,
                                event: ordinal,
                            },
                            event,
                        ),
                        &private,
                    );
                }
                Event::Call {
                    expands: false, ..
                }
                | Event::IndirectCall {
                    ..
                } => {
                    // Preparation cannot rely on write summaries still being solved.
                    // Retain may alternatives but never infer a must fact across a call
                    // that could write through an alias or an escaped receiver.
                    for (&slot, sources) in &mut state {
                        if private(slot)
                            || capture
                                .as_ref()
                                .is_some_and(|c| c.tracks(slot) && c.disjoint(site))
                        {
                            continue;
                        }
                        sources.extend([Source::Initial, Source::Unknown]);
                        sources.insert(Source::Call(
                            crate::rules::lifetime_movement::facts::Site {
                                block: index,
                                event: ordinal,
                            },
                        ));
                    }
                }
                _ => {}
            }
            if capture
                .as_mut()
                .is_some_and(|c| !c.seed_after(site, &mut state, places))
            {
                return (CapturedLoads::empty(owner), HashMap::default());
            }
            if known_watch
                && jacket_config::env::LIFETIME_STATS.is_set()
                && capture.as_ref().is_some_and(|c| {
                    c.bound_slots().any(|slot| {
                        state
                            .get(&slot)
                            .is_some_and(|sources| sources.contains(&Source::Unknown))
                    })
                })
            {
                eprintln!("lifetime-bound-field-stop site={site:?} event={event:?}");
            }
        }
        if capture.as_mut().is_some_and(|c| {
            !c.observe(
                crate::rules::lifetime_movement::facts::Site {
                    block: index,
                    event: body.blocks[index].events.len(),
                },
                &state,
            )
        }) {
            return (CapturedLoads::empty(owner), HashMap::default());
        }
        if capture.as_mut().is_some_and(|c| !c.charge(state.len() + 1)) {
            return (CapturedLoads::empty(owner), HashMap::default());
        }
        probes.normalize(&mut state);
        for &next in &body.blocks[index].successors {
            if skipped.contains(&(index, next))
                || capture.as_ref().is_some_and(|c| !c.live.contains(next))
            {
                continue;
            }
            if capture.as_mut().is_some_and(|c| {
                !c.charge(
                    state.values().map(HashSet::len).sum::<usize>()
                        + states[next]
                            .as_ref()
                            .map_or(0, |s| s.values().map(HashSet::len).sum::<usize>())
                        + 1,
                )
            }) {
                return (CapturedLoads::empty(owner), HashMap::default());
            }
            let changed = if let Some(prior) = &mut states[next] {
                let mut changed = false;
                for (&slot, sources) in &state {
                    let into = prior.get_mut(&slot).unwrap();
                    changed |= probes.join(slot, into, sources);
                }
                changed
            } else {
                states[next] = Some(state.clone());
                true
            };
            if changed {
                queue.push_back(next);
            }
        }
    }
    (result, calls)
}

/// Capture coordinates once, then track one bound byte cell with the same
/// ordered transfer. Both runs retain all paths; the completed first result is
/// the sole immutable coordinate authority, never second-run provisional data.
pub(super) fn requested_bound_loads<'c>(
    r: &super::Resolver<'c>,
    all: &HashMap<super::ProcId, super::Summary>,
    requests: &super::call_slots::ScalarRequests,
    remaining: &mut usize,
    index: &super::captured_values::QueryIndex,
    bound: (
        Val,
        jaic::types::TypeId,
        &[(jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange)],
        crate::rules::lifetime_movement::facts::Site,
        crate::rules::lifetime_movement::facts::Site,
    ),
) -> Option<(
    CapturedLoads<'c>,
    Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>,
    (i128, i128, u64),
    HashSet<crate::rules::lifetime_movement::facts::Site>,
)> {
    *remaining = remaining.checked_sub(requests.reads.len().checked_mul(4)?)?;
    let probes = super::call_slots::Probes::for_scalar_requests(requests);
    let (mut places, mut capture) = request::Capture::reads(
        r,
        &probes.slots().collect(),
        requests.reads.len(),
        all,
        remaining,
        index,
    )?;
    capture.bind(r, &mut places, request::Seed::Image, bound)?;
    let first = collect_requested_loads(
        r,
        requests,
        &places,
        &probes,
        &HashSet::default(),
        &mut capture,
        index,
        None,
    )?;
    capture.sources.clear();
    capture.activate_bound();
    let (discarded, _) = execute(
        &places,
        r.core(),
        &super::call_slots::Probes::default(),
        &HashSet::default(),
        Some(&mut capture),
        Some((r, index, &first)),
    );
    if capture.failed
        || !discarded.matches(r.checked_body())
        || !capture.charge(
            discarded
                .capacity()
                .checked_add(discarded.values().map(HashSet::len).sum())?
                .checked_add(1)?,
        )
    {
        return None;
    }
    let sources = capture.bound_values(r, &places)?;
    let access = capture.bound_access()?;
    let markers = capture.metadata_barriers(r)?;
    capture
        .charge(sources.len())
        .then_some((first, sources, access, markers))
}

fn collect_requested_loads<'c>(
    r: &super::Resolver<'c>,
    requests: &super::call_slots::ScalarRequests,
    places: &Places,
    probes: &super::call_slots::Probes,
    skipped: &HashSet<(usize, usize)>,
    capture: &mut request::Capture<'_>,
    index: &super::captured_values::QueryIndex,
    completed: Option<&CapturedLoads<'_>>,
) -> Option<CapturedLoads<'c>> {
    let (loads, _) = execute(
        places,
        r.core(),
        probes,
        skipped,
        Some(capture),
        completed.map(|loads| (r, index, loads)),
    );
    if capture.failed || !loads.matches(r.checked_body()) || !capture.charge(loads.capacity() + 1) {
        return None;
    }
    for (&value, sources) in loads.iter() {
        if !capture.charge(sources.len() + requests.reads.len() + 1) {
            return None;
        }
        let Some(super::captured_values::Producer::Load(ty, site)) = index.get(value) else {
            return None;
        };
        if !requests
            .reads
            .iter()
            .any(|read| read.value == value && read.ty == ty && read.site == site)
        {
            return None;
        }
    }
    capture.charge(1).then_some(loads)
}

/// Capture one formal-relative pointer slot using the ordinary ordered transfer.
/// The current call environment supplies normal-return reachability only; this
/// MAY result does not certify a complete call poststate or purity.
pub(super) fn before(
    r: &super::Resolver<'_>,
    at: crate::rules::lifetime_movement::facts::Site,
    slot: super::physical_slots::Slot,
    all: &HashMap<super::ProcId, super::Summary>,
    remaining: &mut usize,
    index: &super::captured_values::QueryIndex,
) -> Option<Sources> {
    let (places, mut capture) = request::Capture::new(r, at, slot, all, remaining, index)?;
    execute(
        &places,
        r.core(),
        &super::call_slots::Probes::default(),
        &HashSet::default(),
        Some(&mut capture),
        None,
    );
    let output = capture.sources.len();
    (!capture.failed && output != 0 && capture.charge(output)).then_some(capture.sources)
}

/// Keep allocation-epoch queries separate from descriptor identity and logical reads.
/// The guard restores the previous mode even if a nested query unwinds.
pub(super) struct Query<'a> {
    mode: &'a std::cell::Cell<bool>,
    previous: bool,
}

impl Drop for Query<'_> {
    fn drop(&mut self) {
        self.mode.set(self.previous);
    }
}

impl super::Resolver<'_> {
    pub(super) fn storage_query(&self) -> Query<'_> {
        Query {
            mode: &self.storage_mode,
            previous: self.storage_mode.replace(true),
        }
    }
}

/// A private access must fit the original allocation, regardless of view type.
pub(super) fn private_range(
    c: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
    place: usize,
    width: u64,
    roots: &HashMap<usize, u64>,
) -> bool {
    let Some(bytes) = roots.get(&places.root_of(place)) else {
        return false;
    };
    let Some((root, offset, width)) = inline_range(c, body, places, place, width) else {
        return false;
    };
    root == places.paths[place].0.0 && offset.checked_add(width).is_some_and(|end| end <= *bytes)
}

/// Typed inline extents for private slots; dereferencing leaves this stack object.
pub(super) fn inline_range(
    c: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
    place: usize,
    width: u64,
) -> Option<(Val, u64, u64)> {
    if let Some(range) = typed_inline_range(c, places, place, width) {
        return Some(range);
    }
    let mut found = None;
    for event in body.blocks.iter().flat_map(|b| &b.events) {
        let Event::FieldProjection {
            result,
            base_ty,
            result_ty,
            offset,
            ..
        } = event
        else {
            continue;
        };
        if places.existing(*result, *result_ty) != Some(place) {
            continue;
        }
        for (&base, ty) in &places.types {
            if ty != base_ty
                || base == place
                || places.paths[base].0 != places.paths[place].0
                || !places.paths[place].1.starts_with(&places.paths[base].1)
            {
                continue;
            }
            let (root, begin, _) = typed_inline_range(c, places, base, c.ty.types.size_of(*ty))?;
            let begin = begin.checked_add(*offset)?;
            begin.checked_add(width)?;
            let range = (root, begin, width);
            if found.is_some_and(|prior| prior != range) {
                return None;
            }
            found = Some(range);
        }
    }
    found
}

fn typed_inline_range(
    c: &jaic::sema::Compiler,
    places: &Places,
    place: usize,
    width: u64,
) -> Option<(Val, u64, u64)> {
    let ((root, root_ty), steps) = &places.paths[place];
    let mut ty = *root_ty;
    let mut offset = 0u64;
    for step in steps {
        match step {
            super::Projection::Inline {
                offset: delta,
                ty: selected,
            } => {
                offset = offset.checked_add(*delta)?;
                ty = *selected;
            }
            super::Projection::Field(name) => {
                let st = c.ty.types.as_struct(ty)?;
                let field =
                    c.ty.types
                        .struct_info(st)
                        .fields
                        .iter()
                        .find(|f| f.name == Some(*name))?;
                offset = offset.checked_add(field.offset)?;
                ty = field.ty;
            }
            super::Projection::Element(index) => {
                let jaic::types::TypeKind::Array {
                    kind: jaic::types::ArrayKind::Fixed(count),
                    elem,
                } = c.ty.types.kind(ty)
                else {
                    return None;
                };
                let index = u64::try_from(*index).ok()?;
                if index >= *count {
                    return None;
                }
                offset = offset.checked_add(index.checked_mul(c.ty.types.size_of(*elem))?)?;
                ty = *elem;
            }
            _ => return None,
        }
    }
    offset.checked_add(width)?;
    Some((*root, offset, width))
}

fn ranges_overlap(
    places: &Places,
    a: usize,
    b: usize,
    a_range: Option<(Val, u64, u64)>,
    b_range: Option<(Val, u64, u64)>,
) -> bool {
    if let (Some((a, ao, aw)), Some((b, bo, bw))) = (a_range, b_range) {
        a == b
            && ao.checked_add(aw).is_none_or(|end| end > bo)
            && bo.checked_add(bw).is_none_or(|end| end > ao)
    } else {
        places.paths[a].0.0 == places.paths[b].0.0
    }
}

/// Initial memory reads require a same-address incoming pointer and an inline
/// projection interpreted under that formal's actual checked type. Legacy Copy
/// definitions also include pointer arithmetic, so authorize their source event.
fn stable_incoming_slot(
    c: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
    core: &super::prepared::Core<'_>,
    place: usize,
    private: &impl Fn(usize) -> bool,
) -> bool {
    fn input(
        c: &jaic::sema::Compiler,
        body: &LifetimeBody,
        core: &super::prepared::Core<'_>,
        value: Val,
        private: &impl Fn(usize) -> bool,
        seen: &mut HashSet<Val>,
    ) -> Option<usize> {
        if seen.len() > 24
            || !seen.insert(value)
            || core.address_casts.contains_key(&value)
            || body.blocks.iter().flat_map(|b| &b.events).any(
                |e| matches!(e,Event::Offset {value:v,offset,..} if *v==value && *offset!=Some(0)),
            )
        {
            return None;
        }
        let mut result = None;
        for def in core.values.get(&value)? {
            let formal = match def {
                super::Def::Parameter(formal, ..) => *formal,
                super::Def::Copy(source) => {
                    input(c, body, core, *source, private, &mut seen.clone())?
                }
                super::Def::Load(slot, _) if private(*slot) => {
                    let [source] = core.writes.get(slot)?.as_slice() else {
                        return None;
                    };
                    input(c, body, core, *source, private, &mut seen.clone())?
                }
                _ => return None,
            };
            if !core
                .params
                .values()
                .any(|(index, ty)| *index == formal && c.ty.types.is_pointer(*ty))
                || result.is_some_and(|prior| prior != formal)
            {
                return None;
            }
            result = Some(formal);
        }
        result
    }
    let ((root, root_ty), steps) = &places.paths[place];
    let formal = if let Some(formal) = input(c, body, core, *root, private, &mut HashSet::default())
    {
        formal
    } else if steps.first() == Some(&super::Projection::Deref) && private(places.root_of(place)) {
        let Some([source]) = core.writes.get(&places.root_of(place)).map(Vec::as_slice) else {
            return false;
        };
        let Some(formal) = input(c, body, core, *source, private, &mut HashSet::default()) else {
            return false;
        };
        formal
    } else {
        return false;
    };
    let Some((_, param_ty)) = core.params.values().find(|(n, _)| *n == formal) else {
        return false;
    };
    let Some(pointee) = c.ty.types.pointee(*param_ty) else {
        return false;
    };
    let (mut ty, steps) = if root_ty == param_ty && steps.first() == Some(&super::Projection::Deref)
    {
        (pointee, &steps[1..])
    } else if *root_ty == pointee {
        (*root_ty, steps.as_slice())
    } else {
        return false;
    };
    for step in steps {
        match step {
            super::Projection::Field(name) => {
                let Some((_, field_ty)) =
                    super::physical_slots::field_projection(c, body, places, ty, *name)
                else {
                    return false;
                };
                ty = field_ty;
            }
            super::Projection::Element(index) => {
                let jaic::types::TypeKind::Array {
                    kind: jaic::types::ArrayKind::Fixed(count),
                    elem,
                } = c.ty.types.kind(ty)
                else {
                    return false;
                };
                if u64::try_from(*index)
                    .ok()
                    .is_none_or(|index| index >= *count)
                {
                    return false;
                }
                ty = *elem;
            }
            _ => return false,
        }
    }
    true
}

#[derive(Clone, Copy)]
struct Stored {
    value: Val,
    ty: jaic::types::TypeId,
    constant: Option<u64>,
    site: crate::rules::lifetime_movement::facts::Site,
}

/// Apply a typed inline store to tracked pointer descendants and overlapping slots.
fn store_inline(
    compiler: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
    state: &mut State,
    target: usize,
    probes: &super::call_slots::Probes,
    stored: Stored,
) {
    if state.keys().all(|slot| places.unresolved(*slot)) {
        return;
    }
    let Stored {
        ref value,
        ref ty,
        constant,
        site,
    } = stored;
    let target_range = inline_range(
        compiler,
        body,
        places,
        target,
        compiler.ty.types.size_of(*ty),
    );
    for (&slot, sources) in state {
        if places.unresolved(slot) {
            continue;
        }
        let width = places
            .selected_type(slot)
            .map(|ty| compiler.ty.types.size_of(ty));
        let slot_range = width.and_then(|width| inline_range(compiler, body, places, slot, width));
        if Some(slot) == Some(target) || (target_range.is_some() && target_range == slot_range) {
            if compiler.ty.types.is_pointer(*ty) {
                *sources = HashSet::from_iter([if constant == Some(0) {
                    Source::Constant {
                        ty: *ty,
                        value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0),
                    }
                } else {
                    Source::Value(*value)
                }]);
            } else if let Some(bits) = constant
                && let Some((width, _)) = compiler.ty.types.int_info(*ty)
                && width > 0
                && width <= 64
            {
                let bits = if width == 64 {
                    bits
                } else {
                    bits & ((1_u64 << width) - 1)
                };
                *sources = HashSet::from_iter([Source::Constant {
                    ty: *ty,
                    value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits),
                }]);
            } else if probes.scalar(slot) && places.selected_type(slot) == Some(*ty) {
                *sources = HashSet::from_iter([Source::ScalarValue {
                    ty: *ty,
                    value: *value,
                }]);
            } else if compiler.ir_ty(*ty).is_none()
                && places.paths[slot].1.starts_with(&places.paths[target].1)
            {
                *sources = HashSet::from_iter([Source::Aggregate(*value, *ty, target, slot, site)]);
            } else {
                sources.insert(Source::Unknown);
            }
        } else if let (Some((base, begin, size)), Some((root, offset, width))) =
            (target_range, slot_range)
            && base == root
            && begin <= offset
            && begin
                .checked_add(size)
                .is_some_and(|end| offset.checked_add(width).is_some_and(|last| last <= end))
            && compiler.ir_ty(*ty).is_none()
            && places.paths[slot].1.starts_with(&places.paths[target].1)
        {
            *sources = HashSet::from_iter([Source::Aggregate(*value, *ty, target, slot, site)]);
        } else if ranges_overlap(places, slot, target, slot_range, target_range) {
            sources.insert(Source::Unknown);
        }
    }
}

/// Avoid an epoch graph when the body cannot replace an externally read pointer slot.
fn may_replace(body: &LifetimeBody) -> bool {
    body.blocks.iter().flat_map(|b| &b.events).any(|e| {
        matches!(
            e,
            Event::Barrier
                | Event::Grow { .. }
                | Event::Call {
                    expands: false,
                    ..
                }
                | Event::IndirectCall { .. }
                | Event::Store { .. }
                | Event::Initialize { .. }
        )
    })
}

/// Apply selected default bytes and initialization overlap at the current epoch.
fn initialize(
    compiler: &jaic::sema::Compiler,
    body: &LifetimeBody,
    places: &Places,
    (core, probes): (&super::prepared::Core<'_>, &super::call_slots::Probes),
    state: &mut State,
    (site, event): (crate::rules::lifetime_movement::facts::Site, &Event),
    private: &impl Fn(usize) -> bool,
) {
    match event {
        Event::DefaultImage {
            place,
            ty,
            bytes,
            zeroed,
            complete,
            leaves,
            ..
        } => {
            let target = places.existing(*place, *ty);
            if let Some(target) = target
                && private_range(compiler, body, places, target, *bytes, &core.private_bytes)
                && let Some((root, offset, _)) =
                    inline_range(compiler, body, places, target, *bytes)
            {
                let paired = site
                    .event
                    .checked_sub(1)
                    .and_then(|i| body.blocks.get(site.block)?.events.get(i))
                    .is_some_and(
                        |e| matches!(e,Event::Initialize{place:p,ty:t,..} if p==place && t==ty),
                    );
                let null = paired
                    && *complete
                    && compiler.ty.types.is_pointer(*ty)
                    && *bytes == compiler.ty.types.size_of(*ty)
                    && matches!(leaves.as_slice(),[leaf]
                        if leaf.ty==*ty && leaf.offset==0 && leaf.width==*bytes
                            && leaf.value==jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0));
                if null && let Some(sources) = state.get_mut(&target) {
                    *sources = HashSet::from_iter([Source::Constant {
                        ty: *ty,
                        value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0),
                    }]);
                }
                if !zeroed {
                    return;
                }
                for (&slot, sources) in &mut *state {
                    let Some(ty) = places.selected_type(slot) else {
                        continue;
                    };
                    // Every supported pointer is an eight-byte value whose null
                    // literal lowers to Ptr(0). Retain only these numeric bits;
                    // pointer provenance readers deliberately reject Constant.
                    let pointer_bits = paired
                        && probes.requested(slot)
                        && compiler.ty.types.is_pointer(ty)
                        && compiler.ty.types.size_of(ty) == 8;
                    if compiler.ty.types.int_info(ty).is_none() && !pointer_bits {
                        continue;
                    }
                    if let Some((base, start, width)) =
                        inline_range(compiler, body, places, slot, compiler.ty.types.size_of(ty))
                        && root == base
                        && offset <= start
                        && offset.checked_add(*bytes).is_some_and(|end| {
                            start.checked_add(width).is_some_and(|last| last <= end)
                        })
                    {
                        *sources = HashSet::from_iter([Source::Constant {
                            ty,
                            value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0),
                        }]);
                    }
                }
            }
        }
        Event::Initialize {
            place,
            ty,
            ..
        } => {
            let target = places.existing(*place, *ty);
            if let Some(target) = target.filter(|p| !places.unresolved(*p)) {
                let target_range = inline_range(
                    compiler,
                    body,
                    places,
                    target,
                    compiler.ty.types.size_of(*ty),
                );
                for (&slot, sources) in &mut *state {
                    if places.unresolved(slot) {
                        continue;
                    }
                    let width = places
                        .selected_type(slot)
                        .map(|ty| compiler.ty.types.size_of(ty));
                    let slot_range =
                        width.and_then(|width| inline_range(compiler, body, places, slot, width));
                    if !places.unresolved(slot)
                        && ranges_overlap(places, slot, target, slot_range, target_range)
                    {
                        *sources = HashSet::from_iter([Source::Unknown]);
                    }
                }
                let target_private = private(target);
                for (&slot, sources) in &mut *state {
                    if !target_private
                        && !private(slot)
                        && !places.disjoint_slots(compiler, slot, target)
                    {
                        sources.insert(Source::Unknown);
                    }
                }
            } else {
                for (&slot, sources) in &mut *state {
                    if !private(slot) {
                        sources.extend([Source::Initial, Source::Unknown]);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Replay the ordinary transfer with one guarded, current constructor poststate.
/// The completed map supplies coordinates only; ordinary Core is never changed.
pub(super) fn requested_call_constants(
    r: &super::Resolver<'_>,
    all: &HashMap<super::ProcId, super::Summary>,
    index: &super::captured_values::QueryIndex,
    (address, ty, fields, values, call, at): (
        Val,
        jaic::types::TypeId,
        &[(jaic::types::TypeId, crate::rules::lifetime_facts::ByteRange)],
        [Option<jaic::ide::lifetimes::LifetimeDefaultValue>; 3],
        crate::rules::lifetime_movement::facts::Site,
        crate::rules::lifetime_movement::facts::Site,
    ),
    remaining: &mut usize,
) -> Option<(
    Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>,
    (i128, i128, u64),
)> {
    let probes = super::call_slots::Probes::default();
    let (mut places, mut capture) =
        request::Capture::reads(r, &HashSet::default(), 0, all, remaining, index)?;
    capture.bind(
        r,
        &mut places,
        request::Seed::Call(values),
        (address, ty, fields, call, at),
    )?;
    capture.activate_bound();
    capture.prepare_calls(r, all)?;
    let (discarded, _) = execute(
        &places,
        r.core(),
        &probes,
        &HashSet::default(),
        Some(&mut capture),
        Some((r, index, &r.core().storage_versions)),
    );
    if capture.failed || !discarded.matches(r.checked_body()) {
        return None;
    }
    Some((capture.bound_values(r, &places)?, capture.bound_access()?))
}
