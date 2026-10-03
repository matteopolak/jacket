use crate::{ByteImage, ByteTarget, Endian, Error, LimitKind, Limits, Value};
mod addresses;
mod code_images;
mod code_pointers;
mod opaque_addresses;
pub(crate) use code_pointers::CodePointer;
mod branch_quota;
mod copy_work;
mod fork;
mod host_files;
mod host_heap;
mod layout_cache;
#[cfg(test)]
mod metadata_tests;
pub(crate) mod pools;
mod runtime_intrinsics;
mod runtime_types;
#[cfg(test)]
mod sequence_allocators;
mod sequence_buffers;
mod simd;
mod static_byte_views;
mod storage_reinterpretation;
mod stored_aggregates;
#[cfg(test)]
mod token_retention_tests;
use jai_types::{CastMode, Integer, IntegerType, ScalarType, TypeId, TypeKind, TypeView};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_MEMORY: AtomicU64 = AtomicU64::new(1);
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Projection {
    Bytes { offset: u64, ty: TypeId },
    Sequence(jai_ir::SequenceField),
    Field(usize),
    Index(usize),
}
/// Allocation identity, type and projection path are never raw host addresses.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Pointer {
    origin: PointerOrigin,
    pointee: TypeId,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum PointerOrigin {
    Null,
    Data(DataPointer),
    Code(CodePointer),
    Opaque(opaque_addresses::OpaqueAddress),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DataPointer {
    memory: u64,
    allocation: u64,
    path: Vec<Projection>,
    region: Option<(u64, u64)>,
    restricted_region: bool,
}
impl Pointer {
    pub fn null(pointee: TypeId) -> Self {
        Self {
            origin: PointerOrigin::Null,
            pointee,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self.origin, PointerOrigin::Null)
    }
    pub(crate) fn memory_identity(&self) -> u64 {
        match &self.origin {
            PointerOrigin::Data(data) => data.memory,
            PointerOrigin::Code(code) => code.memory_identity(),
            _ => 0,
        }
    }
    fn allocation_id(&self) -> u64 {
        match &self.origin {
            PointerOrigin::Data(data) => data.allocation,
            _ => 0,
        }
    }
    #[cfg(test)]
    pub(crate) fn allocation_key(&self) -> (u64, u64) {
        self.data_allocation_key()
            .expect("pointer has no data allocation")
    }
    pub(crate) fn data_allocation_key(&self) -> Option<(u64, u64)> {
        match &self.origin {
            PointerOrigin::Data(data) => Some((data.memory, data.allocation)),
            _ => None,
        }
    }
    pub(crate) fn code_pointer(&self) -> Option<CodePointer> {
        match self.origin {
            PointerOrigin::Code(code) => Some(code),
            _ => None,
        }
    }
    pub(crate) fn from_code(code: CodePointer, pointee: TypeId) -> Self {
        Self {
            origin: PointerOrigin::Code(code),
            pointee,
        }
    }
    fn data(&self) -> Result<&DataPointer, Error> {
        match &self.origin {
            PointerOrigin::Data(data) => Ok(data),
            PointerOrigin::Null => Err(Error::NullPointer),
            PointerOrigin::Code(_) => Err(Error::UnsupportedPointerOperation(
                "code address has no data storage",
            )),
            PointerOrigin::Opaque(_) => Err(Error::UnsupportedPointerOperation(
                "numeric address has no allocation provenance",
            )),
        }
    }
    fn data_mut(&mut self) -> Result<&mut DataPointer, Error> {
        match &mut self.origin {
            PointerOrigin::Data(data) => Ok(data),
            PointerOrigin::Null => Err(Error::NullPointer),
            PointerOrigin::Code(_) => Err(Error::UnsupportedPointerOperation(
                "code address has no data storage",
            )),
            PointerOrigin::Opaque(_) => Err(Error::UnsupportedPointerOperation(
                "numeric address has no allocation provenance",
            )),
        }
    }
    /// Only actual data pointers retain dynamic projections.
    pub(crate) fn metadata_cells(&self) -> usize {
        match &self.origin {
            PointerOrigin::Data(data) => data.path.capacity(),
            _ => 0,
        }
    }
    pub fn pointee(&self) -> TypeId {
        self.pointee
    }
    pub(crate) fn retype(&self, pointee: TypeId) -> Self {
        let mut pointer = self.clone();
        pointer.pointee = pointee;
        pointer
    }
}
#[derive(Clone)]
struct Allocation {
    ty: TypeId,
    virtual_base: u64,
    virtual_extent: u64,
    virtual_alignment: u32,
    value: Option<Value>,
    has_stored_aggregate: bool,
    cells: Cell<usize>,
    readonly: bool,
    image: RefCell<Option<ByteImage>>,
}
pub(crate) struct MemorySnapshot {
    allocations: HashMap<u64, Allocation>,
    cells: usize,
    handle_tokens: HashMap<HandleKey, u64>,
    virtual_regions: BTreeMap<u64, (u64, u64)>,
    runtime_types: HashMap<(u64, u64), jai_ir::RuntimeTypeIdentity>,
    pool_ledger: pools::PoolLedger,
    layout_cache: layout_cache::RootLayoutCache,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum HandleKey {
    Procedure {
        signature: TypeId,
        procedure: jai_ir::ProcedureId,
    },
}
struct HandleTokens {
    values: HashMap<HandleKey, u64>,
}
pub struct Memory {
    identity: u64,
    next_allocation: u64,
    next_virtual_address: Cell<u64>,
    virtual_regions: BTreeMap<u64, (u64, u64)>,
    allocations: HashMap<u64, Allocation>,
    cells: Cell<usize>,
    limits: Limits,
    target: ByteTarget,
    handle_tokens: RefCell<HandleTokens>,
    runtime_types: HashMap<(u64, u64), jai_ir::RuntimeTypeIdentity>,
    pool_ledger: pools::PoolLedger,
    layout_cache: RefCell<layout_cache::RootLayoutCache>,
}
impl Memory {
    pub fn new(limits: Limits) -> Self {
        Self::with_layout(limits, jai_types::LayoutPolicy::lp64())
    }
    /// Select language target storage explicitly, independently of the Rust host.
    pub fn with_layout(limits: Limits, layout: jai_types::LayoutPolicy) -> Self {
        Self::with_target(
            limits,
            ByteTarget {
                policy: layout,
                endian: Endian::Little,
            },
        )
    }
    pub fn with_target(limits: Limits, target: ByteTarget) -> Self {
        Self {
            identity: NEXT_MEMORY
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                    value.checked_add(1)
                })
                .expect("virtual memory identity space exhausted"),
            next_allocation: 1,
            next_virtual_address: Cell::new(u64::from(target.policy.pointer().alignment)),
            virtual_regions: BTreeMap::new(),
            allocations: HashMap::new(),
            cells: Cell::new(0),
            limits,
            target,
            handle_tokens: RefCell::new(HandleTokens {
                values: HashMap::new(),
            }),
            runtime_types: HashMap::new(),
            pool_ledger: pools::PoolLedger::default(),
            layout_cache: RefCell::new(layout_cache::RootLayoutCache::new(target.policy)),
        }
    }
    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }
    pub fn target(&self) -> ByteTarget {
        self.target
    }
    pub fn value_cells(&self) -> usize {
        self.cells.get()
    }
    pub(crate) fn value_cell_limit(&self) -> usize {
        self.limits.value_cells
    }
    pub(crate) fn storage_alignment(&self, pointer: &Pointer) -> Result<u32, Error> {
        Ok(self.allocation(pointer)?.virtual_alignment)
    }
    pub(crate) fn snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            allocations: self.allocations.clone(),
            cells: self.cells.get(),
            handle_tokens: self.handle_tokens.borrow().values.clone(),
            virtual_regions: self.virtual_regions.clone(),
            runtime_types: self.runtime_types.clone(),
            pool_ledger: self.pool_ledger.clone(),
            layout_cache: self.layout_cache.borrow().clone(),
        }
    }
    pub(crate) fn restore(&mut self, snapshot: MemorySnapshot) {
        // Never recycle identities: a pointer from a rolled-back host effect stays dangling.
        self.allocations = snapshot.allocations;
        self.cells.set(snapshot.cells);
        self.virtual_regions = snapshot.virtual_regions;
        self.runtime_types = snapshot.runtime_types;
        self.pool_ledger = snapshot.pool_ledger;
        *self.layout_cache.borrow_mut() = snapshot.layout_cache;
        self.handle_tokens.borrow_mut().values = snapshot.handle_tokens;
    }
    pub fn allocate(
        &mut self,
        types: &dyn TypeView,
        ty: TypeId,
        value: Option<Value>,
    ) -> Result<Pointer, Error> {
        self.allocate_with_alignment(types, ty, value, 1)
    }
    pub fn allocate_with_alignment(
        &mut self,
        types: &dyn TypeView,
        ty: TypeId,
        value: Option<Value>,
        requested: u32,
    ) -> Result<Pointer, Error> {
        if !requested.is_power_of_two() {
            return Err(Error::InvalidIr(
                "storage alignment must be a nonzero power of two",
            ));
        }
        match types.kind(ty)? {
            TypeKind::Void | TypeKind::Code => {
                return Err(Error::UnsupportedType(ty));
            }
            kind if kind.record_storage_id().is_some() => {
                types.record_storage_definition(ty)?;
            }
            TypeKind::Enum(id) => {
                types.enumeration(*id)?;
            }
            _ => {}
        }
        if self.allocations.len() >= self.limits.allocations {
            return Err(Error::Limit(LimitKind::Allocations));
        }
        if let Some(value) = &value {
            value.cells(self.limits.value_cells)?;
            value.validate(types, ty, self.limits.evaluation_depth)?;
            self.validate_runtime_type_values(types, value)?;
        }
        let (cells, has_stored_aggregate) = value.as_ref().map_or(Ok((1, false)), |value| {
            value.cells_and_storage(self.limits.value_cells)
        })?;
        let layout = self.layout_reserved(types, ty, cells)?;
        let total = self
            .cells
            .get()
            .checked_add(cells)
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let extent = match &value {
            Some(Value::String(bytes)) => {
                u64::try_from(bytes.len()).map_err(|_| Error::CheckedCast)?
            }
            _ => layout.size,
        };
        let virtual_alignment = requested.max(layout.alignment);
        let virtual_base = self.reserve_virtual_region(extent, u64::from(virtual_alignment))?;
        let allocation = self.next_allocation;
        self.next_allocation = allocation
            .checked_add(1)
            .ok_or(Error::Limit(LimitKind::Allocations))?;
        self.allocations.insert(
            allocation,
            Allocation {
                ty,
                virtual_base,
                virtual_extent: extent,
                virtual_alignment,
                value,
                has_stored_aggregate,
                cells: Cell::new(cells),
                readonly: false,
                image: RefCell::new(None),
            },
        );
        self.cells.set(total);
        self.virtual_regions
            .insert(virtual_base, (allocation, extent));
        Ok(Pointer {
            origin: PointerOrigin::Data(DataPointer {
                memory: self.identity,
                allocation,
                path: vec![],
                region: None,
                restricted_region: false,
            }),
            pointee: ty,
        })
    }
    /// Mutable, aligned caller-owned byte storage for runtime-sized sequence packs.
    pub fn allocate_sequence_buffer(
        &mut self,
        types: &dyn TypeView,
        element: TypeId,
        count: usize,
    ) -> Result<Pointer, Error> {
        let layout = self.layout(types, element)?;
        let length = layout
            .size
            .checked_mul(u64::try_from(count).map_err(|_| Error::CheckedCast)?)
            .and_then(|length| usize::try_from(length).ok())
            .filter(|length| *length < self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let alignment = layout.alignment.max(16);
        let length = if count != 0 {
            length.max(1)
        } else {
            length
        };
        let string = types
            .lookup(&TypeKind::String)
            .ok_or(Error::InvalidIr("string byte storage type is unavailable"))?;
        self.allocate_with_alignment(
            types,
            string,
            Some(Value::String(vec![0; length])),
            alignment,
        )
    }
    fn allocation(&self, pointer: &Pointer) -> Result<&Allocation, Error> {
        if pointer.is_opaque() {
            return Err(Error::UnsupportedPointerOperation(
                "numeric address has no allocation provenance",
            ));
        }
        if pointer.code_pointer().is_some() {
            return Err(Error::UnsupportedPointerOperation(
                "code address has no data storage",
            ));
        }
        if pointer.is_null() {
            return Err(Error::NullPointer);
        }
        if pointer.memory_identity() != self.identity {
            return Err(Error::ForeignPointer);
        }
        self.allocations
            .get(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)
    }
    pub fn cast_pointer(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        pointee: TypeId,
        mode: CastMode,
    ) -> Result<Pointer, Error> {
        if matches!(mode, CastMode::Force(_)) {
            return Err(Error::InvalidIr("force requires a checked storage bitcast"));
        }
        types.kind(pointee)?;
        if pointer.is_opaque() {
            self.validate_opaque_address(types, pointer)?;
            return Ok(pointer.retype(pointee));
        }
        if let Some(code) = pointer.code_pointer() {
            self.validate_code_pointer(types, code)?;
            return Ok(pointer.retype(pointee));
        }
        if !pointer.is_null() {
            self.allocation(pointer)?;
        }
        let mut result = pointer.clone();
        if !pointer.is_null()
            && !pointer.data()?.restricted_region
            && self.allocation(pointer)?.ty == pointee
            && self.byte_offset(types, pointer)? == 0
            && (pointer.data()?.region.is_none()
                || pointer.data()?.region == Some((0, self.allocation(pointer)?.virtual_extent))
                || types.kind(pointee)?.record_storage_id().is_some())
        {
            // A leading base/header field may be downcast to its proven owner.
            // The allocation identity and exact nominal type establish this span.
            result.data_mut()?.region = None;
        }
        if !pointer.is_null() && self.path_type(types, pointer)? != pointee {
            result.data_mut()?.path = vec![Projection::Bytes {
                offset: self.byte_offset(types, pointer)?,
                ty: pointee,
            }];
        }
        result.pointee = pointee;
        Ok(result)
    }
    pub fn same_address(
        &self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
    ) -> Result<bool, Error> {
        if left.is_opaque() || right.is_opaque() {
            return Ok(self.comparable_pointer_bits(types, left)?
                == self.comparable_pointer_bits(types, right)?);
        }
        if left.code_pointer().is_some() || right.code_pointer().is_some() {
            if let Some(code) = left.code_pointer() {
                self.validate_code_pointer(types, code)?;
            } else if !left.is_null() {
                self.allocation(left)?;
            }
            if let Some(code) = right.code_pointer() {
                self.validate_code_pointer(types, code)?;
            } else if !right.is_null() {
                self.allocation(right)?;
            }
            return Ok(left.code_pointer().is_some() && left.code_pointer() == right.code_pointer());
        }
        if left.is_null() || right.is_null() {
            return Ok(left.is_null() && right.is_null());
        }
        self.allocation(left)?;
        self.allocation(right)?;
        Ok(left.memory_identity() == right.memory_identity()
            && left.allocation_id() == right.allocation_id()
            && self.byte_offset(types, left)? == self.byte_offset(types, right)?)
    }
    fn byte_offset(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<u64, Error> {
        let mut ty = self.allocation(pointer)?.ty;
        let mut offset = 0u64;
        for projection in &pointer.data()?.path {
            let relative = match (projection, types.kind(ty)?) {
                (
                    Projection::Bytes {
                        offset: bytes,
                        ty: view,
                    },
                    _,
                ) => {
                    if offset != 0 {
                        return Err(Error::InvalidIr(
                            "byte address must begin its projection path",
                        ));
                    }
                    ty = *view;
                    *bytes
                }
                (Projection::Field(index), kind) if kind.record_storage_id().is_some() => {
                    let layout = self.layout(types, ty)?;
                    let relative = *layout
                        .field_offsets
                        .get(*index)
                        .ok_or(Error::InvalidIr("invalid address field projection"))?;
                    ty = *types
                        .record_storage_definition(ty)?
                        .fields
                        .get(*index)
                        .ok_or(Error::InvalidIr("invalid address field type"))?;
                    relative
                }
                (
                    Projection::Index(index),
                    TypeKind::FixedArray {
                        element,
                        count,
                    },
                ) => {
                    if (*index as u128) > u128::from(*count) {
                        return Err(Error::OutOfBounds {
                            index: *index,
                            length: usize::try_from(*count).unwrap_or(usize::MAX),
                        });
                    }
                    let stride = self
                        .layout(types, ty)?
                        .array_stride
                        .ok_or(Error::InvalidIr("array layout has no stride"))?;
                    ty = *element;
                    stride
                        .checked_mul(u64::try_from(*index).map_err(|_| Error::CheckedCast)?)
                        .ok_or(Error::CheckedCast)?
                }
                (Projection::Index(index), TypeKind::String) => {
                    // Owned byte buffers are VM backing objects, not ABI descriptors.
                    ty = types.scalar(ScalarType::Int(IntegerType::U8));
                    u64::try_from(*index).map_err(|_| Error::CheckedCast)?
                }
                (Projection::Sequence(field), _) => {
                    let ordinal = match field {
                        jai_ir::SequenceField::Count => 0,
                        jai_ir::SequenceField::Data => 1,
                        jai_ir::SequenceField::Allocated => 2,
                    };
                    let relative = *self
                        .layout(types, ty)?
                        .field_offsets
                        .get(ordinal)
                        .ok_or(Error::UnsupportedType(ty))?;
                    ty = sequence_field_type(types, ty, *field)?;
                    relative
                }
                _ => return Err(Error::InvalidIr("invalid address projection")),
            };
            offset = offset.checked_add(relative).ok_or(Error::CheckedCast)?;
        }
        Ok(offset)
    }
    /// Difference in target elements, requiring one live allocation and a common view type.
    pub fn distance(
        &self,
        types: &dyn TypeView,
        left: &Pointer,
        right: &Pointer,
    ) -> Result<i64, Error> {
        self.validate_pointer(types, left)?;
        self.validate_pointer(types, right)?;
        if left.memory_identity() != right.memory_identity()
            || left.allocation_id() != right.allocation_id()
        {
            return Err(Error::InvalidIr(
                "pointer difference requires one allocation",
            ));
        }
        if left.pointee != right.pointee {
            return Err(Error::TypeMismatch {
                expected: left.pointee,
            });
        }
        let stride = self.layout(types, left.pointee)?.size;
        if stride == 0 {
            return Err(Error::InvalidIr(
                "pointer difference requires nonzero element size",
            ));
        }
        let difference = i128::from(self.byte_offset(types, left)?)
            - i128::from(self.byte_offset(types, right)?);
        if difference % i128::from(stride) != 0 {
            return Err(Error::InvalidIr(
                "pointer difference is not an integral element distance",
            ));
        }
        i64::try_from(difference / i128::from(stride)).map_err(|_| Error::CheckedCast)
    }
    pub fn freeze(&mut self, pointer: &Pointer) -> Result<(), Error> {
        self.allocation(pointer)?;
        self.allocations
            .get_mut(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?
            .readonly = true;
        Ok(())
    }
    pub fn sequence_field(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        field: jai_ir::SequenceField,
    ) -> Result<Pointer, Error> {
        let pointer = self.cast_pointer(types, pointer, pointer.pointee, CastMode::Checked)?;
        self.validate_pointer(types, &pointer)?;
        let ty = sequence_field_type(types, pointer.pointee, field)?;
        let mut result = pointer.clone();
        if result.data()?.path.len() >= self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        result.data_mut()?.path.reserve_exact(1);
        result.data_mut()?.path.push(Projection::Sequence(field));
        result.pointee = ty;
        let start = self.byte_offset(types, &result)?;
        let size = self.layout(types, ty)?.size;
        let parent = self.region(types, &pointer)?;
        let end = start.checked_add(size).ok_or(Error::CheckedCast)?;
        if start < parent.0 || end > parent.1 {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(parent.1 - parent.0).unwrap_or(usize::MAX),
            });
        }
        result.data_mut()?.region = Some((start, end));
        Ok(result)
    }
    pub fn release(&mut self, pointer: &Pointer) -> Result<(), Error> {
        if self.allocation(pointer)?.readonly {
            return Err(Error::ReadOnlyStorage);
        }
        if !pointer.data()?.path.is_empty() {
            return Err(Error::InvalidIr("only an allocation root can be released"));
        }
        if self.pool_ledger.owns_descriptor(pointer.allocation_id()) {
            return Err(Error::InvalidIr(
                "finish live pools before releasing their descriptor allocation",
            ));
        }
        let allocation = self
            .allocations
            .remove(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?;
        self.cells.set(self.cells.get() - allocation.cells.get());
        self.virtual_regions.remove(&allocation.virtual_base);
        self.runtime_types
            .retain(|(id, _), _| *id != pointer.allocation_id());
        Ok(())
    }
    pub fn field(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        field: usize,
    ) -> Result<Pointer, Error> {
        let pointer = self.cast_pointer(types, pointer, pointer.pointee, CastMode::Checked)?;
        self.allocation(&pointer)?;
        if types.kind(pointer.pointee)?.record_storage_id().is_none() {
            return Err(Error::UnsupportedType(pointer.pointee));
        }
        let record = types.record_storage_definition(pointer.pointee)?;
        let ty = *record.fields.get(field).ok_or(Error::OutOfBounds {
            index: field,
            length: record.fields.len(),
        })?;
        let mut result = pointer.clone();
        if result.data()?.path.len() >= self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        result.data_mut()?.path.reserve_exact(1);
        result.data_mut()?.path.push(Projection::Field(field));
        result.pointee = ty;
        let start = self.byte_offset(types, &result)?;
        let size = self.layout(types, ty)?.size;
        let parent_region = self.region(types, &pointer)?;
        let end = start.checked_add(size).ok_or(Error::CheckedCast)?;
        if start < parent_region.0 || end > parent_region.1 {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(parent_region.1 - parent_region.0).unwrap_or(usize::MAX),
            });
        }
        result.data_mut()?.region = Some(if record.kind == jai_types::RecordKind::Union {
            parent_region
        } else {
            (start, end)
        });
        Ok(result)
    }
    pub fn index(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        index: usize,
    ) -> Result<Pointer, Error> {
        let pointer = self.cast_pointer(types, pointer, pointer.pointee, CastMode::Checked)?;
        self.allocation(&pointer)?;
        self.validate_pointer(types, &pointer)?;
        let (element, length) = match types.kind(pointer.pointee)? {
            TypeKind::FixedArray {
                element,
                count,
            } => (
                *element,
                usize::try_from(*count).map_err(|_| Error::Limit(LimitKind::ValueCells))?,
            ),
            TypeKind::String => {
                let Value::String(bytes) = self.load(types, &pointer)? else {
                    return Err(Error::InvalidIr("string storage has incorrect value"));
                };
                (types.scalar(ScalarType::Int(IntegerType::U8)), bytes.len())
            }
            _ => return Err(Error::UnsupportedType(pointer.pointee)),
        };
        if index >= length {
            return Err(Error::OutOfBounds {
                index,
                length,
            });
        }
        let mut result = pointer.clone();
        if result.data()?.path.len() >= self.limits.evaluation_depth.min(256) {
            return Err(Error::Limit(LimitKind::EvaluationDepth));
        }
        result.data_mut()?.path.reserve_exact(1);
        result.data_mut()?.path.push(Projection::Index(index));
        result.pointee = element;
        let start = self.byte_offset(types, &pointer)?;
        let size = match types.kind(pointer.pointee)? {
            TypeKind::String => length as u64,
            _ => self.layout(types, pointer.pointee)?.size,
        };
        let end = start.checked_add(size).ok_or(Error::CheckedCast)?;
        let parent_region = self.region(types, &pointer)?;
        if start < parent_region.0 || end > parent_region.1 {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(parent_region.1 - parent_region.0).unwrap_or(usize::MAX),
            });
        }
        result.data_mut()?.region = Some((start, end));
        Ok(result)
    }
    /// Offset within the same array only; one-past pointers cannot be dereferenced.
    pub fn offset(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        offset: isize,
    ) -> Result<Pointer, Error> {
        if let Some(code) = pointer.code_pointer() {
            self.validate_code_pointer(types, code)?;
            return if offset == 0 {
                Ok(pointer.clone())
            } else {
                Err(Error::UnsupportedPointerOperation(
                    "code address arithmetic is unsupported",
                ))
            };
        }
        self.validate_pointer(types, pointer)?;
        if offset == 0 {
            return Ok(pointer.clone());
        }
        let stride = self.layout(types, pointer.pointee)?.size;
        if stride == 0 {
            return Err(Error::InvalidIr(
                "pointer arithmetic requires nonzero element size",
            ));
        }
        let next =
            i128::from(self.byte_offset(types, pointer)?) + (offset as i128) * i128::from(stride);
        let (start, length) = self.region(types, pointer)?;
        let next = u64::try_from(next)
            .ok()
            .filter(|next| *next >= start && *next <= length)
            .ok_or(Error::OutOfBounds {
                index: usize::try_from(next).unwrap_or(usize::MAX),
                length: usize::try_from(length - start).unwrap_or(usize::MAX),
            })?;
        let mut result = pointer.clone();
        result.data_mut()?.path = vec![Projection::Bytes {
            offset: next,
            ty: pointer.pointee,
        }];
        result.pointee = pointer.pointee;
        Ok(result)
    }
    fn storage_length(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<u64, Error> {
        let allocation = self.allocation(pointer)?;
        match &allocation.value {
            Some(Value::String(bytes)) => Ok(bytes.len() as u64),
            _ => Ok(self.layout(types, allocation.ty)?.size),
        }
    }
    fn region(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<(u64, u64), Error> {
        pointer.data()?.region.map_or_else(
            || self.storage_length(types, pointer).map(|end| (0, end)),
            Ok,
        )
    }
    fn validate_access(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<(), Error> {
        let offset = self.byte_offset(types, pointer)?;
        let (start, length) = self.region(types, pointer)?;
        let size = if pointer.data()?.path.is_empty()
            && matches!(self.allocation(pointer)?.value, Some(Value::String(_)))
        {
            length
        } else {
            self.layout(types, pointer.pointee)?.size
        };
        let end = offset.checked_add(size).ok_or(Error::CheckedCast)?;
        if offset < start || end > length {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(length - start).unwrap_or(usize::MAX),
            });
        }
        Ok(())
    }
    pub fn sequence_data(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<Pointer, Error> {
        let (element, count) = match types.kind(pointer.pointee)? {
            TypeKind::FixedArray {
                element,
                count,
            } => (*element, *count),
            TypeKind::String => {
                let Value::String(bytes) = self.load(types, pointer)? else {
                    return Err(Error::InvalidIr("string backing storage mismatch"));
                };
                (
                    types.scalar(ScalarType::Int(IntegerType::U8)),
                    bytes.len() as u64,
                )
            }
            _ => return Err(Error::UnsupportedType(pointer.pointee)),
        };
        if count == 0 {
            Ok(Pointer::null(element))
        } else {
            self.index(types, pointer, 0)
        }
    }
    pub fn validate_slice(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
        count: usize,
    ) -> Result<(), Error> {
        if pointer.is_null() {
            return if count == 0 {
                Ok(())
            } else {
                Err(Error::NullPointer)
            };
        }
        self.validate_pointer(types, pointer)?;
        if count == 0 {
            return Ok(());
        }
        // A zero-sized element has no byte extent, but a nonempty view still
        // requires live nonnull storage. Its logical count is checked by users.
        let element_size = self.layout(types, pointer.pointee)?.size;
        if element_size == 0 {
            return Ok(());
        }
        let last = isize::try_from(count - 1).map_err(|_| Error::Limit(LimitKind::ValueCells))?;
        let final_element = self.offset(types, pointer, last)?;
        // Legal one-past addresses are not storage for the last element of a view.
        // Extent validation does not require the element to be initialized.
        let end = self
            .byte_offset(types, &final_element)?
            .checked_add(element_size)
            .ok_or(Error::CheckedCast)?;
        let (_, length) = self.region(types, pointer)?;
        if end > length {
            return Err(Error::OutOfBounds {
                index: usize::try_from(end).unwrap_or(usize::MAX),
                length: usize::try_from(length).unwrap_or(usize::MAX),
            });
        }
        Ok(())
    }
    fn path_type(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<TypeId, Error> {
        let mut ty = self.allocation(pointer)?.ty;
        for projection in &pointer.data()?.path {
            ty = match (projection, types.kind(ty)?) {
                (
                    Projection::Bytes {
                        ty, ..
                    },
                    _,
                ) => *ty,
                (Projection::Sequence(field), _) => sequence_field_type(types, ty, *field)?,
                (Projection::Field(index), kind) if kind.record_storage_id().is_some() => {
                    let fields = &types.record_storage_definition(ty)?.fields;
                    *fields.get(*index).ok_or(Error::OutOfBounds {
                        index: *index,
                        length: fields.len(),
                    })?
                }
                (
                    Projection::Index(index),
                    TypeKind::FixedArray {
                        element,
                        count,
                    },
                ) if (*index as u128) <= u128::from(*count) => *element,
                (Projection::Index(_), TypeKind::String) => {
                    types.scalar(ScalarType::Int(IntegerType::U8))
                }
                _ => return Err(Error::InvalidIr("virtual pointer has invalid projection")),
            };
        }
        Ok(ty)
    }
    fn validate_pointer(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<(), Error> {
        if let Some(code) = pointer.code_pointer() {
            types.kind(pointer.pointee)?;
            return self.validate_code_pointer(types, code);
        }
        self.path_type(types, pointer)?;
        types.kind(pointer.pointee)?;
        Ok(())
    }
    fn image_for(&self, types: &dyn TypeView, allocation: &Allocation) -> Result<ByteImage, Error> {
        self.ensure_image(types, allocation)?;
        allocation
            .image
            .borrow()
            .as_ref()
            .cloned()
            .ok_or(Error::InvalidIr("byte image initialization failed"))
    }
    fn ensure_image(&self, types: &dyn TypeView, allocation: &Allocation) -> Result<(), Error> {
        if allocation.image.borrow().is_some() {
            return Ok(());
        }
        let value = allocation.value.as_ref().ok_or(Error::Uninitialized)?;
        let mut image = match value {
            Value::String(bytes) => {
                ByteImage::from_bytes(self.target, bytes.clone(), self.limits.value_cells)
            }
            value => ByteImage::encode(
                types,
                self.target,
                allocation.ty,
                value,
                self.limits.value_cells,
            ),
        }?;
        let (cells, total) = self.image_cell_charge(allocation, &image)?;
        self.retokenize_image(types, &mut image)?;
        *allocation.image.borrow_mut() = Some(image);
        allocation.cells.set(cells);
        self.cells.set(total);
        Ok(())
    }
    fn image_cell_charge(
        &self,
        allocation: &Allocation,
        image: &ByteImage,
    ) -> Result<(usize, usize), Error> {
        let image_cells = image
            .len()
            .checked_add(image.metadata_cells())
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let cells = allocation.cells.get().max(image_cells);
        let total = self
            .cells
            .get()
            .checked_sub(allocation.cells.get())
            .and_then(|old| old.checked_add(cells))
            .filter(|total| *total <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok((cells, total))
    }
    fn retokenize_image(&self, types: &dyn TypeView, image: &mut ByteImage) -> Result<(), Error> {
        image.retokenize_handles(|value| {
            let key = match value {
                Value::Pointer(pointer) if pointer.code_pointer().is_some() => {
                    let code = pointer
                        .code_pointer()
                        .ok_or(Error::InvalidIr("missing code address receipt"))?;
                    self.validate_code_pointer(types, code)?;
                    return Ok(code.token());
                }
                Value::Pointer(pointer) if !pointer.is_null() => {
                    self.validate_pointer(types, pointer)?;
                    // Live allocation identity already owns this canonical address.
                    // Retaining a second ledger entry would keep retired frame history.
                    return self
                        .allocation(pointer)?
                        .virtual_base
                        .checked_add(self.byte_offset(types, pointer)?)
                        .ok_or(Error::CheckedCast);
                }
                Value::Procedure {
                    signature,
                    procedure: Some(procedure),
                } => {
                    types.procedure_definition(*signature)?;
                    HandleKey::Procedure {
                        signature: *signature,
                        procedure: *procedure,
                    }
                }
                _ => {
                    return Err(Error::InvalidIr(
                        "relocation must contain a live nonnull handle",
                    ));
                }
            };
            let mut tokens = self.handle_tokens.borrow_mut();
            if let Some(token) = tokens.values.get(&key) {
                return Ok(*token);
            }
            if tokens.values.len() >= self.limits.value_cells {
                return Err(Error::Limit(LimitKind::ValueCells));
            }
            let token =
                self.reserve_virtual_region(1, u64::from(self.target.policy.pointer().alignment))?;
            tokens.values.insert(key, token);
            Ok(token)
        })?;
        self.certify_code_image(types, image)
    }
    pub(crate) fn union_value_field(
        &self,
        types: &dyn TypeView,
        ty: TypeId,
        value: &Value,
        field: usize,
    ) -> Result<Value, Error> {
        if let Value::StoredAggregate(snapshot) = value {
            let value = snapshot.field(types, field, self.limits.value_cells)?;
            self.validate_runtime_type_values(types, &value)?;
            return Ok(value);
        }
        let mut image = ByteImage::encode(types, self.target, ty, value, self.limits.value_cells)?;
        self.retokenize_image(types, &mut image)?;
        let fields = &types.record_storage_definition(ty)?.fields;
        let field_ty = *fields.get(field).ok_or(Error::OutOfBounds {
            index: field,
            length: fields.len(),
        })?;
        let value = image.read_preserving(types, self.target, 0, field_ty)?;
        self.validate_runtime_type_values(types, &value)?;
        Ok(value)
    }
    fn union_projections(
        &self,
        types: &dyn TypeView,
        pointer: &Pointer,
    ) -> Result<Vec<(usize, TypeId, usize)>, Error> {
        let mut ty = self.allocation(pointer)?.ty;
        let mut prefix = pointer.clone();
        prefix.data_mut()?.path.clear();
        let mut unions = vec![];
        for projection in &pointer.data()?.path {
            if let (Projection::Field(field), TypeKind::Record(id)) = (projection, types.kind(ty)?)
                && types.record(*id)?.kind == jai_types::RecordKind::Union
            {
                unions.push((
                    usize::try_from(self.byte_offset(types, &prefix)?)
                        .map_err(|_| Error::CheckedCast)?,
                    ty,
                    *field,
                ));
            }
            prefix.data_mut()?.path.push(projection.clone());
            ty = self.path_type(types, &prefix)?;
        }
        Ok(unions)
    }
    pub fn load(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<Value, Error> {
        let value = self.load_value(types, pointer)?;
        self.validate_runtime_type_values(types, &value)?;
        Ok(value)
    }
    fn load_value(&self, types: &dyn TypeView, pointer: &Pointer) -> Result<Value, Error> {
        self.validate_pointer(types, pointer)?;
        self.validate_access(types, pointer)?;
        let allocation = self.allocation(pointer)?;
        if pointer.data()?.path.is_empty()
            && let Some(Value::String(bytes)) = &allocation.value
        {
            let image = allocation.image.borrow();
            return Ok(Value::String(match image.as_ref() {
                Some(image) => image.read_range(0, image.len())?.to_vec(),
                None => bytes.clone(),
            }));
        }
        if allocation.image.borrow().is_some()
            || allocation.has_stored_aggregate
            || pointer
                .data()?
                .path
                .iter()
                .any(|p| matches!(p, Projection::Bytes { .. }))
            || self.path_type(types, pointer)? != pointer.pointee
            || !self.union_projections(types, pointer)?.is_empty()
        {
            return self.load_stored_aggregate(types, pointer);
        }
        let mut value = self
            .allocation(pointer)?
            .value
            .as_ref()
            .ok_or(Error::Uninitialized)?;
        for (ordinal, projection) in pointer.data()?.path.iter().enumerate() {
            if let Projection::Sequence(field) = projection {
                if ordinal + 1 != pointer.data()?.path.len() {
                    return Err(Error::InvalidIr(
                        "cannot project sequence descriptor scalar further",
                    ));
                }
                return sequence_field_value(value, *field);
            }
            if let (Projection::Index(index), Value::String(bytes)) = (projection, value) {
                if ordinal + 1 != pointer.data()?.path.len() {
                    return Err(Error::InvalidIr("cannot project a string byte further"));
                }
                let byte = *bytes.get(*index).ok_or(Error::OutOfBounds {
                    index: *index,
                    length: bytes.len(),
                })?;
                return reinterpret(
                    types,
                    Value::Int(Integer::wrapping(IntegerType::U8, i128::from(byte))),
                    pointer.pointee,
                );
            }
            value = match (projection, value) {
                (
                    Projection::Field(index),
                    Value::Record {
                        fields, ..
                    },
                ) => fields.get(*index).ok_or(Error::OutOfBounds {
                    index: *index,
                    length: fields.len(),
                })?,
                (
                    Projection::Field(index),
                    Value::Union {
                        field,
                        value,
                        ..
                    },
                ) if index == field => value,
                (
                    Projection::Index(index),
                    Value::Array {
                        elements, ..
                    },
                ) => elements.get(*index).ok_or(Error::OutOfBounds {
                    index: *index,
                    length: elements.len(),
                })?,
                _ => {
                    return Err(Error::InvalidIr(
                        "read of inactive union field or malformed aggregate",
                    ));
                }
            };
        }
        reinterpret(types, value.clone(), pointer.pointee)
    }
    pub fn store(
        &mut self,
        types: &dyn TypeView,
        pointer: &Pointer,
        value: Value,
    ) -> Result<(), Error> {
        self.validate_pointer(types, pointer)?;
        self.validate_access(types, pointer)?;
        if self.allocation(pointer)?.readonly {
            return Err(Error::ReadOnlyStorage);
        }
        value.cells(self.limits.value_cells)?;
        value.validate(types, pointer.pointee, self.limits.evaluation_depth)?;
        self.validate_runtime_type_values(types, &value)?;
        if let Value::StoredAggregate(snapshot) = &value {
            return self.store_stored_aggregate(types, pointer, snapshot);
        }
        let allocation = self.allocation(pointer)?;
        let unions = self.union_projections(types, pointer)?;
        if !pointer.data()?.path.is_empty()
            && (matches!(value, Value::AddressInteger(_))
                || allocation.value.is_none()
                || allocation.image.borrow().is_some()
                || allocation.has_stored_aggregate
                || pointer
                    .data()?
                    .path
                    .iter()
                    .any(|p| matches!(p, Projection::Bytes { .. }))
                || self.path_type(types, pointer)? != pointer.pointee
                || !unions.is_empty())
        {
            let mut image = if allocation.value.is_none() && allocation.image.borrow().is_none() {
                let length = usize::try_from(self.storage_length(types, pointer)?)
                    .map_err(|_| Error::Limit(LimitKind::ValueCells))?;
                ByteImage::uninitialized(self.target, length, self.limits.value_cells)?
            } else {
                self.image_for(types, allocation)?
            };
            image.write(
                types,
                self.target,
                usize::try_from(self.byte_offset(types, pointer)?)
                    .map_err(|_| Error::CheckedCast)?,
                pointer.pointee,
                &value,
            )?;
            self.retokenize_image(types, &mut image)?;
            for (offset, ty, field) in unions {
                image.note_union_field(types, offset, ty, field)?;
            }
            return self.install_intrinsic_image(pointer, image);
        }
        let value = reinterpret(types, value, self.path_type(types, pointer)?)?;
        // Change a temporary root first so bounds/type/limit errors do not partially mutate memory.
        let allocation = self.allocation(pointer)?;
        let mut root = allocation.value.clone();
        if pointer.data()?.path.is_empty() {
            root = Some(value);
        } else {
            let mut destination = root.as_mut().ok_or(Error::Uninitialized)?;
            let mut incoming = Some(value);
            let mut wrote_byte = false;
            for (ordinal, projection) in pointer.data()?.path.iter().enumerate() {
                if let Projection::Sequence(field) = projection {
                    if ordinal + 1 != pointer.data()?.path.len() {
                        return Err(Error::InvalidIr(
                            "cannot project sequence descriptor scalar further",
                        ));
                    }
                    store_sequence_field(
                        destination,
                        *field,
                        incoming
                            .take()
                            .ok_or(Error::InvalidIr("missing sequence store value"))?,
                    )?;
                    wrote_byte = true;
                    break;
                }
                if let (Projection::Index(index), Value::String(bytes)) =
                    (projection, &mut *destination)
                {
                    if ordinal + 1 != pointer.data()?.path.len() {
                        return Err(Error::InvalidIr("cannot project a string byte further"));
                    }
                    let byte = incoming
                        .take()
                        .ok_or(Error::InvalidIr("missing byte store value"))?
                        .integer()?;
                    if byte.ty() != IntegerType::U8 {
                        return Err(Error::InvalidIr("byte store requires u8"));
                    }
                    let length = bytes.len();
                    *bytes.get_mut(*index).ok_or(Error::OutOfBounds {
                        index: *index,
                        length,
                    })? = byte.bits() as u8;
                    wrote_byte = true;
                    break;
                }
                destination = match (projection, destination) {
                    (
                        Projection::Field(index),
                        Value::Record {
                            fields, ..
                        },
                    ) => {
                        let length = fields.len();
                        fields.get_mut(*index).ok_or(Error::OutOfBounds {
                            index: *index,
                            length,
                        })?
                    }
                    (
                        Projection::Field(index),
                        Value::Union {
                            field,
                            value,
                            ..
                        },
                    ) => {
                        if index != field {
                            return Err(Error::InvalidIr("write of inactive union field"));
                        }
                        value
                    }
                    (
                        Projection::Index(index),
                        Value::Array {
                            elements, ..
                        },
                    ) => {
                        let length = elements.len();
                        elements.get_mut(*index).ok_or(Error::OutOfBounds {
                            index: *index,
                            length,
                        })?
                    }
                    _ => {
                        return Err(Error::InvalidIr(
                            "write of inactive union field or malformed aggregate",
                        ));
                    }
                };
            }
            if !wrote_byte {
                *destination = incoming.ok_or(Error::InvalidIr("missing aggregate store value"))?;
            }
        }
        let (cells, has_stored_aggregate) = root.as_ref().map_or(Ok((1, false)), |value| {
            value.cells_and_storage(self.limits.value_cells)
        })?;
        let total = (self.cells.get() - allocation.cells.get())
            .checked_add(cells)
            .filter(|cells| *cells <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let allocation = self
            .allocations
            .get_mut(&pointer.allocation_id())
            .ok_or(Error::DanglingPointer)?;
        allocation.value = root;
        allocation.has_stored_aggregate = has_stored_aggregate;
        *allocation.image.borrow_mut() = None;
        allocation.cells.set(cells);
        self.cells.set(total);
        Ok(())
    }
}

fn scalar_width(types: &dyn TypeView, ty: TypeId) -> Result<Option<u32>, Error> {
    Ok(match types.kind(ty)? {
        TypeKind::Integer(ty) => Some(ty.bits()),
        TypeKind::Float(ty) => Some(ty.bits()),
        TypeKind::Enum(id) => Some(types.enumeration(*id)?.representation.bits()),
        _ => None,
    })
}
fn reinterpret(types: &dyn TypeView, value: Value, ty: TypeId) -> Result<Value, Error> {
    if value.validate(types, ty, 256).is_ok() {
        return Ok(value);
    }
    if let Value::AddressInteger(number) = &value {
        if let TypeKind::Integer(integer) = types.kind(ty)?
            && integer.bits() == number.ty().bits()
        {
            return Ok(crate::Number::address(
                Integer::wrapping(*integer, i128::from(number.bits())),
                number
                    .provenance()
                    .cloned()
                    .ok_or(Error::InvalidIr("address integer has no provenance"))?,
            )
            .into_value());
        }
        return Err(Error::UnsupportedPointerOperation(
            "address-derived bytes cannot be reinterpreted as float or enum",
        ));
    }
    let (bits, width) = match value {
        Value::Int(value) => (value.bits(), value.ty().bits()),
        Value::Float(value) => (value.bits(), value.ty().bits()),
        Value::Enum {
            value, ..
        } => (value.bits(), value.ty().bits()),
        _ => {
            return Err(Error::TypeMismatch {
                expected: ty,
            });
        }
    };
    if scalar_width(types, ty)? != Some(width) {
        return Err(Error::TypeMismatch {
            expected: ty,
        });
    }
    Ok(match types.kind(ty)? {
        TypeKind::Integer(integer) => Value::Int(Integer::wrapping(*integer, i128::from(bits))),
        TypeKind::Float(jai_types::FloatType::F32) => {
            Value::Float(jai_types::FloatValue::F32(bits as u32))
        }
        TypeKind::Float(jai_types::FloatType::F64) => {
            Value::Float(jai_types::FloatValue::F64(bits))
        }
        TypeKind::Enum(id) => Value::Enum {
            ty,
            value: Integer::wrapping(types.enumeration(*id)?.representation, i128::from(bits)),
        },
        _ => {
            return Err(Error::TypeMismatch {
                expected: ty,
            });
        }
    })
}

fn sequence_field_type(
    types: &dyn TypeView,
    ty: TypeId,
    field: jai_ir::SequenceField,
) -> Result<TypeId, Error> {
    let element = match types.kind(ty)? {
        TypeKind::Slice(element) | TypeKind::DynamicArray(element) => *element,
        TypeKind::String => types.scalar(ScalarType::Int(IntegerType::U8)),
        _ => return Err(Error::UnsupportedType(ty)),
    };
    match field {
        jai_ir::SequenceField::Count => Ok(types.scalar(ScalarType::Int(IntegerType::S64))),
        jai_ir::SequenceField::Allocated
            if matches!(types.kind(ty)?, TypeKind::DynamicArray(_)) =>
        {
            Ok(types.scalar(ScalarType::Int(IntegerType::S64)))
        }
        jai_ir::SequenceField::Data => types
            .lookup(&TypeKind::Pointer(element))
            .ok_or(Error::UnsupportedType(ty)),
        _ => Err(Error::UnsupportedType(ty)),
    }
}
fn sequence_field_value(value: &Value, field: jai_ir::SequenceField) -> Result<Value, Error> {
    let (pointer, count, allocated) = match value {
        Value::Slice {
            pointer,
            count,
            ..
        }
        | Value::StringView {
            pointer,
            count,
        } => (pointer, *count, None),
        Value::DynamicArray {
            pointer,
            count,
            allocated,
            ..
        } => (pointer, *count, Some(*allocated)),
        _ => return Err(Error::InvalidIr("sequence projection requires descriptor")),
    };
    Ok(match field {
        jai_ir::SequenceField::Data => Value::Pointer(pointer.clone()),
        jai_ir::SequenceField::Count => {
            Value::Int(Integer::checked(IntegerType::S64, count as i128).ok_or(Error::CheckedCast)?)
        }
        jai_ir::SequenceField::Allocated => Value::Int(
            Integer::checked(
                IntegerType::S64,
                allocated.ok_or(Error::InvalidIr("allocated requires dynamic array"))? as i128,
            )
            .ok_or(Error::CheckedCast)?,
        ),
    })
}
fn store_sequence_field(
    destination: &mut Value,
    field: jai_ir::SequenceField,
    value: Value,
) -> Result<(), Error> {
    let (pointer, count, allocated) = match destination {
        Value::Slice {
            pointer,
            count,
            ..
        }
        | Value::StringView {
            pointer,
            count,
        } => (pointer, count, None),
        Value::DynamicArray {
            pointer,
            count,
            allocated,
            ..
        } => (pointer, count, Some(allocated)),
        _ => return Err(Error::InvalidIr("sequence store requires descriptor")),
    };
    if matches!(value, Value::AddressInteger(_)) {
        return Err(Error::UnsupportedPointerOperation(
            "address-derived integer cannot become a sequence descriptor count",
        ));
    }
    match field {
        jai_ir::SequenceField::Data => *pointer = value.pointer()?.clone(),
        jai_ir::SequenceField::Count => {
            *count = i64::try_from(value.integer()?.value()).map_err(|_| Error::CheckedCast)?
        }
        jai_ir::SequenceField::Allocated => {
            *allocated.ok_or(Error::InvalidIr("allocated requires dynamic array"))? =
                i64::try_from(value.integer()?.value()).map_err(|_| Error::CheckedCast)?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{Integer, IntegerType, RecordKind, ScalarType, TypeRegistry};
    #[test]
    fn nested_projections_preserve_identity_bounds_and_type() {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::U8));
        let array = types.fixed_array(integer, 3).unwrap();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [array, integer]).unwrap();
        let byte = |value| Value::Int(Integer::wrapping(IntegerType::U8, value));
        let mut memory = Memory::new(Limits::default());
        let pointer = memory
            .allocate(
                &types,
                record,
                Some(Value::Record {
                    ty: record,
                    fields: vec![
                        Value::Array {
                            ty: array,
                            elements: vec![byte(1), byte(2), byte(3)],
                        },
                        byte(4),
                    ],
                }),
            )
            .unwrap();
        let items = memory.field(&types, &pointer, 0).unwrap();
        let second = memory.index(&types, &items, 1).unwrap();
        memory.store(&types, &second, byte(9)).unwrap();
        assert_eq!(memory.load(&types, &second).unwrap(), byte(9));
        let third = memory.offset(&types, &second, 1).unwrap();
        assert_eq!(memory.load(&types, &third).unwrap(), byte(3));
        let one_past = memory.offset(&types, &third, 1).unwrap();
        assert!(matches!(
            memory.load(&types, &one_past),
            Err(Error::OutOfBounds { .. })
        ));
        assert_eq!(memory.distance(&types, &one_past, &second).unwrap(), 2);
        assert!(matches!(
            memory.offset(&types, &one_past, 1),
            Err(Error::OutOfBounds { .. })
        ));
        assert!(matches!(
            memory.index(&types, &items, 3),
            Err(Error::OutOfBounds { .. })
        ));
        assert!(matches!(
            memory.store(&types, &second, Value::Bool(true)),
            Err(Error::TypeMismatch { .. })
        ));
        let other = Memory::new(Limits::default());
        assert!(matches!(
            other.load(&types, &second),
            Err(Error::ForeignPointer)
        ));
        memory.release(&pointer).unwrap();
        assert!(matches!(
            memory.load(&types, &second),
            Err(Error::DanglingPointer)
        ));
    }
    #[test]
    fn rollback_does_not_reuse_pointer_identities_and_limits_are_atomic() {
        let types = TypeRegistry::new();
        let boolean = types.scalar(ScalarType::Bool);
        let mut memory = Memory::new(Limits::default());
        memory
            .prepare_layout(&types, boolean, usize::MAX)
            .1
            .unwrap();
        let layout_cells = memory.value_cells();
        memory.limits.allocations = 1;
        memory.limits.value_cells = layout_cells + 1;
        let snapshot = memory.snapshot();
        let stale = memory
            .allocate(&types, boolean, Some(Value::Bool(true)))
            .unwrap();
        let before_failed_allocation = memory.value_cells();
        assert!(matches!(
            memory.allocate(&types, boolean, None),
            Err(Error::Limit(LimitKind::Allocations))
        ));
        assert_eq!(memory.value_cells(), before_failed_allocation);
        memory.restore(snapshot);
        let fresh = memory
            .allocate(&types, boolean, Some(Value::Bool(false)))
            .unwrap();
        assert_ne!(stale, fresh);
        assert!(matches!(
            memory.load(&types, &stale),
            Err(Error::DanglingPointer)
        ));
        assert_eq!(memory.value_cells(), layout_cells + 1);
    }
}
