use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, ScalarType, TypeId,
    TypeRegistry, Variadic,
};

const CALLEE: usize = 31;

fn signature(
    types: &mut TypeRegistry,
    parameters: &[TypeId],
    convention: CallingConvention,
    variadic: Variadic,
) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention,
            context: ContextMode::None,
            variadic,
        })
        .unwrap()
}

fn publish(
    mut types: TypeRegistry,
    callee: TypeId,
    statement: Statement,
) -> Result<Library, IrError> {
    let caller = signature(&mut types, &[], CallingConvention::Jai, Variadic::None);
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id: ProcedureId::new(17),
            signature: caller,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![statement],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .prototypes(vec![ProcedurePrototype {
            id: ProcedureId::new(CALLEE),
            signature: callee,
            origin: PrototypeOrigin::Compiler,
        }])
        .finish_library()
}

fn invoke(indirect: bool, callee: TypeId, arguments: Vec<(ParameterId, ValueExpr)>) -> Statement {
    if indirect {
        Statement::IndirectCallResults {
            inline_hint: jai_types::InlineHint::Automatic,
            callee: Box::new(ValueExpr::ProcedureValue {
                procedure: ProcedureId::new(CALLEE),
                ty: callee,
            }),
            arguments,
            destinations: vec![],
        }
    } else {
        Statement::CallVoid(Call::new(ProcedureId::new(CALLEE), arguments))
    }
}

fn fixture() -> (TypeRegistry, TypeId, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let element = types.scalar(ScalarType::Int(IntegerType::S64));
    let slice = types.slice(element).unwrap();
    let callee = signature(
        &mut types,
        &[slice, slice],
        CallingConvention::Jai,
        Variadic::Jai {
            parameter: 1,
            element,
        },
    );
    (types, element, slice, callee)
}

fn integer(value: i128) -> ValueExpr {
    ValueExpr::Int(IntExpr::constant(
        Integer::checked(IntegerType::S64, value).unwrap(),
    ))
}

fn pack(ty: TypeId, parts: Vec<SequencePackPart>) -> ValueExpr {
    ValueExpr::SequenceConcat {
        ty,
        parts,
    }
}

fn arguments(slice: TypeId, value: ValueExpr) -> Vec<(ParameterId, ValueExpr)> {
    vec![
        (ParameterId::new(1), value),
        (ParameterId::new(0), ValueExpr::Zero(slice)),
    ]
}

#[test]
fn direct_and_indirect_jai_pack_calls_publish_in_source_order() {
    for indirect in [false, true] {
        let (types, _, slice, callee) = fixture();
        let value = pack(
            slice,
            vec![
                SequencePackPart::Element(integer(7)),
                SequencePackPart::Spread(ValueExpr::Zero(slice)),
                SequencePackPart::Element(integer(9)),
            ],
        );
        let library = publish(
            types,
            callee,
            invoke(indirect, callee, arguments(slice, value)),
        )
        .unwrap();
        let stored = match &library.procedures()[0].body.statements[0] {
            Statement::CallVoid(call) => &call.arguments,
            Statement::IndirectCallResults {
                arguments, ..
            } => arguments,
            _ => panic!("call shape changed during publication"),
        };
        assert_eq!(stored[0].0.index(), 1);
        assert_eq!(stored[1].0.index(), 0);
        let ValueExpr::SequenceConcat {
            parts, ..
        } = &stored[0].1
        else {
            panic!("ordered pack was replaced during publication");
        };
        assert_eq!(parts.len(), 3);
        for (index, expected) in [(0, 7), (2, 9)] {
            let SequencePackPart::Element(ValueExpr::Int(value)) = &parts[index] else {
                panic!("pack element order changed");
            };
            let IntExprKind::Constant(value) = value.kind() else {
                panic!("pack element changed");
            };
            assert_eq!(value.value(), expected);
        }
        assert!(matches!(parts[1], SequencePackPart::Spread(_)));
    }
}

#[test]
fn empty_ordered_pack_is_a_valid_exact_jai_argument() {
    for indirect in [false, true] {
        let (types, _, slice, callee) = fixture();
        publish(
            types,
            callee,
            invoke(indirect, callee, arguments(slice, pack(slice, vec![]))),
        )
        .unwrap();
    }
}

#[test]
fn standalone_and_nonpack_parameter_concatenations_are_rejected() {
    let (types, _, slice, callee) = fixture();
    assert!(matches!(
        publish(types, callee, Statement::DiscardValue(pack(slice, vec![]))),
        Err(IrError::InvalidValue(actual)) if actual == slice
    ));
    for indirect in [false, true] {
        let (types, _, slice, callee) = fixture();
        assert!(matches!(
            publish(types, callee, invoke(indirect, callee, vec![
                (ParameterId::new(0), pack(slice, vec![])),
                (ParameterId::new(1), ValueExpr::Zero(slice)),
            ])),
            Err(IrError::InvalidValue(actual)) if actual == slice
        ));
    }
}

#[test]
fn nested_concatenations_are_rejected_inside_spreads_and_conditionals() {
    for indirect in [false, true] {
        for conditional in [false, true] {
            let (types, _, slice, callee) = fixture();
            let nested = pack(slice, vec![]);
            let value = if conditional {
                ValueExpr::Conditional {
                    ty: slice,
                    expression: Box::new(Conditional {
                        condition: BoolExpr::Constant(true),
                        then_value: nested,
                        else_value: ValueExpr::Zero(slice),
                    }),
                }
            } else {
                pack(slice, vec![SequencePackPart::Spread(nested)])
            };
            assert!(matches!(
                publish(types, callee, invoke(indirect, callee, arguments(slice, value))),
                Err(IrError::InvalidValue(actual)) if actual == slice
            ));
        }
    }
}

#[test]
fn nonvariadic_and_c_calls_cannot_accept_ordered_jai_packs() {
    for indirect in [false, true] {
        for kind in 0..3 {
            let (mut types, _, slice, _) = fixture();
            let (convention, variadic) = match kind {
                0 => (CallingConvention::Jai, Variadic::None),
                1 => (CallingConvention::C, Variadic::None),
                2 => (
                    CallingConvention::C,
                    Variadic::C {
                        fixed_parameters: 2,
                    },
                ),
                _ => unreachable!(),
            };
            let callee = signature(&mut types, &[slice, slice], convention, variadic);
            assert!(matches!(
                publish(types, callee, invoke(indirect, callee, arguments(slice, pack(slice, vec![])))),
                Err(IrError::InvalidValue(actual)) if actual == slice
            ));
        }
    }
}

#[test]
fn pack_type_must_be_the_exact_declared_element_slice() {
    for indirect in [false, true] {
        for kind in 0..3 {
            let (mut types, element, slice, callee) = fixture();
            let wrong = match kind {
                0 => {
                    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
                    types.slice(byte).unwrap()
                }
                1 => types.fixed_array(element, 0).unwrap(),
                2 => types.dynamic_array(element).unwrap(),
                _ => unreachable!(),
            };
            assert!(
                publish(
                    types,
                    callee,
                    invoke(indirect, callee, arguments(slice, pack(wrong, vec![]))),
                )
                .is_err()
            );
        }
    }
}

#[test]
fn spread_requires_slice_identity_even_when_element_types_match() {
    for indirect in [false, true] {
        for kind in 0..3 {
            let (mut types, element, slice, callee) = fixture();
            let wrong = match kind {
                0 => types.fixed_array(element, 0).unwrap(),
                1 => types.dynamic_array(element).unwrap(),
                2 => {
                    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
                    types.slice(byte).unwrap()
                }
                _ => unreachable!(),
            };
            let value = pack(
                slice,
                vec![SequencePackPart::Spread(ValueExpr::Zero(wrong))],
            );
            assert!(matches!(
                publish(types, callee, invoke(indirect, callee, arguments(slice, value))),
                Err(IrError::TypeMismatch { expected, actual }) if expected == slice && actual == wrong
            ));
        }
    }
}

#[test]
fn scalar_pack_elements_require_the_exact_declared_type() {
    for indirect in [false, true] {
        let (types, element, slice, callee) = fixture();
        let wrong = types.scalar(ScalarType::Int(IntegerType::U8));
        let value = pack(
            slice,
            vec![SequencePackPart::Element(ValueExpr::Zero(wrong))],
        );
        assert!(matches!(
            publish(types, callee, invoke(indirect, callee, arguments(slice, value))),
            Err(IrError::TypeMismatch { expected, actual }) if expected == element && actual == wrong
        ));
    }
}

#[test]
fn foreign_registry_pack_type_is_rejected() {
    for indirect in [false, true] {
        let (types, _, slice, callee) = fixture();
        let mut foreign = TypeRegistry::new();
        let element = foreign.scalar(ScalarType::Int(IntegerType::S64));
        let foreign_slice = foreign.slice(element).unwrap();
        assert!(matches!(
            publish(
                types,
                callee,
                invoke(
                    indirect,
                    callee,
                    arguments(slice, pack(foreign_slice, vec![]))
                )
            ),
            Err(IrError::Type(_))
        ));
    }
}

#[test]
fn allocation_charge_checks_padding_metadata_alignment_and_overflow() {
    assert_eq!(sequence_temp_allocation_charge(8, 8), Some(87));
    assert_eq!(sequence_temp_allocation_charge(8, 32), Some(103));
    // Zero bytes still represent a nonempty pack of zero-sized elements.
    // Empty packs bypass this helper in the caller.
    assert_eq!(sequence_temp_allocation_charge(0, 1), Some(80));
    for alignment in [0, 3, 7] {
        assert_eq!(sequence_temp_allocation_charge(1, alignment), None);
    }
    let largest = u64::MAX - 15 - SEQUENCE_TEMP_ALLOCATION_OVERHEAD;
    assert_eq!(sequence_temp_allocation_charge(largest, 16), Some(u64::MAX));
    assert_eq!(sequence_temp_allocation_charge(largest + 1, 16), None);
    assert_eq!(sequence_temp_allocation_charge(u64::MAX, 16), None);
}
