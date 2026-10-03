use jai_ir::*;
use jai_types::{
    AnyField, AnySchema, Integer, IntegerType, RecordKind, ScalarType, TypeRegistry, TypeView,
};
use std::collections::HashMap;

fn fixture() -> (TypeRegistry, AnySchema) {
    let mut types = TypeRegistry::new();
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, []).unwrap();
    let any = types.reserve_any();
    types.define_any(any, header).unwrap();
    let schema = AnySchema::validate(&types, any, header).unwrap();
    (types, schema)
}

fn check(types: &dyn TypeView, value: &ValueExpr) -> Result<(), IrError> {
    verify_expression(types, value, &HashMap::new(), &[], &Places::default()).map(|_| ())
}

fn zero_descriptor(schema: AnySchema) -> Vec<ValueExpr> {
    [AnyField::Type, AnyField::ValuePointer]
        .map(|field| ValueExpr::Zero(schema.field(field).ty))
        .into()
}

#[test]
fn universal_storage_uses_shared_record_nodes_and_exact_field_owners() {
    let (mut types, schema) = fixture();
    let value = ValueExpr::Record {
        ty: schema.ty(),
        fields: zero_descriptor(schema),
    };
    check(&types, &value).unwrap();
    let field = schema.field(AnyField::ValuePointer);
    check(
        &types,
        &ValueExpr::Field {
            base: Box::new(value),
            field: field.id,
            ty: field.ty,
        },
    )
    .unwrap();
    let imitation = types.reserve_record(RecordKind::Struct);
    types
        .define_record(imitation, [schema.field(AnyField::Type).ty, field.ty])
        .unwrap();
    assert!(
        check(
            &types,
            &ValueExpr::Field {
                base: Box::new(ValueExpr::Record {
                    ty: imitation,
                    fields: zero_descriptor(schema),
                }),
                field: field.id,
                ty: field.ty,
            }
        )
        .is_err()
    );
    assert!(types.record_definition(schema.ty()).is_err());
}

#[test]
fn universal_record_build_requires_all_fields_and_exact_pointer_types() {
    let (mut types, schema) = fixture();
    let descriptor = schema.field(AnyField::Type);
    let payload = schema.field(AnyField::ValuePointer);
    check(
        &types,
        &ValueExpr::RecordBuild {
            ty: schema.ty(),
            initializers: vec![
                (payload.id, ValueExpr::Zero(payload.ty)),
                (descriptor.id, ValueExpr::Zero(descriptor.ty)),
            ],
        },
    )
    .unwrap();
    assert!(matches!(
        check(
            &types,
            &ValueExpr::RecordBuild {
                ty: schema.ty(),
                initializers: vec![(payload.id, ValueExpr::Zero(payload.ty))],
            }
        ),
        Err(IrError::Arity { .. })
    ));
    let other_header = types.reserve_record(RecordKind::Struct);
    types.define_record(other_header, []).unwrap();
    let other_pointer = types.pointer(other_header).unwrap();
    assert!(matches!(
        check(
            &types,
            &ValueExpr::Record {
                ty: schema.ty(),
                fields: vec![ValueExpr::Zero(other_pointer), ValueExpr::Zero(payload.ty)],
            }
        ),
        Err(IrError::TypeMismatch { .. })
    ));
}

#[test]
fn universal_constants_publish_with_nominal_identity() {
    let (types, schema) = fixture();
    let constant = ConstantValue {
        ty: schema.ty(),
        kind: ConstantKind::Record(
            [AnyField::Type, AnyField::ValuePointer]
                .map(|field| ConstantValue {
                    ty: schema.field(field).ty,
                    kind: ConstantKind::Zero,
                })
                .into(),
        ),
    };
    let global = Global::new_typed(0, constant, &types).unwrap();
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .finish_library()
        .unwrap();
    assert_eq!(library.globals()[0].ty(), schema.ty());
}

#[test]
fn universal_storage_cannot_replace_nominal_context_or_union() {
    let (mut types, schema) = fixture();
    assert!(Place::context(schema.ty(), &types).is_err());
    let payload = schema.field(AnyField::ValuePointer);
    assert!(
        check(
            &types,
            &ValueExpr::Union {
                ty: schema.ty(),
                field: payload.id,
                value: Box::new(ValueExpr::Zero(payload.ty)),
            }
        )
        .is_err()
    );
    let pointer = types.pointer(schema.ty()).unwrap();
    assert!(
        ProgramBuilder::new(types.freeze().unwrap())
            .context(ContextDefinition {
                record_type: schema.ty(),
                pointer_type: pointer,
                default: ConstantValue {
                    ty: schema.ty(),
                    kind: ConstantKind::Zero,
                },
            })
            .finish_library()
            .is_err()
    );
}

#[test]
fn materialized_address_keeps_exact_payload_and_pointer_identity() {
    let mut types = TypeRegistry::new();
    let scalar = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(scalar).unwrap();
    let value = || {
        ValueExpr::Int(IntExpr::constant(
            Integer::checked(IntegerType::S64, 42).unwrap(),
        ))
    };
    check(
        &types,
        &ValueExpr::AddressOfValue {
            value: Box::new(value()),
            ty: pointer,
        },
    )
    .unwrap();
    let bool_pointer = types.pointer(types.scalar(ScalarType::Bool)).unwrap();
    assert!(matches!(
        check(
            &types,
            &ValueExpr::AddressOfValue {
                value: Box::new(value()),
                ty: bool_pointer,
            }
        ),
        Err(IrError::TypeMismatch { .. })
    ));
    assert!(matches!(
        check(
            &types,
            &ValueExpr::AddressOfValue {
                value: Box::new(value()),
                ty: scalar,
            }
        ),
        Err(IrError::InvalidValue(actual)) if actual == scalar
    ));
    let compile_only = types.code_type();
    let code_pointer = types.pointer(compile_only).unwrap();
    assert!(matches!(
        check(
            &types,
            &ValueExpr::AddressOfValue {
                value: Box::new(ValueExpr::Zero(compile_only)),
                ty: code_pointer,
            }
        ),
        Err(IrError::InvalidValue(actual)) if actual == compile_only
    ));
}
