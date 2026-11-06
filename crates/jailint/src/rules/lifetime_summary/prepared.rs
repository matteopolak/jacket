//! Body definitions shared after normal-return reachability settles.
//! The target universe and live blocks are part of preparation's validity boundary;
//! summary-dependent allocator context and queries stay in each resolver.
use super::{
    Def, Event, HashMap, HashSet, LifetimeBody, LifetimeOperand, Path, Places, ProcId, Summary,
    Val, allocator_context, allocators, interfaces, observable, returning_blocks,
};
use crate::rules::lifetime_facts::{CheckedBody, CheckedSession};
use jaic::sema::Compiler;
use std::rc::Rc;

pub(in crate::rules) struct Resolver<'a> {
    pub places: Places,
    pub(super) allocators: &'a allocators::Allocators,
    pub(super) context: allocator_context::Context,
    pub(super) allocation_mode: std::cell::Cell<bool>,
    pub(super) storage_mode: std::cell::Cell<bool>,
    pub(super) return_mode: std::cell::Cell<bool>,
    pub(super) return_definitions: std::cell::OnceCell<(HashSet<usize>, HashSet<usize>)>,
    pub(super) may_field_origins: std::cell::RefCell<HashMap<super::FieldQuery, bool>>,
    pub(super) field_origins: std::cell::RefCell<HashMap<super::FieldQuery, Path>>,
    pub(super) observable_calls: observable::Calls<'a>,
    pub(super) ordering_base: Option<super::Ordering>,
    pub(super) rounded_address_base: Option<usize>,
    pub(super) comparisons:
        std::cell::OnceCell<HashMap<Val, &'a jaic::ide::lifetimes::LifetimeComparison>>,
    pub(super) return_cuts: std::cell::RefCell<HashMap<usize, bool>>,
    pub(super) return_contracts: HashMap<usize, (Path, Option<(usize, u64)>)>,
    core: std::rc::Rc<Core<'a>>,
    pub(super) targets: &'a super::super::lifetime_targets::Universe,
}

pub(in crate::rules) struct Core<'a> {
    pub(super) compiler: &'a Compiler,
    pub(super) body: &'a LifetimeBody,
    pub(super) context_slot: Option<Val>,
    capture_index: std::cell::OnceCell<super::captured_values::IndexResult>,
    pub(super) storage_versions: super::storage_versions::CapturedLoads<'a>,
    pub(super) call_slots: super::call_slots::Snapshots,
    pub(super) scalar_requests: Option<Box<super::call_slots::ScalarRequests>>,
    pub(super) native_reads: Option<Box<NativeReads>>,
    pub(super) load_sources: HashMap<jaic::ir::Val, jaic::ir::Val>,
    pub(super) write_sources: HashMap<usize, jaic::ir::Val>,
    pub(super) nullable_sources: HashMap<usize, jaic::ir::Val>,
    // Calls in expansions can share both a callee SSA value and a source span.
    pub(super) interfaces: HashMap<*const Event, Summary>,
    pub(super) address_casts: HashMap<jaic::ir::Val, &'a jaic::ide::lifetimes::LifetimeAddressCast>,
    pub(super) values: HashMap<jaic::ir::Val, Vec<Def<'a>>>,
    pub(super) writes: HashMap<usize, Vec<jaic::ir::Val>>,
    pub(super) params: HashMap<usize, (usize, jaic::types::TypeId)>,
    pub(super) exposed: HashSet<usize>,
    pub(super) owned: HashSet<usize>,
    /// Original extents for logical private slot roots, including checked parameter spills.
    pub(super) private_bytes: HashMap<usize, u64>,
    pub(super) indirect_types: HashMap<jaic::ir::Val, jaic::types::TypeId>,
}

/// A body's definitions and places before any summary-dependent context is added.
#[derive(Clone)]
pub(in crate::rules) struct Prepared<'a> {
    core: Rc<Core<'a>>,
    places: Places,
}

impl<'a> Prepared<'a> {
    pub fn build(
        checked: CheckedBody<'a>,
        all: &HashMap<ProcId, Summary>,
        targets: &super::super::lifetime_targets::Universe,
    ) -> Self {
        Self::build_requested(checked, all, targets, false)
    }

    /// Native inventories use stable checked-call demand, never summary completeness.
    pub(super) fn build_requested(
        checked: CheckedBody<'a>,
        all: &HashMap<ProcId, Summary>,
        targets: &super::super::lifetime_targets::Universe,
        native_requested: bool,
    ) -> Self {
        let compiler = checked.compiler();
        let body = checked.body();
        let timer = super::super::lifetime_stats::Timer::start();
        let mut places = Places::new(compiler, body);
        let mut probes = super::call_slots::Probes::collect(compiler, body, &mut places);
        let selector = context_slot(compiler, body, &mut places);
        let mut core = Core {
            compiler,
            body,
            context_slot: selector,
            capture_index: std::cell::OnceCell::new(),
            storage_versions: super::storage_versions::CapturedLoads::empty(checked),
            call_slots: HashMap::default(),
            scalar_requests: None,
            native_reads: None,
            load_sources: HashMap::default(),
            write_sources: HashMap::default(),
            nullable_sources: HashMap::default(),
            interfaces: interfaces::collect(body),
            address_casts: body.address_casts.iter().map(|c| (c.value, c)).collect(),
            values: HashMap::default(),
            writes: HashMap::default(),
            params: HashMap::default(),
            exposed: HashSet::default(),
            owned: HashSet::default(),
            private_bytes: HashMap::default(),
            indirect_types: body
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .filter_map(|e| match e {
                    Event::IndirectCall {
                        callee,
                        ty,
                        ..
                    } => Some((*callee, *ty)),
                    _ => None,
                })
                .collect(),
        };
        let live = returning_blocks(body, all, targets);
        for e in body
            .blocks
            .iter()
            .enumerate()
            .filter(|(i, _)| live.contains(*i))
            .flat_map(|(_, b)| &b.events)
        {
            match e {
                Event::Expression {
                    operand: LifetimeOperand::Place(v),
                    ty,
                    entity: Some(id),
                    ..
                } if matches!(
                    compiler.entity(*id).kind,
                    jaic::sema::scope::EntityKind::Local { .. }
                ) =>
                {
                    let p = places.place(*v, *ty);
                    core.owned.insert(places.root_of(p));
                }
                Event::Parameter {
                    formal,
                    place,
                    incoming,
                    ty,
                    ..
                } => {
                    let p = places.place(*place, *ty);
                    core.params.insert(p, (*formal, *ty));
                    core.owned.insert(p);
                    core.values
                        .entry(*incoming)
                        .or_default()
                        .push(Def::Parameter(*formal, *place, *ty));
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
                    if matches!(e, Event::ContextRead { .. }) && core.context_slot != Some(*place) {
                        continue;
                    }
                    let p = places.place(*place, *ty);
                    if value == place && compiler.ir_ty(*ty).is_none() {
                        // An aggregate rvalue is its address. Preserve plain aggregate
                        // copies, but do not add a self-reference to a call's SSA result.
                        if !core.values.get(value).is_some_and(|defs| {
                            defs.iter()
                                .any(|d| matches!(d, Def::Call(..) | Def::Resize(..)))
                        }) {
                            core.values.entry(*value).or_default().push(Def::Address(p));
                        }
                        continue;
                    }
                    core.values
                        .entry(*value)
                        .or_default()
                        .push(Def::Load(p, *ty));
                }
                Event::Store {
                    place,
                    value,
                    ty,
                    ..
                }
                | Event::ContextSet {
                    place,
                    value,
                    ty,
                    ..
                } => {
                    if matches!(e, Event::ContextSet { .. }) && core.context_slot != Some(*place) {
                        continue;
                    }
                    let p = places.place(*place, *ty);
                    core.writes.entry(p).or_default().push(*value);
                    if compiler.ir_ty(*ty).is_none()
                        && place != value
                        && !core.values.contains_key(place)
                    {
                        core.values.insert(*place, vec![Def::Address(p)]);
                    }
                }
                Event::Copy {
                    value,
                    source,
                }
                | Event::Offset {
                    value,
                    source,
                    ..
                } if value != source => core
                    .values
                    .entry(*value)
                    .or_default()
                    .push(Def::Copy(*source)),
                Event::Address {
                    place,
                    ty,
                    ..
                } => {
                    let p = places.place(*place, *ty);
                    core.values.entry(*place).or_default().push(Def::Address(p));
                    core.exposed.insert(p);
                }
                Event::View {
                    value,
                    source,
                    from,
                    to,
                    ..
                } => {
                    if compiler.ty.types.is_pointer(*from) && compiler.ty.types.is_pointer(*to) {
                        if value != source {
                            core.values
                                .entry(*value)
                                .or_default()
                                .push(Def::Copy(*source));
                        }
                    } else {
                        let p = places.place(*source, *from);
                        let target = places.place(*value, *to);
                        if target != p {
                            core.values.entry(*value).or_default().push(Def::Borrow(p));
                        }
                    }
                }
                Event::Borrow {
                    value,
                    region,
                    region_ty,
                    ..
                } => {
                    let p = places.place(*region, *region_ty);
                    core.values.entry(*value).or_default().push(Def::Borrow(p));
                }
                Event::Index {
                    base,
                    result,
                    base_ty,
                    ..
                } => {
                    let p = places.place(*base, *base_ty);
                    core.values.entry(*result).or_default().push(Def::Borrow(p));
                }
                Event::Call {
                    callee,
                    args,
                    results,
                    ..
                } => {
                    for &(i, v, _) in results {
                        core.values
                            .entry(v)
                            .or_default()
                            .push(Def::Call(*callee, args, i));
                    }
                }
                Event::IndirectCall {
                    allocator: true,
                    callee,
                    args,
                    results,
                    ..
                } => {
                    for &(_, v, _) in results {
                        core.values
                            .entry(v)
                            .or_default()
                            .push(Def::Resize(*callee, args));
                    }
                }
                Event::IndirectCall {
                    callee,
                    args,
                    results,
                    allocator: false,
                    ..
                } => {
                    for &(index, v, _) in results {
                        core.values
                            .entry(v)
                            .or_default()
                            .push(Def::Indirect(*callee, args, index));
                    }
                }
                _ => {}
            }
        }

        core.finish_definition_index(compiler, body, &places);
        if native_requested {
            core.native_reads = native_reads(compiler, body, &places, &core);
        }
        probes.request_scalars(compiler, body, &places, &core);
        let (loads, calls) = super::storage_versions::loads(&places, &core, &probes);
        core.storage_versions = loads;
        core.call_slots = calls;
        core.scalar_requests = probes.into_scalar_requests();
        if jacket_config::env::LIFETIME_STATS.is_set()
            && let Some(requests) = &core.scalar_requests
        {
            eprintln!(
                "lifetime-scalar-retained proc={} requests={} work_bound={} payload_bytes={}",
                body.procedure.0,
                requests.reads.len(),
                requests.work_bound,
                std::mem::size_of_val(requests.as_ref())
                    + requests.reads.capacity()
                        * std::mem::size_of::<super::call_slots::ScalarRead>()
            );
        }
        timer.finish(
            "prepare",
            Some(body.procedure),
            body.blocks.iter().map(|b| b.events.len()).sum(),
        );
        Self {
            core: Rc::new(core),
            places,
        }
    }
}

/// Prepared bodies shared by the passes of one summary build.
pub(super) struct Cache<'a> {
    session: CheckedSession<'a>,
    bodies: HashMap<ProcId, Prepared<'a>>,
}
impl<'a> Cache<'a> {
    pub fn new(compiler: &'a Compiler) -> Self {
        Self {
            session: CheckedSession::new(compiler),
            bodies: HashMap::default(),
        }
    }
    pub fn prepare_requested(
        &mut self,
        id: ProcId,
        all: &HashMap<ProcId, Summary>,
        targets: &super::super::lifetime_targets::Universe,
        native_requested: bool,
    ) -> &Prepared<'a> {
        self.bodies.entry(id).or_insert_with(|| {
            // Callers enumerate the already validated, immutable retained registry.
            let checked = self
                .session
                .body(id)
                .expect("validated summary body remains registered");
            Prepared::build_requested(checked, all, targets, native_requested)
        })
    }
}


impl<'a> Prepared<'a> {
    pub fn for_body(checked: CheckedBody<'a>, summaries: &super::Summaries) -> Self {
        Self::build(checked, &summaries.procs, &summaries.targets)
    }
}

impl<'a> Resolver<'a> {
    pub fn for_prepared(prepared: &Prepared<'a>, summaries: &'a super::Summaries) -> Self {
        Self::from_prepared(
            prepared,
            &summaries.procs,
            &summaries.targets,
            &summaries.allocators,
        )
    }

    /// A resolver for `id`, preparing its body the first time any pass asks.
    pub(super) fn cached<'c: 'a>(
        cache: &mut Cache<'c>,
        id: ProcId,
        all: &HashMap<ProcId, Summary>,
        targets: &'a super::super::lifetime_targets::Universe,
        allocators: &'a allocators::Allocators,
    ) -> Self {
        let prepared = cache.prepare_requested(id, all, targets, false);
        Self::from_prepared(prepared, all, targets, allocators)
    }

    /// A resolver over `prepared`, with the context that depends on `all` built fresh.
    pub fn from_prepared(
        prepared: &Prepared<'a>,
        all: &HashMap<ProcId, Summary>,
        targets: &'a super::super::lifetime_targets::Universe,
        allocators: &'a allocators::Allocators,
    ) -> Self {
        let compiler = prepared.core.compiler;
        let body = prepared.core.body;
        let timer = super::super::lifetime_stats::Timer::start();
        let mut this = Self {
            places: prepared.places.clone(),
            allocators,
            context: allocator_context::Context::default(),
            allocation_mode: std::cell::Cell::new(false),
            storage_mode: std::cell::Cell::new(false),
            return_mode: std::cell::Cell::new(false),
            return_definitions: std::cell::OnceCell::new(),
            may_field_origins: std::cell::RefCell::new(HashMap::default()),
            field_origins: std::cell::RefCell::new(HashMap::default()),
            observable_calls: observable::Calls::build(compiler, body, all),
            ordering_base: all.get(&body.procedure).and_then(|s| s.ordering),
            rounded_address_base: None,
            comparisons: std::cell::OnceCell::new(),
            return_cuts: std::cell::RefCell::new(HashMap::default()),
            return_contracts: all
                .get(&body.procedure)
                .map(|s| s.returns.clone())
                .unwrap_or_default(),
            core: Rc::clone(&prepared.core),
            targets,
        };
        this.rounded_address_base = this.rounded_address(all);
        this.refresh_context(all);
        timer.finish("context", Some(body.procedure), body.blocks.len());
        this
    }
}

impl<'a> Resolver<'a> {
    /// Borrow the prepared facts without exposing their ownership handle.
    pub(super) fn core(&self) -> &Core<'a> {
        &self.core
    }

    #[cfg(test)]
    pub(super) fn set_scalar_work_bound(&mut self, work: usize) {
        Rc::get_mut(&mut self.core)
            .unwrap()
            .scalar_requests
            .as_mut()
            .unwrap()
            .work_bound = work;
    }
}

impl<'a> std::ops::Deref for Resolver<'a> {
    type Target = Core<'a>;

    fn deref(&self) -> &Core<'a> {
        &self.core
    }
}

impl Core<'_> {
    /// Exact SSA alternatives captured at this pointer load's event.
    /// Initial, unsupported or possibly clobbered storage has no exact alternatives.
    pub(in crate::rules) fn captured_pointer_values(
        &self,
        value: jaic::ir::Val,
        budget: &mut usize,
    ) -> Option<impl Iterator<Item = jaic::ir::Val> + '_> {
        use super::storage_versions::Source;
        let sources = self.storage_versions.get(&value)?;
        let Some(remaining) = budget.checked_sub(sources.len()) else {
            *budget = 0;
            return None;
        };
        *budget = remaining;
        if sources.is_empty() || sources.iter().any(|s| !matches!(s, Source::Value(_))) {
            return None;
        }
        Some(sources.iter().filter_map(|s| match s {
            Source::Value(v) => Some(*v),
            _ => None,
        }))
    }

    /// A single complete stored SSA value; mixed or uncertain captures stay unknown.
    pub(in crate::rules) fn captured_pointer_value(
        &self,
        value: jaic::ir::Val,
    ) -> Option<jaic::ir::Val> {
        self.captured_pointer_values(value, &mut 1)?.next()
    }

    /// One typed constant captured at the actual read; unknown alternatives reject.
    /// For example, a later pointer overwrite cannot retain an earlier null fact.
    pub(in crate::rules) fn captured_constant(
        &self,
        value: jaic::ir::Val,
    ) -> Option<(jaic::types::TypeId, u64)> {
        let sources = self.storage_versions.get(&value)?;
        if sources.len() != 1 {
            return None;
        }
        match sources.iter().next()? {
            super::storage_versions::Source::Constant {
                ty,
                value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits),
            } => Some((*ty, *bits)),
            _ => None,
        }
    }
}


impl Prepared<'_> {
    /// The same complete capture query used by caller transfer and origin labels.
    pub(in crate::rules) fn captured_pointer_values(
        &self,
        value: jaic::ir::Val,
        budget: &mut usize,
    ) -> Option<impl Iterator<Item = jaic::ir::Val> + '_> {
        self.core.captured_pointer_values(value, budget)
    }
}

/// Native inventories reserve collection only; each selected transfer reserves its own work.
pub(super) struct NativeReads {
    pub frame_footprint: usize,
    pub factory_work: usize,
    pub reads: Vec<super::call_slots::ScalarRead>,
    pub collection_work: usize,
}

/// A bounded inventory only: ordinary pointer execution does not acquire scalar state.
fn native_reads(
    c: &Compiler,
    body: &LifetimeBody,
    places: &Places,
    core: &Core<'_>,
) -> Option<Box<NativeReads>> {
    let events = body
        .blocks
        .iter()
        .try_fold(0usize, |n, b| n.checked_add(b.events.len()))?;
    let edges = body
        .blocks
        .iter()
        .try_fold(0usize, |n, b| n.checked_add(b.successors.len()))?;
    if events > 256
        || events
            .checked_add(edges)?
            .checked_add(body.blocks.len())?
            .checked_add(body.read_only_regions.len())?
            > 256
    {
        return None;
    }
    collect_scalar_reads(c, body, places, core, &mut 8192)
}

/// Query-owned typed captures only; this grants no whole-native-Frame eligibility.
pub(super) fn requested_scalar_reads(
    r: &Resolver<'_>,
    remaining: &mut usize,
) -> Option<Box<NativeReads>> {
    collect_scalar_reads(r.compiler, r.body, &r.places, r.core(), remaining)
}

fn collect_scalar_reads(
    c: &Compiler,
    body: &LifetimeBody,
    places: &Places,
    core: &Core<'_>,
    remaining: &mut usize,
) -> Option<Box<NativeReads>> {
    let signature = c.pr.procs.get(body.procedure.0 as usize)?.sig.as_ref()?;
    if signature.params.len() > 8
        || signature.c_varargs
        || signature.params.iter().any(|p| {
            p.variadic
                || (!c.ty.types.is_pointer(p.ty)
                    && p.ty != jaic::types::TypeId::BOOL
                    && c.ty.types.int_info(p.ty).is_none())
        })
        || signature.returns.len() != 1
        || !c.ty.types.is_pointer(signature.returns[0])
        || c.ty.types.size_of(signature.returns[0]) != 8
        || body.blocks.is_empty()
        || body.blocks.len() > 32
        || body.address_casts.len() > 64
        || body.comparisons.len() > 128
    {
        return None;
    }
    let initial = *remaining;
    *remaining = remaining.checked_sub(signature.params.len() + signature.returns.len() + 16)?;
    let mut events = 0usize;
    let mut edges = 0usize;
    let mut results = 0usize;
    let mut reads = 0usize;
    let mut call = false;
    for b in &body.blocks {
        *remaining = remaining.checked_sub(
            b.events
                .len()
                .checked_add(b.successors.len())?
                .checked_add(1)?,
        )?;
        events = events.checked_add(b.events.len())?;
        edges = edges.checked_add(b.successors.len())?;
        if edges > 64 || b.successors.iter().any(|s| *s >= body.blocks.len()) {
            return None;
        }
        for e in &b.events {
            if let Event::Call {
                results: out, ..
            }
            | Event::IndirectCall {
                results: out, ..
            } = e
            {
                results = results.checked_add(out.len())?;
                if results > 64 {
                    return None;
                }
            }
            if let Event::Call {
                callee, ..
            } = e
            {
                call |= c
                    .pr
                    .procs
                    .get(callee.0 as usize)
                    .and_then(|p| p.sig.as_ref())
                    .is_some_and(|s| s.returns.len() == 1 && c.ty.types.is_pointer(s.returns[0]));
            }
            if let Event::Load {
                ty, ..
            } = e
                && (*ty == jaic::types::TypeId::BOOL || c.ty.types.int_info(*ty).is_some())
            {
                reads += 1;
                if reads > 64 {
                    return None;
                }
            }
            if jaic::interrupt::should_stop() {
                return None;
            }
        }
    }
    if !call {
        return None;
    }
    let frame_footprint = events
        .checked_add(edges)?
        .checked_add(body.blocks.len())?
        .checked_add(body.read_only_regions.len())?;
    let factory_work = signature.params.len() + signature.returns.len() + 16;
    let collection_work = events
        .checked_mul(3)?
        .checked_add(edges + body.blocks.len() + results * 2)?
        .checked_add(reads.checked_mul(core.exposed.len().checked_add(16)?)?)?;
    if collection_work > 8192 {
        return None;
    }
    let prepaid = initial - *remaining;
    let required = collection_work.checked_add(factory_work)?;
    *remaining = remaining.checked_sub(required.checked_sub(prepaid)?)?;
    let mut out = Vec::with_capacity(reads);
    for (block, b) in body.blocks.iter().enumerate() {
        for (event, e) in b.events.iter().enumerate() {
            let Event::Load {
                place,
                value,
                ty,
                ..
            } = e
            else {
                continue;
            };
            if *ty != jaic::types::TypeId::BOOL && c.ty.types.int_info(*ty).is_none() {
                continue;
            }
            if !(1..=8).contains(&c.ty.types.size_of(*ty)) {
                return None;
            }
            let slot = places.existing(*place, *ty)?;
            if !places.paths[slot].1.is_empty() || !core.private_slot(c, body, places, slot) {
                continue;
            }
            out.push(super::call_slots::ScalarRead {
                site: crate::rules::lifetime_movement::facts::Site {
                    block,
                    event,
                },
                value: *value,
                ty: *ty,
                slot,
            });
        }
    }
    (!jaic::interrupt::should_stop()).then(|| {
        Box::new(NativeReads {
            frame_footprint,
            factory_work,
            reads: out,
            collection_work,
        })
    })
}

impl Resolver<'_> {
    /// Reserve the completed construction receipt once at the caller's proof entry.
    /// Nested queries use this view, without re-paying a retained-body scan.
    pub(in crate::rules) fn capture_index(
        &self,
        remaining: &mut usize,
    ) -> Option<super::captured_values::QueryIndex> {
        self.core
            .capture_index
            .get_or_init(|| super::captured_values::IndexResult::build(self))
            .reserve(remaining)
    }
}

impl<'a> Core<'a> {
    pub(super) fn checked_body(&self) -> CheckedBody<'a> {
        let owner = self.storage_versions.owner();
        debug_assert!(
            std::ptr::eq(self.compiler, owner.compiler()) && std::ptr::eq(self.body, owner.body())
        );
        owner
    }
}
impl Resolver<'_> {
    /// Identity authenticates the immutable analysis owner, never a runtime object.
    /// Do not initialize another index as a side effect of this check.
    pub(in crate::rules) fn owns_capture_index(
        &self,
        index: &super::captured_values::QueryIndex,
    ) -> bool {
        self.core
            .capture_index
            .get()
            .is_some_and(|result| result.owns(index))
    }
}

/// Admit one actual selector cell; the pointed Context gains no ownership.
fn context_slot(c: &Compiler, body: &LifetimeBody, places: &mut Places) -> Option<Val> {
    use crate::rules::lifetime_facts::StackRoots;
    if body.context_input.is_some() != c.proc(body.procedure).sig.as_ref()?.has_context {
        return None;
    }
    let mut selected = None;
    let mut remaining = 8192usize;
    for event in body.blocks.iter().flat_map(|block| &block.events) {
        remaining = remaining.checked_sub(1)?;
        if jaic::interrupt::should_stop() {
            return None;
        }
        let (Event::ContextRead {
            place,
            ty,
            ..
        }
        | Event::ContextSet {
            place,
            ty,
            ..
        }) = event
        else {
            continue;
        };
        if *ty != jaic::types::TypeId::VOID_PTR || selected.is_some_and(|old| old != *place) {
            return None;
        }
        selected = Some(*place);
    }
    let selected = selected?;
    let metadata = body
        .stack_roots
        .len()
        .checked_add(body.slot_aliases.len())?;
    remaining.checked_sub(metadata)?;
    let roots = StackRoots::capture(body, remaining)?;
    let root = roots.canonical(selected);
    if roots.extent(root).flatten()? < c.ty.types.size_of(jaic::types::TypeId::VOID_PTR) {
        return None;
    }
    // Ordinary roots are materialized later by the definition inventory; create
    // this one only after its original allocation has been authenticated.
    let slot = places.place(selected, jaic::types::TypeId::VOID_PTR);
    (places.paths[slot].1.is_empty() && !jaic::interrupt::should_stop()).then_some(selected)
}
