use super::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, ProcedureType, ScalarType, TypeRegistry, Variadic,
};

fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn integer(value: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(IntegerType::S64, value))
}
fn root(plan: &Plan) -> NodeId {
    let Entry::Node(root) = plan.entry else {
        panic!("expected expression entry")
    };
    root
}

#[test]
fn literal_plan_owns_bytes_and_admits_payload_before_copying() {
    let types = TypeRegistry::new();
    let signatures = HashMap::new();
    let places = Places::default();
    let mut expression = ValueExpr::StringBytes {
        ty: types.string(),
        bytes: vec![1, 2, 3],
    };
    let checked = verify_expression(&types, &expression, &signatures, &[], &places).unwrap();
    let plan = compile_checked_expression(checked, Limits::default()).unwrap();
    let ValueExpr::StringBytes {
        bytes, ..
    } = &mut expression
    else {
        unreachable!()
    };
    bytes.fill(7);
    drop(expression);
    let NodeKind::Apply {
        op: ApplyOp::StringBytes {
            bytes, ..
        },
        ..
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected owned literal")
    };
    assert_eq!(bytes.as_ref(), [1, 2, 3]);

    let expression = ValueExpr::StringBytes {
        ty: types.string(),
        bytes: vec![1; 4096],
    };
    let checked = verify_expression(&types, &expression, &signatures, &[], &places).unwrap();
    assert!(matches!(
        compile_checked_expression(
            checked,
            Limits {
                value_cells: 64,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::ValueCells))
    ));
}

#[test]
fn storage_bitcast_keeps_a_place_operand_without_loading_its_source() {
    let types = TypeRegistry::new();
    let source_type = types.scalar(ScalarType::Int(IntegerType::U64));
    let target_type = types.float(jai_types::FloatType::F64);
    let cast = jai_types::StorageBitcast::prove(
        &types,
        jai_types::LayoutPolicy::lp64(),
        source_type,
        target_type,
        jai_types::StorageBitcastStrength::EqualSize,
    )
    .unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: source_type,
            kind: ConstantKind::Zero,
        },
        &types,
    )
    .unwrap();
    let expression = ValueExpr::StorageBitcast {
        source: StorageBitcastSource::Place(global.place()),
        cast,
    };
    let signatures = HashMap::new();
    let places = Places::default();
    let globals = [global];
    let checked = verify_expression(&types, &expression, &signatures, &globals, &places).unwrap();
    let plan = compile_checked_expression(checked, Limits::default()).unwrap();
    let NodeKind::Apply {
        op:
            ApplyOp::StorageBitcast {
                cast: actual,
                from_place: true,
            },
        operands,
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected storage bitcast")
    };
    assert_eq!(*actual, cast);
    assert_eq!(operands.len(), 1);
    assert!(matches!(
        plan.nodes[operands[0].index()].kind,
        NodeKind::Place {
            op: PlaceOp::Global(_),
            ..
        }
    ));
    assert!(!plan.nodes.iter().any(|node| matches!(
        node.kind,
        NodeKind::Apply {
            op: ApplyOp::Load,
            ..
        }
    )));
}

#[test]
fn lazy_boolean_plan_keeps_the_call_in_a_separately_scheduled_branch() {
    let mut types = TypeRegistry::new();
    let boolean = types.scalar(jai_types::ScalarType::Bool);
    let callee = ProcedureId::new(7);
    let signature = signature(&mut types, vec![], vec![boolean]);
    let signatures = HashMap::from([(callee, signature)]);
    let places = Places::default();
    let expression = ValueExpr::Bool(BoolExpr::And(
        Box::new(BoolExpr::Constant(false)),
        Box::new(BoolExpr::Call(Call::new(callee, vec![]))),
    ));
    let checked = verify_expression(&types, &expression, &signatures, &[], &places).unwrap();
    let plan = compile_checked_expression(checked, Limits::default()).unwrap();
    let NodeKind::Apply {
        operands, ..
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected scalar wrapper")
    };
    let NodeKind::ShortCircuit {
        op: ShortCircuitOp::And,
        left,
        right,
    } = &plan.nodes[operands[0].index()].kind
    else {
        panic!("expected lazy branch")
    };
    assert!(matches!(
        plan.nodes[left.index()].kind,
        NodeKind::Apply {
            op: ApplyOp::Literal(Value::Bool(false)),
            ..
        }
    ));
    assert!(
        matches!(plan.nodes[right.index()].kind, NodeKind::Call { target: CallTarget::Direct(id), .. } if id == callee)
    );
}

#[test]
fn calls_retain_argument_order_before_parameter_reordering() {
    let mut types = TypeRegistry::new();
    let integer_type = types.scalar(ScalarType::Int(IntegerType::S64));
    let callee = ProcedureId::new(3);
    let signature = signature(
        &mut types,
        vec![integer_type, integer_type],
        vec![integer_type],
    );
    let signatures = HashMap::from([(callee, signature)]);
    let places = Places::default();
    let call = Call::new(
        callee,
        vec![
            (ParameterId::new(1), ValueExpr::Int(integer(10))),
            (ParameterId::new(0), ValueExpr::Int(integer(20))),
        ],
    );
    let checked = verify_call(&types, &call, &signatures, &[], &places).unwrap();
    let plan = compile_checked_call(checked, Limits::default()).unwrap();
    let NodeKind::Call {
        arguments,
        signature: actual,
        ..
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected call")
    };
    assert_eq!(*actual, signature);
    assert_eq!(
        arguments
            .iter()
            .map(|(id, _)| id.index())
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert!(arguments[0].1.index() < arguments[1].1.index());
}

#[test]
fn indexed_places_have_a_descriptor_capture_phase_before_index_calls() {
    let mut types = TypeRegistry::new();
    let integer_type = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(integer_type).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: slice,
            kind: ConstantKind::Zero,
        },
        &types,
    )
    .unwrap();
    let callee = ProcedureId::new(9);
    let signature = signature(&mut types, vec![], vec![integer_type]);
    let signatures = HashMap::from([(callee, signature)]);
    let mut places = PlaceRegistry::new();
    let place = places
        .index(
            global.place(),
            IntExpr::new(
                IntegerType::S64,
                IntExprKind::Call(Call::new(callee, vec![])),
            ),
            &types,
        )
        .unwrap();
    let places = places.freeze();
    let expression = ValueExpr::Load(place);
    let globals = [global];
    let checked = verify_expression(&types, &expression, &signatures, &globals, &places).unwrap();
    let plan = compile_checked_expression(checked, Limits::default()).unwrap();
    let NodeKind::Apply {
        op: ApplyOp::Load,
        operands,
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected indexed load")
    };
    let NodeKind::IndexPlace {
        base,
        index,
        base_type,
        ..
    } = &plan.nodes[operands[0].index()].kind
    else {
        panic!("expected capture phase")
    };
    assert_eq!(*base_type, slice);
    assert!(matches!(
        plan.nodes[base.index()].kind,
        NodeKind::Place {
            op: PlaceOp::Global(_),
            ..
        }
    ));
    assert!(matches!(
        plan.nodes[index.index()].kind,
        NodeKind::Call { .. }
    ));
}

#[test]
fn pack_parts_distinguish_places_and_immediate_spread_snapshots() {
    let mut types = TypeRegistry::new();
    let integer_type = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(integer_type, 2).unwrap();
    let slice = types.slice(integer_type).unwrap();
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: array,
            kind: ConstantKind::Zero,
        },
        &types,
    )
    .unwrap();
    let mut places = PlaceRegistry::new();
    let first = places.index(global.place(), integer(0), &types).unwrap();
    let places = places.freeze();
    let variadic = ProcedureId::new(1);
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![slice].into(),
            results: vec![integer_type].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::Jai {
                parameter: 0,
                element: integer_type,
            },
        })
        .unwrap();
    let signatures = HashMap::from([(variadic, signature)]);
    let call = Call::new(
        variadic,
        vec![(
            ParameterId::new(0),
            ValueExpr::SequenceConcat {
                ty: slice,
                parts: vec![
                    SequencePackPart::Element(ValueExpr::Load(first)),
                    SequencePackPart::Spread(ValueExpr::ArrayToSlice {
                        array: global.place(),
                        ty: slice,
                    }),
                    SequencePackPart::Element(ValueExpr::Int(integer(42))),
                ],
            },
        )],
    );
    let globals = [global];
    let checked = verify_call(&types, &call, &signatures, &globals, &places).unwrap();
    let plan = compile_checked_call(checked, Limits::default()).unwrap();
    let NodeKind::Call {
        arguments, ..
    } = &plan.nodes[root(&plan).index()].kind
    else {
        panic!("expected call")
    };
    let NodeKind::SequencePack {
        parts, ..
    } = &plan.nodes[arguments[0].1.index()].kind
    else {
        panic!("expected ordered pack")
    };
    assert!(matches!(parts[0].mode, PackPartMode::ElementPlace));
    assert!(matches!(parts[1].mode, PackPartMode::Spread));
    assert!(matches!(parts[2].mode, PackPartMode::ElementValue));
}

#[test]
fn checked_deep_trees_still_obey_the_plan_depth_limit() {
    let types = TypeRegistry::new();
    let mut boolean = BoolExpr::Constant(true);
    for _ in 0..20 {
        boolean = BoolExpr::Not(Box::new(boolean));
    }
    let expression = ValueExpr::Bool(boolean);
    let signatures = HashMap::new();
    let places = Places::default();
    let checked = verify_expression(&types, &expression, &signatures, &[], &places).unwrap();
    assert!(matches!(
        compile_checked_expression(
            checked,
            Limits {
                evaluation_depth: 8,
                ..Limits::default()
            }
        ),
        Err(Error::Limit(LimitKind::EvaluationDepth))
    ));
}
