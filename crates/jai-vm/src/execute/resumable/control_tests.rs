//! Checked control-flow parity with actual suspension, without source replay.
use super::*;
use jai_types::{
    CallingConvention, ContextMode, Direction, IntegerType, RecordKind, Relation, ScalarType,
    TypeRegistry, Variadic,
};
use std::cell::Cell;

struct Provider {
    library: Library,
    pending: Cell<Option<ProcedureId>>,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        self.library.types()
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        self.library.signatures()
    }
    fn globals(&self) -> &[Global] {
        self.library.globals()
    }
    fn places(&self) -> Option<&Places> {
        Some(self.library.places())
    }
    fn context(&self) -> Option<&ContextDefinition> {
        self.library.context()
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if self.pending.get() == Some(id) {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        self.library
            .checked_procedure(id)
            .map_or(ProcedureAvailability::Missing, ProcedureAvailability::Ready)
    }
}
fn signature(
    types: &mut TypeRegistry,
    parameters: Vec<TypeId>,
    results: Vec<TypeId>,
    context: ContextMode,
) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn int(value: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(IntegerType::S64, value))
}
fn value(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}
fn binary(op: IntOp, left: IntExpr, right: IntExpr) -> IntExpr {
    IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(op, Box::new(left), Box::new(right)),
    )
}
fn compare(op: Relation, left: IntExpr, right: IntExpr) -> BoolExpr {
    BoolExpr::CompareInts(op, Box::new(left), Box::new(right))
}
fn call_int(id: usize) -> IntExpr {
    IntExpr::new(
        IntegerType::S64,
        IntExprKind::Call(Call::new(ProcedureId::new(id), vec![])),
    )
}
fn falls(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::FallsThrough,
    }
}
fn ends(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::Terminates,
    }
}
fn ret(expression: IntExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnInt(expression),
    })
}
fn ret_value(expression: ValueExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![expression]),
    })
}
fn procedure(
    id: usize,
    signature: TypeId,
    parameters: Vec<Local>,
    locals: Vec<Local>,
    statements: Vec<Statement>,
) -> Procedure {
    Procedure {
        id: ProcedureId::new(id),
        signature,
        parameters,
        locals,
        body: ends(statements),
        cleanups: vec![],
    }
}
fn global(types: &TypeRegistry) -> (Global, IntPlace) {
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        types,
    );
    let place = IntPlace::try_from_place(global.place(), types).unwrap();
    (global, place)
}
fn append(place: IntPlace, digit: i128) -> Statement {
    Statement::StoreInt(
        place,
        binary(
            IntOp::Add,
            binary(IntOp::Multiply, IntExpr::load(place), int(10)),
            int(digit),
        ),
    )
}
fn read_global(vm: &Vm<'_, Provider, crate::NoEffects>) -> Value {
    vm.memory
        .load(vm.provider.types(), vm.globals[0].as_ref().unwrap())
        .unwrap()
}
fn assert_suspended(vm: &Vm<'_, Provider, crate::NoEffects>, outcome: ResumableOutcome, id: usize) {
    assert_eq!(
        outcome,
        ResumableOutcome::Suspended(vec![Dependency::Procedure(ProcedureId::new(id))])
    );
    assert!(vm.resumable_values().is_none());
}
fn finish(vm: &mut Vm<'_, Provider, crate::NoEffects>, expected: i128) {
    assert_eq!(vm.resumable_values(), Some([value(expected)].as_slice()));
    assert_eq!(
        vm.finish_resumable_validated(|_, _| Ok(())).outcome,
        Outcome::Complete(vec![value(expected)])
    );
}

#[test]
fn lazy_boolean_and_conditional_nodes_skip_pending_branches() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let main_signature = signature(&mut types, vec![], vec![word], ContextMode::None);
    let bool_signature = signature(&mut types, vec![], vec![boolean], ContextMode::None);
    let (global, place) = global(&types);
    let pending = || BoolExpr::Call(Call::new(ProcedureId::new(1), vec![]));
    let chosen = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Conditional(Box::new(Conditional {
            condition: BoolExpr::And(Box::new(BoolExpr::Constant(false)), Box::new(pending())),
            then_value: binary(IntOp::Divide, int(1), int(0)),
            else_value: int(42),
        })),
    );
    let main = procedure(
        0,
        main_signature,
        vec![],
        vec![],
        vec![
            Statement::If(
                BoolExpr::Or(Box::new(BoolExpr::Constant(true)), Box::new(pending())),
                falls(vec![Statement::StoreInt(place, chosen)]),
                falls(vec![Statement::DiscardInt(binary(
                    IntOp::Divide,
                    int(1),
                    int(0),
                ))]),
            ),
            ret(IntExpr::load(place)),
        ],
    );
    let wait = procedure(
        1,
        bool_signature,
        vec![],
        vec![],
        vec![Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::ReturnBool(BoolExpr::Constant(false)),
        })],
    );
    let provider = Provider {
        library: ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .procedures(vec![main, wait])
            .finish_library()
            .unwrap(),
        pending: Cell::new(Some(ProcedureId::new(1))),
    };
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(vm.statistics.calls, 1);
    finish(&mut vm, 42);
    assert_eq!(
        Vm::new(&provider, crate::NoEffects, Limits::default())
            .unwrap()
            .execute(ProcedureId::new(0), vec![])
            .outcome,
        Outcome::Complete(vec![value(42)])
    );
}

#[test]
fn string_index_places_capture_the_descriptor_before_a_pending_index_and_keep_literal_readonly() {
    for write in [false, true] {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let text_type = types.string();
        let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
        let (trace_global, trace) = global(&types);
        let text = Global::new_typed(
            1,
            ConstantValue {
                ty: text_type,
                kind: ConstantKind::StringBytes(b"abc".to_vec()),
            },
            &types,
        )
        .unwrap();
        let mut places = PlaceRegistry::new();
        let indexed = places.index(text.place(), call_int(1), &types).unwrap();
        let byte = ValueExpr::Load(indexed);
        let widened = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(
                CastMode::Checked,
                Box::new(IntExpr::new(
                    IntegerType::U8,
                    IntExprKind::Value(Box::new(byte)),
                )),
            ),
        );
        let statements = if write {
            vec![
                Statement::Store(
                    indexed,
                    ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::U8, 120))),
                ),
                ret(int(42)),
            ]
        } else {
            vec![ret(widened)]
        };
        let main = procedure(0, scalar, vec![], vec![], statements);
        let index = procedure(
            1,
            scalar,
            vec![],
            vec![],
            vec![
                append(trace, 1),
                Statement::Store(
                    text.place(),
                    ValueExpr::StringBytes {
                        ty: text_type,
                        bytes: b"xyz".to_vec(),
                    },
                ),
                ret(int(0)),
            ],
        );
        let provider = Provider {
            library: ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![trace_global, text])
                .places(places.freeze())
                .procedures(vec![main, index])
                .finish_library()
                .unwrap(),
            pending: Cell::new(Some(ProcedureId::new(1))),
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let outcome = vm
            .start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome;
        assert_suspended(&vm, outcome, 1);
        let storage = vm.globals[1].as_ref().unwrap();
        let Value::StringView {
            pointer: original, ..
        } = vm.memory.load(provider.types(), storage).unwrap()
        else {
            panic!("expected descriptor")
        };
        assert_eq!(vm.statistics.calls, 2);
        provider.pending.set(None);
        let resumed = vm.resume_resumable().outcome;
        if write {
            assert_eq!(resumed, ResumableOutcome::Failed(Error::ReadOnlyStorage));
            assert_eq!(vm.memory.allocation_count(), 0);
        } else {
            assert_eq!(resumed, ResumableOutcome::AwaitingPublication);
            assert_eq!(read_global(&vm), value(1));
            let storage = vm.globals[1].as_ref().unwrap();
            let current = vm.memory.load(provider.types(), storage).unwrap();
            assert_eq!(
                vm.materialize_value(&current),
                Ok(Value::String(b"xyz".to_vec()))
            );
            assert_eq!(
                vm.memory.store(
                    provider.types(),
                    &original,
                    Value::Int(Integer::wrapping(IntegerType::U8, 120))
                ),
                Err(Error::ReadOnlyStorage)
            );
            finish(&mut vm, 97);
        }
        let expected = if write {
            Outcome::Failed(Error::ReadOnlyStorage)
        } else {
            Outcome::Complete(vec![value(97)])
        };
        assert_eq!(
            Vm::new(&provider, crate::NoEffects, Limits::default())
                .unwrap()
                .execute(ProcedureId::new(0), vec![])
                .outcome,
            expected
        );
    }
}

fn argument_provider() -> (Provider, TypeId, TypeId) {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
    let callee_signature = signature(&mut types, vec![word, word], vec![word], ContextMode::None);
    let selector_signature = signature(
        &mut types,
        vec![],
        vec![callee_signature],
        ContextMode::None,
    );
    let (global, trace) = global(&types);
    let left = Local::new_typed(ProcedureId::new(1), 0, word, &types).unwrap();
    let right = Local::new_typed(ProcedureId::new(1), 1, word, &types).unwrap();
    let left_place = IntPlace::try_from_place(left.place(), &types).unwrap();
    let right_place = IntPlace::try_from_place(right.place(), &types).unwrap();
    let target = procedure(
        1,
        callee_signature,
        vec![left, right],
        vec![left, right],
        vec![ret(binary(
            IntOp::Add,
            binary(IntOp::Multiply, IntExpr::load(left_place), int(10)),
            IntExpr::load(right_place),
        ))],
    );
    let first = procedure(
        2,
        scalar,
        vec![],
        vec![],
        vec![append(trace, 1), ret(int(1))],
    );
    let compiled_default = procedure(
        3,
        scalar,
        vec![],
        vec![],
        vec![append(trace, 2), ret(int(2))],
    );
    let selector = procedure(
        4,
        selector_signature,
        vec![],
        vec![],
        vec![
            append(trace, 9),
            ret_value(ValueExpr::ProcedureValue {
                procedure: ProcedureId::new(1),
                ty: callee_signature,
            }),
        ],
    );
    let provider = Provider {
        library: ProgramBuilder::new(types.freeze().unwrap())
            .globals(vec![global])
            .procedures(vec![target, first, compiled_default, selector])
            .finish_library()
            .unwrap(),
        pending: Cell::new(Some(ProcedureId::new(1))),
    };
    (provider, word, callee_signature)
}
fn ordered_arguments() -> Vec<(ParameterId, ValueExpr)> {
    // Default expansion has already produced checked expressions. Preserve their
    // source order, then reorder only when binding the callee parameters.
    vec![
        (ParameterId::new(1), ValueExpr::Int(call_int(2))),
        (ParameterId::new(0), ValueExpr::Int(call_int(3))),
    ]
}

#[test]
fn compiled_default_arguments_finish_once_before_a_pending_direct_callee() {
    let (provider, _, _) = argument_provider();
    let call = Call::new(ProcedureId::new(1), ordered_arguments());
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let outcome = vm.start_resumable_call(&call).outcome;
    assert_suspended(&vm, outcome, 1);
    assert_eq!(read_global(&vm), value(12));
    assert_eq!(vm.statistics.calls, 3);
    provider.pending.set(None);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(read_global(&vm), value(12));
    assert_eq!(vm.statistics.calls, 3);
    finish(&mut vm, 21);
    assert_eq!(
        Vm::new(&provider, crate::NoEffects, Limits::default())
            .unwrap()
            .evaluate_call(&call)
            .outcome,
        Outcome::Complete(vec![value(21)])
    );
}

#[test]
fn indirect_callee_is_captured_once_and_checked_before_compiled_default_arguments() {
    let (provider, word, callee_signature) = argument_provider();
    let expression = ValueExpr::IndirectCall {
        inline_hint: jai_types::InlineHint::Automatic,
        callee: Box::new(ValueExpr::Call {
            call: Call::new(ProcedureId::new(4), vec![]),
            ty: callee_signature,
        }),
        arguments: ordered_arguments(),
        ty: word,
    };
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let outcome = vm.start_resumable_expression(&expression).outcome;
    assert_suspended(&vm, outcome, 1);
    assert_eq!(read_global(&vm), value(9));
    provider.pending.set(None);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(read_global(&vm), value(912));
    finish(&mut vm, 21);
    assert_eq!(
        Vm::new(&provider, crate::NoEffects, Limits::default())
            .unwrap()
            .evaluate(&expression)
            .outcome,
        Outcome::Complete(vec![value(21)])
    );
}

#[test]
fn suspended_bound_while_condition_preserves_bindings_break_and_continue_edges() {
    for skip_and_break in [false, true] {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
        let (global, counter) = global(&types);
        let sum = Local::new_typed(ProcedureId::new(0), 0, word, &types)
            .unwrap()
            .integer(&types)
            .unwrap();
        let bound = Local::new_typed(ProcedureId::new(0), 1, word, &types)
            .unwrap()
            .integer(&types)
            .unwrap();
        let loop_id = LoopId::new(0);
        let mut body = vec![];
        if skip_and_break {
            for (number, transfer) in [
                (2, Transfer::Continue(loop_id)),
                (3, Transfer::Break(loop_id)),
            ] {
                body.push(Statement::If(
                    compare(Relation::Equal, IntExpr::load(bound.place()), int(number)),
                    ends(vec![Statement::Exit(Exit {
                        cleanups: vec![],
                        transfer,
                    })]),
                    falls(vec![]),
                ));
            }
        }
        body.push(Statement::StoreInt(
            sum.place(),
            binary(
                IntOp::Add,
                IntExpr::load(sum.place()),
                IntExpr::load(bound.place()),
            ),
        ));
        let main = procedure(
            0,
            scalar,
            vec![],
            vec![sum.local(), bound.local()],
            vec![
                Statement::StoreInt(sum.place(), int(0)),
                Statement::While {
                    id: loop_id,
                    condition: LoopCondition::BoundInt(bound, call_int(1)),
                    body: falls(body),
                },
                ret(IntExpr::load(sum.place())),
            ],
        );
        let next = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Conditional(Box::new(Conditional {
                condition: compare(Relation::LessEqual, IntExpr::load(counter), int(3)),
                then_value: IntExpr::load(counter),
                else_value: int(0),
            })),
        );
        let condition = procedure(
            1,
            scalar,
            vec![],
            vec![],
            vec![
                Statement::StoreInt(counter, binary(IntOp::Add, IntExpr::load(counter), int(1))),
                ret(next),
            ],
        );
        let provider = Provider {
            library: ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .procedures(vec![main, condition])
                .finish_library()
                .unwrap(),
            pending: Cell::new(Some(ProcedureId::new(1))),
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let outcome = vm
            .start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome;
        assert_suspended(&vm, outcome, 1);
        provider.pending.set(None);
        assert_eq!(
            vm.resume_resumable().outcome,
            ResumableOutcome::AwaitingPublication
        );
        let expected = if skip_and_break {
            1
        } else {
            6
        };
        assert_eq!(
            read_global(&vm),
            value(if skip_and_break {
                3
            } else {
                4
            })
        );
        assert_eq!(
            vm.statistics.calls,
            if skip_and_break {
                4
            } else {
                5
            }
        );
        finish(&mut vm, expected);
        assert_eq!(
            Vm::new(&provider, crate::NoEffects, Limits::default())
                .unwrap()
                .execute(ProcedureId::new(0), vec![])
                .outcome,
            Outcome::Complete(vec![value(expected)])
        );
    }
}

#[test]
fn suspended_forward_and_reverse_ranges_keep_endpoint_progress_without_wrapping() {
    for direction in [Direction::Forward, Direction::Reverse] {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
        let (global, trace) = global(&types);
        let iterator = Local::new(
            ProcedureId::new(0),
            0,
            ScalarType::Int(IntegerType::U8),
            &types,
        )
        .integer(&types)
        .unwrap();
        let widened = IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(CastMode::Checked, Box::new(IntExpr::load(iterator.place()))),
        );
        let main = procedure(
            0,
            scalar,
            vec![],
            vec![iterator.local()],
            vec![
                Statement::Range(RangeLoop {
                    id: LoopId::new(0),
                    iterator,
                    start: IntExpr::constant(Integer::wrapping(IntegerType::U8, 254)),
                    end: IntExpr::constant(Integer::wrapping(IntegerType::U8, 255)),
                    direction,
                    body: falls(vec![
                        Statement::StoreInt(
                            trace,
                            binary(
                                IntOp::Add,
                                binary(IntOp::Multiply, IntExpr::load(trace), int(256)),
                                widened,
                            ),
                        ),
                        Statement::DiscardInt(call_int(1)),
                    ]),
                }),
                ret(IntExpr::load(trace)),
            ],
        );
        let wait = procedure(1, scalar, vec![], vec![], vec![ret(int(0))]);
        let provider = Provider {
            library: ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .procedures(vec![main, wait])
                .finish_library()
                .unwrap(),
            pending: Cell::new(Some(ProcedureId::new(1))),
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let outcome = vm
            .start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome;
        assert_suspended(&vm, outcome, 1);
        assert_eq!(
            read_global(&vm),
            value(if direction == Direction::Forward {
                254
            } else {
                255
            })
        );
        provider.pending.set(None);
        assert_eq!(
            vm.resume_resumable().outcome,
            ResumableOutcome::AwaitingPublication
        );
        let expected = if direction == Direction::Forward {
            65279
        } else {
            65534
        };
        assert_eq!(vm.statistics.calls, 3);
        finish(&mut vm, expected);
        assert_eq!(
            Vm::new(&provider, crate::NoEffects, Limits::default())
                .unwrap()
                .execute(ProcedureId::new(0), vec![])
                .outcome,
            Outcome::Complete(vec![value(expected)])
        );
    }
}

#[test]
fn cases_subject_and_through_progress_survive_two_pending_boundaries() {
    for matches in [false, true] {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
        let boolean_signature = signature(&mut types, vec![], vec![boolean], ContextMode::None);
        let (global, trace) = global(&types);
        let main = procedure(
            0,
            scalar,
            vec![],
            vec![],
            vec![
                Statement::Cases(Cases {
                    default_position: None,
                    default_through: false,
                    subject: Box::new(append(trace, 1)),
                    arms: vec![
                        CaseArm {
                            condition: BoolExpr::Call(Call::new(ProcedureId::new(1), vec![])),
                            body: falls(vec![append(trace, 2)]),
                            through: true,
                        },
                        CaseArm {
                            condition: BoolExpr::Call(Call::new(ProcedureId::new(3), vec![])),
                            body: falls(vec![append(trace, 3), Statement::DiscardInt(call_int(2))]),
                            through: false,
                        },
                    ],
                    default: Some(falls(vec![append(trace, 4)])),
                    flow: Flow::FallsThrough,
                    exhaustive: false,
                }),
                ret(IntExpr::load(trace)),
            ],
        );
        let condition = procedure(
            1,
            boolean_signature,
            vec![],
            vec![],
            vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnBool(BoolExpr::Constant(matches)),
            })],
        );
        let wait = procedure(2, scalar, vec![], vec![], vec![ret(int(0))]);
        let skipped_condition = procedure(
            3,
            boolean_signature,
            vec![],
            vec![],
            vec![
                append(trace, 9),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnBool(BoolExpr::Constant(false)),
                }),
            ],
        );
        let provider = Provider {
            library: ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .procedures(vec![main, condition, wait, skipped_condition])
                .finish_library()
                .unwrap(),
            pending: Cell::new(Some(ProcedureId::new(1))),
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let outcome = vm
            .start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome;
        assert_suspended(&vm, outcome, 1);
        assert_eq!(read_global(&vm), value(1));
        provider.pending.set(Some(ProcedureId::new(2)));
        let resumed = vm.resume_resumable().outcome;
        let expected = if matches {
            assert_suspended(&vm, resumed, 2);
            assert_eq!(read_global(&vm), value(123));
            provider.pending.set(None);
            assert_eq!(
                vm.resume_resumable().outcome,
                ResumableOutcome::AwaitingPublication
            );
            123
        } else {
            assert_eq!(resumed, ResumableOutcome::AwaitingPublication);
            194
        };
        assert_eq!(vm.statistics.calls, 3);
        finish(&mut vm, expected);
        provider.pending.set(None);
        assert_eq!(
            Vm::new(&provider, crate::NoEffects, Limits::default())
                .unwrap()
                .execute(ProcedureId::new(0), vec![])
                .outcome,
            Outcome::Complete(vec![value(expected)])
        );
    }
}

#[test]
fn pending_cleanup_retains_captured_procedure_and_outer_push_contexts() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [word]).unwrap();
    let pointer = types.pointer(record).unwrap();
    let context = ContextDefinition {
        record_type: record,
        pointer_type: pointer,
        default: ConstantValue {
            ty: record,
            kind: ConstantKind::Record(vec![ConstantValue {
                ty: word,
                kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, 10)),
            }]),
        },
    };
    let mut places = PlaceRegistry::new();
    let field = places
        .field(
            Place::context(record, &types).unwrap(),
            types.field(record, 0).unwrap().id,
            &types,
        )
        .unwrap();
    let read = || {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Value(Box::new(ValueExpr::Load(field))),
        )
    };
    let increment = |amount| {
        Statement::Store(
            field,
            ValueExpr::Int(binary(IntOp::Add, read(), int(amount))),
        )
    };
    let copy = |number| ValueExpr::Record {
        ty: record,
        fields: vec![ValueExpr::Int(int(number))],
    };
    let implicit = signature(&mut types, vec![], vec![word], ContextMode::Implicit);
    let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
    let outer = PushContextId::new(ProcedureId::new(0), 0);
    let inner = PushContextId::new(ProcedureId::new(0), 1);
    let (observed_global, observed) = global(&types);
    let mut main = procedure(
        0,
        implicit,
        vec![],
        vec![],
        vec![Statement::PushContext {
            id: outer,
            value: copy(7),
            body: ends(vec![Statement::PushContext {
                id: inner,
                value: copy(50),
                body: ends(vec![Statement::Exit(Exit {
                    cleanups: vec![CleanupId::new(0), CleanupId::new(1)],
                    transfer: Transfer::ReturnInt(read()),
                })]),
            }]),
        }],
    );
    main.cleanups = vec![
        Cleanup {
            context: CleanupContext::Procedure,
            body: falls(vec![
                increment(2),
                Statement::DiscardInt(call_int(1)),
                increment(1),
            ]),
        },
        Cleanup {
            context: CleanupContext::Push(outer),
            body: falls(vec![increment(3), Statement::StoreInt(observed, read())]),
        },
    ];
    let wait = procedure(1, scalar, vec![], vec![], vec![ret(int(0))]);
    let reader = procedure(2, implicit, vec![], vec![], vec![ret(read())]);
    let provider = Provider {
        library: ProgramBuilder::new(types.freeze().unwrap())
            .context(context)
            .globals(vec![observed_global])
            .places(places.freeze())
            .procedures(vec![main, wait, reader])
            .finish_library()
            .unwrap(),
        pending: Cell::new(Some(ProcedureId::new(1))),
    };
    let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
    let outcome = vm
        .start_resumable_procedure(ProcedureId::new(0), vec![])
        .outcome;
    assert_suspended(&vm, outcome, 1);
    let default = vm.default_context.as_ref().unwrap();
    assert_eq!(vm.current_context.as_ref(), Some(default));
    let default_field = vm.memory.field(provider.types(), default, 0).unwrap();
    assert_eq!(
        vm.memory.load(provider.types(), &default_field).unwrap(),
        value(12)
    );
    let outer_root = vm.frames[0].push_contexts.get(&outer).unwrap();
    let outer_field = vm.memory.field(provider.types(), outer_root, 0).unwrap();
    provider.pending.set(None);
    assert_eq!(
        vm.resume_resumable().outcome,
        ResumableOutcome::AwaitingPublication
    );
    assert_eq!(
        vm.memory.load(provider.types(), &default_field).unwrap(),
        value(13)
    );
    assert_eq!(read_global(&vm), value(10));
    // The pushed roots have been released with the returning frame.
    assert_eq!(
        vm.memory.load(provider.types(), &outer_field),
        Err(Error::DanglingPointer)
    );
    finish(&mut vm, 50);
    assert_eq!(
        vm.execute(ProcedureId::new(2), vec![]).outcome,
        Outcome::Complete(vec![value(13)])
    );
    assert_eq!(
        Vm::new(&provider, crate::NoEffects, Limits::default())
            .unwrap()
            .execute(ProcedureId::new(0), vec![])
            .outcome,
        Outcome::Complete(vec![value(50)])
    );
}

#[test]
fn middle_default_fallthrough_survives_actual_pending_boundaries() {
    for matches in [false, true] {
        let mut types = TypeRegistry::new();
        let word = types.scalar(ScalarType::Int(IntegerType::S64));
        let boolean = types.scalar(ScalarType::Bool);
        let scalar = signature(&mut types, vec![], vec![word], ContextMode::None);
        let boolean_signature = signature(&mut types, vec![], vec![boolean], ContextMode::None);
        let (global, trace) = global(&types);
        let main = procedure(
            0,
            scalar,
            vec![],
            vec![],
            vec![
                Statement::Cases(Cases {
                    default_position: Some(1),
                    default_through: true,
                    subject: Box::new(append(trace, 1)),
                    arms: vec![
                        CaseArm {
                            condition: BoolExpr::Call(Call::new(ProcedureId::new(1), vec![])),
                            body: falls(vec![append(trace, 2)]),
                            through: true,
                        },
                        CaseArm {
                            condition: BoolExpr::Call(Call::new(ProcedureId::new(3), vec![])),
                            body: falls(vec![append(trace, 3), Statement::DiscardInt(call_int(2))]),
                            through: false,
                        },
                    ],
                    default: Some(falls(vec![append(trace, 4)])),
                    flow: Flow::FallsThrough,
                    exhaustive: false,
                }),
                ret(IntExpr::load(trace)),
            ],
        );
        let condition = procedure(
            1,
            boolean_signature,
            vec![],
            vec![],
            vec![Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnBool(BoolExpr::Constant(matches)),
            })],
        );
        let wait = procedure(2, scalar, vec![], vec![], vec![ret(int(0))]);
        let skipped_condition = procedure(
            3,
            boolean_signature,
            vec![],
            vec![],
            vec![
                append(trace, 9),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnBool(BoolExpr::Constant(false)),
                }),
            ],
        );
        let provider = Provider {
            library: ProgramBuilder::new(types.freeze().unwrap())
                .globals(vec![global])
                .procedures(vec![main, condition, wait, skipped_condition])
                .finish_library()
                .unwrap(),
            pending: Cell::new(Some(ProcedureId::new(1))),
        };
        let mut vm = Vm::new(&provider, crate::NoEffects, Limits::default()).unwrap();
        let outcome = vm
            .start_resumable_procedure(ProcedureId::new(0), vec![])
            .outcome;
        assert_suspended(&vm, outcome, 1);
        assert_eq!(read_global(&vm), value(1));
        provider.pending.set(Some(ProcedureId::new(2)));
        let resumed = vm.resume_resumable().outcome;
        let expected = if matches {
            assert_suspended(&vm, resumed, 2);
            assert_eq!(read_global(&vm), value(1243));
            provider.pending.set(None);
            assert_eq!(
                vm.resume_resumable().outcome,
                ResumableOutcome::AwaitingPublication
            );
            1243
        } else {
            assert_suspended(&vm, resumed, 2);
            assert_eq!(read_global(&vm), value(1943));
            provider.pending.set(None);
            assert_eq!(
                vm.resume_resumable().outcome,
                ResumableOutcome::AwaitingPublication
            );
            1943
        };
        assert_eq!(
            vm.statistics.calls,
            if matches {
                3
            } else {
                4
            }
        );
        finish(&mut vm, expected);
        provider.pending.set(None);
        assert_eq!(
            Vm::new(&provider, crate::NoEffects, Limits::default())
                .unwrap()
                .execute(ProcedureId::new(0), vec![])
                .outcome,
            Outcome::Complete(vec![value(expected)])
        );
    }
}
