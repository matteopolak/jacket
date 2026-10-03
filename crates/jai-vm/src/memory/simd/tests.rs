use super::*;
use jai_types::{RecordKind, TypeRegistry};

fn bytes(types: &mut TypeRegistry, memory: &mut Memory, count: u64) -> Pointer {
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let array = types.fixed_array(byte, count).unwrap();
    let value = Value::Array {
        ty: array,
        elements: (0..count)
            .map(|value| Value::Int(Integer::wrapping(IntegerType::U8, i128::from(value))))
            .collect(),
    };
    let root = memory.allocate(types, array, Some(value)).unwrap();
    memory.sequence_data(types, &root).unwrap()
}

#[test]
fn unaligned_transfers_snapshot_bytes_and_preserve_neighbors() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let source = bytes(&mut types, &mut memory, 64);
    let unaligned = memory.offset(&types, &source, 1).unwrap();
    let snapshot = memory.simd_read_bytes(&types, &unaligned, 32).unwrap();
    assert_eq!(snapshot, (1..33).collect::<Vec<_>>());
    let destination = memory.offset(&types, &source, 2).unwrap();
    memory
        .simd_write_bytes(&types, &destination, &snapshot)
        .unwrap();
    assert_eq!(
        memory.simd_read_bytes(&types, &destination, 32).unwrap(),
        snapshot
    );
    assert_eq!(
        memory.load(&types, &source).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 0))
    );
    let following = memory.offset(&types, &source, 34).unwrap();
    assert_eq!(
        memory.load(&types, &following).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 34))
    );
}

#[test]
fn transfers_reject_invalid_handles_and_immutable_or_uninitialized_storage() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let source = bytes(&mut types, &mut memory, 32);
    let byte = source.pointee();
    assert_eq!(
        memory.simd_read_bytes(&types, &Pointer::null(byte), 16),
        Err(Error::NullPointer)
    );
    let past = memory.offset(&types, &source, 17).unwrap();
    assert!(matches!(
        memory.simd_read_bytes(&types, &past, 16),
        Err(Error::OutOfBounds { .. })
    ));
    let mut foreign = Memory::new(Limits::default());
    assert_eq!(
        foreign.simd_write_bytes(&types, &source, &[0; 16]),
        Err(Error::ForeignPointer)
    );
    memory.freeze(&source).unwrap();
    assert_eq!(
        memory.simd_write_bytes(&types, &source, &[0; 16]),
        Err(Error::ReadOnlyStorage)
    );
    let array = types.fixed_array(byte, 32).unwrap();
    let hole = memory.allocate(&types, array, None).unwrap();
    assert_eq!(
        memory.simd_read_bytes(&types, &hole, 32),
        Err(Error::Uninitialized)
    );
    memory.simd_write_bytes(&types, &hole, &[42; 32]).unwrap();
    assert_eq!(
        memory.simd_read_bytes(&types, &hole, 32).unwrap(),
        vec![42; 32]
    );
    memory.release(&hole).unwrap();
    assert_eq!(
        memory.simd_read_bytes(&types, &hole, 16),
        Err(Error::DanglingPointer)
    );
}

#[test]
fn field_regions_cannot_be_widened_by_vector_transfers() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [word, word, word]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            record,
            Some(Value::Record {
                ty: record,
                fields: vec![Value::Int(Integer::wrapping(IntegerType::U64, 0)); 3],
            }),
        )
        .unwrap();
    let field = memory.field(&types, &root, 0).unwrap();
    let view = memory
        .cast_pointer(&types, &field, byte, CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.simd_read_bytes(&types, &view, 16),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        memory.simd_write_bytes(&types, &view, &[0; 16]),
        Err(Error::OutOfBounds { .. })
    ));
}

#[test]
fn numeric_loads_reject_address_provenance_and_stores_invalidate_only_written_handles() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = types.pointer(word).unwrap();
    let array = types.fixed_array(pointer, 4).unwrap();
    let mut memory = Memory::new(Limits::default());
    let target = memory
        .allocate(
            &types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::U64, 0))),
        )
        .unwrap();
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![Value::Pointer(target.clone()); 4],
            }),
        )
        .unwrap();
    let data = memory.sequence_data(&types, &root).unwrap();
    assert!(matches!(
        memory.simd_read_bytes(&types, &data, 16),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    memory.simd_write_bytes(&types, &data, &[0xff; 16]).unwrap();
    assert!(memory.load(&types, &data).is_err());
    let third = memory.offset(&types, &data, 2).unwrap();
    assert_eq!(memory.load(&types, &third).unwrap(), Value::Pointer(target));
}

#[test]
fn register_reservation_budget_rejection_leaves_destination_bytes_unchanged() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits {
        value_cells: 64,
        ..Limits::default()
    });
    let destination = bytes(&mut types, &mut memory, 16);
    let original = memory.simd_read_bytes(&types, &destination, 16).unwrap();
    assert_eq!(
        memory.simd_write_bytes_reserved(&types, &destination, &[42; 16], 64),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(
        memory.simd_read_bytes(&types, &destination, 16).unwrap(),
        original
    );
}
