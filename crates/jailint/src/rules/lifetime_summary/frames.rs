//! Closed checked helper footprints. Missing frames never imply no effects.
use super::physical_slots::Slot;
use super::{
    Compiler, Effect, Event, HashMap, HashSet, Kind, ProcId, Resolver, Span, Summary, TypeId,
    TypeKind, Val,
};
use crate::rules::lifetime_facts::BodyIdentity;

mod raw;
pub(super) mod read_only;

const MAX_EVENTS: usize = 128;
const MAX_FOOTPRINTS: usize = 32;
const MAX_WORK: usize = 8192;

/// Current body identity or an independently checked declaration constraint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Requirement {
    Body(BodyIdentity),
    Constraint(ProcId),
}
impl Requirement {
    pub(super) fn procedure(&self) -> ProcId {
        match self {
            Self::Body(body) => body.procedure(),
            Self::Constraint(id) => *id,
        }
    }
    pub(super) fn current(&self, c: &Compiler) -> bool {
        match self {
            Self::Body(body) => body.current(c),
            Self::Constraint(id) => c.pr.procs.get(id.0 as usize).is_some(),
        }
    }
}

/// Complete MAY effects, expressed as bounded typed slots or conditional raw
/// accesses. Only the typed variant has a guaranteed full-write subset. Neither
/// identity validation nor an empty typed list discharges access or guard obligations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Frame {
    owner: Requirement,
    dependencies: Vec<Requirement>,
    footprint: Footprint,
    pub condition: Option<read_only::Guard>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Typed {
    pub reads: Vec<Slot>,
    pub writes: Vec<Slot>,
    pub must: Vec<Slot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Footprint {
    Typed(Typed),
    ConditionalRaw(raw::Conditional),
    /// Discharged only by the owning guarded native-return witness.
    NativeReturn,
}

impl Frame {
    /// Bounded typed consumers cannot silently ignore raw access obligations.
    pub fn typed(&self) -> Option<&Typed> {
        match &self.footprint {
            Footprint::Typed(typed) => Some(typed),
            Footprint::ConditionalRaw(_) | Footprint::NativeReturn => None,
        }
    }

    /// Raw access and pointer-publication obligations remain explicit to callers.
    pub(super) fn raw(&self) -> Option<&raw::Conditional> {
        match &self.footprint {
            Footprint::ConditionalRaw(raw) => Some(raw),
            Footprint::Typed(_) | Footprint::NativeReturn => None,
        }
    }

    pub(super) fn requirements(&self) -> impl Iterator<Item = &Requirement> {
        std::iter::once(&self.owner).chain(&self.dependencies)
    }

    pub(super) fn requirement_count(&self) -> usize {
        self.dependencies.len() + 1
    }

    /// Owned buffers beyond the inline Frame; shared syntax is excluded.
    pub(super) fn owned_buffer_bytes(&self) -> usize {
        self.dependencies.capacity() * std::mem::size_of::<Requirement>()
            + match &self.footprint {
                Footprint::Typed(t) => {
                    (t.reads.capacity() + t.writes.capacity() + t.must.capacity())
                        * std::mem::size_of::<Slot>()
                }
                Footprint::ConditionalRaw(_) => {
                    self.raw().map_or(0, raw::Conditional::payload_bytes)
                }
                Footprint::NativeReturn => 0,
            }
    }

    /// Current summaries must still carry no independent declaration effects.
    pub(super) fn compatible(&self, all: &HashMap<ProcId, Summary>) -> bool {
        self.requirements()
            .all(|i| all.get(&i.procedure()).is_some_and(|s| s.trusted.is_none()))
    }

    pub fn current(&self, c: &Compiler) -> bool {
        self.requirements().all(|identity| identity.current(c))
    }
}

/// Whole-byte record writes need no recursive field or pointer-leaf inventory.
pub(super) fn ordinary_record(c: &Compiler, ty: TypeId) -> bool {
    c.ty.types
        .as_struct(ty)
        .is_some_and(|id| !c.ty.types.struct_info(id).is_union && c.ty.types.size_of(ty) > 0)
}

/// Select dependencies of typed descriptor-to-element borrow candidates. This
/// follows the existing checked call graph, independently of inferred effects.
pub(super) fn demand(c: &Compiler, graph: &HashMap<ProcId, Vec<ProcId>>) -> HashSet<ProcId> {
    let mut pending = Vec::new();
    for (&id, edges) in graph {
        if jaic::interrupt::should_stop() {
            return HashSet::default();
        }
        let Some(sig) = c.pr.procs[id.0 as usize].sig.as_ref() else {
            continue;
        };
        if sig.params.len() > 32 || sig.returns.len() > 8 {
            continue;
        }
        if sig.params.iter().any(|p| {
            c.ty.types.pointee(p.ty).is_some_and(|ty| {
                matches!(c.ty.types.kind(ty), TypeKind::Array {kind:super::ArrayKind::Resizable,elem}
                    if sig.returns.iter().any(|r| c.ty.types.pointee(*r)==Some(*elem)))
            })
        }) { pending.extend(edges.iter().copied()); }
    }
    let mut needed = HashSet::default();
    while let Some(id) = pending.pop() {
        if jaic::interrupt::should_stop() {
            return HashSet::default();
        }
        if needed.insert(id)
            && let Some(edges) = graph.get(&id)
        {
            pending.extend(edges.iter().copied());
        }
    }
    needed
}

/// One observer of the existing summary event traversal. Failure is absorbing;
/// no partially checked footprint is published as a complete frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Purpose {
    ClosedWrite,
    EntryReadOnly,
    ConditionalRaw,
    BitReturn,
    NativeReturn,
}

#[derive(Clone)]
pub(super) struct Builder {
    purpose: Purpose,
    owner: Option<BodyIdentity>,
    dependencies: Vec<Requirement>,
    reads: Vec<Slot>,
    writes: Vec<Slot>,
    effects: Vec<Vec<(Effect, Vec<Span>)>>,
    typed_reads: HashSet<Val>,
    work: usize,
    events: usize,
    visits: usize,
    composed: usize,
    raw: raw::Conditional,
    validated_outputs: Vec<Val>,
    capture_index: Option<super::captured_values::QueryIndex>,
}

impl Builder {
    pub fn new(r: &Resolver<'_>, requested: bool, recursive: bool) -> Self {
        let raw = r.compiler.pr.procs[r.body.procedure.0 as usize]
            .sig
            .as_ref()
            .is_some_and(|s| {
                requested
                    && !recursive
                    && s.params.len() <= 32
                    && s.returns.is_empty()
                    && !s.c_varargs
                    && s.params.iter().any(|p| {
                        r.compiler
                            .ty
                            .types
                            .pointee(p.ty)
                            .is_some_and(|t| t == TypeId::VOID)
                    })
            });
        Self::checked(
            r,
            requested,
            recursive,
            if raw {
                Purpose::ConditionalRaw
            } else {
                Purpose::ClosedWrite
            },
            0,
        )
    }

    fn read_only(r: &Resolver<'_>, prior_work: usize) -> Self {
        Self::checked(r, true, false, Purpose::EntryReadOnly, prior_work)
    }

    /// Carry the spent budget and its already-charged view as one continuation.
    fn read_only_after(r: &Resolver<'_>, prior: &Self) -> Option<Self> {
        if prior
            .capture_index
            .as_ref()
            .is_some_and(|index| !r.owns_capture_index(index))
        {
            return None;
        }
        let mut out = Self::read_only(r, prior.work);
        if !out.active() {
            return None;
        }
        out.capture_index.clone_from(&prior.capture_index);
        Some(out)
    }

    /// Only a completed bit graph may authorize these exact pointer return values.
    pub(super) fn bit_return(
        r: &Resolver<'_>,
        plan: &super::address_arithmetic::bit_returns::PendingBits,
    ) -> Self {
        let mut out = Self::checked(r, true, false, Purpose::BitReturn, plan.observer_work());
        if !r.owns_capture_index(plan.observer_index()) {
            out.owner = None;
            return out;
        }
        out.capture_index = Some(plan.observer_index().clone());
        let outputs = plan.observer_outputs();
        if !out.active() {
            return out;
        }
        if out.charge(outputs.len()).is_none() {
            out.owner = None;
        } else {
            out.validated_outputs.extend_from_slice(outputs);
        }
        out
    }

    /// Native calls and prefix writes remain explicit obligations in the owner.
    /// The result cannot be consumed as an ordinary bounded typed frame.
    pub(super) fn native_return(
        r: &Resolver<'_>,
        start: &super::native_returns::ObserverStart<'_>,
    ) -> Self {
        let mut out = Self::checked(r, false, false, Purpose::NativeReturn, start.spent());
        if !r.owns_capture_index(start.index()) {
            return out;
        }
        out.capture_index = Some(start.index().clone());
        // Prepared's native inventory preflights this same immutably borrowed
        // body's signature and complete retained footprint before it is frozen.
        let Some(inventory) = r.core().native_reads.as_ref() else {
            return out;
        };
        if inventory.frame_footprint > 256 || out.charge(inventory.factory_work).is_none() {
            return out;
        }
        out.events = inventory.frame_footprint;
        out.owner = BodyIdentity::capture(r.compiler, r.body);
        out
    }

    /// Fork only a freshly checked native observer; immutable body validation
    /// is shared as a receipt, while each path owns all its mutable state.
    pub(super) fn native_path(&mut self, prior_work: usize, output: Val) -> Option<Self> {
        if self.purpose != Purpose::NativeReturn || self.visits != 0 || !self.active() {
            return None;
        }
        if !self.resume_work(prior_work) {
            return None;
        }
        self.charge(self.typed_reads.len() + 2)?;
        let mut out = self.clone();
        out.validated_outputs.push(output);
        Some(out)
    }

    /// Continue one shared native witness budget, never reset a spent budget.
    pub(super) fn resume_work(&mut self, work: usize) -> bool {
        if work < self.work || work > MAX_WORK {
            self.owner = None;
            return false;
        }
        self.work = work;
        true
    }

    /// The native witness retains and discharges this exact call's other effects.
    pub(super) fn represented_call(&mut self, identities: &[Requirement]) -> Option<()> {
        if self.purpose != Purpose::NativeReturn || !self.active() {
            return None;
        }
        self.charge(1)?;
        self.visits += 1;
        for identity in identities {
            self.charge(self.dependencies.len() + 1)?;
            if !self.dependencies.contains(identity) {
                if self.dependencies.len() == MAX_FOOTPRINTS {
                    self.owner = None;
                    return None;
                }
                self.dependencies.push(identity.clone());
            }
        }
        Some(())
    }

    pub(super) fn spent(&self) -> usize {
        self.work
    }

    fn checked(
        r: &Resolver<'_>,
        requested: bool,
        recursive: bool,
        purpose: Purpose,
        prior_work: usize,
    ) -> Self {
        let out = Self {
            purpose,
            owner: None,
            dependencies: Vec::new(),
            reads: Vec::new(),
            writes: Vec::new(),
            effects: Vec::new(),
            typed_reads: HashSet::default(),
            work: prior_work,
            events: 0,
            visits: 0,
            composed: 0,
            raw: raw::Conditional::default(),
            validated_outputs: Vec::new(),
            capture_index: None,
        };
        if prior_work >= MAX_WORK
            || !requested
            || recursive
            || r.compiler.pr.procs[r.body.procedure.0 as usize]
                .sig
                .as_ref()
                .is_none_or(|s| {
                    (if matches!(purpose, Purpose::BitReturn | Purpose::NativeReturn) {
                        s.returns.len() != 1
                            || !r.compiler.ty.types.is_pointer(s.returns[0])
                            || r.compiler.ty.types.size_of(s.returns[0]) != 8
                    } else {
                        !s.returns.is_empty()
                            && !(purpose == Purpose::ClosedWrite
                                && s.returns.len() == 1
                                && s.params.is_empty()
                                && !s.has_context
                                && r.compiler.ty.types.int_info(s.returns[0]).is_some())
                    }) || s.c_varargs
                        || s.params.len() > 32
                        || s.params.iter().any(|p| {
                            p.variadic
                                || !r.compiler.ty.types.pointee(p.ty).map_or_else(
                                    || Self::scalar(r, p.ty),
                                    |ty| match purpose {
                                        Purpose::ClosedWrite => {
                                            Self::scalar(r, ty) || ordinary_record(r.compiler, ty)
                                        }
                                        Purpose::ConditionalRaw
                                        | Purpose::BitReturn
                                        | Purpose::NativeReturn => true,
                                        Purpose::EntryReadOnly => {
                                            r.compiler.ty.types.size_of(ty) > 0
                                        }
                                    },
                                )
                        })
                })
        {
            return out;
        }
        let event_limit = if purpose == Purpose::NativeReturn {
            256
        } else {
            MAX_EVENTS
        };
        if r.body.blocks.is_empty() || r.body.blocks.len() > event_limit {
            return out;
        }
        if purpose == Purpose::ConditionalRaw
            && (r.body.blocks.len() != 1
                || !r.body.blocks[0].successors.is_empty()
                || !r.body.blocks[0].normal_return)
        {
            return out;
        }
        let mut events = 0usize;
        let mut edges = 0usize;
        for b in &r.body.blocks {
            events = events.saturating_add(b.events.len());
            edges = edges.saturating_add(b.successors.len());
            if events > event_limit || edges > event_limit {
                return out;
            }
        }
        let Some(footprint) =
            (events + r.body.blocks.len() + edges).checked_add(r.body.read_only_regions.len())
        else {
            return out;
        };
        if footprint > event_limit
            || prior_work
                .checked_add(footprint * 3)
                .is_none_or(|n| n > MAX_WORK)
        {
            return out;
        }
        let owner = BodyIdentity::capture(r.compiler, r.body);
        let typed_reads = if owner.is_some() {
            r.body
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .filter_map(|e| match e {
                    Event::Indirect {
                        place, ..
                    } => Some(*place),
                    Event::FieldProjection {
                        result, ..
                    } if purpose == Purpose::EntryReadOnly => Some(*result),
                    _ => None,
                })
                .collect()
        } else {
            HashSet::default()
        };
        Self {
            purpose,
            owner,
            dependencies: Vec::new(),
            reads: Vec::new(),
            writes: Vec::new(),
            effects: if purpose == Purpose::ClosedWrite {
                vec![Vec::new(); r.body.blocks.len()]
            } else {
                Vec::new()
            },
            typed_reads,
            work: prior_work + footprint * 3,
            events: footprint,
            visits: 0,
            composed: 0,
            raw: raw::Conditional::default(),
            validated_outputs: Vec::new(),
            capture_index: None,
        }
    }

    pub fn active(&self) -> bool {
        self.owner.is_some()
    }

    fn charge(&mut self, amount: usize) -> Option<()> {
        self.work = self.work.checked_add(amount)?;
        (self.work <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
    }

    fn scalar(r: &Resolver<'_>, ty: TypeId) -> bool {
        !r.compiler.ty.types.is_pointer(ty)
            && r.compiler.ty.types.as_struct(ty).is_none()
            && !matches!(
                r.compiler.ty.types.kind(ty),
                TypeKind::Array { .. } | TypeKind::String
            )
            && (1..=8).contains(&r.compiler.ty.types.size_of(ty))
    }

    /// Reserve index construction once for this existing bounded frame proof.
    fn indexed(&mut self, r: &Resolver<'_>) -> Option<super::captured_values::QueryIndex> {
        if self.capture_index.is_none() {
            let mut remaining = MAX_WORK.checked_sub(self.work)?;
            let result = r.capture_index(&mut remaining);
            self.work = MAX_WORK - remaining;
            self.capture_index = Some(result?);
        }
        self.capture_index.clone()
    }

    fn captured_address(
        &mut self,
        r: &Resolver<'_>,
        value: Val,
    ) -> Option<super::address_objects::Object> {
        let index = self.indexed(r)?;
        let mut remaining = MAX_WORK.checked_sub(self.work)?;
        let result = index.address(r, value, &mut remaining);
        self.work = MAX_WORK - remaining;
        result
    }

    fn slot(
        &mut self,
        r: &Resolver<'_>,
        place: Val,
        ty: TypeId,
        width: u64,
    ) -> Option<Option<Slot>> {
        self.charge(12 + r.exposed.len())?;
        let p = r.places.existing(place, ty)?;
        if r.private_root(p, width) {
            return Some(None);
        }
        // The native owner explicitly represents its selected external calls;
        // every other public access remains unsupported in this first slice.
        if self.purpose == Purpose::NativeReturn {
            return None;
        }
        let object = self.captured_address(r, place)?;
        let super::address_objects::Base::Input(formal) = object.base else {
            return None;
        };
        Some(Some(r.valid_slot(Slot {
            formal,
            offset: object.offset,
            width,
        })?))
    }

    /// Aggregate copies read their source bytes, unlike a scalar SSA store.
    fn aggregate_source(&mut self, r: &Resolver<'_>, value: Val, ty: TypeId) -> Option<()> {
        self.charge(r.body.read_only_regions.len() + r.places.paths.len() + 1)?;
        let width = r.compiler.ty.types.size_of(ty);
        let mut immutable = false;
        for region in &r.body.read_only_regions {
            if region.value == value {
                if region.ty != ty || region.bytes != width {
                    return None;
                }
                immutable = true;
            }
        }
        if immutable {
            return Some(());
        }
        let source = r.places.existing(value, ty)?;
        r.private_root(source, width).then_some(())
    }

    /// Only a direct unique typed immutable read may supply this scalar return.
    fn readonly_return(&mut self, r: &Resolver<'_>, value: Val, ty: TypeId) -> Option<()> {
        let index = self.indexed(r)?;
        self.charge(1)?;
        let super::captured_values::Producer::Load(selected, site) = index.get(value)? else {
            return None;
        };
        if selected != ty {
            return None;
        }
        let Event::Load {
            place,
            value: actual,
            ty: loaded,
            ..
        } = r.body.blocks.get(site.block)?.events.get(site.event)?
        else {
            return None;
        };
        if *actual != value || *loaded != ty {
            return None;
        }
        self.charge(r.body.read_only_regions.len())?;
        if !r
            .body
            .read_only_regions
            .iter()
            .any(|region| region.value == *place)
        {
            return None;
        }
        self.aggregate_source(r, *place, ty)
    }

    fn full_formal(r: &Resolver<'_>, slot: Slot) -> Option<super::Path> {
        let sig = r.compiler.pr.procs[r.body.procedure.0 as usize]
            .sig
            .as_ref()?;
        let ty = r
            .compiler
            .ty
            .types
            .pointee(sig.params.get(slot.formal)?.ty)?;
        (slot.offset == 0
            && (Self::scalar(r, ty) || ordinary_record(r.compiler, ty))
            && slot.width == r.compiler.ty.types.size_of(ty))
        .then_some(super::Path {
            root: super::Root::Parameter(slot.formal),
            steps: Vec::new(),
        })
    }

    fn add(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        slot: Slot,
        write: bool,
        must: bool,
    ) -> Option<()> {
        self.charge(self.reads.len() + self.writes.len() + 1)?;
        let set = if write {
            &mut self.writes
        } else {
            &mut self.reads
        };
        if !set.contains(&slot) {
            set.push(slot);
        }
        if self.reads.len() + self.writes.len() > MAX_FOOTPRINTS {
            return None;
        }
        if must && let Some(target) = Self::full_formal(r, slot) {
            self.effects[block].push((
                Effect {
                    target,
                    kind: Kind::Write,
                    guard: None,
                },
                Vec::new(),
            ));
        }
        Some(())
    }

    fn access(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        place: Val,
        ty: TypeId,
        width: u64,
        write: bool,
    ) -> Option<()> {
        if width == 0 {
            return Some(());
        }
        if !write {
            self.charge(r.body.read_only_regions.len())?;
            if r.body
                .read_only_regions
                .iter()
                .any(|region| region.value == place)
            {
                return self.aggregate_source(r, place, ty);
            }
        }
        if let Some(slot) = self.slot(r, place, ty, width)? {
            if self.purpose == Purpose::BitReturn {
                return None;
            }
            if write && self.purpose == Purpose::EntryReadOnly {
                return None;
            }
            self.add(r, block, slot, write, write)?;
        }
        Some(())
    }

    fn call(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        callee: ProcId,
        args: &[super::LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Option<()> {
        let frame = all.get(&callee)?.frame.as_ref()?;
        let typed = frame.typed()?;
        if !frame.current(r.compiler)
            || !frame.compatible(all)
            || frame.condition.is_some()
            || (self.purpose == Purpose::EntryReadOnly && !typed.writes.is_empty())
        {
            return None;
        }
        self.charge(
            (frame.dependencies.len() + 1)
                .checked_mul(self.dependencies.len() + MAX_FOOTPRINTS + 1)?,
        )?;
        for identity in frame.requirements() {
            if !self.dependencies.contains(identity) {
                if self.dependencies.len() == MAX_FOOTPRINTS {
                    return None;
                }
                self.dependencies.push(identity.clone());
            }
        }
        let sig = r.compiler.pr.procs.get(callee.0 as usize)?.sig.as_ref()?;
        self.charge(args.len().checked_mul(args.len() + 1)?)?;
        if args.len() != sig.params.len()
            || args.iter().any(|a| {
                a.spread
                    || sig
                        .params
                        .get(a.formal)
                        .is_none_or(|p| p.ty != a.ty || p.variadic)
                    || args.iter().filter(|b| b.formal == a.formal).count() != 1
            })
        {
            return None;
        }
        for (set, write) in [(&typed.reads, false), (&typed.writes, true)] {
            for &slot in set {
                self.charge(typed.must.len() + sig.params.len() + self.events + 1)?;
                let argument = args.iter().find(|a| a.formal == slot.formal)?;
                let object = self.captured_address(r, super::value(argument)?)?;
                let super::address_objects::Base::Input(formal) = object.base else {
                    return None;
                };
                let actual = r.valid_slot(Slot {
                    formal,
                    offset: object.offset.checked_add(slot.offset)?,
                    width: slot.width,
                })?;
                self.composed += 1;
                self.add(r, block, actual, write, write && typed.must.contains(&slot))?;
            }
        }
        Some(())
    }

    pub fn observe(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        ordinal: usize,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) {
        if !self.active() {
            return;
        }
        self.visits += 1;
        if self.event(r, block, ordinal, event, all).is_none() {
            self.owner = None;
        }
    }

    fn event(
        &mut self,
        r: &Resolver<'_>,
        block: usize,
        ordinal: usize,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<()> {
        self.charge(1)?;
        // A checked Type.enum_member lookup yields an immediate constant, not
        // an address or a runtime read. Runtime member paths remain below.
        if matches!(event,Event::Field{base:super::LifetimeOperand::Other,result:super::LifetimeOperand::Other,base_ty:TypeId::TYPE,result_ty,indirect:false,descriptor:false,..}
            if matches!(r.compiler.ty.types.kind(*result_ty),TypeKind::Enum(_)))
        {
            return Some(());
        }
        if self.purpose == Purpose::ConditionalRaw {
            return self.raw_event(r, block, ordinal, event);
        }
        match event {
            Event::Store {place,ty,..} if self.purpose==Purpose::NativeReturn
                && r.compiler.ty.types.is_pointer(*ty) => {
                self.charge(r.exposed.len()+8)?;
                let p=r.places.existing(*place,*ty)?;
                r.private_root(p,r.compiler.ty.types.size_of(*ty)).then_some(())
            }
            Event::Return {index:0,value,..} if matches!(self.purpose,Purpose::BitReturn|Purpose::NativeReturn) => {
                self.charge(self.validated_outputs.len())?;
                self.validated_outputs.contains(value).then_some(())
            }
            // Returning an actual integer constant publishes no captured pointer.
            // Every preceding event still passes the ordinary effect observer.
            Event::Return {index:0,ty,constant,value,..}
                if self.purpose==Purpose::ClosedWrite && r.compiler.ty.types.int_info(*ty).is_some()=> {
                let sig = r.compiler.proc(r.body.procedure).sig.as_ref()?;
                if !sig.params.is_empty() || sig.has_context || sig.returns.as_slice()!=[*ty] {return None;}
                if constant.is_some() {Some(())} else {self.readonly_return(r,*value,*ty)}
            },
            Event::FieldProjection {..}|Event::Field {..} if self.purpose==Purpose::EntryReadOnly=>Some(()),
            Event::Load {place,ty,..}|Event::Indirect {place,ty} if self.purpose==Purpose::EntryReadOnly =>
                self.access(r,block,*place,*ty,r.compiler.ty.types.size_of(*ty),false),
            Event::Store {place,value,ty,..} => {
                if !Self::scalar(r,*ty) {
                    // The compiler's actual entry spill copies the incoming value.
                    if r.body.blocks[block].events.iter().any(|e|matches!(e,Event::Parameter {place:p,incoming,ty:t,..} if p==place && incoming==value && t==ty)) {
                        let p=r.places.existing(*place,*ty)?;
                        return r.private_root(p,r.compiler.ty.types.size_of(*ty)).then_some(());
                    }
                    if !ordinary_record(r.compiler,*ty) {return None}
                    self.aggregate_source(r,*value,*ty)?;
                    return self.access(r,block,*place,*ty,r.compiler.ty.types.size_of(*ty),true);
                }
                self.access(r,block,*place,*ty,r.compiler.ty.types.size_of(*ty),true)
            }
            Event::Load {place,ty,..} | Event::Indirect {place,ty} => {
                if !Self::scalar(r,*ty) && !ordinary_record(r.compiler,*ty) {
                    let p=r.places.existing(*place,*ty)?;
                    return r.private_root(p,r.compiler.ty.types.size_of(*ty)).then_some(());
                }
                self.access(r,block,*place,*ty,r.compiler.ty.types.size_of(*ty),false)
            }
            Event::Initialize {place,ty,..} => {
                matches!(r.body.blocks[block].events.get(ordinal+1),Some(Event::DefaultImage {place:p,ty:t,..}) if p==place && t==ty).then_some(())
            }
            Event::DefaultImage {place,ty,bytes,zeroed,complete,..} => {
                if !zeroed && !complete {return None}
                if !matches!(ordinal.checked_sub(1).and_then(|i|r.body.blocks[block].events.get(i)),Some(Event::Initialize {place:p,ty:t,..}) if p==place && t==ty) {return None}
                self.access(r,block,*place,*ty,*bytes,true)
            }
            Event::ReadMemory {value,..} => self.typed_reads.contains(value).then_some(()),
            Event::Call {callee,expands:true,..} => {
                if all.get(callee).is_none_or(|s|s.trusted.is_some()) {return None;}
                let dependency = Requirement::Constraint(*callee);
                self.charge(self.dependencies.len()+1)?;
                if !self.dependencies.contains(&dependency) {
                    if self.dependencies.len()==MAX_FOOTPRINTS {return None;}
                    self.dependencies.push(dependency);
                }
                Some(())
            },
            Event::Call {callee,args,..} => self.call(r,block,*callee,args,all),
            Event::ContextSet {..}|Event::ContextRead {..}|Event::ContextAlias {..}
            |Event::Expression {..}|Event::Parameter {..}|Event::Use {..}|Event::Address {..}
            |Event::Copy {..}|Event::IntegerBinary {..}|Event::Boolean {..}|Event::Branch {..}
            |Event::Local {..}|Event::ScopeEnd {..}|Event::StatementEnd {..}|Event::StaticChoice {..}
            |Event::Expansion {..}|Event::Cleanup {..} => Some(()),
            _=>None,
        }
    }

    pub fn finish(
        &mut self,
        r: &Resolver<'_>,
        live: &super::flow::Live,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<Frame> {
        self.charge(self.dependencies.len() + 1)?;
        let result = self.prove(r, live).filter(|f| f.compatible(all));
        if self.events > 0 && jacket_config::env::LIFETIME_STATS.is_set() {
            let payload = result
                .as_ref()
                .map_or(0, |f| std::mem::size_of::<Frame>() + f.owned_buffer_bytes());
            eprintln!(
                "lifetime-frame proc={} visited={} composed={} work={} complete={} payload_bytes={}",
                r.body.procedure.0,
                self.visits,
                self.composed,
                self.work,
                result.is_some(),
                payload
            );
        }
        result
    }

    fn prove(&mut self, r: &Resolver<'_>, live: &super::flow::Live) -> Option<Frame> {
        let owner = self.owner.take()?;
        if live.is_empty() {
            return None;
        }
        if self.purpose == Purpose::NativeReturn {
            if !self.reads.is_empty() || !self.writes.is_empty() || jaic::interrupt::should_stop() {
                return None;
            }
            return Some(Frame {
                owner: Requirement::Body(owner),
                dependencies: std::mem::take(&mut self.dependencies),
                footprint: Footprint::NativeReturn,
                condition: None,
            });
        }
        if self.purpose == Purpose::ConditionalRaw {
            if jaic::interrupt::should_stop() {
                return None;
            }
            return Some(Frame {
                owner: Requirement::Body(owner),
                dependencies: std::mem::take(&mut self.dependencies),
                footprint: Footprint::ConditionalRaw(std::mem::take(&mut self.raw)),
                condition: None,
            });
        }
        let mut must = Vec::new();
        for &slot in &self.writes {
            let Some(target) = Self::full_formal(r, slot) else {
                continue;
            };
            // Bound the existing witness-cut traversal per candidate.
            self.work = self
                .work
                .checked_add(self.events.checked_mul(self.events + 1)?)?;
            if self.work > MAX_WORK || jaic::interrupt::should_stop() {
                return None;
            }
            if super::flow::guaranteed(
                r.body,
                &self.effects,
                &Effect {
                    target,
                    kind: Kind::Write,
                    guard: None,
                },
            )
            .is_some()
            {
                must.push(slot);
            }
        }
        if !r.body.blocks.iter().any(|b| b.normal_return) || jaic::interrupt::should_stop() {
            return None;
        }
        Some(Frame {
            owner: Requirement::Body(owner),
            dependencies: std::mem::take(&mut self.dependencies),
            footprint: Footprint::Typed(Typed {
                reads: std::mem::take(&mut self.reads),
                writes: std::mem::take(&mut self.writes),
                must,
            }),
            condition: None,
        })
    }
}

#[cfg(test)]
mod ownership_tests {
    use super::*;

    #[test]
    fn frame_continuations_keep_one_reservation_and_monotone_work() {
        let c = super::super::tests::stdlib_allocations::checked_source(
            "first::(p:*s64){p.*=1;} second::(p:*s64){p.*=2;} main::(){}",
        );
        let all = super::super::Summaries::build(&c);
        let session = crate::rules::lifetime_facts::CheckedSession::new(&c);
        let key = |name: &str| {
            ProcId(
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap() as u32,
            )
        };
        let a = super::super::Prepared::for_body(session.body(key("first")).unwrap(), &all);
        let b = super::super::Prepared::for_body(session.body(key("second")).unwrap(), &all);
        let r = Resolver::for_prepared(&a, &all);
        let other = Resolver::for_prepared(&b, &all);
        let mut first = Builder::read_only(&r, 0);
        first.indexed(&r).unwrap();
        let reserved = first.work;
        let mut next = Builder::read_only_after(&r, &first).unwrap();
        let before = next.work;
        next.indexed(&r).unwrap();
        assert_eq!(
            next.work, before,
            "continuation must not reserve construction twice"
        );
        next.charge(7).unwrap();
        let last = Builder::read_only_after(&r, &next).unwrap();
        assert!(last.work >= next.work && last.work > reserved);
        let mut independent = Builder::read_only(&r, 0);
        assert!(independent.work < reserved);
        independent.indexed(&r).unwrap();
        assert_eq!(
            independent.work, reserved,
            "new proof reserves the receipt again"
        );
        assert!(Builder::read_only_after(&other, &first).is_none());
        next.owner = None;
        let failed_work = next.work;
        let fallback = Builder::read_only_after(&r, &next).unwrap();
        assert!(
            fallback.work >= failed_work,
            "failed polarity retains spent work"
        );
        next.work = MAX_WORK;
        assert!(Builder::read_only_after(&r, &next).is_none());
    }
}
