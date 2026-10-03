use super::*;
use jai_types::{LayoutPolicy, RecordKind, ScalarLayout, TypeRegistry};

fn fixture(flat: bool, capacity: i128) -> (TypeRegistry, TypeId, Value) {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let void = types.void();
    let pointer = types.pointer(void).unwrap();
    let ty = types.reserve_record(RecordKind::Struct);
    let mut fields = vec![integer, integer, pointer, integer];
    let mut values = vec![
        number(capacity),
        number(0),
        Value::Pointer(Pointer::null(void)),
        number(0),
    ];
    if flat {
        fields.push(integer);
        values.push(number(8));
    }
    types.define_record(ty, fields).unwrap();
    (types, ty, Value::Record { ty, fields: values })
}
fn number(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}
fn admit_pool_layouts(memory: &Memory, types: &dyn TypeView, pool: &Pointer) {
    memory
        .prepare_pointer_layouts(types, pool, true, usize::MAX)
        .1
        .unwrap();
    for &field in types
        .record_storage_definition(pool.pointee)
        .unwrap()
        .fields
        .iter()
    {
        memory.prepare_layout(types, field, usize::MAX).1.unwrap();
    }
    memory
        .prepare_layout(types, types.lookup(&TypeKind::String).unwrap(), usize::MAX)
        .1
        .unwrap();
}
fn ilp32() -> LayoutPolicy {
    LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap()
}
fn get(
    memory: &mut Memory,
    types: &dyn TypeView,
    pool: &Pointer,
    flat: bool,
    size: usize,
) -> Pointer {
    memory
        .pool_operation(types, pool, flat, PoolOperation::Get(size))
        .unwrap()
        .unwrap()
}

#[test]
fn pool_blocks_reuse_then_release_aliases_on_both_target_widths() {
    for policy in [LayoutPolicy::lp64(), ilp32()] {
        let (types, ty, value) = fixture(false, 128);
        let mut memory = Memory::with_layout(Limits::default(), policy);
        let pool = memory.allocate(&types, ty, Some(value)).unwrap();
        let first = get(&mut memory, &types, &pool, false, 64);
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let first_byte = memory
            .cast_pointer(&types, &first, byte, CastMode::Checked)
            .unwrap();
        memory
            .store(
                &types,
                &first_byte,
                Value::Int(Integer::wrapping(IntegerType::U8, 42)),
            )
            .unwrap();
        let left = memory
            .load(&types, &memory.field(&types, &pool, 1).unwrap())
            .unwrap()
            .integer()
            .unwrap()
            .value();
        assert_eq!(left, 128 - i128::from(policy.pointer().size) - 64);
        let second = get(&mut memory, &types, &pool, false, 64);
        assert_eq!(memory.allocation_count(), 3);
        assert!(!memory.same_address(&types, &first, &second).unwrap());
        assert!(memory.release(&pool).is_err());
        memory
            .pool_operation(&types, &pool, false, PoolOperation::Reset(false))
            .unwrap();
        let reused = get(&mut memory, &types, &pool, false, 64);
        assert!(memory.same_address(&types, &reused, &first).unwrap());
        assert_eq!(
            memory
                .load(&types, &first_byte)
                .unwrap()
                .integer()
                .unwrap()
                .value(),
            42
        );
        memory
            .pool_operation(&types, &pool, false, PoolOperation::Release)
            .unwrap();
        assert_eq!(
            memory.load(&types, &first_byte),
            Err(Error::DanglingPointer)
        );
        assert_eq!(memory.allocation_count(), 1);
        assert!(
            memory
                .load(&types, &memory.field(&types, &pool, 2).unwrap())
                .unwrap()
                .pointer()
                .unwrap()
                .is_null()
        );
        memory.release(&pool).unwrap();
        // Complete target facts remain cached after releasing all value storage.
        assert_eq!(memory.value_cells(), memory.layout_cache.borrow().cells());
    }
}

#[test]
fn flat_reset_poison_keeps_storage_and_invalidates_opaque_pointer_bytes() {
    let (mut types, ty, value) = fixture(true, 128);
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    let first = get(&mut memory, &types, &pool, true, 16);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let byte_pointer = memory
        .cast_pointer(&types, &first, byte, CastMode::Checked)
        .unwrap();
    let pointer_type = types.pointer(byte).unwrap();
    let slot = memory
        .cast_pointer(&types, &first, pointer_type, CastMode::Checked)
        .unwrap();
    memory
        .store(&types, &slot, Value::Pointer(byte_pointer.clone()))
        .unwrap();
    memory
        .pool_operation(&types, &pool, true, PoolOperation::Reset(true))
        .unwrap();
    assert_eq!(
        memory
            .load(&types, &byte_pointer)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        0xcc
    );
    assert!(matches!(
        memory.load(&types, &slot),
        Err(Error::InvalidIr(
            "byte storage cannot forge pointer provenance"
        ))
    ));
    let reused = get(&mut memory, &types, &pool, true, 16);
    assert!(memory.same_address(&types, &first, &reused).unwrap());
    memory
        .pool_operation(&types, &pool, true, PoolOperation::Release)
        .unwrap();
    assert_eq!(
        memory.load(&types, &byte_pointer),
        Err(Error::DanglingPointer)
    );
}

#[test]
fn descriptor_aliases_share_ownership_and_copies_cannot_adopt_blocks() {
    let (types, ty, value) = fixture(true, 128);
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    let first = get(&mut memory, &types, &pool, true, 8);
    let alias = memory
        .cast_pointer(&types, &pool, ty, CastMode::Unchecked)
        .unwrap();
    let second = get(&mut memory, &types, &alias, true, 8);
    assert_ne!(first, second);
    let copied = memory.load(&types, &pool).unwrap();
    let clone = memory.allocate(&types, ty, Some(copied)).unwrap();
    assert!(matches!(
        memory.pool_operation(&types, &clone, true, PoolOperation::Release),
        Err(Error::InvalidIr(_))
    ));
    memory
        .pool_operation(&types, &alias, true, PoolOperation::Release)
        .unwrap();
    assert_eq!(memory.allocation_count(), 2);
}

#[test]
fn pool_limits_and_corruption_fail_before_new_allocation_or_cursor_changes() {
    let (types, ty, value) = fixture(false, 128);
    let mut memory = Memory::new(Limits {
        value_cells: 80,
        ..Limits::default()
    });
    let pool = memory.allocate(&types, ty, Some(value.clone())).unwrap();
    admit_pool_layouts(&memory, &types, &pool);
    let cells = memory.value_cells();
    assert_eq!(
        memory.pool_operation(&types, &pool, false, PoolOperation::Get(8)),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), cells);
    assert_eq!(memory.allocation_count(), 1);
    assert_eq!(memory.load(&types, &pool).unwrap(), value);
    let void = types.void();
    assert!(
        memory
            .pool_operation(&types, &Pointer::null(ty), false, PoolOperation::Get(0))
            .unwrap()
            .unwrap()
            .is_null()
    );
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    let block = get(&mut memory, &types, &pool, false, 8);
    memory
        .store(
            &types,
            &memory.field(&types, &pool, 2).unwrap(),
            Value::Pointer(Pointer::null(void)),
        )
        .unwrap();
    let count = memory.allocation_count();
    assert!(matches!(
        memory.pool_operation(&types, &pool, false, PoolOperation::Get(8)),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(memory.allocation_count(), count);
    assert!(!block.is_null());
}

#[test]
fn nested_pool_descriptors_require_inner_finish_and_failed_outer_release_rolls_back() {
    let (types, ty, value) = fixture(true, 256);
    let mut memory = Memory::new(Limits::default());
    let outer = memory.allocate(&types, ty, Some(value)).unwrap();
    let region = get(&mut memory, &types, &outer, true, 40);
    let inner = memory
        .cast_pointer(&types, &region, ty, CastMode::Checked)
        .unwrap();
    memory.write_pool_integer(&types, &inner, 0, 64).unwrap();
    let child = get(&mut memory, &types, &inner, true, 8);
    let before = memory.value_cells();
    assert!(
        memory
            .pool_operation(&types, &outer, true, PoolOperation::Release)
            .is_err()
    );
    assert_eq!(memory.value_cells(), before);
    assert_eq!(memory.allocation_count(), 3);
    assert!(
        memory
            .pool_operation(&types, &outer, true, PoolOperation::Reset(true))
            .is_err()
    );
    assert_eq!(memory.value_cells(), before);
    memory
        .pool_operation(&types, &inner, true, PoolOperation::Release)
        .unwrap();
    memory
        .pool_operation(&types, &outer, true, PoolOperation::Release)
        .unwrap();
    assert_eq!(
        memory.byte_set(&types, &child, 0, 1),
        Err(Error::DanglingPointer)
    );
    assert_eq!(memory.allocation_count(), 1);
}

#[test]
fn work_preflight_covers_default_capacity_without_materializing_root_images() {
    let (types, ty, value) = fixture(true, 0);
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    admit_pool_layouts(&memory, &types, &pool);
    let before = memory.value_cells();
    let cost = memory
        .pool_work_cost(&types, &pool, true, PoolOperation::Get(1))
        .unwrap();
    assert!(cost > 65536);
    assert_eq!(memory.value_cells(), before);
    assert!(memory.allocation(&pool).unwrap().image.borrow().is_none());
    assert_eq!(
        memory
            .pool_work_cost(&types, &Pointer::null(ty), true, PoolOperation::Get(0))
            .unwrap(),
        0
    );
}

#[test]
fn pool_requested_region_cannot_be_widened_by_casting_to_its_string_backing() {
    let (types, ty, value) = fixture(true, 128);
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    let first = get(&mut memory, &types, &pool, true, 8);
    let second = get(&mut memory, &types, &pool, true, 8);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let second_byte = memory
        .cast_pointer(&types, &second, byte, CastMode::Checked)
        .unwrap();
    memory
        .store(
            &types,
            &second_byte,
            Value::Int(Integer::wrapping(IntegerType::U8, 42)),
        )
        .unwrap();
    let escape = memory
        .cast_pointer(&types, &first, types.string(), CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.byte_set(&types, &escape, 0, 128),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(
        memory
            .load(&types, &second_byte)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        42
    );
}

#[test]
fn zeroed_cursor_fields_do_not_change_a_live_descriptors_nominal_owner() {
    let (mut types, ty, value) = fixture(true, 128);
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let void = types.void();
    let pointer = types.pointer(void).unwrap();
    let other = types.reserve_record(RecordKind::Struct);
    types
        .define_record(other, [integer, integer, pointer, integer, integer])
        .unwrap();
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    get(&mut memory, &types, &pool, true, 8);
    memory.write_pool_cursor(&types, &pool, None, 0).unwrap();
    let forged = memory
        .cast_pointer(&types, &pool, other, CastMode::Unchecked)
        .unwrap();
    assert_eq!(
        memory.pool_operation(&types, &forged, true, PoolOperation::Get(8)),
        Err(Error::InvalidIr(
            "pool descriptor is already owned by another nominal pool type"
        ))
    );
    assert_eq!(memory.allocation_count(), 2);
}

#[test]
fn retained_blocks_can_be_reused_and_freed_when_new_capacity_would_exceed_the_limit() {
    let (types, ty, value) = fixture(true, 64);
    let mut memory = Memory::new(Limits {
        value_cells: 160,
        ..Limits::default()
    });
    let pool = memory.allocate(&types, ty, Some(value)).unwrap();
    get(&mut memory, &types, &pool, true, 8);
    memory.write_pool_integer(&types, &pool, 0, 65536).unwrap();
    get(&mut memory, &types, &pool, true, 8);
    let before = memory.load(&types, &pool).unwrap();
    assert_eq!(
        memory.pool_operation(&types, &pool, true, PoolOperation::Get(128)),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.load(&types, &pool).unwrap(), before);
    memory
        .pool_operation(&types, &pool, true, PoolOperation::Reset(false))
        .unwrap();
    memory
        .store(&types, &memory.field(&types, &pool, 4).unwrap(), number(-1))
        .unwrap();
    memory
        .pool_operation(&types, &pool, true, PoolOperation::Release)
        .unwrap();
    assert_eq!(memory.allocation_count(), 1);
}

#[test]
fn readonly_descriptors_fail_before_allocating_owned_blocks() {
    let (types, ty, value) = fixture(true, 128);
    let mut memory = Memory::new(Limits::default());
    let pool = memory.allocate(&types, ty, Some(value.clone())).unwrap();
    memory.freeze(&pool).unwrap();
    assert_eq!(
        memory.pool_work_cost(&types, &pool, true, PoolOperation::Get(8)),
        Err(Error::ReadOnlyStorage)
    );
    assert_eq!(
        memory.pool_operation(&types, &pool, true, PoolOperation::Get(8)),
        Err(Error::ReadOnlyStorage)
    );
    assert_eq!(memory.allocation_count(), 1);
    assert_eq!(memory.load(&types, &pool).unwrap(), value);
}

#[test]
fn address_derived_configuration_fails_before_snapshot_or_block_allocation() {
    for field in [0, 4] {
        let (types, ty, value) = fixture(true, 128);
        let mut memory = Memory::new(Limits::default());
        let pool = memory.allocate(&types, ty, Some(value)).unwrap();
        let address = memory
            .pointer_to_integer(&types, &pool, IntegerType::S64, CastMode::Unchecked)
            .unwrap()
            .into_value();
        memory
            .store(
                &types,
                &memory.field(&types, &pool, field).unwrap(),
                address,
            )
            .unwrap();
        let before = memory.load(&types, &pool).unwrap();
        let cells = memory.value_cells();
        let failure = Err(Error::UnsupportedPointerOperation(
            "pool configuration must use portable integers",
        ));
        assert_eq!(
            memory.pool_work_cost(&types, &pool, true, PoolOperation::Get(8)),
            failure
        );
        assert_eq!(
            memory.pool_operation(&types, &pool, true, PoolOperation::Get(8)),
            Err(Error::UnsupportedPointerOperation(
                "pool configuration must use portable integers"
            ))
        );
        assert_eq!(memory.allocation_count(), 1);
        assert_eq!(memory.value_cells(), cells);
        assert_eq!(memory.load(&types, &pool).unwrap(), before);
    }
}

#[test]
fn cold_copied_aggregate_configuration_uses_retained_storage_without_materializing_it() {
    let (mut types, ty, value) = fixture(true, 128);
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let outer = types.reserve_record(RecordKind::Struct);
    types.define_record(outer, [integer, ty]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let image =
        ByteImage::encode(&types, memory.target, ty, &value, memory.limits.value_cells).unwrap();
    let copy = Value::StoredAggregate(
        crate::StoredAggregate::new(ty, value, image, memory.limits.value_cells).unwrap(),
    );
    for (container, field) in [
        (
            memory.allocate(&types, ty, Some(copy.clone())).unwrap(),
            None,
        ),
        (
            memory
                .allocate(
                    &types,
                    outer,
                    Some(Value::Record {
                        ty: outer,
                        fields: vec![number(7), copy],
                    }),
                )
                .unwrap(),
            Some(1),
        ),
    ] {
        let pool = field.map_or_else(
            || container.clone(),
            |index| memory.field(&types, &container, index).unwrap(),
        );
        admit_pool_layouts(&memory, &types, &pool);
        let cells = memory.value_cells();
        assert!(
            memory
                .pool_work_cost(&types, &pool, true, PoolOperation::Get(8))
                .unwrap()
                > 128
        );
        assert_eq!(memory.value_cells(), cells);
        assert!(
            memory
                .allocation(&container)
                .unwrap()
                .image
                .borrow()
                .is_none()
        );
        get(&mut memory, &types, &pool, true, 8);
        memory
            .pool_operation(&types, &pool, true, PoolOperation::Release)
            .unwrap();
    }
}

#[test]
fn failure_after_block_allocation_restores_storage_and_never_reuses_its_identity() {
    let (types, ty, _) = fixture(true, 8);
    let mut bytes = vec![0u8; 40];
    bytes[..8].copy_from_slice(&8i64.to_le_bytes());
    bytes[32..].copy_from_slice(&8i64.to_le_bytes());
    let value = Value::String(bytes);
    let mut memory = Memory::new(Limits::default());
    let storage = memory
        .allocate(&types, types.string(), Some(value.clone()))
        .unwrap();
    let pool = memory
        .cast_pointer(&types, &storage, ty, CastMode::Unchecked)
        .unwrap();
    admit_pool_layouts(&memory, &types, &pool);
    // Admit the existing facts first, then permit exactly the new block and
    // ledger. The new pointer relocation must fail the cumulative image cap.
    memory.limits.value_cells = memory.value_cells() + STATE_CELLS + BLOCK_CELLS + 9;
    let next = memory.next_allocation;
    let cells = memory.value_cells();
    assert_eq!(
        memory.pool_operation(&types, &pool, true, PoolOperation::Get(8)),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert!(
        memory.next_allocation > next,
        "fixture must fail after allocating the block"
    );
    assert_eq!(memory.allocation_count(), 1);
    assert_eq!(memory.value_cells(), cells);
    assert!(memory.pool_ledger.states.is_empty());
    assert!(memory.pool_ledger.owners.is_empty());
    assert!(memory.pool_ledger.identities.is_empty());
    // Inspect without creating an image that would exceed the intentionally tiny limit.
    assert_eq!(
        memory.allocation(&pool).unwrap().value.as_ref(),
        Some(&value)
    );
    let retired = memory.next_allocation;
    memory.limits.value_cells = 1024;
    let live = get(&mut memory, &types, &pool, true, 8);
    assert!(live.allocation_key().1 >= retired);
    memory
        .pool_operation(&types, &pool, true, PoolOperation::Release)
        .unwrap();
}
