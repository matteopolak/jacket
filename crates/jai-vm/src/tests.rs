use super::*;
use jai_ir::*;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, RecordKind, ScalarType,
    TypeId, TypeRegistry, TypeView,
};
#[path = "address_integer_regressions.rs"]
mod address_integer_regressions;
#[path = "execution_regressions.rs"]
mod execution_regressions;
#[path = "runtime_type_regressions.rs"]
mod runtime_type_regressions;
#[path = "sequence_concat_regressions.rs"]
mod sequence_concat_regressions;
#[path = "sequence_regressions.rs"]
mod sequence_regressions;
#[path = "static_graph_regressions.rs"]
mod static_graph_regressions;
#[path = "static_publication_regressions.rs"]
mod static_publication_regressions;
#[path = "zero_work_regressions.rs"]
mod zero_work_regressions;

struct Fixture {
    types: TypeRegistry,
    procedures: Vec<Procedure>,
    globals: Vec<Global>,
    pending: Option<ProcedureId>,
    compiler: Option<CompilerProcedure>,
    places: Places,
    signatures: std::collections::HashMap<ProcedureId, TypeId>,
}
impl ProcedureProvider for Fixture {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &std::collections::HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        if self.pending == Some(id) {
            return ProcedureAvailability::Pending(Dependency::Procedure(id));
        }
        if id.index() == 99 {
            return self.compiler.map_or(
                ProcedureAvailability::Foreign,
                ProcedureAvailability::Compiler,
            );
        }
        let Some(procedure) = self.procedures.iter().find(|procedure| procedure.id == id) else {
            return ProcedureAvailability::Missing;
        };
        match verify_procedure(
            &self.types,
            procedure,
            &self.signatures,
            &self.globals,
            &self.places,
        ) {
            Ok(checked) => ProcedureAvailability::Ready(checked),
            Err(IrError::Type(jai_types::TypeError::Incomplete(ty))) => {
                ProcedureAvailability::Pending(Dependency::Type(ty))
            }
            Err(error) => ProcedureAvailability::Failed(Error::IrValidation(error.to_string())),
        }
    }
    fn globals(&self) -> &[Global] {
        &self.globals
    }
    fn places(&self) -> Option<&Places> {
        Some(&self.places)
    }
}
fn fixture() -> Fixture {
    Fixture {
        types: TypeRegistry::new(),
        procedures: vec![],
        globals: vec![],
        pending: None,
        compiler: None,
        places: Places::default(),
        signatures: std::collections::HashMap::new(),
    }
}
fn int(value: i128) -> IntExpr {
    typed(IntegerType::S64, value)
}
fn typed(ty: IntegerType, value: i128) -> IntExpr {
    IntExpr::constant(Integer::wrapping(ty, value))
}
fn value(value: i128) -> Value {
    Value::Int(Integer::wrapping(IntegerType::S64, value))
}
fn binary(op: IntOp, a: IntExpr, b: IntExpr) -> IntExpr {
    IntExpr::new(a.ty(), IntExprKind::Binary(op, Box::new(a), Box::new(b)))
}
fn call(id: usize, arguments: Vec<(usize, ValueExpr)>) -> Call {
    Call::new(
        ProcedureId::new(id),
        arguments
            .into_iter()
            .map(|(index, value)| (ParameterId::new(index), value))
            .collect(),
    )
}
fn call_int(id: usize, arguments: Vec<(usize, ValueExpr)>) -> IntExpr {
    IntExpr::new(IntegerType::S64, IntExprKind::Call(call(id, arguments)))
}
fn block(statements: Vec<Statement>) -> Block {
    let flow = match statements.last() {
        Some(Statement::Exit(_)) => Flow::Terminates,
        Some(Statement::Block(block)) => block.flow,
        Some(Statement::If(_, yes, no))
            if yes.flow == Flow::Terminates && no.flow == Flow::Terminates =>
        {
            Flow::Terminates
        }
        Some(Statement::Cases(cases)) => cases.flow,
        _ => Flow::FallsThrough,
    };
    Block {
        statements,
        flow,
    }
}
fn ret(expression: IntExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnInt(expression),
    })
}
fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: jai_types::Variadic::None,
        })
        .unwrap()
}
fn procedure(
    fixture: &mut Fixture,
    index: usize,
    parameters: Vec<Local>,
    locals: Vec<Local>,
    statements: Vec<Statement>,
) -> Procedure {
    let integer = fixture.types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = signature(
        &mut fixture.types,
        parameters.iter().map(|p| p.ty()).collect(),
        vec![integer],
    );
    fixture
        .signatures
        .insert(ProcedureId::new(index), signature);
    Procedure {
        id: ProcedureId::new(index),
        signature,
        parameters,
        locals,
        body: block(statements),
        cleanups: vec![],
    }
}
fn local(fixture: &Fixture, procedure: usize, index: usize) -> IntLocal {
    Local::new(
        ProcedureId::new(procedure),
        index,
        ScalarType::Int(IntegerType::S64),
        &fixture.types,
    )
    .integer(&fixture.types)
    .unwrap()
}
fn run(fixture: &Fixture, index: usize) -> Execution {
    Vm::new(fixture, NoEffects, Limits::default())
        .unwrap()
        .execute(ProcedureId::new(index), vec![])
}

#[test]
fn procedures_execute_before_unrelated_types_are_complete() {
    let mut f = fixture();
    f.types.reserve_record(RecordKind::Struct);
    let p = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![ret(binary(IntOp::Multiply, int(6), int(7)))],
    );
    f.procedures.push(p);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(42)]));
}
#[test]
fn arguments_execute_in_ir_order_before_parameter_reordering() {
    let mut f = fixture();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let g = global.storage().place();
    let gp = IntPlace::try_from_place(g, &f.types).unwrap();
    f.globals.push(global);
    let tick = procedure(
        &mut f,
        1,
        vec![],
        vec![],
        vec![
            Statement::StoreInt(gp, binary(IntOp::Add, IntExpr::load(gp), int(1))),
            ret(IntExpr::load(gp)),
        ],
    );
    let a = local(&f, 2, 0);
    let b = local(&f, 2, 1);
    let pair = procedure(
        &mut f,
        2,
        vec![a.local(), b.local()],
        vec![a.local(), b.local()],
        vec![ret(binary(
            IntOp::Add,
            binary(IntOp::Multiply, IntExpr::load(a.place()), int(10)),
            IntExpr::load(b.place()),
        ))],
    );
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![ret(call_int(
            2,
            vec![
                (1, ValueExpr::Int(call_int(1, vec![]))),
                (0, ValueExpr::Int(call_int(1, vec![]))),
            ],
        ))],
    );
    f.procedures.extend([root, tick, pair]);
    let result = run(&f, 0);
    assert_eq!(result.outcome, Outcome::Complete(vec![value(21)]));
    assert_eq!(result.statistics.calls, 4);
}
#[test]
fn conditional_and_boolean_branches_are_lazy() {
    let mut f = fixture();
    let invalid = || binary(IntOp::Divide, int(1), int(0));
    let expression = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Conditional(Box::new(Conditional {
            condition: BoolExpr::And(
                Box::new(BoolExpr::Constant(false)),
                Box::new(BoolExpr::FromInt(Box::new(invalid()))),
            ),
            then_value: invalid(),
            else_value: int(7),
        })),
    );
    let p = procedure(&mut f, 0, vec![], vec![], vec![ret(expression)]);
    f.procedures.push(p);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(7)]));
}
#[test]
fn cleanup_runs_in_explicit_order_after_return_snapshot() {
    let mut f = fixture();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let gp = IntPlace::try_from_place(global.place(), &f.types).unwrap();
    f.globals.push(global);
    let n = local(&f, 0, 0);
    let mut p = procedure(
        &mut f,
        0,
        vec![],
        vec![n.local()],
        vec![
            Statement::StoreInt(n.place(), int(5)),
            Statement::Exit(Exit {
                cleanups: vec![CleanupId::new(1), CleanupId::new(0)],
                transfer: Transfer::ReturnInt(IntExpr::load(n.place())),
            }),
        ],
    );
    p.cleanups = vec![
        block(vec![Statement::StoreInt(
            gp,
            binary(
                IntOp::Add,
                binary(IntOp::Multiply, IntExpr::load(gp), int(10)),
                int(1),
            ),
        )])
        .into(),
        block(vec![
            Statement::StoreInt(n.place(), int(99)),
            Statement::StoreInt(gp, int(2)),
        ])
        .into(),
    ];
    let getter = procedure(&mut f, 1, vec![], vec![], vec![ret(IntExpr::load(gp))]);
    f.procedures.extend([p, getter]);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(5)])
    );
    assert_eq!(
        vm.execute(ProcedureId::new(1), vec![]).outcome,
        Outcome::Complete(vec![value(21)])
    );
    assert_eq!(vm.memory().allocation_count(), 1);
}
#[test]
fn pending_nested_calls_rollback_global_writes_and_retry_genuinely() {
    let mut f = fixture();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 3)),
        &f.types,
    );
    let gp = IntPlace::try_from_place(global.place(), &f.types).unwrap();
    f.globals.push(global);
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![Statement::StoreInt(gp, int(99)), ret(call_int(1, vec![]))],
    );
    let pending = procedure(&mut f, 1, vec![], vec![], vec![ret(IntExpr::load(gp))]);
    let getter = procedure(&mut f, 2, vec![], vec![], vec![ret(IntExpr::load(gp))]);
    f.procedures.extend([root, pending, getter]);
    f.pending = Some(ProcedureId::new(1));
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Pending(vec![Dependency::Procedure(ProcedureId::new(1))])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
    assert_eq!(
        vm.execute(ProcedureId::new(2), vec![]).outcome,
        Outcome::Complete(vec![value(3)])
    );
    drop(vm);
    f.pending = None;
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(99)]));
}
#[test]
fn inclusive_ranges_stop_before_fixed_width_endpoint_wraps() {
    for direction in [Direction::Forward, Direction::Reverse] {
        let mut f = fixture();
        let iterator = Local::new(
            ProcedureId::new(0),
            0,
            ScalarType::Int(IntegerType::U8),
            &f.types,
        )
        .integer(&f.types)
        .unwrap();
        let n = local(&f, 0, 1);
        let p = procedure(
            &mut f,
            0,
            vec![],
            vec![iterator.local(), n.local()],
            vec![
                Statement::StoreInt(n.place(), int(0)),
                Statement::Range(RangeLoop {
                    id: LoopId::new(0),
                    iterator,
                    start: typed(IntegerType::U8, 254),
                    end: typed(IntegerType::U8, 255),
                    direction,
                    body: block(vec![Statement::StoreInt(
                        n.place(),
                        binary(IntOp::Add, IntExpr::load(n.place()), int(1)),
                    )]),
                }),
                ret(IntExpr::load(n.place())),
            ],
        );
        f.procedures.push(p);
        assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(2)]));
    }
}
#[test]
fn through_case_enters_next_body_without_evaluating_its_condition() {
    let mut f = fixture();
    let n = local(&f, 0, 0);
    let cases = Cases {
        subject: Box::new(Statement::StoreInt(n.place(), int(1))),
        arms: vec![
            CaseArm {
                condition: BoolExpr::Constant(true),
                body: block(vec![Statement::StoreInt(n.place(), int(2))]),
                through: true,
            },
            CaseArm {
                condition: BoolExpr::FromInt(Box::new(binary(IntOp::Divide, int(1), int(0)))),
                body: block(vec![Statement::StoreInt(
                    n.place(),
                    binary(IntOp::Add, IntExpr::load(n.place()), int(3)),
                )]),
                through: false,
            },
        ],
        default: None,
        flow: Flow::FallsThrough,
        exhaustive: false,
    };
    let p = procedure(
        &mut f,
        0,
        vec![],
        vec![n.local()],
        vec![Statement::Cases(cases), ret(IntExpr::load(n.place()))],
    );
    f.procedures.push(p);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(5)]));
}
#[test]
fn fuel_and_stack_limits_terminate_nonterminating_checked_bodies() {
    let mut f = fixture();
    let p = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::While {
                id: LoopId::new(0),
                condition: LoopCondition::Value(BoolExpr::Constant(true)),
                body: block(vec![]),
            },
            ret(int(0)),
        ],
    );
    f.procedures.push(p);
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            fuel: 20,
            ..Limits::default()
        },
    )
    .unwrap();
    let result = vm.execute(ProcedureId::new(0), vec![]);
    assert_eq!(
        result.outcome,
        Outcome::Failed(Error::Limit(LimitKind::Fuel))
    );
    assert_eq!(result.statistics.steps, 20);
    drop(vm);
    f.procedures.clear();
    let p = procedure(&mut f, 0, vec![], vec![], vec![ret(call_int(0, vec![]))]);
    f.procedures.push(p);
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            stack_depth: 4,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Failed(Error::Limit(LimitKind::StackDepth))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}
#[test]
fn integer_arithmetic_uses_fixed_width_bits_and_checked_failures() {
    for ty in [
        IntegerType::S8,
        IntegerType::U8,
        IntegerType::S64,
        IntegerType::U64,
    ] {
        assert_eq!(
            scalar::binary(
                ty,
                IntOp::Add,
                Integer::wrapping(ty, ty.max()),
                Integer::wrapping(ty, 1)
            )
            .unwrap()
            .value(),
            ty.min()
        );
        assert_eq!(
            scalar::binary(
                ty,
                IntOp::Divide,
                Integer::wrapping(ty, 1),
                Integer::wrapping(ty, 0)
            ),
            Err(Error::Arithmetic(ArithmeticError::ZeroDivisor))
        );
        assert_eq!(
            scalar::binary(
                ty,
                IntOp::ShiftLeft,
                Integer::wrapping(ty, 1),
                Integer::wrapping(ty, i128::from(ty.bits()))
            ),
            Err(Error::Arithmetic(ArithmeticError::ShiftCount))
        );
        if ty.signed() {
            assert_eq!(
                scalar::binary_with_check(
                    ty,
                    IntOp::Divide,
                    Integer::wrapping(ty, ty.min()),
                    Integer::wrapping(ty, -1),
                    CheckMode::Enabled,
                ),
                Err(Error::Arithmetic(ArithmeticError::SignedDivisionOverflow))
            );
        }
    }
}
#[test]
fn casts_keep_checked_and_unchecked_results_distinct() {
    let mut f = fixture();
    let narrow = |mode| IntExpr::new(IntegerType::U8, IntExprKind::Cast(mode, Box::new(int(256))));
    let checked = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![ret(IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(
                jai_types::CastMode::Checked,
                Box::new(narrow(jai_types::CastMode::Checked)),
            ),
        ))],
    );
    let unchecked = procedure(
        &mut f,
        1,
        vec![],
        vec![],
        vec![ret(IntExpr::new(
            IntegerType::S64,
            IntExprKind::Cast(
                jai_types::CastMode::Checked,
                Box::new(narrow(jai_types::CastMode::Unchecked)),
            ),
        ))],
    );
    f.procedures.extend([checked, unchecked]);
    assert_eq!(run(&f, 0).outcome, Outcome::Failed(Error::CheckedCast));
    assert_eq!(run(&f, 1).outcome, Outcome::Complete(vec![value(0)]));
}
#[derive(Default)]
struct RecordingEffects {
    pending: bool,
    staged: Vec<CompilerRequest>,
    committed: Vec<CompilerRequest>,
    finishes: Vec<bool>,
}
impl CompilerEffects for RecordingEffects {
    fn begin(&mut self) {
        self.staged.clear();
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        self.staged.push(request);
        if self.pending {
            EffectOutcome::Pending(EffectKey(7))
        } else {
            EffectOutcome::Ready(CompilerResponse::Unit)
        }
    }
    fn finish(&mut self, commit: bool) -> Result<(), Error> {
        self.finishes.push(commit);
        if commit {
            self.committed.append(&mut self.staged);
        } else {
            self.staged.clear();
        }
        Ok(())
    }
}
#[test]
fn compiler_effects_are_typed_and_transactional_and_foreign_calls_fail() {
    let mut f = fixture();
    let string = f.types.string();
    let signature = signature(&mut f.types, vec![string], vec![]);
    f.signatures.insert(ProcedureId::new(99), signature);
    f.compiler = Some(CompilerProcedure {
        signature,
        intrinsic: CompilerIntrinsic::Message(MessageLevel::Info),
    });
    let mut vm = Vm::new(
        &f,
        RecordingEffects {
            pending: true,
            ..RecordingEffects::default()
        },
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(99), vec![Value::String(b"hello".to_vec())])
            .outcome,
        Outcome::Pending(vec![Dependency::Effect(EffectKey(7))])
    );
    assert!(vm.effects().committed.is_empty());
    vm.effects_mut().pending = false;
    assert_eq!(
        vm.execute(ProcedureId::new(99), vec![Value::String(b"hello".to_vec())])
            .outcome,
        Outcome::Complete(vec![])
    );
    assert_eq!(
        vm.effects().committed,
        vec![CompilerRequest::Message {
            level: MessageLevel::Info,
            text: "hello".into()
        }]
    );
    assert_eq!(
        vm.execute(ProcedureId::new(99), vec![Value::String(vec![255])])
            .outcome,
        Outcome::Failed(Error::InvalidIr(
            "compiler intrinsic requires a UTF-8 string"
        ))
    );
    drop(vm);
    f.compiler = None;
    assert_eq!(
        run(&f, 99).outcome,
        Outcome::Failed(Error::UnsupportedForeignProcedure(ProcedureId::new(99)))
    );
}

#[test]
fn aggregate_storage_field_initializers_and_multiresults_execute() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = f.types.scalar(ScalarType::Bool);
    let record = f.types.reserve_record(RecordKind::Struct);
    f.types.define_record(record, [integer, boolean]).unwrap();
    let n = Local::new_typed(ProcedureId::new(0), 0, record, &f.types).unwrap();
    let first = f.types.field(record, 0).unwrap().id;
    let second = f.types.field(record, 1).unwrap().id;
    let mut places = PlaceRegistry::new();
    let field = places.field(n.place(), first, &f.types).unwrap();
    let field = IntPlace::try_from_place(field, &f.types).unwrap();
    f.places = places.freeze();
    let callee_signature = signature(&mut f.types, vec![], vec![record, boolean]);
    f.signatures.insert(ProcedureId::new(1), callee_signature);
    let callee = Procedure {
        id: ProcedureId::new(1),
        signature: callee_signature,
        parameters: vec![],
        locals: vec![],
        body: block(vec![Statement::Exit(Exit {
            cleanups: vec![],
            transfer: Transfer::ReturnValues(vec![
                ValueExpr::RecordBuild {
                    ty: record,
                    initializers: vec![
                        (second, ValueExpr::Bool(BoolExpr::Constant(true))),
                        (first, ValueExpr::Int(int(42))),
                    ],
                },
                ValueExpr::Bool(BoolExpr::Constant(true)),
            ]),
        })]),
        cleanups: vec![],
    };
    let root = procedure(
        &mut f,
        0,
        vec![],
        vec![n],
        vec![
            Statement::CallResults {
                call: call(1, vec![]),
                destinations: vec![Some(n.place()), None],
            },
            Statement::StoreInt(field, binary(IntOp::Add, IntExpr::load(field), int(1))),
            ret(IntExpr::load(field)),
        ],
    );
    f.procedures.extend([root, callee]);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(43)]));
}
#[test]
fn reachable_incomplete_types_become_pending_and_unused_globals_stay_lazy() {
    let mut f = fixture();
    let pending = f.types.reserve_record(RecordKind::Struct);
    f.globals.push(Global::new(
        0,
        GlobalInitializer::Value(ConstantValue {
            ty: pending,
            kind: ConstantKind::Zero,
        }),
        &f.types,
    ));
    let scalar = procedure(&mut f, 0, vec![], vec![], vec![ret(int(9))]);
    f.procedures.push(scalar);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(9)]));
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Zero(pending)).outcome,
        Outcome::Pending(vec![Dependency::Type(pending)])
    );
    assert_eq!(
        vm.evaluate(&ValueExpr::Load(f.globals[0].place())).outcome,
        Outcome::Pending(vec![Dependency::Type(pending)])
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}
#[test]
fn aggregate_zero_fails_before_large_host_allocation() {
    let mut f = fixture();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::U64));
    let enormous = f.types.fixed_array(integer, u64::MAX).unwrap();
    let mut vm = Vm::new(
        &f,
        NoEffects,
        Limits {
            value_cells: 10,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Zero(enormous)).outcome,
        Outcome::Failed(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(vm.memory().allocation_count(), 0);
}
#[test]
fn named_outer_loop_transfer_runs_cleanup_once() {
    let mut f = fixture();
    let n = local(&f, 0, 0);
    let mut root = procedure(
        &mut f,
        0,
        vec![],
        vec![n.local()],
        vec![
            Statement::StoreInt(n.place(), int(0)),
            Statement::While {
                id: LoopId::new(0),
                condition: LoopCondition::Value(BoolExpr::Constant(true)),
                body: block(vec![Statement::While {
                    id: LoopId::new(1),
                    condition: LoopCondition::Value(BoolExpr::Constant(true)),
                    body: block(vec![Statement::Exit(Exit {
                        cleanups: vec![CleanupId::new(0)],
                        transfer: Transfer::Break(LoopId::new(0)),
                    })]),
                }]),
            },
            ret(IntExpr::load(n.place())),
        ],
    );
    root.cleanups = vec![
        block(vec![Statement::StoreInt(
            n.place(),
            binary(IntOp::Add, IntExpr::load(n.place()), int(1)),
        )])
        .into(),
    ];
    f.procedures.push(root);
    assert_eq!(run(&f, 0).outcome, Outcome::Complete(vec![value(1)]));
}
#[test]
fn uninitialized_reads_and_cross_procedure_places_are_explicit_errors() {
    let mut f = fixture();
    let n = local(&f, 0, 0);
    let p = procedure(
        &mut f,
        0,
        vec![],
        vec![n.local()],
        vec![ret(IntExpr::load(n.place()))],
    );
    f.procedures.push(p);
    assert_eq!(run(&f, 0).outcome, Outcome::Failed(Error::Uninitialized));
    let foreign = local(&f, 1, 0);
    let p = procedure(
        &mut f,
        2,
        vec![],
        vec![],
        vec![ret(IntExpr::load(foreign.place()))],
    );
    f.procedures.push(p);
    assert!(matches!(
        run(&f, 2).outcome,
        Outcome::Failed(Error::IrValidation(_))
    ));
}
#[test]
fn pending_type_does_not_hide_foreign_type_registry_identity() {
    let mut f = fixture();
    let foreign = TypeRegistry::new();
    let ty = foreign.scalar(ScalarType::Bool);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.evaluate(&ValueExpr::Zero(ty)).outcome,
        Outcome::Failed(Error::Type(jai_types::TypeError::ForeignType(_)))
    ));
    drop(vm);
    let incomplete = f.types.reserve_record(RecordKind::Struct);
    assert_eq!(
        Vm::new(&f, NoEffects, Limits::default())
            .unwrap()
            .evaluate(&ValueExpr::Zero(incomplete))
            .outcome,
        Outcome::Pending(vec![Dependency::Type(incomplete)])
    );
}

#[test]
fn borrowed_effect_handlers_preserve_the_same_transaction_protocol() {
    let mut f = fixture();
    let string = f.types.string();
    let signature = signature(&mut f.types, vec![string], vec![]);
    f.signatures.insert(ProcedureId::new(99), signature);
    f.compiler = Some(CompilerProcedure {
        signature,
        intrinsic: CompilerIntrinsic::Message(MessageLevel::Info),
    });
    let mut effects = RecordingEffects::default();
    {
        let borrowed: &mut dyn CompilerEffects = &mut effects;
        let mut vm = Vm::new(&f, borrowed, Limits::default()).unwrap();
        assert_eq!(
            vm.execute(
                ProcedureId::new(99),
                vec![Value::String(b"borrowed".to_vec())]
            )
            .outcome,
            Outcome::Complete(vec![])
        );
    }
    assert_eq!(effects.finishes, vec![true]);
    assert_eq!(effects.committed.len(), 1);
}

#[test]
fn source_workspace_adapters_preserve_parameter_order_and_signed_sentinel() {
    let mut f = fixture();
    let string = f.types.string();
    let integer = f.types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = signature(&mut f.types, vec![string, integer], vec![]);
    let current = WorkspaceId::from_raw(7).unwrap();
    f.signatures.insert(ProcedureId::new(99), signature);
    f.compiler = Some(CompilerProcedure {
        signature,
        intrinsic: CompilerIntrinsic::SourceAddString {
            current_workspace: current,
        },
    });
    let mut vm = Vm::new(&f, RecordingEffects::default(), Limits::default()).unwrap();
    assert_eq!(
        vm.execute(
            ProcedureId::new(99),
            vec![Value::String(b"x :: 42;".to_vec()), value(-1)]
        )
        .outcome,
        Outcome::Complete(vec![])
    );
    assert_eq!(
        vm.effects().committed,
        vec![CompilerRequest::AddSource {
            workspace: current,
            source: "x :: 42;".into()
        }]
    );
    assert!(matches!(
        vm.execute(
            ProcedureId::new(99),
            vec![Value::String(b"bad".to_vec()), value(-2)]
        )
        .outcome,
        Outcome::Failed(Error::InvalidIr(_))
    ));
}
#[test]
fn source_error_reports_stop_execution_instead_of_inert_success() {
    let mut f = fixture();
    let string = f.types.string();
    let signature = signature(&mut f.types, vec![string], vec![]);
    f.signatures.insert(ProcedureId::new(99), signature);
    f.compiler = Some(CompilerProcedure {
        signature,
        intrinsic: CompilerIntrinsic::SourceReport {
            level: MessageLevel::Error,
        },
    });
    let mut vm = Vm::new(&f, RecordingEffects::default(), Limits::default()).unwrap();
    assert_eq!(
        vm.execute(
            ProcedureId::new(99),
            vec![Value::String(b"build is invalid".to_vec())]
        )
        .outcome,
        Outcome::Failed(Error::CompilerReported("build is invalid".into()))
    );
    assert!(vm.effects().committed.is_empty());
    assert_eq!(vm.effects().finishes, vec![false]);
}

#[test]
fn vm_state_survives_provider_borrow_changes_and_rejects_foreign_registry() {
    let mut f = fixture();
    let global = Global::new(
        0,
        GlobalInitializer::Int(Integer::wrapping(IntegerType::S64, 0)),
        &f.types,
    );
    let gp = IntPlace::try_from_place(global.place(), &f.types).unwrap();
    f.globals.push(global);
    let increment = procedure(
        &mut f,
        0,
        vec![],
        vec![],
        vec![
            Statement::StoreInt(gp, binary(IntOp::Add, IntExpr::load(gp), int(1))),
            ret(IntExpr::load(gp)),
        ],
    );
    f.procedures.push(increment);
    let mut vm = Vm::new(&f, NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(1)])
    );
    let state = vm.into_state();
    let unused = Global::new(1, GlobalInitializer::Bool(true), &f.types);
    f.globals.push(unused);
    let mut vm = Vm::with_state(&f, NoEffects, Limits::default(), state).unwrap();
    assert_eq!(
        vm.execute(ProcedureId::new(0), vec![]).outcome,
        Outcome::Complete(vec![value(2)])
    );
    let state = vm.into_state();
    let other = fixture();
    assert!(matches!(
        Vm::with_state(&other, NoEffects, Limits::default(), state),
        Err(Error::InvalidIr(
            "VM state belongs to another type registry"
        ))
    ));
}
