use crate::{Error, Number, Pointer, StoredAggregate};
use jai_types::{FloatValue, Integer, RecordKind, TypeId, TypeKind, TypeView};

/// VM values retain language types; pointers are opaque virtual handles.
/// Descriptor fields retain native signed counts; element consumers validate their use.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Value {
    StoredAggregate(StoredAggregate),
    Type {
        descriptor: Option<Pointer>,
    },
    Int(Integer),
    AddressInteger(Number),
    Procedure {
        signature: TypeId,
        procedure: Option<jai_ir::ProcedureId>,
    },
    Distinct {
        ty: TypeId,
        value: Box<Value>,
    },
    Bool(bool),
    Float(FloatValue),
    String(Vec<u8>),
    StringView {
        pointer: Pointer,
        count: i64,
    },
    DynamicArray {
        ty: TypeId,
        pointer: Pointer,
        count: i64,
        allocated: i64,
        allocator: Option<Box<Value>>,
    },
    Pointer(Pointer),
    Record {
        ty: TypeId,
        fields: Vec<Value>,
    },
    Union {
        ty: TypeId,
        field: usize,
        value: Box<Value>,
    },
    Array {
        ty: TypeId,
        elements: Vec<Value>,
    },
    Slice {
        ty: TypeId,
        pointer: Pointer,
        count: i64,
    },
    Enum {
        ty: TypeId,
        value: Integer,
    },
}
impl Value {
    /// Inspection only; copying an aggregate must retain its storage carrier.
    pub fn semantic(&self) -> &Value {
        match self {
            Self::StoredAggregate(value) => value.decoded_semantic().unwrap_or(self),
            value => value,
        }
    }

    pub fn integer(&self) -> Result<Integer, Error> {
        match self {
            Self::Int(value) => Ok(*value),
            Self::AddressInteger(value) => Ok(value.integer()),
            _ => Err(Error::InvalidIr("expected integer value")),
        }
    }
    /// Preserve virtual address provenance for integer dataflow.
    pub fn number(&self) -> Result<Number, Error> {
        match self {
            Self::Int(value) => Ok(Number::plain(*value)),
            Self::AddressInteger(value) => Ok(value.clone()),
            _ => Err(Error::InvalidIr("expected integer value")),
        }
    }
    pub fn pointer(&self) -> Result<&Pointer, Error> {
        match self {
            Self::Pointer(pointer) => Ok(pointer),
            _ => Err(Error::InvalidIr("expected pointer value")),
        }
    }
    pub fn float(&self) -> Result<FloatValue, Error> {
        match self {
            Self::Float(value) => Ok(*value),
            _ => Err(Error::InvalidIr("expected floating-point value")),
        }
    }
    pub fn boolean(&self) -> Result<bool, Error> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(Error::InvalidIr("expected boolean value")),
        }
    }
    /// Validation works before the whole registry is frozen and reports incomplete types.
    pub fn validate(
        &self,
        types: &dyn TypeView,
        expected: TypeId,
        maximum_depth: usize,
    ) -> Result<(), Error> {
        let mut pending = vec![(self, expected, 0usize)];
        while let Some((value, ty, depth)) = pending.pop() {
            if depth > maximum_depth {
                return Err(Error::Limit(crate::LimitKind::EvaluationDepth));
            }
            let mismatch = || Error::TypeMismatch {
                expected: ty,
            };
            if let Self::StoredAggregate(snapshot) = value {
                if snapshot.ty() != ty {
                    return Err(mismatch());
                }
                snapshot.validate_storage(types)?;
                if let Some(semantic) = snapshot.decoded_semantic() {
                    pending.push((semantic, ty, depth + 1));
                }
                continue;
            }
            match (types.kind(ty)?, value) {
                (
                    TypeKind::Type,
                    Self::Type {
                        descriptor: None,
                    },
                ) => {}
                (
                    TypeKind::Type,
                    Self::Type {
                        descriptor: Some(pointer),
                    },
                ) if !pointer.is_null()
                    && !pointer.is_opaque()
                    && types.runtime_type_header() == Some(pointer.pointee()) => {}
                (
                    TypeKind::Procedure(_),
                    Self::Procedure {
                        signature, ..
                    },
                ) if *signature == ty => {}
                (
                    TypeKind::Distinct(id),
                    Self::Distinct {
                        ty: actual,
                        value,
                    },
                ) if *actual == ty => {
                    pending.push((value, types.distinct(*id)?.representation, depth + 1))
                }
                (TypeKind::Bool, Self::Bool(_)) | (TypeKind::String, Self::String(_)) => {}
                (
                    TypeKind::String,
                    Self::StringView {
                        pointer, ..
                    },
                ) if pointer.pointee()
                    == types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)) => {}
                (
                    TypeKind::DynamicArray(element),
                    Self::DynamicArray {
                        ty: actual,
                        pointer,
                        allocator,
                        ..
                    },
                ) if *actual == ty && *element == pointer.pointee() => {
                    match (allocator_schema(types)?, allocator) {
                        (Some(schema), Some(allocator)) => {
                            pending.push((allocator.as_ref(), schema.ty(), depth + 1));
                        }
                        (None, None) => {}
                        _ => return Err(mismatch()),
                    }
                }
                (TypeKind::Integer(expected), Self::Int(actual)) if *expected == actual.ty() => {}
                (TypeKind::Integer(expected), Self::AddressInteger(actual))
                    if *expected == actual.ty() && actual.provenance().is_some() => {}
                (TypeKind::Float(expected), Self::Float(actual)) if *expected == actual.ty() => {}
                (TypeKind::Pointer(pointee), Self::Pointer(pointer))
                    if *pointee == pointer.pointee() => {}
                (
                    TypeKind::Enum(id),
                    Self::Enum {
                        ty: actual,
                        value,
                    },
                ) if ty == *actual => {
                    if types.enumeration(*id)?.representation != value.ty() {
                        return Err(mismatch());
                    }
                }
                (
                    kind,
                    Self::Record {
                        ty: actual,
                        fields,
                    },
                ) if ty == *actual && kind.record_storage_id().is_some() => {
                    let record = types.record_storage_definition(ty)?;
                    if record.kind != RecordKind::Struct || fields.len() != record.fields.len() {
                        return Err(mismatch());
                    }
                    pending.extend(
                        fields
                            .iter()
                            .zip(record.fields.iter())
                            .map(|(value, &ty)| (value, ty, depth + 1)),
                    );
                }
                (
                    TypeKind::Record(id),
                    Self::Union {
                        ty: actual,
                        field,
                        value,
                    },
                ) if ty == *actual => {
                    let record = types.record(*id)?;
                    if record.kind != RecordKind::Union {
                        return Err(mismatch());
                    }
                    let field_ty = record.fields.get(*field).ok_or(Error::OutOfBounds {
                        index: *field,
                        length: record.fields.len(),
                    })?;
                    pending.push((value, *field_ty, depth + 1));
                }
                (
                    TypeKind::FixedArray {
                        element,
                        count,
                    },
                    Self::Array {
                        ty: actual,
                        elements,
                    },
                ) if ty == *actual => {
                    if u64::try_from(elements.len()).ok() != Some(*count) {
                        return Err(mismatch());
                    }
                    pending.extend(elements.iter().map(|value| (value, *element, depth + 1)));
                }
                (
                    TypeKind::Slice(element),
                    Self::Slice {
                        ty: actual,
                        pointer,
                        ..
                    },
                ) if ty == *actual => {
                    if *element != pointer.pointee() {
                        return Err(mismatch());
                    }
                }
                _ => return Err(mismatch()),
            }
        }
        Ok(())
    }
    pub(crate) fn cells(&self, maximum: usize) -> Result<usize, Error> {
        Ok(self.cells_and_storage(maximum)?.0)
    }
    /// Compute the allocation's storage flag during its existing bounded walk.
    pub(crate) fn cells_and_storage(&self, maximum: usize) -> Result<(usize, bool), Error> {
        let mut pending = vec![self];
        let mut cells = 0usize;
        let mut has_storage = false;
        while let Some(value) = pending.pop() {
            cells = cells
                .checked_add(1)
                .filter(|n| *n <= maximum)
                .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
            match value {
                Self::StoredAggregate(snapshot) => {
                    has_storage = true;
                    cells = cells
                        .checked_add(snapshot.storage_cells())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                    if let Some(semantic) = snapshot.decoded_semantic() {
                        pending.push(semantic);
                    }
                }
                Self::Record {
                    fields, ..
                } => {
                    cells = cells
                        .checked_add(fields.capacity() - fields.len())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                    pending.extend(fields);
                }
                Self::Array {
                    elements, ..
                } => {
                    cells = cells
                        .checked_add(elements.capacity() - elements.len())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                    pending.extend(elements);
                }
                Self::Union {
                    value, ..
                }
                | Self::Distinct {
                    value, ..
                } => pending.push(value),
                Self::AddressInteger(number) => {
                    cells = cells
                        .checked_add(number.metadata_cells())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                }
                Self::Pointer(pointer)
                | Self::Slice {
                    pointer, ..
                }
                | Self::StringView {
                    pointer, ..
                }
                | Self::Type {
                    descriptor: Some(pointer),
                } => {
                    cells = cells
                        .checked_add(pointer.metadata_cells())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                }
                Self::DynamicArray {
                    pointer,
                    allocator,
                    ..
                } => {
                    cells = cells
                        .checked_add(pointer.metadata_cells())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                    if let Some(allocator) = allocator {
                        pending.push(allocator.as_ref());
                    }
                }
                Self::String(value) => {
                    cells = cells
                        .checked_add(value.capacity())
                        .filter(|n| *n <= maximum)
                        .ok_or(Error::Limit(crate::LimitKind::ValueCells))?;
                }
                _ => {}
            }
        }
        Ok((cells, has_storage))
    }
}

/// Only the registry's certified nominal role can supply descriptor allocator storage.
pub(crate) fn allocator_schema(
    types: &dyn TypeView,
) -> Result<Option<jai_types::AllocatorSchema>, Error> {
    let Some(schema) = types.allocator_schema() else {
        return Ok(None);
    };
    let current = jai_types::AllocatorSchema::validate(types, schema.ty(), schema.mode_type())
        .map_err(|error| match error {
            jai_types::AllocatorError::Type(error) => Error::Type(error),
            jai_types::AllocatorError::Layout(error) => Error::from(error),
            _ => Error::InvalidIr("allocator role proof does not match the type view"),
        })?;
    if current != schema {
        return Err(Error::InvalidIr(
            "allocator role proof does not match the type view",
        ));
    }
    Ok(Some(schema))
}

/// Descriptor fields preserve signed counts; byte consumers require nonnegative sizes.
pub(crate) fn checked_sequence_count(count: i64) -> Result<usize, Error> {
    usize::try_from(count).map_err(|_| Error::CheckedCast)
}
