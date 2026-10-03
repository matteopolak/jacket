use super::*;
use crate::{AddressProvenance, Number};
use jai_types::{RecordKind, TypeRegistry};

#[test]
fn cold_snapshots_charge_the_root_and_warm_scalar_reads_charge_only_the_selection() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let array = types.fixed_array(word, 4096).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![Value::Int(Integer::wrapping(IntegerType::U64, 42)); 4096],
            }),
        )
        .unwrap();
    let first = memory.index(&types, &root, 0).unwrap();
    memory.prepare_layout(&types, word, usize::MAX).1.unwrap();
    let baseline = memory.value_cells();
    assert_eq!(memory.allocation(&root).unwrap().cells.get(), 4097);
    assert_eq!(memory.load_work_cost(&types, &first).unwrap(), 2);
    assert_eq!(
        memory
            .sequence_snapshot_work_cost(&types, &first, 8)
            .unwrap(),
        4096 * 8 + 4097 + 1,
    );
    // Preflight itself does not materialize the root or change its charge.
    assert_eq!(memory.value_cells(), baseline);
    assert!(memory.allocation(&root).unwrap().image.borrow().is_none());
    assert_eq!(
        memory.sequence_snapshot(&types, &first, 8).unwrap().len(),
        8
    );
    assert_eq!(
        memory
            .sequence_snapshot_work_cost(&types, &first, 8)
            .unwrap(),
        1
    );
    assert_eq!(memory.load_work_cost(&types, &first).unwrap(), 10);
    assert_eq!(
        memory
            .load(&types, &first)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        42
    );
}

#[test]
fn selected_decoded_numbers_charge_origins_without_charging_unrelated_bytes() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut memory = Memory::new(Limits::default());
    let number = Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        AddressProvenance::Derived {
            memory: memory.identity,
            allocations: (1..=128).collect(),
        },
    );
    let root = memory
        .allocate(&types, word, Some(number.into_value()))
        .unwrap();
    memory.sequence_snapshot(&types, &root, 8).unwrap();
    assert_eq!(memory.load_work_cost(&types, &root).unwrap(), 8 + 1 + 128);
}

#[test]
fn raw_zero_stride_aggregate_reads_are_precharged_before_expansion() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let array = types.fixed_array(empty, 10_000).unwrap();
    let text = types.string();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(&types, text, Some(Value::String(vec![0])))
        .unwrap();
    let view = memory
        .cast_pointer(&types, &root, array, CastMode::Checked)
        .unwrap();
    for ty in [array, empty] {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let baseline = memory.value_cells();
    assert_eq!(memory.allocation(&root).unwrap().cells.get(), 2);
    assert!(memory.load_work_cost(&types, &view).unwrap() >= 10_001);
    assert_eq!(memory.value_cells(), baseline);
    assert!(memory.allocation(&root).unwrap().image.borrow().is_none());
    assert_eq!(
        memory.load(&types, &view).unwrap().cells(20_000).unwrap(),
        10_001
    );
    assert!(memory.load_work_cost(&types, &view).unwrap() >= 10_001);
}

#[test]
fn empty_snapshots_do_not_inspect_invalid_backing() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let memory = Memory::new(Limits::default());
    assert_eq!(
        memory.sequence_snapshot_work_cost(&types, &Pointer::null(word), 0),
        Ok(0)
    );
}

#[test]
fn partial_stores_bound_whole_roots_and_zero_stride_values() {
    let mut types = TypeRegistry::new();
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let array = types.fixed_array(empty, 4096).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![
                    Value::Record {
                        ty: empty,
                        fields: vec![]
                    };
                    4096
                ],
            }),
        )
        .unwrap();
    let first = memory.index(&types, &root, 0).unwrap();
    assert_eq!(memory.store_work_cost(&types, &first).unwrap(), 4098);
    memory.freeze(&root).unwrap();
    assert_eq!(
        memory.store_work_cost(&types, &first),
        Err(Error::ReadOnlyStorage)
    );
}

#[test]
fn zero_work_counts_nodes_without_expanding_zero_stride_arrays_or_inactive_union_fields() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let empty = types.reserve_record(RecordKind::Struct);
    types.define_record(empty, []).unwrap();
    let array = types.fixed_array(empty, 100_000).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [word, array]).unwrap();
    let memory = Memory::new(Limits::default());
    assert_eq!(memory.zero_value_work_cost(&types, array), Ok(100_001));
    assert_eq!(memory.zero_value_work_cost(&types, union), Ok(2));
    assert_eq!(memory.allocation_count(), 0);
    assert_eq!(memory.value_cells(), 0);
}

#[test]
fn memoized_zero_shapes_recheck_depth_when_the_same_child_is_nested_further() {
    let mut types = TypeRegistry::new();
    let mut shared = types.reserve_record(RecordKind::Struct);
    types.define_record(shared, []).unwrap();
    for _ in 0..200 {
        let parent = types.reserve_record(RecordKind::Struct);
        types.define_record(parent, [shared]).unwrap();
        shared = parent;
    }
    let mut deeper = shared;
    for _ in 0..100 {
        let parent = types.reserve_record(RecordKind::Struct);
        types.define_record(parent, [deeper]).unwrap();
        deeper = parent;
    }
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, [shared, deeper]).unwrap();
    let memory = Memory::new(Limits::default());
    assert_eq!(
        memory.zero_value_work_cost(&types, root),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    );
    assert_eq!(memory.value_cells(), 0);
}
