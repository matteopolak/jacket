use super::*;
use crate::{Limits, Memory, Pointer};
use jai_types::{RecordLayout, TypeRegistry};

fn int(ty: IntegerType, bits: i128) -> Value {
    Value::Int(Integer::wrapping(ty, bits))
}
fn pointer_target(width: u64, endian: Endian) -> ByteTarget {
    use jai_types::ScalarLayout;
    ByteTarget {
        policy: LayoutPolicy::new(
            ScalarLayout::new(width, width as u32),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
        endian,
    }
}

#[test]
fn native_pointer_capsules_normalize_before_byte_encoding_without_authority() {
    use jai_ir::NativePointerConstant;
    use jai_types::CastMode;
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = types.pointer(byte).unwrap();
    for width in [4u64, 8] {
        let target = pointer_target(width, Endian::Big);
        let capsule = NativePointerConstant::new(
            pointer_ty,
            Integer::wrapping(IntegerType::U64, 0),
            CastMode::Checked,
            &types,
        )
        .unwrap();
        let value = crate::constants::native_pointer(&types, &capsule, target).unwrap();
        assert_eq!(value, Value::Pointer(Pointer::null(byte)));
        let image = ByteImage::encode(&types, target, pointer_ty, &value, 128).unwrap();
        assert_eq!(image.bytes(), vec![0; width as usize]);
        assert_eq!(image.metadata_cells(), 0);
        assert_eq!(image.read(&types, target, 0, pointer_ty).unwrap(), value);
        let unknown = NativePointerConstant::new(
            pointer_ty,
            Integer::wrapping(IntegerType::U64, 1),
            CastMode::Unchecked,
            &types,
        )
        .unwrap();
        let numeric = crate::constants::native_pointer(&types, &unknown, target).unwrap();
        assert_eq!(
            numeric.pointer().unwrap().opaque_address_bits(),
            Some((1, width as u32 * 8))
        );
        assert_eq!(numeric.pointer().unwrap().data_allocation_key(), None);
    }
}

#[test]
fn native_pointer_truncation_can_be_null_only_on_selected_narrow_target() {
    use jai_ir::NativePointerConstant;
    use jai_types::CastMode;
    let mut types = TypeRegistry::new();
    let pointer_ty = types.pointer(types.void()).unwrap();
    let narrow = pointer_target(4, Endian::Little);
    let source = Integer::wrapping(IntegerType::U64, 1i128 << 32);
    let checked =
        NativePointerConstant::new(pointer_ty, source, CastMode::Checked, &types).unwrap();
    assert!(matches!(
        crate::constants::native_pointer(&types, &checked, narrow),
        Err(Error::CheckedCast)
    ));
    for mode in [CastMode::Unchecked, CastMode::Truncate] {
        let capsule = NativePointerConstant::new(pointer_ty, source, mode, &types).unwrap();
        let null = crate::constants::native_pointer(&types, &capsule, narrow).unwrap();
        assert!(null.pointer().unwrap().is_null());
        let image = ByteImage::encode(&types, narrow, pointer_ty, &null, 128).unwrap();
        assert_eq!(image.bytes(), &[0; 4]);
        assert_eq!(
            crate::constants::native_pointer(&types, &capsule, ByteTarget::default())
                .unwrap()
                .pointer()
                .unwrap()
                .opaque_address_bits(),
            Some((1u64 << 32, 64))
        );
    }
}

#[test]
fn scalar_bytes_use_target_endian_and_alias_writes_preserve_other_bytes() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let little = ByteTarget::default();
    let mut image = ByteImage::encode(
        &types,
        little,
        word,
        &int(IntegerType::U64, 0x1122334455667788),
        64,
    )
    .unwrap();
    assert_eq!(
        image.bytes(),
        &[0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11]
    );
    image
        .write(&types, little, 2, byte, &int(IntegerType::U8, 0xaa))
        .unwrap();
    assert_eq!(
        image.read(&types, little, 0, word).unwrap(),
        int(IntegerType::U64, 0x1122334455aa7788)
    );
    let big = ByteTarget {
        endian: Endian::Big,
        ..little
    };
    let image = ByteImage::encode(
        &types,
        big,
        word,
        &int(IntegerType::U64, 0x1122334455667788),
        64,
    )
    .unwrap();
    assert_eq!(
        image.bytes(),
        &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]
    );
    assert!(matches!(
        image.read(&types, little, 0, word),
        Err(Error::InvalidIr(_))
    ));
}

#[test]
fn array_record_padding_packing_and_union_views_share_layout_offsets() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let array = types.fixed_array(word, 2).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [byte, array, byte]).unwrap();
    let packed = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_layout(
            packed,
            [byte, word],
            RecordLayout {
                packed: true,
                ..RecordLayout::default()
            },
        )
        .unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [word, array]).unwrap();
    let target = ByteTarget::default();
    let mut image = ByteImage::encode(
        &types,
        target,
        record,
        &Value::Record {
            ty: record,
            fields: vec![
                int(IntegerType::U8, 7),
                Value::Array {
                    ty: array,
                    elements: vec![int(IntegerType::U32, 10), int(IntegerType::U32, 20)],
                },
                int(IntegerType::U8, 9),
            ],
        },
        128,
    )
    .unwrap();
    assert_eq!(image.len(), 16);
    assert_eq!(&image.bytes()[1..4], &[0; 3]);
    image
        .write(&types, target, 8, word, &int(IntegerType::U32, 30))
        .unwrap();
    let Value::Record { fields, .. } = image.read(&types, target, 0, record).unwrap() else {
        panic!()
    };
    assert_eq!(
        fields[1],
        Value::Array {
            ty: array,
            elements: vec![int(IntegerType::U32, 10), int(IntegerType::U32, 30)]
        }
    );
    let image = ByteImage::encode(
        &types,
        target,
        packed,
        &Value::Record {
            ty: packed,
            fields: vec![int(IntegerType::U8, 1), int(IntegerType::U32, 0x12345678)],
        },
        128,
    )
    .unwrap();
    assert_eq!(image.bytes(), &[1, 0x78, 0x56, 0x34, 0x12]);
    let mut image = ByteImage::encode(
        &types,
        target,
        union,
        &Value::Union {
            ty: union,
            field: 1,
            value: Box::new(Value::Array {
                ty: array,
                elements: vec![int(IntegerType::U32, 4), int(IntegerType::U32, 5)],
            }),
        },
        128,
    )
    .unwrap();
    assert_eq!(
        image.read_union_field(&types, 0, union, 0).unwrap(),
        int(IntegerType::U32, 4)
    );
    image
        .write(&types, target, 0, word, &int(IntegerType::U32, 99))
        .unwrap();
    assert_eq!(
        image.read_union_field(&types, 0, union, 1).unwrap(),
        Value::Array {
            ty: array,
            elements: vec![int(IntegerType::U32, 99), int(IntegerType::U32, 5)]
        }
    );
    image.note_union_field(&types, 0, union, 0).unwrap();
    assert_eq!(
        image.read(&types, target, 0, union).unwrap(),
        Value::Union {
            ty: union,
            field: 0,
            value: Box::new(int(IntegerType::U32, 99)),
        }
    );
}

#[test]
fn virtual_pointer_provenance_survives_whole_handle_writes_and_rejects_byte_forgery() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = types.pointer(word).unwrap();
    let void = types.void();
    let void_pointer_ty = types.pointer(void).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory
        .allocate(&types, word, Some(int(IntegerType::U64, 4)))
        .unwrap();
    let target = ByteTarget::default();
    let original = Value::Pointer(pointer);
    let mut image = ByteImage::encode(&types, target, pointer_ty, &original, 64).unwrap();
    assert_eq!(image.read(&types, target, 0, pointer_ty).unwrap(), original);
    let Value::Pointer(retyped) = image.read(&types, target, 0, void_pointer_ty).unwrap() else {
        panic!()
    };
    assert_eq!(retyped.pointee(), void);
    assert!(
        memory
            .same_address(&types, original.pointer().unwrap(), &retyped)
            .unwrap()
    );
    // Even writing the exact same visible byte destroys the opaque relocation.
    let current = image.bytes()[0];
    image
        .write(
            &types,
            target,
            0,
            byte,
            &int(IntegerType::U8, i128::from(current)),
        )
        .unwrap();
    assert!(matches!(
        image.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    image
        .write(&types, target, 0, pointer_ty, &original)
        .unwrap();
    assert_eq!(image.read(&types, target, 0, pointer_ty).unwrap(), original);
    image
        .write(&types, target, 0, word, &int(IntegerType::U64, 0))
        .unwrap();
    assert_eq!(
        image.read(&types, target, 0, pointer_ty).unwrap(),
        Value::Pointer(Pointer::null(word))
    );
}

#[test]
fn failed_writes_are_atomic_and_float_views_keep_nan_bits() {
    let types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let float = types.float(FloatType::F64);
    let target = ByteTarget::default();
    let mut image = ByteImage::encode(
        &types,
        target,
        integer,
        &int(IntegerType::U64, 0x7ff8000000000123),
        64,
    )
    .unwrap();
    assert_eq!(
        image.read(&types, target, 0, float).unwrap(),
        Value::Float(FloatValue::F64(0x7ff8000000000123))
    );
    let before = image.bytes().to_vec();
    assert!(matches!(
        image.write(&types, target, 4, integer, &int(IntegerType::U64, 0)),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(image.bytes(), before);
    assert!(matches!(
        ByteImage::encode(&types, target, integer, &int(IntegerType::U64, 0), 4),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn descriptor_relocations_and_counts_round_trip_without_host_addresses() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    types.pointer(byte).unwrap();
    let array = types.fixed_array(byte, 3).unwrap();
    let slice = types.slice(byte).unwrap();
    let dynamic = types.dynamic_array(byte).unwrap();
    let string = types.string();
    let mut memory = Memory::new(Limits::default());
    let backing = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![
                    int(IntegerType::U8, 1),
                    int(IntegerType::U8, 2),
                    int(IntegerType::U8, 3),
                ],
            }),
        )
        .unwrap();
    let pointer = memory.index(&types, &backing, 0).unwrap();
    let target = ByteTarget::default();
    for (ty, value) in [
        (
            slice,
            Value::Slice {
                ty: slice,
                pointer: pointer.clone(),
                count: 3,
            },
        ),
        (
            dynamic,
            Value::DynamicArray {
                ty: dynamic,
                pointer: pointer.clone(),
                count: 2,
                allocated: 3,
                allocator: None,
            },
        ),
        (string, Value::StringView { pointer, count: 3 }),
    ] {
        let image = ByteImage::encode(&types, target, ty, &value, 128).unwrap();
        assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
    }
    assert!(matches!(
        ByteImage::encode(
            &types,
            target,
            string,
            &Value::String(b"owned".to_vec()),
            128
        ),
        Err(Error::InvalidIr(_))
    ));
}

#[test]
fn zero_sized_nested_arrays_charge_total_materialized_values() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let inner = types.fixed_array(empty, 8).unwrap();
    let outer = types.fixed_array(inner, 8).unwrap();
    let seed = types.fixed_array(empty, 0).unwrap();
    let target = ByteTarget::default();
    let image = ByteImage::encode(
        &types,
        target,
        seed,
        &Value::Array {
            ty: seed,
            elements: vec![],
        },
        16,
    )
    .unwrap();
    assert_eq!(image.len(), 0);
    // Every individual array fits the budget, but the aggregate does not.
    assert!(matches!(
        image.read(&types, target, 0, outer),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
    let value = Value::Array {
        ty: outer,
        elements: (0..8)
            .map(|_| Value::Array {
                ty: inner,
                elements: (0..8)
                    .map(|_| Value::Record {
                        ty: empty,
                        fields: vec![],
                    })
                    .collect(),
            })
            .collect(),
    };
    assert!(matches!(
        ByteImage::encode(&types, target, outer, &value, 16),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn procedure_relocations_reject_data_handles_and_byte_reconstruction() {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: jai_types::CallingConvention::Jai,
            context: jai_types::ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = types.pointer(byte).unwrap();
    let procedure = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(41)),
    };
    let target = ByteTarget::default();
    let mut image = ByteImage::encode(&types, target, signature, &procedure, 64).unwrap();
    assert_eq!(image.read(&types, target, 0, signature).unwrap(), procedure);
    assert!(matches!(
        image.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(
            "unowned procedure bytes cannot construct a code pointer"
        ))
    ));
    let byte_value = image.bytes()[0];
    image
        .write(
            &types,
            target,
            0,
            byte,
            &int(IntegerType::U8, i128::from(byte_value)),
        )
        .unwrap();
    assert!(matches!(
        image.read(&types, target, 0, signature),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let null = Value::Procedure {
        signature,
        procedure: None,
    };
    image.write(&types, target, 0, signature, &null).unwrap();
    assert_eq!(image.read(&types, target, 0, signature).unwrap(), null);
}

#[test]
fn raw_backing_bytes_allow_scalar_views_without_inventing_pointer_provenance() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_ty = types.pointer(byte).unwrap();
    let target = ByteTarget::default();
    let image = ByteImage::from_bytes(target, vec![1, 2, 3, 4, 5, 6, 7, 8], 8).unwrap();
    assert_eq!(
        image.read(&types, target, 0, word).unwrap(),
        int(IntegerType::U32, 0x04030201)
    );
    let numeric = image.read(&types, target, 0, pointer_ty).unwrap();
    assert_eq!(
        numeric.pointer().unwrap().opaque_address_bits(),
        Some((0x0807060504030201, 64))
    );
    assert_eq!(numeric.pointer().unwrap().data_allocation_key(), None);
    assert!(matches!(
        ByteImage::from_bytes(target, vec![0; 9], 8),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn narrow_target_descriptor_offsets_use_target_pointer_size() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    types.pointer(byte).unwrap();
    let string = types.string();
    let policy = LayoutPolicy::new(
        jai_types::ScalarLayout::new(4, 4),
        [
            jai_types::ScalarLayout::new(1, 1),
            jai_types::ScalarLayout::new(2, 2),
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        [
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        jai_types::ScalarLayout::new(1, 1),
    )
    .unwrap();
    let target = ByteTarget {
        policy,
        endian: Endian::Big,
    };
    let mut memory = Memory::new(Limits::default());
    let pointer = memory
        .allocate(&types, byte, Some(int(IntegerType::U8, 65)))
        .unwrap();
    let value = Value::StringView { pointer, count: 1 };
    let image = ByteImage::encode(&types, target, string, &value, 64).unwrap();
    assert_eq!(image.len(), 12);
    assert_eq!(&image.bytes()[..8], &[0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(image.read(&types, target, 0, string).unwrap(), value);
}

#[test]
fn build_target_conversion_preserves_explicit_storage_facts() {
    let profile = jai_types::BuildTarget {
        operating_system: jai_types::OperatingSystem::Other("independent-test-target".into()),
        architecture: jai_types::Architecture::Other("explicit-byte-order".into()),
        layout: LayoutPolicy::lp64(),
        byte_order: jai_types::ByteOrder::Big,
    };
    let target = ByteTarget::from(&profile);
    assert_eq!(target.policy, profile.layout);
    assert_eq!(target.endian, Endian::Big);
}

#[test]
fn descriptors_do_not_require_an_interned_pointer_type() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let string = types.string();
    let slice = types.slice(byte).unwrap();
    let dynamic = types.dynamic_array(byte).unwrap();
    let array = types.fixed_array(byte, 2).unwrap();
    assert!(types.lookup(&TypeKind::Pointer(byte)).is_none());
    let mut memory = Memory::new(Limits::default());
    let storage = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![int(IntegerType::U8, 65), int(IntegerType::U8, 66)],
            }),
        )
        .unwrap();
    let pointer = memory.index(&types, &storage, 0).unwrap();
    let target = ByteTarget::default();
    for (ty, value) in [
        (
            string,
            Value::StringView {
                pointer: pointer.clone(),
                count: 2,
            },
        ),
        (
            slice,
            Value::Slice {
                ty: slice,
                pointer: pointer.clone(),
                count: 2,
            },
        ),
        (
            dynamic,
            Value::DynamicArray {
                ty: dynamic,
                pointer,
                count: 1,
                allocated: 2,
                allocator: None,
            },
        ),
    ] {
        let mut image = ByteImage::encode(&types, target, ty, &value, 128).unwrap();
        image.retokenize_handles(|_| Ok(17)).unwrap();
        assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
    }
    for (ty, value) in [
        (
            string,
            Value::StringView {
                pointer: Pointer::null(byte),
                count: 0,
            },
        ),
        (
            slice,
            Value::Slice {
                ty: slice,
                pointer: Pointer::null(byte),
                count: 0,
            },
        ),
        (
            dynamic,
            Value::DynamicArray {
                ty: dynamic,
                pointer: Pointer::null(byte),
                count: 0,
                allocated: 0,
                allocator: None,
            },
        ),
    ] {
        let image = ByteImage::encode(&types, target, ty, &value, 128).unwrap();
        assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
    }
    assert!(types.lookup(&TypeKind::Pointer(byte)).is_none());
}

#[test]
fn signed_descriptor_fields_round_trip_without_consumer_validation() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let signed = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(byte).unwrap();
    let dynamic = types.dynamic_array(byte).unwrap();
    let target = ByteTarget::default();
    for (ty, value) in [
        (
            types.string(),
            Value::StringView {
                pointer: Pointer::null(byte),
                count: i64::MIN,
            },
        ),
        (
            slice,
            Value::Slice {
                ty: slice,
                pointer: Pointer::null(byte),
                count: -7,
            },
        ),
        (
            dynamic,
            Value::DynamicArray {
                ty: dynamic,
                pointer: Pointer::null(byte),
                count: 9,
                allocated: -3,
                allocator: None,
            },
        ),
    ] {
        let mut image = ByteImage::encode(&types, target, ty, &value, 128).unwrap();
        assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
        let count = match &value {
            Value::Slice { count, .. }
            | Value::StringView { count, .. }
            | Value::DynamicArray { count, .. } => *count,
            _ => unreachable!(),
        };
        assert_eq!(
            image
                .read(&types, target, 0, signed)
                .unwrap()
                .integer()
                .unwrap()
                .value(),
            i128::from(count)
        );
        image
            .write(
                &types,
                target,
                0,
                signed,
                &Value::Int(Integer::wrapping(IntegerType::S64, -1)),
            )
            .unwrap();
        match image.read(&types, target, 0, ty).unwrap() {
            Value::Slice { count, .. }
            | Value::StringView { count, .. }
            | Value::DynamicArray { count, .. } => assert_eq!(count, -1),
            _ => panic!("expected descriptor"),
        }
        if ty == dynamic {
            let offset = 16;
            assert_eq!(
                image
                    .read(&types, target, offset, signed)
                    .unwrap()
                    .integer()
                    .unwrap()
                    .value(),
                -3
            );
        }
    }
}

#[test]
fn runtime_type_null_storage_requires_no_header_schema() {
    let types = TypeRegistry::new();
    let ty = types.meta_type();
    let value = Value::Type { descriptor: None };
    for target in [
        ByteTarget::default(),
        ByteTarget {
            policy: LayoutPolicy::new(
                jai_types::ScalarLayout::new(4, 4),
                [
                    jai_types::ScalarLayout::new(1, 1),
                    jai_types::ScalarLayout::new(2, 2),
                    jai_types::ScalarLayout::new(4, 4),
                    jai_types::ScalarLayout::new(8, 4),
                ],
                [
                    jai_types::ScalarLayout::new(4, 4),
                    jai_types::ScalarLayout::new(8, 4),
                ],
                jai_types::ScalarLayout::new(1, 1),
            )
            .unwrap(),
            endian: Endian::Big,
        },
    ] {
        let image = ByteImage::encode(&types, target, ty, &value, 64).unwrap();
        assert_eq!(image.len() as u64, target.policy.pointer().size);
        assert!(image.bytes().iter().all(|byte| *byte == 0));
        assert!(!image.range_has_provenance(0, image.len()).unwrap());
        assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
        let mut forged = image.clone();
        forged.write_range(0, &[1]).unwrap();
        assert!(matches!(
            forged.read(&types, target, 0, ty),
            Err(Error::InvalidIr(_))
        ));
    }
}

#[test]
fn runtime_type_storage_shares_ordinary_pointer_handles_and_address_origins() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, [word]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let pointer_ty = types.pointer(header).unwrap();
    let ty = types.meta_type();
    let target = ByteTarget::default();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory
        .allocate(
            &types,
            header,
            Some(Value::Record {
                ty: header,
                fields: vec![int(IntegerType::U64, 17)],
            }),
        )
        .unwrap();
    let value = Value::Type {
        descriptor: Some(pointer.clone()),
    };
    let mut image = ByteImage::encode(&types, target, ty, &value, 128).unwrap();
    image
        .retokenize_handles(|handle| {
            assert_eq!(handle, &Value::Pointer(pointer.clone()));
            Ok(42)
        })
        .unwrap();
    assert_eq!(image.read(&types, target, 0, ty).unwrap(), value);
    assert_eq!(
        image.read(&types, target, 0, pointer_ty).unwrap(),
        Value::Pointer(pointer.clone())
    );
    assert_eq!(
        image
            .read(&types, target, 0, word)
            .unwrap()
            .number()
            .unwrap()
            .provenance(),
        Some(&crate::AddressProvenance::Pointer(pointer.clone()))
    );
    let mut copied = ByteImage::from_bytes(target, vec![0; 8], 128).unwrap();
    copied.copy_range_from(&image, 0, 0, 8).unwrap();
    assert_eq!(copied.read(&types, target, 0, ty).unwrap(), value);
    copied
        .write(&types, target, 0, pointer_ty, &Value::Pointer(pointer))
        .unwrap();
    assert_eq!(copied.read(&types, target, 0, ty).unwrap(), value);
    let current = copied.bytes()[0];
    copied.write_range(0, &[current]).unwrap();
    assert!(matches!(
        copied.read(&types, target, 0, ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    copied.fill_range(0, 8, 0).unwrap();
    assert_eq!(
        copied.read(&types, target, 0, ty).unwrap(),
        Value::Type { descriptor: None }
    );
    let forged = ByteImage::from_bytes(target, image.bytes().to_vec(), 128).unwrap();
    assert!(matches!(
        forged.read(&types, target, 0, ty),
        Err(Error::InvalidIr(_))
    ));
}
