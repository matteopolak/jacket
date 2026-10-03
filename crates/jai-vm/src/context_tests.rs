//! Context tests start from independently verified libraries, with no reference code.
use crate::*;
use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, RecordKind, ScalarType,
    TypeRegistry, Variadic,
};

struct Fixture {
    types: TypeRegistry,
    context: ContextDefinition,
    places: PlaceRegistry,
    field: Place,
    procedures: Vec<Procedure>,
}
impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S64));
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, [integer]).unwrap();
        let pointer = types.pointer(record).unwrap();
        let mut places = PlaceRegistry::new();
        let root = Place::context(record, &types).unwrap();
        let field = places
            .field(root, types.field(record, 0).unwrap().id, &types)
            .unwrap();
        Self {
            types,
            context: ContextDefinition {
                record_type: record,
                pointer_type: pointer,
                default: ConstantValue {
                    ty: record,
                    kind: ConstantKind::Record(vec![ConstantValue {
                        ty: integer,
                        kind: ConstantKind::Int(number(10)),
                    }]),
                },
            },
            places,
            field,
            procedures: vec![],
        }
    }
    fn add(
        &mut self,
        mode: ContextMode,
        statements: Vec<Statement>,
        cleanups: Vec<Cleanup>,
    ) -> ProcedureId {
        let id = ProcedureId::new(self.procedures.len());
        let integer = self.types.scalar(ScalarType::Int(IntegerType::S64));
        let signature = self
            .types
            .procedure(ProcedureType {
                parameters: [].into(),
                results: [integer].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: mode,
                variadic: Variadic::None,
            })
            .unwrap();
        self.procedures.push(Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            body: block(statements, Flow::Terminates),
            cleanups,
        });
        id
    }
    fn finish(self) -> Result<Library, IrError> {
        ProgramBuilder::new(self.types.freeze().unwrap())
            .context(self.context)
            .places(self.places.freeze())
            .procedures(self.procedures)
            .finish_library()
    }
    fn copy(&self, value: i128) -> ValueExpr {
        ValueExpr::Record {
            ty: self.context.record_type,
            fields: vec![ValueExpr::Int(int(value))],
        }
    }
    fn read(&self) -> IntExpr {
        IntExpr::new(
            IntegerType::S64,
            IntExprKind::Value(Box::new(ValueExpr::Load(self.field))),
        )
    }
}
fn number(value: i128) -> Integer {
    Integer::wrapping(IntegerType::S64, value)
}
fn int(value: i128) -> IntExpr {
    IntExpr::constant(number(value))
}
fn block(statements: Vec<Statement>, flow: Flow) -> Block {
    Block {
        statements,
        flow,
    }
}
fn ret(value: IntExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnInt(value),
    })
}
fn call(procedure: ProcedureId) -> Call {
    Call::new(procedure, vec![])
}
fn call_int(procedure: ProcedureId) -> IntExpr {
    IntExpr::new(IntegerType::S64, IntExprKind::Call(call(procedure)))
}
fn assert_result(execution: Execution, value: i128) {
    assert_eq!(
        execution.outcome,
        Outcome::Complete(vec![Value::Int(number(value))])
    );
}

#[test]
fn implicit_calls_share_mutations_and_successful_requests_keep_default_context() {
    let mut fixture = Fixture::new();
    let read = fixture.read();
    let increment = fixture.add(
        ContextMode::Implicit,
        vec![
            Statement::Store(
                fixture.field,
                ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Binary(IntOp::Add, Box::new(read.clone()), Box::new(int(2))),
                )),
            ),
            ret(read.clone()),
        ],
        vec![],
    );
    let caller = fixture.add(
        ContextMode::Implicit,
        vec![Statement::DiscardInt(call_int(increment)), ret(read)],
        vec![],
    );
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_result(vm.execute(caller, vec![]), 12);
    let state = vm.into_state();
    let mut vm = Vm::with_state(&library, NoEffects, Limits::default(), state).unwrap();
    assert_result(vm.execute(caller, vec![]), 14);
}

#[test]
fn unused_context_does_not_consume_the_allocation_budget() {
    let mut fixture = Fixture::new();
    let caller = fixture.add(ContextMode::None, vec![ret(int(3))], vec![]);
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(
        &library,
        NoEffects,
        Limits {
            allocations: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_result(vm.evaluate(&ValueExpr::Int(int(1))), 1);
    assert_result(vm.execute(caller, vec![]), 3);
}

#[test]
fn root_context_value_and_place_access_materialize_default_and_keep_copy_semantics() {
    let mut fixture = Fixture::new();
    let record_type = fixture.context.record_type;
    let field = fixture.field;
    let mutator = fixture.add(
        ContextMode::Implicit,
        vec![
            Statement::Store(field, ValueExpr::Int(int(30))),
            ret(fixture.read()),
        ],
        vec![],
    );
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    let snapshot = vm.evaluate(&ValueExpr::Context {
        ty: record_type,
    });
    assert_result(vm.execute(mutator, vec![]), 30);
    assert_result(vm.evaluate(&ValueExpr::Load(field)), 30);
    assert_eq!(
        snapshot.outcome,
        Outcome::Complete(vec![Value::Record {
            ty: record_type,
            fields: vec![Value::Int(number(10))],
        }])
    );
}

#[test]
fn push_copies_current_record_and_restores_outer_context_after_nested_call() {
    let mut fixture = Fixture::new();
    let read = fixture.read();
    let mutate = fixture.add(
        ContextMode::Implicit,
        vec![
            Statement::Store(fixture.field, ValueExpr::Int(int(90))),
            ret(read.clone()),
        ],
        vec![],
    );
    let caller = fixture.add(
        ContextMode::Implicit,
        vec![
            Statement::PushContext {
                id: PushContextId::new(ProcedureId::new(1), 0),
                value: ValueExpr::Context {
                    ty: fixture.context.record_type,
                },
                body: block(
                    vec![Statement::DiscardInt(call_int(mutate))],
                    Flow::FallsThrough,
                ),
            },
            ret(read),
        ],
        vec![],
    );
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_result(vm.execute(caller, vec![]), 10);
}

#[test]
fn return_captures_pushed_value_before_defer_and_restores_default_storage() {
    let mut fixture = Fixture::new();
    let read = fixture.read();
    let caller = fixture.add(
        ContextMode::Implicit,
        vec![Statement::PushContext {
            id: PushContextId::new(ProcedureId::new(0), 0),
            value: fixture.copy(7),
            body: block(
                vec![Statement::Exit(Exit {
                    cleanups: vec![CleanupId::new(0)],
                    transfer: Transfer::ReturnInt(read.clone()),
                })],
                Flow::Terminates,
            ),
        }],
        vec![Cleanup {
            context: CleanupContext::Push(PushContextId::new(ProcedureId::new(0), 0)),
            body: block(
                vec![Statement::Store(fixture.field, ValueExpr::Int(int(99)))],
                Flow::FallsThrough,
            ),
        }],
    );
    let reader = fixture.add(ContextMode::Implicit, vec![ret(read)], vec![]);
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_result(vm.execute(caller, vec![]), 7);
    assert_result(vm.execute(reader, vec![]), 10);
}

#[test]
fn no_context_procedure_can_establish_explicit_context_for_implicit_callee() {
    let mut fixture = Fixture::new();
    let reader = fixture.add(ContextMode::Implicit, vec![ret(fixture.read())], vec![]);
    let caller = fixture.add(
        ContextMode::None,
        vec![Statement::PushContext {
            id: PushContextId::new(ProcedureId::new(1), 0),
            value: fixture.copy(50),
            body: block(vec![ret(call_int(reader))], Flow::Terminates),
        }],
        vec![],
    );
    let library = fixture.finish().unwrap();
    assert_result(
        Vm::new(&library, NoEffects, Limits::default())
            .unwrap()
            .execute(caller, vec![]),
        50,
    );
}

#[test]
fn no_context_procedure_cannot_inherit_context_implicitly() {
    let mut fixture = Fixture::new();
    let reader = fixture.add(ContextMode::Implicit, vec![ret(fixture.read())], vec![]);
    fixture.add(ContextMode::None, vec![ret(call_int(reader))], vec![]);
    assert!(matches!(fixture.finish(), Err(IrError::MissingContext)));
}

#[test]
fn failed_push_restores_context_and_rolls_back_default_mutation() {
    let mut fixture = Fixture::new();
    let read = fixture.read();
    let failing = fixture.add(
        ContextMode::Implicit,
        vec![
            Statement::Store(fixture.field, ValueExpr::Int(int(33))),
            Statement::PushContext {
                id: PushContextId::new(ProcedureId::new(0), 0),
                value: fixture.copy(7),
                body: block(
                    vec![ret(IntExpr::new(
                        IntegerType::S64,
                        IntExprKind::Binary(IntOp::Divide, Box::new(int(1)), Box::new(int(0))),
                    ))],
                    Flow::Terminates,
                ),
            },
        ],
        vec![],
    );
    let reader = fixture.add(ContextMode::Implicit, vec![ret(read)], vec![]);
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.execute(failing, vec![]).outcome,
        Outcome::Failed(Error::Arithmetic(ArithmeticError::ZeroDivisor))
    ));
    assert_result(vm.execute(reader, vec![]), 10);
}

#[test]
fn loop_break_and_continue_restore_pushed_context_before_next_control_edge() {
    for breaking in [true, false] {
        let mut fixture = Fixture::new();
        let id = ProcedureId::new(0);
        let loop_id = LoopId::new(0);
        let iterator = Local::new(id, 0, ScalarType::Int(IntegerType::S64), &fixture.types)
            .integer(&fixture.types)
            .unwrap();
        let caller = fixture.add(
            ContextMode::Implicit,
            vec![
                Statement::Range(RangeLoop {
                    id: loop_id,
                    iterator,
                    start: int(0),
                    end: int(1),
                    direction: Direction::Forward,
                    body: block(
                        vec![Statement::PushContext {
                            id: PushContextId::new(id, 0),
                            value: fixture.copy(7),
                            body: block(
                                vec![Statement::Exit(Exit {
                                    cleanups: vec![],
                                    transfer: if breaking {
                                        Transfer::Break(loop_id)
                                    } else {
                                        Transfer::Continue(loop_id)
                                    },
                                })],
                                Flow::Terminates,
                            ),
                        }],
                        Flow::Terminates,
                    ),
                }),
                ret(fixture.read()),
            ],
            vec![],
        );
        fixture.procedures[caller.index()].locals = vec![iterator.local()];
        let library = fixture.finish().unwrap();
        assert_result(
            Vm::new(
                &library,
                NoEffects,
                Limits {
                    allocations: 3,
                    ..Limits::default()
                },
            )
            .unwrap()
            .execute(caller, vec![]),
            10,
        );
    }
}

#[test]
fn cleanups_use_lexically_captured_procedure_and_outer_push_contexts() {
    let mut fixture = Fixture::new();
    let procedure = ProcedureId::new(0);
    let outer = PushContextId::new(procedure, 0);
    let inner = PushContextId::new(procedure, 1);
    let read = fixture.read();
    let increment = |amount| {
        block(
            vec![Statement::Store(
                fixture.field,
                ValueExpr::Int(IntExpr::new(
                    IntegerType::S64,
                    IntExprKind::Binary(IntOp::Add, Box::new(read.clone()), Box::new(int(amount))),
                )),
            )],
            Flow::FallsThrough,
        )
    };
    let cleanups = vec![
        Cleanup {
            context: CleanupContext::Procedure,
            body: increment(2),
        },
        Cleanup {
            context: CleanupContext::Push(outer),
            body: increment(3),
        },
    ];
    let caller = fixture.add(
        ContextMode::Implicit,
        vec![Statement::PushContext {
            id: outer,
            value: fixture.copy(7),
            body: block(
                vec![
                    Statement::PushContext {
                        id: inner,
                        value: fixture.copy(50),
                        body: block(
                            vec![
                                Statement::Cleanup(CleanupId::new(0)),
                                Statement::Cleanup(CleanupId::new(1)),
                            ],
                            Flow::FallsThrough,
                        ),
                    },
                    ret(read.clone()),
                ],
                Flow::Terminates,
            ),
        }],
        cleanups,
    );
    let reader = fixture.add(ContextMode::Implicit, vec![ret(read)], vec![]);
    let library = fixture.finish().unwrap();
    let mut vm = Vm::new(&library, NoEffects, Limits::default()).unwrap();
    assert_result(vm.execute(caller, vec![]), 10);
    assert_result(vm.execute(reader, vec![]), 12);
}
