//! Target-layout byte storage with opaque virtual-handle relocations.
//!
//! Bytes never contain host addresses. Pointer/procedure values can be recovered
//! only while their complete relocation survives; arbitrary bytes cannot forge one.
use crate::{Error, LimitKind, StoredAggregate, Value};
use jai_types::{
    FloatType, FloatValue, Integer, IntegerType, Layout, LayoutEngine, LayoutPolicy, RecordKind,
    ScalarType, TypeId, TypeKind, TypeView,
};
use std::sync::atomic::AtomicU64;

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
const MAX_DEPTH: usize = 128;

mod allocators;
mod comparison;
mod handles;
mod initialization;
mod metadata;
mod partial_snapshots;
mod provenance;
mod ranges;
mod storage_reinterpretation;
use provenance::ByteSpan;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Endian {
    #[default]
    Little,
    Big,
}
/// VM storage policy, independent of Rust's host ABI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByteTarget {
    pub policy: LayoutPolicy,
    pub endian: Endian,
}
impl Default for ByteTarget {
    fn default() -> Self {
        Self {
            policy: LayoutPolicy::lp64(),
            endian: Endian::Little,
        }
    }
}
impl From<&jai_types::BuildTarget> for ByteTarget {
    fn from(target: &jai_types::BuildTarget) -> Self {
        Self {
            policy: target.layout,
            endian: match target.byte_order {
                jai_types::ByteOrder::Little => Endian::Little,
                jai_types::ByteOrder::Big => Endian::Big,
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Relocation {
    offset: usize,
    length: usize,
    value: Value,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct UnionView {
    offset: usize,
    length: usize,
    ty: TypeId,
    field: usize,
}
/// Padding is initialized to zero; virtual handles retain their provenance.
#[derive(Clone, Debug)]
pub struct ByteImage {
    bytes: Vec<u8>,
    initialized: Vec<bool>,
    provenance: Vec<ByteSpan>,
    relocations: Vec<Relocation>,
    unions: Vec<UnionView>,
    target: ByteTarget,
    limit: usize,
}
impl PartialEq for ByteImage {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
            && self.initialized == other.initialized
            && self.provenance == other.provenance
            && self.relocations == other.relocations
            && self.unions == other.unions
            && self.target == other.target
    }
}
impl Eq for ByteImage {}
impl std::hash::Hash for ByteImage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
        self.initialized.hash(state);
        self.provenance.hash(state);
        self.relocations.hash(state);
        self.unions.hash(state);
        self.target.hash(state);
    }
}
impl ByteImage {
    /// Raw backing bytes carry no data-pointer or procedure provenance.
    pub fn from_bytes(target: ByteTarget, bytes: Vec<u8>, limit: usize) -> Result<Self, Error> {
        if bytes.len() > limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        Ok(Self {
            initialized: vec![true; bytes.len()],
            bytes,
            provenance: vec![],
            relocations: vec![],
            unions: vec![],
            target,
            limit,
        })
    }
    pub fn encode(
        types: &dyn TypeView,
        target: ByteTarget,
        ty: TypeId,
        value: &Value,
        limit: usize,
    ) -> Result<Self, Error> {
        value.validate(types, ty, MAX_DEPTH)?;
        value.cells(limit)?;
        metadata::encoded_metadata_cells(value, limit)?;
        if let Value::StoredAggregate(snapshot) = value {
            snapshot.image().check_target(target)?;
            return snapshot.image().clone_with_limit(limit);
        }
        let mut layouts = LayoutEngine::new(types, target.policy);
        let length = size(layout(&mut layouts, ty)?.size, limit)?;
        let mut result = Self {
            bytes: vec![0; length],
            initialized: vec![true; length],
            provenance: vec![],
            relocations: vec![],
            unions: vec![],
            target,
            limit,
        };
        result.encode_at(types, &mut layouts, 0, ty, value, 0)?;
        result.normalize_metadata();
        result.check_metadata_cells(result.metadata_cells())?;
        Ok(result)
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn target(&self) -> ByteTarget {
        self.target
    }
    pub fn read(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
    ) -> Result<Value, Error> {
        self.check_target(target)?;
        let mut layouts = LayoutEngine::new(types, target.policy);
        let mut remaining = self.limit;
        self.decode_at(types, &mut layouts, offset, ty, 0, &mut remaining)
    }
    pub(crate) fn read_preserving(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
    ) -> Result<Value, Error> {
        self.read_preserving_with_limit(types, target, offset, ty, self.limit)
    }
    pub(crate) fn read_preserving_with_limit(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        limit: usize,
    ) -> Result<Value, Error> {
        self.read_aggregate(types, target, offset, ty, limit, false)
    }
    pub(crate) fn read_partial_preserving_with_limit(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        limit: usize,
    ) -> Result<Value, Error> {
        self.read_aggregate(types, target, offset, ty, limit, true)
    }
    fn read_aggregate(
        &self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        limit: usize,
        allow_unknown: bool,
    ) -> Result<Value, Error> {
        self.check_target(target)?;
        let mut layouts = LayoutEngine::new(types, target.policy);
        let mut remaining = limit;
        if let Some(snapshot) =
            self.partial_snapshot(types, &mut layouts, offset, ty, limit, allow_unknown)?
        {
            return Ok(snapshot);
        }
        let semantic = self.decode_at(types, &mut layouts, offset, ty, 0, &mut remaining)?;
        let mut pending = vec![&semantic];
        let mut has_union = false;
        while let Some(value) = pending.pop() {
            match value {
                Value::Union { .. } => {
                    has_union = true;
                    break;
                }
                Value::Record { fields, .. }
                | Value::Array {
                    elements: fields, ..
                } => pending.extend(fields),
                Value::Distinct { value, .. } => pending.push(value),
                _ => {}
            }
        }
        if !has_union {
            return Ok(semantic);
        }
        let length = size(layout(&mut layouts, ty)?.size, limit)?;
        let range = self.range(offset, length)?;
        let storage = length
            .checked_add(self.range_metadata_cells(&range))
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        semantic
            .cells(limit)?
            .checked_add(storage)
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let mut image = self.extract_range(offset, length)?;
        image.limit = limit;
        Ok(Value::StoredAggregate(StoredAggregate::new(
            ty, semantic, image, limit,
        )?))
    }
    pub(crate) fn clone_with_limit(&self, limit: usize) -> Result<Self, Error> {
        if self.len() > limit || self.metadata_cells() > limit {
            return Err(Error::Limit(LimitKind::ValueCells));
        }
        let mut image = self.clone();
        image.limit = limit;
        Ok(image)
    }
    pub(crate) fn fully_initialized(&self) -> bool {
        self.initialized.iter().all(|v| *v)
    }
    pub(crate) fn validate_memory_provenance(&self, memory: u64) -> Result<(), Error> {
        for span in &self.provenance {
            if span
                .memory_identity()
                .is_some_and(|identity| identity != memory)
            {
                return Err(Error::ForeignPointer);
            }
        }
        Ok(())
    }
    /// Decode with a specific union member at this location, including inactive members.
    pub fn read_union_field(
        &self,
        types: &dyn TypeView,
        offset: usize,
        ty: TypeId,
        field: usize,
    ) -> Result<Value, Error> {
        let TypeKind::Record(id) = types.kind(ty)? else {
            return Err(Error::UnsupportedType(ty));
        };
        let record = types.record(*id)?;
        if record.kind != RecordKind::Union {
            return Err(Error::UnsupportedType(ty));
        }
        let field_ty = *record.fields.get(field).ok_or(Error::OutOfBounds {
            index: field,
            length: record.fields.len(),
        })?;
        self.read(types, self.target, offset, field_ty)
    }
    /// Select the active reconstruction view after a typed union-field store.
    pub fn note_union_field(
        &mut self,
        types: &dyn TypeView,
        offset: usize,
        ty: TypeId,
        field: usize,
    ) -> Result<(), Error> {
        let TypeKind::Record(id) = types.kind(ty)? else {
            return Err(Error::UnsupportedType(ty));
        };
        let record = types.record(*id)?;
        if record.kind != RecordKind::Union {
            return Err(Error::UnsupportedType(ty));
        }
        if field >= record.fields.len() {
            return Err(Error::OutOfBounds {
                index: field,
                length: record.fields.len(),
            });
        }
        let mut layouts = LayoutEngine::new(types, self.target.policy);
        let length = size(layout(&mut layouts, ty)?.size, self.limit)?;
        self.range(offset, length)?;
        let replaced = usize::from(self.union_field(offset, ty).is_some());
        self.check_metadata_cells(
            self.metadata_cells()
                .saturating_sub(replaced)
                .saturating_add(1),
        )?;
        self.unions
            .retain(|old| old.offset != offset || old.ty != ty);
        self.unions.push(UnionView {
            offset,
            length,
            ty,
            field,
        });
        self.normalize_metadata();
        Ok(())
    }
    /// A failed write does not mutate either bytes or handle provenance.
    pub fn write(
        &mut self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        value: &Value,
    ) -> Result<(), Error> {
        self.write_normalized(types, target, offset, ty, value, |_| Ok(()))
    }
    /// Normalize a typed patch before committing it, so later partial writes
    /// retain canonical address bits even when its complete relocation vanishes.
    pub(crate) fn write_normalized(
        &mut self,
        types: &dyn TypeView,
        target: ByteTarget,
        offset: usize,
        ty: TypeId,
        value: &Value,
        normalize: impl FnOnce(&mut ByteImage) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.check_target(target)?;
        let mut patch = Self::encode(types, target, ty, value, self.limit)?;
        normalize(&mut patch)?;
        let end = self.range(offset, patch.len())?.end;
        let retained_handles = self
            .relocations
            .iter()
            .filter(|r| !overlap(offset, end, r.offset, r.offset + r.length))
            .fold(0usize, |sum, r| sum.saturating_add(r.metadata_cells()));
        let retained_unions = self
            .unions
            .iter()
            .filter(|u| {
                u.offset
                    .checked_sub(offset)
                    .is_none_or(|relative| patch.union_field(relative, u.ty).is_none())
            })
            .count();
        self.check_metadata_cells(
            self.retained_provenance_cells(&(offset..end))
                .saturating_add(retained_handles)
                .saturating_add(retained_unions)
                .saturating_add(patch.metadata_cells()),
        )?;
        self.clear_provenance(&(offset..end));
        self.provenance
            .extend(patch.provenance.into_iter().map(|mut span| {
                span.offset += offset;
                span
            }));
        self.bytes[offset..end].copy_from_slice(&patch.bytes);
        self.initialized[offset..end].copy_from_slice(&patch.initialized);
        self.relocations
            .retain(|r| !overlap(offset, end, r.offset, r.offset + r.length));
        self.relocations
            .extend(patch.relocations.into_iter().map(|mut r| {
                r.offset += offset;
                r
            }));
        for mut union in patch.unions {
            union.offset += offset;
            self.unions
                .retain(|old| old.offset != union.offset || old.ty != union.ty);
            self.unions.push(union);
        }
        self.normalize_metadata();
        Ok(())
    }
    fn check_target(&self, target: ByteTarget) -> Result<(), Error> {
        if target != self.target {
            return Err(Error::InvalidIr("byte storage target policy changed"));
        }
        Ok(())
    }
    fn range(&self, offset: usize, length: usize) -> Result<std::ops::Range<usize>, Error> {
        let end = offset.checked_add(length).ok_or(Error::OutOfBounds {
            index: usize::MAX,
            length: self.len(),
        })?;
        if end > self.len() {
            return Err(Error::OutOfBounds {
                index: end,
                length: self.len(),
            });
        }
        Ok(offset..end)
    }
    fn put_bits(&mut self, offset: usize, length: usize, bits: u64) -> Result<(), Error> {
        let range = self.range(offset, length)?;
        if length > 8 {
            return Err(Error::InvalidIr("scalar storage wider than 64 bits"));
        }
        let bytes = match self.target.endian {
            Endian::Little => bits.to_le_bytes(),
            Endian::Big => bits.to_be_bytes(),
        };
        let source = match self.target.endian {
            Endian::Little => &bytes[..length],
            Endian::Big => &bytes[8 - length..],
        };
        self.bytes[range].copy_from_slice(source);
        Ok(())
    }
    fn bits(&self, offset: usize, length: usize) -> Result<u64, Error> {
        let range = self.initialized_range(offset, length)?;
        let source = &self.bytes[range];
        if length > 8 {
            return Err(Error::InvalidIr("scalar storage wider than 64 bits"));
        }
        let mut bytes = [0u8; 8];
        match self.target.endian {
            Endian::Little => {
                bytes[..length].copy_from_slice(source);
                Ok(u64::from_le_bytes(bytes))
            }
            Endian::Big => {
                bytes[8 - length..].copy_from_slice(source);
                Ok(u64::from_be_bytes(bytes))
            }
        }
    }
    fn encode_at(
        &mut self,
        types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        offset: usize,
        ty: TypeId,
        value: &Value,
        depth: usize,
    ) -> Result<(), Error> {
        depth_check(depth)?;
        let storage = layout(layouts, ty)?.clone();
        let length = size(storage.size, self.limit)?;
        self.range(offset, length)?;
        if let Value::StoredAggregate(snapshot) = value {
            return self.copy_range_from(snapshot.image(), 0, offset, length);
        }
        match (types.kind(ty)?, value) {
            (TypeKind::Bool, Value::Bool(value)) => {
                self.put_bits(offset, length, u64::from(*value))?
            }
            (TypeKind::Integer(_), Value::Int(value))
            | (TypeKind::Enum(_), Value::Enum { value, .. }) => {
                self.put_bits(offset, length, value.bits())?
            }
            (TypeKind::Integer(_), Value::AddressInteger(number)) => {
                self.put_bits(offset, length, number.bits())?;
                self.mark_number_provenance(offset, length, number)?;
            }
            (TypeKind::Float(_), Value::Float(value)) => {
                self.put_bits(offset, length, value.bits())?
            }
            (TypeKind::Type, Value::Type { descriptor }) => {
                if let Some(pointer) = descriptor {
                    let header = types.runtime_type_header().ok_or(Error::InvalidIr(
                        "nonnull runtime Type requires a bound descriptor header",
                    ))?;
                    if pointer.pointee() != header || pointer.is_null() || pointer.is_opaque() {
                        return Err(Error::TypeMismatch { expected: ty });
                    }
                    self.encode_handle(offset, length, &Value::Pointer(pointer.clone()))?;
                }
            }
            (TypeKind::Pointer(_), Value::Pointer(_))
            | (TypeKind::Procedure(_), Value::Procedure { .. }) => {
                self.encode_handle(offset, length, value)?;
            }
            (TypeKind::Distinct(id), Value::Distinct { value, .. }) => self.encode_at(
                types,
                layouts,
                offset,
                types.distinct(*id)?.representation,
                value,
                depth + 1,
            )?,
            (kind, Value::Record { fields, .. }) if kind.record_storage_id().is_some() => {
                let record = types.record_storage_definition(ty)?;
                for (index, value) in fields.iter().enumerate() {
                    self.encode_at(
                        types,
                        layouts,
                        at(offset, storage.field_offsets[index])?,
                        record.fields[index],
                        value,
                        depth + 1,
                    )?;
                }
            }
            (TypeKind::Record(id), Value::Union { field, value, .. }) => {
                self.unions.push(UnionView {
                    offset,
                    length,
                    ty,
                    field: *field,
                });
                self.encode_at(
                    types,
                    layouts,
                    offset,
                    types.record(*id)?.fields[*field],
                    value,
                    depth + 1,
                )?;
            }
            (TypeKind::FixedArray { element, .. }, Value::Array { elements, .. }) => {
                let stride = storage
                    .array_stride
                    .ok_or(Error::InvalidIr("array layout missing stride"))?;
                for (index, value) in elements.iter().enumerate() {
                    self.encode_at(
                        types,
                        layouts,
                        at(
                            offset,
                            stride
                                .checked_mul(index as u64)
                                .ok_or(Error::Limit(LimitKind::ValueCells))?,
                        )?,
                        *element,
                        value,
                        depth + 1,
                    )?;
                }
            }
            (TypeKind::String, Value::String(_)) => {
                return Err(Error::InvalidIr(
                    "owned string requires backing descriptor before byte encoding",
                ));
            }
            (TypeKind::String, Value::StringView { pointer, count })
            | (TypeKind::Slice(_), Value::Slice { pointer, count, .. })
            | (TypeKind::DynamicArray(_), Value::DynamicArray { pointer, count, .. }) => {
                let integer = types.scalar(ScalarType::Int(IntegerType::S64));
                let count_value = Value::Int(
                    Integer::checked(IntegerType::S64, *count as i128).ok_or(Error::CheckedCast)?,
                );
                self.encode_at(
                    types,
                    layouts,
                    at(offset, storage.field_offsets[0])?,
                    integer,
                    &count_value,
                    depth + 1,
                )?;
                depth_check(depth + 1)?;
                self.encode_handle(
                    at(offset, storage.field_offsets[1])?,
                    size(self.target.policy.pointer().size, self.limit)?,
                    &Value::Pointer(pointer.clone()),
                )?;
                if let Value::DynamicArray {
                    allocated,
                    allocator,
                    ..
                } = value
                {
                    let allocated = Value::Int(
                        Integer::checked(IntegerType::S64, *allocated as i128)
                            .ok_or(Error::CheckedCast)?,
                    );
                    self.encode_at(
                        types,
                        layouts,
                        at(offset, storage.field_offsets[2])?,
                        integer,
                        &allocated,
                        depth + 1,
                    )?;
                    if let Some(payload) = allocator {
                        let schema =
                            crate::value::allocator_schema(types)?.ok_or(Error::InvalidIr(
                                "allocator payload requires the certified allocator role",
                            ))?;
                        self.allocator_extent(types, layouts, schema, &storage)?;
                        self.encode_at(
                            types,
                            layouts,
                            at(offset, storage.field_offsets[3])?,
                            schema.ty(),
                            payload,
                            depth + 1,
                        )?;
                    }
                }
            }
            _ => return Err(Error::TypeMismatch { expected: ty }),
        }
        Ok(())
    }
    fn decode_at(
        &self,
        types: &dyn TypeView,
        layouts: &mut LayoutEngine<'_>,
        offset: usize,
        ty: TypeId,
        depth: usize,
        remaining: &mut usize,
    ) -> Result<Value, Error> {
        depth_check(depth)?;
        *remaining = remaining
            .checked_sub(1)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        let storage = layout(layouts, ty)?.clone();
        let length = size(storage.size, self.limit)?;
        self.range(offset, length)?;
        Ok(match types.kind(ty)? {
            TypeKind::Bool => {
                self.reject_address_view(
                    offset,
                    length,
                    "address bytes cannot be interpreted as booleans",
                )?;
                Value::Bool(self.bits(offset, length)? & 1 != 0)
            }
            TypeKind::Integer(integer) => {
                self.decode_number(offset, length, *integer, remaining)?
            }
            TypeKind::Float(FloatType::F32) => {
                self.reject_address_view(
                    offset,
                    length,
                    "address bytes cannot be interpreted as floating point",
                )?;
                Value::Float(FloatValue::F32(self.bits(offset, length)? as u32))
            }
            TypeKind::Float(FloatType::F64) => {
                self.reject_address_view(
                    offset,
                    length,
                    "address bytes cannot be interpreted as floating point",
                )?;
                Value::Float(FloatValue::F64(self.bits(offset, length)?))
            }
            TypeKind::Enum(id) => {
                self.reject_address_view(
                    offset,
                    length,
                    "address bytes cannot be interpreted as enum values",
                )?;
                Value::Enum {
                    ty,
                    value: Integer::wrapping(
                        types.enumeration(*id)?.representation,
                        i128::from(self.bits(offset, length)?),
                    ),
                }
            }
            TypeKind::Type => {
                let descriptor = if let Some(header) = types.runtime_type_header() {
                    let pointer = self.decode_pointer(offset, length, header, remaining)?;
                    if pointer.is_null() {
                        None
                    } else if pointer.is_opaque() {
                        return Err(Error::InvalidIr(
                            "byte storage cannot forge a runtime Type descriptor",
                        ));
                    } else {
                        Some(pointer)
                    }
                } else {
                    self.reject_address_view(
                        offset,
                        length,
                        "nonnull runtime Type requires a bound descriptor header",
                    )?;
                    if self.bits(offset, length)? != 0 {
                        return Err(Error::InvalidIr(
                            "byte storage cannot forge a runtime Type descriptor",
                        ));
                    }
                    None
                };
                Value::Type { descriptor }
            }
            TypeKind::Pointer(pointee) => {
                Value::Pointer(self.decode_pointer(offset, length, *pointee, remaining)?)
            }
            TypeKind::Procedure(_) => self.decode_procedure(offset, length, ty)?,
            TypeKind::Distinct(id) => Value::Distinct {
                ty,
                value: Box::new(self.decode_at(
                    types,
                    layouts,
                    offset,
                    types.distinct(*id)?.representation,
                    depth + 1,
                    remaining,
                )?),
            },
            kind if kind.record_storage_id().is_some() => {
                let record = types.record_storage_definition(ty)?;
                if record.kind == RecordKind::Union {
                    let field = self.union_field(offset, ty).unwrap_or(0);
                    let field_ty = *record
                        .fields
                        .get(field)
                        .ok_or(Error::InvalidIr("empty union has no value representation"))?;
                    Value::Union {
                        ty,
                        field,
                        value: Box::new(self.decode_at(
                            types,
                            layouts,
                            offset,
                            field_ty,
                            depth + 1,
                            remaining,
                        )?),
                    }
                } else {
                    if record.fields.len() > *remaining {
                        return Err(Error::Limit(LimitKind::ValueCells));
                    }
                    let mut fields = Vec::with_capacity(record.fields.len());
                    for (index, field) in record.fields.iter().enumerate() {
                        fields.push(self.decode_at(
                            types,
                            layouts,
                            at(offset, storage.field_offsets[index])?,
                            *field,
                            depth + 1,
                            remaining,
                        )?);
                    }
                    Value::Record { ty, fields }
                }
            }
            TypeKind::FixedArray { element, count } => {
                let count = size(*count, *remaining)?;
                let stride = storage
                    .array_stride
                    .ok_or(Error::InvalidIr("array layout missing stride"))?;
                let mut elements = Vec::with_capacity(count);
                for index in 0..count {
                    elements.push(
                        self.decode_at(
                            types,
                            layouts,
                            at(
                                offset,
                                stride
                                    .checked_mul(index as u64)
                                    .ok_or(Error::Limit(LimitKind::ValueCells))?,
                            )?,
                            *element,
                            depth + 1,
                            remaining,
                        )?,
                    );
                }
                Value::Array { ty, elements }
            }
            TypeKind::String | TypeKind::Slice(_) | TypeKind::DynamicArray(_) => {
                let element = match types.kind(ty)? {
                    TypeKind::String => types.scalar(ScalarType::Int(IntegerType::U8)),
                    TypeKind::Slice(element) | TypeKind::DynamicArray(element) => *element,
                    _ => unreachable!(),
                };
                let integer = types.scalar(ScalarType::Int(IntegerType::S64));
                let count = descriptor_count(self.decode_at(
                    types,
                    layouts,
                    at(offset, storage.field_offsets[0])?,
                    integer,
                    depth + 1,
                    remaining,
                )?)?;
                depth_check(depth + 1)?;
                *remaining = remaining
                    .checked_sub(1)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                let pointer = self.decode_pointer(
                    at(offset, storage.field_offsets[1])?,
                    size(self.target.policy.pointer().size, self.limit)?,
                    element,
                    remaining,
                )?;
                match types.kind(ty)? {
                    TypeKind::String => Value::StringView { pointer, count },
                    TypeKind::Slice(_) => Value::Slice { ty, pointer, count },
                    TypeKind::DynamicArray(_) => {
                        let allocated = descriptor_count(self.decode_at(
                            types,
                            layouts,
                            at(offset, storage.field_offsets[2])?,
                            integer,
                            depth + 1,
                            remaining,
                        )?)?;
                        let allocator = self.decode_allocator(
                            types,
                            layouts,
                            offset,
                            &storage,
                            depth + 1,
                            remaining,
                        )?;
                        Value::DynamicArray {
                            ty,
                            pointer,
                            count,
                            allocated,
                            allocator,
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => return Err(Error::UnsupportedType(ty)),
        })
    }
    fn handle(&self, offset: usize, length: usize) -> Option<&Relocation> {
        let index = self
            .relocations
            .binary_search_by_key(&offset, |r| r.offset)
            .ok()?;
        self.relocations.get(index).filter(|r| r.length == length)
    }
    fn union_field(&self, offset: usize, ty: TypeId) -> Option<usize> {
        let start = self.unions.partition_point(|u| u.offset < offset);
        self.unions[start..]
            .iter()
            .take_while(|u| u.offset == offset)
            .find(|u| u.ty == ty)
            .map(|u| u.field)
    }
    // Nonoverlapping handles and address spans are ordered by physical offset.
    // Nested union views may share an offset and retain stable order there.
    fn normalize_metadata(&mut self) {
        self.provenance.sort_by_key(|p| p.offset);
        self.relocations.sort_by_key(|r| r.offset);
        self.unions.sort_by_key(|u| u.offset);
    }
}
fn layout<'a>(engine: &'a mut LayoutEngine<'_>, ty: TypeId) -> Result<&'a Layout, Error> {
    engine.layout(ty).map_err(Error::from)
}
fn size(value: u64, limit: usize) -> Result<usize, Error> {
    usize::try_from(value)
        .ok()
        .filter(|value| *value <= limit)
        .ok_or(Error::Limit(LimitKind::ValueCells))
}
fn at(base: usize, offset: u64) -> Result<usize, Error> {
    base.checked_add(usize::try_from(offset).map_err(|_| Error::Limit(LimitKind::ValueCells))?)
        .ok_or(Error::Limit(LimitKind::ValueCells))
}
fn depth_check(depth: usize) -> Result<(), Error> {
    if depth >= MAX_DEPTH {
        return Err(Error::Limit(LimitKind::EvaluationDepth));
    }
    Ok(())
}
fn descriptor_count(value: Value) -> Result<i64, Error> {
    let number = value.number()?;
    if number.provenance().is_some() {
        return Err(Error::UnsupportedPointerOperation(
            "address-derived bytes cannot form sequence lengths",
        ));
    }
    i64::try_from(number.value()).map_err(|_| Error::CheckedCast)
}
fn overlap(a: usize, b: usize, c: usize, d: usize) -> bool {
    a < b && c < d && a < d && c < b
}

#[cfg(test)]
mod tests;
