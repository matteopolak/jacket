use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, InlineHint, ProcedureType, ScalarType, TypeId, TypeRegistry,
    Variadic,
};
use std::collections::HashMap;

fn signature(types: &mut TypeRegistry, parameters: &[TypeId], results: &[TypeId]) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}

fn bytes(ty: TypeId) -> ValueExpr {
    ValueExpr::StringBytes {
        ty,
        bytes: vec![b'a', 0, b'b', 0xff],
    }
}

fn compare(relation: Equality, left: ValueExpr, right: ValueExpr) -> ValueExpr {
    ValueExpr::Bool(BoolExpr::CompareStrings(
        relation,
        Box::new(left),
        Box::new(right),
    ))
}

#[test]
fn string_comparison_proves_constants_loads_and_calls_with_embedded_nul_bytes() {
    let mut types = TypeRegistry::new();
    let string = types.string();
    let callee = signature(&mut types, &[], &[string]);
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: string,
            kind: ConstantKind::StringBytes(vec![b'a', 0, b'b', 0xff]),
        },
        &types,
    )
    .unwrap();
    let operands = [
        bytes(string),
        ValueExpr::Load(global.place()),
        ValueExpr::Call {
            call: Call::new(ProcedureId::new(93), vec![]),
            ty: string,
        },
    ];
    let globals = [global];
    let signatures = HashMap::from([(ProcedureId::new(93), callee)]);
    let places = Places::default();
    for relation in [Equality::Equal, Equality::NotEqual] {
        for left in &operands {
            for right in &operands {
                let expression = compare(relation, left.clone(), right.clone());
                let proof =
                    verify_expression(&types, &expression, &signatures, &globals, &places).unwrap();
                assert_eq!(
                    proof.expression().type_id(proof.types()),
                    types.scalar(ScalarType::Bool)
                );
            }
        }
        verify_expression(
            &types,
            &compare(
                relation,
                ValueExpr::StringBytes {
                    ty: string,
                    bytes: vec![],
                },
                bytes(string),
            ),
            &signatures,
            &globals,
            &places,
        )
        .unwrap();
    }
}

#[test]
fn either_nonstring_operand_is_rejected_even_when_both_operands_agree() {
    let types = TypeRegistry::new();
    let string = types.string();
    let boolean = types.scalar(ScalarType::Bool);
    for (left, right) in [
        (ValueExpr::Zero(boolean), bytes(string)),
        (bytes(string), ValueExpr::Zero(boolean)),
        (ValueExpr::Zero(boolean), ValueExpr::Zero(boolean)),
    ] {
        assert!(
            verify_expression(
                &types,
                &compare(Equality::Equal, left, right),
                &HashMap::new(),
                &[],
                &Places::default(),
            )
            .is_err()
        );
    }
}

#[test]
fn either_foreign_registry_string_operand_is_rejected() {
    let types = TypeRegistry::new();
    let foreign = TypeRegistry::new();
    for left_foreign in [false, true] {
        let (left, right) = if left_foreign {
            (bytes(foreign.string()), bytes(types.string()))
        } else {
            (bytes(types.string()), bytes(foreign.string()))
        };
        assert!(matches!(
            verify_expression(
                &types,
                &compare(Equality::Equal, left, right),
                &HashMap::new(),
                &[],
                &Places::default()
            ),
            Err(IrError::Type(_))
        ));
    }
}

#[test]
fn string_comparison_rejects_invalid_children_in_unselected_conditional_branches() {
    let types = TypeRegistry::new();
    let string = types.string();
    let boolean = types.scalar(ScalarType::Bool);
    for condition in [false, true] {
        let valid = BoolExpr::CompareStrings(
            Equality::Equal,
            Box::new(bytes(string)),
            Box::new(bytes(string)),
        );
        let invalid = BoolExpr::CompareStrings(
            Equality::Equal,
            Box::new(ValueExpr::Zero(boolean)),
            Box::new(bytes(string)),
        );
        let (then_value, else_value) = if condition {
            (valid, invalid)
        } else {
            (invalid, valid)
        };
        let expression = ValueExpr::Bool(BoolExpr::Conditional(Box::new(Conditional {
            condition: BoolExpr::Constant(condition),
            then_value,
            else_value,
        })));
        assert!(
            verify_expression(
                &types,
                &expression,
                &HashMap::new(),
                &[],
                &Places::default()
            )
            .is_err()
        );

        let (then_value, else_value) = if condition {
            (bytes(string), ValueExpr::Zero(boolean))
        } else {
            (ValueExpr::Zero(boolean), bytes(string))
        };
        let operand = ValueExpr::Conditional {
            ty: string,
            expression: Box::new(Conditional {
                condition: BoolExpr::Constant(condition),
                then_value,
                else_value,
            }),
        };
        assert!(matches!(
            verify_expression(
                &types,
                &compare(Equality::Equal, operand, bytes(string)),
                &HashMap::new(),
                &[],
                &Places::default()
            ),
            Err(IrError::TypeMismatch { .. })
        ));
    }
}

fn ordered_arguments() -> Vec<(ParameterId, ValueExpr)> {
    vec![
        (
            ParameterId::new(1),
            ValueExpr::Bool(BoolExpr::Constant(true)),
        ),
        (
            ParameterId::new(0),
            ValueExpr::Bool(BoolExpr::Constant(false)),
        ),
    ]
}

fn assert_arguments(call: &Call) {
    assert_eq!(call.arguments.len(), 2);
    assert_eq!(call.arguments[0].0.index(), 1);
    assert_eq!(call.arguments[1].0.index(), 0);
    assert!(matches!(
        call.arguments[0].1,
        ValueExpr::Bool(BoolExpr::Constant(true))
    ));
    assert!(matches!(
        call.arguments[1].1,
        ValueExpr::Bool(BoolExpr::Constant(false))
    ));
}

#[test]
fn call_constructor_and_hint_updates_preserve_procedure_and_ordered_arguments() {
    let call = Call::new(ProcedureId::new(93), ordered_arguments());
    assert_eq!(call.inline_hint(), InlineHint::Automatic);
    assert_arguments(&call);
    for hint in [InlineHint::Always, InlineHint::Never, InlineHint::Automatic] {
        let call = call.clone().with_inline_hint(hint);
        assert_eq!(call.inline_hint(), hint);
        assert_eq!(call.procedure, ProcedureId::new(93));
        assert_arguments(&call);
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

fn block(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::FallsThrough,
    }
}

fn returns(value: ValueExpr) -> Block {
    Block {
        statements: vec![Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::ReturnValues(vec![value]),
        })],
        flow: Flow::Terminates,
    }
}

#[test]
fn direct_known_call_publication_preserves_each_call_site_hint() {
    for hint in [InlineHint::Automatic, InlineHint::Always, InlineHint::Never] {
        let mut types = TypeRegistry::new();
        let boolean = types.scalar(ScalarType::Bool);
        let target = signature(&mut types, &[boolean, boolean], &[]);
        let caller = signature(&mut types, &[], &[]);
        let parameters = (0..2)
            .map(|index| Local::new_typed(ProcedureId::new(93), index, boolean, &types).unwrap())
            .collect::<Vec<_>>();
        let mut target_procedure = procedure(93, target, block(vec![]));
        target_procedure.parameters = parameters.clone();
        target_procedure.locals = parameters;
        let call = Call::new(ProcedureId::new(93), ordered_arguments()).with_inline_hint(hint);
        let library = ProgramBuilder::new(types.freeze().unwrap())
            .procedures(vec![
                procedure(17, caller, block(vec![Statement::CallVoid(call)])),
                target_procedure,
            ])
            .finish_library()
            .unwrap();
        let Statement::CallVoid(call) = &library.procedures()[0].body.statements[0] else {
            panic!("direct call changed during publication");
        };
        assert_eq!(call.inline_hint(), hint);
        assert_arguments(call);
    }
}

fn indirect(hint: InlineHint, value_form: bool, source: usize) -> Result<Library, IrError> {
    let mut types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    let target = signature(&mut types, &[], &[boolean]);
    let factory = signature(&mut types, &[], &[target]);
    let caller = signature(&mut types, &[], &[]);
    let mut globals = vec![];
    let callee = match source {
        0 => ValueExpr::ProcedureValue {
            procedure: ProcedureId::new(93),
            ty: target,
        },
        1 => ValueExpr::Zero(target),
        2 => {
            let global = Global::new_typed(
                0,
                ConstantValue {
                    ty: target,
                    kind: ConstantKind::Procedure(ProcedureId::new(93)),
                },
                &types,
            )
            .unwrap();
            let value = ValueExpr::Load(global.place());
            globals.push(global);
            value
        }
        3 => ValueExpr::Call {
            call: Call::new(ProcedureId::new(94), vec![]),
            ty: target,
        },
        _ => unreachable!(),
    };
    let statement = if value_form {
        Statement::DiscardValue(ValueExpr::IndirectCall {
            inline_hint: hint,
            callee: Box::new(callee),
            arguments: vec![],
            ty: boolean,
        })
    } else {
        Statement::IndirectCallResults {
            inline_hint: hint,
            callee: Box::new(callee),
            arguments: vec![],
            destinations: vec![None],
        }
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .globals(globals)
        .procedures(vec![
            procedure(17, caller, block(vec![statement])),
            procedure(
                93,
                target,
                returns(ValueExpr::Bool(BoolExpr::Constant(true))),
            ),
            procedure(
                94,
                factory,
                returns(ValueExpr::ProcedureValue {
                    procedure: ProcedureId::new(93),
                    ty: target,
                }),
            ),
        ])
        .finish_library()
}

#[test]
fn indirect_always_hint_accepts_only_literal_procedure_values_in_both_forms() {
    for value_form in [false, true] {
        let library = indirect(InlineHint::Always, value_form, 0).unwrap();
        let stored = match &library.procedures()[0].body.statements[0] {
            Statement::DiscardValue(ValueExpr::IndirectCall { inline_hint, .. }) => *inline_hint,
            Statement::IndirectCallResults { inline_hint, .. } => *inline_hint,
            _ => panic!("indirect call changed during publication"),
        };
        assert_eq!(stored, InlineHint::Always);
        for source in 1..4 {
            assert!(matches!(
                indirect(InlineHint::Always, value_form, source),
                Err(IrError::InvalidValue(_))
            ));
        }
    }
}

#[test]
fn indirect_automatic_and_never_hints_accept_normally_typed_nonliteral_callees() {
    for hint in [InlineHint::Automatic, InlineHint::Never] {
        for value_form in [false, true] {
            for source in 0..4 {
                let library = indirect(hint, value_form, source).unwrap();
                let stored = match &library.procedures()[0].body.statements[0] {
                    Statement::DiscardValue(ValueExpr::IndirectCall { inline_hint, .. }) => {
                        *inline_hint
                    }
                    Statement::IndirectCallResults { inline_hint, .. } => *inline_hint,
                    _ => panic!("indirect call changed during publication"),
                };
                assert_eq!(stored, hint);
            }
        }
    }
}

#[test]
fn permissive_indirect_hints_still_require_a_procedure_callee() {
    let types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    for hint in [InlineHint::Automatic, InlineHint::Never] {
        let expression = ValueExpr::IndirectCall {
            inline_hint: hint,
            callee: Box::new(ValueExpr::Zero(boolean)),
            arguments: vec![],
            ty: boolean,
        };
        assert!(
            verify_expression(
                &types,
                &expression,
                &HashMap::new(),
                &[],
                &Places::default()
            )
            .is_err()
        );
    }
}
