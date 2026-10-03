use super::*;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

fn fixture() -> (TypeRegistry, TypeId, TypeId, Value) {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let pointer = types.pointer(types.void()).unwrap();
    let value = Value::Procedure {
        signature,
        procedure: Some(jai_ir::ProcedureId::new(97)),
    };
    (types, signature, pointer, value)
}
fn admitted(memory: &Memory, types: &dyn TypeView, ty: TypeId, value: &Value) -> ByteImage {
    let mut image = ByteImage::encode(types, memory.target(), ty, value, 1_000).unwrap();
    memory.retokenize_image(types, &mut image).unwrap();
    memory.certify_code_image(types, &mut image).unwrap();
    image
}

#[test]
fn only_canonical_receipts_enable_code_pointer_and_integer_views() {
    let (types, signature, pointer_ty, value) = fixture();
    let memory = Memory::new(Limits::default());
    let target = memory.target();
    let mut unowned = ByteImage::encode(&types, target, signature, &value, 1_000).unwrap();
    let original = unowned.clone();
    assert!(unowned.read(&types, target, 0, pointer_ty).is_err());
    let integer = types.scalar(ScalarType::Int(IntegerType::U64));
    assert!(unowned.read(&types, target, 0, integer).is_err());
    assert!(memory.certify_code_image(&types, &mut unowned).is_err());
    assert_eq!(unowned, original);
    let image = admitted(&memory, &types, signature, &value);
    let pointer = image
        .read(&types, target, 0, pointer_ty)
        .unwrap()
        .pointer()
        .unwrap()
        .clone();
    let code = pointer.code_pointer().unwrap();
    assert_eq!(code.signature(), signature);
    assert!(pointer.data_allocation_key().is_none());
    let number = image
        .read(&types, target, 0, integer)
        .unwrap()
        .number()
        .unwrap();
    assert_eq!(number.bits(), code.token());
    assert!(
        matches!(number.provenance(), Some(crate::AddressProvenance::Pointer(p)) if p.code_pointer() == Some(code))
    );
    let mut origins = 0;
    image
        .visit_address_origins(|_, _| {
            origins += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(origins, 0);
    for ty in [
        types.scalar(ScalarType::Bool),
        types.float(jai_types::FloatType::F64),
    ] {
        assert!(matches!(
            image.read(&types, target, 0, ty),
            Err(Error::UnsupportedPointerOperation(_))
        ));
    }
}

#[test]
fn reboxed_code_slot_recovers_only_original_signature_and_canonical_token() {
    let (mut types, signature, pointer_ty, value) = fixture();
    let memory = Memory::new(Limits::default());
    let image = admitted(&memory, &types, signature, &value);
    let boxed = image.read(&types, memory.target(), 0, pointer_ty).unwrap();
    let mut slot = ByteImage::encode(&types, memory.target(), pointer_ty, &boxed, 1_000).unwrap();
    assert_eq!(
        slot.read(&types, memory.target(), 0, signature).unwrap(),
        value
    );
    let other = types
        .procedure(ProcedureType {
            parameters: Box::new([types.scalar(ScalarType::Int(IntegerType::U8))]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    assert_eq!(
        slot.read(&types, memory.target(), 0, other),
        Err(Error::TypeMismatch { expected: other })
    );
    slot.retokenize_handles(|_| Ok(13)).unwrap();
    assert!(slot.read(&types, memory.target(), 0, signature).is_err());
    assert!(slot.read(&types, memory.target(), 0, pointer_ty).is_err());
}

#[test]
fn complete_copies_keep_code_receipts_fragments_cannot_reconstruct_them() {
    let (types, signature, pointer_ty, value) = fixture();
    let memory = Memory::new(Limits::default());
    let image = admitted(&memory, &types, signature, &value);
    let mut copied = ByteImage::from_bytes(memory.target(), vec![0; 8], 1_000).unwrap();
    copied.copy_range_from(&image, 0, 0, 8).unwrap();
    assert_eq!(
        copied.read(&types, memory.target(), 0, signature).unwrap(),
        value
    );
    assert!(copied.read(&types, memory.target(), 0, pointer_ty).is_ok());
    copied.copy_range_from(&image, 0, 0, 4).unwrap();
    copied.copy_range_from(&image, 4, 4, 4).unwrap();
    assert_eq!(copied.bytes(), image.bytes());
    assert!(copied.read(&types, memory.target(), 0, signature).is_err());
    assert!(copied.read(&types, memory.target(), 0, pointer_ty).is_err());
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let number = copied
        .read(&types, memory.target(), 0, byte)
        .unwrap()
        .number()
        .unwrap();
    assert!(
        matches!(number.provenance(), Some(crate::AddressProvenance::Derived { allocations, .. }) if allocations.is_empty())
    );
}

#[test]
fn failed_certification_is_atomic_and_canonical_proof_retains_receipts() {
    let (mut types, signature, _, value) = fixture();
    let pair = types.fixed_array(signature, 2).unwrap();
    let values = Value::Array {
        ty: pair,
        elements: vec![value.clone(), value.clone()],
    };
    let memory = Memory::new(Limits::default());
    let reference = admitted(&memory, &types, pair, &values);
    let mut canonical = ByteImage::encode(&types, memory.target(), pair, &values, 1_000).unwrap();
    canonical.retokenize_handles_from(&reference).unwrap();
    assert_eq!(canonical, reference);
    let receipt = memory.code_pointer(&types, &value).unwrap();
    let before = canonical.clone();
    let mut calls = 0;
    assert!(
        canonical
            .certify_code_handles(|_| {
                calls += 1;
                if calls == 2 {
                    Err(Error::CheckedCast)
                } else {
                    Ok(Pointer::from_code(receipt, signature))
                }
            })
            .is_err()
    );
    assert_eq!(canonical, before);
}

#[test]
fn empty_code_origin_span_splits_and_copies_charge_records_atomically() {
    let (types, signature, _, procedure) = fixture();
    let mut memory = Memory::new(Limits::default());
    let target = memory.target();
    let code_image = admitted(&memory, &types, signature, &procedure);
    let fragment = code_image.extract_range(0, 4).unwrap();
    assert_eq!(fragment.metadata_cells(), 1);
    assert_eq!(fragment.range_metadata_work(0, 1).unwrap(), 1);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let derived = fragment
        .read(&types, target, 0, byte)
        .unwrap()
        .number()
        .unwrap();
    assert_eq!(derived.origin_count(), 0);
    assert_eq!(derived.metadata_cells(), 0);
    let allocations = (0..15)
        .map(|_| {
            memory
                .allocate(&types, byte, None)
                .unwrap()
                .allocation_key()
                .1
        })
        .collect();
    let origins = crate::Number::address(
        Integer::wrapping(IntegerType::U64, 42),
        crate::AddressProvenance::Derived {
            memory: memory.identity,
            allocations,
        },
    );
    let large = ByteImage::encode(
        &types,
        target,
        types.scalar(ScalarType::Int(IntegerType::U64)),
        &origins.into_value(),
        64,
    )
    .unwrap();
    let plain = ByteImage::from_bytes(target, vec![0; 4], 64).unwrap();
    let mut destination =
        ByteImage::concatenate(&[large, plain, fragment.clone()], target, 16).unwrap();
    assert_eq!(destination.metadata_cells(), 16);
    let before = destination.clone();
    for result in [
        destination.write_range(13, &[0]),
        destination.fill_range(13, 1, 0),
        destination.copy_range_from(&fragment, 0, 13, 1),
        destination.copy_range_within(12, 13, 1),
        destination.fill_range_number(13, 1, derived.clone()),
        destination.write(&types, target, 13, byte, &derived.into_value()),
    ] {
        assert_eq!(result, Err(Error::Limit(LimitKind::ValueCells)));
    }
    assert_eq!(destination, before);
    destination.copy_range_from(&fragment, 0, 12, 4).unwrap();
    assert_eq!(destination.metadata_cells(), 16);
    assert!(
        matches!(destination.read(&types, target, 12, byte).unwrap().number().unwrap().provenance(), Some(crate::AddressProvenance::Derived { allocations, .. }) if allocations.is_empty())
    );
}
