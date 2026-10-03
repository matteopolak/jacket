use crate::{ByteImage, ByteTarget, Endian, Error, Limits, Memory, Value};
use jai_types::{Integer, IntegerType, RecordKind, ScalarType, TypeRegistry};

fn word(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::U64, value))
}

#[test]
fn range_bounds_and_target_errors_leave_destination_unchanged() {
    let target = ByteTarget::default();
    let source = ByteImage::from_bytes(target, vec![1, 2, 3, 4], 64).unwrap();
    let mut destination = ByteImage::from_bytes(target, vec![8, 9, 10, 11], 64).unwrap();
    let before = destination.bytes().to_vec();
    assert!(matches!(
        destination.copy_range_from(&source, 2, 0, 3),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        destination.copy_range_from(&source, 0, 3, 2),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        destination.fill_range(usize::MAX, 2, 0),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        destination.write_range(4, &[1]),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        destination.copy_range_within(0, 3, 2),
        Err(Error::OutOfBounds { .. })
    ));
    let other = ByteImage::from_bytes(
        ByteTarget {
            endian: Endian::Big,
            ..target
        },
        vec![0; 4],
        64,
    )
    .unwrap();
    assert!(matches!(
        destination.copy_range_from(&other, 0, 0, 4),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(destination.bytes(), before);
    assert_eq!(destination.read_range(4, 0).unwrap(), &[]);
    destination.fill_range(4, 0, 255).unwrap();
    destination.copy_range_from(&source, 4, 4, 0).unwrap();
    assert_eq!(destination.bytes(), before);
}

#[test]
fn whole_handle_copy_preserves_pointer_provenance_but_partial_copy_does_not() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, integer, Some(word(5))).unwrap();
    let pointer_value = Value::Pointer(pointer);
    let target = ByteTarget::default();
    let source = ByteImage::encode(&types, target, pointer_ty, &pointer_value, 64).unwrap();
    let mut destination = ByteImage::from_bytes(target, vec![0; 16], 64).unwrap();
    destination.copy_range_from(&source, 0, 8, 8).unwrap();
    assert_eq!(
        destination.read(&types, target, 8, pointer_ty).unwrap(),
        pointer_value
    );
    destination.copy_range_from(&source, 0, 0, 7).unwrap();
    assert!(matches!(
        destination.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(
        destination.read(&types, target, 8, pointer_ty).unwrap(),
        pointer_value
    );
    let current = destination.bytes()[8];
    destination.write_range(8, &[current]).unwrap();
    assert!(matches!(
        destination.read(&types, target, 8, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    destination.fill_range(8, 8, 0).unwrap();
    assert!(
        destination
            .read(&types, target, 8, pointer_ty)
            .unwrap()
            .pointer()
            .unwrap()
            .is_null()
    );
}

#[test]
fn whole_union_copy_preserves_view_and_partial_raw_write_invalidates_it() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [integer, integer]).unwrap();
    let target = ByteTarget::default();
    let value = Value::Union {
        ty: union,
        field: 1,
        value: Box::new(word(0x1234)),
    };
    let source = ByteImage::encode(&types, target, union, &value, 64).unwrap();
    let mut destination = ByteImage::from_bytes(target, vec![0; 16], 64).unwrap();
    destination.copy_range_from(&source, 0, 8, 8).unwrap();
    assert_eq!(destination.read(&types, target, 8, union).unwrap(), value);
    destination
        .write_range(15, &[destination.bytes()[15]])
        .unwrap();
    let Value::Union { field, .. } = destination.read(&types, target, 8, union).unwrap() else {
        panic!()
    };
    assert_eq!(field, 0);
    destination.copy_range_from(&source, 0, 8, 7).unwrap();
    let Value::Union { field, .. } = destination.read(&types, target, 8, union).unwrap() else {
        panic!()
    };
    assert_eq!(field, 0);
}

#[test]
fn overlapping_within_copy_has_explicit_memmove_snapshot_semantics() {
    let target = ByteTarget::default();
    let mut image = ByteImage::from_bytes(target, vec![0, 1, 2, 3, 4, 5, 6, 7], 64).unwrap();
    image.copy_range_within(0, 2, 6).unwrap();
    assert_eq!(image.bytes(), &[0, 1, 0, 1, 2, 3, 4, 5]);
    image.copy_range_within(2, 0, 6).unwrap();
    assert_eq!(image.bytes(), &[0, 1, 2, 3, 4, 5, 4, 5]);
}

#[test]
fn overlapping_memmove_moves_complete_relocation_and_invalidates_shredded_original() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, integer, Some(word(5))).unwrap();
    let value = Value::Pointer(pointer);
    let target = ByteTarget::default();
    let source = ByteImage::encode(&types, target, pointer_ty, &value, 64).unwrap();
    let mut image = ByteImage::from_bytes(target, vec![0; 16], 64).unwrap();
    image.copy_range_from(&source, 0, 0, 8).unwrap();
    image.copy_range_within(0, 4, 8).unwrap();
    assert_eq!(image.read(&types, target, 4, pointer_ty).unwrap(), value);
    assert!(matches!(
        image.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    image.copy_range_within(4, 0, 8).unwrap();
    assert_eq!(image.read(&types, target, 0, pointer_ty).unwrap(), value);
    match image.read(&types, target, 4, pointer_ty) {
        Ok(Value::Pointer(pointer)) => assert!(pointer.is_null()),
        Err(Error::UnsupportedPointerOperation(_)) => {}
        other => panic!("unexpected shredded handle result: {other:?}"),
    }
    // No complete handle remains at offset four, even when its visible bits are zero.
    assert!(!image.relocations.iter().any(|r| r.offset == 4));
}

#[test]
fn batch_assembly_preserves_handle_number_union_and_initialization_metadata() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [integer, integer]).unwrap();
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: jai_types::CallingConvention::Jai,
            context: jai_types::ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, integer, Some(word(5))).unwrap();
    let target = ByteTarget::default();
    let pointer_value = Value::Pointer(pointer.clone());
    let address = crate::Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        crate::AddressProvenance::Derived {
            memory: pointer.memory_identity(),
            allocations: vec![pointer.allocation_key().1].into_boxed_slice(),
        },
    )
    .into_value();
    let union_value = Value::Union {
        ty: union,
        field: 1,
        value: Box::new(address.clone()),
    };
    let procedure = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(17)),
    };
    let mut hole = ByteImage::uninitialized(target, 2, 128).unwrap();
    hole.write_range(0, &[7]).unwrap();
    let parts = vec![
        ByteImage::encode(&types, target, pointer_ty, &pointer_value, 128).unwrap(),
        ByteImage::encode(&types, target, integer, &address, 128).unwrap(),
        ByteImage::encode(&types, target, union, &union_value, 128).unwrap(),
        hole,
        ByteImage::encode(&types, target, signature, &procedure, 128).unwrap(),
    ];
    let image = ByteImage::concatenate(&parts, target, 128).unwrap();
    assert_eq!(image.len(), 34);
    assert_eq!(
        image.read(&types, target, 0, pointer_ty).unwrap(),
        pointer_value
    );
    assert_eq!(image.read(&types, target, 8, integer).unwrap(), address);
    assert_eq!(image.read(&types, target, 16, union).unwrap(), union_value);
    assert_eq!(image.read_range(24, 1).unwrap(), &[7]);
    assert!(matches!(image.read_range(25, 1), Err(Error::Uninitialized)));
    assert_eq!(
        image.read(&types, target, 26, signature).unwrap(),
        procedure
    );
}

#[test]
fn extract_range_clones_only_complete_handles_and_downgrades_partial_address_bytes() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let half = types.scalar(ScalarType::Int(IntegerType::U32));
    let pointer_ty = types.pointer(integer).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, integer, Some(word(5))).unwrap();
    let value = Value::Pointer(pointer.clone());
    let target = ByteTarget::default();
    let source = ByteImage::encode(&types, target, pointer_ty, &value, 64).unwrap();
    let full = source.extract_range(0, 8).unwrap();
    assert_eq!(full.read(&types, target, 0, pointer_ty).unwrap(), value);
    let partial = source.extract_range(1, 4).unwrap();
    assert_eq!(partial.len(), 4);
    assert_eq!(
        partial
            .read(&types, target, 0, half)
            .unwrap()
            .number()
            .unwrap()
            .provenance(),
        Some(&crate::AddressProvenance::Derived {
            memory: pointer.memory_identity(),
            allocations: vec![pointer.allocation_key().1].into_boxed_slice(),
        })
    );
    assert!(matches!(
        source.extract_range(usize::MAX, 2),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(source.read(&types, target, 0, pointer_ty).unwrap(), value);
}

#[test]
fn batch_assembly_rejects_total_bounds_and_target_mismatch_before_building() {
    let target = ByteTarget::default();
    let first = ByteImage::from_bytes(target, vec![1; 4], 64).unwrap();
    let second = ByteImage::from_bytes(target, vec![2; 4], 64).unwrap();
    assert!(matches!(
        ByteImage::concatenate(&[first.clone(), second], target, 7),
        Err(Error::Limit(crate::LimitKind::ValueCells))
    ));
    let other = ByteImage::from_bytes(
        ByteTarget {
            endian: Endian::Big,
            ..target
        },
        vec![0; 4],
        64,
    )
    .unwrap();
    assert!(matches!(
        ByteImage::concatenate(&[first.clone(), other], target, 64),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(first.bytes(), &[1; 4]);
    assert!(ByteImage::concatenate(&[], target, 0).unwrap().is_empty());
}

#[test]
fn offset_indexed_metadata_survives_earlier_stores_and_middle_extraction() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types
        .define_record(union, [pointer_ty, pointer_ty])
        .unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, integer, Some(word(5))).unwrap();
    let target = ByteTarget::default();
    let value = Value::Union {
        ty: union,
        field: 1,
        value: Box::new(Value::Pointer(pointer.clone())),
    };
    let part = ByteImage::encode(&types, target, union, &value, 2048).unwrap();
    let mut image = ByteImage::concatenate(&vec![part.clone(); 128], target, 2048).unwrap();
    // Replace low offsets after all later metadata already exists.
    image.write(&types, target, 16, union, &value).unwrap();
    image.fill_range(0, 8, 0).unwrap();
    image.copy_range_from(&part, 0, 0, 8).unwrap();
    image.note_union_field(&types, 8, union, 1).unwrap();
    image
        .fill_range_number(
            24,
            8,
            crate::Number::address(
                Integer::wrapping(IntegerType::U64, 42),
                crate::AddressProvenance::Pointer(pointer.clone()),
            ),
        )
        .unwrap();
    assert!(
        image
            .provenance
            .windows(2)
            .all(|p| p[0].offset + p[0].length <= p[1].offset)
    );
    assert!(
        image
            .relocations
            .windows(2)
            .all(|r| r[0].offset < r[1].offset)
    );
    assert!(image.unions.windows(2).all(|u| u[0].offset <= u[1].offset));
    for slot in 0..128 {
        if slot == 3 {
            assert!(matches!(
                image.read(&types, target, slot * 8, pointer_ty),
                Err(Error::UnsupportedPointerOperation(_))
            ));
            let number = image
                .read(&types, target, slot * 8, integer)
                .unwrap()
                .number()
                .unwrap();
            assert_eq!(number.allocation_ids(), vec![pointer.allocation_key().1]);
        } else {
            assert_eq!(image.read(&types, target, slot * 8, union).unwrap(), value);
            assert!(
                image
                    .read(&types, target, slot * 8, integer)
                    .unwrap()
                    .number()
                    .unwrap()
                    .provenance()
                    .is_some()
            );
        }
    }
    let middle = image.extract_range(63 * 8, 3 * 8).unwrap();
    assert_eq!(middle.relocations.len(), 3);
    assert_eq!(middle.unions.len(), 3);
    assert_eq!(middle.provenance.len(), 3);
    for slot in 0..3 {
        assert_eq!(middle.read(&types, target, slot * 8, union).unwrap(), value);
    }
}
