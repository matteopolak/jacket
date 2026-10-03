//! Validation work certified by the exact immutable StaticData publication.
use super::{StaticData, StaticDataError};
use jai_types::{
    AllocatorSchema, DistinctDefinition, DistinctId, EnumDefinition, EnumId, FloatType,
    ProcedureType, ProcedureTypeId, RecordDefinition, RecordId, RecordReflectionPolicy, ScalarType,
    TypeError, TypeId, TypeKind, TypeView,
};
use std::cell::Cell;

// runtime_type's active-path checks inspect at most 256 entries. Charging every
// registry visit by that bound also covers the bounded layout/cache bookkeeping.
const TYPE_VISIT_WORK: usize = 256;

pub(super) struct ValidationWork {
    cells: Cell<usize>,
    overflow: Cell<bool>,
}
impl ValidationWork {
    fn new() -> Self {
        Self {
            cells: Cell::new(0),
            overflow: Cell::new(false),
        }
    }
    pub(super) fn add(&self, amount: usize) -> Result<(), StaticDataError> {
        let next = self
            .cells
            .get()
            .checked_add(amount)
            .ok_or(StaticDataError::Limit("validation work"))?;
        self.cells.set(next);
        Ok(())
    }
    fn visit(&self, id: TypeId, amount: usize) -> Result<(), TypeError> {
        let result = amount
            .checked_mul(TYPE_VISIT_WORK)
            .and_then(|amount| self.cells.get().checked_add(amount));
        if self.overflow.get() || result.is_none() {
            self.overflow.set(true);
            return Err(TypeError::NotAValue(id));
        }
        self.cells.set(result.unwrap());
        Ok(())
    }
    fn plain_visit(&self) {
        if self.add(TYPE_VISIT_WORK).is_err() {
            self.overflow.set(true);
        }
    }
}

struct ValidationTypes<'a> {
    types: &'a dyn TypeView,
    work: &'a ValidationWork,
}
impl TypeView for ValidationTypes<'_> {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        self.work.visit(id, 1)?;
        self.types.kind(id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.work.plain_visit();
        self.types.lookup(kind)
    }
    fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        self.work.plain_visit();
        self.types.lookup_procedure(signature)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.work.plain_visit();
        self.types.scalar(ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        self.work.plain_visit();
        self.types.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.work.plain_visit();
        self.types.any_type()
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        self.work.plain_visit();
        self.types.runtime_type_header()
    }
    fn allocator_schema(&self) -> Option<AllocatorSchema> {
        self.work.plain_visit();
        self.types.allocator_schema()
    }
    fn record_reflection_policy(&self, id: TypeId) -> Result<RecordReflectionPolicy, TypeError> {
        self.work.visit(id, 1)?;
        self.types.record_reflection_policy(id)
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        let ty = self.types.record_type(id)?;
        self.work.visit(ty, 1)?;
        let definition = self.types.record(id)?;
        let count = definition
            .fields
            .len()
            .checked_add(definition.layout.field_alignments.len())
            .and_then(|n| n.checked_add(definition.layout.field_placements.len()))
            .and_then(|n| n.checked_mul(8));
        self.work.visit(ty, count.unwrap_or(usize::MAX))?;
        Ok(definition)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        self.work.plain_visit();
        self.types.enumeration(id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        self.work.plain_visit();
        self.types.distinct(id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        self.work.plain_visit();
        self.types.procedure_type(id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        self.work.plain_visit();
        self.types.record_type(id)
    }
    fn code_type(&self) -> TypeId {
        self.work.plain_visit();
        self.types.code_type()
    }
    fn meta_type(&self) -> TypeId {
        self.work.plain_visit();
        self.types.meta_type()
    }
}

pub(super) fn validate_and_measure(
    data: &StaticData,
    types: &dyn TypeView,
) -> Result<usize, StaticDataError> {
    let work = ValidationWork::new();
    let measured = ValidationTypes {
        types,
        work: &work,
    };
    let result = data.validate_inner(&measured, Some(&work));
    if work.overflow.get() {
        return Err(StaticDataError::Limit("validation work"));
    }
    result?;
    Ok(work.cells.get())
}
