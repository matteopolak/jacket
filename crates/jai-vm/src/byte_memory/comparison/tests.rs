use crate::{ByteImage, ByteTarget, Error, Limits, Memory, Value};
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};

#[test]
fn complete_equal_identity_ranges_ignore_opaque_token_values_and_compare_plain_gaps() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory
        .allocate(
            &types,
            word,
            Some(Value::Int(Integer::wrapping(IntegerType::U64, 7))),
        )
        .unwrap();
    let target = ByteTarget::default();
    let mut first =
        ByteImage::encode(&types, target, pointer_ty, &Value::Pointer(pointer), 128).unwrap();
    let mut second = first.clone();
    first.retokenize_handles(|_| Ok(1)).unwrap();
    second.retokenize_handles(|_| Ok(77)).unwrap();
    let prefix = ByteImage::from_bytes(target, vec![1, 2], 128).unwrap();
    let suffix = ByteImage::from_bytes(target, vec![3, 4], 128).unwrap();
    let left =
        ByteImage::concatenate(&[prefix.clone(), first, suffix.clone()], target, 128).unwrap();
    let mut right = ByteImage::concatenate(&[prefix, second, suffix], target, 128).unwrap();
    assert_ne!(left.bytes(), right.bytes());
    assert!(left.range_has_provenance(0, 12).unwrap());
    assert!(!left.range_has_provenance(0, 2).unwrap());
    assert!(
        left.range_provenance_equivalent(&right, 0, 0, 12, |a, b| Ok(a == b))
            .unwrap()
    );
    assert!(
        !left
            .range_provenance_equivalent(&right, 0, 0, 12, |_, _| Ok(false))
            .unwrap()
    );
    right.write_range(11, &[5]).unwrap();
    assert!(
        !left
            .range_provenance_equivalent(&right, 0, 0, 12, |a, b| Ok(a == b))
            .unwrap()
    );
}

#[test]
fn partial_derived_and_uninitialized_ranges_never_prove_equality() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(word).unwrap();
    let mut memory = Memory::new(Limits::default());
    let pointer = memory.allocate(&types, word, None).unwrap();
    let target = ByteTarget::default();
    let image =
        ByteImage::encode(&types, target, pointer_ty, &Value::Pointer(pointer), 128).unwrap();
    assert!(matches!(
        image.range_provenance_equivalent(&image, 1, 1, 7, |_, _| panic!("no partial callback")),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let number = image.read(&types, target, 0, word).unwrap();
    let integer_storage = ByteImage::encode(&types, target, word, &number, 128).unwrap();
    assert!(matches!(
        integer_storage.range_provenance_equivalent(&integer_storage, 0, 0, 8, |_, _| panic!(
            "no integer callback"
        )),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let partial = image.extract_range(1, 7).unwrap();
    assert!(matches!(
        partial
            .range_provenance_equivalent(&partial, 0, 0, 7, |_, _| panic!("no derived callback")),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let holes = ByteImage::uninitialized(target, 8, 128).unwrap();
    assert!(matches!(
        holes.range_has_provenance(0, 8),
        Err(Error::Uninitialized)
    ));
    assert!(matches!(
        holes.range_provenance_equivalent(&holes, 0, 0, 8, |_, _| panic!()),
        Err(Error::Uninitialized)
    ));
    assert!(
        image
            .range_provenance_equivalent(&image, 8, 8, 0, |_, _| panic!())
            .unwrap()
    );
    assert!(matches!(
        image.range_has_provenance(usize::MAX, 2),
        Err(Error::OutOfBounds { .. })
    ));
}

#[test]
fn procedure_identity_comparison_propagates_callback_errors_and_target_mismatch() {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: jai_types::CallingConvention::Jai,
            context: jai_types::ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap();
    let target = ByteTarget::default();
    let value = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(3)),
    };
    let first = ByteImage::encode(&types, target, signature, &value, 64).unwrap();
    let mut second = first.clone();
    second.retokenize_handles(|_| Ok(42)).unwrap();
    assert!(
        first
            .range_provenance_equivalent(&second, 0, 0, 8, |a, b| Ok(a == b))
            .unwrap()
    );
    assert!(matches!(
        first.range_provenance_equivalent(&second, 0, 0, 8, |_, _| Err(Error::ForeignPointer)),
        Err(Error::ForeignPointer)
    ));
    assert_eq!(first.read(&types, target, 0, signature).unwrap(), value);
    let other = ByteImage::from_bytes(
        ByteTarget {
            endian: crate::Endian::Big,
            ..target
        },
        vec![0; 8],
        64,
    )
    .unwrap();
    assert!(matches!(
        first.range_provenance_equivalent(&other, 0, 0, 8, |_, _| panic!()),
        Err(Error::InvalidIr(_))
    ));
}
