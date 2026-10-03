use super::*;
use crate::{Limits, Value};
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};
fn byte(types: &TypeRegistry) -> jai_types::TypeId {
    types.scalar(ScalarType::Int(IntegerType::U8))
}
fn bytes(memory: &Memory, types: &TypeRegistry, pointer: &Pointer) -> Pointer {
    memory
        .cast_pointer(types, pointer, byte(types), CastMode::Checked)
        .unwrap()
}
#[test]
fn zero_size_and_null_contracts_have_real_lifetimes() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let mut heap = VirtualHeap::default();
    heap.free(&mut memory, &types, &Pointer::null(types.void()))
        .unwrap();
    let empty = heap.malloc(&mut memory, &types, 0).unwrap();
    assert!(!empty.is_null());
    assert_eq!(heap.allocation_count(), 1);
    assert_eq!(heap.live_bytes(), 0);
    assert!(
        memory
            .load(&types, &bytes(&memory, &types, &empty))
            .is_err()
    );
    let null = heap.realloc(&mut memory, &types, &empty, 0).unwrap();
    assert!(null.is_null());
    assert_eq!(heap.allocation_count(), 0);
    assert_eq!(
        heap.free(&mut memory, &types, &empty),
        Err(Error::DanglingPointer)
    );
}
#[test]
fn partial_initialization_grows_and_shrinks_without_creating_bytes() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    for ty in [types.string(), byte(&types)] {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let layout_cells = memory.value_cells();
    let mut heap = VirtualHeap::default();
    let original = heap.malloc(&mut memory, &types, 4).unwrap();
    let original_bytes = bytes(&memory, &types, &original);
    assert_eq!(
        memory.load(&types, &original_bytes),
        Err(Error::Uninitialized)
    );
    memory.byte_set(&types, &original_bytes, 7, 2).unwrap();
    let grown = heap.realloc(&mut memory, &types, &original, 8).unwrap();
    assert_eq!(
        memory.load(&types, &original_bytes),
        Err(Error::DanglingPointer)
    );
    let grown_bytes = bytes(&memory, &types, &grown);
    assert_eq!(
        memory.load(&types, &grown_bytes).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 7))
    );
    assert_eq!(
        memory.load(&types, &memory.offset(&types, &grown_bytes, 2).unwrap()),
        Err(Error::Uninitialized)
    );
    assert_eq!(
        memory.load(&types, &memory.offset(&types, &grown_bytes, 7).unwrap()),
        Err(Error::Uninitialized)
    );
    let small = heap.realloc(&mut memory, &types, &grown, 1).unwrap();
    assert_eq!(heap.live_bytes(), 1);
    assert_eq!(
        memory
            .load(&types, &bytes(&memory, &types, &small))
            .unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 7))
    );
    heap.free(&mut memory, &types, &small).unwrap();
    assert_eq!(memory.allocation_count(), 0);
    assert_eq!(memory.value_cells(), layout_cells);
}
#[test]
fn pointer_relocations_survive_resize_and_do_not_become_native_addresses() {
    let mut types = TypeRegistry::new();
    let pointer_type = types.pointer(byte(&types)).unwrap();
    let mut memory = Memory::new(Limits::default());
    let target = memory
        .allocate(
            &types,
            byte(&types),
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 9))),
        )
        .unwrap();
    let mut heap = VirtualHeap::default();
    let root = heap.malloc(&mut memory, &types, 16).unwrap();
    let slot = memory
        .cast_pointer(&types, &root, pointer_type, CastMode::Checked)
        .unwrap();
    memory
        .store(&types, &slot, Value::Pointer(target.clone()))
        .unwrap();
    let replacement = heap.realloc(&mut memory, &types, &root, 32).unwrap();
    let slot = memory
        .cast_pointer(&types, &replacement, pointer_type, CastMode::Checked)
        .unwrap();
    assert_eq!(memory.load(&types, &slot).unwrap(), Value::Pointer(target));
    let raw = bytes(&memory, &types, &replacement);
    assert!(matches!(
        memory.host_read_bytes(&types, &raw, 8),
        Err(Error::UnsupportedPointerOperation(_))
    ));
}
#[test]
fn exact_ledger_membership_rejects_other_frames_tokens_and_shifted_pointers() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let mut heap = VirtualHeap::default();
    let root = heap.malloc(&mut memory, &types, 8).unwrap();
    let address = memory
        .pointer_to_integer(&types, &root, IntegerType::U64, CastMode::Checked)
        .unwrap();
    let guessed = crate::Number::plain(address.integer());
    let guessed = memory
        .integer_to_pointer(&types, guessed, types.void(), CastMode::Unchecked)
        .unwrap();
    assert!(guessed.is_opaque());
    assert_eq!(
        heap.free(&mut memory, &types, &guessed),
        Err(Error::InvalidIr("pointer is not owned by the virtual heap"))
    );
    let shifted = memory
        .offset(&types, &bytes(&memory, &types, &root), 1)
        .unwrap();
    assert!(heap.free(&mut memory, &types, &shifted).is_err());
    let unrelated = memory.allocate(&types, byte(&types), None).unwrap();
    assert!(heap.free(&mut memory, &types, &unrelated).is_err());
    memory.freeze(&unrelated).unwrap();
    assert!(heap.free(&mut memory, &types, &unrelated).is_err());
    let file_type = types.reserve_record(jai_types::RecordKind::Struct);
    types.define_record(file_type, [byte(&types)]).unwrap();
    let mut files = crate::virtual_files::VirtualFiles::default();
    let file = files
        .opened(
            crate::host_effects::HostPath::new(
                crate::host_effects::FileRootId::allocate(),
                "fixture",
            )
            .unwrap(),
            crate::virtual_files::FileOpenMode::Read,
            vec![],
        )
        .unwrap();
    let mut tokens = crate::file_tokens::FileTokens::new(&types, file_type).unwrap();
    let token = tokens.mint(&types, &mut memory, file).unwrap();
    let token = memory
        .cast_pointer(&types, &token, types.void(), CastMode::Checked)
        .unwrap();
    assert!(heap.free(&mut memory, &types, &token).is_err());
    let mut foreign = Memory::new(Limits::default());
    assert_eq!(
        heap.free(&mut foreign, &types, &root),
        Err(Error::ForeignPointer)
    );
    heap.free(&mut memory, &types, &root).unwrap();
    assert_eq!(
        heap.free(&mut memory, &types, &root),
        Err(Error::DanglingPointer)
    );
}
#[test]
fn preflight_limits_and_transaction_snapshots_leave_prior_roots_live() {
    let types = TypeRegistry::new();
    let layouts = [types.string(), byte(&types)];
    let preparation = Memory::new(Limits::default());
    for ty in layouts {
        preparation
            .prepare_layout(&types, ty, usize::MAX)
            .1
            .unwrap();
    }
    let layout_cells = preparation.value_cells();
    let mut memory = Memory::new(Limits {
        value_cells: layout_cells + 24,
        ..Limits::default()
    });
    for ty in layouts {
        memory.prepare_layout(&types, ty, usize::MAX).1.unwrap();
    }
    let mut heap = VirtualHeap::new(HeapLimits {
        allocations: 2,
        bytes: 20,
    });
    let root = heap.malloc(&mut memory, &types, 8).unwrap();
    let view = bytes(&memory, &types, &root);
    memory.byte_set(&types, &view, 3, 8).unwrap();
    let memory_before = memory.snapshot();
    let heap_before = heap.clone();
    let baseline = memory.value_cells();
    assert!(heap.realloc_work_cost(&memory, &types, &root, 24).is_err());
    assert!(heap.realloc(&mut memory, &types, &root, 24).is_err());
    assert_eq!(heap.live_bytes(), 8);
    assert_eq!(memory.value_cells(), baseline);
    let transient = heap.realloc(&mut memory, &types, &root, 12).unwrap();
    memory.restore(memory_before);
    heap = heap_before;
    assert_eq!(
        memory.load(&types, &view).unwrap(),
        Value::Int(Integer::wrapping(IntegerType::U8, 3))
    );
    assert_eq!(
        heap.free(&mut memory, &types, &transient),
        Err(Error::DanglingPointer)
    );
    heap.free(&mut memory, &types, &root).unwrap();
    assert_eq!(memory.value_cells(), layout_cells);
}

#[test]
fn retained_relocation_budget_failure_keeps_the_original_allocation() {
    let mut types = TypeRegistry::new();
    let pointer_type = types.pointer(byte(&types)).unwrap();
    let mut memory = Memory::new(Limits {
        value_cells: 52,
        ..Limits::default()
    });
    let target = memory.allocate(&types, byte(&types), None).unwrap();
    let mut heap = VirtualHeap::default();
    let root = heap.malloc(&mut memory, &types, 16).unwrap();
    let slot = memory
        .cast_pointer(&types, &root, pointer_type, CastMode::Checked)
        .unwrap();
    memory
        .store(&types, &slot, Value::Pointer(target.clone()))
        .unwrap();
    let cells = memory.value_cells();
    assert_eq!(
        heap.realloc(&mut memory, &types, &root, 32),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(memory.value_cells(), cells);
    assert_eq!(memory.allocation_count(), 2);
    assert_eq!(heap.live_bytes(), 16);
    assert_eq!(memory.load(&types, &slot).unwrap(), Value::Pointer(target));
}

#[test]
fn fork_bounds_keep_retained_empty_table_capacity_and_charge_before_inspection() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let mut heap = VirtualHeap::default();
    let root = heap.malloc(&mut memory, &types, 8).unwrap();
    heap.free(&mut memory, &types, &root).unwrap();
    assert_eq!(heap.allocation_count(), 0);
    assert!(heap.roots.capacity() > 0);
    let expected = heap.roots.capacity() + 1;
    let mut charged = Vec::new();
    let cells = heap
        .fork_bounds(&mut |work| {
            charged.push(work);
            Ok(())
        })
        .unwrap();
    assert_eq!(cells, expected);
    assert_eq!(charged, vec![expected as u64]);
    let mut attempts = 0;
    assert_eq!(
        heap.fork_bounds(&mut |_| {
            attempts += 1;
            Err(Error::Limit(LimitKind::Fuel))
        }),
        Err(Error::Limit(LimitKind::Fuel))
    );
    assert_eq!(attempts, 1);
    assert_eq!(heap.roots.capacity() + 1, expected);
}

#[test]
fn fork_bounds_count_live_root_metadata_without_heap_payload() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let mut heap = VirtualHeap::default();
    let root = heap.malloc(&mut memory, &types, 8192).unwrap();
    let expected = 1
        + heap.roots.capacity()
        + heap
            .roots
            .keys()
            .map(Pointer::metadata_cells)
            .sum::<usize>();
    let mut charged = 0;
    assert_eq!(
        heap.fork_bounds(&mut |work| {
            charged += work;
            Ok(())
        })
        .unwrap(),
        expected
    );
    assert_eq!(charged, (heap.roots.capacity() + 1) as u64);
    assert!(expected < heap.live_bytes());
    assert_eq!(heap.live_bytes(), 8192);
    heap.free(&mut memory, &types, &root).unwrap();
}
