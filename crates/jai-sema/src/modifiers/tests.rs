use super::*;
use crate::compile_time::ReadyProcedures;
use jai_ir::{
    Block, BoolExpr, Flow, IntExpr, Procedure, ProcedureId, Statement, Transfer, ValueExpr,
};
use jai_source::Symbols;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, ScalarType, TypeRegistry,
    Variadic,
};
use jai_vm::{Limits, NoEffects};
use std::collections::HashMap;

fn procedure(types: &mut TypeRegistry, accept: bool) -> (Procedure, TypeId) {
    let string = types.string();
    let number = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([types.scalar(ScalarType::Bool), string, number]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    (
        Procedure {
            id: ProcedureId::new(0),
            signature,
            parameters: vec![],
            locals: vec![],
            cleanups: vec![],
            body: Block {
                statements: vec![Statement::Exit(jai_ir::Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnValues(vec![
                        ValueExpr::Bool(BoolExpr::Constant(accept)),
                        ValueExpr::StringBytes {
                            ty: string,
                            bytes: b"constraint failed".to_vec(),
                        },
                        ValueExpr::Int(IntExpr::constant(
                            Integer::checked(IntegerType::S64, 8).unwrap(),
                        )),
                    ]),
                })],
                flow: Flow::Terminates,
            },
        },
        signature,
    )
}

#[test]
fn checked_modifier_publishes_only_accepted_complete_bindings() {
    for accept in [false, true] {
        let mut types = TypeRegistry::new();
        let (procedure, signature) = procedure(&mut types, accept);
        let procedures = HashMap::from([(procedure.id, procedure)]);
        let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
        let places = jai_ir::Places::default();
        let provider =
            ReadyProcedures::new(&types, &procedures, &signatures, &[], &places).unwrap();
        let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
        let mut symbols = Symbols::default();
        let name = symbols.intern("N");
        let initial = Substitution {
            types: vec![],
            constants: vec![ConstantBinding {
                name,
                value: BakedValue::integer(Integer::checked(IntegerType::S64, 3).unwrap(), &types),
            }],
            callables: vec![],
        };
        let original = initial.clone();
        let plan = ModifierPlan::new(vec![ModifierSlot::Baked {
            name,
            ty: types.scalar(ScalarType::Int(IntegerType::S64)),
        }])
        .unwrap();
        let result = execute(
            &mut vm,
            &Call::new(ProcedureId::new(0), vec![]),
            &plan,
            &initial,
            &types,
            256,
        );
        assert_eq!(initial, original);
        match result {
            ModifierOutcome::Accepted(modified) if accept => assert_eq!(
                modified
                    .constant(name)
                    .unwrap()
                    .as_integer()
                    .unwrap()
                    .value(),
                8
            ),
            ModifierOutcome::Rejected {
                reason,
            } if !accept => {
                assert_eq!(reason, "constraint failed")
            }
            other => panic!("unexpected modifier outcome {other:?}"),
        }
    }
}

#[test]
fn pending_modifier_does_not_publish_or_mutate_the_input() {
    let mut types = TypeRegistry::new();
    let (_, signature) = procedure(&mut types, true);
    let procedures = HashMap::new();
    let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
    let places = jai_ir::Places::default();
    let provider = ReadyProcedures::new(&types, &procedures, &signatures, &[], &places).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let initial = Substitution::default();
    let plan = ModifierPlan::new(vec![]).unwrap();
    assert!(matches!(
        execute(
            &mut vm,
            &Call::new(ProcedureId::new(0), vec![]),
            &plan,
            &initial,
            &types,
            256,
        ),
        ModifierOutcome::Pending(_)
    ));
    assert_eq!(initial, Substitution::default());
}

#[test]
fn invalid_modifier_plan_and_result_shape_fail_without_publication() {
    let mut types = TypeRegistry::new();
    let (procedure, signature) = procedure(&mut types, true);
    let procedures = HashMap::from([(procedure.id, procedure)]);
    let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
    let places = jai_ir::Places::default();
    let provider = ReadyProcedures::new(&types, &procedures, &signatures, &[], &places).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let mut symbols = Symbols::default();
    let name = symbols.intern("T");
    assert!(
        ModifierPlan::new(vec![
            ModifierSlot::Type {
                name
            },
            ModifierSlot::Type {
                name
            }
        ])
        .is_err()
    );
    let plan = ModifierPlan::new(vec![]).unwrap();
    assert!(matches!(
        execute(
            &mut vm,
            &Call::new(ProcedureId::new(0), vec![]),
            &plan,
            &Substitution::default(),
            &types,
            256,
        ),
        ModifierOutcome::Failed(Error::InvalidIr(
            "specialization modifier result count mismatch"
        ))
    ));
}

#[test]
fn rejection_rolls_back_vm_global_writes_while_acceptance_commits() {
    for accept in [false, true] {
        let mut types = TypeRegistry::new();
        let (mut procedure, signature) = procedure(&mut types, accept);
        let global = jai_ir::Global::new(
            0,
            jai_ir::GlobalInitializer::Int(Integer::checked(IntegerType::S64, 7).unwrap()),
            &types,
        );
        let place = global.place();
        procedure.body.statements.insert(
            0,
            Statement::Store(
                place,
                ValueExpr::Int(IntExpr::constant(
                    Integer::checked(IntegerType::S64, 99).unwrap(),
                )),
            ),
        );
        let globals = vec![global];
        let procedures = HashMap::from([(procedure.id, procedure)]);
        let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
        let places = jai_ir::Places::default();
        let provider =
            ReadyProcedures::new(&types, &procedures, &signatures, &globals, &places).unwrap();
        let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
        let mut symbols = Symbols::default();
        let name = symbols.intern("N");
        let initial = Substitution {
            types: vec![],
            constants: vec![ConstantBinding {
                name,
                value: BakedValue::integer(Integer::checked(IntegerType::S64, 3).unwrap(), &types),
            }],
            callables: vec![],
        };
        let plan = ModifierPlan::new(vec![ModifierSlot::Baked {
            name,
            ty: types.scalar(ScalarType::Int(IntegerType::S64)),
        }])
        .unwrap();
        let outcome = execute(
            &mut vm,
            &Call::new(ProcedureId::new(0), vec![]),
            &plan,
            &initial,
            &types,
            256,
        );
        assert_eq!(matches!(outcome, ModifierOutcome::Accepted(_)), accept);
        let loaded = vm.evaluate(&ValueExpr::Load(place));
        assert!(
            matches!(loaded.outcome, Outcome::Complete(values) if matches!(&values[..], [Value::Int(n)] if n.value() == if accept {99} else {7}))
        );
    }
}

#[test]
fn scalar_output_cannot_be_reinterpreted_as_a_type_identity() {
    let mut types = TypeRegistry::new();
    let (procedure, signature) = procedure(&mut types, true);
    let procedures = HashMap::from([(procedure.id, procedure)]);
    let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
    let places = jai_ir::Places::default();
    let provider = ReadyProcedures::new(&types, &procedures, &signatures, &[], &places).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let mut symbols = Symbols::default();
    let name = symbols.intern("T");
    let plan = ModifierPlan::new(vec![ModifierSlot::Type {
        name,
    }])
    .unwrap();
    let outcome = execute(
        &mut vm,
        &Call::new(ProcedureId::new(0), vec![]),
        &plan,
        &Substitution::default(),
        &types,
        256,
    );
    assert!(matches!(outcome, ModifierOutcome::Failed(_)));
}

#[test]
fn type_slot_decodes_an_actual_canonical_descriptor() {
    use jai_ir::{
        ConstantKind, ConstantValue, RuntimeTypeConstant, StaticDataBuilder, StaticDataLimits,
        StaticValue, StaticValueKind,
    };
    use jai_types::{
        LayoutPolicy, RecordKind, ReflectionGraph, ReflectionMetadata, ReflectionReadiness,
        TypeInfoTag,
    };
    use std::sync::Arc;
    let mut types = TypeRegistry::new();
    let tag = types.reserve_enum(IntegerType::U32);
    let boolean_tag = Integer::checked(IntegerType::U32, TypeInfoTag::Bool as i128).unwrap();
    types.define_enum(tag, vec![boolean_tag]).unwrap();
    let number = types.scalar(ScalarType::Int(IntegerType::S64));
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, vec![tag, number]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let boolean = types.scalar(ScalarType::Bool);
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        boolean,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("reflection pending")
    };
    let mut data = StaticDataBuilder::new();
    let object = data.reserve(header, &types).unwrap();
    data.define_type_descriptor(
        object,
        StaticValue {
            ty: header,
            kind: StaticValueKind::Record(vec![
                StaticValue::constant(ConstantValue {
                    ty: tag,
                    kind: ConstantKind::Enum(boolean_tag),
                }),
                StaticValue::constant(ConstantValue {
                    ty: number,
                    kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                }),
            ]),
        },
        &graph,
        graph.descriptor(boolean).unwrap().id,
        &types,
    )
    .unwrap();
    let data = Arc::new(data.finish(&types, StaticDataLimits::default()).unwrap());
    let runtime_type = RuntimeTypeConstant::new(data, object, &types).unwrap();
    let string = types.string();
    let meta = types.meta_type();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([boolean, string, meta]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let procedure = Procedure {
        id: ProcedureId::new(0),
        signature,
        parameters: vec![],
        locals: vec![],
        cleanups: vec![],
        body: Block {
            statements: vec![Statement::Exit(jai_ir::Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnValues(vec![
                    ValueExpr::Bool(BoolExpr::Constant(true)),
                    ValueExpr::StringBytes {
                        ty: string,
                        bytes: vec![],
                    },
                    ValueExpr::RuntimeType(runtime_type),
                ]),
            })],
            flow: Flow::Terminates,
        },
    };
    let procedures = HashMap::from([(procedure.id, procedure)]);
    let signatures = HashMap::from([(ProcedureId::new(0), signature)]);
    let places = jai_ir::Places::default();
    let provider = ReadyProcedures::new(&types, &procedures, &signatures, &[], &places).unwrap();
    let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
    let mut symbols = Symbols::default();
    let name = symbols.intern("T");
    let original_ty = types.scalar(ScalarType::Int(IntegerType::U8));
    let initial = Substitution {
        types: vec![crate::polymorphism::TypeBinding {
            name,
            ty: original_ty,
        }],
        constants: vec![ConstantBinding {
            name,
            value: BakedValue::Type(original_ty),
        }],
        callables: vec![],
    };
    let plan = ModifierPlan::new(vec![ModifierSlot::Type {
        name,
    }])
    .unwrap();
    let ModifierOutcome::Accepted(final_bindings) = execute(
        &mut vm,
        &Call::new(ProcedureId::new(0), vec![]),
        &plan,
        &initial,
        &types,
        256,
    ) else {
        panic!("modifier failed")
    };
    assert_eq!(final_bindings.ty(name), Some(boolean));
    assert_eq!(initial.ty(name), Some(original_ty));
    assert_eq!(
        final_bindings.constant(name),
        Some(&BakedValue::Type(boolean))
    );
    assert_eq!(initial.constant(name), Some(&BakedValue::Type(original_ty)));
}
