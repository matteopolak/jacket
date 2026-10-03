use super::*;
use jai_types::{
    CallingConvention, ContextMode, ProcedureType, RecordKind, TypeRegistry, Variadic,
};

fn int(bits: u64) -> Value {
    Value::Int(Integer::wrapping(IntegerType::U64, i128::from(bits)))
}
fn fork(memory: &Memory) -> Memory {
    memory
        .fork_private_branch(memory.snapshot_work_cost().unwrap())
        .unwrap()
}

#[test]
fn branch_writes_release_and_high_water_counters_are_private() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut parent = Memory::new(Limits::default());
    let pointer = parent.allocate(&types, word, Some(int(17))).unwrap();
    let parent_layout = parent.prepared_layout(&types, word).unwrap();
    let mut child = fork(&parent);
    assert_eq!(child.identity, parent.identity);
    assert_eq!(child.next_allocation, parent.next_allocation);
    assert_eq!(
        child.next_virtual_address.get(),
        parent.next_virtual_address.get()
    );
    assert!(std::sync::Arc::ptr_eq(
        &parent_layout,
        &child.prepared_layout(&types, word).unwrap()
    ));
    child.store(&types, &pointer, int(99)).unwrap();
    assert_eq!(parent.load(&types, &pointer).unwrap(), int(17));
    assert_eq!(child.load(&types, &pointer).unwrap(), int(99));
    child.release(&pointer).unwrap();
    assert_eq!(child.load(&types, &pointer), Err(Error::DanglingPointer));
    assert_eq!(parent.load(&types, &pointer).unwrap(), int(17));
    let next = child.allocate(&types, word, Some(int(3))).unwrap();
    assert_ne!(next.allocation_key(), pointer.allocation_key());
    assert!(child.next_virtual_address.get() > parent.next_virtual_address.get());
    assert_eq!(parent.allocation_count(), 1);
    assert_eq!(parent.next_allocation, 2);
}

#[test]
fn initialized_and_unknown_bytes_clone_without_shared_mutability() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [word, word]).unwrap();
    let mut parent = Memory::new(Limits::default());
    let root = parent.allocate(&types, record, None).unwrap();
    let first = parent.field(&types, &root, 0).unwrap();
    let second = parent.field(&types, &root, 1).unwrap();
    parent.store(&types, &first, int(42)).unwrap();
    let mut child = fork(&parent);
    assert_eq!(child.load(&types, &second), Err(Error::Uninitialized));
    child.store(&types, &second, int(87)).unwrap();
    child.store(&types, &first, int(1)).unwrap();
    assert_eq!(parent.load(&types, &first).unwrap(), int(42));
    assert_eq!(parent.load(&types, &second), Err(Error::Uninitialized));
    assert_eq!(child.load(&types, &second).unwrap(), int(87));
}

#[test]
fn data_pointer_and_code_receipts_keep_inherited_branch_provenance() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut parent = Memory::new(Limits::default());
    let data = parent.allocate(&types, word, Some(int(42))).unwrap();
    let slot = parent
        .allocate(&types, pointer_ty, Some(Value::Pointer(data.clone())))
        .unwrap();
    parent
        .ensure_image(&types, parent.allocation(&slot).unwrap())
        .unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let procedure = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(47)),
    };
    let code_slot = parent
        .allocate(&types, signature, Some(procedure.clone()))
        .unwrap();
    parent
        .ensure_image(&types, parent.allocation(&code_slot).unwrap())
        .unwrap();
    let code = parent.code_pointer(&types, &procedure).unwrap();
    let mut child = fork(&parent);
    let child_data = child
        .load(&types, &slot)
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    child.store(&types, &child_data, int(77)).unwrap();
    assert_eq!(parent.load(&types, &data).unwrap(), int(42));
    assert_eq!(child.load(&types, &data).unwrap(), int(77));
    child.validate_code_pointer(&types, code).unwrap();
    assert_eq!(child.code_pointer(&types, &procedure).unwrap(), code);
    assert_eq!(child.load(&types, &code_slot).unwrap(), procedure);
    child
        .handle_tokens
        .borrow_mut()
        .values
        .remove(&HandleKey::Procedure {
            signature,
            procedure: code.procedure(),
        });
    assert_eq!(
        child.validate_code_pointer(&types, code),
        Err(Error::DanglingPointer)
    );
    parent.validate_code_pointer(&types, code).unwrap();
}

#[test]
fn precharge_counts_retired_table_capacity_and_denies_before_clone() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut parent = Memory::new(Limits::default());
    let pointers: Vec<_> = (0..256)
        .map(|_| parent.allocate(&types, word, None).unwrap())
        .collect();
    for pointer in pointers {
        parent.release(&pointer).unwrap();
    }
    let cost = parent.snapshot_work_cost().unwrap();
    assert!(cost > parent.value_cells() * 3);
    assert!(cost >= parent.allocations.capacity());
    let count = parent.next_allocation;
    let address = parent.next_virtual_address.get();
    let cache = parent.prepared_layout(&types, word).unwrap();
    let references = std::sync::Arc::strong_count(&cache);
    assert!(matches!(
        parent.fork_private_branch(cost - 1),
        Err(Error::Limit(LimitKind::Fuel))
    ));
    assert_eq!(std::sync::Arc::strong_count(&cache), references);
    assert_eq!(parent.next_allocation, count);
    assert_eq!(parent.next_virtual_address.get(), address);
    let child = parent.fork_private_branch(cost).unwrap();
    assert_eq!(child.allocation_count(), 0);
    assert_eq!(child.value_cells(), parent.value_cells());
    assert_eq!(std::sync::Arc::strong_count(&cache), references + 1);
}

#[test]
fn snapshot_cost_covers_both_semantic_storage_and_image_mask() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let array = types.fixed_array(word, 128).unwrap();
    let mut parent = Memory::new(Limits::default());
    let root = parent
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![int(9); 128],
            }),
        )
        .unwrap();
    parent
        .ensure_image(&types, parent.allocation(&root).unwrap())
        .unwrap();
    let allocation = parent.allocation(&root).unwrap();
    let image = allocation.image.borrow();
    let image = image.as_ref().unwrap();
    let copied = allocation
        .value
        .as_ref()
        .unwrap()
        .cells(parent.limits.value_cells)
        .unwrap()
        + image.len() * 2
        + image.metadata_cells();
    assert!(parent.snapshot_work_cost().unwrap() >= copied);
    let child = fork(&parent);
    assert_eq!(
        child.allocation(&root).unwrap().image.borrow().as_ref(),
        Some(image)
    );
}

#[test]
fn snapshot_cost_overflow_is_a_structured_fuel_error() {
    let parent = Memory::new(Limits::default());
    parent.cells.set(usize::MAX);
    assert_eq!(
        parent.snapshot_work_cost(),
        Err(Error::Limit(LimitKind::Fuel))
    );
    assert!(matches!(
        parent.fork_private_branch(usize::MAX),
        Err(Error::Limit(LimitKind::Fuel))
    ));
}
