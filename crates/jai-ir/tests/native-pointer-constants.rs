use jai_ir::{
    ConstantKind, ConstantValue, Global, NativePointerConstant, PlaceRegistry, ValueExpr,
    verify_constant_procedures, verify_expression,
};
use jai_types::{CastMode, Integer, IntegerType, TypeRegistry};
use std::collections::HashMap;

#[test]
fn sealed_native_constant_requires_its_exact_enclosing_pointer_type() {
    let mut types = TypeRegistry::new();
    let opaque = types.pointer(types.void()).unwrap();
    let byte = types
        .pointer(types.scalar(jai_types::ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let native = NativePointerConstant::new(
        opaque,
        Integer::wrapping(IntegerType::S64, -1),
        CastMode::Truncate,
        &types,
    )
    .unwrap();
    let valid = ConstantValue {
        ty: opaque,
        kind: ConstantKind::NativePointer(native.clone()),
    };
    Global::new_typed(0, valid.clone(), &types).unwrap();
    assert!(
        Global::new_typed(
            0,
            ConstantValue {
                ty: byte,
                kind: ConstantKind::NativePointer(native),
            },
            &types
        )
        .is_err()
    );
    assert!(verify_constant_procedures(&types, &valid, &HashMap::new()).is_ok());
}

#[test]
fn expression_conversion_retains_the_integer_and_cast_policy() {
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let source = Integer::wrapping(IntegerType::U64, 1_i128 << 32);
    let native = NativePointerConstant::new(pointer, source, CastMode::Truncate, &types).unwrap();
    let expression = ConstantValue {
        ty: pointer,
        kind: ConstantKind::NativePointer(native),
    }
    .into_expression();
    let places = PlaceRegistry::new().freeze();
    assert!(verify_expression(&types, &expression, &HashMap::new(), &[], &places).is_ok());
    let ValueExpr::NativePointer(value) = expression else {
        panic!("native constant must retain its target normalization capsule");
    };
    assert_eq!(value.type_id(), pointer);
    assert_eq!(value.mode(), CastMode::Truncate);
    assert!(
        matches!(value.source(), jai_ir::NativePointerSource::Strong(actual) if *actual == source)
    );
}

#[test]
fn weak_pointer_literals_are_normalized_only_after_target_selection() {
    use jai_types::{CastMode, TypeRegistry};
    let mut types = TypeRegistry::new();
    let ty = types.pointer(types.void()).unwrap();
    let weak = NativePointerConstant::new_weak(ty, 1i128 << 32, CastMode::Checked, &types).unwrap();
    assert!(weak.address(32).is_err());
    assert_eq!(weak.address(64).unwrap().bits(), 1u64 << 32);
    let maximum =
        NativePointerConstant::new_weak(ty, i128::from(u64::MAX), CastMode::Checked, &types)
            .unwrap();
    assert_eq!(maximum.address(64).unwrap().bits(), u64::MAX);
    assert!(maximum.address(32).is_err());
    let negative = NativePointerConstant::new_weak(ty, -1, CastMode::Checked, &types).unwrap();
    assert_eq!(negative.address(32).unwrap().bits(), u64::from(u32::MAX));
    assert_eq!(negative.address(64).unwrap().bits(), u64::MAX);
    let truncated =
        NativePointerConstant::new_weak(ty, i128::MAX, CastMode::Truncate, &types).unwrap();
    assert_eq!(truncated.address(32).unwrap().bits(), u64::from(u32::MAX));
}
