use jai_ir::{
    FloatExpr, IntExpr, IntExprKind, IrError, PlaceRegistry, ValueExpr, verify_expression,
};
use jai_types::{CastMode, FloatValue, Integer, IntegerType, TypeRegistry};
use std::collections::HashMap;

#[test]
fn float_to_integer_proof_admits_only_the_explicit_checked_policy() {
    let types = TypeRegistry::new();
    let signatures = HashMap::new();
    let places = PlaceRegistry::new().freeze();
    for mode in [CastMode::Checked, CastMode::Unchecked, CastMode::Truncate] {
        let expression = ValueExpr::Int(IntExpr::new(
            IntegerType::S64,
            IntExprKind::FromFloat(
                mode,
                Box::new(FloatExpr::constant(FloatValue::F64(42_f64.to_bits()))),
            ),
        ));
        let result = verify_expression(&types, &expression, &signatures, &[], &places);
        if mode == CastMode::Checked {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(IrError::InvalidValue(_))));
        }
    }
}

#[test]
fn truncating_integer_proof_preserves_the_source_policy_and_static_backing() {
    let types = TypeRegistry::new();
    let signatures = HashMap::new();
    let places = PlaceRegistry::new().freeze();
    let expression = ValueExpr::Int(IntExpr::new(
        IntegerType::U8,
        IntExprKind::Cast(
            CastMode::Truncate,
            Box::new(IntExpr::constant(
                Integer::checked(IntegerType::S64, 256).unwrap(),
            )),
        ),
    ));
    assert!(verify_expression(&types, &expression, &signatures, &[], &places).is_ok());
    assert!(jai_ir::is_static_value(&expression));
    let ValueExpr::Int(integer) = &expression else {
        unreachable!()
    };
    assert!(matches!(
        integer.kind(),
        IntExprKind::Cast(CastMode::Truncate, _)
    ));
}
