//! Checked expression captures through the actual VM and O0/O2 native generator.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_ir::*;
use jai_types::*;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn integer(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}
fn value(value: i128) -> ValueExpr {
    ValueExpr::Int(integer(value))
}
fn as_int(value: ValueExpr) -> IntExpr {
    IntExpr::new(IntegerType::S64, IntExprKind::Value(Box::new(value)))
}
fn binary(op: IntOp, left: IntExpr, right: IntExpr) -> IntExpr {
    IntExpr::new(
        IntegerType::S64,
        IntExprKind::Binary(op, Box::new(left), Box::new(right)),
    )
}
fn result(value: ValueExpr) -> Statement {
    Statement::Exit(Exit {
        cleanups: vec![],
        transfer: Transfer::ReturnValues(vec![value]),
    })
}
fn signature(types: &mut TypeRegistry, parameters: &[TypeId], result: TypeId) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: vec![result].into_boxed_slice(),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}
fn bound(binding: ExpressionBindingId, ty: TypeId) -> ValueExpr {
    ValueExpr::Bound { binding, ty }
}
fn bind(bindings: Vec<(ExpressionBindingId, ValueExpr)>, body: ValueExpr, ty: TypeId) -> ValueExpr {
    ValueExpr::Bind {
        bindings,
        body: Box::new(body),
        ty,
    }
}
fn procedure(
    id: ProcedureId,
    signature: TypeId,
    parameters: Vec<Local>,
    statements: Vec<Statement>,
) -> Procedure {
    Procedure {
        id,
        signature,
        locals: parameters.clone(),
        parameters,
        cleanups: vec![],
        body: Block {
            statements,
            flow: Flow::Terminates,
        },
    }
}
struct Fixture {
    types: TypeRegistry,
    int: TypeId,
    counter: Global,
    counter_place: IntPlace,
    tick: Procedure,
    main: ProcedureId,
    main_signature: TypeId,
}
impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let counter = Global::new(
            0,
            GlobalInitializer::Int(Integer::checked(IntegerType::S64, 0).unwrap()),
            &types,
        );
        let counter_place = IntPlace::try_from_place(counter.place(), &types).unwrap();
        let tick_id = ProcedureId::new(0);
        let main = ProcedureId::new(1);
        let parameter = Local::new_typed(tick_id, 0, int, &types).unwrap();
        let parameter_place = IntPlace::try_from_place(parameter.place(), &types).unwrap();
        let tick_signature = signature(&mut types, &[int], int);
        let main_signature = signature(&mut types, &[], int);
        let tick = procedure(
            tick_id,
            tick_signature,
            vec![parameter],
            vec![
                Statement::StoreInt(
                    counter_place,
                    binary(
                        IntOp::Add,
                        binary(IntOp::Multiply, IntExpr::load(counter_place), integer(10)),
                        IntExpr::load(parameter_place),
                    ),
                ),
                result(ValueExpr::Int(IntExpr::load(parameter_place))),
            ],
        );
        Self {
            types,
            int,
            counter,
            counter_place,
            tick,
            main,
            main_signature,
        }
    }
    fn binding(&self, index: usize) -> ExpressionBindingId {
        ExpressionBindingId::new(self.main, index)
    }
    fn tick(&self, n: i128) -> ValueExpr {
        ValueExpr::Call {
            call: Call::new(self.tick.id, vec![(ParameterId::new(0), value(n))]),
            ty: self.int,
        }
    }
    fn finish(
        self,
        main_value: ValueExpr,
        extra: Vec<Procedure>,
        prototypes: Vec<ProcedurePrototype>,
    ) -> Program {
        let mut procedures = vec![
            self.tick,
            procedure(
                self.main,
                self.main_signature,
                vec![],
                vec![result(main_value)],
            ),
        ];
        procedures.extend(extra);
        ProgramBuilder::new(self.types.freeze().unwrap())
            .procedures(procedures)
            .prototypes(prototypes)
            .globals(vec![self.counter])
            .finish(EntryPoint::Int(self.main))
            .unwrap()
    }
}
fn execute(program: &Program, expected: i32, compare_vm: bool) {
    if compare_vm {
        let execution = jai_vm::execute(program, jai_vm::Limits::default());
        assert!(
            matches!(execution.outcome,jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==i128::from(expected))),
            "{:?}",
            execution.outcome
        );
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Scratch(PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-expression-bindings-ir-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target =
            jai_codegen::target::NativeTarget::select(&jai_codegen::target::TargetOptions {
                optimization: jai_codegen::optimization::Optimization {
                    bitcode: optimization,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, program, &target).unwrap();
        module.verify().unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let linked = native_tools::clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut process = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = process.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{optimization:?}");
                break;
            }
            if Instant::now() >= deadline {
                process.kill().unwrap();
                process.wait().unwrap();
                panic!("generated capture program timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
#[test]
fn sequential_producers_can_read_earlier_captures_without_repeating_effects() {
    let fixture = Fixture::new();
    let first = fixture.binding(0);
    let second = fixture.binding(1);
    let body = ValueExpr::Int(binary(
        IntOp::Add,
        binary(
            IntOp::Add,
            as_int(bound(first, fixture.int)),
            as_int(bound(second, fixture.int)),
        ),
        IntExpr::load(fixture.counter_place),
    ));
    let expression = bind(
        vec![
            (first, fixture.tick(1)),
            (
                second,
                ValueExpr::Int(binary(
                    IntOp::Add,
                    as_int(bound(first, fixture.int)),
                    as_int(fixture.tick(2)),
                )),
            ),
        ],
        body,
        fixture.int,
    );
    execute(&fixture.finish(expression, vec![], vec![]), 16, true);
}
#[test]
fn nested_scopes_preserve_outer_captures_and_selected_arms_stay_lazy() {
    let fixture = Fixture::new();
    let outer = fixture.binding(0);
    let inner = fixture.binding(1);
    let skipped = fixture.binding(2);
    let nested = bind(
        vec![(inner, fixture.tick(2))],
        ValueExpr::Int(binary(
            IntOp::Add,
            as_int(bound(outer, fixture.int)),
            as_int(bound(inner, fixture.int)),
        )),
        fixture.int,
    );
    let chosen = ValueExpr::Conditional {
        ty: fixture.int,
        expression: Box::new(Conditional {
            condition: BoolExpr::Constant(false),
            then_value: bind(
                vec![(skipped, fixture.tick(9))],
                bound(skipped, fixture.int),
                fixture.int,
            ),
            else_value: nested,
        }),
    };
    let expression = bind(
        vec![(outer, fixture.tick(1))],
        ValueExpr::Int(binary(
            IntOp::Add,
            binary(
                IntOp::Add,
                as_int(chosen),
                as_int(bound(outer, fixture.int)),
            ),
            IntExpr::load(fixture.counter_place),
        )),
        fixture.int,
    );
    execute(&fixture.finish(expression, vec![], vec![]), 16, true);
}
#[test]
fn indirect_callee_is_captured_before_ordered_argument_effects() {
    let mut fixture = Fixture::new();
    let target_id = ProcedureId::new(2);
    let target_signature = signature(&mut fixture.types, &[fixture.int, fixture.int], fixture.int);
    let first_parameter = Local::new_typed(target_id, 0, fixture.int, &fixture.types).unwrap();
    let second_parameter = Local::new_typed(target_id, 1, fixture.int, &fixture.types).unwrap();
    let target = procedure(
        target_id,
        target_signature,
        vec![first_parameter, second_parameter],
        vec![result(ValueExpr::Int(IntExpr::load(fixture.counter_place)))],
    );
    let callee_capture = fixture.binding(0);
    let argument_capture = fixture.binding(1);
    let callee = bind(
        vec![(callee_capture, fixture.tick(1))],
        ValueExpr::ProcedureValue {
            procedure: target_id,
            ty: target_signature,
        },
        target_signature,
    );
    let expression = ValueExpr::IndirectCall {
        inline_hint: InlineHint::Automatic,
        callee: Box::new(callee),
        arguments: vec![
            (
                ParameterId::new(0),
                bind(
                    vec![(argument_capture, fixture.tick(2))],
                    bound(argument_capture, fixture.int),
                    fixture.int,
                ),
            ),
            (ParameterId::new(1), fixture.tick(3)),
        ],
        ty: fixture.int,
    };
    execute(&fixture.finish(expression, vec![target], vec![]), 123, true);
}
#[test]
fn captured_phase_predicate_prunes_compiler_arm_after_producer_effects() {
    let fixture = Fixture::new();
    let boolean = fixture.types.scalar(ScalarType::Bool);
    let captured = fixture.binding(0);
    let compiler = ProcedureId::new(7);
    let producer = ValueExpr::Bool(BoolExpr::And(
        Box::new(BoolExpr::CompareInts(
            Relation::Equal,
            Box::new(as_int(fixture.tick(1))),
            Box::new(integer(1)),
        )),
        Box::new(BoolExpr::CompileTime),
    ));
    let body = ValueExpr::Conditional {
        ty: fixture.int,
        expression: Box::new(Conditional {
            condition: BoolExpr::Value(Box::new(bound(captured, boolean))),
            then_value: ValueExpr::Call {
                call: Call::new(compiler, vec![]),
                ty: fixture.int,
            },
            else_value: ValueExpr::Int(binary(
                IntOp::Multiply,
                IntExpr::load(fixture.counter_place),
                integer(42),
            )),
        }),
    };
    let expression = bind(vec![(captured, producer)], body, fixture.int);
    let prototype = ProcedurePrototype {
        id: compiler,
        signature: fixture.main_signature,
        origin: PrototypeOrigin::Compiler,
    };
    execute(
        &fixture.finish(expression, vec![], vec![prototype]),
        42,
        false,
    );
}

#[test]
fn repeated_place_projection_is_rescanned_under_each_binding_scope() {
    let mut fixture = Fixture::new();
    let boolean = fixture.types.scalar(ScalarType::Bool);
    let array_type = fixture.types.fixed_array(fixture.int, 1).unwrap();
    let array = Local::new_typed(fixture.main, 0, array_type, &fixture.types).unwrap();
    let capture = fixture.binding(0);
    let compiler = ProcedureId::new(7);
    let index = IntExpr::new(
        IntegerType::S64,
        IntExprKind::Conditional(Box::new(Conditional {
            condition: BoolExpr::Value(Box::new(bound(capture, boolean))),
            then_value: as_int(ValueExpr::Call {
                call: Call::new(compiler, vec![]),
                ty: fixture.int,
            }),
            else_value: integer(0),
        })),
    );
    let mut places = PlaceRegistry::new();
    let projected = places.index(array.place(), index, &fixture.types).unwrap();
    let statements = vec![
        Statement::Store(array.place(), ValueExpr::Zero(array_type)),
        Statement::DiscardValue(bind(
            vec![(capture, ValueExpr::Bool(BoolExpr::Constant(false)))],
            ValueExpr::Load(projected),
            fixture.int,
        )),
        Statement::DiscardValue(bind(
            vec![(capture, ValueExpr::Bool(BoolExpr::Constant(true)))],
            ValueExpr::Load(projected),
            fixture.int,
        )),
        result(value(42)),
    ];
    let mut main = procedure(fixture.main, fixture.main_signature, vec![], statements);
    main.locals.push(array);
    let program = ProgramBuilder::new(fixture.types.freeze().unwrap())
        .procedures(vec![fixture.tick, main])
        .globals(vec![fixture.counter])
        .places(places.freeze())
        .prototypes(vec![ProcedurePrototype {
            id: compiler,
            signature: fixture.main_signature,
            origin: PrototypeOrigin::Compiler,
        }])
        .finish(EntryPoint::Int(fixture.main))
        .unwrap();
    let error = jai_codegen::native_reachability::Reachable::executable(
        program.library(),
        EntryPoint::Int(fixture.main),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        jai_codegen::native_reachability::Error::CompilerRequest { chain }
            if chain == [fixture.main, compiler]
    ));
}
