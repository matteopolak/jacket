use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, DistinctKind, ProcedureType, RecordKind, ScalarType, TypeId,
    TypeRegistry, Variadic,
};
use std::collections::HashMap;

const TARGET: usize = 93;

fn signature(
    types: &mut TypeRegistry,
    convention: CallingConvention,
    context: ContextMode,
) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention,
            context,
            variadic: Variadic::None,
        })
        .unwrap()
}

fn leaf(ty: TypeId, id: usize) -> ConstantValue {
    ConstantValue {
        ty,
        kind: ConstantKind::Procedure(ProcedureId::new(id)),
    }
}

fn prototype(signature: TypeId) -> ProcedurePrototype {
    ProcedurePrototype {
        id: ProcedureId::new(TARGET),
        signature,
        origin: PrototypeOrigin::Compiler,
    }
}

fn aggregate_constants(
    types: &mut TypeRegistry,
    procedure: TypeId,
    id: usize,
) -> Vec<ConstantValue> {
    let array = types.fixed_array(procedure, 2).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [array]).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [procedure]).unwrap();
    let field = types.field(union, 0).unwrap().id;
    let distinct = types.reserve_distinct(DistinctKind::Distinct);
    types.define_distinct(distinct, procedure).unwrap();
    vec![
        leaf(procedure, id),
        ConstantValue {
            ty: array,
            kind: ConstantKind::Array(vec![leaf(procedure, id), leaf(procedure, id)]),
        },
        ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: array,
                kind: ConstantKind::Array(vec![leaf(procedure, id), leaf(procedure, id)]),
            }]),
        },
        ConstantValue {
            ty: union,
            kind: ConstantKind::Union {
                field,
                value: Box::new(leaf(procedure, id)),
            },
        },
        ConstantValue {
            ty: distinct,
            kind: ConstantKind::Distinct(Box::new(leaf(procedure, id))),
        },
    ]
}

#[test]
fn scalar_and_nested_global_procedure_constants_publish_against_prototypes() {
    let mut types = TypeRegistry::new();
    let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    let constants = aggregate_constants(&mut types, target, TARGET);
    let ledger = HashMap::from([(ProcedureId::new(TARGET), target)]);
    let globals = constants
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            verify_constant_procedures(&types, &value, &ledger).unwrap();
            Global::new_typed(index, value, &types).unwrap()
        })
        .collect();
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .globals(globals)
        .prototypes(vec![prototype(target)])
        .finish_library()
        .unwrap();
    assert_eq!(library.globals().len(), 5);
    assert_eq!(library.signature(ProcedureId::new(TARGET)), Some(target));
}

#[test]
fn source_procedure_definitions_also_prove_global_constant_identity() {
    let mut types = TypeRegistry::new();
    let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    let global = Global::new_typed(0, leaf(target, TARGET), &types).unwrap();
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .procedures(vec![Procedure {
            id: ProcedureId::new(TARGET),
            signature: target,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
        .unwrap();
    assert!(
        library
            .checked_procedure(ProcedureId::new(TARGET))
            .is_some()
    );
}

#[test]
fn type_shape_constructor_defers_unknown_id_rejection_until_full_publication() {
    for shape in 0..5 {
        let mut types = TypeRegistry::new();
        let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
        let mut values = aggregate_constants(&mut types, target, TARGET + 1);
        let value = values.remove(shape);
        let global = Global::new_typed(0, value, &types).unwrap();
        assert!(matches!(
            ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .prototypes(vec![prototype(target)])
                .finish_library(),
            Err(IrError::UnknownIdentity { kind: "constant procedure", index }) if index == TARGET + 1
        ));
    }
}

#[test]
fn every_global_wrapper_requires_the_exact_signature_of_its_procedure_id() {
    for shape in 0..5 {
        let mut types = TypeRegistry::new();
        let declared = signature(&mut types, CallingConvention::Jai, ContextMode::None);
        let actual = signature(&mut types, CallingConvention::C, ContextMode::None);
        let mut values = aggregate_constants(&mut types, declared, TARGET);
        let global = Global::new_typed(0, values.remove(shape), &types).unwrap();
        assert!(matches!(
            ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .prototypes(vec![prototype(actual)])
                .finish_library(),
            Err(IrError::TypeMismatch { expected, actual: found }) if expected == actual && found == declared
        ));
    }
}

#[test]
fn borrowed_leaf_proof_preserves_calling_convention_context_parameters_and_results() {
    let mut types = TypeRegistry::new();
    let declared = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    let boolean = types.scalar(ScalarType::Bool);
    let mut wrong = vec![
        signature(&mut types, CallingConvention::C, ContextMode::None),
        signature(&mut types, CallingConvention::Jai, ContextMode::Implicit),
    ];
    for (parameters, results) in [(vec![boolean], vec![]), (vec![], vec![boolean])] {
        wrong.push(
            types
                .procedure(ProcedureType {
                    parameters: parameters.into(),
                    results: results.into(),
                    return_abi: jai_types::ForeignReturnAbi::Natural,
                    convention: CallingConvention::Jai,
                    context: ContextMode::None,
                    variadic: Variadic::None,
                })
                .unwrap(),
        );
    }
    for actual in wrong {
        assert!(matches!(
            verify_constant_procedures(&types, &leaf(declared, TARGET), &HashMap::from([(ProcedureId::new(TARGET), actual)])),
            Err(IrError::TypeMismatch { expected, actual: found }) if expected == actual && found == declared
        ));
    }
}

#[test]
fn borrowed_leaf_proof_preserves_variadic_signature_identity() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Bool);
    let pack = types.slice(element).unwrap();
    let make = |variadic| ProcedureType {
        parameters: vec![pack].into(),
        results: Box::new([]),
        return_abi: jai_types::ForeignReturnAbi::Natural,
        convention: CallingConvention::Jai,
        context: ContextMode::None,
        variadic,
    };
    let declared = types.procedure(make(Variadic::None)).unwrap();
    let actual = types
        .procedure(make(Variadic::Jai {
            parameter: 0,
            element,
        }))
        .unwrap();
    assert!(matches!(
        verify_constant_procedures(
            &types,
            &leaf(declared, TARGET),
            &HashMap::from([(ProcedureId::new(TARGET), actual)]),
        ),
        Err(IrError::TypeMismatch { expected, actual: found }) if expected == actual && found == declared
    ));
}

#[test]
fn typed_null_procedure_constants_do_not_require_a_signature_ledger_entry() {
    let mut types = TypeRegistry::new();
    let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    let value = ConstantValue {
        ty: target,
        kind: ConstantKind::Zero,
    };
    verify_constant_procedures(&types, &value, &HashMap::new()).unwrap();
    let global = Global::new_typed(0, value, &types).unwrap();
    ProgramBuilder::new(types.freeze().unwrap())
        .globals(vec![global])
        .finish_library()
        .unwrap();
}

#[test]
fn constant_constructor_rejects_procedure_leaves_with_nonprocedure_types() {
    let types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    assert!(matches!(
        Global::new_typed(0, leaf(boolean, TARGET), &types),
        Err(IrError::InvalidConstant(actual)) if actual == boolean
    ));
}

#[test]
fn context_default_procedure_constants_require_a_matching_published_identity() {
    for case in 0..3 {
        let mut types = TypeRegistry::new();
        let declared = signature(&mut types, CallingConvention::Jai, ContextMode::None);
        let other = signature(&mut types, CallingConvention::C, ContextMode::None);
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [declared]).unwrap();
        let pointer = types.pointer(record).unwrap();
        let context = ContextDefinition {
            record_type: record,
            pointer_type: pointer,
            default: ConstantValue {
                ty: record,
                kind: ConstantKind::Record(vec![leaf(declared, TARGET)]),
            },
        };
        let prototypes = match case {
            0 => vec![prototype(declared)],
            1 => vec![prototype(other)],
            2 => vec![],
            _ => unreachable!(),
        };
        let result = ProgramBuilder::new(types.freeze().unwrap())
            .context(context)
            .prototypes(prototypes)
            .finish_library();
        match case {
            0 => {
                result.unwrap();
            }
            1 => assert!(
                matches!(result, Err(IrError::TypeMismatch { expected, actual }) if expected == other && actual == declared)
            ),
            2 => assert!(matches!(
                result,
                Err(IrError::UnknownIdentity {
                    kind: "constant procedure",
                    index: TARGET
                })
            )),
            _ => unreachable!(),
        }
    }
}

fn static_expression(
    types: &mut TypeRegistry,
    procedure: TypeId,
    unrelated_root: bool,
) -> ValueExpr {
    let array = types.fixed_array(procedure, 2).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [array]).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(record, types).unwrap();
    builder
        .define(
            object,
            StaticValue {
                ty: record,
                kind: StaticValueKind::Record(vec![StaticValue {
                    ty: array,
                    kind: StaticValueKind::Array(vec![
                        StaticValue::constant(leaf(procedure, TARGET)),
                        StaticValue::constant(leaf(procedure, TARGET)),
                    ]),
                }]),
            },
        )
        .unwrap();
    let (addressed, addressed_ty) = if unrelated_root {
        let boolean = types.scalar(ScalarType::Bool);
        let root = builder.reserve(boolean, types).unwrap();
        builder
            .define(
                root,
                StaticValue::constant(ConstantValue {
                    ty: boolean,
                    kind: ConstantKind::Bool(true),
                }),
            )
            .unwrap();
        (root, boolean)
    } else {
        (object, record)
    };
    let pointer = types.pointer(addressed_ty).unwrap();
    let data = builder.finish(types, StaticDataLimits::default()).unwrap();
    ValueExpr::StaticAddress {
        data: data.into(),
        address: StaticAddress::new(addressed),
        ty: pointer,
    }
}

#[test]
fn static_address_closure_proves_nested_and_unaddressed_procedure_constants() {
    for unrelated_root in [false, true] {
        for case in 0..3 {
            let mut types = TypeRegistry::new();
            let declared = signature(&mut types, CallingConvention::Jai, ContextMode::None);
            let other = signature(&mut types, CallingConvention::C, ContextMode::None);
            let expression = static_expression(&mut types, declared, unrelated_root);
            let prototypes = match case {
                0 => vec![prototype(declared)],
                1 => vec![prototype(other)],
                2 => vec![],
                _ => unreachable!(),
            };
            let result = ProgramBuilder::new(types.freeze().unwrap())
                .prototypes(prototypes)
                .procedures(vec![Procedure {
                    id: ProcedureId::new(17),
                    signature: declared,
                    parameters: vec![],
                    locals: vec![],
                    cleanups: vec![],
                    body: Block {
                        statements: vec![Statement::DiscardValue(expression)],
                        flow: Flow::FallsThrough,
                    },
                }])
                .finish_library();
            match case {
                0 => {
                    result.unwrap();
                }
                1 => assert!(
                    matches!(result, Err(IrError::TypeMismatch { expected, actual }) if expected == other && actual == declared)
                ),
                2 => assert!(matches!(
                    result,
                    Err(IrError::UnknownIdentity {
                        kind: "constant procedure",
                        index: TARGET
                    })
                )),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn borrowed_leaf_proof_ignores_unused_incomplete_nominal_types() {
    let mut types = TypeRegistry::new();
    let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    types.reserve_record(RecordKind::Struct);
    types.reserve_distinct(DistinctKind::Distinct);
    verify_constant_procedures(
        &types,
        &leaf(target, TARGET),
        &HashMap::from([(ProcedureId::new(TARGET), target)]),
    )
    .unwrap();
    let boolean = types.scalar(ScalarType::Bool);
    verify_constant_procedures(
        &types,
        &ConstantValue {
            ty: boolean,
            kind: ConstantKind::Bool(false),
        },
        &HashMap::new(),
    )
    .unwrap();
}

#[test]
fn borrowed_leaf_proof_rejects_foreign_and_nonprocedure_signature_ledgers() {
    let mut types = TypeRegistry::new();
    let mut foreign = TypeRegistry::new();
    let foreign_signature = signature(&mut foreign, CallingConvention::Jai, ContextMode::None);
    let boolean = types.scalar(ScalarType::Bool);
    for ty in [foreign_signature, boolean] {
        assert!(matches!(
            verify_constant_procedures(
                &types,
                &leaf(ty, TARGET),
                &HashMap::from([(ProcedureId::new(TARGET), ty)])
            ),
            Err(IrError::Type(_))
        ));
    }
    let local = signature(&mut types, CallingConvention::Jai, ContextMode::None);
    assert!(matches!(
        verify_constant_procedures(&types, &leaf(local, TARGET), &HashMap::new()),
        Err(IrError::UnknownIdentity {
            kind: "constant procedure",
            index: TARGET
        })
    ));
}

#[test]
fn borrowed_proof_bounds_deep_malformed_trees_without_recursing_or_owning_them() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let mut types = TypeRegistry::new();
            let target = signature(&mut types, CallingConvention::Jai, ContextMode::None);
            let mut value = leaf(target, TARGET);
            for _ in 0..10_000 {
                value = ConstantValue {
                    ty: target,
                    kind: ConstantKind::Distinct(Box::new(value)),
                };
            }
            assert!(matches!(
                verify_constant_procedures(
                    &types,
                    &value,
                    &HashMap::from([(ProcedureId::new(TARGET), target)])
                ),
                Err(IrError::VerificationDepth)
            ));
            // The public helper borrows staging data. This intentionally malformed
            // test tree must be dismantled iteratively by its owner after rejection.
            while let ConstantKind::Distinct(inner) = value.kind {
                value = *inner;
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
