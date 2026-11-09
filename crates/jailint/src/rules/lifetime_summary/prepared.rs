//! The parts of a body's resolver that do not depend on the summaries solved so far.
//! They are built once per body and shared by every pass that needs a resolver for it.
use super::{
    Def, Event, HashMap, HashSet, LifetimeBody, LifetimeOperand, Places, ProcId, Resolver, Summary,
    allocator_context, allocators, interfaces, observable, returning_blocks,
};
use jaic::sema::Compiler;
use std::rc::Rc;

pub(in crate::rules) struct Core<'a> {
    pub(super) load_sources: HashMap<jaic::ir::Val, jaic::ir::Val>,
    /// Loads of a field outside this body's own locals that read back the value just stored.
    pub(super) stored_loads: HashMap<jaic::ir::Val, jaic::ir::Val>,
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
        compiler: &'a Compiler,
        body: &'a LifetimeBody,
        all: &HashMap<ProcId, Summary>,
        targets: &super::super::lifetime_targets::Universe,
    ) -> Self {
        let mut places = Places::new(compiler, body);
        let mut core = Core {
            load_sources: HashMap::default(),
            stored_loads: HashMap::default(),
            write_sources: HashMap::default(),
            nullable_sources: HashMap::default(),
            interfaces: interfaces::collect(body),
            address_casts: body.address_casts.iter().map(|c| (c.value, c)).collect(),
            values: HashMap::default(),
            writes: HashMap::default(),
            params: HashMap::default(),
            exposed: HashSet::default(),
            owned: HashSet::default(),
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
                        .push(Def::Parameter(*formal));
                }
                Event::Load {
                    place,
                    value,
                    ty,
                    ..
                } => {
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
                } => {
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

        core.finish_definition_index(body, &places);
        Self {
            core: Rc::new(core),
            places,
        }
    }
}

/// Prepared bodies shared by the passes of one summary build.
pub(super) type Cache<'a> = HashMap<ProcId, Prepared<'a>>;

impl<'a> Prepared<'a> {
    pub fn for_body(
        compiler: &'a Compiler,
        body: &'a LifetimeBody,
        summaries: &super::Summaries,
    ) -> Self {
        Self::build(compiler, body, &summaries.procs, &summaries.targets)
    }
}

impl<'a> Resolver<'a> {
    pub fn for_prepared(
        compiler: &'a Compiler,
        body: &'a LifetimeBody,
        prepared: &Prepared<'a>,
        summaries: &'a super::Summaries,
    ) -> Self {
        Self::from_prepared(
            compiler,
            body,
            prepared,
            &summaries.procs,
            &summaries.targets,
            &summaries.allocators,
        )
    }

    /// A resolver for `id`, preparing its body the first time any pass asks.
    pub fn cached<'c: 'a>(
        cache: &mut Cache<'c>,
        compiler: &'c Compiler,
        id: ProcId,
        body: &'c LifetimeBody,
        all: &HashMap<ProcId, Summary>,
        targets: &'a super::super::lifetime_targets::Universe,
        allocators: &'a allocators::Allocators,
    ) -> Self {
        let prepared = cache
            .entry(id)
            .or_insert_with(|| Prepared::build(compiler, body, all, targets));
        Self::from_prepared(compiler, body, prepared, all, targets, allocators)
    }

    /// A resolver over `prepared`, with the context that depends on `all` built fresh.
    pub fn from_prepared(
        compiler: &'a Compiler,
        body: &'a LifetimeBody,
        prepared: &Prepared<'a>,
        all: &HashMap<ProcId, Summary>,
        targets: &'a super::super::lifetime_targets::Universe,
        allocators: &'a allocators::Allocators,
    ) -> Self {
        let mut this = Self {
            places: prepared.places.clone(),
            allocators,
            context: allocator_context::Context::default(),
            allocation_mode: std::cell::Cell::new(false),
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
            compiler,
            body,
            targets,
        };
        this.rounded_address_base = this.rounded_address(all);
        this.refresh_context(all);
        this
    }
}

impl<'a> std::ops::Deref for Resolver<'a> {
    type Target = Core<'a>;

    fn deref(&self) -> &Core<'a> {
        &self.core
    }
}
