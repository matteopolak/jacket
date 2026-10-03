use super::*;
use jai_types::{LayoutPolicy, ScalarLayout, TypeRegistry};
fn derived(address: &Number, value: i128, ty: IntegerType) -> Number {
    Number::address(
        Integer::wrapping(ty, value),
        AddressProvenance::Derived {
            memory: address.memory_identity().unwrap(),
            allocations: address.allocation_ids().into(),
        },
    )
}
fn target32() -> ByteTarget {
    ByteTarget {
        policy: LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 8),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
        endian: Endian::Little,
    }
}
#[test]
fn virtual_addresses_preserve_alignment_stride_and_byte_image_identity() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_type = types.pointer(byte).unwrap();
    let array = types.fixed_array(word, 2).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![
                    Value::Int(Integer::wrapping(IntegerType::U64, 1)),
                    Value::Int(Integer::wrapping(IntegerType::U64, 2)),
                ],
            }),
        )
        .unwrap();
    let first = memory.index(&types, &root, 0).unwrap();
    let second = memory.index(&types, &root, 1).unwrap();
    let a = memory
        .pointer_to_integer(&types, &first, IntegerType::U64, CastMode::Checked)
        .unwrap();
    let b = memory
        .pointer_to_integer(&types, &second, IntegerType::S64, CastMode::Checked)
        .unwrap();
    assert_eq!(a.bits() % 8, 0);
    assert_eq!(b.bits() - a.bits(), 8);
    let bytes = memory
        .cast_pointer(&types, &first, byte, CastMode::Checked)
        .unwrap();
    let byte1 = memory.offset(&types, &bytes, 1).unwrap();
    let address = memory
        .pointer_to_integer(&types, &byte1, IntegerType::U64, CastMode::Checked)
        .unwrap();
    assert_eq!(address.bits() - a.bits(), 1);
    let recovered = memory
        .integer_to_pointer(&types, address.clone(), byte, CastMode::Checked)
        .unwrap();
    assert!(memory.same_address(&types, &byte1, &recovered).unwrap());
    let slot = memory
        .allocate(&types, pointer_type, Some(Value::Pointer(byte1)))
        .unwrap();
    let word_slot = memory
        .cast_pointer(&types, &slot, word, CastMode::Checked)
        .unwrap();
    assert_eq!(
        memory.load(&types, &word_slot).unwrap(),
        address.clone().into_value()
    );
}
#[test]
fn inverse_virtual_casts_preserve_exposed_bounds_and_lifetime() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let array = types.fixed_array(byte, 3).unwrap();
    let mut memory = Memory::new(Limits::default());
    let root = memory
        .allocate(
            &types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![Value::Int(Integer::wrapping(IntegerType::U8, 4)); 3],
            }),
        )
        .unwrap();
    let pointer = memory.sequence_data(&types, &root).unwrap();
    let address = memory
        .pointer_to_integer(&types, &pointer, IntegerType::U64, CastMode::Checked)
        .unwrap();
    let end_pointer = memory.offset(&types, &pointer, 3).unwrap();
    let end_address = memory
        .pointer_to_integer(&types, &end_pointer, IntegerType::U64, CastMode::Checked)
        .unwrap();
    let end = memory
        .integer_to_pointer(&types, end_address, byte, CastMode::Checked)
        .unwrap();
    assert_eq!(memory.distance(&types, &end, &pointer).unwrap(), 3);
    assert!(matches!(
        memory.load(&types, &end),
        Err(Error::OutOfBounds { .. })
    ));
    assert!(matches!(
        memory.integer_to_pointer(
            &types,
            derived(&address, i128::from(address.bits() + 4), IntegerType::U64),
            byte,
            CastMode::Checked
        ),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    memory.release(&root).unwrap();
    assert_eq!(
        memory.integer_to_pointer(&types, address.clone(), byte, CastMode::Checked),
        Err(Error::DanglingPointer)
    );
}
#[test]
fn target_width_casts_keep_bit_patterns_and_reject_unknown_addresses() {
    let types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let mut memory = Memory::with_target(Limits::default(), target32());
    let root = memory
        .allocate(
            &types,
            byte,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 7))),
        )
        .unwrap();
    let address = memory
        .pointer_to_integer(&types, &root, IntegerType::S32, CastMode::Checked)
        .unwrap();
    assert!(
        memory
            .same_address(
                &types,
                &root,
                &memory
                    .integer_to_pointer(&types, address.clone(), byte, CastMode::Checked)
                    .unwrap()
            )
            .unwrap()
    );
    assert_eq!(
        memory.integer_to_pointer(
            &types,
            derived(
                &address,
                (1i128 << 32) + i128::from(address.bits()),
                IntegerType::U64
            ),
            byte,
            CastMode::Checked
        ),
        Err(Error::CheckedCast)
    );
    assert!(matches!(
        memory.integer_to_pointer(
            &types,
            derived(
                &address,
                (1i128 << 32) + i128::from(address.bits()),
                IntegerType::U64
            ),
            byte,
            CastMode::Unchecked
        ),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert!(matches!(
        memory.pointer_to_integer(&types, &root, IntegerType::U16, CastMode::Unchecked),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(
        memory
            .pointer_to_integer(
                &types,
                &Pointer::null(byte),
                IntegerType::U8,
                CastMode::Checked
            )
            .unwrap()
            .bits(),
        0
    );
    let numeric = memory
        .integer_to_pointer(
            &types,
            Number::plain(Integer::wrapping(IntegerType::U32, 0xffff)),
            byte,
            CastMode::Checked,
        )
        .unwrap();
    assert_eq!(numeric.opaque_address_bits(), Some((0xffff, 32)));
    assert_eq!(
        memory.load(&types, &numeric),
        Err(Error::UnsupportedPointerOperation(
            "numeric address has no allocation provenance"
        ))
    );
    assert!(
        memory
            .integer_to_pointer(
                &types,
                Number::plain(Integer::wrapping(IntegerType::U64, 1i128 << 32)),
                byte,
                CastMode::Unchecked
            )
            .unwrap()
            .is_null()
    );
}
