//! Demand-only cache of complete target layouts, with bounded cold computation.
use crate::{Error, LimitKind};
use jai_types::{Layout, LayoutEngine, LayoutPolicy, ScalarType, TypeId, TypeKind, TypeView};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct RootLayoutCache {
    policy: LayoutPolicy,
    registry: Option<TypeId>,
    layouts: HashMap<TypeId, ReadyLayout>,
    cells: usize,
}
#[derive(Clone)]
struct ReadyLayout {
    layout: Arc<Layout>,
    decoded: DecodedShape,
    codec_work: usize,
    allocator_role: Option<Option<jai_types::AllocatorSchema>>,
}
#[derive(Clone, Copy)]
struct DecodedShape {
    cells: usize,
    height: usize,
}
#[derive(Clone, Copy)]
struct TypeFacts {
    offsets: usize,
    decoded: DecodedShape,
    codec_work: usize,
    allocator_role: Option<Option<jai_types::AllocatorSchema>>,
}
impl RootLayoutCache {
    pub(super) fn new(policy: LayoutPolicy) -> Self {
        Self {
            policy,
            registry: None,
            layouts: HashMap::new(),
            cells: 0,
        }
    }
    pub(super) fn cells(&self) -> usize {
        self.cells
    }
    pub(super) fn table_capacity(&self) -> usize {
        self.layouts.capacity()
    }
    fn prepared(&self, types: &dyn TypeView, ty: TypeId) -> Result<Arc<Layout>, Error> {
        self.check_registry(types, ty)?;
        self.ready(types, ty)?
            .map(|ready| Arc::clone(&ready.layout))
            .ok_or(Error::InvalidIr(
                "target layout was not prepared before execution",
            ))
    }
    fn prepared_decoded(&self, types: &dyn TypeView, ty: TypeId) -> Result<DecodedShape, Error> {
        self.check_registry(types, ty)?;
        self.ready(types, ty)?
            .map(|ready| ready.decoded)
            .ok_or(Error::InvalidIr(
                "decoded type shape was not prepared before execution",
            ))
    }
    fn prepared_codec_work(&self, types: &dyn TypeView, ty: TypeId) -> Result<usize, Error> {
        self.check_registry(types, ty)?;
        self.ready(types, ty)?
            .map(|ready| ready.codec_work)
            .ok_or(Error::InvalidIr(
                "codec layout work was not prepared before execution",
            ))
    }
    fn ready(&self, types: &dyn TypeView, ty: TypeId) -> Result<Option<&ReadyLayout>, Error> {
        let ready = self.layouts.get(&ty);
        if ready.is_some_and(|ready| {
            ready
                .allocator_role
                .is_some_and(|role| role != types.allocator_schema())
        }) {
            return Err(Error::InvalidIr(
                "prepared type storage belongs to an earlier allocator role",
            ));
        }
        Ok(ready)
    }
    fn check_registry(&self, types: &dyn TypeView, ty: TypeId) -> Result<TypeId, Error> {
        types.kind(ty)?;
        let registry = types.scalar(ScalarType::Bool);
        if self.registry.is_some_and(|old| old != registry) {
            return Err(Error::InvalidIr(
                "layout cache belongs to a different type registry",
            ));
        }
        Ok(registry)
    }
    /// Cold cost includes the full demanded closure, even dependencies cached as other roots.
    /// No layout is calculated or retained by this preflight.
    #[cfg(test)]
    pub(super) fn work_cost(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
        cell_limit: usize,
        work_limit: usize,
    ) -> Result<usize, Error> {
        self.check_registry(types, ty)?;
        if self.cells > cell_limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        if self.ready(types, ty)?.is_some() {
            if work_limit == 0 {
                return Err(Error::Limit(LimitKind::Fuel));
            }
            return Ok(1);
        }
        let plan = preflight(types, ty, cell_limit, work_limit)?;
        self.check_retention(plan.root_offsets, cell_limit)?;
        self.checked_work(plan.work, work_limit)
    }
    /// VM callers use `layout_attempt` and charge its consumed work, including errors.
    /// The independent work bound also applies when Memory is used without a VM.
    pub(super) fn layout(
        &mut self,
        types: &dyn TypeView,
        ty: TypeId,
        cell_limit: usize,
        work_limit: usize,
    ) -> Result<Arc<Layout>, Error> {
        self.layout_attempt(types, ty, cell_limit, work_limit).1
    }
    /// Failed and Pending computations report consumed preflight work too.
    pub(super) fn layout_attempt(
        &mut self,
        types: &dyn TypeView,
        ty: TypeId,
        cell_limit: usize,
        work_limit: usize,
    ) -> (usize, Result<Arc<Layout>, Error>) {
        let mut budget = Budget {
            cells: 0,
            work: 0,
            cell_limit,
            work_limit,
        };
        let result = (|| {
            budget.reserve(0, 1)?;
            let registry = self.check_registry(types, ty)?;
            if self.cells > cell_limit {
                return Err(Error::Limit(LimitKind::ValueCells));
            }
            if let Some(layout) = self.ready(types, ty)? {
                return Ok(Arc::clone(&layout.layout));
            }
            let facts = preflight_inner(types, ty, &mut budget)?;
            let root_offsets = facts.offsets;
            let growth = if self.layouts.len() == self.layouts.capacity() {
                self.layouts.capacity()
            } else {
                0
            };
            budget.reserve(0, growth)?;
            let cells = self.check_retention(root_offsets, cell_limit)?;
            let mut engine = LayoutEngine::new(types, self.policy);
            let layout = engine.layout(ty)?;
            if layout.field_offsets.len() != root_offsets {
                return Err(Error::InvalidIr(
                    "type storage changed during layout computation",
                ));
            }
            let layout = Arc::new(layout.clone());
            self.layouts.insert(
                ty,
                ReadyLayout {
                    layout: Arc::clone(&layout),
                    decoded: facts.decoded,
                    codec_work: facts.codec_work,
                    allocator_role: facts.allocator_role,
                },
            );
            self.registry = Some(registry);
            self.cells = cells;
            Ok(layout)
        })();
        (budget.work, result)
    }
    fn check_retention(&self, offsets: usize, limit: usize) -> Result<usize, Error> {
        self.cells
            .checked_add(1)
            .and_then(|n| n.checked_add(offsets))
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
    #[cfg(test)]
    fn checked_work(&self, work: usize, limit: usize) -> Result<usize, Error> {
        let growth = if self.layouts.len() == self.layouts.capacity() {
            self.layouts.capacity()
        } else {
            0
        };
        work.checked_add(growth)
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::Fuel))
    }
}
#[cfg(test)]
struct Plan {
    root_offsets: usize,
    work: usize,
}
struct Budget {
    cells: usize,
    work: usize,
    cell_limit: usize,
    work_limit: usize,
}
impl Budget {
    fn reserve(&mut self, cells: usize, work: usize) -> Result<(), Error> {
        let Some(next_work) = self
            .work
            .checked_add(work)
            .filter(|n| *n <= self.work_limit)
        else {
            self.work = self.work_limit;
            return Err(Error::Limit(LimitKind::Fuel));
        };
        self.work = next_work;
        let next_cells = self
            .cells
            .checked_add(cells)
            .filter(|n| *n <= self.cell_limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.cells = next_cells;
        self.work = next_work;
        Ok(())
    }
}
#[cfg(test)]
fn preflight(
    types: &dyn TypeView,
    root: TypeId,
    cell_limit: usize,
    work_limit: usize,
) -> Result<Plan, Error> {
    let mut budget = Budget {
        cells: 0,
        work: 0,
        cell_limit,
        work_limit,
    };
    budget.reserve(0, 1)?;
    let root_offsets = preflight_inner(types, root, &mut budget)?.offsets;
    Ok(Plan {
        root_offsets,
        work: budget.work,
    })
}
fn preflight_inner(
    types: &dyn TypeView,
    root: TypeId,
    budget: &mut Budget,
) -> Result<TypeFacts, Error> {
    budget.reserve(1, 0)?;
    let mut pending = vec![(root, false)];
    let mut visiting = HashSet::new();
    let mut facts: HashMap<TypeId, TypeFacts> = HashMap::new();
    // One root visit, then one visit per child edge and another per record edge.
    // Keep the closure total separate from postorder value-size multiplication.
    let mut codec_work = 1usize;
    let mut allocator_role = None;
    while let Some((ty, finish)) = pending.pop() {
        budget.reserve(0, 1)?;
        if facts.contains_key(&ty) {
            continue;
        }
        if finish {
            let mut decoded = DecodedShape {
                cells: 1,
                height: 0,
            };
            let mut include = |child: TypeId, copies: usize, union: bool| -> Result<(), Error> {
                let child = facts
                    .get(&child)
                    .ok_or(Error::InvalidIr("layout preflight lost a shape dependency"))?
                    .decoded;
                if copies != 0 {
                    let cells = child.cells.saturating_mul(copies);
                    decoded.cells = if union {
                        decoded.cells.max(cells.saturating_add(1))
                    } else {
                        decoded.cells.saturating_add(cells)
                    };
                    decoded.height = decoded.height.max(child.height.saturating_add(1));
                }
                Ok(())
            };
            let count = match types.kind(ty)? {
                kind if kind.record_storage_id().is_some() => {
                    let record = types.record_storage_definition(ty)?;
                    budget.reserve(0, record.fields.len())?;
                    for field in &record.fields {
                        include(*field, 1, record.kind == jai_types::RecordKind::Union)?;
                    }
                    record.fields.len()
                }
                TypeKind::String | TypeKind::Slice(_) => 2,
                TypeKind::DynamicArray(_) => {
                    if let Some(schema) = types.allocator_schema() {
                        include(schema.ty(), 1, false)?;
                    }
                    4
                }
                TypeKind::Distinct(id) => {
                    let repr = types.distinct(*id)?.representation;
                    include(repr, 1, false)?;
                    facts
                        .get(&repr)
                        .ok_or(Error::InvalidIr("layout preflight lost a dependency"))?
                        .offsets
                }
                TypeKind::FixedArray { element, count } => {
                    include(
                        *element,
                        usize::try_from(*count).unwrap_or(usize::MAX),
                        false,
                    )?;
                    0
                }
                _ => 0,
            };
            // Both the engine's dependency layout and a distinct representation clone
            // retain offsets; root cloning is charged separately below.
            budget.reserve(count, count.saturating_mul(3).saturating_add(1))?;
            facts.insert(
                ty,
                TypeFacts {
                    offsets: count,
                    decoded,
                    codec_work: 0,
                    allocator_role: None,
                },
            );
            visiting.remove(&ty);
            continue;
        }
        if !visiting.insert(ty) {
            return Err(jai_types::LayoutError::RecursiveValue(ty).into());
        }
        // Bound visited IDs, postorder bookkeeping and the engine's own cache.
        budget.reserve(4, 4)?;
        pending.push((ty, true));
        match types.kind(ty)? {
            kind if kind.record_storage_id().is_some() => {
                let record = types.record_storage_definition(ty)?;
                let fields = &record.fields;
                // Explicit alignment lists can be malformed and wider than fields.
                // LayoutEngine validates every supplied entry, so bound that work too.
                budget.reserve(0, record.layout.field_alignments.len().saturating_mul(2))?;
                // Placement validation borrows FieldIds and creates a temporary
                // checked-ordinal list; neither becomes additional cached offsets.
                budget.reserve(
                    record.layout.field_placements.len(),
                    record.layout.field_placements.len().saturating_mul(2),
                )?;
                budget.reserve(fields.len(), fields.len().saturating_mul(4))?;
                codec_work = fields
                    .len()
                    .checked_mul(2)
                    .and_then(|edges| codec_work.checked_add(edges))
                    .ok_or(Error::Limit(LimitKind::Fuel))?;
                pending.extend(fields.iter().rev().map(|field| (*field, false)));
            }
            TypeKind::FixedArray { element, .. } => {
                budget.reserve(1, 4)?;
                codec_work = codec_work
                    .checked_add(1)
                    .ok_or(Error::Limit(LimitKind::Fuel))?;
                pending.push((*element, false));
            }
            TypeKind::Distinct(id) => {
                let representation = types.distinct(*id)?.representation;
                budget.reserve(1, 4)?;
                codec_work = codec_work
                    .checked_add(1)
                    .ok_or(Error::Limit(LimitKind::Fuel))?;
                pending.push((representation, false));
            }
            TypeKind::DynamicArray(_) => {
                allocator_role = Some(types.allocator_schema());
                if let Some(schema) = types.allocator_schema() {
                    budget.reserve(1, 4)?;
                    codec_work = codec_work
                        .checked_add(1)
                        .ok_or(Error::Limit(LimitKind::Fuel))?;
                    pending.push((schema.ty(), false));
                }
            }
            // Descriptor and pointer layouts never demand pointee definitions.
            _ => {}
        }
    }
    let mut root = *facts
        .get(&root)
        .ok_or(Error::InvalidIr("layout preflight did not finish its root"))?;
    budget.reserve(root.offsets, root.offsets.saturating_add(1))?;
    root.codec_work = codec_work
        .checked_mul(4)
        .ok_or(Error::Limit(LimitKind::Fuel))?;
    root.allocator_role = allocator_role;
    Ok(root)
}

impl super::Memory {
    /// Static codec closure work from charged preparation, even for empty arrays.
    /// A fresh codec LayoutEngine still scans this closure; lookup does not.
    pub(crate) fn prepared_codec_layout_work(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
    ) -> Result<usize, Error> {
        self.layout_cache.borrow().prepared_codec_work(types, ty)
    }
    /// Cache-only decoded node bound, admitted by the same charged layout preparation.
    pub(crate) fn prepared_decoded_cells(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
    ) -> Result<usize, Error> {
        let shape = self.layout_cache.borrow().prepared_decoded(types, ty)?;
        if shape.cells > self.limits.value_cells {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        if shape.height > self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        Ok(shape.cells)
    }
    /// Standalone Memory uses its bounded layout fallback; VM callers prepare first.
    pub(super) fn decoded_cells(&self, types: &dyn TypeView, ty: TypeId) -> Result<usize, Error> {
        self.layout(types, ty)?;
        self.prepared_decoded_cells(types, ty)
    }
    /// Cache-only access after explicit VM preparation; this never computes a layout.
    pub(crate) fn prepared_layout(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
    ) -> Result<Arc<Layout>, Error> {
        self.layout_cache.borrow().prepared(types, ty)
    }
    fn cache_cell_limit(&self) -> Result<usize, Error> {
        let retained = self.layout_cache.borrow().cells();
        let objects = self
            .cells
            .get()
            .checked_sub(retained)
            .ok_or(Error::InvalidIr(
                "layout cache charge exceeds live memory cells",
            ))?;
        self.limits
            .value_cells
            .checked_sub(objects)
            .ok_or(Error::Limit(LimitKind::ValueCells))
    }
    /// Prepare one demanded layout under the VM's actual remaining work budget.
    /// The caller charges returned work before propagating the result, including Pending.
    pub(crate) fn prepare_layout(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
        available_work: usize,
    ) -> (usize, Result<(), Error>) {
        let limit = match self.cache_cell_limit() {
            Ok(limit) => limit,
            Err(error) => return (0, Err(error)),
        };
        let previous = self.layout_cache.borrow().cells();
        let (work, result) =
            self.layout_cache
                .borrow_mut()
                .layout_attempt(types, ty, limit, available_work);
        if result.is_ok() {
            let added = self.layout_cache.borrow().cells() - previous;
            self.cells.set(self.cells.get() + added);
        }
        (work, result.map(|_| ()))
    }
    /// Address-only preparation does not demand an incomplete pointee's layout.
    pub(crate) fn prepare_pointer_layouts(
        &self,
        types: &dyn TypeView,
        pointer: &super::Pointer,
        include_pointee: bool,
        available_work: usize,
    ) -> (usize, Result<(), Error>) {
        let mut work = 0usize;
        let result = (|| {
            if let Some(code) = pointer.code_pointer() {
                work = usize::from(available_work != 0);
                if available_work == 0 {
                    return Err(Error::Limit(LimitKind::Fuel));
                }
                self.validate_code_pointer(types, code)?;
                return if include_pointee {
                    Err(Error::UnsupportedPointerOperation(
                        "code address has no data storage",
                    ))
                } else {
                    Ok(())
                };
            }
            if pointer.is_opaque() {
                if available_work == 0 {
                    return Err(Error::Limit(LimitKind::Fuel));
                }
                work = 1;
                self.validate_opaque_address(types, pointer)?;
                return if include_pointee {
                    Err(Error::UnsupportedPointerOperation(
                        "numeric address has no allocation provenance",
                    ))
                } else {
                    Ok(())
                };
            }
            if pointer.is_null() {
                return if include_pointee {
                    Err(Error::NullPointer)
                } else {
                    Ok(())
                };
            }
            let mut ty = self.allocation(pointer)?.ty;
            let mut prepare = |ty| -> Result<(), Error> {
                let (cost, result) = self.prepare_layout(types, ty, available_work - work);
                work += cost;
                result
            };
            for projection in &pointer.data()?.path {
                match projection {
                    super::Projection::Bytes { ty: view, .. } => ty = *view,
                    super::Projection::Field(index) => {
                        prepare(ty)?;
                        ty = *types
                            .record_storage_definition(ty)?
                            .fields
                            .get(*index)
                            .ok_or(Error::InvalidIr("invalid prepared layout field"))?;
                    }
                    super::Projection::Index(_) => match types.kind(ty)? {
                        TypeKind::FixedArray { element, .. } => {
                            prepare(ty)?;
                            ty = *element;
                        }
                        TypeKind::String => {
                            ty = types.scalar(ScalarType::Int(jai_types::IntegerType::U8))
                        }
                        _ => return Err(Error::InvalidIr("invalid prepared layout index")),
                    },
                    super::Projection::Sequence(field) => {
                        prepare(ty)?;
                        ty = super::sequence_field_type(types, ty, *field)?;
                    }
                }
            }
            if include_pointee {
                prepare(pointer.pointee)?;
            }
            Ok(())
        })();
        (work, result)
    }
    /// A field projection needs both its parent offsets and the selected field extent.
    pub(crate) fn prepare_field_layouts(
        &self,
        types: &dyn TypeView,
        pointer: &super::Pointer,
        field: usize,
        available_work: usize,
    ) -> (usize, Result<(), Error>) {
        if pointer.is_null() {
            return (0, Err(Error::NullPointer));
        }
        let (mut work, path) = self.prepare_pointer_layouts(types, pointer, false, available_work);
        let result = (|| {
            path?;
            let (cost, result) = self.prepare_layout(types, pointer.pointee, available_work - work);
            work += cost;
            result?;
            let fields = &types.record_storage_definition(pointer.pointee)?.fields;
            let child = *fields.get(field).ok_or(Error::OutOfBounds {
                index: field,
                length: fields.len(),
            })?;
            let (cost, result) = self.prepare_layout(types, child, available_work - work);
            work += cost;
            result
        })();
        (work, result)
    }
    pub(super) fn layout(&self, types: &dyn TypeView, ty: TypeId) -> Result<Arc<Layout>, Error> {
        self.layout_reserved(types, ty, 0)
    }
    pub(super) fn layout_reserved(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
        reservation: usize,
    ) -> Result<Arc<Layout>, Error> {
        let limit = self
            .cache_cell_limit()?
            .checked_sub(reservation)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let previous = self.layout_cache.borrow().cells();
        let result = self.layout_cache.borrow_mut().layout(
            types,
            ty,
            limit,
            self.limits.value_cells.saturating_mul(16),
        );
        if result.is_ok() {
            let added = self.layout_cache.borrow().cells() - previous;
            self.cells.set(self.cells.get() + added);
        }
        result
    }
}

#[cfg(test)]
mod tests;
