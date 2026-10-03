use super::*;
use jai_types::{
    CastMode, FloatType, FloatValue, Integer, IntegerType, RecordKind, ScalarType, TypeRegistry,
};

fn integer(ty: IntegerType, bits: u64) -> Value {
    Value::Int(Integer::wrapping(ty, i128::from(bits)))
}

#[test]
fn byte_cast_arithmetic_updates_original_integer_and_float_views() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let float = types.float(FloatType::F64);
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            word,
            Some(integer(IntegerType::U64, 0x1122334455667788)),
        )
        .unwrap();
    let bytes = memory
        .cast_pointer(&types, &root, byte, CastMode::Checked)
        .unwrap();
    let second = memory.offset(&types, &bytes, 1).unwrap();
    assert_eq!(
        memory.load(&types, &second).unwrap(),
        integer(IntegerType::U8, 0x77)
    );
    memory
        .store(&types, &second, integer(IntegerType::U8, 0xaa))
        .unwrap();
    assert_eq!(
        memory.load(&types, &root).unwrap(),
        integer(IntegerType::U64, 0x112233445566aa88)
    );
    let float_pointer = memory
        .cast_pointer(&types, &root, float, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory.load(&types, &float_pointer).unwrap(),
        Value::Float(FloatValue::F64(0x112233445566aa88))
    );
    let end = memory.offset(&types, &bytes, 8).unwrap();
    assert_eq!(memory.distance(&types, &end, &bytes).unwrap(), 8);
    assert!(matches!(
        memory.load(&types, &end),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        memory.validate_slice(&types, &bytes, 9),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        memory.validate_slice(&types, &end, 1),
        Err(Error::OutOfBounds { .. })
    ));
}

#[test]
fn addresses_use_target_layout_offsets_including_packed_fields_and_union_aliases() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let packed = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            packed,
            [byte, word],
            jai_types::RecordLayout {
                packed: true,
                ..Default::default()
            },
        )
        .unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [byte, word]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            packed,
            Some(Value::Record {
                ty: packed,
                fields: vec![integer(IntegerType::U8, 1), integer(IntegerType::U64, 2)],
            }),
        )
        .unwrap();
    let first = memory.field(&types, &root, 0).unwrap();
    let second = memory.field(&types, &root, 1).unwrap();
    let root_bytes = memory
        .cast_pointer(&types, &root, byte, CastMode::Checked)
        .unwrap();
    let second_bytes = memory
        .cast_pointer(&types, &second, byte, CastMode::Checked)
        .unwrap();
    assert!(memory.same_address(&types, &root, &first).unwrap());
    assert!(
        memory
            .same_address(
                &types,
                &memory.offset(&types, &root_bytes, 1).unwrap(),
                &second_bytes
            )
            .unwrap()
    );
    let root = memory
        .allocate(
            &types,
            union,
            Some(Value::Union {
                ty: union,
                field: 1,
                value: Box::new(integer(IntegerType::U64, 0x1020304050607080)),
            }),
        )
        .unwrap();
    let narrow = memory.field(&types, &root, 0).unwrap();
    let wide = memory.field(&types, &root, 1).unwrap();
    assert!(memory.same_address(&types, &narrow, &wide).unwrap());
    assert_eq!(
        memory.load(&types, &narrow).unwrap(),
        integer(IntegerType::U8, 0x80)
    );
    memory
        .store(&types, &narrow, integer(IntegerType::U8, 0xab))
        .unwrap();
    assert_eq!(
        memory.load(&types, &wide).unwrap(),
        integer(IntegerType::U64, 0x10203040506070ab)
    );
    assert_eq!(
        memory.load(&types, &root).unwrap().semantic(),
        &Value::Union {
            ty: union,
            field: 0,
            value: Box::new(integer(IntegerType::U8, 0xab))
        }
    );
}

#[test]
fn pointer_slot_views_preserve_identity_and_project_using_the_reinterpreted_record() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let original = types.reserve_record(RecordKind::Struct);
    types.define_record(original, [word, word]).unwrap();
    let view = types.reserve_record(RecordKind::Struct);
    types.define_record(view, [word, word]).unwrap();
    let original_ptr = types.pointer(original).unwrap();
    let view_ptr = types.pointer(view).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            original,
            Some(Value::Record {
                ty: original,
                fields: vec![integer(IntegerType::U64, 4), integer(IntegerType::U64, 9)],
            }),
        )
        .unwrap();
    let slot = memory
        .allocate(&types, original_ptr, Some(Value::Pointer(root.clone())))
        .unwrap();
    let slot_view = memory
        .cast_pointer(&types, &slot, view_ptr, CastMode::Unchecked)
        .unwrap();
    let pointed = memory
        .load(&types, &slot_view)
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    assert!(memory.same_address(&types, &root, &pointed).unwrap());
    let field = memory.field(&types, &pointed, 1).unwrap();
    memory
        .store(&types, &field, integer(IntegerType::U64, 11))
        .unwrap();
    assert_eq!(
        memory
            .load(&types, &memory.field(&types, &root, 1).unwrap())
            .unwrap(),
        integer(IntegerType::U64, 11)
    );
}

#[test]
fn reinterpreted_projections_cannot_escape_an_inherited_field_or_descriptor_region() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let large = types.reserve_record(RecordKind::Struct);
    types.define_record(large, [word, word]).unwrap();
    let unrelated = types.reserve_record(RecordKind::Struct);
    types.define_record(unrelated, [word, word]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            large,
            Some(Value::Record {
                ty: large,
                fields: vec![integer(IntegerType::U64, 1), integer(IntegerType::U64, 2)],
            }),
        )
        .unwrap();
    let first = memory.field(&types, &root, 0).unwrap();
    let owner = memory
        .cast_pointer(&types, &first, large, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory
            .load(&types, &memory.field(&types, &owner, 1).unwrap())
            .unwrap(),
        integer(IntegerType::U64, 2)
    );
    let invalid = memory
        .cast_pointer(&types, &first, unrelated, CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.field(&types, &invalid, 1),
        Err(Error::OutOfBounds { .. })
    ));
    let two = types.fixed_array(word, 2).unwrap();
    let invalid = memory
        .cast_pointer(&types, &first, two, CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.index(&types, &invalid, 1),
        Err(Error::OutOfBounds { .. })
    ));
    let slice = types.slice(word).unwrap();
    types.pointer(word).unwrap();
    let descriptor = memory
        .allocate(
            &types,
            slice,
            Some(Value::Slice {
                ty: slice,
                pointer: Pointer::null(word),
                count: 0,
            }),
        )
        .unwrap();
    let count = memory
        .sequence_field(&types, &descriptor, jai_ir::SequenceField::Count)
        .unwrap();
    let end = memory.offset(&types, &count, 1).unwrap();
    assert!(matches!(
        memory.load(&types, &end),
        Err(Error::OutOfBounds { .. })
    ));
}

#[test]
fn byte_images_follow_configured_endianness_and_rollback_restores_aliases() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let mut memory = Memory::with_target(
        Limits::default(),
        ByteTarget {
            policy: jai_types::LayoutPolicy::lp64(),
            endian: Endian::Big,
        },
    );
    let root = memory
        .allocate(
            &types,
            word,
            Some(integer(IntegerType::U64, 0x1122334455667788)),
        )
        .unwrap();
    let bytes = memory
        .cast_pointer(&types, &root, byte, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory.load(&types, &bytes).unwrap(),
        integer(IntegerType::U8, 0x11)
    );
    let snapshot = memory.snapshot();
    memory
        .store(&types, &bytes, integer(IntegerType::U8, 0xaa))
        .unwrap();
    assert_eq!(
        memory.load(&types, &root).unwrap(),
        integer(IntegerType::U64, 0xaa22334455667788)
    );
    memory.restore(snapshot);
    assert_eq!(
        memory.load(&types, &root).unwrap(),
        integer(IntegerType::U64, 0x1122334455667788)
    );
}

#[test]
fn byte_writes_cannot_forge_a_virtual_pointer_and_failed_writes_are_atomic() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = types.pointer(byte).unwrap();
    let mut memory = Memory::new(Limits::default());
    let target = memory
        .allocate(&types, byte, Some(integer(IntegerType::U8, 42)))
        .unwrap();
    let slot = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(target.clone())))
        .unwrap();
    let bytes = memory
        .cast_pointer(&types, &slot, byte, CastMode::Unchecked)
        .unwrap();
    let end = memory.offset(&types, &bytes, 8).unwrap();
    assert!(matches!(
        memory.store(&types, &end, integer(IntegerType::U8, 1)),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(
        memory.load(&types, &slot).unwrap(),
        Value::Pointer(target.clone())
    );
    memory
        .store(&types, &bytes, integer(IntegerType::U8, 0xff))
        .unwrap();
    assert!(matches!(
        memory.load(&types, &slot),
        Err(Error::InvalidIr(_) | Error::UnsupportedPointerOperation(_))
    ));
    memory
        .store(&types, &slot, Value::Pointer(target.clone()))
        .unwrap();
    assert_eq!(memory.load(&types, &slot).unwrap(), Value::Pointer(target));
}

#[test]
fn boolean_byte_alias_reads_the_same_low_bit_as_native_i1_storage() {
    let types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(&types, boolean, Some(Value::Bool(false)))
        .unwrap();
    let bytes = memory
        .cast_pointer(&types, &root, byte, CastMode::Checked)
        .unwrap();
    for (bits, expected) in [(42, false), (43, true), (254, false), (255, true)] {
        memory
            .store(&types, &bytes, integer(IntegerType::U8, bits))
            .unwrap();
        assert_eq!(memory.load(&types, &root).unwrap(), Value::Bool(expected));
        assert_eq!(
            memory.load(&types, &bytes).unwrap(),
            integer(IntegerType::U8, bits)
        );
    }
}
