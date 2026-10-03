use super::*;
use jai_types::{
    AllocatorField, AllocatorMode, AllocatorSchema, CallingConvention, ContextMode, ProcedureType,
    RecordKind, TypeRegistry, Variadic,
};

fn fixture() -> (TypeRegistry, AllocatorSchema, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let dynamic = types.dynamic_array(byte).unwrap();
    let mode = types.reserve_enum(IntegerType::S64);
    types
        .define_enum(mode, AllocatorMode::ALL.map(AllocatorMode::value))
        .unwrap();
    let data = types.pointer(types.void()).unwrap();
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    let procedure = types
        .procedure(ProcedureType {
            parameters: Box::new([mode, size, size, data, data]),
            results: Box::new([data]),
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let allocator = types.reserve_record(RecordKind::Struct);
    types.define_record(allocator, [procedure, data]).unwrap();
    let schema = types.bind_allocator(allocator, mode).unwrap();
    (types, schema, byte, dynamic)
}

#[test]
fn allocator_projection_aliases_exact_slots_and_preserves_subregion() {
    let (types, schema, byte, dynamic) = fixture();
    let signature = schema.field(AllocatorField::Procedure).ty;
    let procedure = jai_ir::ProcedurePrototype {
        id: jai_ir::ProcedureId::new(17),
        signature,
        origin: jai_ir::PrototypeOrigin::SourceContract {
            symbol: "memory_allocator_fixture".into(),
        },
    };
    let mut memory = Memory::new(Limits::default());
    let payload = Value::Record {
        ty: schema.ty(),
        fields: vec![
            Value::Procedure {
                signature,
                procedure: Some(procedure.id),
            },
            Value::Pointer(Pointer::null(types.void())),
        ],
    };
    let value = Value::DynamicArray {
        ty: dynamic,
        pointer: Pointer::null(byte),
        count: -1,
        allocated: -2,
        allocator: Some(Box::new(payload.clone())),
    };
    let root = memory
        .allocate(&types, dynamic, Some(value.clone()))
        .unwrap();
    let allocator = memory.sequence_allocator(&types, &root).unwrap();
    assert_eq!(memory.load(&types, &allocator).unwrap(), payload);
    let procedure = memory
        .field(
            &types,
            &allocator,
            schema.field(AllocatorField::Procedure).id.index(),
        )
        .unwrap();
    let code = memory.load(&types, &procedure).unwrap();
    let code_view = memory
        .cast_pointer(
            &types,
            &procedure,
            schema.field(AllocatorField::Data).ty,
            CastMode::Checked,
        )
        .unwrap();
    assert!(
        memory
            .load(&types, &code_view)
            .unwrap()
            .pointer()
            .unwrap()
            .code_pointer()
            .is_some()
    );
    let replacement = Value::Procedure {
        signature,
        procedure: None,
    };
    memory
        .store(&types, &procedure, replacement.clone())
        .unwrap();
    assert_eq!(memory.load(&types, &procedure).unwrap(), replacement);
    assert_ne!(code, replacement);
    let raw = memory
        .cast_pointer(&types, &allocator, byte, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory.region(&types, &raw).unwrap(),
        memory.region(&types, &allocator).unwrap()
    );
    assert!(memory.offset(&types, &raw, -1).is_err());
    let end = memory.offset(&types, &raw, 16).unwrap();
    assert!(memory.load(&types, &end).is_err());
    memory.freeze(&root).unwrap();
    assert!(matches!(
        memory.store(&types, &procedure, code),
        Err(Error::ReadOnlyStorage)
    ));
}

#[test]
fn no_role_projection_and_wrong_descriptor_type_are_rejected() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let dynamic = types.dynamic_array(byte).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            dynamic,
            Some(Value::DynamicArray {
                ty: dynamic,
                pointer: Pointer::null(byte),
                count: 0,
                allocated: 0,
                allocator: None,
            }),
        )
        .unwrap();
    assert!(matches!(
        memory.sequence_allocator(&types, &root),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let scalar = memory
        .allocate(
            &types,
            byte,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 0))),
        )
        .unwrap();
    assert!(matches!(
        memory.sequence_allocator(&types, &scalar),
        Err(Error::UnsupportedType(_))
    ));
}

#[test]
fn undefined_allocator_slot_is_copyable_without_becoming_readable() {
    let (types, schema, byte, dynamic) = fixture();
    let mut memory = Memory::new(Limits::default());
    let target = memory.target();
    let signature = schema.field(AllocatorField::Procedure).ty;
    let mut image = ByteImage::uninitialized(target, 16, 1024).unwrap();
    image
        .write(
            &types,
            target,
            0,
            signature,
            &Value::Procedure {
                signature,
                procedure: None,
            },
        )
        .unwrap();
    let payload = image
        .read_partial_preserving_with_limit(&types, target, 0, schema.ty(), 1024)
        .unwrap();
    let descriptor = Value::DynamicArray {
        ty: dynamic,
        pointer: Pointer::null(byte),
        count: 0,
        allocated: 0,
        allocator: Some(Box::new(payload)),
    };
    let source = memory.allocate(&types, dynamic, Some(descriptor)).unwrap();
    let allocator = memory.sequence_allocator(&types, &source).unwrap();
    let copy = memory.load(&types, &allocator).unwrap();
    assert!(
        matches!(&copy, Value::StoredAggregate(snapshot) if snapshot.decoded_semantic().is_none())
    );
    let data = memory
        .field(
            &types,
            &allocator,
            schema.field(AllocatorField::Data).id.index(),
        )
        .unwrap();
    assert!(matches!(
        memory.load(&types, &data),
        Err(Error::Uninitialized)
    ));
    let destination = memory
        .allocate(
            &types,
            dynamic,
            Some(crate::constants::zero(&types, dynamic, Limits::default()).unwrap()),
        )
        .unwrap();
    let dest_allocator = memory.sequence_allocator(&types, &destination).unwrap();
    memory.store(&types, &dest_allocator, copy).unwrap();
    let copied_data = memory
        .field(
            &types,
            &dest_allocator,
            schema.field(AllocatorField::Data).id.index(),
        )
        .unwrap();
    assert!(matches!(
        memory.load(&types, &copied_data),
        Err(Error::Uninitialized)
    ));
    let Value::DynamicArray {
        allocator: Some(payload),
        ..
    } = memory.load(&types, &destination).unwrap()
    else {
        panic!("copy requires the typed payload");
    };
    let Value::StoredAggregate(snapshot) = payload.as_ref() else {
        panic!("undefined slot requires a carrier")
    };
    assert_eq!(snapshot.image(), &image);
}
