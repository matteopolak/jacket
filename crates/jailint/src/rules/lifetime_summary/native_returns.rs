//! Conditional native return witnesses consumed at an actual guarded allocation call.
//! Control paths reuse flow; memory and resource closure reuse the Frame observer.
use super::address_arithmetic::producers::{Producer, Producers};
use super::{Compiler, Event, HashMap, HashSet, ProcId, Resolver, Summary, TypeId, Val};
use crate::rules::lifetime_movement::facts::Site;
use std::cell::Cell;

mod expressions;
mod paths;
pub(super) mod ranges;
use expressions::{Node, Op, Parser};
use paths::{Trace, feasible, traces};

/// Conditional returned bytes under completed all-path captures.
/// The caller must still establish the original native object's success,
/// liveness and effect obligations; this is not a complete constructor frame.
pub(super) fn captured_object_fields(
    r: &Resolver<'_>,
    all: &HashMap<ProcId, Summary>,
    selected_value: Option<Val>,
    fields: &[(TypeId, crate::rules::lifetime_facts::ByteRange)],
    remaining: &mut usize,
) -> Option<(
    Site,
    ranges::Range,
    Vec<jaic::ide::lifetimes::LifetimeDefaultLeaf>,
    Option<ReturnLifetime>,
)> {
    let index = r.capture_index(remaining)?;
    let inventory = super::prepared::requested_scalar_reads(r, remaining)?;
    let work = Work(Cell::new(MAX_WORK.checked_sub(*remaining)?));
    let outcome = (|| {
        work.charge(r.body.blocks.len() * 2 + r.body.blocks.first()?.events.len() + 1)?;
        let mut edge = |_: usize, to: usize, visited: &mut u32| {
            work.charge(r.body.blocks.get(to)?.events.len() + 2)?;
            let bit = 1u32.checked_shl(u32::try_from(to).ok()?)?;
            if *visited & bit != 0 {
                return None;
            }
            *visited |= bit;
            Some(())
        };
        let reached = super::flow::states_except_edges(
            r.body,
            &super::flow::Live::all(r.body),
            1u32,
            &HashSet::default(),
            |into, from| {
                let _ = work.charge(1);
                let old = *into;
                *into |= *from;
                old != *into
            },
            |_, _, _| {},
            Some(&mut edge),
        )?;
        work.charge(1)?;
        let mut selected_value = selected_value;
        let mut found = None;
        let mut entry_guard = None;
        for (block, b) in r.body.blocks.iter().enumerate() {
            work.charge(b.events.len() + 1)?;
            if reached[block].is_none() {
                continue;
            }
            for (event, e) in b.events.iter().enumerate() {
                if let Event::Return {
                    value: returned,
                    constant,
                    index,
                    ty,
                    ..
                } = e
                {
                    if *index != 0 || !r.compiler.ty.types.is_pointer(*ty) {
                        return None;
                    }
                    if *constant != Some(0) {
                        if selected_value
                            .replace(*returned)
                            .is_some_and(|value| value != *returned)
                        {
                            return None;
                        }
                        if found
                            .replace(Site {
                                block,
                                event,
                            })
                            .is_some()
                        {
                            return None;
                        }
                    }
                }
                if let Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } = e
                {
                    if block == 0 {
                        entry_guard = Some(*condition);
                    }
                    if then_block == else_block || b.successors != [*then_block, *else_block] {
                        return None;
                    }
                }
            }
        }
        let at = found?;
        let value = selected_value?;
        let mut roots = vec![value];
        roots.extend(entry_guard);
        let ty = r
            .compiler
            .ty
            .types
            .pointee(r.compiler.proc(r.body.procedure).sig.as_ref()?.returns[0])?;
        work.charge(r.body.address_casts.len() + r.body.blocks[at.block].events.len())?;
        let mut addresses = r
            .body
            .address_casts
            .iter()
            .filter(|cast| cast.to_pointer && cast.integer_bytes == 8);
        let address = addresses.next()?.value;
        if addresses.next().is_some() || !r.body.blocks[at.block].successors.is_empty() {
            return None;
        }
        let mut image = None;
        for (event, e) in r.body.blocks[at.block]
            .events
            .iter()
            .enumerate()
            .take(at.event)
        {
            if let Event::Store {
                place,
                value,
                ty: selected,
                ..
            } = e
                && *selected == ty
            {
                work.charge(8)?;
                // A compiler-created copy temporary cannot be this returned
                // native object. Do not select it merely for its matching type.
                if r.places.existing(*place, *selected).is_some_and(|slot| {
                    r.places.paths[slot].1.is_empty()
                        && r.private_root(slot, r.compiler.ty.types.size_of(*selected))
                }) {
                    continue;
                }
                work.charge(r.body.read_only_regions.len())?;
                if !r
                    .body
                    .read_only_regions
                    .iter()
                    .any(|region| region.value == *value && region.ty == ty)
                {
                    continue;
                }
                if image
                    .replace(Site {
                        block: at.block,
                        event,
                    })
                    .is_some()
                {
                    return None;
                }
                roots.push(*place);
            }
        }
        let image = image?;
        for e in &r.body.blocks[at.block].events[image.event..at.event] {
            if let Event::Store {
                place,
                ty,
                ..
            } = e
                && r.places
                    .existing(*place, *ty)
                    .is_some_and(|slot| r.places.unresolved(slot))
            {
                roots.push(*place);
            }
        }
        *remaining = MAX_WORK.saturating_sub(work.0.get());
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-bound-candidate address={address:?} image={image:?} at={at:?} roots={} remaining={remaining}",
                roots.len()
            );
        }
        let reads = demanded_reads(r, &index, &inventory.reads, roots, remaining)?;
        let requests = super::call_slots::ScalarRequests {
            reads,
            work_bound: 0,
        };
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-object-demand reads={} remaining={}",
                requests.reads.len(),
                remaining
            );
        }
        let (loads, sources, access, markers) = super::storage_versions::requested_bound_loads(
            r,
            all,
            &requests,
            remaining,
            &index,
            (
                address,
                ty,
                fields,
                image,
                Site {
                    block: at.block,
                    event: r.body.blocks[at.block].events.len(),
                },
            ),
        )?;
        let mut spent = MAX_WORK.checked_sub(*remaining)?;
        let result = ranges::captured_address_range(r, all, &loads, &index, value, &mut spent);
        *remaining = MAX_WORK.saturating_sub(spent);
        let (root, range) = result?;
        let alias = index.relative_write(r, (&loads, Some(address)), (value, ty), remaining, 0)?;
        if alias != (address, 0, r.compiler.ty.types.size_of(ty))
            || r.compiler.ty.types.size_of(ty) > range.bytes
            || !range.covers_relative(access)
        {
            return None;
        }
        *remaining = remaining.checked_sub(1)?;
        let Producer::Call(_, acquisition, 0, _) = index.get(root)? else {
            return None;
        };
        work.0.set(MAX_WORK.checked_sub(*remaining)?);
        let lifetime = return_lifetime(
            r,
            all,
            &loads,
            &index,
            root,
            acquisition,
            at,
            reached[at.block]?,
            &markers,
            &work,
        );
        *remaining = MAX_WORK.checked_sub(work.0.get())?;
        Some((acquisition, range, sources, lifetime))
    })();
    *remaining = (*remaining).min(MAX_WORK.saturating_sub(work.0.get()));
    outcome
}

/// Select requests only. Legacy write rows overselect dependencies; they never
/// stand in for reaching definitions or justify a value in the final graph.
fn demanded_reads(
    r: &Resolver<'_>,
    index: &super::captured_values::QueryIndex,
    candidates: &[super::call_slots::ScalarRead],
    mut pending: Vec<Val>,
    remaining: &mut usize,
) -> Option<Vec<super::call_slots::ScalarRead>> {
    let mut seen = HashSet::default();
    let mut reads = Vec::new();
    while let Some(value) = pending.pop() {
        *remaining = remaining.checked_sub(2)?;
        if !seen.insert(value) {
            continue;
        }
        if seen.len() > 128 {
            return None;
        }
        match index.get(value) {
            Some(Producer::Copy(source) | Producer::Cast(source, _, _) | Producer::Not(source)) => {
                pending.push(source)
            }
            Some(Producer::Binary(_, a, b, _, _, _) | Producer::And(a, b) | Producer::Or(a, b)) => {
                pending.extend([a, b])
            }
            Some(Producer::Comparison(id)) => {
                let cmp = r.body.comparisons.get(id)?;
                pending.extend([cmp.left, cmp.right]);
            }
            Some(Producer::Load(ty, site)) => {
                let (Event::Load {
                    place,
                    value: actual,
                    ty: selected,
                    ..
                }
                | Event::ContextRead {
                    place,
                    value: actual,
                    ty: selected,
                    ..
                }) = r.body.blocks.get(site.block)?.events.get(site.event)?
                else {
                    return None;
                };
                if *actual != value || *selected != ty {
                    return None;
                }
                let slot = r.places.existing(*place, ty)?;
                *remaining = remaining.checked_sub(candidates.len() + r.exposed.len() + 8)?;
                let read = if r.compiler.ty.types.is_pointer(ty) {
                    if !r.places.paths[slot].1.is_empty()
                        || !r.core().private_slot(r.compiler, r.body, &r.places, slot)
                    {
                        if jacket_config::env::LIFETIME_STATS.is_set() {
                            eprintln!(
                                "lifetime-bound-request-stop value={value:?} ty={ty:?} slot={slot} path={:?}",
                                r.places.paths[slot]
                            );
                        }
                        return None;
                    }
                    super::call_slots::ScalarRead {
                        site,
                        value,
                        ty,
                        slot,
                    }
                } else {
                    candidates
                        .iter()
                        .find(|read| read.site == site && read.value == value && read.ty == ty)?
                        .clone()
                };
                let writes = r.writes.get(&slot);
                *remaining = remaining.checked_sub(writes.map_or(0, Vec::len) + 1)?;
                if let Some(writes) = writes {
                    pending.extend(writes.iter().copied());
                }
                reads.push(read);
            }
            Some(Producer::Call(callee, site, _, _)) => {
                let Event::Call {
                    callee: actual,
                    args,
                    ..
                } = r.body.blocks.get(site.block)?.events.get(site.event)?
                else {
                    return None;
                };
                if *actual != callee || args.len() > 8 {
                    return None;
                }
                *remaining = remaining.checked_sub(args.len() + 1)?;
                for arg in args {
                    if arg.constant.is_none()
                        && let super::LifetimeOperand::Value(value) = arg.operand
                    {
                        pending.push(value);
                    }
                }
            }
            Some(Producer::Parameter(..)) => {}
            None => {
                let rows = index.captures(value);
                *remaining = remaining.checked_sub(rows.len())?;
                for row in rows {
                    if let super::captured_values::CaptureRow::Projection(base, ..)
                    | super::captured_values::CaptureRow::Offset(base, ..) = row
                    {
                        pending.push(*base);
                    }
                }
            }
        }
        if jaic::interrupt::should_stop() {
            return None;
        }
    }
    *remaining = remaining.checked_sub(reads.len() + 1)?;
    Some(reads)
}

const MAX_WORK: usize = 8192;
const MAX_PATHS: usize = 16;
const MAX_NODES: usize = 32;

struct Work(Cell<usize>);
impl Work {
    fn charge(&self, n: usize) -> Option<()> {
        let next = self.0.get().saturating_add(n);
        self.0.set(next);
        (next <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) enum Allocation {
    Foreign(super::primitives::Allocation),
    Checked(Checked),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Checked {
    signature: TypeId,
    params: Vec<TypeId>,
    cases: Vec<Case>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PrefixAccess {
    pointer: usize,
    stored_value: Option<usize>,
    alignment: u64,
    offset: i64,
    width: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Case {
    frame: super::frames::Frame,
    nodes: Vec<Node>,
    guards: Vec<(usize, bool)>,
    result: usize,
    native: usize,
    prefix: Vec<PrefixAccess>,
    guarded: bool,
    // Conditional preservation of this fresh unpublished object, not global purity.
    boundaries: Vec<(ProcId, std::rc::Rc<super::contracts::UnpassedStorage>)>,
}

impl Allocation {
    pub fn foreign(&self) -> Option<&super::primitives::Allocation> {
        match self {
            Self::Foreign(foreign) => Some(foreign),
            Self::Checked(_) => None,
        }
    }
    fn collect_requirements(&self, work: &Work) -> Option<Vec<super::frames::Requirement>> {
        let mut out = Vec::new();
        if let Self::Checked(checked) = self {
            work.charge(checked.cases.len() + 1)?;
            for case in &checked.cases {
                let count = case.frame.requirement_count();
                if count > 33 {
                    return None;
                }
                work.charge(count)?;
                out.extend(case.frame.requirements().cloned());
            }
        }
        Some(out)
    }

    fn current(
        &self,
        c: &Compiler,
        id: ProcId,
        all: &HashMap<ProcId, Summary>,
        depth: usize,
        work: &Work,
    ) -> bool {
        if depth >= 8 || work.charge(1).is_none() {
            return false;
        }
        let valid = match self {
            Self::Foreign(contract) => {
                work.charge(16).is_some()
                    && contract.current(c, id)
                    && all.get(&id).is_some_and(|s| {
                        s.may.is_empty()
                            && s.guaranteed.is_empty()
                            && s.returns.is_empty()
                            && s.elements.is_empty()
                            && s.allocation_effects.is_empty()
                            && s.locks.is_none()
                            && !s.observable.exit
                            && !s.observable.output
                            && s.memory.is_none()
                    })
            }
            Self::Checked(checked) => {
                if checked.params.len() > 8
                    || checked.cases.is_empty()
                    || checked.cases.len() > MAX_PATHS
                    || work
                        .charge(checked.params.len() + checked.cases.len() + 1)
                        .is_none()
                {
                    return false;
                }
                let Some(sig) = c.pr.procs.get(id.0 as usize).and_then(|p| p.sig.as_ref()) else {
                    return false;
                };
                if sig.ty != checked.signature
                    || sig.params.len() != checked.params.len()
                    || sig
                        .params
                        .iter()
                        .zip(&checked.params)
                        .any(|(p, t)| p.ty != *t || p.variadic)
                {
                    return false;
                }
                for case in &checked.cases {
                    let identities = case.frame.requirement_count();
                    if identities > 33
                        || case.nodes.len() > MAX_NODES
                        || case.guards.len() > 32
                        || case.prefix.len() > 4
                        || work.charge(case.nodes.len() + identities * 2 + 1).is_none()
                    {
                        return false;
                    }
                    if case
                        .frame
                        .requirements()
                        .next()
                        .is_none_or(|b| b.procedure() != id)
                        || !case.frame.current(c)
                        || !case.frame.compatible(all)
                    {
                        return false;
                    }
                    if case.boundaries.len() > 4 {
                        return false;
                    }
                    for (callee, proof) in &case.boundaries {
                        if work.charge(proof.authentication_work()).is_none()
                            || all
                                .get(callee)
                                .is_none_or(|s| !proof.current(c, *callee, s))
                        {
                            return false;
                        }
                    }
                    for node in &case.nodes {
                        match &node.op {
                            Op::Native(child, _, _) => {
                                let Some(proof) =
                                    all.get(child).and_then(|s| s.native_allocation.as_ref())
                                else {
                                    return false;
                                };
                                if !proof.current(c, *child, all, depth + 1, work) {
                                    return false;
                                }
                            }
                            Op::Bits(child, _, _) => {
                                let Some(proof) =
                                    all.get(child).and_then(|s| s.bit_return.as_ref())
                                else {
                                    return false;
                                };
                                let count = proof.requirement_count();
                                if count > 33
                                    || work.charge(count * 2 + 1).is_none()
                                    || !proof.current(c, all)
                                {
                                    return false;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                true
            }
        };
        valid && work.charge(0).is_some()
    }
}

#[derive(Clone, Copy)]
enum RoleKind {
    Native,
    Bits,
    Prefix,
    PreservesUnpassed,
}
struct Role {
    site: Site,
    callee: ProcId,
    kind: RoleKind,
    identities: Vec<super::frames::Requirement>,
}
fn role(
    r: &Resolver<'_>,
    all: &HashMap<ProcId, Summary>,
    callee: ProcId,
    site: Site,
    work: &Work,
) -> Option<Role> {
    work.charge(1)?;
    let summary = all.get(&callee)?;
    let (kind, identities) = if let Some(proof) = &summary.native_allocation {
        if !proof.current(r.compiler, callee, all, 0, work) {
            return None;
        }
        (RoleKind::Native, proof.collect_requirements(work)?)
    } else if let Some(proof) = &summary.bit_return {
        let count = proof.requirement_count();
        if count > 33 {
            return None;
        }
        work.charge(count * 2 + 1)?;
        if !proof.current(r.compiler, all) {
            return None;
        }
        work.charge(count)?;
        (RoleKind::Bits, proof.requirements().cloned().collect())
    } else if let Some(proof) = &summary.external_preservation {
        work.charge(proof.authentication_work())?;
        if !proof.current(r.compiler, callee, summary) {
            return None;
        }
        let Event::Call {
            args,
            results,
            expands: false,
            ..
        } = r.body.blocks.get(site.block)?.events.get(site.event)?
        else {
            return None;
        };
        if !args.is_empty()
            || results.len() != 1
            || results[0].0 != 0
            || results[0].2 != TypeId::S32
        {
            return None;
        }
        (RoleKind::PreservesUnpassed, Vec::new())
    } else {
        let frame = summary.frame.as_ref()?;
        let count = frame.requirement_count();
        if count > 33 {
            return None;
        }
        work.charge(count * 2 + 1)?;
        if frame.raw().is_none() || !frame.current(r.compiler) || !frame.compatible(all) {
            return None;
        }
        work.charge(count)?;
        (RoleKind::Prefix, frame.requirements().cloned().collect())
    };
    Some(Role {
        site,
        callee,
        kind,
        identities,
    })
}

struct Pending {
    trace: Trace,
    roles: Vec<std::rc::Rc<Role>>,
}

/// Borrow the charged index and its live monotone counter as one observer input.
/// Construction stays private to the native proof's actual entry path.
pub(super) struct ObserverStart<'a> {
    work: &'a Work,
    index: &'a super::captured_values::QueryIndex,
}
impl ObserverStart<'_> {
    pub(super) fn spent(&self) -> usize {
        self.work.0.get()
    }
    pub(super) fn index(&self) -> &super::captured_values::QueryIndex {
        self.index
    }
}

pub(super) struct Builder {
    work: Work,
    producers: super::captured_values::QueryIndex,
    pending: Vec<Pending>,
    blocks: Vec<Option<super::frames::Builder>>,
    template: super::frames::Builder,
    roles: Vec<std::rc::Rc<Role>>,
    signature: TypeId,
    params: Vec<TypeId>,
}

impl Builder {
    pub fn new(r: &Resolver<'_>, _all: &HashMap<ProcId, Summary>, recursive: bool) -> Option<Self> {
        if recursive {
            return None;
        }
        let inventory = r.core().native_reads.as_ref()?;
        let work = Work(Cell::new(inventory.collection_work));
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-start proc={} reads={} blocks={} work={}",
                r.body.procedure.0,
                inventory.reads.len(),
                r.body.blocks.len(),
                work.0.get()
            );
        }
        let traces = traces(r, &work)?;
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-traces proc={} traces={} work={}",
                r.body.procedure.0,
                traces.len(),
                work.0.get()
            );
        }
        let mut remaining = MAX_WORK.checked_sub(work.0.get())?;
        let producers = r.capture_index(&mut remaining);
        work.0.set(MAX_WORK - remaining);
        let producers = producers?;
        let mut pending = Vec::new();
        let start = ObserverStart {
            work: &work,
            index: &producers,
        };
        let mut template = super::frames::Builder::native_return(r, &start);
        work.0.set(template.spent());
        if !template.active() {
            return None;
        }
        for trace in traces {
            if !feasible(&trace, &producers, &work)? {
                continue;
            }
            if trace.constant.is_some() {
                continue;
            }
            work.charge(1)?;
            pending.push(Pending {
                trace,
                roles: Vec::new(),
            });
        }
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-paths proc={} pending={} work={}",
                r.body.procedure.0,
                pending.len(),
                work.0.get()
            );
        }
        if pending.is_empty() {
            return None;
        }
        work.charge(r.body.blocks.len() + pending.len() * 2)?;
        let mut outputs = HashMap::default();
        let mut used = 0u32;
        for p in &pending {
            used |= p.trace.block_mask;
            if outputs
                .insert(p.trace.returned.block, p.trace.result)
                .is_some_and(|old| old != p.trace.result)
            {
                return None;
            }
        }
        let fallback = pending[0].trace.result;
        let mut blocks = Vec::with_capacity(r.body.blocks.len());
        for block in 0..r.body.blocks.len() {
            if used & (1u32 << block) != 0 {
                let output = outputs.get(&block).copied().unwrap_or(fallback);
                let frame = template.native_path(work.0.get(), output)?;
                work.0.set(frame.spent());
                blocks.push(Some(frame));
            } else {
                blocks.push(None);
            }
        }
        let sig = r.compiler.proc(r.body.procedure).sig.as_ref()?;
        work.charge(sig.params.len() + 1)?;
        let params = sig.params.iter().map(|p| p.ty).collect();
        Some(Self {
            work,
            producers,
            pending,
            blocks,
            template,
            roles: Vec::new(),
            signature: sig.ty,
            params,
        })
    }

    pub fn observe(
        &mut self,
        r: &Resolver<'_>,
        site: Site,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) {
        if self.work.charge(1).is_none() {
            return;
        }
        let Some(frame) = self.blocks.get_mut(site.block).and_then(Option::as_mut) else {
            return;
        };
        if !frame.active() {
            return;
        }
        if !frame.resume_work(self.work.0.get()) {
            return;
        }
        let role = if let Event::Call {
            callee,
            expands: false,
            ..
        } = event
        {
            role(r, all, *callee, site, &self.work)
        } else {
            None
        };
        if !frame.resume_work(self.work.0.get()) {
            return;
        }
        if let Some(role) = role {
            let _ = frame.represented_call(&role.identities);
            self.roles.push(std::rc::Rc::new(role));
        } else {
            frame.observe(r, site.block, site.event, event, all);
        }
        self.work.0.set(frame.spent());
        if !frame.active() && jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-event-stop proc={} site={:?} event={:?} work={}",
                r.body.procedure.0,
                site,
                event,
                self.work.0.get()
            );
        }
    }

    pub fn finish(
        mut self,
        r: &Resolver<'_>,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Allocation> {
        let mut cases = Vec::new();
        let mut blocks = Vec::with_capacity(self.blocks.len());
        for frame in &mut self.blocks {
            let proof = if let Some(frame) = frame {
                if !frame.resume_work(self.work.0.get()) {
                    return None;
                }
                let proof = frame.finish(r, &super::flow::Live::all(r.body), all);
                self.work.0.set(frame.spent());
                proof
            } else {
                None
            };
            blocks.push(proof);
        }
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-observed proc={} valid={} total={} work={}",
                r.body.procedure.0,
                blocks.iter().flatten().count(),
                blocks.len(),
                self.work.0.get()
            );
        }
        let mut closed = 0u32;
        self.work.charge(blocks.len())?;
        for (block, proof) in blocks.iter().enumerate() {
            if proof.is_some() {
                closed |= 1u32 << block;
            }
        }
        for pending in &mut self.pending {
            self.work.charge(1)?;
            if pending.trace.block_mask & !closed != 0 {
                continue;
            }
            self.work.charge(self.roles.len() + 1)?;
            pending.roles = self
                .roles
                .iter()
                .filter(|role| pending.trace.block_mask & (1u32 << role.site.block) != 0)
                .cloned()
                .collect();
            let mut frame = self
                .template
                .native_path(self.work.0.get(), pending.trace.result)?;
            let mut valid = true;
            for &block in &pending.trace.blocks {
                let Some(proof) = &blocks[block] else {
                    valid = false;
                    break;
                };
                let identities: Vec<_> = proof.requirements().cloned().collect();
                frame.represented_call(&identities)?;
            }
            self.work.0.set(frame.spent());
            if !valid {
                continue;
            }
            let frame = frame.finish(r, &super::flow::Live::all(r.body), all)?;
            let inventory = r.core().native_reads.as_ref()?;
            let reservation = storage_work(r, inventory, &pending.trace)?;
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-native-storage proc={} reserve={} work={}",
                    r.body.procedure.0,
                    reservation,
                    self.work.0.get()
                );
            }
            self.work
                .charge(reservation + inventory.reads.len() * 2 + 1)?;
            let requests = super::call_slots::ScalarRequests {
                reads: inventory.reads.clone(),
                work_bound: reservation,
            };
            let probes = super::call_slots::Probes::for_scalar_requests(&requests);
            let (loads, _) = super::storage_versions::loads_except(
                &r.places,
                r.core(),
                &probes,
                &pending.trace.cuts,
            );
            let mut parser = Parser::new(r, all, &loads, &self.producers, &self.work)?;
            let Some(case) = case(r, all, pending, frame, &mut parser) else {
                if jacket_config::env::LIFETIME_STATS.is_set() {
                    eprintln!(
                        "lifetime-native-case-stop proc={} nodes={} work={}",
                        r.body.procedure.0,
                        parser.nodes.len(),
                        self.work.0.get()
                    );
                    eprintln!("lifetime-native-case-nodes {:?}", parser.nodes);
                }
                continue;
            };
            cases.push(case);
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!(
                    "lifetime-native-case-complete proc={} cases={} work={}",
                    r.body.procedure.0,
                    cases.len(),
                    self.work.0.get()
                );
            }
        }
        self.work.charge(cases.len())?;
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-finish proc={} cases={} work={}",
                r.body.procedure.0,
                cases.len(),
                self.work.0.get()
            );
        }
        // These are conditional, fully selected paths, not existential return
        // alternatives. Applicability requires every recorded Branch predicate
        // to be definite. The first differing edge excludes every other path;
        // inputs that select an unsupported/undetermined path remain Unknown.
        if cases.is_empty() {
            return None;
        }
        let proof = Allocation::Checked(Checked {
            signature: self.signature,
            params: self.params,
            cases,
        });
        (proof.current(r.compiler, r.body.procedure, all, 0, &self.work)).then_some(proof)
    }
}

fn storage_work(
    r: &Resolver<'_>,
    inventory: &super::prepared::NativeReads,
    trace: &Trace,
) -> Option<usize> {
    let pointer = super::storage_versions::pointer_slots(r.compiler, r.body, &r.places);
    let scalar: HashSet<_> = inventory.reads.iter().map(|read| read.slot).collect();
    let slots = pointer.union(&scalar).count();
    let events = r.body.blocks.iter().map(|b| b.events.len()).sum::<usize>();
    let mut writes = 0usize;
    let mut loads = 0usize;
    let mut calls = 0usize;
    let mut entry_work = 0usize;
    let mut selected = 0usize;
    for &block in &trace.blocks {
        for e in &r.body.blocks[block].events {
            selected += 1;
            if matches!(
                e,
                Event::Store { .. } | Event::Initialize { .. } | Event::DefaultImage { .. }
            ) {
                writes += 1;
            }
            if matches!(e, Event::Load { .. }) {
                loads += 1;
            }
            if matches!(
                e,
                Event::Call {
                    expands: false,
                    ..
                } | Event::IndirectCall { .. }
            ) {
                calls += 1;
            }
            if let Event::Store {
                place,
                value,
                ty,
                ..
            } = e
                && !r.compiler.ty.types.is_pointer(*ty)
                && r.places
                    .existing(*place, *ty)
                    .is_none_or(|p| !scalar.contains(&p))
            {
                entry_work = entry_work.checked_add(r.values.get(value).map_or(0, Vec::len))?;
            }
        }
    }
    events
        .checked_mul(3)?
        .checked_add(selected)?
        .checked_add(writes.checked_mul(slots.checked_mul(8)?.checked_add(4)?)?)?
        .checked_add(loads.checked_mul(inventory.reads.len().checked_add(4)?)?)?
        .checked_add(calls.checked_mul(slots.checked_mul(r.exposed.len().checked_add(8)?)?)?)?
        .checked_add(
            trace
                .blocks
                .len()
                .checked_mul(slots.checked_mul(10)?.checked_add(4)?)?,
        )?
        .checked_add(entry_work)
}

fn case(
    r: &Resolver<'_>,
    all: &HashMap<ProcId, Summary>,
    pending: &Pending,
    frame: super::frames::Frame,
    parser: &mut Parser<'_, '_>,
) -> Option<Case> {
    let result = parser.value(pending.trace.result, 0)?;
    if jacket_config::env::LIFETIME_STATS.is_set() {
        eprintln!(
            "lifetime-native-case-result proc={} node={} nodes={:?}",
            r.body.procedure.0, result, parser.nodes
        );
    }
    let native = parser
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| matches!(n.op, Op::Native(..)).then_some(i))
        .collect::<Vec<_>>();
    if native.len() != 1 {
        return None;
    }
    let native = native[0];
    let mut guards = Vec::new();
    parser.work.charge(pending.trace.branches.len() * 2 + 1)?;
    let mut guarded = false;
    for &(site, condition, when) in &pending.trace.branches {
        let id = parser.value(condition, 0)?;
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-case-guard proc={} site={:?} node={} when={} op={:?}",
                r.body.procedure.0, site, id, when, parser.nodes[id]
            );
        }
        let native_guard = native_test(&parser.nodes, id, native);
        // Before success is independently established, a compound predicate
        // cannot smuggle that outcome into the prefix of its own proof.
        if !guarded && native_guard.is_none() && parser.native_dependent[id] {
            return None;
        }
        if let Some(present) = native_guard {
            if present != when {
                return None;
            }
            // The opposite acquisition outcome must not produce any nonnull
            // normal return. This proves success from the caller's real guard.
            // Fix only the evaluated prefix, then inspect EVERY suffix of the
            // opposite native outcome. Later selected-path cuts are not premises.
            parser
                .work
                .charge(pending.trace.branches.len() + pending.trace.blocks.len() + 1)?;
            let mut cuts = HashSet::default();
            for &(prior, _, taken) in &pending.trace.branches {
                if prior == site {
                    break;
                }
                let Event::Branch {
                    then_block,
                    else_block,
                    ..
                } = r.body.blocks[prior.block].events.get(prior.event)?
                else {
                    return None;
                };
                cuts.insert((
                    prior.block,
                    if taken {
                        *else_block
                    } else {
                        *then_block
                    },
                ));
            }
            let Op::Native(_, acquired, _) = parser.nodes[native].op else {
                return None;
            };
            let acquired_block = pending
                .trace
                .blocks
                .iter()
                .position(|b| *b == acquired.block)?;
            let guard_block = pending.trace.blocks.iter().position(|b| *b == site.block)?;
            if acquired_block > guard_block
                || (acquired_block == guard_block && acquired.event >= site.event)
            {
                return None;
            }
            let Event::Branch {
                then_block,
                else_block,
                ..
            } = r.body.blocks[site.block].events.get(site.event)?
            else {
                return None;
            };
            cuts.insert((
                site.block,
                if when {
                    *then_block
                } else {
                    *else_block
                },
            ));
            parser.work.charge(
                r.body
                    .blocks
                    .iter()
                    .map(|b| b.events.len() + b.successors.len() + 1)
                    .sum(),
            )?;
            let states = super::flow::states_except(
                r.body,
                &super::flow::Live::all(r.body),
                (),
                &cuts,
                |(), ()| false,
                |(), _, _| {},
            )?;
            for (b, block) in r.body.blocks.iter().enumerate() {
                if states[b].is_none() || !block.normal_return {
                    continue;
                }
                let mut returns = block
                    .events
                    .iter()
                    .filter(|e| matches!(e, Event::Return { .. }));
                if !matches!(returns.next(),Some(Event::Return{index:0,constant:Some(0),ty,..}) if r.compiler.ty.types.is_pointer(*ty))
                    || returns.next().is_some()
                {
                    return None;
                }
            }
            guarded = true;
            if jacket_config::env::LIFETIME_STATS.is_set() {
                eprintln!("lifetime-native-guarded proc={}", r.body.procedure.0);
            }
        }
        guards.push((id, when));
    }
    let mut boundaries = Vec::new();
    for role in &pending.roles {
        if matches!(role.kind, RoleKind::PreservesUnpassed) {
            if boundaries.len() >= 4 {
                return None;
            }
            let proof = all.get(&role.callee)?.external_preservation.as_ref()?;
            parser.work.charge(proof.authentication_work() + 1)?;
            if !proof.current(r.compiler, role.callee, all.get(&role.callee)?) {
                return None;
            }
            boundaries.push((role.callee, proof.clone()));
        }
    }
    let mut prefix = Vec::new();
    for role in &pending.roles {
        if !matches!(role.kind, RoleKind::Prefix) {
            continue;
        }
        let raw = all.get(&role.callee)?.frame.as_ref()?.raw()?;
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-prefix proc={} raw={:?}",
                r.body.procedure.0, raw
            );
        }
        if raw.reads.len() + raw.writes.len() > 4 {
            return None;
        }
        let args = parser.arguments(role.callee, role.site, 0)?;
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-native-prefix-args proc={} args={:?} native={}",
                r.body.procedure.0, args, native
            );
        }
        parser
            .work
            .charge((raw.reads.len() + raw.writes.len()) * 4 + 1)?;
        for read in &raw.reads {
            prefix.push(PrefixAccess {
                pointer: *args.get(read.pointer.formal)?,
                stored_value: None,
                alignment: r.compiler.ty.types.align_of(read.ty),
                offset: read.pointer.displacement,
                width: read.width,
            });
        }
        for write in &raw.writes {
            if write.value.displacement != 0 {
                return None;
            }
            let pointer = *args.get(write.access.pointer.formal)?;
            let value = *args.get(write.value.formal)?;
            if value != native {
                return None;
            }
            prefix.push(PrefixAccess {
                pointer,
                stored_value: Some(value),
                alignment: r.compiler.ty.types.align_of(write.access.ty),
                offset: write.access.pointer.displacement,
                width: write.access.width,
            });
        }
    }
    // Transforming a null allocation can produce nonzero machine bits. Only an
    // explicit, correctly framed native success edge permits such a return.
    if (result != native || !prefix.is_empty()) && !guarded {
        return None;
    }
    if guards.len() != pending.trace.branches.len()
        || parser
            .nodes
            .iter()
            .filter(|n| matches!(n.op, Op::Native(..)))
            .count()
            != 1
    {
        return None;
    }
    if parser.nodes.iter().any(
        |node| matches!(node.op,Op::Bits(_,site,_) if !pending.roles.iter().any(|r|r.site==site)),
    ) {
        return None;
    }
    let _ = r;
    Some(Case {
        frame,
        nodes: std::mem::take(&mut parser.nodes),
        guards,
        result,
        native,
        prefix,
        guarded,
        boundaries,
    })
}

/// `true` denotes present native storage, not an arbitrary pointer's liveness.
fn native_test(nodes: &[Node], id: usize, native: usize) -> Option<bool> {
    match &nodes.get(id)?.op {
        Op::Not(inner) => Some(!native_test(nodes, *inner, native)?),
        Op::Compare(op, a, b)
            if (*a == native && matches!(nodes.get(*b)?.op, Op::Constant(0)))
                || (*b == native && matches!(nodes.get(*a)?.op, Op::Constant(0))) =>
        {
            match op {
                jaic::ir::CmpOp::Eq => Some(false),
                jaic::ir::CmpOp::Ne => Some(true),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests;

/// The selected native instance succeeds and survives to this return. This does
/// not establish no-publication or preservation after the caller receives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ReturnLifetime {
    requirements: Vec<super::frames::Requirement>,
    natives: Vec<(ProcId, std::rc::Rc<Allocation>)>,
}
impl ReturnLifetime {
    pub(super) fn current(
        &self,
        c: &Compiler,
        all: &HashMap<ProcId, Summary>,
        remaining: &mut usize,
    ) -> bool {
        let Some(spent) = MAX_WORK.checked_sub(*remaining) else {
            return false;
        };
        let work = Work(Cell::new(spent));
        let valid = self.requirements.len() <= 33
            && self.natives.len() <= 8
            && work
                .charge(self.requirements.len() * 2 + self.natives.len() + 1)
                .is_some()
            && self.requirements.iter().all(|requirement| {
                requirement.current(c)
                    && all
                        .get(&requirement.procedure())
                        .is_some_and(|s| s.trusted.is_none())
            })
            && self
                .natives
                .iter()
                .all(|(id, contract)| contract.preserves_existing(c, *id, all, &work));
        *remaining = MAX_WORK.saturating_sub(work.0.get());
        valid && !jaic::interrupt::should_stop()
    }
}
impl Allocation {
    /// Only authenticated malloc preserves an already-live client allocation.
    /// The selected declaration's independent effects remain constraints.
    fn preserves_existing(
        &self,
        c: &Compiler,
        id: ProcId,
        all: &HashMap<ProcId, Summary>,
        work: &Work,
    ) -> bool {
        if !matches!(self, Self::Foreign(_)) || !self.current(c, id, all, 0, work) {
            return false;
        }
        let Some(p) = c.pr.procs.get(id.0 as usize) else {
            return false;
        };
        let Some(s) = all.get(&id) else {
            return false;
        };
        p.lit.header.notes.is_empty()
            && c.pr.proc_decl_notes.get(&id).is_none_or(Vec::is_empty)
            && !s.changes_allocator
            && !s.pushes_context
            && s.external_preservation.is_none()
            && s.return_fields.is_empty()
            && s.may_return_fields.is_empty()
            && s.pointer_returns.is_empty()
            && s.value_writes.is_empty()
            && s.reads.is_empty()
            && s.may_reads.is_empty()
            && s.unknown_write.is_empty()
            && s.escapes.is_empty()
            && s.allocations.is_empty()
            && s.may_returns.is_empty()
            && s.adjusted_returns.is_empty()
            && s.ordering.is_none()
    }
}

/// Audit only the existing nonnull-return predecessor mask. Byte effects remain
/// the ordered transfer's responsibility; this checks releases/callbacks.
fn return_lifetime<'c>(
    r: &Resolver<'c>,
    all: &HashMap<ProcId, Summary>,
    loads: &super::storage_versions::CapturedLoads<'c>,
    index: &super::captured_values::QueryIndex,
    root: Val,
    acquisition: Site,
    at: Site,
    mask: u32,
    markers: &HashSet<Site>,
    work: &Work,
) -> Option<ReturnLifetime> {
    let mut stage = "acquisition";
    let result = (|| {
        if acquisition.block != 0 || at.block == 0 || all.get(&r.body.procedure)?.trusted.is_some()
        {
            return None;
        }
        let Producer::Call(callee, site, 0, _) = index.get(root)? else {
            return None;
        };
        if site != acquisition {
            return None;
        }
        let contract = all.get(&callee)?.native_allocation.as_ref()?;
        if !contract.preserves_existing(r.compiler, callee, all, work) {
            return None;
        }
        work.charge(2)?;
        let mut proof = ReturnLifetime {
            requirements: vec![super::frames::Requirement::Body(
                crate::rules::lifetime_facts::BodyIdentity::capture(r.compiler, r.body)?,
            )],
            natives: vec![(callee, contract.clone())],
        };
        let entry = r.body.blocks.first()?;
        work.charge(entry.events.len() + 1)?;
        let (branch, condition, then_block, else_block) = entry
            .events
            .iter()
            .enumerate()
            .find_map(|(event, e)| match e {
                Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } => Some((event, *condition, *then_block, *else_block)),
                _ => None,
            })?;
        if branch <= acquisition.event || entry.successors != [then_block, else_block] {
            return None;
        }
        stage = "native-null-guard";
        let mut parser = Parser::new(r, all, loads, index, work)?;
        stage = "native-root";
        let native = parser.value(root, 0)?;
        stage = "native-condition";
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-return-condition value={condition:?} producer={:?} sources={:?}",
                index.get(condition).map(|row| std::mem::discriminant(&row)),
                loads.get(&condition)
            );
        }
        let condition = parser.value(condition, 0)?;
        stage = "native-test";
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-return-guard native={native} condition={condition} nodes={:?} mask={mask:x}",
                parser.nodes
            );
        }
        let truth = native_test(&parser.nodes, condition, native)?;
        let (success, failure) = if truth {
            (then_block, else_block)
        } else {
            (else_block, then_block)
        };
        if success == failure
            || mask & 1u32.checked_shl(u32::try_from(success).ok()?)? == 0
            || mask & 1u32.checked_shl(u32::try_from(failure).ok()?)? != 0
        {
            return None;
        }
        stage = "interval";
        for (block, b) in r.body.blocks.iter().enumerate() {
            work.charge(1)?;
            if mask & 1u32.checked_shl(u32::try_from(block).ok()?)? == 0 {
                continue;
            }
            for (event, e) in b.events.iter().enumerate() {
                if block == 0 && event <= acquisition.event
                    || block == at.block && event >= at.event
                {
                    continue;
                }
                work.charge(1)?;
                let site = Site {
                    block,
                    event,
                };
                if jacket_config::env::LIFETIME_STATS.is_set()
                    && matches!(
                        e,
                        Event::Call { .. }
                            | Event::IndirectCall { .. }
                            | Event::Barrier
                            | Event::Grow { .. }
                    )
                {
                    eprintln!(
                        "lifetime-return-interval site={site:?} event={e:?} work={}",
                        work.0.get()
                    );
                }
                match e {
                    Event::Call {
                        callee,
                        expands: true,
                        ..
                    } => {
                        if all.get(callee)?.trusted.is_some() || proof.requirements.len() == 33 {
                            return None;
                        }
                        work.charge(1)?;
                        proof
                            .requirements
                            .push(super::frames::Requirement::Constraint(*callee));
                    }
                    Event::Call {
                        callee, ..
                    } => {
                        let summary = all.get(callee)?;
                        if let Some(contract) = &summary.native_allocation {
                            if proof.natives.len() == 8
                                || !contract.preserves_existing(r.compiler, *callee, all, work)
                            {
                                return None;
                            }
                            work.charge(1)?;
                            proof.natives.push((*callee, contract.clone()));
                        } else {
                            let frame = summary.frame.as_ref()?;
                            let sig = r.compiler.proc(*callee).sig.as_ref()?;
                            let count = frame.requirement_count();
                            if count > 33 || proof.requirements.len().checked_add(count)? > 33 {
                                return None;
                            }
                            work.charge(count * 3 + 1)?;
                            let typed = frame.typed()?;
                            if sig.has_context
                                || !sig.params.is_empty()
                                || frame.condition.is_some()
                                || !typed.reads.is_empty()
                                || !typed.writes.is_empty()
                                || !frame.current(r.compiler)
                                || !frame.compatible(all)
                            {
                                return None;
                            }
                            proof.requirements.extend(frame.requirements().cloned());
                        }
                    }
                    Event::IndirectCall {
                        ..
                    }
                    | Event::Grow {
                        ..
                    } => return None,
                    Event::Barrier if !markers.contains(&site) => return None,
                    _ => {}
                }
            }
        }
        if jaic::interrupt::should_stop() {
            None
        } else {
            Some(proof)
        }
    })();
    if result.is_none() && jacket_config::env::LIFETIME_STATS.is_set() {
        eprintln!("lifetime-return-stop stage={stage} work={}", work.0.get());
    }
    result
}
