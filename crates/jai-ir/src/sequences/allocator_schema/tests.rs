use super::*;
use jai_types::*;

fn registry() -> (TypeRegistry, TypeId, AllocatorSchema) {
    let mut types = TypeRegistry::new();
    let mode = types.reserve_enum(IntegerType::S64);
    types
        .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
        .unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(types.void()).unwrap();
    let procedure = types
        .procedure(ProcedureType {
            parameters: Box::new([mode, size, size, pointer, pointer]),
            results: Box::new([pointer]),
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = types.reserve_record(RecordKind::Struct);
    types
        .define_record(allocator, [procedure, pointer])
        .unwrap();
    let schema = types.bind_allocator(allocator, mode).unwrap();
    let dynamic = types.dynamic_array(size).unwrap();
    (types, dynamic, schema)
}

struct ForeignRole<'a> {
    types: &'a TypeRegistry,
    schema: AllocatorSchema,
}
impl TypeView for ForeignRole<'_> {
    fn kind(&self, id: TypeId) -> Result<&TypeKind, TypeError> {
        self.types.kind(id)
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
    fn allocator_schema(&self) -> Option<AllocatorSchema> {
        Some(self.schema)
    }
    fn record(&self, id: RecordId) -> Result<&RecordDefinition, TypeError> {
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
fn allocator_field_uses_exact_role_and_its_owner_bound_subfields() {
    let (types, dynamic, schema) = registry();
    let mut places = crate::PlaceRegistry::new();
    let local = crate::Local::new_typed(crate::ProcedureId::new(0), 0, dynamic, &types).unwrap();
    let allocator = places
        .sequence_field(local.place(), SequenceField::Allocator, &types)
        .unwrap();
    assert_eq!(allocator.ty(), schema.ty());
    for field in [AllocatorField::Procedure, AllocatorField::Data] {
        let selected = schema.field(field);
        let place = places.field(allocator, selected.id, &types).unwrap();
        assert_eq!(place.ty(), selected.ty);
    }
}

#[test]
fn copied_role_from_another_arena_cannot_authorize_a_projection() {
    let (types, dynamic, _) = registry();
    let (_, _, foreign) = registry();
    let view = ForeignRole {
        types: &types,
        schema: foreign,
    };
    assert!(allocator_type(&view, dynamic).is_err());
    let local = crate::Local::new_typed(crate::ProcedureId::new(0), 0, dynamic, &types).unwrap();
    assert!(
        crate::PlaceRegistry::new()
            .sequence_field(local.place(), SequenceField::Allocator, &view)
            .is_err()
    );
}

#[test]
fn absent_role_and_other_sequence_kinds_cannot_guess_allocator_storage() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Int(IntegerType::S64));
    let dynamic = types.dynamic_array(element).unwrap();
    assert!(allocator_type(&types, dynamic).is_err());
    let (mut types, _, _) = registry();
    let slice = types
        .slice(types.scalar(ScalarType::Int(IntegerType::S64)))
        .unwrap();
    assert!(allocator_type(&types, slice).is_err());
    assert!(allocator_type(&types, types.string()).is_err());
}
