use super::*;
use crate::{Limits, Memory, Pointer};
use jai_types::{
    AllocatorField, AllocatorMode, AllocatorSchema, CallingConvention, ContextMode, ProcedureType,
    ScalarLayout, TypeRegistry, Variadic,
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

fn target(width: u64) -> ByteTarget {
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
        endian: Endian::Little,
    }
}

fn descriptor(ty: TypeId, byte: TypeId, payload: Value) -> Value {
    Value::DynamicArray {
        ty,
        pointer: Pointer::null(byte),
        count: -3,
        allocated: -9,
        allocator: Some(Box::new(payload)),
    }
}

#[test]
fn certified_zero_allocator_is_a_typed_payload_on_both_pointer_widths() {
    let (types, schema, byte, dynamic) = fixture();
    let payload = Value::Record {
        ty: schema.ty(),
        fields: vec![
            Value::Procedure {
                signature: schema.field(AllocatorField::Procedure).ty,
                procedure: None,
            },
            Value::Pointer(Pointer::null(types.void())),
        ],
    };
    for width in [4, 8] {
        let target = target(width);
        let value = descriptor(dynamic, byte, payload.clone());
        let image = ByteImage::encode(&types, target, dynamic, &value, 512).unwrap();
        let storage = LayoutEngine::new(&types, target.policy)
            .layout(dynamic)
            .unwrap()
            .clone();
        assert_eq!(image.len(), storage.size as usize);
        assert_eq!(image.metadata_cells(), 0);
        assert_eq!(image.read(&types, target, 0, dynamic).unwrap(), value);
        let mut absent = value.clone();
        if let Value::DynamicArray { allocator, .. } = &mut absent {
            *allocator = None;
        }
        assert!(ByteImage::encode(&types, target, dynamic, &absent, 512).is_err());
    }
}

#[test]
fn allocator_procedure_and_data_slots_keep_typed_relocations_across_copy() {
    let (types, schema, byte, dynamic) = fixture();
    let signature = schema.field(AllocatorField::Procedure).ty;
    let procedure = jai_ir::ProcedurePrototype {
        id: jai_ir::ProcedureId::new(7),
        signature,
        origin: jai_ir::PrototypeOrigin::SourceContract {
            symbol: "byte_codec_allocator_fixture".into(),
        },
    };
    for width in [4, 8] {
        let target = target(width);
        let mut memory = Memory::with_target(Limits::default(), target);
        let data = memory
            .allocate(
                &types,
                byte,
                Some(Value::Int(Integer::wrapping(IntegerType::U8, 42))),
            )
            .unwrap();
        let opaque = memory
            .cast_pointer(&types, &data, types.void(), jai_types::CastMode::Checked)
            .unwrap();
        let payload = Value::Record {
            ty: schema.ty(),
            fields: vec![
                Value::Procedure {
                    signature,
                    procedure: Some(procedure.id),
                },
                Value::Pointer(opaque.clone()),
            ],
        };
        let value = descriptor(dynamic, byte, payload.clone());
        let image = ByteImage::encode(&types, target, dynamic, &value, 512).unwrap();
        assert!(image.metadata_cells() >= 4);
        assert!(matches!(
            crate::byte_memory::metadata::encoded_metadata_cells(&value, 3),
            Err(Error::Limit(LimitKind::ValueCells))
        ));
        assert_eq!(image.read(&types, target, 0, dynamic).unwrap(), value);
        let tail = LayoutEngine::new(&types, target.policy)
            .layout(dynamic)
            .unwrap()
            .field_offsets[3] as usize;
        assert_eq!(
            image.read(&types, target, tail, signature).unwrap(),
            match &payload {
                Value::Record { fields, .. } => fields[0].clone(),
                _ => unreachable!(),
            }
        );
        let mut copy = ByteImage::from_bytes(target, vec![0; image.len()], 512).unwrap();
        copy.copy_range_from(&image, 0, 0, image.len()).unwrap();
        assert_eq!(copy.read(&types, target, 0, dynamic).unwrap(), value);
        let slot = tail + width as usize;
        copy.write_range(slot, &[copy.bytes()[slot]]).unwrap();
        assert!(copy.read(&types, target, 0, dynamic).is_err());
        assert_eq!(
            image
                .read(&types, target, slot, schema.field(AllocatorField::Data).ty)
                .unwrap(),
            Value::Pointer(opaque)
        );
    }
}

#[test]
fn partial_allocator_payload_copy_preserves_uninitialized_pointer_slot() {
    let (types, schema, byte, dynamic) = fixture();
    let target = target(8);
    let signature = schema.field(AllocatorField::Procedure).ty;
    let mut allocator = ByteImage::uninitialized(target, 16, 512).unwrap();
    allocator
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
    let payload = allocator
        .read_partial_preserving_with_limit(&types, target, 0, schema.ty(), 512)
        .unwrap();
    assert!(matches!(&payload, Value::StoredAggregate(_)));
    let value = descriptor(dynamic, byte, payload);
    let image = ByteImage::encode(&types, target, dynamic, &value, 512).unwrap();
    let decoded = image.read(&types, target, 0, dynamic).unwrap();
    let Value::DynamicArray {
        allocator: Some(payload),
        ..
    } = &decoded
    else {
        panic!("typed allocator payload required")
    };
    let Value::StoredAggregate(snapshot) = payload.as_ref() else {
        panic!("initialization mask must survive")
    };
    assert_eq!(
        snapshot.field(&types, 0, 512).unwrap(),
        Value::Procedure {
            signature,
            procedure: None
        }
    );
    assert!(matches!(
        snapshot.field(&types, 1, 512),
        Err(Error::Uninitialized)
    ));
    let encoded = ByteImage::encode(&types, target, dynamic, &decoded, 512).unwrap();
    assert_eq!(encoded, image);
    assert!(matches!(
        encoded.read_range(32, 8),
        Err(Error::Uninitialized)
    ));
}
