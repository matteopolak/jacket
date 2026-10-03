//! Checked source storage for Compiler.Runtime_Info and its global-data catalog.
use crate::{
    FieldDescriptor, IntegerType, Layout, LayoutEngine, LayoutError, LayoutPolicy, RecordKind,
    RuntimeTypeSchema, TypeError, TypeId, TypeKind, TypeView,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeInfoField {
    TypeTable,
    GlobalDataInfo,
}

/// The semantic source binder supplies the adopted nominal identities. This
/// proof checks their exact storage relationship; it does not adopt names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeInfoSchema {
    ty: TypeId,
    runtime_type: RuntimeTypeSchema,
    table: FieldDescriptor,
    global: FieldDescriptor,
    global_data: TypeId,
    segment: TypeId,
}

#[derive(Debug)]
pub enum RuntimeInfoError {
    Type(TypeError),
    Layout(LayoutError),
    InvalidRecord(TypeId),
    InvalidTypeTable(TypeId),
    InvalidGlobalData(TypeId),
}
impl From<TypeError> for RuntimeInfoError {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}
impl From<LayoutError> for RuntimeInfoError {
    fn from(value: LayoutError) -> Self {
        Self::Layout(value)
    }
}
impl fmt::Display for RuntimeInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(f),
            Self::Layout(error) => error.fmt(f),
            Self::InvalidRecord(_) => f.write_str("Runtime_Info must be a two-field source struct"),
            Self::InvalidTypeTable(_) => f.write_str(
                "Runtime_Info.type_table must be a slice of canonical Type_Info pointers",
            ),
            Self::InvalidGlobalData(_) => f.write_str(
                "Runtime_Info global-data catalog does not match the source storage contract",
            ),
        }
    }
}
impl std::error::Error for RuntimeInfoError {}

impl RuntimeInfoSchema {
    pub fn validate(
        types: &dyn TypeView,
        ty: TypeId,
        global_data: TypeId,
    ) -> Result<Self, RuntimeInfoError> {
        let runtime_type = RuntimeTypeSchema::from_view(types)?;
        let record = types.record_definition(ty)?;
        if record.kind != RecordKind::Struct || record.fields.len() != 2 {
            return Err(RuntimeInfoError::InvalidRecord(ty));
        }
        let table = types.field(ty, 0)?;
        let global = types.field(ty, 1)?;
        if *types.kind(table.ty)? != TypeKind::Slice(runtime_type.descriptor_type()) {
            return Err(RuntimeInfoError::InvalidTypeTable(table.ty));
        }
        if *types.kind(global.ty)? != TypeKind::Pointer(global_data) {
            return Err(RuntimeInfoError::InvalidGlobalData(global.ty));
        }
        let data = types.record_definition(global_data)?;
        if data.kind != RecordKind::Struct
            || data.fields.len() != 2
            || *types.kind(data.fields[0])? != TypeKind::Integer(IntegerType::U64)
        {
            return Err(RuntimeInfoError::InvalidGlobalData(global_data));
        }
        let TypeKind::Slice(segment) = *types.kind(data.fields[1])? else {
            return Err(RuntimeInfoError::InvalidGlobalData(global_data));
        };
        let segment_record = types.record_definition(segment)?;
        if segment_record.kind != RecordKind::Struct || segment_record.fields.len() != 2 {
            return Err(RuntimeInfoError::InvalidGlobalData(segment));
        }
        let TypeKind::Enum(tag) = *types.kind(segment_record.fields[0])? else {
            return Err(RuntimeInfoError::InvalidGlobalData(segment));
        };
        let tag = types.enumeration(tag)?;
        if tag.representation != IntegerType::U16
            || tag.values.len() != 5
            || !tag
                .values
                .iter()
                .map(|value| value.value())
                .eq([0, 1, 2, 3, 5])
        {
            return Err(RuntimeInfoError::InvalidGlobalData(segment));
        }
        let TypeKind::Slice(byte) = *types.kind(segment_record.fields[1])? else {
            return Err(RuntimeInfoError::InvalidGlobalData(segment));
        };
        if *types.kind(byte)? != TypeKind::Integer(IntegerType::U8) {
            return Err(RuntimeInfoError::InvalidGlobalData(segment));
        }
        Ok(Self {
            ty,
            runtime_type,
            table,
            global,
            global_data,
            segment,
        })
    }

    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn runtime_type_schema(self) -> RuntimeTypeSchema {
        self.runtime_type
    }
    pub fn global_data_type(self) -> TypeId {
        self.global_data
    }
    pub fn global_data_segment_type(self) -> TypeId {
        self.segment
    }
    pub fn field(self, field: RuntimeInfoField) -> FieldDescriptor {
        match field {
            RuntimeInfoField::TypeTable => self.table,
            RuntimeInfoField::GlobalDataInfo => self.global,
        }
    }
    pub fn revalidate(self, types: &dyn TypeView) -> Result<(), RuntimeInfoError> {
        self.runtime_type.validate(types)?;
        if Self::validate(types, self.ty, self.global_data)? != self {
            return Err(RuntimeInfoError::InvalidRecord(self.ty));
        }
        Ok(())
    }
    pub fn layout(
        self,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<Layout, RuntimeInfoError> {
        self.revalidate(types)?;
        let mut engine = LayoutEngine::new(types, policy);
        // The global pointer does not depend on pointee layout. Check the actual
        // source catalog records too, so invalid field layout cannot hide there.
        engine.layout(self.global_data)?;
        engine.layout(self.segment)?;
        Ok(engine.layout(self.ty)?.clone())
    }
}

#[cfg(test)]
mod tests;
