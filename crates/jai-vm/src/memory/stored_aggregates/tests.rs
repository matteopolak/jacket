use super::*;
use jai_types::{Integer, IntegerType, RecordKind, ScalarType, TypeRegistry};

fn int(ty: IntegerType, bits: u64) -> Value {
    Value::Int(Integer::wrapping(ty, i128::from(bits)))
}
fn fixture() -> (TypeRegistry, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer = types.pointer(word).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [byte, word, pointer]).unwrap();
    (types, union, word)
}
fn small_store(memory: &mut Memory, types: &dyn TypeView, root: &Pointer) {
    let small = memory.field(types, root, 0).unwrap();
    memory
        .store(types, &small, int(IntegerType::U8, 42))
        .unwrap();
}
fn wide(memory: &Memory, types: &dyn TypeView, root: &Pointer) -> Result<Value, Error> {
    memory.load(types, &memory.field(types, root, 1)?)
}
fn initialized(memory: &mut Memory, types: &dyn TypeView, union: TypeId) -> Pointer {
    memory
        .allocate(
            types,
            union,
            Some(Value::Union {
                ty: union,
                field: 1,
                value: Box::new(int(IntegerType::U64, 0x1122334455667788)),
            }),
        )
        .unwrap()
}

#[test]
fn ordinary_copy_preserves_inactive_tail_and_independent_storage() {
    let (types, union, _) = fixture();
    let mut memory = Memory::new(Limits::default());
    let source = initialized(&mut memory, &types, union);
    small_store(&mut memory, &types, &source);
    let value = memory.load(&types, &source).unwrap();
    assert!(matches!(value, Value::StoredAggregate(_)));
    let copy = memory.allocate(&types, union, None).unwrap();
    memory.store(&types, &copy, value.clone()).unwrap();
    assert_eq!(
        wide(&memory, &types, &copy)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
    let source_wide = memory.field(&types, &source, 1).unwrap();
    memory
        .store(&types, &source_wide, int(IntegerType::U64, 1))
        .unwrap();
    assert_eq!(
        wide(&memory, &types, &copy)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
    let Value::StoredAggregate(snapshot) = value else {
        unreachable!()
    };
    assert!(snapshot.publication_semantic(&types, 1024).is_err());
}

#[test]
fn ordinary_copy_preserves_initialization_holes() {
    let (types, union, _) = fixture();
    let mut memory = Memory::new(Limits::default());
    let source = memory.allocate(&types, union, None).unwrap();
    small_store(&mut memory, &types, &source);
    let value = memory.load(&types, &source).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        unreachable!()
    };
    assert!(snapshot.publication_semantic(&types, 1024).is_err());
    let copy = memory.allocate(&types, union, None).unwrap();
    memory.store(&types, &copy, value).unwrap();
    assert_eq!(wide(&memory, &types, &copy), Err(Error::Uninitialized));
    assert_eq!(
        memory
            .load(&types, &memory.field(&types, &copy, 0).unwrap())
            .unwrap(),
        int(IntegerType::U8, 42)
    );
}

#[test]
fn ordinary_copy_preserves_inactive_partial_address_provenance() {
    let (types, union, word) = fixture();
    let mut memory = Memory::new(Limits::default());
    let payload = memory
        .allocate(&types, word, Some(int(IntegerType::U64, 7)))
        .unwrap();
    let source = memory
        .allocate(
            &types,
            union,
            Some(Value::Union {
                ty: union,
                field: 2,
                value: Box::new(Value::Pointer(payload)),
            }),
        )
        .unwrap();
    small_store(&mut memory, &types, &source);
    let before = wide(&memory, &types, &source).unwrap().number().unwrap();
    assert!(before.provenance().is_some());
    let value = memory.load(&types, &source).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        unreachable!()
    };
    assert!(snapshot.publication_semantic(&types, 1024).is_err());
    let mut origins = Vec::new();
    snapshot
        .image()
        .visit_address_origins(|memory, allocation| {
            origins.push((memory, allocation));
            Ok(())
        })
        .unwrap();
    assert!(!origins.is_empty());
    assert!(
        origins
            .iter()
            .all(|(identity, _)| *identity == memory.identity)
    );
    let copy = memory.allocate(&types, union, None).unwrap();
    memory.store(&types, &copy, value.clone()).unwrap();
    assert_eq!(
        wide(&memory, &types, &copy).unwrap().number().unwrap(),
        before
    );
    let mut foreign = Memory::new(Limits::default());
    let destination = foreign.allocate(&types, union, None).unwrap();
    assert_eq!(
        foreign.store(&types, &destination, value),
        Err(Error::ForeignPointer)
    );
    assert_eq!(
        foreign.load(&types, &destination),
        Err(Error::Uninitialized)
    );
}

#[test]
fn nested_field_copy_and_update_preserve_snapshot() {
    let (mut types, union, _) = fixture();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [union, byte]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let source = initialized(&mut memory, &types, union);
    small_store(&mut memory, &types, &source);
    let value = memory.load(&types, &source).unwrap();
    let record_root = memory
        .allocate(
            &types,
            record,
            Some(Value::Record {
                ty: record,
                fields: vec![value, int(IntegerType::U8, 3)],
            }),
        )
        .unwrap();
    let nested = memory.field(&types, &record_root, 0).unwrap();
    assert_eq!(
        wide(&memory, &types, &nested)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
    let Value::StoredAggregate(snapshot) = memory.load(&types, &record_root).unwrap() else {
        unreachable!()
    };
    let updated = snapshot
        .with_field(&types, 1, &int(IntegerType::U8, 9), 1024)
        .unwrap();
    let Value::StoredAggregate(updated) = updated else {
        unreachable!()
    };
    assert_eq!(
        updated.field(&types, 1, 1024).unwrap(),
        int(IntegerType::U8, 9)
    );
    let Value::StoredAggregate(member) = updated.field(&types, 0, 1024).unwrap() else {
        unreachable!()
    };
    assert_eq!(
        member
            .field(&types, 1, 1024)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
}

#[test]
fn snapshot_identity_includes_inactive_bytes_and_holes() {
    let (types, union, _) = fixture();
    let mut memory = Memory::new(Limits::default());
    let tail = initialized(&mut memory, &types, union);
    small_store(&mut memory, &types, &tail);
    let holes = memory.allocate(&types, union, None).unwrap();
    small_store(&mut memory, &types, &holes);
    let canonical = memory
        .allocate(
            &types,
            union,
            Some(Value::Union {
                ty: union,
                field: 0,
                value: Box::new(int(IntegerType::U8, 42)),
            }),
        )
        .unwrap();
    memory
        .ensure_image(&types, memory.allocation(&canonical).unwrap())
        .unwrap();
    let values = [tail, holes, canonical].map(|p| memory.load(&types, &p).unwrap());
    assert_eq!(values[0].semantic(), values[1].semantic());
    assert_eq!(values[1].semantic(), values[2].semantic());
    assert_eq!(
        values
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        3
    );
}

#[test]
fn canonical_snapshot_can_publish_and_storage_cells_are_bounded() {
    let (types, union, _) = fixture();
    let value = Value::Union {
        ty: union,
        field: 0,
        value: Box::new(int(IntegerType::U8, 42)),
    };
    let image = ByteImage::encode(&types, ByteTarget::default(), union, &value, 1024).unwrap();
    let snapshot = image
        .read_preserving(&types, ByteTarget::default(), 0, union)
        .unwrap();
    let Value::StoredAggregate(carrier) = &snapshot else {
        unreachable!()
    };
    assert_eq!(carrier.publication_semantic(&types, 1024).unwrap(), &value);
    let cells = snapshot.cells(1024).unwrap();
    assert_eq!(
        snapshot.cells(cells - 1),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(
        image.read_preserving_with_limit(&types, ByteTarget::default(), 0, union, cells - 1),
        Err(Error::Limit(LimitKind::ValueCells))
    );
}

#[test]
fn complete_pointer_member_copy_and_publication_keep_handle_identity() {
    let (types, union, word) = fixture();
    let mut memory = Memory::new(Limits::default());
    let payload = memory
        .allocate(&types, word, Some(int(IntegerType::U64, 7)))
        .unwrap();
    let source = memory
        .allocate(
            &types,
            union,
            Some(Value::Union {
                ty: union,
                field: 2,
                value: Box::new(Value::Pointer(payload.clone())),
            }),
        )
        .unwrap();
    memory
        .ensure_image(&types, memory.allocation(&source).unwrap())
        .unwrap();
    let value = memory.load(&types, &source).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        unreachable!()
    };
    assert!(snapshot.publication_semantic(&types, 1024).is_ok());
    let copy = memory.allocate(&types, union, None).unwrap();
    memory.store(&types, &copy, value).unwrap();
    let actual = memory
        .load(&types, &memory.field(&types, &copy, 2).unwrap())
        .unwrap();
    assert_eq!(actual.pointer().unwrap(), &payload);
}

#[test]
fn array_and_distinct_copies_preserve_embedded_union_storage() {
    let (mut types, union, _) = fixture();
    let array = types.fixed_array(union, 1).unwrap();
    let distinct = types.reserve_distinct(jai_types::DistinctKind::Distinct);
    types.define_distinct(distinct, array).unwrap();
    let mut memory = Memory::new(Limits::default());
    let source = initialized(&mut memory, &types, union);
    small_store(&mut memory, &types, &source);
    let union_value = memory.load(&types, &source).unwrap();
    let root = memory
        .allocate(
            &types,
            distinct,
            Some(Value::Distinct {
                ty: distinct,
                value: Box::new(Value::Array {
                    ty: array,
                    elements: vec![union_value],
                }),
            }),
        )
        .unwrap();
    memory
        .ensure_image(&types, memory.allocation(&root).unwrap())
        .unwrap();
    let value = memory.load(&types, &root).unwrap();
    let Value::StoredAggregate(snapshot) = &value else {
        unreachable!()
    };
    let repr = snapshot.representation(&types, 1024).unwrap();
    let copy = memory.allocate(&types, array, Some(repr)).unwrap();
    let element = memory.index(&types, &copy, 0).unwrap();
    assert_eq!(
        wide(&memory, &types, &element)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
    assert!(snapshot.publication_semantic(&types, 1024).is_err());
}

#[test]
fn storage_cache_updates_with_structural_commits_and_transaction_restore() {
    let (mut types, union, _) = fixture();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [union]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let source = initialized(&mut memory, &types, union);
    small_store(&mut memory, &types, &source);
    let carrier = memory.load(&types, &source).unwrap();
    let root = memory
        .allocate(
            &types,
            record,
            Some(Value::Record {
                ty: record,
                fields: vec![carrier],
            }),
        )
        .unwrap();
    assert!(memory.allocation(&root).unwrap().has_stored_aggregate);
    let snapshot = memory.snapshot();
    memory
        .store(
            &types,
            &root,
            Value::Record {
                ty: record,
                fields: vec![Value::Union {
                    ty: union,
                    field: 0,
                    value: Box::new(int(IntegerType::U8, 1)),
                }],
            },
        )
        .unwrap();
    assert!(!memory.allocation(&root).unwrap().has_stored_aggregate);
    memory.restore(snapshot);
    assert!(memory.allocation(&root).unwrap().has_stored_aggregate);
    let member = memory.field(&types, &root, 0).unwrap();
    assert_eq!(
        wide(&memory, &types, &member)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        0x112233445566772a
    );
    let failed = memory.store(&types, &root, int(IntegerType::U8, 1));
    assert!(failed.is_err());
    assert!(memory.allocation(&root).unwrap().has_stored_aggregate);
}
