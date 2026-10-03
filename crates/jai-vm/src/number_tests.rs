use super::*;
use crate::{ArithmeticError, Error, Limits, Memory, scalar};
use jai_ir::CheckMode;
use jai_types::{CastMode, IntOp, ScalarType, TypeRegistry};

fn pointer(types: &TypeRegistry, memory: &mut Memory) -> Pointer {
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    memory
        .allocate(
            types,
            integer,
            Some(Value::Int(Integer::wrapping(IntegerType::S64, 1))),
        )
        .unwrap()
}
fn plain(value: i128) -> Number {
    Number::plain(Integer::wrapping(IntegerType::U64, value))
}
fn address(pointer: &Pointer, value: i128) -> Number {
    Number::address(
        Integer::wrapping(IntegerType::U64, value),
        AddressProvenance::Pointer(pointer.clone()),
    )
}
fn binary(op: IntOp, a: Number, b: Number) -> Number {
    scalar::binary_number(
        IntegerType::U64,
        op,
        a,
        b,
        CheckMode::Disabled,
        Limits::default().value_cells,
    )
    .unwrap()
}
fn assert_derived(number: &Number, pointer: &Pointer) {
    assert_eq!(
        number.provenance(),
        Some(&AddressProvenance::Derived {
            memory: pointer.memory_identity(),
            allocations: vec![pointer.allocation_key().1].into_boxed_slice()
        })
    );
}

#[test]
fn same_bit_plain_literals_remain_untainted_and_publish_as_plain_values() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let pointer = pointer(&types, &mut memory);
    let addressed = address(&pointer, 0x1000);
    let literal = plain(0x1000);
    assert_eq!(addressed.integer(), literal.integer());
    assert_ne!(addressed, literal);
    assert!(literal.provenance().is_none());
    assert_eq!(
        literal.into_value(),
        Value::Int(Integer::wrapping(IntegerType::U64, 0x1000))
    );
    assert!(
        matches!(addressed.into_value(), Value::AddressInteger(value) if value.provenance().is_some())
    );
}

#[test]
fn arithmetic_and_zero_results_preserve_address_dependency() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let pointer = pointer(&types, &mut memory);
    let original = address(&pointer, 0x1000);
    let shifted = binary(IntOp::Add, original.clone(), plain(16));
    assert_eq!(shifted.value(), 0x1010);
    assert_derived(&shifted, &pointer);
    let recovered = binary(IntOp::Subtract, shifted, plain(16));
    assert_eq!(recovered.value(), original.value());
    assert_derived(&recovered, &pointer);
    for zero in [
        binary(IntOp::Subtract, original.clone(), original.clone()),
        binary(IntOp::Multiply, original.clone(), plain(0)),
        binary(IntOp::BitAnd, original, plain(0)),
    ] {
        assert_eq!(zero.bits(), 0);
        assert_derived(&zero, &pointer);
    }
    assert_eq!(binary(IntOp::Add, plain(2), plain(3)), plain(5));
}

#[test]
fn provenance_follows_either_operand_and_same_memory_allocation_combinations() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let first = pointer(&types, &mut memory);
    let second = pointer(&types, &mut memory);
    let right_only = binary(IntOp::Add, plain(1), address(&first, 0x1000));
    assert_derived(&right_only, &first);
    let both = binary(
        IntOp::Add,
        address(&first, 0x1000),
        address(&second, 0x2000),
    );
    assert_eq!(both.value(), 0x3000);
    assert_eq!(
        both.provenance(),
        Some(&AddressProvenance::Derived {
            memory: first.memory_identity(),
            allocations: vec![first.allocation_key().1, second.allocation_key().1]
                .into_boxed_slice(),
        })
    );
}

#[test]
fn distinct_virtual_memories_cannot_be_combined() {
    let types = TypeRegistry::new();
    let mut first_memory = Memory::new(Limits::default());
    let mut second_memory = Memory::new(Limits::default());
    let first = pointer(&types, &mut first_memory);
    let second = pointer(&types, &mut second_memory);
    for op in [IntOp::Add, IntOp::Subtract, IntOp::BitAnd] {
        assert_eq!(
            scalar::binary_number(
                IntegerType::U64,
                op,
                address(&first, 0x1000),
                address(&second, 0x1000),
                CheckMode::Disabled,
                Limits::default().value_cells,
            ),
            Err(Error::ForeignPointer)
        );
    }
}

#[test]
fn casts_negation_and_complement_keep_dependency_after_bit_changes() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let pointer = pointer(&types, &mut memory);
    let original = address(&pointer, 0x1234);
    let truncated =
        scalar::cast_number(IntegerType::U8, original.clone(), CastMode::Unchecked).unwrap();
    assert_eq!(truncated.value(), 0x34);
    assert_derived(&truncated, &pointer);
    assert_eq!(
        scalar::cast_number(IntegerType::U8, original.clone(), CastMode::Checked),
        Err(Error::CheckedCast)
    );
    let negated =
        scalar::negate_number(IntegerType::U64, original.clone(), CheckMode::Disabled).unwrap();
    assert_eq!(negated.bits(), 0u64.wrapping_sub(0x1234));
    assert_derived(&negated, &pointer);
    let complemented = scalar::complement_number(IntegerType::U64, original).unwrap();
    assert_eq!(complemented.bits(), !0x1234u64);
    assert_derived(&complemented, &pointer);
    assert_eq!(
        scalar::cast_number(IntegerType::U8, plain(0x1234), CastMode::Unchecked)
            .unwrap()
            .provenance(),
        None
    );
    assert_eq!(
        scalar::negate_number(IntegerType::U64, plain(1), CheckMode::Disabled)
            .unwrap()
            .provenance(),
        None
    );
    assert_eq!(
        scalar::complement_number(IntegerType::U64, plain(0))
            .unwrap()
            .provenance(),
        None
    );
}

#[test]
fn explicit_truncation_wraps_all_target_widths_without_removing_address_origins() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let pointer = pointer(&types, &mut memory);
    for target in [
        IntegerType::S8,
        IntegerType::U8,
        IntegerType::S16,
        IntegerType::U16,
        IntegerType::S32,
        IntegerType::U32,
        IntegerType::S64,
        IntegerType::U64,
    ] {
        for source in [
            Integer::wrapping(IntegerType::U64, i128::from(u64::MAX)),
            Integer::wrapping(IntegerType::S64, -129),
        ] {
            let expected = Integer::wrapping(target, source.value());
            let ordinary =
                scalar::cast_number(target, Number::plain(source), CastMode::Truncate).unwrap();
            assert_eq!(ordinary.integer(), expected);
            assert!(ordinary.provenance().is_none());
            let addressed = Number::address(source, AddressProvenance::Pointer(pointer.clone()));
            let addressed = scalar::cast_number(target, addressed, CastMode::Truncate).unwrap();
            assert_eq!(addressed.integer(), expected);
            assert_derived(&addressed, &pointer);
            assert!(addressed.portable_integer().is_err());
        }
    }
}

#[test]
fn checked_arithmetic_and_cast_errors_do_not_remove_guarded_provenance() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let pointer = pointer(&types, &mut memory);
    let maximum = Number::address(
        Integer::wrapping(IntegerType::U8, 255),
        AddressProvenance::Pointer(pointer.clone()),
    );
    assert_eq!(
        scalar::binary_number(
            IntegerType::U8,
            IntOp::Add,
            maximum,
            Number::plain(Integer::wrapping(IntegerType::U8, 1)),
            CheckMode::Enabled,
            Limits::default().value_cells,
        ),
        Err(Error::Arithmetic(ArithmeticError::IntegerOverflow))
    );
    assert_eq!(
        scalar::negate_number(IntegerType::U64, address(&pointer, 1), CheckMode::Enabled),
        Err(Error::Arithmetic(ArithmeticError::IntegerOverflow))
    );
}

#[test]
fn origin_union_is_sorted_deduplicated_and_bounded_before_allocation() {
    let types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let first = pointer(&types, &mut memory);
    let second = pointer(&types, &mut memory);
    let same = scalar::binary_number(
        IntegerType::U64,
        IntOp::Add,
        address(&first, 1),
        address(&first, 2),
        CheckMode::Disabled,
        1,
    )
    .unwrap();
    assert_eq!(same.origin_count(), 1);
    assert_eq!(same.allocation_ids(), vec![first.allocation_key().1]);
    assert_eq!(
        scalar::binary_number(
            IntegerType::U64,
            IntOp::Add,
            address(&first, 1),
            address(&second, 2),
            CheckMode::Disabled,
            1
        ),
        Err(Error::Limit(crate::LimitKind::ValueCells))
    );
    let canonical = Number::address(
        Integer::wrapping(IntegerType::U64, 3),
        AddressProvenance::Derived {
            memory: first.memory_identity(),
            allocations: vec![
                second.allocation_key().1,
                first.allocation_key().1,
                second.allocation_key().1,
            ]
            .into_boxed_slice(),
        },
    );
    assert_eq!(canonical.origin_count(), 2);
    assert_eq!(
        canonical.allocation_ids(),
        vec![first.allocation_key().1, second.allocation_key().1]
    );
}
