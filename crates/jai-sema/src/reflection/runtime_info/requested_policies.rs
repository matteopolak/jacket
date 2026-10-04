//! A query observes retained policy facts through the actual canonical registry.
use jai_types::*;
use std::collections::HashMap;

pub(super) struct RequestedPolicies<'a> {
    types: &'a TypeRegistry,
    policies: &'a HashMap<TypeId, RecordReflectionPolicy>,
}
impl<'a> RequestedPolicies<'a> {
    pub(super) fn new(
        types: &'a TypeRegistry,
        policies: &'a HashMap<TypeId, RecordReflectionPolicy>,
    ) -> Self {
        Self {
            types,
            policies,
        }
    }
}
impl TypeView for RequestedPolicies<'_> {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        self.types.kind(id)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.types.lookup(kind)
    }
    fn lookup_procedure(&self, signature: &ProcedureType) -> Option<TypeId> {
        self.types.lookup_procedure(signature)
    }
    fn scalar(&self, ty: ScalarType) -> TypeId {
        self.types.scalar(ty)
    }
    fn float(&self, ty: FloatType) -> TypeId {
        self.types.float(ty)
    }
    fn any_type(&self) -> Option<TypeId> {
        self.types.any_type()
    }
    fn runtime_type_header(&self) -> Option<TypeId> {
        self.types.runtime_type_header()
    }
    fn allocator_schema(&self) -> Option<AllocatorSchema> {
        self.types.allocator_schema()
    }
    fn record_reflection_policy(
        &self,
        record: TypeId,
    ) -> Result<RecordReflectionPolicy, TypeError> {
        let current = self.types.record_reflection_policy(record)?;
        Ok(self.policies.get(&record).copied().unwrap_or(current))
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        TypeView::record(self.types, id)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        TypeView::enumeration(self.types, id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        TypeView::distinct(self.types, id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        TypeView::procedure_type(self.types, id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        TypeView::record_type(self.types, id)
    }
}
