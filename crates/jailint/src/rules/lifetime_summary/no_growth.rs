//! Conditional append borrows assembled from checked calls and captured SSA.
//! No procedure name, annotation or allocator classification grants this witness.
use super::address_objects::Base;
use super::frames::read_only::Guard;
use super::physical_slots::Slot;
use super::storage_versions::Source;
use super::{
    Compiler, Event, HashMap, HashSet, LifetimeArgument, ProcId, Resolver, Summary, TypeId,
    TypeKind, Val,
};
use crate::rules::lifetime_facts::BodyIdentity;
use crate::rules::lifetime_movement::facts::Site;
use jaic::layout::resizable;

pub(in crate::rules) mod calls;

const MAX_EVENTS: usize = 128;
const MAX_WORK: usize = 8192;
const MAX_CALLS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Condition {
    guard: Guard,
    mapped: [Slot; 2],
}

/// An effect conditional on caller bounds, header/backing separation and the
/// recorded child guards. It is not an unconditional cleanup/return origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct Borrow {
    owner: BodyIdentity,
    dependencies: Vec<super::frames::Requirement>,
    conditions: Vec<Condition>,
    pub receiver: usize,
    pub header: TypeId,
    pub element: TypeId,
    pub result: usize,
    pub data_read: Site,
    pub count_read: Site,
    pub offset: Site,
    pub initialize: Site,
    pub count_write: Site,
}

impl Borrow {
    pub(super) fn compatibility_work(&self, all: &HashMap<ProcId, Summary>) -> usize {
        self.dependencies.len()
            + 1
            + all
                .get(&self.owner.procedure())
                .and_then(|s| s.trusted.as_ref())
                .map_or(0, |t| t.work())
    }

    pub(super) fn compatible(&self, all: &HashMap<ProcId, Summary>) -> bool {
        all.get(&self.owner.procedure()).is_some_and(|s| {
            s.trusted
                .as_ref()
                .is_none_or(|t| t.covered_by_append(self.receiver))
        }) && self
            .dependencies
            .iter()
            .all(|i| all.get(&i.procedure()).is_some_and(|s| s.trusted.is_none()))
    }

    pub fn current(&self, c: &Compiler) -> bool {
        self.owner.current(c) && self.dependencies.iter().all(|d| d.current(c))
    }

    pub fn guards(&self, count: i64, capacity: i64) -> bool {
        self.conditions.iter().all(|c| {
            c.guard.evaluate(|slot| {
                let mapped = if slot == c.guard.left.slot {
                    c.mapped[0]
                } else if slot == c.guard.right.slot {
                    c.mapped[1]
                } else {
                    return None;
                };
                if mapped.formal != self.receiver || mapped.width != 8 {
                    return None;
                }
                match mapped.offset {
                    resizable::COUNT => Some(count),
                    resizable::ALLOCATED => Some(capacity),
                    _ => None,
                }
            }) == Some(true)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Element {
    value: Val,
    offset: Site,
    data: Site,
    count: Site,
}

pub(super) struct Builder {
    valid: bool,
    capture_index: super::captured_values::QueryIndex,
    owner: BodyIdentity,
    header: TypeId,
    element: TypeId,
    pointer: TypeId,
    width: u64,
    work: usize,
    dependencies: Vec<super::frames::Requirement>,
    conditions: Vec<Condition>,
    initialized: Option<(Element, Site)>,
    count_write: Option<Site>,
    returned: Option<(Element, usize, Site)>,
}

impl Builder {
    pub fn new(r: &Resolver<'_>, recursive: bool) -> Option<Self> {
        if recursive
            || r.body.blocks.len() != 1
            || !r.body.blocks[0].normal_return
            || !r.body.blocks[0].successors.is_empty()
            || r.body.blocks[0].events.len() > MAX_EVENTS
        {
            return None;
        }
        let sig = r.compiler.pr.procs[r.body.procedure.0 as usize]
            .sig
            .as_ref()?;
        if sig.params.len() != 1
            || sig.params[0].variadic
            || sig.c_varargs
            || sig.returns.len() != 1
        {
            return None;
        }
        let header = r.compiler.ty.types.pointee(sig.params[0].ty)?;
        let TypeKind::Array {
            kind: super::ArrayKind::Resizable,
            elem,
        } = *r.compiler.ty.types.kind(header)
        else {
            return None;
        };
        if r.compiler.ty.types.pointee(sig.returns[0]) != Some(elem)
            || !(super::frames::ordinary_record(r.compiler, elem)
                || matches!(
                    r.compiler.ir_ty(elem),
                    Some(
                        jaic::ir::Ty::I8
                            | jaic::ir::Ty::I16
                            | jaic::ir::Ty::I32
                            | jaic::ir::Ty::I64
                            | jaic::ir::Ty::F32
                            | jaic::ir::Ty::F64
                    )
                ))
        {
            return None;
        }
        let mut remaining = MAX_WORK.checked_sub(r.body.blocks[0].events.len())?;
        let capture_index = r.capture_index(&mut remaining)?;
        Some(Self {
            valid: true,
            capture_index,
            owner: BodyIdentity::capture(r.compiler, r.body)?,
            header,
            element: elem,
            pointer: sig.returns[0],
            width: r.compiler.ty.types.size_of(elem),
            work: MAX_WORK - remaining,
            dependencies: Vec::new(),
            conditions: Vec::new(),
            initialized: None,
            count_write: None,
            returned: None,
        })
    }

    fn charge(&mut self, n: usize) -> Option<()> {
        self.work = self.work.checked_add(n)?;
        (self.work <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
    }

    fn address(&mut self, r: &Resolver<'_>, value: Val, width: u64) -> Option<Slot> {
        let mut left = MAX_WORK.checked_sub(self.work)?;
        let object = self.capture_index.address(r, value, &mut left);
        self.work = MAX_WORK - left;
        let object = object?;
        let Base::Input(0) = object.base else {
            return None;
        };
        let slot = r.valid_slot(Slot {
            formal: 0,
            offset: object.offset,
            width,
        })?;
        (slot.offset.checked_add(width)? <= r.compiler.ty.types.size_of(self.header))
            .then_some(slot)
    }

    fn private(&mut self, r: &Resolver<'_>, place: Val, ty: TypeId) -> Option<bool> {
        self.private_bytes(r, place, ty, r.compiler.ty.types.size_of(ty))
    }

    fn private_bytes(
        &mut self,
        r: &Resolver<'_>,
        place: Val,
        ty: TypeId,
        bytes: u64,
    ) -> Option<bool> {
        self.charge(r.places.paths.len() + r.body.blocks[0].events.len())?;
        let p = r.places.existing(place, ty)?;
        Some(r.private_root(p, bytes))
    }

    fn load(
        &mut self,
        r: &Resolver<'_>,
        value: Val,
        ty: TypeId,
        offset: u64,
        before: Site,
    ) -> Option<Site> {
        self.charge(r.body.blocks[0].events.len())?;
        let mut found = None;
        for (event, e) in r.body.blocks[0].events.iter().enumerate() {
            if let Event::Load {
                place,
                value: result,
                ty: selected,
                ..
            } = e
                && *result == value
            {
                if found.is_some() || *selected != ty || event >= before.event {
                    return None;
                }
                let slot = self.address(r, *place, r.compiler.ty.types.size_of(ty))?;
                if slot.offset != offset {
                    return None;
                }
                found = Some(Site {
                    block: 0,
                    event,
                });
            }
        }
        found
    }

    fn element(
        &mut self,
        r: &Resolver<'_>,
        value: Val,
        before: Site,
        seen: &mut HashSet<Val>,
        depth: usize,
    ) -> Option<Element> {
        self.charge(r.body.blocks[0].events.len() + 1)?;
        if depth > 24 || !seen.insert(value) || r.address_casts.contains_key(&value) {
            return None;
        }
        let mut result = None;
        for (event, e) in r.body.blocks[0].events.iter().enumerate() {
            let produced = match e {
                Event::Offset {
                    value: target,
                    source,
                    stride,
                    subscript: Some(index),
                    ..
                } if *target == value => {
                    if *stride != self.element || event >= before.event {
                        return None;
                    }
                    let site = Site {
                        block: 0,
                        event,
                    };
                    Some(Element {
                        value,
                        offset: site,
                        data: self.load(r, *source, self.pointer, resizable::DATA, site)?,
                        count: self.load(r, *index, TypeId::S64, resizable::COUNT, site)?,
                    })
                }
                Event::Copy {
                    value: target,
                    source,
                } if *target == value && *source != value => {
                    Some(self.element(r, *source, before, seen, depth + 1)?)
                }
                Event::Load {
                    value: target, ..
                } if *target == value => {
                    if event >= before.event {
                        return None;
                    }
                    let sources = r.storage_versions.get(&value)?;
                    self.charge(sources.len())?;
                    if sources.is_empty() {
                        return None;
                    }
                    let mut loaded = None;
                    for source in sources {
                        let Source::Value(source) = source else {
                            return None;
                        };
                        let current = self.element(
                            r,
                            *source,
                            Site {
                                block: 0,
                                event,
                            },
                            seen,
                            depth + 1,
                        )?;
                        if loaded.is_some_and(|old| old != current) {
                            return None;
                        }
                        loaded = Some(current);
                    }
                    Some(loaded?)
                }
                Event::View {
                    value: target,
                    source,
                    ..
                } if *target == value && *source != value => return None,
                Event::Call {
                    results, ..
                }
                | Event::IndirectCall {
                    results, ..
                } if results.iter().any(|(_, v, _)| *v == value) => return None,
                _ => None,
            };
            if let Some(current) = produced {
                if result.is_some_and(|old| old != current) {
                    return None;
                }
                result = Some(current);
            }
        }
        seen.remove(&value);
        result
    }

    fn map_slot(
        &mut self,
        r: &Resolver<'_>,
        slot: Slot,
        args: &[LifetimeArgument],
    ) -> Option<Slot> {
        self.charge(args.len() + 1)?;
        let arg = args.iter().find(|a| a.formal == slot.formal && !a.spread)?;
        let mut base = self.address(r, super::value(arg)?, 0)?;
        base.offset = base.offset.checked_add(slot.offset)?;
        base.width = slot.width;
        r.valid_slot(base)
    }

    fn call(
        &mut self,
        r: &Resolver<'_>,
        site: Site,
        callee: ProcId,
        args: &[LifetimeArgument],
        all: &HashMap<ProcId, Summary>,
    ) -> Option<()> {
        let frame = all.get(&callee)?.frame.as_ref()?;
        let typed = frame.typed()?;
        if !frame.current(r.compiler)
            || !frame.compatible(all)
            || self.dependencies.len() >= MAX_CALLS
        {
            return None;
        }
        let sig = r.compiler.pr.procs.get(callee.0 as usize)?.sig.as_ref()?;
        self.charge(
            args.len()
                .checked_mul(args.len() + 1)?
                .checked_add(typed.reads.len() + typed.writes.len() + typed.must.len())?,
        )?;
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
        self.charge(frame.requirements().count().checked_mul(MAX_CALLS + 1)?)?;
        for identity in frame.requirements() {
            if !self.dependencies.contains(identity) {
                if self.dependencies.len() == MAX_CALLS {
                    return None;
                }
                self.dependencies.push(identity.clone());
            }
        }
        if let Some(guard) = &frame.condition {
            if self.count_write.is_some() || !typed.writes.is_empty() {
                return None;
            }
            let mapped = [
                self.map_slot(r, guard.left.slot, args)?,
                self.map_slot(r, guard.right.slot, args)?,
            ];
            if mapped.iter().any(|s| {
                s.width != 8 || !matches!(s.offset, resizable::COUNT | resizable::ALLOCATED)
            }) {
                return None;
            }
            for &slot in &typed.reads {
                self.map_slot(r, slot, args)?;
            }
            self.charge(16)?;
            self.conditions.push(Condition {
                guard: guard.clone(),
                mapped,
            });
            return Some(());
        }
        if typed.writes.is_empty() {
            for &slot in &typed.reads {
                self.map_slot(r, slot, args)?;
            }
            return Some(());
        }
        if self.initialized.is_some() || self.count_write.is_some() {
            return None;
        }
        let mut node = None;
        let mut full = false;
        for (&slot, write) in typed
            .reads
            .iter()
            .map(|s| (s, false))
            .chain(typed.writes.iter().map(|s| (s, true)))
        {
            let arg = args.iter().find(|a| a.formal == slot.formal && !a.spread)?;
            let current = self.element(r, super::value(arg)?, site, &mut HashSet::default(), 0)?;
            if node.is_some_and(|old| old != current)
                || slot.offset.checked_add(slot.width)? > self.width
            {
                return None;
            }
            node = Some(current);
            full |=
                write && slot.offset == 0 && slot.width == self.width && typed.must.contains(&slot);
        }
        if !full {
            return None;
        }
        self.initialized = Some((node?, site));
        Some(())
    }

    pub fn observe(
        &mut self,
        r: &Resolver<'_>,
        site: Site,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) {
        if self.valid && self.event(r, site, event, all).is_none() {
            self.valid = false;
        }
    }

    fn event(
        &mut self,
        r: &Resolver<'_>,
        site: Site,
        event: &Event,
        all: &HashMap<ProcId, Summary>,
    ) -> Option<()> {
        self.charge(1)?;
        match event {
            Event::Load {
                place,
                ty,
                ..
            }
            | Event::Indirect {
                place,
                ty,
            } => {
                if self.private(r, *place, *ty)? {
                    return Some(());
                }
                r.compiler.ir_ty(*ty)?;
                self.address(r, *place, r.compiler.ty.types.size_of(*ty))
                    .map(|_| ())
            }
            Event::Store {
                place,
                value,
                ty,
                ..
            } => {
                r.compiler.ir_ty(*ty)?;
                if self.private(r, *place, *ty)? {
                    return Some(());
                }
                let slot = self.address(r, *place, r.compiler.ty.types.size_of(*ty))?;
                if *ty != TypeId::S64
                    || slot.offset != resizable::COUNT
                    || self.count_write.is_some()
                    || self.initialized.is_none()
                {
                    return None;
                }
                self.charge(r.body.blocks[0].events.len())?;
                let mut delta = None;
                for (ordinal, e) in r.body.blocks[0].events.iter().enumerate() {
                    if let Event::IntegerBinary {
                        value: target,
                        ty: TypeId::S64,
                        left,
                        right,
                        op: jaic::ir::BinOp::Add,
                        left_constant,
                        right_constant,
                    } = e
                        && target == value
                    {
                        if delta.is_some() || ordinal >= site.event {
                            return None;
                        }
                        let input = if *right_constant == Some(1) {
                            *left
                        } else if *left_constant == Some(1) {
                            *right
                        } else {
                            return None;
                        };
                        delta = Some(self.load(
                            r,
                            input,
                            TypeId::S64,
                            resizable::COUNT,
                            Site {
                                block: 0,
                                event: ordinal,
                            },
                        )?);
                    }
                }
                delta?;
                self.count_write = Some(site);
                Some(())
            }
            Event::Initialize {place, ty, ..} => {
                matches!(r.body.blocks[0].events.get(site.event+1),Some(Event::DefaultImage{place:p,ty:t,..}) if p==place && t==ty).then_some(())
            }
            Event::DefaultImage {place, ty, bytes, zeroed, complete, ..} => {
                if !zeroed && !complete {return None}
                if !matches!(site.event.checked_sub(1).and_then(|i|r.body.blocks[0].events.get(i)),Some(Event::Initialize{place:p,ty:t,..}) if p==place && t==ty) {return None}
                self.private_bytes(r,*place,*ty,*bytes)?.then_some(())
            }
            Event::Call {
                callee,
                args,
                expands: false,
                ..
            } => self.call(r, site, *callee, args, all),
            Event::Return {
                value,
                index,
                ty,
                ..
            } => {
                if self.returned.is_some()
                    || self.count_write.is_none()
                    || *ty != self.pointer
                    || *index != 0
                {
                    return None;
                }
                let node = self.element(r, *value, site, &mut HashSet::default(), 0)?;
                if self.initialized?.0 != node || node.count.event >= self.count_write?.event {
                    return None;
                }
                self.returned = Some((node, *index, site));
                Some(())
            }
            Event::ReadMemory {
                value, ..
            } => {
                self.charge(r.body.blocks[0].events.len())?;
                let ty = r.body.blocks[0].events.iter().find_map(|e| match e {
                    Event::Indirect {
                        place,
                        ty,
                    } if place == value => Some(*ty),
                    _ => None,
                })?;
                self.address(r, *value, r.compiler.ty.types.size_of(ty))
                    .map(|_| ())
            }
            Event::View {
                from,
                to,
                ..
            } if r.compiler.ty.types.is_pointer(*from) && r.compiler.ty.types.is_pointer(*to) => {
                Some(())
            }
            Event::Call {callee,expands:true,..} => {
                if all.get(callee).is_none_or(|s|s.trusted.is_some()) {return None;}
                let dependency=super::frames::Requirement::Constraint(*callee);
                self.charge(self.dependencies.len()+1)?;
                if !self.dependencies.contains(&dependency) {
                    if self.dependencies.len()==MAX_CALLS {return None;}
                    self.dependencies.push(dependency);
                }
                Some(())
            },
            Event::FieldProjection {
                ..
            }
            | Event::Field {
                ..
            }
            | Event::Boolean {
                ..
            }
            | Event::ContextSet {
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
            | Event::Procedure {
                ..
            } => Some(()),
            _ => None,
        }
    }

    pub fn finish(mut self, r: &Resolver<'_>, all: &HashMap<ProcId, Summary>) -> Option<Borrow> {
        self.charge(
            self.dependencies.len()
                + 1
                + all
                    .get(&self.owner.procedure())
                    .and_then(|s| s.trusted.as_ref())
                    .map_or(0, |t| t.work()),
        )?;
        if !self.valid
            || jaic::interrupt::should_stop()
            || !self.owner.current(r.compiler)
            || self.dependencies.iter().any(|d| !d.current(r.compiler))
        {
            return None;
        }
        let (node, result, _) = self.returned?;
        let result = Borrow {
            owner: self.owner,
            dependencies: self.dependencies,
            conditions: self.conditions,
            receiver: 0,
            header: self.header,
            element: self.element,
            result,
            data_read: node.data,
            count_read: node.count,
            offset: node.offset,
            initialize: self.initialized?.1,
            count_write: self.count_write?,
        };
        result.compatible(all).then_some(result)
    }
}
