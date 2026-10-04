use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, FloatType, FloatValue, Integer, IntegerType, RecordKind,
    ScalarType, TypeError, TypeId, TypeRegistry, TypeView, Variadic,
};
use std::collections::HashMap;

fn signature(types: &mut TypeRegistry, parameters: &[TypeId], results: &[TypeId]) -> TypeId {
    types
        .procedure(jai_types::ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap()
}

fn block(statements: Vec<Statement>, flow: Flow) -> Block {
    Block {
        statements,
        flow,
    }
}

fn procedure(id: usize, signature: TypeId, body: Block) -> Procedure {
    Procedure {
        id: ProcedureId::new(id),
        signature,
        parameters: vec![],
        locals: vec![],
        body,
        cleanups: vec![],
    }
}

fn integer(ty: IntegerType, value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(ty, value).unwrap())
}

fn exit(transfer: Transfer) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer,
    })
}

fn expression_error(types: &dyn TypeView, value: &ValueExpr) -> IrError {
    match verify_expression(types, value, &HashMap::new(), &[], &Places::default()) {
        Err(error) => error,
        Ok(_) => panic!("invalid expression passed verification"),
    }
}

#[test]
fn sparse_procedures_and_foreign_prototypes_publish_complete_signatures() {
    let mut types = TypeRegistry::new();
    let own_signature = signature(&mut types, &[], &[]);
    let foreign_signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(17);
    let external = ProcedureId::new(93);
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure(
            id.index(),
            own_signature,
            block(
                vec![Statement::CallVoid(Call::new(external, vec![]))],
                Flow::FallsThrough,
            ),
        )])
        .prototypes(vec![ProcedurePrototype {
            id: external,
            signature: foreign_signature,
            origin: PrototypeOrigin::Foreign {
                symbol: "foreign_fn".into(),
                library: None,
            },
        }])
        .finish(EntryPoint::Void(id))
        .unwrap();
    assert_eq!(program.procedure_by_id(id).unwrap().id, id);
    assert!(program.procedure_by_id(external).is_none());
    assert_eq!(
        program.library().signature(external),
        Some(foreign_signature)
    );
    assert_eq!(program.signatures().len(), 2);
    assert!(program.checked_procedure(id).is_some());
}

#[test]
fn duplicate_body_id_is_rejected() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![
            procedure(7, sig, block(vec![], Flow::FallsThrough)),
            procedure(7, sig, block(vec![], Flow::FallsThrough)),
        ])
        .finish_library();
    assert!(result.is_err());
}

#[test]
fn body_and_prototype_cannot_share_an_identity() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure(4, sig, block(vec![], Flow::FallsThrough))])
        .prototypes(vec![ProcedurePrototype {
            id: ProcedureId::new(4),
            signature: sig,
            origin: PrototypeOrigin::Compiler,
        }])
        .finish_library();
    assert!(matches!(
        result,
        Err(IrError::DuplicateIdentity {
            kind: "procedure prototype",
            index: 4
        })
    ));
}

#[test]
fn foreign_prototype_requires_c_abi_without_context() {
    for (convention, context) in [
        (CallingConvention::Jai, ContextMode::None),
        (CallingConvention::C, ContextMode::Implicit),
    ] {
        let mut types = TypeRegistry::new();
        let sig = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention,
                context,
                variadic: Variadic::None,
            })
            .unwrap();
        let result = ProgramBuilder::new(types.freeze().unwrap())
            .prototypes(vec![ProcedurePrototype {
                id: ProcedureId::new(2),
                signature: sig,
                origin: PrototypeOrigin::Foreign {
                    symbol: "foreign_fn".into(),
                    library: None,
                },
            }])
            .finish_library();
        assert!(matches!(
            result,
            Err(IrError::UnknownIdentity {
                kind: "foreign ABI",
                index: 2
            })
        ));
    }
}

#[test]
fn local_load_cannot_cross_procedure_ownership() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let foreign = Local::new(ProcedureId::new(9), 0, ScalarType::Bool, &types);
    let mut owner = procedure(9, sig, block(vec![], Flow::FallsThrough));
    owner.locals.push(foreign);
    let user = procedure(
        3,
        sig,
        block(
            vec![Statement::DiscardValue(ValueExpr::Load(foreign.place()))],
            Flow::FallsThrough,
        ),
    );
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![owner, user])
        .finish_library();
    assert!(
        matches!(result, Err(IrError::LocalOwner { expected, actual }) if expected == ProcedureId::new(3) && actual == ProcedureId::new(9))
    );
}

#[test]
fn foreign_type_id_cannot_enter_checked_library() {
    let types = TypeRegistry::new().freeze().unwrap();
    let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
    let global = Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: foreign,
            kind: ConstantKind::Bool(true),
        }),
        &types,
    );
    assert!(
        matches!(ProgramBuilder::new(types).globals(vec![global]).finish_library(), Err(IrError::Type(TypeError::ForeignType(id))) if id == foreign)
    );
}

#[test]
fn record_build_preserves_initializer_order_and_requires_every_field_once() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [int, boolean]).unwrap();
    let first = types.field(record, 0).unwrap().id;
    let second = types.field(record, 1).unwrap().id;
    let types = types.freeze().unwrap();
    let value = ValueExpr::RecordBuild {
        ty: record,
        initializers: vec![
            (second, ValueExpr::Bool(BoolExpr::Constant(true))),
            (first, ValueExpr::Int(integer(IntegerType::S64, 8))),
        ],
    };
    let signatures = HashMap::new();
    let places = Places::default();
    let checked = verify_expression(&types, &value, &signatures, &[], &places).unwrap();
    let ValueExpr::RecordBuild {
        initializers, ..
    } = checked.expression()
    else {
        panic!()
    };
    assert_eq!(initializers[0].0, second);
    let duplicate = ValueExpr::RecordBuild {
        ty: record,
        initializers: vec![
            (first, ValueExpr::Int(integer(IntegerType::S64, 8))),
            (first, ValueExpr::Int(integer(IntegerType::S64, 9))),
        ],
    };
    assert!(matches!(
        expression_error(&types, &duplicate),
        IrError::DuplicateIdentity {
            kind: "record field",
            index: 0
        }
    ));
    let missing = ValueExpr::RecordBuild {
        ty: record,
        initializers: vec![(first, ValueExpr::Int(integer(IntegerType::S64, 8)))],
    };
    assert!(matches!(
        expression_error(&types, &missing),
        IrError::Arity {
            kind: "record initializers",
            expected: 2,
            actual: 1
        }
    ));
}

#[test]
fn float_constant_must_match_declared_width() {
    let types = TypeRegistry::new().freeze().unwrap();
    let value = ValueExpr::Float(FloatExpr::new(
        FloatType::F64,
        FloatExprKind::Constant(FloatValue::F32(1.0f32.to_bits())),
    ));
    assert!(matches!(
        expression_error(&types, &value),
        IrError::TypeMismatch { .. }
    ));
}

#[test]
fn enum_requires_explicit_integer_unwrap() {
    let mut types = TypeRegistry::new();
    let enumeration = types.reserve_enum(IntegerType::U8);
    let integer = Integer::checked(IntegerType::U8, 3).unwrap();
    types.define_enum(enumeration, [integer]).unwrap();
    let types = types.freeze().unwrap();
    let value = ValueExpr::Enum {
        ty: enumeration,
        value: integer,
    };
    let implicit = ValueExpr::Int(IntExpr::new(
        IntegerType::U8,
        IntExprKind::Value(Box::new(value.clone())),
    ));
    assert!(matches!(
        expression_error(&types, &implicit),
        IrError::TypeMismatch { .. }
    ));
    let explicit = ValueExpr::Int(IntExpr::new(
        IntegerType::U8,
        IntExprKind::EnumValue(Box::new(value)),
    ));
    assert!(verify_expression(&types, &explicit, &HashMap::new(), &[], &Places::default()).is_ok());
}

#[test]
fn return_arity_is_checked_against_signature() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let sig = signature(&mut types, &[], &[int]);
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure(
            1,
            sig,
            block(vec![exit(Transfer::ReturnVoid)], Flow::Terminates),
        )])
        .finish_library();
    assert!(matches!(
        result,
        Err(IrError::Arity {
            kind: "return values",
            expected: 1,
            actual: 0
        })
    ));
}

#[test]
fn forged_flow_summary_is_rejected() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure(0, sig, block(vec![], Flow::Terminates))])
        .finish_library();
    assert!(matches!(result, Err(IrError::InvalidFlow)));
}

#[test]
fn statement_after_terminal_transfer_is_rejected() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let result = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure(
            0,
            sig,
            block(
                vec![
                    exit(Transfer::ReturnVoid),
                    Statement::DiscardBool(BoolExpr::Constant(true)),
                ],
                Flow::Terminates,
            ),
        )])
        .finish_library();
    assert!(matches!(result, Err(IrError::InvalidFlow)));
}

#[test]
fn cleanup_cannot_return_from_procedure() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let mut proc = procedure(0, sig, block(vec![], Flow::FallsThrough));
    proc.cleanups
        .push(block(vec![exit(Transfer::ReturnVoid)], Flow::Terminates).into());
    assert!(matches!(
        ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![proc])
            .finish_library(),
        Err(IrError::ReturnFromCleanup)
    ));
}

#[test]
fn cleanup_self_dependency_inside_cases_subject_is_rejected() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let mut proc = procedure(0, sig, block(vec![], Flow::FallsThrough));
    proc.cleanups.push(
        block(
            vec![Statement::Cases(Cases {
                default_position: None,
                default_through: false,
                subject: Box::new(Statement::Cleanup(CleanupId::new(0))),
                arms: vec![],
                default: Some(block(vec![], Flow::FallsThrough)),
                flow: Flow::FallsThrough,
                exhaustive: false,
            })],
            Flow::FallsThrough,
        )
        .into(),
    );
    assert!(
        matches!(ProgramBuilder::new(types.freeze().unwrap()).procedures(vec![proc]).finish_library(), Err(IrError::CleanupCycle(id)) if id == CleanupId::new(0))
    );
}

#[test]
fn boolean_case_exhaustiveness_requires_stable_local_subject() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let global = Global::new(0, GlobalInitializer::Bool(false), &types);
    let place = BoolPlace::try_from_place(global.place(), &types).unwrap();
    let arms = [false, true]
        .into_iter()
        .map(|value| CaseArm {
            condition: BoolExpr::CompareBools(
                Equality::Equal,
                Box::new(BoolExpr::Load(place)),
                Box::new(BoolExpr::Constant(value)),
            ),
            body: block(vec![exit(Transfer::ReturnVoid)], Flow::Terminates),
            through: false,
        })
        .collect();
    let proc = procedure(
        0,
        sig,
        block(
            vec![Statement::Cases(Cases {
                default_position: None,
                default_through: false,
                subject: Box::new(Statement::StoreBool(place, BoolExpr::Constant(true))),
                arms,
                default: None,
                flow: Flow::Terminates,
                exhaustive: true,
            })],
            Flow::Terminates,
        ),
    );
    assert!(matches!(
        ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .procedures(vec![proc])
            .finish_library(),
        Err(IrError::InvalidExhaustiveness)
    ));
}

#[test]
fn ready_procedure_ignores_unused_incomplete_global_but_checks_reference() {
    let mut types = TypeRegistry::new();
    let incomplete = types.reserve_record(RecordKind::Struct);
    let sig = signature(&mut types, &[], &[]);
    let global = Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: incomplete,
            kind: ConstantKind::Zero,
        }),
        &types,
    );
    let globals = [global];
    let signatures = HashMap::from([(ProcedureId::new(8), sig)]);
    let places = Places::default();
    let untouched = procedure(8, sig, block(vec![], Flow::FallsThrough));
    assert!(verify_procedure(&types, &untouched, &signatures, &globals, &places).is_ok());
    let referenced = procedure(
        8,
        sig,
        block(
            vec![Statement::DiscardValue(ValueExpr::Load(globals[0].place()))],
            Flow::FallsThrough,
        ),
    );
    assert!(
        matches!(verify_procedure(&types, &referenced, &signatures, &globals, &places), Err(IrError::Type(TypeError::Incomplete(id))) if id == incomplete)
    );
}

#[test]
fn ready_global_identities_reject_compile_only_code_storage() {
    let mut types = TypeRegistry::new();
    let signature = signature(&mut types, &[], &[]);
    let code = types.code_type();
    let globals = [Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: code,
            kind: ConstantKind::Zero,
        }),
        &types,
    )];
    let procedure = procedure(0, signature, block(vec![], Flow::FallsThrough));
    let signatures = HashMap::from([(procedure.id, signature)]);
    assert!(
        matches!(verify_procedure(&types, &procedure, &signatures, &globals, &Places::default()), Err(IrError::InvalidValue(ty)) if ty == code)
    );
}

#[test]
fn root_expression_has_no_local_frame() {
    let types = TypeRegistry::new().freeze().unwrap();
    let local = Local::new(ProcedureId::new(0), 0, ScalarType::Bool, &types);
    assert!(matches!(
        expression_error(&types, &ValueExpr::Load(local.place())),
        IrError::UnknownIdentity {
            kind: "expression local",
            index: 0
        }
    ));
}

#[test]
fn root_conditionals_check_each_unselected_branch() {
    let types = TypeRegistry::new().freeze().unwrap();
    let ty = types.scalar(ScalarType::Bool);
    let local = Local::new(ProcedureId::new(0), 0, ScalarType::Bool, &types);
    for condition in [false, true] {
        let invalid = ValueExpr::Load(local.place());
        let valid = ValueExpr::Bool(BoolExpr::Constant(true));
        let (then_value, else_value) = if condition {
            (valid, invalid)
        } else {
            (invalid, valid)
        };
        let value = ValueExpr::Conditional {
            ty,
            expression: Box::new(Conditional {
                condition: BoolExpr::Constant(condition),
                then_value,
                else_value,
            }),
        };
        assert!(matches!(
            expression_error(&types, &value),
            IrError::UnknownIdentity {
                kind: "expression local",
                index: 0
            }
        ));
    }
}

#[test]
fn root_call_rejects_duplicate_parameter_bindings() {
    let mut types = TypeRegistry::new();
    let ty = types.scalar(ScalarType::Bool);
    let sig = signature(&mut types, &[ty, ty], &[]);
    let types = types.freeze().unwrap();
    let id = ProcedureId::new(21);
    let signatures = HashMap::from([(id, sig)]);
    let call = Call::new(
        id,
        vec![
            (
                ParameterId::new(0),
                ValueExpr::Bool(BoolExpr::Constant(false)),
            ),
            (
                ParameterId::new(0),
                ValueExpr::Bool(BoolExpr::Constant(true)),
            ),
        ],
    );
    assert!(matches!(
        verify_call(&types, &call, &signatures, &[], &Places::default()),
        Err(IrError::DuplicateIdentity {
            kind: "call parameter",
            index: 0
        })
    ));
}

#[test]
fn pointer_offset_requires_s64_operand() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Bool);
    let ty = types.pointer(element).unwrap();
    let types = types.freeze().unwrap();
    for width in [IntegerType::S8, IntegerType::S64] {
        let value = ValueExpr::PointerOffset {
            pointer: Box::new(ValueExpr::Zero(ty)),
            offset: integer(width, 1),
            subtract: false,
            ty,
        };
        let result =
            verify_expression(&types, &value, &HashMap::new(), &[], &Places::default()).map(|_| ());
        if width == IntegerType::S64 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(IrError::IntegerMismatch {
                    expected: IntegerType::S64,
                    actual: IntegerType::S8
                })
            ));
        }
    }
}

#[test]
fn value_index_requires_s64_operand() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Bool);
    let array = types.fixed_array(element, 1).unwrap();
    let types = types.freeze().unwrap();
    for width in [IntegerType::S8, IntegerType::S64] {
        let value = ValueExpr::Index {
            check: CheckMode::Enabled,
            base: Box::new(ValueExpr::Array {
                ty: array,
                elements: vec![ValueExpr::Bool(BoolExpr::Constant(true))],
            }),
            index: integer(width, 0),
            ty: element,
        };
        let result =
            verify_expression(&types, &value, &HashMap::new(), &[], &Places::default()).map(|_| ());
        if width == IntegerType::S64 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(
                result,
                Err(IrError::IntegerMismatch {
                    expected: IntegerType::S64,
                    actual: IntegerType::S8
                })
            ));
        }
    }
}

#[test]
fn checked_expression_rejects_foreign_projection_arena() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Bool);
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [element]).unwrap();
    let global = Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: element,
                kind: ConstantKind::Bool(true),
            }]),
        }),
        &types,
    );
    let field = types.field(record, 0).unwrap().id;
    let mut source = PlaceRegistry::new();
    let projected = source.field(global.place(), field, &types).unwrap();
    let types = types.freeze().unwrap();
    let expression = ValueExpr::Load(projected);
    assert!(matches!(
        verify_expression(
            &types,
            &expression,
            &HashMap::new(),
            &[global],
            &Places::default()
        ),
        Err(IrError::ForeignProjection(_))
    ));
}

#[test]
fn deeply_nested_integer_is_rejected_without_execution() {
    let types = TypeRegistry::new().freeze().unwrap();
    let mut expression = integer(IntegerType::S64, 1);
    for _ in 0..300 {
        expression = IntExpr::new(IntegerType::S64, IntExprKind::Negate(Box::new(expression)));
    }
    assert!(matches!(
        expression_error(&types, &ValueExpr::Int(expression)),
        IrError::VerificationDepth
    ));
}

#[test]
fn deeply_nested_block_is_rejected_without_execution() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let mut body = block(vec![], Flow::FallsThrough);
    for _ in 0..300 {
        body = block(vec![Statement::Block(body)], Flow::FallsThrough);
    }
    assert!(matches!(
        ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![procedure(0, sig, body)])
            .finish_library(),
        Err(IrError::VerificationDepth)
    ));
}

#[test]
fn deeply_nested_record_constant_is_rejected_without_execution() {
    let mut types = TypeRegistry::new();
    let leaf = types.scalar(ScalarType::Bool);
    let mut value = ConstantValue {
        ty: leaf,
        kind: ConstantKind::Bool(true),
    };
    for _ in 0..300 {
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [value.ty]).unwrap();
        value = ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![value]),
        };
    }
    let global = Global::new(0, GlobalInitializer::Value(value), &types);
    assert!(matches!(
        ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .finish_library(),
        Err(IrError::VerificationDepth)
    ));
}

#[test]
fn pointer_difference_accepts_s64_result_for_nonzero_pointee() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(element).unwrap();
    let types = types.freeze().unwrap();
    let value = ValueExpr::Int(IntExpr::new(
        IntegerType::S64,
        IntExprKind::PointerDifference {
            left: Box::new(ValueExpr::Zero(pointer)),
            right: Box::new(ValueExpr::Zero(pointer)),
        },
    ));
    assert!(verify_expression(&types, &value, &HashMap::new(), &[], &Places::default()).is_ok());
}

#[test]
fn pointer_difference_rejects_non_s64_result() {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Bool);
    let pointer = types.pointer(element).unwrap();
    let types = types.freeze().unwrap();
    let value = ValueExpr::Int(IntExpr::new(
        IntegerType::S8,
        IntExprKind::PointerDifference {
            left: Box::new(ValueExpr::Zero(pointer)),
            right: Box::new(ValueExpr::Zero(pointer)),
        },
    ));
    assert!(matches!(
        expression_error(&types, &value),
        IrError::IntegerMismatch {
            expected: IntegerType::S64,
            actual: IntegerType::S8
        }
    ));
}

#[test]
fn pointer_difference_rejects_zero_sized_pointee() {
    let mut types = TypeRegistry::new();
    let element = types.reserve_record(RecordKind::Struct);
    types.define_record(element, []).unwrap();
    let pointer = types.pointer(element).unwrap();
    let types = types.freeze().unwrap();
    let value = ValueExpr::Int(IntExpr::new(
        IntegerType::S64,
        IntExprKind::PointerDifference {
            left: Box::new(ValueExpr::Zero(pointer)),
            right: Box::new(ValueExpr::Zero(pointer)),
        },
    ));
    assert!(matches!(expression_error(&types, &value), IrError::InvalidValue(id) if id == element));
}

fn overlapping_record_chain(length: usize, reverse_fields: bool) -> (TypeRegistry, TypeId) {
    let mut types = TypeRegistry::new();
    let mut previous = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut fields = Vec::with_capacity(length);
    for _ in 0..length {
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [previous]).unwrap();
        fields.push(record);
        previous = record;
    }
    if reverse_fields {
        fields.reverse();
    }
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, fields).unwrap();
    (types, root)
}

#[test]
fn type_depth_includes_memoized_descendants_visited_shallow_first() {
    // Each root field shares the previous field's subtree. A visited-type set
    // alone would hide the growing depth after checking its shallowest field.
    let (types, root) = overlapping_record_chain(400, false);
    assert!(matches!(
        expression_error(&types, &ValueExpr::Zero(root)),
        IrError::VerificationDepth
    ));
}

#[test]
fn type_depth_rejects_overlapping_chain_visited_deep_first() {
    let (types, root) = overlapping_record_chain(400, true);
    assert!(matches!(
        expression_error(&types, &ValueExpr::Zero(root)),
        IrError::VerificationDepth
    ));
}

#[test]
fn type_depth_accepts_shorter_overlapping_chain_in_both_orders() {
    for reverse_fields in [false, true] {
        let (types, root) = overlapping_record_chain(100, reverse_fields);
        assert!(
            verify_expression(
                &types,
                &ValueExpr::Zero(root),
                &HashMap::new(),
                &[],
                &Places::default()
            )
            .is_ok()
        );
    }
}


#[test]
fn forged_default_position_and_last_default_through_are_rejected() {
    for (position, through) in [(Some(2), false), (Some(1), true)] {
        let mut types = TypeRegistry::new();
        let sig = signature(&mut types, &[], &[]);
        let cases = Cases {
            subject: Box::new(Statement::Block(block(vec![], Flow::FallsThrough))),
            arms: vec![CaseArm {
                condition: BoolExpr::Constant(true),
                body: block(vec![], Flow::FallsThrough),
                through: false,
            }],
            default: Some(block(vec![], Flow::FallsThrough)),
            default_position: position,
            default_through: through,
            exhaustive: false,
            flow: Flow::FallsThrough,
        };
        let proc = procedure(
            0,
            sig,
            block(vec![Statement::Cases(cases)], Flow::FallsThrough),
        );
        assert!(matches!(
            ProgramBuilder::new(types.freeze().unwrap())
                .procedures(vec![proc])
                .finish_library(),
            Err(IrError::InvalidFlow)
        ));
    }
}

#[test]
fn default_first_through_chain_proves_real_return_flow() {
    let mut types = TypeRegistry::new();
    let sig = signature(&mut types, &[], &[]);
    let cases = Cases {
        subject: Box::new(Statement::Block(block(vec![], Flow::FallsThrough))),
        arms: vec![CaseArm {
            condition: BoolExpr::Constant(false),
            body: block(vec![exit(Transfer::ReturnVoid)], Flow::Terminates),
            through: false,
        }],
        default: Some(block(vec![], Flow::FallsThrough)),
        default_position: Some(0),
        default_through: true,
        exhaustive: false,
        flow: Flow::Terminates,
    };
    let proc = procedure(
        0,
        sig,
        block(vec![Statement::Cases(cases)], Flow::Terminates),
    );
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![proc])
        .finish_library()
        .unwrap();
}
