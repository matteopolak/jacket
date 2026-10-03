use super::*;
use jai_types::*;
use std::cell::Cell;

struct Counted<'a> {
    types: &'a TypeRegistry,
    records: Cell<usize>,
}
impl TypeView for Counted<'_> {
    fn kind(&self, ty: TypeId) -> Result<&TypeKind, TypeError> {
        self.types.kind(ty)
    }
    fn lookup(&self, kind: &TypeKind) -> Option<TypeId> {
        self.types.lookup(kind)
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
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
        self.records.set(self.records.get() + 1);
        self.types.record(id)
    }
    fn enumeration(&self, id: EnumId) -> Result<&EnumDefinition, TypeError> {
        self.types.enumeration(id)
    }
    fn distinct(&self, id: DistinctId) -> Result<&DistinctDefinition, TypeError> {
        self.types.distinct(id)
    }
    fn procedure_type(&self, id: ProcedureTypeId) -> Result<&ProcedureType, TypeError> {
        self.types.procedure_type(id)
    }
    fn record_type(&self, id: RecordId) -> Result<TypeId, TypeError> {
        self.types.record_type(id)
    }
}

#[test]
fn warm_swap_preflight_uses_admitted_shape_without_rescanning_wide_records() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let wide = types.reserve_record(RecordKind::Struct);
    types.define_record(wide, vec![word; 2048]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let first = memory.allocate(&types, wide, None).unwrap();
    let second = memory.allocate(&types, wide, None).unwrap();
    let expected = memory.swap_work_cost(&types, &first, &second).unwrap();
    assert!(expected >= 2 * 2049);
    let counted = Counted {
        types: &types,
        records: Cell::new(0),
    };
    for _ in 0..20 {
        assert_eq!(
            memory.swap_work_cost(&counted, &first, &second).unwrap(),
            expected
        );
    }
    assert_eq!(counted.records.get(), 0);
}
