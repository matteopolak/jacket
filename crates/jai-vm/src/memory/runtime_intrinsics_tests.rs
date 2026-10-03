use super::*;
use jai_types::{
    DistinctKind, Integer, IntegerType, LayoutPolicy, RecordKind, ScalarLayout, TypeRegistry,
};

fn integer(ty: IntegerType, value: i128) -> Value {
    Value::Int(Integer::wrapping(ty, value))
}
fn byte_array(types: &mut TypeRegistry, bytes: &[u8]) -> (TypeId, Value) {
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let ty = types.fixed_array(byte, bytes.len() as u64).unwrap();
    (
        ty,
        Value::Array {
            ty,
            elements: bytes
                .iter()
                .map(|&byte| integer(IntegerType::U8, i128::from(byte)))
                .collect(),
        },
    )
}

#[test]
fn copying_byte_subranges_and_comparing_unsigned_bytes() {
    let mut types = TypeRegistry::new();
    let (ty, initial) = byte_array(&mut types, &[1, 2, 200, 4]);
    let (_, empty) = byte_array(&mut types, &[0, 0, 0, 0]);
    let mut memory = Memory::new(Limits::default());
    let source = memory.allocate(&types, ty, Some(initial)).unwrap();
    let destination = memory.allocate(&types, ty, Some(empty)).unwrap();
    let source_bytes = memory.sequence_data(&types, &source).unwrap();
    let destination_bytes = memory.sequence_data(&types, &destination).unwrap();
    let destination_second = memory.offset(&types, &destination_bytes, 1).unwrap();
    memory
        .byte_copy(&types, &destination_second, &source_bytes, 3)
        .unwrap();
    assert_eq!(
        memory.load(&types, &destination).unwrap(),
        byte_array(&mut types, &[0, 1, 2, 200]).1
    );
    assert!(
        memory
            .byte_compare(&types, &source_bytes, &destination_bytes, 4)
            .unwrap()
            > 0
    );
    assert_eq!(
        memory
            .byte_compare(&types, &destination_second, &source_bytes, 3)
            .unwrap(),
        0
    );
    memory
        .byte_set(&types, &destination_second, 255, 3)
        .unwrap();
    assert!(
        memory
            .byte_compare(&types, &source_bytes, &destination_second, 3)
            .unwrap()
            < 0
    );
}

#[test]
fn whole_writes_initialize_storage_but_partial_writes_and_bad_extents_are_atomic() {
    let mut types = TypeRegistry::new();
    let (ty, expected) = byte_array(&mut types, &[9, 9, 9, 9]);
    let mut memory = Memory::new(Limits::default());
    let destination = memory.allocate(&types, ty, None).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let bytes = memory
        .cast_pointer(&types, &destination, byte, CastMode::Unchecked)
        .unwrap();
    assert_eq!(
        memory.byte_set(&types, &bytes, 9, 3),
        Err(Error::Uninitialized)
    );
    assert_eq!(memory.load(&types, &destination), Err(Error::Uninitialized));
    memory.byte_set(&types, &bytes, 9, 4).unwrap();
    assert_eq!(memory.load(&types, &destination).unwrap(), expected);
    let second = memory.offset(&types, &bytes, 1).unwrap();
    assert!(matches!(
        memory.byte_set(&types, &second, 7, 4),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(memory.load(&types, &destination).unwrap(), expected);
    assert!(memory.byte_copy(&types, &second, &bytes, 3).is_err());
    assert_eq!(memory.load(&types, &destination).unwrap(), expected);
}

#[test]
fn byte_operations_preserve_partial_initialization_created_by_typed_field_writes() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [byte, byte]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let source = memory.allocate(&types, record, None).unwrap();
    let source_first = memory.field(&types, &source, 0).unwrap();
    memory
        .store(&types, &source_first, integer(IntegerType::U8, 42))
        .unwrap();
    let source_bytes = memory
        .cast_pointer(&types, &source, byte, CastMode::Unchecked)
        .unwrap();
    let destination = memory
        .allocate(
            &types,
            record,
            Some(Value::Record {
                ty: record,
                fields: vec![integer(IntegerType::U8, 0), integer(IntegerType::U8, 9)],
            }),
        )
        .unwrap();
    let destination_bytes = memory
        .cast_pointer(&types, &destination, byte, CastMode::Unchecked)
        .unwrap();
    memory
        .byte_copy(&types, &destination_bytes, &source_bytes, 2)
        .unwrap();
    assert_eq!(
        memory
            .byte_compare(&types, &source_bytes, &destination_bytes, 1)
            .unwrap(),
        0
    );
    assert_eq!(
        memory.byte_compare(&types, &source_bytes, &destination_bytes, 2),
        Err(Error::Uninitialized)
    );
    let second = memory.field(&types, &destination, 1).unwrap();
    assert_eq!(memory.load(&types, &second), Err(Error::Uninitialized));
    assert_eq!(memory.load(&types, &destination), Err(Error::Uninitialized));
    assert!(matches!(
        memory.byte_set(&types, &destination_bytes, 7, 3),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(memory.load(&types, &second), Err(Error::Uninitialized));
    memory.byte_set(&types, &second, 9, 1).unwrap();
    assert_eq!(
        memory.load(&types, &destination).unwrap(),
        Value::Record {
            ty: record,
            fields: vec![integer(IntegerType::U8, 42), integer(IntegerType::U8, 9)],
        }
    );
    assert_eq!(memory.load(&types, &source), Err(Error::Uninitialized));
}

#[test]
fn null_zero_count_and_readonly_or_foreign_storage() {
    let mut types = TypeRegistry::new();
    let (ty, value) = byte_array(&mut types, &[1, 2]);
    let void = types.void();
    let null = Pointer::null(void);
    let mut memory = Memory::new(Limits::default());
    assert_eq!(memory.byte_set(&types, &null, 9, 0), Ok(()));
    assert_eq!(memory.byte_copy(&types, &null, &null, 0), Ok(()));
    assert_eq!(memory.byte_compare(&types, &null, &null, 0), Ok(0));
    assert_eq!(
        memory.byte_set(&types, &null, 9, 1),
        Err(Error::NullPointer)
    );
    let root = memory.allocate(&types, ty, Some(value.clone())).unwrap();
    let bytes = memory.sequence_data(&types, &root).unwrap();
    memory.freeze(&root).unwrap();
    assert_eq!(
        memory.byte_set(&types, &bytes, 9, 1),
        Err(Error::ReadOnlyStorage)
    );
    let mut foreign = Memory::new(Limits::default());
    let other = foreign.allocate(&types, ty, Some(value)).unwrap();
    assert_eq!(
        memory.byte_compare(&types, &bytes, &other, 1),
        Err(Error::ForeignPointer)
    );
}

#[test]
fn copying_handles_preserves_provenance_and_byte_filling_cannot_forge_handles() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let target = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::S64, 42)))
        .unwrap();
    let source = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(target.clone())))
        .unwrap();
    let destination = memory.allocate(&types, pointer_ty, None).unwrap();
    memory.byte_copy(&types, &destination, &source, 8).unwrap();
    assert_eq!(
        memory.load(&types, &destination).unwrap(),
        Value::Pointer(target)
    );
    memory.byte_set(&types, &destination, 0x77, 8).unwrap();
    assert!(memory.load(&types, &destination).is_err());
    memory.byte_set(&types, &destination, 0, 8).unwrap();
    assert_eq!(
        memory.load(&types, &destination).unwrap(),
        Value::Pointer(Pointer::null(integer_ty))
    );
}

#[test]
fn address_derived_fill_bytes_keep_provenance_after_raw_copy() {
    let types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let mut memory = Memory::new(Limits::default());
    let origin = memory
        .allocate(&types, byte, Some(integer(IntegerType::U8, 1)))
        .unwrap();
    let address_byte = crate::Number::address(
        Integer::wrapping(IntegerType::U8, 0),
        crate::AddressProvenance::Derived {
            memory: origin.memory_identity(),
            allocations: vec![origin.allocation_key().1].into_boxed_slice(),
        },
    );
    let filled = memory
        .allocate(&types, byte, Some(integer(IntegerType::U8, 9)))
        .unwrap();
    let copied = memory
        .allocate(&types, byte, Some(integer(IntegerType::U8, 7)))
        .unwrap();
    memory
        .byte_set_number(&types, &filled, &address_byte, 1)
        .unwrap();
    memory.byte_copy(&types, &copied, &filled, 1).unwrap();
    for pointer in [&filled, &copied] {
        let number = memory.load(&types, pointer).unwrap().number().unwrap();
        assert_eq!(number.bits(), 0);
        assert_eq!(number.provenance(), address_byte.provenance());
        assert!(matches!(
            number.portable_integer(),
            Err(Error::UnsupportedPointerOperation(_))
        ));
    }
}

#[test]
fn code_address_fill_and_copy_keep_nonportable_provenance_without_data_origins() {
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
    let opaque = types.pointer(types.void()).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let procedure = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(97)),
    };
    let ilp32 = LayoutPolicy::new(
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
    .unwrap();
    for policy in [ilp32, LayoutPolicy::lp64()] {
        let mut memory = Memory::with_target(
            Limits::default(),
            ByteTarget {
                policy,
                endian: crate::Endian::Little,
            },
        );
        let mut image =
            ByteImage::encode(&types, memory.target(), signature, &procedure, 1_000).unwrap();
        memory.retokenize_image(&types, &mut image).unwrap();
        memory.certify_code_image(&types, &mut image).unwrap();
        let code = image.read(&types, memory.target(), 0, opaque).unwrap();
        let address = memory
            .pointer_to_integer(
                &types,
                code.pointer().unwrap(),
                IntegerType::U64,
                CastMode::Unchecked,
            )
            .unwrap();
        let address_byte = image
            .read(&types, memory.target(), 0, byte)
            .unwrap()
            .number()
            .unwrap();
        let filled = memory
            .allocate(&types, byte, Some(integer(IntegerType::U8, 0)))
            .unwrap();
        let copied = memory
            .allocate(&types, byte, Some(integer(IntegerType::U8, 0)))
            .unwrap();
        memory
            .byte_set_number(&types, &filled, &address_byte, 1)
            .unwrap();
        memory.byte_copy(&types, &copied, &filled, 1).unwrap();
        for pointer in [&filled, &copied] {
            let fragment = memory.load(&types, pointer).unwrap().number().unwrap();
            assert_eq!(fragment.bits(), address.bits() & 255);
            assert!(
                matches!(fragment.provenance(), Some(crate::AddressProvenance::Derived { allocations, .. }) if allocations.is_empty())
            );
            assert!(matches!(
                fragment.portable_integer(),
                Err(Error::UnsupportedPointerOperation(_))
            ));
            assert!(matches!(
                memory.integer_to_pointer(&types, fragment, types.void(), CastMode::Unchecked),
                Err(Error::UnsupportedPointerOperation(_))
            ));
        }
    }
}

#[test]
fn address_integer_atomic_operands_and_observed_values_reject_without_writes() {
    let types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::U64));
    let mut memory = Memory::new(Limits::default());
    let origin = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::U64, 42)))
        .unwrap();
    let address = memory
        .pointer_to_integer(&types, &origin, IntegerType::U64, CastMode::Unchecked)
        .unwrap();
    let addressed = address.clone().into_value();
    let literal = Value::Int(address.integer());
    let storage = memory
        .allocate(&types, integer_ty, Some(addressed.clone()))
        .unwrap();
    assert!(matches!(
        memory.compare_and_swap(&types, &storage, &literal, &integer(IntegerType::U64, 7)),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(memory.load(&types, &storage).unwrap(), addressed);
    let plain = memory
        .allocate(&types, integer_ty, Some(literal.clone()))
        .unwrap();
    for (expected, replacement) in [
        (&literal, &addressed),
        (&addressed, &integer(IntegerType::U64, 7)),
    ] {
        assert!(matches!(
            memory.compare_and_swap(&types, &plain, expected, replacement),
            Err(Error::UnsupportedPointerOperation(_))
        ));
        assert_eq!(memory.load(&types, &plain).unwrap(), literal);
    }
}

#[test]
fn equal_virtual_addresses_have_equal_bytes_across_separate_images_and_cast_views() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let target = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::S64, 42)))
        .unwrap();
    let first = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(target.clone())))
        .unwrap();
    let second = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(target.clone())))
        .unwrap();
    assert_eq!(memory.byte_compare(&types, &first, &second, 8).unwrap(), 0);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let alias = memory
        .cast_pointer(&types, &target, byte, CastMode::Unchecked)
        .unwrap();
    let alias = memory
        .cast_pointer(&types, &alias, integer_ty, CastMode::Unchecked)
        .unwrap();
    let third = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(alias)))
        .unwrap();
    assert_eq!(memory.byte_compare(&types, &first, &third, 8).unwrap(), 0);
}

#[test]
fn comparing_address_bytes_requires_complete_identity_proof() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let first = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::U64, 1)))
        .unwrap();
    let second = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::U64, 2)))
        .unwrap();
    let first_pointer = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(first.clone())))
        .unwrap();
    let second_pointer = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(second)))
        .unwrap();
    assert_eq!(
        memory.byte_compare(&types, &first_pointer, &first_pointer, 8),
        Ok(0)
    );
    for (right, count) in [(&second_pointer, 8), (&first_pointer, 1)] {
        assert!(matches!(
            memory.byte_compare(&types, &first_pointer, right, count),
            Err(Error::UnsupportedPointerOperation(_))
        ));
    }
    let address = memory
        .pointer_to_integer(&types, &first, IntegerType::U64, CastMode::Unchecked)
        .unwrap();
    let same_virtual_bits = memory
        .allocate(&types, integer_ty, Some(Value::Int(address.integer())))
        .unwrap();
    assert!(matches!(
        memory.byte_compare(&types, &first_pointer, &same_virtual_bits, 8),
        Err(Error::UnsupportedPointerOperation(_))
    ));
}

#[test]
fn typed_swap_preserves_handles_and_failure_leaves_both_values_unchanged() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let first = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::S64, 7)))
        .unwrap();
    let second = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::S64, 42)))
        .unwrap();
    let a = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(first.clone())))
        .unwrap();
    let b = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(second.clone())))
        .unwrap();
    memory.swap_values(&types, &a, &b).unwrap();
    assert_eq!(
        memory.load(&types, &a).unwrap(),
        Value::Pointer(second.clone())
    );
    assert_eq!(
        memory.load(&types, &b).unwrap(),
        Value::Pointer(first.clone())
    );
    let uninitialized = memory.allocate(&types, integer_ty, None).unwrap();
    assert_eq!(
        memory.swap_values(&types, &first, &uninitialized),
        Err(Error::Uninitialized)
    );
    assert_eq!(
        memory.load(&types, &first).unwrap(),
        integer(IntegerType::S64, 7)
    );
    memory.freeze(&second).unwrap();
    assert_eq!(
        memory.swap_values(&types, &first, &second),
        Err(Error::ReadOnlyStorage)
    );
    assert_eq!(
        memory.load(&types, &first).unwrap(),
        integer(IntegerType::S64, 7)
    );
    assert_eq!(
        memory.load(&types, &second).unwrap(),
        integer(IntegerType::S64, 42)
    );
}

#[test]
fn typed_swap_data_aliases_keep_work_stable_without_retained_handle_history() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S64));
    let array_ty = types.fixed_array(integer_ty, 64).unwrap();
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory.allocate(&types, array_ty, None).unwrap();
    let data = memory.sequence_data(&types, &root).unwrap();
    let slot = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(data.clone())))
        .unwrap();
    assert_eq!(memory.byte_compare(&types, &slot, &slot, 8), Ok(0));
    // Offset aliases demand the element layout without retaining new value storage.
    memory
        .prepare_layout(&types, integer_ty, usize::MAX)
        .1
        .unwrap();
    let initial_cells = memory.value_cells();
    let initial_cost = memory.swap_work_cost(&types, &slot, &slot).unwrap();
    assert!(memory.handle_tokens.borrow().values.is_empty());
    let initial_handle_capacity = memory.handle_tokens.borrow().values.capacity();
    for index in 1..64 {
        let alias = memory.offset(&types, &data, index).unwrap();
        memory.store(&types, &slot, Value::Pointer(alias)).unwrap();
        assert_eq!(memory.byte_compare(&types, &slot, &slot, 8), Ok(0));
    }
    assert_eq!(memory.value_cells(), initial_cells);
    assert!(memory.handle_tokens.borrow().values.is_empty());
    assert_eq!(
        memory.handle_tokens.borrow().values.capacity(),
        initial_handle_capacity
    );
    assert_eq!(
        memory.swap_work_cost(&types, &slot, &slot).unwrap(),
        initial_cost
    );
    memory.swap_values(&types, &slot, &slot).unwrap();
    assert_eq!(
        memory.load(&types, &slot).unwrap(),
        Value::Pointer(memory.offset(&types, &data, 63).unwrap())
    );
}

#[test]
fn typed_swap_charges_retained_procedure_ledger_capacity_with_stable_live_cells() {
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
    let procedure = |index| Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(index)),
    };
    let mut memory = Memory::new(Limits::default());
    let slot = memory
        .allocate(&types, signature, Some(procedure(0)))
        .unwrap();
    assert_eq!(memory.byte_compare(&types, &slot, &slot, 8), Ok(0));
    let initial_cost = memory.swap_work_cost(&types, &slot, &slot).unwrap();
    let initial_cells = memory.value_cells();
    let initial_capacity = memory.handle_tokens.borrow().values.capacity();
    for index in 1..64 {
        memory.store(&types, &slot, procedure(index)).unwrap();
        assert_eq!(memory.byte_compare(&types, &slot, &slot, 8), Ok(0));
    }
    assert_eq!(memory.value_cells(), initial_cells);
    assert_eq!(memory.handle_tokens.borrow().values.len(), 64);
    let retained_capacity = memory.handle_tokens.borrow().values.capacity();
    assert!(retained_capacity > initial_capacity);
    assert_eq!(
        memory.swap_work_cost(&types, &slot, &slot).unwrap() - initial_cost,
        u64::try_from(retained_capacity - initial_capacity).unwrap()
    );
    memory.swap_values(&types, &slot, &slot).unwrap();
    assert_eq!(memory.load(&types, &slot).unwrap(), procedure(63));
}

#[test]
fn typed_swap_rejects_partial_overlap_and_allows_identical_initialized_places() {
    let mut types = TypeRegistry::new();
    let (ty, initial) = byte_array(&mut types, &[1, 2, 3, 4, 5, 6, 7, 8]);
    let mut memory = Memory::new(Limits::default());
    let root = memory.allocate(&types, ty, Some(initial.clone())).unwrap();
    let bytes = memory.sequence_data(&types, &root).unwrap();
    let second_byte = memory.offset(&types, &bytes, 2).unwrap();
    let u32_ty = types.scalar(ScalarType::Int(IntegerType::U32));
    let first = memory
        .cast_pointer(&types, &bytes, u32_ty, CastMode::Unchecked)
        .unwrap();
    let second = memory
        .cast_pointer(&types, &second_byte, u32_ty, CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.swap_values(&types, &first, &second),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(memory.load(&types, &root).unwrap(), initial);
    memory.swap_values(&types, &first, &first).unwrap();
    assert_eq!(memory.load(&types, &root).unwrap(), initial);
}

#[test]
fn pointer_subobject_bounds_and_memory_limits_survive_byte_casts() {
    let mut types = TypeRegistry::new();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [s64, s64]).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            record,
            Some(Value::Record {
                ty: record,
                fields: vec![integer(IntegerType::S64, 1), integer(IntegerType::S64, 2)],
            }),
        )
        .unwrap();
    let field = memory.field(&types, &root, 0).unwrap();
    let void = memory
        .cast_pointer(&types, &field, types.void(), CastMode::Unchecked)
        .unwrap();
    assert!(matches!(
        memory.byte_set(&types, &void, 0, 9),
        Err(Error::OutOfBounds { .. })
    ));
    assert_eq!(
        memory.load(&types, &field).unwrap(),
        integer(IntegerType::S64, 1)
    );
    let mut bounded = Memory::new(Limits {
        value_cells: 8,
        ..Limits::default()
    });
    let source = bounded
        .allocate(&types, s64, Some(integer(IntegerType::S64, 1)))
        .unwrap();
    let destination = bounded.allocate(&types, s64, None).unwrap();
    assert_eq!(
        bounded.byte_copy(&types, &destination, &source, 8),
        Err(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(
        bounded.load(&types, &destination),
        Err(Error::Uninitialized)
    );
}

#[test]
fn compare_and_swap_returns_observed_value_and_respects_pointer_address_identity() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer_ty = types.pointer(integer_ty).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(&types, integer_ty, Some(integer(IntegerType::S64, 1)))
        .unwrap();
    assert_eq!(
        memory
            .compare_and_swap(
                &types,
                &root,
                &integer(IntegerType::S64, 2),
                &integer(IntegerType::S64, 3)
            )
            .unwrap(),
        (false, integer(IntegerType::S64, 1))
    );
    assert_eq!(
        memory
            .compare_and_swap(
                &types,
                &root,
                &integer(IntegerType::S64, 1),
                &integer(IntegerType::S64, 3)
            )
            .unwrap(),
        (true, integer(IntegerType::S64, 1))
    );
    assert_eq!(
        memory.load(&types, &root).unwrap(),
        integer(IntegerType::S64, 3)
    );
    let source = memory
        .allocate(&types, pointer_ty, Some(Value::Pointer(root.clone())))
        .unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let alias = memory
        .cast_pointer(&types, &root, byte, CastMode::Unchecked)
        .unwrap();
    let alias = memory
        .cast_pointer(&types, &alias, integer_ty, CastMode::Unchecked)
        .unwrap();
    let replacement = Value::Pointer(Pointer::null(integer_ty));
    assert!(
        memory
            .compare_and_swap(&types, &source, &Value::Pointer(alias), &replacement)
            .unwrap()
            .0
    );
    assert_eq!(memory.load(&types, &source).unwrap(), replacement);
}

#[test]
fn compare_and_swap_supports_nominal_variants_and_rejects_readonly_destinations() {
    let mut types = TypeRegistry::new();
    let integer_ty = types.scalar(ScalarType::Int(IntegerType::S32));
    let nominal = types.reserve_distinct(DistinctKind::Distinct);
    types.define_distinct(nominal, integer_ty).unwrap();
    let value = |n| Value::Distinct {
        ty: nominal,
        value: Box::new(integer(IntegerType::S32, n)),
    };
    let mut memory = Memory::new(Limits::default());
    let root = memory.allocate(&types, nominal, Some(value(1))).unwrap();
    assert_eq!(
        memory
            .compare_and_swap(&types, &root, &value(1), &value(2))
            .unwrap(),
        (true, value(1))
    );
    memory.freeze(&root).unwrap();
    assert_eq!(
        memory.compare_and_swap(&types, &root, &value(9), &value(3)),
        Err(Error::ReadOnlyStorage)
    );
}
