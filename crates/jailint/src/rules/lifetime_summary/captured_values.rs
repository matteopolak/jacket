//! Complete body-local SSA definitions shared by bit and native return readers.
use super::{Event, HashMap, LifetimeBody, ProcId, TypeId, Val};
use crate::rules::lifetime_movement::facts::Site;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::rules) enum Producer {
    Parameter(usize, TypeId),
    Load(TypeId, Site),
    Copy(Val),
    Binary(TypeId, Val, Val, jaic::ir::BinOp, Option<u64>, Option<u64>),
    Cast(Val, bool, u8),
    Not(Val),
    And(Val, Val),
    Or(Val, Val),
    Comparison(usize),
    Call(ProcId, Site, usize, TypeId),
}

/// Missing and poisoned definitions are unknown; insertion order cannot select a winner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CaptureRow {
    /// A fully expanded call observes an existing producer; it cannot create one.
    Observation,
    Parameter(usize, TypeId),
    Load(TypeId),
    Copy(Val),
    View(Val, TypeId, TypeId),
    Projection(Val, TypeId, TypeId, u64),
    Offset(Val, Option<i128>, TypeId),
    Cast,
    Address,
    Local,
    Metadata,
    Terminal,
}
struct Definition {
    producer: Option<Producer>,
    typed: bool,
    captures: Vec<CaptureRow>,
}
pub(in crate::rules) struct Producers {
    rows: HashMap<Val, Definition>,
}
/// A charged per-proof view. Cloning shares immutable checked definitions only.
#[derive(Clone)]
pub(in crate::rules) struct QueryIndex(Rc<Producers>);
impl std::ops::Deref for QueryIndex {
    type Target = Producers;
    fn deref(&self) -> &Producers {
        &self.0
    }
}
/// A failed attempt never exposes a partial map as usable opaque terminals.
pub(super) struct IndexResult {
    index: Option<Rc<Producers>>,
    pub work: usize,
}
impl IndexResult {
    pub fn build(core: &super::prepared::Core<'_>) -> Self {
        let checked = core.checked_body();
        let body = checked.body();
        let mut work = 0;
        let index = Producers::build(checked, core.context_slot, &mut work, 8192).map(Rc::new);
        if jacket_config::env::LIFETIME_STATS.is_set() {
            eprintln!(
                "lifetime-capture-index proc={} complete={} work={} payload_bytes={}",
                body.procedure.0,
                index.is_some(),
                work,
                index.as_ref().map_or(0, |i| i.payload_bytes())
            );
        }
        Self {
            index,
            work,
        }
    }
    pub fn owns(&self, index: &QueryIndex) -> bool {
        self.index
            .as_ref()
            .is_some_and(|own| Rc::ptr_eq(own, &index.0))
    }
    pub fn reserve(&self, remaining: &mut usize) -> Option<QueryIndex> {
        *remaining = remaining.checked_sub(self.work)?;
        self.index.as_ref().map(|i| QueryIndex(Rc::clone(i)))
    }
}

fn charge(work: &mut usize, n: usize, limit: usize) -> Option<()> {
    *work = work.checked_add(n)?;
    (*work <= limit && !jaic::interrupt::should_stop()).then_some(())
}

impl Producers {
    /// Reserve the complete retained-row inventory before materialization.
    fn preflight(body: &LifetimeBody, work: &mut usize, limit: usize) -> Option<()> {
        charge(
            work,
            body.blocks.len() + body.address_casts.len() + body.comparisons.len(),
            limit,
        )?;
        let mut count = body
            .address_casts
            .len()
            .checked_add(body.comparisons.len())?;
        for b in &body.blocks {
            charge(work, b.events.len() + 1, limit)?;
            count = count.checked_add(b.events.len())?;
            for e in &b.events {
                if let Event::Call {
                    results, ..
                }
                | Event::IndirectCall {
                    results, ..
                } = e
                {
                    charge(work, results.len(), limit)?;
                    count = count.checked_add(results.len())?;
                }
            }
        }
        charge(work, count.checked_mul(3)?, limit)?;
        Some(())
    }

    /// Build the complete index before any query. For example, a later View
    /// defining an already seen result must poison that result in either order.
    pub fn build(
        checked: crate::rules::lifetime_facts::CheckedBody<'_>,
        context_slot: Option<Val>,
        work: &mut usize,
        limit: usize,
    ) -> Option<Self> {
        let body = checked.body();
        let compiler = checked.compiler();
        Self::preflight(body, work, limit)?;
        let mut out = Self {
            rows: HashMap::default(),
        };
        for (block, b) in body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                let site = Site {
                    block,
                    event,
                };
                let capture = match e {
                    Event::Parameter {
                        incoming,
                        formal,
                        ty,
                        ..
                    } => Some((*incoming, CaptureRow::Parameter(*formal, *ty))),
                    Event::Load {
                        place,
                        value,
                        ty,
                        ..
                    } if place == value && compiler.ir_ty(*ty).is_none() => {
                        Some((*value, CaptureRow::Observation))
                    }
                    Event::Load {
                        value,
                        ty,
                        ..
                    } => Some((*value, CaptureRow::Load(*ty))),
                    Event::ContextRead {
                        value,
                        place,
                        ty,
                    } => Some((
                        *value,
                        if context_slot == Some(*place) && *ty == TypeId::VOID_PTR {
                            CaptureRow::Load(*ty)
                        } else {
                            CaptureRow::Observation
                        },
                    )),
                    Event::Copy {
                        value,
                        source,
                    } => Some((*value, CaptureRow::Copy(*source))),
                    Event::View {
                        value,
                        source,
                        from,
                        to,
                        ..
                    } => Some((*value, CaptureRow::View(*source, *from, *to))),
                    Event::FieldProjection {
                        result,
                        base,
                        base_ty,
                        result_ty,
                        offset,
                    } => Some((
                        *result,
                        CaptureRow::Projection(*base, *base_ty, *result_ty, *offset),
                    )),
                    Event::Offset {
                        value,
                        source,
                        offset,
                        stride,
                        ..
                    } => Some((*value, CaptureRow::Offset(*source, *offset, *stride))),
                    Event::Address {
                        place, ..
                    } => Some((*place, CaptureRow::Address)),
                    Event::Local {
                        place, ..
                    } => Some((*place, CaptureRow::Local)),
                    Event::Field {
                        result: super::LifetimeOperand::Value(v) | super::LifetimeOperand::Place(v),
                        ..
                    } => Some((*v, CaptureRow::Metadata)),
                    Event::IntegerBinary {
                        value, ..
                    }
                    | Event::Boolean {
                        value, ..
                    }
                    | Event::Borrow {
                        value, ..
                    }
                    | Event::ContextAlias {
                        value, ..
                    }
                    | Event::Procedure {
                        value, ..
                    }
                    | Event::EmptyDescriptor {
                        value, ..
                    } => Some((*value, CaptureRow::Terminal)),
                    Event::Index {
                        result, ..
                    } => Some((*result, CaptureRow::Terminal)),
                    Event::BoxAny {
                        place, ..
                    } => Some((*place, CaptureRow::Terminal)),
                    _ => None,
                };
                let row = match e {
                    Event::FieldProjection {
                        base,
                        result,
                        offset: 0,
                        ..
                    } if base == result => None,
                    Event::Field {
                        ..
                    } => None,
                    Event::View {
                        value,
                        source,
                        from,
                        to,
                        ..
                    } if value == source
                        && compiler.ty.types.is_pointer(*from)
                        && compiler.ty.types.is_pointer(*to)
                        && compiler.ir_ty(*from) == Some(jaic::ir::Ty::Ptr)
                        && compiler.ir_ty(*to) == Some(jaic::ir::Ty::Ptr) =>
                    {
                        None
                    }
                    Event::Parameter {
                        incoming,
                        formal,
                        ty,
                        ..
                    } => Some((*incoming, Some(Producer::Parameter(*formal, *ty)))),
                    Event::Load {
                        place,
                        value,
                        ty,
                        ..
                    } if place == value && compiler.ir_ty(*ty).is_none() => None,
                    Event::Load {
                        value,
                        ty,
                        ..
                    } => Some((*value, Some(Producer::Load(*ty, site)))),
                    Event::ContextRead {
                        value,
                        place,
                        ty,
                    } => Some((
                        *value,
                        (context_slot == Some(*place) && *ty == TypeId::VOID_PTR)
                            .then_some(Producer::Load(*ty, site)),
                    )),
                    Event::Copy {
                        value,
                        source,
                    } if value != source => Some((*value, Some(Producer::Copy(*source)))),
                    Event::IntegerBinary {
                        value,
                        ty,
                        left,
                        right,
                        op,
                        left_constant,
                        right_constant,
                    } => Some((
                        *value,
                        Some(Producer::Binary(
                            *ty,
                            *left,
                            *right,
                            *op,
                            *left_constant,
                            *right_constant,
                        )),
                    )),
                    Event::Boolean {
                        value,
                        operation: jaic::ide::lifetimes::LifetimeBoolean::Not(source),
                    } => Some((*value, Some(Producer::Not(*source)))),
                    Event::Boolean {
                        value,
                        operation: jaic::ide::lifetimes::LifetimeBoolean::And(a, b),
                    } => Some((*value, Some(Producer::And(*a, *b)))),
                    Event::Boolean {
                        value,
                        operation: jaic::ide::lifetimes::LifetimeBoolean::Or(a, b),
                    } => Some((*value, Some(Producer::Or(*a, *b)))),
                    Event::Offset {
                        value, ..
                    }
                    | Event::Borrow {
                        value, ..
                    }
                    | Event::ContextAlias {
                        value, ..
                    }
                    | Event::View {
                        value, ..
                    }
                    | Event::Procedure {
                        value, ..
                    }
                    | Event::EmptyDescriptor {
                        value, ..
                    } => Some((*value, None)),
                    Event::FieldProjection {
                        result, ..
                    }
                    | Event::Index {
                        result, ..
                    } => Some((*result, None)),
                    Event::Local {
                        place, ..
                    }
                    | Event::BoxAny {
                        place, ..
                    } => Some((*place, None)),
                    // Expanded calls observe values defined by their recorded
                    // runtime operations; they are not another SSA producer.
                    Event::Call {
                        expands: true,
                        results,
                        ..
                    } => {
                        for &(_, value, _) in results {
                            out.record(value, Some(CaptureRow::Observation), None);
                        }
                        None
                    }
                    Event::Call {
                        callee,
                        results,
                        ..
                    } => {
                        for &(index, value, ty) in results {
                            out.record(
                                value,
                                Some(CaptureRow::Terminal),
                                Some(Some(Producer::Call(*callee, site, index, ty))),
                            );
                        }
                        None
                    }
                    Event::IndirectCall {
                        results, ..
                    } => {
                        for &(_, value, _) in results {
                            out.record(value, Some(CaptureRow::Terminal), Some(None));
                        }
                        None
                    }
                    _ => None,
                };
                match (capture, row) {
                    (Some((value, capture)), Some((typed_value, typed))) => {
                        if value != typed_value {
                            return None;
                        }
                        out.record(value, Some(capture), Some(typed));
                    }
                    (Some((value, capture)), None) => out.record(value, Some(capture), None),
                    (None, Some((value, typed))) => out.record(value, None, Some(typed)),
                    (None, None) => {}
                }
            }
        }
        for cast in &body.address_casts {
            out.record(
                cast.value,
                Some(CaptureRow::Cast),
                Some(Some(Producer::Cast(
                    cast.source,
                    cast.to_pointer,
                    cast.integer_bytes,
                ))),
            );
        }
        for (index, comparison) in body.comparisons.iter().enumerate() {
            out.record(
                comparison.value,
                Some(CaptureRow::Terminal),
                Some(Some(Producer::Comparison(index))),
            );
        }
        (!jaic::interrupt::should_stop()).then_some(out)
    }

    /// Three bounded logical operations: one entry, capture publication, typed merge.
    /// None means no typed update; Some(None) is an absorbing poisoned definition.
    fn record(&mut self, value: Val, capture: Option<CaptureRow>, typed: Option<Option<Producer>>) {
        let entry = self.rows.entry(value).or_insert(Definition {
            producer: None,
            typed: false,
            captures: Vec::new(),
        });
        if let Some(capture) = capture {
            entry.captures.push(capture);
        }
        if let Some(typed) = typed {
            if !entry.typed {
                entry.producer = typed;
                entry.typed = true;
            } else if entry.producer != typed {
                entry.producer = None;
            }
        }
    }
    pub fn contains(&self, value: Val) -> bool {
        self.rows.contains_key(&value)
    }
    pub fn get(&self, value: Val) -> Option<Producer> {
        self.rows.get(&value)?.producer
    }
    pub(super) fn captures(&self, value: Val) -> &[CaptureRow] {
        self.rows.get(&value).map_or(&[], |d| d.captures.as_slice())
    }
    fn payload_bytes(&self) -> usize {
        self.rows.capacity() * std::mem::size_of::<(Val, Definition)>()
            + self
                .rows
                .values()
                .map(|d| d.captures.capacity() * std::mem::size_of::<CaptureRow>())
                .sum::<usize>()
    }
}

use super::address_objects::{Base, Object};
use super::storage_versions::Source;
use super::{HashSet, Resolver};
use crate::rules::lifetime_facts::ByteRange;

fn take(work: &mut usize, n: usize) -> Option<()> {
    *work = work.checked_sub(n)?;
    (!jaic::interrupt::should_stop()).then_some(())
}
impl QueryIndex {
    /// Every alternative comes from the actual ordered Load epoch. Numeric bits
    /// and aggregate snapshots never enter a pointer-identity query.
    pub(super) fn pointer_leaf(
        &self,
        r: &Resolver<'_>,
        value: Val,
        work: &mut usize,
        depth: usize,
    ) -> Option<(Val, usize)> {
        take(work, 1)?;
        if !r.owns_capture_index(self) {
            return None;
        }
        self.pointer_node(
            r,
            value,
            work,
            depth,
            &mut HashSet::default(),
            (&r.core().storage_versions, None),
        )
    }
    /// Polarity of an exact pointer/null comparison at its captured epoch.
    /// This proves equality only; flow and allocation obligations stay outside.
    pub(super) fn null_test(
        &self,
        r: &Resolver<'_>,
        loads: &super::storage_versions::CapturedLoads<'_>,
        value: Val,
        root: Val,
        work: &mut usize,
        depth: usize,
    ) -> Option<bool> {
        take(work, 1)?;
        if depth >= 24 || !r.owns_capture_index(self) || !loads.matches(r.checked_body()) {
            return None;
        }
        match self.get(value)? {
            Producer::Not(source) => {
                Some(!self.null_test(r, loads, source, root, work, depth + 1)?)
            }
            Producer::Comparison(id) => {
                take(work, 1)?;
                let cmp = r.body.comparisons.get(id)?;
                if cmp.ty != jaic::ir::Ty::Ptr
                    || !matches!(cmp.op, jaic::ir::CmpOp::Eq | jaic::ir::CmpOp::Ne)
                {
                    return None;
                }
                let operand = match (cmp.left_constant, cmp.right_constant) {
                    (None, Some(0)) => cmp.left,
                    (Some(0), None) => cmp.right,
                    _ => return None,
                };
                let leaf = self.pointer_node(
                    r,
                    operand,
                    work,
                    depth + 1,
                    &mut HashSet::default(),
                    (loads, None),
                )?;
                (leaf.0 == root).then_some(cmp.op == jaic::ir::CmpOp::Ne)
            }
            _ => None,
        }
    }

    fn pointer_node(
        &self,
        r: &Resolver<'_>,
        value: Val,
        work: &mut usize,
        depth: usize,
        seen: &mut HashSet<Val>,
        scope: (&super::storage_versions::CapturedLoads<'_>, Option<Val>),
    ) -> Option<(Val, usize)> {
        take(work, self.captures(value).len() + 1)?;
        if depth >= 24 {
            return None;
        }
        if scope.1 == Some(value) {
            return Some((value, depth));
        }
        if !seen.insert(value) {
            return None;
        }
        let result = (|| {
            if !self.captures(value).is_empty()
                && self.get(value).is_none()
                && self.captures(value).iter().all(|row| match *row {
                    CaptureRow::Observation | CaptureRow::Metadata => true,
                    CaptureRow::Projection(base, _, _, 0) => base == value,
                    CaptureRow::View(source, from, to) => {
                        source == value
                            && r.compiler.ty.types.is_pointer(from)
                            && r.compiler.ty.types.is_pointer(to)
                            && r.compiler.ir_ty(from) == Some(jaic::ir::Ty::Ptr)
                            && r.compiler.ir_ty(to) == Some(jaic::ir::Ty::Ptr)
                    }
                    _ => false,
                })
            {
                return None;
            }
            let mut result: Option<(Val, usize)> = None;
            let mut opaque = 0usize;
            for row in self.captures(value) {
                let current = match *row {
                    CaptureRow::Observation => continue,
                    CaptureRow::Copy(source) if source != value => {
                        Some(self.pointer_node(r, source, work, depth + 1, seen, scope)?)
                    }
                    CaptureRow::View(source, from, to) => {
                        if !r.compiler.ty.types.is_pointer(from)
                            || !r.compiler.ty.types.is_pointer(to)
                            || r.compiler.ty.types.size_of(from) != r.compiler.ty.types.size_of(to)
                        {
                            return None;
                        }
                        if source == value {
                            None
                        } else {
                            Some(self.pointer_node(r, source, work, depth + 1, seen, scope)?)
                        }
                    }
                    CaptureRow::Load(ty) => {
                        if !r.compiler.ty.types.is_pointer(ty) {
                            return None;
                        }
                        let Some(sources) = scope.0.get(&value) else {
                            if jacket_config::env::LIFETIME_STATS.is_set() {
                                eprintln!(
                                    "lifetime-bound-missing-load value={value:?} ty={ty:?} rows={:?}",
                                    self.captures(value)
                                );
                            }
                            return None;
                        };
                        take(work, sources.len())?;
                        let mut loaded: Option<(Val, usize)> = None;
                        for source in sources {
                            let Source::Value(source) = source else {
                                return None;
                            };
                            let next =
                                self.pointer_node(r, *source, work, depth + 1, seen, scope)?;
                            if loaded.is_some_and(|old| old.0 != next.0) {
                                return None;
                            }
                            loaded = Some((next.0, loaded.map_or(next.1, |old| old.1.max(next.1))));
                        }
                        Some(loaded?)
                    }
                    CaptureRow::Offset(source, offset, _)
                        if source != value || offset != Some(0) =>
                    {
                        opaque += 1;
                        Some((value, depth))
                    }
                    CaptureRow::Projection(base, _, _, offset) if base != value || offset != 0 => {
                        opaque += 1;
                        Some((value, depth))
                    }
                    CaptureRow::Terminal | CaptureRow::Parameter(..) => {
                        opaque += 1;
                        Some((value, depth))
                    }
                    CaptureRow::Cast => return None,
                    _ => None,
                };
                if opaque > 1 {
                    return None;
                }
                if let Some(next) = current {
                    if result.is_some_and(|old| old.0 != next.0) {
                        return None;
                    }
                    result = Some((next.0, result.map_or(next.1, |old| old.1.max(next.1))));
                }
            }
            Some(result.unwrap_or((value, depth)))
        })();
        seen.remove(&value);
        result.filter(|_| !jaic::interrupt::should_stop())
    }

    /// Typed inline byte addressing; ownership/extent validation remains with
    /// the consumer. No entry-slot or syntactic single-write recovery exists.
    pub(super) fn address(&self, r: &Resolver<'_>, value: Val, work: &mut usize) -> Option<Object> {
        take(work, 1)?;
        if !r.owns_capture_index(self) {
            return None;
        }
        let object = self.address_node(r, value, work, 0, &mut 256, &mut HashSet::default())?;
        if matches!(object.base, Base::Local(_)) {
            take(work, r.places.paths.len())?;
        }
        r.validate_lock_object(object)
    }
    fn address_node(
        &self,
        r: &Resolver<'_>,
        value: Val,
        work: &mut usize,
        depth: usize,
        budget: &mut usize,
        seen: &mut HashSet<Val>,
    ) -> Option<Object> {
        take(work, self.captures(value).len() + 1)?;
        *budget = budget.checked_sub(1)?;
        if depth > 24 || !seen.insert(value) {
            return None;
        }
        let result = (|| {
            let mut projected = None;
            for row in self.captures(value) {
                if let CaptureRow::Projection(base, _, _, offset) = *row
                    && base != value
                {
                    let prior = self.address_node(r, base, work, depth + 1, budget, seen)?;
                    let object = Object {
                        base: prior.base,
                        offset: prior.offset.checked_add(offset)?,
                    };
                    if projected.is_some_and(|old| old != object) {
                        return None;
                    }
                    projected = Some(object);
                }
            }
            // Retain counted address query's original projection precedence.
            if projected.is_some() {
                return projected;
            }
            if self
                .captures(value)
                .iter()
                .any(|row| matches!(row,CaptureRow::Offset(_,offset,_) if *offset!=Some(0)))
            {
                return None;
            }
            let mut result = None;
            for row in self.captures(value) {
                let object = match *row {
                    CaptureRow::Observation => continue,
                    CaptureRow::Parameter(formal, ty) => {
                        if !r.compiler.ty.types.is_pointer(ty) {
                            return None;
                        }
                        Object::input(formal)
                    }
                    CaptureRow::Copy(source) if source != value => {
                        self.address_node(r, source, work, depth + 1, budget, seen)?
                    }
                    CaptureRow::View(source, from, to) if source != value => {
                        if !r.compiler.ty.types.is_pointer(from)
                            || !r.compiler.ty.types.is_pointer(to)
                        {
                            return None;
                        }
                        self.address_node(r, source, work, depth + 1, budget, seen)?
                    }
                    CaptureRow::Offset(source, Some(0), _) if source != value => {
                        self.address_node(r, source, work, depth + 1, budget, seen)?
                    }
                    CaptureRow::Load(ty) => {
                        if !r.compiler.ty.types.is_pointer(ty) {
                            return None;
                        }
                        let sources = r.storage_versions.get(&value)?;
                        take(work, sources.len())?;
                        let mut sources = sources.iter();
                        let Source::Value(source) = sources.next()? else {
                            return None;
                        };
                        if sources.next().is_some() {
                            return None;
                        }
                        self.address_node(r, *source, work, depth + 1, budget, seen)?
                    }
                    CaptureRow::Metadata | CaptureRow::Local | CaptureRow::Projection(..) => {
                        continue;
                    }
                    CaptureRow::Copy(source) | CaptureRow::Offset(source, Some(0), _)
                        if source == value =>
                    {
                        continue;
                    }
                    CaptureRow::View(source, from, to) if source == value => {
                        if !r.compiler.ty.types.is_pointer(from)
                            || !r.compiler.ty.types.is_pointer(to)
                        {
                            return None;
                        }
                        continue;
                    }
                    CaptureRow::Address => return None,
                    _ => return None,
                };
                if result.is_some_and(|old| old != object) {
                    return None;
                }
                result = Some(object);
            }
            result.or(Some(Object {
                base: Base::Local(value),
                offset: 0,
            }))
        })();
        seen.remove(&value);
        result.filter(|_| !jaic::interrupt::should_stop())
    }

    pub(in crate::rules) fn write_location(
        &self,
        r: &Resolver<'_>,
        value: Val,
        ty: TypeId,
        work: &mut usize,
        depth: usize,
    ) -> Option<(Val, ByteRange)> {
        let (root, offset, width) = self.relative_write(
            r,
            (&r.core().storage_versions, None),
            (value, ty),
            work,
            depth,
        )?;
        Some((
            root,
            ByteRange {
                offset: u64::try_from(offset).ok()?,
                width,
            },
        ))
    }

    /// Coordinates relative to one explicitly bound address SSA, never an
    /// allocation/liveness certificate. The supplied captures own this exact cut.
    pub(super) fn relative_write(
        &self,
        r: &Resolver<'_>,
        scope: (&super::storage_versions::CapturedLoads<'_>, Option<Val>),
        target: (Val, TypeId),
        work: &mut usize,
        depth: usize,
    ) -> Option<(Val, i128, u64)> {
        take(work, 1)?;
        if !r.owns_capture_index(self) || !scope.0.matches(r.checked_body()) {
            return None;
        }
        self.write_location_node(r, scope, target, work, depth)
    }

    fn write_location_node(
        &self,
        r: &Resolver<'_>,
        scope: (&super::storage_versions::CapturedLoads<'_>, Option<Val>),
        (value, ty): (Val, TypeId),
        work: &mut usize,
        depth: usize,
    ) -> Option<(Val, i128, u64)> {
        take(work, self.captures(value).len() + 1)?;
        if depth >= 24 {
            return None;
        }
        let width = r.compiler.ty.types.size_of(ty);
        if width == 0 {
            return None;
        }
        if scope.1 == Some(value) {
            return Some((value, 0, width));
        }
        let mut selected = None;
        let mut view = None;
        let mut incompatible_view = false;
        let mut displaced = None;
        for row in self.captures(value) {
            if let CaptureRow::Offset(source, offset, stride) = *row
                && (source != value || offset != Some(0))
            {
                if scope.1.is_none() || source == value {
                    return None;
                }
                take(work, 1)?;
                let stride_bytes = r.compiler.ty.types.size_of(stride);
                if stride_bytes == 0 {
                    return None;
                }
                let delta = offset?.checked_mul(i128::from(stride_bytes))?;
                i64::try_from(delta).ok()?;
                let current = (source, delta);
                if displaced.is_some_and(|old| old != current) {
                    return None;
                }
                displaced = Some(current);
            }
            if let CaptureRow::Projection(base, base_ty, result_ty, offset) = *row {
                let selected_width = r.compiler.ty.types.size_of(result_ty);
                if selected_width == 0
                    || offset.checked_add(selected_width)? > r.compiler.ty.types.size_of(base_ty)
                {
                    return None;
                }
                if base == value {
                    if offset != 0 {
                        return None;
                    }
                    if result_ty == ty && base_ty != result_ty {
                        let next = (base_ty, result_ty);
                        if view.replace(next).is_some_and(|old| old != next) {
                            return None;
                        }
                    } else if result_ty != ty && base_ty != result_ty {
                        incompatible_view = true;
                    }
                    continue;
                }
                let current = (base, base_ty, result_ty, offset);
                if selected.is_some_and(|old| old != current) {
                    return None;
                }
                selected = Some(current);
            }
        }

        if let Some((base, base_ty, result_ty, offset)) = selected
            && base != value
        {
            if displaced.is_some()
                || (result_ty != ty && (incompatible_view || view != Some((result_ty, ty))))
                || width > r.compiler.ty.types.size_of(result_ty)
            {
                return None;
            }
            let (root, at, _) =
                self.write_location_node(r, scope, (base, base_ty), work, depth + 1)?;
            let at = at.checked_add(i128::from(offset))?;
            if scope.1.is_some() {
                i64::try_from(at).ok()?;
            }
            return Some((root, at, width));
        }
        if let Some((source, delta)) = displaced {
            if view.is_some() {
                return None;
            }
            let (root, at, _) =
                self.write_location_node(r, scope, (source, ty), work, depth + 1)?;
            let at = at.checked_add(delta)?;
            i64::try_from(at).ok()?;
            return Some((root, at, width));
        }
        let (leaf, used) =
            self.pointer_node(r, value, work, depth, &mut HashSet::default(), scope)?;
        if leaf != value {
            return self.write_location_node(r, scope, (leaf, ty), work, used + 1);
        }
        (scope.1.is_none() && !jaic::interrupt::should_stop()).then_some((value, 0, width))
    }
}

#[cfg(test)]
mod record_tests {
    use super::*;
    #[test]
    fn combined_record_preserves_capture_order_and_absorbing_typed_conflicts() {
        let value = Val(17);
        let parameter = Producer::Parameter(0, TypeId::U64);
        for capture_first in [false, true] {
            let mut rows = Producers {
                rows: HashMap::default(),
            };
            if capture_first {
                rows.record(value, Some(CaptureRow::Copy(Val(3))), None);
            }
            rows.record(value, None, Some(Some(parameter)));
            if !capture_first {
                rows.record(value, Some(CaptureRow::Copy(Val(3))), None);
            }
            rows.record(value, Some(CaptureRow::Copy(Val(3))), Some(Some(parameter)));
            assert!(rows.get(value) == Some(parameter));
            assert_eq!(
                rows.captures(value),
                [CaptureRow::Copy(Val(3)), CaptureRow::Copy(Val(3))]
            );
            rows.record(value, None, Some(None));
            rows.record(value, None, Some(Some(parameter)));
            assert!(
                rows.get(value).is_none(),
                "a poisoned definition cannot recover"
            );
        }
        for reverse in [false, true] {
            let mut rows = Producers {
                rows: HashMap::default(),
            };
            let definitions = if reverse {
                [Producer::Copy(Val(2)), parameter]
            } else {
                [parameter, Producer::Copy(Val(2))]
            };
            for definition in definitions {
                rows.record(value, None, Some(Some(definition)));
            }
            assert!(
                rows.get(value).is_none(),
                "conflicts reject independently of row order"
            );
        }
    }
}
