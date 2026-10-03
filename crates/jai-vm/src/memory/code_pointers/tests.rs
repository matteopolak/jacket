use super::*;
use jai_types::{
    CallingConvention, ContextMode, LayoutPolicy, ProcedureType, ScalarLayout, TypeRegistry,
    Variadic,
};

fn catalog() -> (TypeRegistry, TypeId, Value) {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([integer]),
            results: Box::new([integer]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let value = Value::Procedure {
        signature,
        procedure: Some(ProcedureId::new(91_237)),
    };
    (types, signature, value)
}

fn admit(memory: &Memory, types: &dyn TypeView, signature: TypeId, value: &Value) -> ByteImage {
    let mut image = ByteImage::encode(types, memory.target(), signature, value, 1_000).unwrap();
    memory.retokenize_image(types, &mut image).unwrap();
    image
}

#[test]
fn canonical_code_tokens_have_no_data_allocation_and_keep_exact_signature() {
    let (mut types, signature, value) = catalog();
    let void_pointer = types.pointer(types.void()).unwrap();
    let mut memory = Memory::new(Limits::default());
    let unowned = ByteImage::encode(&types, memory.target(), signature, &value, 1_000).unwrap();
    assert!(matches!(
        memory.code_pointer(&types, &value),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert!(
        unowned
            .read(&types, memory.target(), 0, void_pointer)
            .is_err()
    );
    let image = admit(&memory, &types, signature, &value);
    let pointer = memory.code_pointer(&types, &value).unwrap();
    let again = admit(&memory, &types, signature, &value);
    assert_eq!(image, again);
    assert_eq!(memory.allocation_count(), 0);
    assert!(memory.virtual_regions.is_empty());
    assert_ne!(pointer.token, pointer.procedure.index() as u64);
    assert_eq!(
        pointer.token % u64::from(memory.target().policy.pointer().alignment),
        0
    );
    assert_eq!(
        memory
            .code_pointer_value(&types, pointer, signature)
            .unwrap(),
        value
    );
    assert_eq!(
        image.read(&types, memory.target(), 0, signature).unwrap(),
        value
    );
    let Value::Pointer(opaque) = image
        .read(&types, memory.target(), 0, void_pointer)
        .unwrap()
    else {
        panic!("canonical procedure bytes must retain opaque code origin");
    };
    assert_eq!(opaque.code_pointer(), Some(pointer));
    assert_eq!(opaque.memory_identity(), pointer.memory_identity());
    assert_eq!(opaque.data_allocation_key(), None);
    assert!(!opaque.is_null());
    assert!(memory.load(&types, &opaque).is_err());
    assert!(memory.release(&opaque).is_err());
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let data = memory
        .allocate(
            &types,
            byte,
            Some(Value::Int(Integer::wrapping(IntegerType::U8, 42))),
        )
        .unwrap();
    let data_address = memory
        .pointer_to_integer(&types, &data, IntegerType::U64, CastMode::Checked)
        .unwrap();
    assert_ne!(data_address.bits(), pointer.token);
    memory.release(&data).unwrap();
    memory.validate_code_pointer(&types, pointer).unwrap();
}

#[test]
fn receipts_reject_foreign_memory_types_wrong_signatures_and_unknown_handles() {
    let (mut types, signature, value) = catalog();
    let memory = Memory::new(Limits::default());
    admit(&memory, &types, signature, &value);
    let pointer = memory.code_pointer(&types, &value).unwrap();
    let other_memory = Memory::new(Limits::default());
    admit(&other_memory, &types, signature, &value);
    assert_eq!(
        other_memory.validate_code_pointer(&types, pointer),
        Err(Error::ForeignPointer)
    );
    let (foreign, _, _) = catalog();
    assert!(matches!(
        memory.validate_code_pointer(&foreign, pointer),
        Err(Error::Type(jai_types::TypeError::ForeignType(_)))
    ));
    let other_signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    assert_eq!(
        memory.code_pointer_value(&types, pointer, other_signature),
        Err(Error::TypeMismatch {
            expected: other_signature
        })
    );
    assert!(
        memory
            .code_pointer(
                &types,
                &Value::Procedure {
                    signature: other_signature,
                    procedure: Some(pointer.procedure)
                }
            )
            .is_err()
    );
    assert_eq!(
        memory.code_pointer(
            &types,
            &Value::Procedure {
                signature,
                procedure: None
            }
        ),
        Err(Error::NullProcedure)
    );
}

#[test]
fn rollback_invalidates_new_receipts_and_never_recycles_their_tokens() {
    let (types, signature, value) = catalog();
    let mut memory = Memory::new(Limits::default());
    let before = memory.snapshot();
    admit(&memory, &types, signature, &value);
    let rolled_back = memory.code_pointer(&types, &value).unwrap();
    memory.restore(before);
    assert_eq!(
        memory.validate_code_pointer(&types, rolled_back),
        Err(Error::DanglingPointer)
    );
    admit(&memory, &types, signature, &value);
    let current = memory.code_pointer(&types, &value).unwrap();
    assert_ne!(rolled_back.token, current.token);
    assert_eq!(
        memory.validate_code_pointer(&types, rolled_back),
        Err(Error::DanglingPointer)
    );
    let retained = memory.snapshot();
    memory.restore(retained);
    memory.validate_code_pointer(&types, current).unwrap();
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

#[test]
fn integer_roundtrip_requires_the_exact_receipt_and_selected_target_width() {
    let (types, signature, value) = catalog();
    for policy in [ilp32(), LayoutPolicy::lp64()] {
        for endian in [Endian::Little, Endian::Big] {
            let memory = Memory::with_target(
                Limits::default(),
                ByteTarget {
                    policy,
                    endian,
                },
            );
            admit(&memory, &types, signature, &value);
            let pointer = memory.code_pointer(&types, &value).unwrap();
            let integer = memory
                .code_pointer_integer(&types, pointer, IntegerType::U64)
                .unwrap();
            assert_eq!(
                memory
                    .code_pointer_from_integer(&types, pointer, integer, CastMode::Checked)
                    .unwrap(),
                pointer
            );
            assert!(matches!(
                memory.code_pointer_integer(&types, pointer, IntegerType::U16),
                Err(Error::UnsupportedPointerOperation(_))
            ));
            let changed = Integer::wrapping(IntegerType::U64, i128::from(integer.bits() + 1));
            assert!(matches!(
                memory.code_pointer_from_integer(&types, pointer, changed, CastMode::Checked),
                Err(Error::UnsupportedPointerOperation(_))
            ));
            if policy.pointer().size == 4 {
                let wide =
                    Integer::wrapping(IntegerType::U64, i128::from(integer.bits()) + (1i128 << 32));
                assert_eq!(
                    memory.code_pointer_from_integer(&types, pointer, wide, CastMode::Checked),
                    Err(Error::CheckedCast)
                );
                assert_eq!(
                    memory
                        .code_pointer_from_integer(&types, pointer, wide, CastMode::Unchecked)
                        .unwrap(),
                    pointer
                );
            }
        }
    }
}
