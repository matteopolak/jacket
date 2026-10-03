use jai_ir::*;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};
use std::collections::HashMap;

fn signature(types: &mut TypeRegistry, context: ContextMode) -> jai_types::TypeId {
    types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            convention: CallingConvention::Jai,
            context,
            variadic: Variadic::None,
        })
        .unwrap()
}

#[test]
fn null_and_bound_procedures_support_address_truth_and_equality() {
    let mut types = TypeRegistry::new();
    let ty = signature(&mut types, ContextMode::Implicit);
    let id = ProcedureId::new(31);
    let signatures = HashMap::from([(id, ty)]);
    let places = Places::default();
    let null = ValueExpr::Zero(ty);
    let bound = ValueExpr::ProcedureValue { procedure: id, ty };
    for value in [
        ValueExpr::Bool(BoolExpr::FromPointer(Box::new(null.clone()))),
        ValueExpr::Bool(BoolExpr::FromPointer(Box::new(bound.clone()))),
        ValueExpr::Bool(BoolExpr::ComparePointers(
            Equality::Equal,
            Box::new(bound),
            Box::new(null),
        )),
    ] {
        verify_expression(&types, &value, &signatures, &[], &places).unwrap();
    }
}

#[test]
fn procedure_address_comparison_preserves_context_signature_identity() {
    let mut types = TypeRegistry::new();
    let implicit = signature(&mut types, ContextMode::Implicit);
    let contextless = signature(&mut types, ContextMode::None);
    let expression = ValueExpr::Bool(BoolExpr::ComparePointers(
        Equality::Equal,
        Box::new(ValueExpr::Zero(implicit)),
        Box::new(ValueExpr::Zero(contextless)),
    ));
    assert!(matches!(
        verify_expression(
            &types,
            &expression,
            &HashMap::new(),
            &[],
            &Places::default()
        ),
        Err(IrError::TypeMismatch { .. })
    ));
}

#[test]
fn procedure_address_cannot_enter_data_pointer_arithmetic() {
    let mut types = TypeRegistry::new();
    let ty = signature(&mut types, ContextMode::None);
    let expression = ValueExpr::PointerOffset {
        pointer: Box::new(ValueExpr::Zero(ty)),
        offset: IntExpr::constant(
            jai_types::Integer::checked(jai_types::IntegerType::S64, 1).unwrap(),
        ),
        subtract: false,
        ty,
    };
    assert!(matches!(
        verify_expression(&types, &expression, &HashMap::new(), &[], &Places::default()),
        Err(IrError::InvalidValue(actual)) if actual == ty
    ));
}
