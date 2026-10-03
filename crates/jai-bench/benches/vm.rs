use divan::{Bencher, counter::ItemsCount};
#[path = "vm/sequence_indices.rs"]
mod sequence_indices;
use jai_ir::*;
use jai_types::*;
use jai_vm::{Error, LimitKind, Limits, NoEffects, Outcome, Value, Vm};

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

fn checked_execute(
    vm: &mut Vm<'_, Program, NoEffects>,
    entry: ProcedureId,
    expected: Integer,
    expected_live_allocations: usize,
) -> jai_vm::Execution {
    let execution = completed_execute(vm, entry, expected);
    assert_eq!(vm.memory().allocation_count(), expected_live_allocations);
    execution
}

fn completed_execute(
    vm: &mut Vm<'_, Program, NoEffects>,
    entry: ProcedureId,
    expected: Integer,
) -> jai_vm::Execution {
    let execution = vm.execute(entry, vec![]);
    assert!(
        matches!(&execution.outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if *value == expected)),
        "successful VM benchmark failed: {execution:?}"
    );
    execution
}

fn source_program(source: String) -> Program {
    let path = std::path::Path::new("/bench/main.jai");
    let mut inputs = jai_modules::SourceOverlay::default();
    inputs.insert(path, source.into_bytes()).unwrap();
    let graph = jai_modules::ModuleGraph::load_with_provider(
        path,
        jai_modules::GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut NoEffects,
    )
    .unwrap()
}

#[divan::bench(args = [4, 64])]
fn any_box_borrow_reuse_execute(bencher: Bencher, count: usize) {
    any_box_benchmark(bencher, count, Limits::default());
}

#[divan::bench]
fn any_box_borrow_1024_reuse_fuel_32m_execute(bencher: Bencher) {
    any_box_benchmark(
        bencher,
        1024,
        Limits {
            fuel: 32_000_000,
            ..Limits::default()
        },
    );
}

fn any_program(count: usize) -> Program {
    source_program(format!(
        "sum::(args:[]Any)->int {{result:=0; for i:0..args.count-1 result+=(cast(*int)args[i].value_pointer).*; return result;}} main::()->int {{payloads:[{count}]int; boxes:[{count}]Any; for i:0..{count}-1 {{payloads[i]=i+1; boxes[i]=payloads[i];}} return sum(boxes);}}"
    ))
}

fn any_box_benchmark(bencher: Bencher, count: usize, limits: Limits) {
    let program = any_program(count);
    let mut vm = Vm::new(&program, NoEffects, limits).unwrap();
    let entry = match program.entry() {
        EntryPoint::Int(id) => id,
        _ => panic!("expected integer entry"),
    };
    let expected =
        Integer::checked(IntegerType::S64, count as i128 * (count as i128 + 1) / 2).unwrap();
    completed_execute(&mut vm, entry, expected);
    // Canonical Type descriptors are persistent VM backing allocations.
    let persistent_allocations = vm.memory().allocation_count();
    assert_eq!(persistent_allocations, 2);
    let mut result = checked_execute(&mut vm, entry, expected, persistent_allocations);
    for _ in 1..40 {
        result = checked_execute(&mut vm, entry, expected, persistent_allocations);
    }
    report_workload(
        "any_box_borrow",
        count,
        limits.fuel,
        &result,
        "complete",
        persistent_allocations,
        40,
    );
    bencher
        .counter(ItemsCount::new(count))
        .bench_local(|| checked_execute(&mut vm, entry, expected, persistent_allocations));
}

#[divan::bench(args = [4, 64])]
fn any_box_borrow_fresh_execute(bencher: Bencher, count: usize) {
    any_box_fresh_benchmark(bencher, count, Limits::default());
}

#[divan::bench]
fn any_box_borrow_1024_fresh_fuel_32m_execute(bencher: Bencher) {
    any_box_fresh_benchmark(
        bencher,
        1024,
        Limits {
            fuel: 32_000_000,
            ..Limits::default()
        },
    );
}

fn any_box_fresh_benchmark(bencher: Bencher, count: usize, limits: Limits) {
    let program = any_program(count);
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry")
    };
    let expected =
        Integer::checked(IntegerType::S64, count as i128 * (count as i128 + 1) / 2).unwrap();
    let execute = || {
        let mut vm = Vm::new(&program, NoEffects, limits).unwrap();
        checked_execute(&mut vm, entry, expected, 2)
    };
    let result = execute();
    report_workload(
        "any_box_borrow_fresh",
        count,
        limits.fuel,
        &result,
        "complete",
        2,
        0,
    );
    bencher.counter(ItemsCount::new(count)).bench_local(execute);
}

#[divan::bench]
fn any_box_borrow_1024_default_fuel_rejection(bencher: Bencher) {
    let program = any_program(1024);
    let mut vm = Vm::new(&program, NoEffects, Limits::default()).unwrap();
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry")
    };
    let mut execute = || {
        let result = vm.execute(entry, vec![]);
        assert!(matches!(
            result.outcome,
            Outcome::Failed(Error::Limit(LimitKind::Fuel))
        ));
        assert_eq!(
            vm.memory().allocation_count(),
            0,
            "failed request must roll back allocations"
        );
        result
    };
    let result = execute();
    report_workload(
        "any_box_borrow_default_fuel_rejection",
        1024,
        Limits::default().fuel,
        &result,
        "fuel_limit",
        0,
        0,
    );
    bencher
        .counter(ItemsCount::new(1usize))
        .bench_local(execute);
}

fn report_workload(
    name: &str,
    count: usize,
    fuel: u64,
    result: &jai_vm::Execution,
    expected: &str,
    live_allocations: usize,
    warmup_executions: usize,
) {
    eprintln!(
        "JAI_BENCH_WORKLOAD {{\"name\":\"{name}\",\"count\":{count},\"fuel\":{fuel},\"steps\":{},\"calls\":{},\"expected_outcome\":\"{expected}\",\"live_allocations\":{live_allocations},\"warmup_executions\":{warmup_executions}}}",
        result.statistics.steps, result.statistics.calls
    );
}

#[divan::bench(args = [4, 64, 1024])]
fn checked_call_execute(bencher: Bencher, count: usize) {
    let program = source_program(format!(
        "step::(value:int)->int {{return value+1;}} main::()->int {{result:=0; for i:1..{count} result=step(result); return result;}}"
    ));
    let mut vm = Vm::new(&program, NoEffects, Limits::default()).unwrap();
    let entry = match program.entry() {
        EntryPoint::Int(id) => id,
        _ => panic!("expected integer entry"),
    };
    let expected = Integer::checked(IntegerType::S64, count as i128).unwrap();
    // The implicit source context is intentionally persistent across successful requests.
    let result = checked_execute(&mut vm, entry, expected, 1);
    assert_eq!(result.statistics.calls, count as u64 + 1);
    report_workload(
        "checked_call",
        count,
        Limits::default().fuel,
        &result,
        "complete",
        1,
        1,
    );
    bencher.counter(ItemsCount::new(count)).bench_local(|| {
        let result = checked_execute(&mut vm, entry, expected, 1);
        assert_eq!(result.statistics.calls, count as u64 + 1);
        result
    });
}

fn integer(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}

fn range_program(iterations: usize) -> Program {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let id = ProcedureId::new(0);
    let sum = Local::new_typed(id, 0, int, &types).unwrap();
    let iterator = Local::new_typed(id, 1, int, &types).unwrap();
    let sum_place = sum.integer(&types).unwrap().place();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let body = Block {
        flow: Flow::Terminates,
        statements: vec![
            Statement::StoreInt(sum_place, integer(0)),
            Statement::Range(RangeLoop {
                id: LoopId::new(0),
                iterator: iterator.integer(&types).unwrap(),
                start: integer(1),
                end: integer(iterations as i128),
                direction: Direction::Forward,
                body: Block {
                    flow: Flow::FallsThrough,
                    statements: vec![Statement::StoreInt(
                        sum_place,
                        IntExpr::new(
                            IntegerType::S64,
                            IntExprKind::Binary(
                                IntOp::Add,
                                Box::new(IntExpr::load(sum_place)),
                                Box::new(IntExpr::load(iterator.integer(&types).unwrap().place())),
                            ),
                        ),
                    )],
                },
            }),
            Statement::Exit(Exit {
                cleanups: vec![],
                transfer: Transfer::ReturnInt(IntExpr::load(sum_place)),
            }),
        ],
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![sum, iterator],
            body,
            cleanups: vec![],
        }])
        .finish(EntryPoint::Int(id))
        .unwrap()
}

fn assert_sum(vm: &mut Vm<'_, Program, NoEffects>, iterations: usize) -> jai_vm::Execution {
    let expected = iterations as i128 * (iterations as i128 + 1) / 2;
    checked_execute(
        vm,
        ProcedureId::new(0),
        Integer::checked(IntegerType::S64, expected).unwrap(),
        0,
    )
}

#[divan::bench(args = [4, 64, 1024])]
fn range_execute_cold(bencher: Bencher, iterations: usize) {
    let program = range_program(iterations);
    let expected = Integer::checked(
        IntegerType::S64,
        iterations as i128 * (iterations as i128 + 1) / 2,
    )
    .unwrap();
    let result = assert_sum(
        &mut Vm::new(&program, NoEffects, Limits::default()).unwrap(),
        iterations,
    );
    report_workload(
        "range_cold",
        iterations,
        Limits::default().fuel,
        &result,
        "complete",
        0,
        0,
    );
    bencher
        .counter(ItemsCount::new(iterations))
        .bench_local(|| {
            let mut vm = Vm::new(&program, NoEffects, Limits::default()).unwrap();
            checked_execute(&mut vm, ProcedureId::new(0), expected, 0)
        });
}

#[divan::bench(args = [4, 64, 1024])]
fn range_execute_reuse(bencher: Bencher, iterations: usize) {
    let program = range_program(iterations);
    let expected = Integer::checked(
        IntegerType::S64,
        iterations as i128 * (iterations as i128 + 1) / 2,
    )
    .unwrap();
    let mut vm = Vm::new(&program, NoEffects, Limits::default()).unwrap();
    assert_sum(&mut vm, iterations);
    let result = assert_sum(&mut vm, iterations);
    report_workload(
        "range_reuse",
        iterations,
        Limits::default().fuel,
        &result,
        "complete",
        0,
        2,
    );
    bencher
        .counter(ItemsCount::new(iterations))
        .bench_local(|| checked_execute(&mut vm, ProcedureId::new(0), expected, 0));
}

fn record_program(fields: usize) -> Program {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![int; fields]).unwrap();
    let last = types.field(record, fields - 1).unwrap().id;
    let id = ProcedureId::new(0);
    let original = Local::new_typed(id, 0, record, &types).unwrap();
    let copy = Local::new_typed(id, 1, record, &types).unwrap();
    let mut places = PlaceRegistry::new();
    let original_last = places.field(original.place(), last, &types).unwrap();
    let copy_last = places.field(copy.place(), last, &types).unwrap();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![original, copy],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Store(
                    original.place(),
                    ValueExpr::Record {
                        ty: record,
                        fields: (0..fields)
                            .map(|index| ValueExpr::Int(integer(index as i128)))
                            .collect(),
                    },
                ),
                Statement::Store(copy.place(), ValueExpr::Load(original.place())),
                Statement::Store(original_last, ValueExpr::Int(integer(-1))),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnInt(IntExpr::load(
                        IntPlace::try_from_place(copy_last, &types).unwrap(),
                    )),
                }),
            ],
        },
    };
    ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![procedure])
        .places(places.freeze())
        .finish(EntryPoint::Int(id))
        .unwrap()
}

#[divan::bench(args = [4, 64, 1024])]
fn record_copy_execute(bencher: Bencher, fields: usize) {
    let program = record_program(fields);
    let mut vm = Vm::new(&program, NoEffects, Limits::default()).unwrap();
    let expected = Integer::checked(IntegerType::S64, fields as i128 - 1).unwrap();
    let result = checked_execute(&mut vm, ProcedureId::new(0), expected, 0);
    report_workload(
        "record_copy",
        fields,
        Limits::default().fuel,
        &result,
        "complete",
        0,
        1,
    );
    bencher
        .counter(ItemsCount::new(fields))
        .bench_local(|| checked_execute(&mut vm, ProcedureId::new(0), expected, 0));
}
