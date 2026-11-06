//! Bounded typed storage requests serviced by the existing ordered pass.
use super::storage_versions::{Source, Sources};
use super::{
    ArrayKind, Compiler, Event, HashMap, LifetimeBody, LifetimeOperand, Places, TypeId, TypeKind,
    Val,
};
use crate::rules::lifetime_facts::{ByteRange, StackRoots, current_body};
use crate::rules::lifetime_movement::facts::Site;

use jaic::fxhash::HashSet;

const MAX_SCAN: usize = 8192;
const MAX_REQUESTS: usize = 64;

#[derive(Clone, Debug)]
pub(super) struct Request {
    pub formal: usize,
    pub header: usize,
    pub ty: TypeId,
    pub element: TypeId,
    slots: [usize; 3],
}

#[derive(Default)]
pub(super) struct Probes {
    calls: HashMap<Site, Vec<Request>>,
    starts: HashMap<Site, Site>,
    slots: HashSet<usize>,
    scalar_reads: Vec<ScalarRead>,
    scalar_slots: HashSet<usize>,
    scalar_work_bound: usize,
}

/// Immutable optional requests and reserved work, not a measured execution cost.
#[derive(Clone, Debug)]
pub(super) struct ScalarRequests {
    pub reads: Vec<ScalarRead>,
    pub work_bound: usize,
}

/// An actual selected private scalar read, registered before preparation freezes.
#[derive(Clone, Debug)]
pub(super) struct ScalarRead {
    pub site: Site,
    pub value: Val,
    pub ty: TypeId,
    pub slot: usize,
}

#[derive(Clone, Debug)]
pub(super) struct Snapshot {
    pub request: Request,
    pub prestate: Site,
    sources: [Sources; 3],
}

pub(super) type Snapshots = HashMap<(Site, usize), Snapshot>;

impl Probes {
    /// Select typed requests, not procedure effects. A pointer-returning checked
    /// call with an actual stack descriptor may later qualify for a borrow proof.
    pub fn collect(c: &Compiler, body: &LifetimeBody, places: &mut Places) -> Self {
        let mut roots = None;
        let mut pending = Vec::new();
        let mut scanned = 0;
        for (block, bb) in body.blocks.iter().enumerate() {
            for (event, e) in bb.events.iter().enumerate() {
                scanned += 1;
                if scanned > MAX_SCAN || jaic::interrupt::should_stop() {
                    return Self::default();
                }
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
                if !c
                    .ed
                    .facts()
                    .and_then(|f| f.lifetime_bodies.get(callee))
                    .is_some_and(|b| current_body(c, *callee, b))
                {
                    continue;
                }
                scanned = scanned.saturating_add(args.len().saturating_mul(3));
                if scanned > MAX_SCAN {
                    return Self::default();
                }
                let mut formals = HashMap::<usize, usize>::default();
                for arg in args {
                    *formals.entry(arg.formal).or_default() += 1;
                }
                let Some(signature) =
                    c.pr.procs
                        .get(callee.0 as usize)
                        .and_then(|p| p.sig.as_ref())
                else {
                    continue;
                };
                for arg in args {
                    let Some(param) = signature.params.get(arg.formal) else {
                        continue;
                    };
                    if arg.spread
                        || param.variadic
                        || formals[&arg.formal] != 1
                        || param.ty != arg.ty
                    {
                        continue;
                    }
                    let (LifetimeOperand::Value(value), Some(ty)) =
                        (arg.operand, c.ty.types.pointee(arg.ty))
                    else {
                        continue;
                    };
                    let TypeKind::Array {
                        elem,
                        kind: ArrayKind::Resizable,
                    } = *c.ty.types.kind(c.ty.types.repr(ty))
                    else {
                        continue;
                    };
                    let Some((_, _, pointer)) = results
                        .iter()
                        .find(|(_, _, result)| c.ty.types.pointee(*result) == Some(elem))
                    else {
                        continue;
                    };
                    let Some(roots) = roots
                        .get_or_insert_with(|| StackRoots::capture(body, MAX_SCAN))
                        .as_ref()
                    else {
                        return Self::default();
                    };
                    let value = roots.canonical(value);
                    let Some(header) = places.existing(value, ty) else {
                        continue;
                    };
                    if !places.paths[header].1.is_empty()
                        || !(ByteRange {
                            offset: 0,
                            width: c.ty.types.size_of(ty),
                        })
                        .within(roots.extent(value).flatten())
                    {
                        continue;
                    }
                    if pending.len() == MAX_REQUESTS {
                        return Self::default();
                    }
                    pending.push((
                        Site {
                            block,
                            event,
                        },
                        arg.formal,
                        header,
                        ty,
                        elem,
                        *pointer,
                    ));
                }
            }
        }
        let Some(Some(roots)) = roots else {
            return Self::default();
        };
        let mut result = Self::default();
        for (site, formal, header, ty, element, pointer) in pending {
            let mut slots = [0; 3];
            let roles = [
                (jaic::layout::resizable::COUNT, TypeId::S64),
                (jaic::layout::resizable::ALLOCATED, TypeId::S64),
                (jaic::layout::resizable::DATA, pointer),
            ];
            for (slot, (offset, ty)) in slots.iter_mut().zip(roles) {
                let Some(place) = places.inline_probe(c, &roots, header, offset, ty) else {
                    return Self::default();
                };
                *slot = place;
            }
            result.slots.extend(slots);
            result.calls.entry(site).or_default().push(Request {
                formal,
                header,
                ty,
                element,
                slots,
            });
        }
        for &site in result.calls.keys() {
            let start = call_start(c, body, site).unwrap_or(site);
            result.starts.insert(start, site);
        }
        result
    }

    /// Request only the bounded scalar reads needed by pointer/Bool return transforms.
    /// Failure leaves all existing pointer and descriptor-call requests unchanged.
    pub fn request_scalars(
        &mut self,
        c: &Compiler,
        body: &LifetimeBody,
        places: &Places,
        core: &super::prepared::Core<'_>,
    ) {
        if !current_body(c, body.procedure, body) {
            return;
        }
        let Some(sig) =
            c.pr.procs
                .get(body.procedure.0 as usize)
                .and_then(|p| p.sig.as_ref())
        else {
            return;
        };
        if sig.params.len() > 8
            || sig.returns.len() != 1
            || c.ty.types.pointee(sig.returns[0]).is_none()
            || c.ty.types.size_of(sig.returns[0]) != 8
            || !sig.params.iter().any(|p| p.ty == TypeId::BOOL)
            || !sig.params.iter().any(|p| c.ty.types.is_pointer(p.ty))
            || !sig
                .params
                .iter()
                .any(|p| matches!(p.ty, TypeId::U64 | TypeId::S64))
            || body.blocks.len() > 32
        {
            return;
        }
        let Some(scope) = scalar_scope(c, body, places, core) else {
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-scalar-requests proc={} admission=unknown",
                    body.procedure.0
                );
            }
            return;
        };
        let reads = scope.reads;
        let mut pending = Vec::with_capacity(reads);
        for (block, bb) in body.blocks.iter().enumerate() {
            for (event, e) in bb.events.iter().enumerate() {
                let Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } = e
                else {
                    continue;
                };
                if !matches!(*ty, TypeId::U64 | TypeId::S64 | TypeId::BOOL) {
                    continue;
                }
                let Some(slot) = places.existing(*place, *ty) else {
                    continue;
                };
                if !places.paths[slot].1.is_empty() {
                    continue;
                }
                if places.selected_type(slot) != Some(*ty)
                    || !core.private_slot(c, body, places, slot)
                {
                    continue;
                }
                pending.push(ScalarRead {
                    site: Site {
                        block,
                        event,
                    },
                    value: *value,
                    ty: *ty,
                    slot,
                });
            }
        }
        let Some(work) = scope.reservation(c, body, places, &pending) else {
            return;
        };
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-scalar-requests proc={} events={} edges={} reads={} pointer_slots={} work_bound={} limit={} payload_bytes={}",
                body.procedure.0,
                scope.events,
                scope.edges,
                pending.len(),
                scope.pointer_slots.len(),
                work,
                MAX_SCAN,
                pending.capacity() * std::mem::size_of::<ScalarRead>()
                    + std::mem::size_of::<ScalarRequests>()
            );
        }
        if work > MAX_SCAN || jaic::interrupt::should_stop() {
            return;
        }
        self.scalar_work_bound = work;
        for read in &pending {
            self.slots.insert(read.slot);
            self.scalar_slots.insert(read.slot);
        }
        self.scalar_reads = pending;
    }

    /// Scalar sources are exposed only at the exact requested typed load event.
    pub fn captures_scalar(&self, site: Site, value: Val, ty: TypeId, slot: usize) -> bool {
        self.scalar_reads.iter().any(|read| {
            read.site == site && read.value == value && read.ty == ty && read.slot == slot
        })
    }

    /// Optional snapshots use exact typed contents, including pointer bit constants.
    pub fn requested(&self, slot: usize) -> bool {
        self.slots.contains(&slot)
    }

    pub fn scalar(&self, slot: usize) -> bool {
        self.scalar_slots.contains(&slot)
    }

    /// Reuse the ordinary transfer for an already admitted immutable scalar inventory.
    pub(super) fn for_scalar_requests(requests: &ScalarRequests) -> Self {
        let slots: HashSet<_> = requests.reads.iter().map(|r| r.slot).collect();
        Self {
            slots: slots.clone(),
            scalar_slots: slots,
            scalar_reads: requests.reads.clone(),
            scalar_work_bound: requests.work_bound,
            ..Self::default()
        }
    }

    pub fn into_scalar_requests(self) -> Option<Box<ScalarRequests>> {
        (!self.scalar_reads.is_empty()).then(|| {
            Box::new(ScalarRequests {
                reads: self.scalar_reads,
                work_bound: self.scalar_work_bound,
            })
        })
    }

    pub fn slots(&self) -> impl Iterator<Item = usize> + '_ {
        self.slots.iter().copied()
    }

    /// Query slots need exact values; ordinary pointer slots keep their MAY join.
    pub fn normalize(&self, state: &mut HashMap<usize, Sources>) {
        for slot in &self.slots {
            if let Some(sources) = state.get_mut(slot)
                && sources.len() != 1
            {
                *sources = Sources::from_iter([Source::Unknown]);
            }
        }
    }

    pub fn join(&self, slot: usize, into: &mut Sources, from: &Sources) -> bool {
        if !self.slots.contains(&slot) {
            let before = into.len();
            into.extend(from);
            return before != into.len();
        }
        if into == from || into.len() == 1 && into.contains(&Source::Unknown) {
            return false;
        }
        *into = Sources::from_iter([Source::Unknown]);
        true
    }

    /// Join only requested prestate slots; never retain a whole memory map per call.
    pub fn observe(&self, site: Site, state: &HashMap<usize, Sources>, out: &mut Snapshots) {
        let Some(&call) = self.starts.get(&site) else {
            return;
        };
        for request in self.calls.get(&call).into_iter().flatten() {
            let into = out
                .entry((call, request.formal))
                .or_insert_with(|| Snapshot {
                    request: request.clone(),
                    prestate: site,
                    sources: std::array::from_fn(|_| Sources::default()),
                });
            for (sources, slot) in into.sources.iter_mut().zip(request.slots) {
                let source = state
                    .get(&slot)
                    .filter(|values| values.len() == 1)
                    .and_then(|values| values.iter().next())
                    .copied()
                    .unwrap_or(Source::Unknown);
                if sources.is_empty() {
                    sources.insert(source);
                } else if !sources.contains(&source) {
                    *sources = Sources::from_iter([Source::Unknown]);
                }
            }
        }
    }
}

/// Only an ordinary call's declared-effect marker is an invocation boundary.
/// Expanded bodies have already run and cannot use this post-expansion marker.
fn call_start(c: &Compiler, body: &LifetimeBody, site: Site) -> Option<Site> {
    let block = body.blocks.get(site.block)?;
    let event = site.event.checked_sub(1)?;
    let Event::Grow {
        region,
        ty,
        call_event,
        ..
    } = block.events.get(event)?
    else {
        return None;
    };
    if *call_event != site.event {
        return None;
    }
    let Event::Call {
        callee,
        args,
        expands: false,
        ..
    } = block.events.get(*call_event)?
    else {
        return None;
    };
    let current = c.ed.facts()?.lifetime_bodies.get(callee)?;
    if !current_body(c, *callee, current) {
        return None;
    }
    let mut first = args.iter().filter(|arg| arg.formal == 0);
    let arg = first.next()?;
    if first.next().is_some()
        || arg.spread
        || c.ty.types.pointee(arg.ty) != Some(*ty)
        || !matches!(arg.operand,LifetimeOperand::Value(value) if value==*region)
    {
        return None;
    }
    Some(Site {
        block: site.block,
        event,
    })
}

impl Snapshot {
    /// Every reaching alternative must agree on this checked signed count value.
    fn integer(&self, slot: usize) -> Option<i64> {
        let mut chosen = None;
        for source in &self.sources[slot] {
            let Source::Constant {
                ty: TypeId::S64,
                value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits),
            } = source
            else {
                return None;
            };
            if chosen.is_some_and(|old| old != *bits) {
                return None;
            }
            chosen = Some(*bits);
        }
        chosen.map(|bits| bits as i64)
    }

    pub fn bounds(&self) -> Option<(i64, i64)> {
        Some((self.integer(0)?, self.integer(1)?))
    }

    /// Exact typed null bits at this invocation snapshot, never pointer provenance.
    pub fn null_data(&self, c: &Compiler) -> bool {
        self.sources[2].len() == 1
            && self.sources[2].iter().all(|source| {
                matches!(source, Source::Constant {ty,value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0)}
                    if c.ty.types.pointee(*ty) == Some(self.request.element))
            })
    }

    pub fn data(&self) -> Option<Val> {
        let mut chosen = None;
        for source in &self.sources[2] {
            let Source::Value(value) = source else {
                return None;
            };
            if chosen.is_some_and(|old| old != *value) {
                return None;
            }
            chosen = Some(*value);
        }
        chosen
    }
}


/// Admission bounds the existing transfer before adding any optional state slots.
struct ScalarScope {
    events: usize,
    edges: usize,
    reads: usize,
    pointer_slots: HashSet<usize>,
}

fn scalar_private(
    c: &Compiler,
    body: &LifetimeBody,
    places: &Places,
    core: &super::prepared::Core<'_>,
    value: Val,
    ty: TypeId,
    bytes: u64,
) -> Option<usize> {
    if !matches!(ty, TypeId::U64 | TypeId::S64 | TypeId::BOOL) && !c.ty.types.is_pointer(ty) {
        return None;
    }
    if bytes == 0 || bytes != c.ty.types.size_of(ty) {
        return None;
    }
    let slot = places.existing(value, ty)?;
    if !places.paths[slot].1.is_empty() {
        return None;
    }
    (places.selected_type(slot) == Some(ty) && core.private_slot(c, body, places, slot))
        .then_some(slot)
}

fn scalar_scope(
    c: &Compiler,
    body: &LifetimeBody,
    places: &Places,
    core: &super::prepared::Core<'_>,
) -> Option<ScalarScope> {
    let mut scope = ScalarScope {
        events: 0,
        edges: 0,
        reads: 0,
        pointer_slots: HashSet::default(),
    };
    let mut branch = None;
    // No allocation until retained block/event/edge/request sizes are bounded.
    for (block, bb) in body.blocks.iter().enumerate() {
        scope.events = scope.events.checked_add(bb.events.len())?;
        scope.edges = scope.edges.checked_add(bb.successors.len())?;
        if scope.events > 128
            || scope.edges > 64
            || bb.successors.iter().any(|s| *s >= body.blocks.len())
        {
            return None;
        }
        for e in &bb.events {
            if matches!(e,Event::Load{ty,..} if matches!(*ty,TypeId::U64|TypeId::S64|TypeId::BOOL))
            {
                scope.reads += 1;
            }
            if scope.reads > 32 || jaic::interrupt::should_stop() {
                return None;
            }
            if let Event::Branch {
                then_block,
                else_block,
                ..
            } = e
                && branch.replace((block, *then_block, *else_block)).is_some()
            {
                return None;
            }
        }
    }
    let (branch, yes, no) = branch?;
    if yes == no
        || body.blocks[branch].successors.len() != 2
        || !body.blocks[branch].successors.contains(&yes)
        || !body.blocks[branch].successors.contains(&no)
        || body.blocks.iter().enumerate().any(|(i, b)| {
            (i != branch && b.successors.len() > 1) || (b.normal_return && !b.successors.is_empty())
        })
    {
        return None;
    }
    for bb in &body.blocks {
        for (ordinal, e) in bb.events.iter().enumerate() {
            let access = match e {
                Event::Load {
                    place,
                    ty,
                    ..
                }
                | Event::Store {
                    place,
                    ty,
                    ..
                }
                | Event::Initialize {
                    place,
                    ty,
                    ..
                } => Some((*place, *ty, c.ty.types.size_of(*ty))),
                Event::DefaultImage {
                    place,
                    ty,
                    bytes,
                    ..
                } => {
                    if ordinal == 0
                        || !matches!(&bb.events[ordinal-1],Event::Initialize{place:p,ty:t,..} if p==place && t==ty)
                    {
                        return None;
                    }
                    Some((*place, *ty, *bytes))
                }
                // This first request scope has no address publication at all. Typed
                // PlaceId inequality is therefore never used as exposure evidence.
                Event::Address {
                    ..
                }
                | Event::Call {
                    ..
                }
                | Event::IndirectCall {
                    ..
                }
                | Event::Barrier
                | Event::Grow {
                    ..
                }
                | Event::BoxAny {
                    ..
                }
                | Event::Field {
                    ..
                }
                | Event::FieldProjection {
                    ..
                }
                | Event::Index {
                    ..
                }
                | Event::Indirect {
                    ..
                }
                | Event::ReadMemory {
                    ..
                }
                | Event::Iteration {
                    ..
                } => return None,
                _ => None,
            };
            if let Some((value, ty, bytes)) = access {
                let slot = scalar_private(c, body, places, core, value, ty, bytes)?;
                if c.ty.types.is_pointer(ty) {
                    scope.pointer_slots.insert(slot);
                }
            }
        }
    }
    let mut covered = vec![false; body.blocks.len()];
    let live = super::flow::Live::all(body);
    for excluded in [yes, no] {
        let mut repeated = false;
        let states = super::flow::states_except(
            body,
            &live,
            (),
            &HashSet::from_iter([(branch, excluded)]),
            |(), ()| {
                repeated = true;
                false
            },
            |(), _, _| {},
        )?;
        if repeated
            || states[branch].is_none()
            || !body
                .blocks
                .iter()
                .zip(&states)
                .any(|(b, s)| b.normal_return && s.is_some())
        {
            return None;
        }
        for (seen, state) in covered.iter_mut().zip(states) {
            *seen |= state.is_some();
        }
    }
    // Entry plus closure under every ordinary successor proves exact union coverage.
    if !covered.first().copied().unwrap_or(false)
        || body
            .blocks
            .iter()
            .enumerate()
            .any(|(i, b)| covered[i] && b.successors.iter().any(|s| !covered[*s]))
    {
        return None;
    }
    Some(scope)
}

impl ScalarScope {
    /// Reserve all three storage runs, including full state clones, source sets
    /// of at most two paths, query searches and the old per-Store Parameter scan.
    fn reservation(
        &self,
        c: &Compiler,
        body: &LifetimeBody,
        places: &Places,
        reads: &[ScalarRead],
    ) -> Option<usize> {
        let scalar_slots: HashSet<_> = reads.iter().map(|r| r.slot).collect();
        let slots = self.pointer_slots.len().checked_add(scalar_slots.len())?;
        let memory = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter(|e| {
                matches!(
                    e,
                    Event::Load { .. }
                        | Event::Store { .. }
                        | Event::Initialize { .. }
                        | Event::DefaultImage { .. }
                )
            })
            .count();
        let scans = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter(|e| match e {
                Event::Store {
                    place,
                    ty,
                    ..
                } => {
                    !c.ty.types.is_pointer(*ty)
                        && !places
                            .existing(*place, *ty)
                            .is_some_and(|p| scalar_slots.contains(&p))
                }
                _ => false,
            })
            .count();
        let path_cost = self
            .events
            .checked_add(self.edges)?
            .checked_add(body.blocks.len())?;
        let collection = self
            .events
            .checked_mul(4)?
            .checked_add(self.edges)?
            .checked_add(self.reads.checked_mul(12)?)?;
        let transfer = memory
            .checked_mul(slots.checked_mul(3)?.checked_add(1)?)?
            .checked_add(self.reads.checked_mul(reads.len())?)?
            .checked_add(self.events)?;
        let clone_join = (body.blocks.len() + self.edges)
            .checked_mul(slots.checked_mul(6)?.checked_add(reads.len() * 2 + 2)?)?;
        collection
            .checked_add(path_cost.checked_mul(2)?)?
            .checked_add(
                transfer
                    .checked_add(clone_join)?
                    .checked_add(scans.checked_mul(self.events)?)?
                    .checked_mul(4)?,
            )
    }
}
