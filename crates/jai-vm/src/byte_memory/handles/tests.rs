use crate::{ByteImage, ByteTarget, Endian, Error, Limits, Memory, Value};
use jai_types::{Integer, IntegerType, LayoutPolicy, ScalarLayout, ScalarType, TypeRegistry};

fn pointer_fixture(types: &mut TypeRegistry, memory: &mut Memory) -> (jai_types::TypeId, Value) {
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    let pointer_ty = types.pointer(integer).unwrap();
    let pointer = memory
        .allocate(
            types,
            integer,
            Some(Value::Int(Integer::wrapping(IntegerType::U64, 5))),
        )
        .unwrap();
    (pointer_ty, Value::Pointer(pointer))
}

#[test]
fn borrowed_reference_token_map_keeps_identity_and_missing_handles_are_atomic() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, first) = pointer_fixture(&mut types, &mut memory);
    let (_, second) = pointer_fixture(&mut types, &mut memory);
    let pair = types.fixed_array(pointer_ty, 2).unwrap();
    let target = ByteTarget::default();
    let value = Value::Array {
        ty: pair,
        elements: vec![first.clone(), second.clone()],
    };
    let mut reference = ByteImage::encode(&types, target, pair, &value, 128).unwrap();
    reference
        .retokenize_handles(|value| Ok(if value == &first { 100 } else { 200 }))
        .unwrap();
    let reversed = Value::Array {
        ty: pair,
        elements: vec![second, first],
    };
    let mut canonical = ByteImage::encode(&types, target, pair, &reversed, 128).unwrap();
    canonical.retokenize_handles_from(&reference).unwrap();
    assert_eq!(&canonical.bytes()[..8], &reference.bytes()[8..]);
    assert_eq!(&canonical.bytes()[8..], &reference.bytes()[..8]);
    assert_eq!(canonical.read(&types, target, 0, pair).unwrap(), reversed);

    let original = canonical.clone();
    reference
        .write(
            &types,
            target,
            0,
            pointer_ty,
            &Value::Pointer(crate::Pointer::null(
                types.scalar(ScalarType::Int(IntegerType::U64)),
            )),
        )
        .unwrap();
    assert!(matches!(
        canonical.retokenize_handles_from(&reference),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(canonical, original);
}

#[test]
fn borrowed_reference_token_map_rejects_inconsistent_duplicate_tokens_before_mutation() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, pointer) = pointer_fixture(&mut types, &mut memory);
    let pair = types.fixed_array(pointer_ty, 2).unwrap();
    let value = Value::Array {
        ty: pair,
        elements: vec![pointer.clone(), pointer],
    };
    let target = ByteTarget::default();
    // Standalone encoding assigns fresh tokens per occurrence. Memory normally
    // retokenizes these; a raw inconsistent reference cannot certify publication.
    let reference = ByteImage::encode(&types, target, pair, &value, 128).unwrap();
    let mut canonical = ByteImage::encode(&types, target, pair, &value, 128).unwrap();
    canonical.retokenize_handles(|_| Ok(42)).unwrap();
    let original = canonical.clone();
    assert!(matches!(
        canonical.retokenize_handles_from(&reference),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(canonical, original);
}

#[test]
fn same_address_handles_can_have_identical_bytes_across_images_and_views() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, pointer) = pointer_fixture(&mut types, &mut memory);
    let void = types.void();
    let void_pointer = types.pointer(void).unwrap();
    let view = Value::Pointer(pointer.pointer().unwrap().retype(void));
    let target = ByteTarget::default();
    let mut first = ByteImage::encode(&types, target, pointer_ty, &pointer, 64).unwrap();
    let mut second = ByteImage::encode(&types, target, void_pointer, &view, 64).unwrap();
    first.retokenize_handles(|_| Ok(0x12345678)).unwrap();
    second.retokenize_handles(|_| Ok(0x12345678)).unwrap();
    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.read(&types, target, 0, pointer_ty).unwrap(), pointer);
    assert_eq!(second.read(&types, target, 0, void_pointer).unwrap(), view);
}

#[test]
fn inconsistent_duplicate_handle_tokens_and_callback_errors_are_atomic() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, pointer) = pointer_fixture(&mut types, &mut memory);
    let pair = types.fixed_array(pointer_ty, 2).unwrap();
    let value = Value::Array {
        ty: pair,
        elements: vec![pointer.clone(), pointer],
    };
    let target = ByteTarget::default();
    let mut image = ByteImage::encode(&types, target, pair, &value, 64).unwrap();
    let original = image.bytes().to_vec();
    let mut token = 10;
    assert!(matches!(
        image.retokenize_handles(|_| {
            token += 1;
            Ok(token)
        }),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(image.bytes(), original);
    assert_eq!(image.read(&types, target, 0, pair).unwrap(), value);
    let mut calls = 0;
    let failure = Error::EffectRejected("intentional token provider failure".into());
    assert_eq!(
        image.retokenize_handles(|_| {
            calls += 1;
            if calls == 2 {
                Err(failure.clone())
            } else {
                Ok(42)
            }
        }),
        Err(failure)
    );
    assert_eq!(image.bytes(), original);
    image.retokenize_handles(|_| Ok(42)).unwrap();
    assert_eq!(&image.bytes()[..8], &image.bytes()[8..]);
}

#[test]
fn null_or_out_of_target_width_tokens_do_not_mutate_image() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, pointer) = pointer_fixture(&mut types, &mut memory);
    let policy = LayoutPolicy::new(
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
    let target = ByteTarget {
        policy,
        endian: Endian::Big,
    };
    let mut image = ByteImage::encode(&types, target, pointer_ty, &pointer, 64).unwrap();
    let original = image.bytes().to_vec();
    assert!(matches!(
        image.retokenize_handles(|_| Ok(0)),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(image.bytes(), original);
    assert!(matches!(
        image.retokenize_handles(|_| Ok(1u64 << 32)),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(image.bytes(), original);
    image.retokenize_handles(|_| Ok(0x12345678)).unwrap();
    assert_eq!(image.bytes(), &[0x12, 0x34, 0x56, 0x78]);
    assert_eq!(image.read(&types, target, 0, pointer_ty).unwrap(), pointer);
}

#[test]
fn retokenization_does_not_restore_shredded_handles_or_visit_nulls() {
    let mut types = TypeRegistry::new();
    let mut memory = Memory::new(Limits::default());
    let (pointer_ty, pointer) = pointer_fixture(&mut types, &mut memory);
    let target = ByteTarget::default();
    let mut image = ByteImage::encode(&types, target, pointer_ty, &pointer, 64).unwrap();
    image.retokenize_handles(|_| Ok(42)).unwrap();
    image.write_range(0, &[42]).unwrap();
    image
        .retokenize_handles(|_| panic!("shredded relocation must not be visited"))
        .unwrap();
    assert!(matches!(
        image.read(&types, target, 0, pointer_ty),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    image.fill_range(0, 8, 0).unwrap();
    image
        .retokenize_handles(|_| panic!("null bytes have no relocation"))
        .unwrap();
    assert!(
        image
            .read(&types, target, 0, pointer_ty)
            .unwrap()
            .pointer()
            .unwrap()
            .is_null()
    );
}

#[test]
fn procedure_tokens_can_be_stable_and_keep_distinct_procedures_separate() {
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
    let value = |id| Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(id)),
    };
    let target = ByteTarget::default();
    let mut first = ByteImage::encode(&types, target, signature, &value(7), 64).unwrap();
    let mut same = ByteImage::encode(&types, target, signature, &value(7), 64).unwrap();
    let mut other = ByteImage::encode(&types, target, signature, &value(8), 64).unwrap();
    let token = |value: &Value| match value {
        Value::Procedure {
            procedure: Some(id),
            ..
        } => Ok(id.index() as u64 + 100),
        _ => Err(Error::InvalidIr("unexpected relocation class")),
    };
    first.retokenize_handles(token).unwrap();
    same.retokenize_handles(token).unwrap();
    other.retokenize_handles(token).unwrap();
    assert_eq!(first.bytes(), same.bytes());
    assert_ne!(first.bytes(), other.bytes());
    assert_eq!(first.read(&types, target, 0, signature).unwrap(), value(7));
    assert_eq!(other.read(&types, target, 0, signature).unwrap(), value(8));
}
