use jai_ir::*;
use jai_types::{
    Integer, IntegerType, LayoutPolicy, RecordKind, ScalarType, StorageBitcast,
    StorageBitcastStrength, TypeRegistry,
};
use std::collections::HashMap;

fn check(types: &dyn jai_types::TypeView, value: &ValueExpr) -> Result<(), IrError> {
    verify_expression(types, value, &HashMap::new(), &[], &Places::default()).map(|_| ())
}
fn integer(types: &TypeRegistry) -> jai_types::TypeId {
    types.scalar(ScalarType::Int(IntegerType::U64))
}

#[test]
fn cast_result_identity_is_owned_by_exact_layout_receipt() {
    let mut types = TypeRegistry::new();
    let int = integer(&types);
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int]).unwrap();
    let cast = StorageBitcast::prove(
        &types,
        LayoutPolicy::lp64(),
        int,
        record,
        StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let value = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::Int(IntExpr::constant(
            Integer::wrapping(IntegerType::U64, 42),
        )))),
        cast,
    };
    assert_eq!(value.type_id(&types), record);
    check(&types, &value).unwrap();
    check(&types.freeze().unwrap(), &value).unwrap();
}

#[test]
fn receipt_does_not_allow_an_operand_with_another_type_or_arena() {
    let types = TypeRegistry::new();
    let int = integer(&types);
    let cast = StorageBitcast::prove(
        &types,
        LayoutPolicy::lp64(),
        int,
        int,
        StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let wrong = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::Bool(BoolExpr::Constant(true)))),
        cast,
    };
    assert!(matches!(
        check(&types, &wrong),
        Err(IrError::TypeMismatch { .. })
    ));
    let foreign = TypeRegistry::new();
    assert!(matches!(
        check(&foreign, &wrong),
        Err(IrError::Type(jai_types::TypeError::ForeignType(_)))
    ));
}

#[test]
fn target_runtime_type_still_requires_canonical_descriptor_schema() {
    let types = TypeRegistry::new();
    let int = integer(&types);
    let cast = StorageBitcast::prove(
        &types,
        LayoutPolicy::lp64(),
        int,
        types.meta_type(),
        StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let value = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Value(Box::new(ValueExpr::Zero(int))),
        cast,
    };
    assert!(check(&types, &value).is_err());
}

#[test]
fn force_cannot_be_inserted_into_a_numeric_integer_cast() {
    let types = TypeRegistry::new();
    let value = ValueExpr::Int(IntExpr::new(
        IntegerType::U64,
        IntExprKind::Cast(
            CastMode::Force(StorageBitcastStrength::EqualSize),
            Box::new(IntExpr::constant(Integer::wrapping(IntegerType::U64, 42))),
        ),
    ));
    assert!(matches!(
        check(&types, &value),
        Err(IrError::InvalidValue(_))
    ));
}
