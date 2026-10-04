//! Match the mixed-module native checks in the interpreter's Runtime phase.
use super::mixed_module::{Fixture, target};
use divan::{Bencher, counter::ItemsCount};
use jai_codegen::optimization::BitcodeOptimization;
use jai_ir::{EntryPoint, ProcedureId, Program};
use jai_vm::{ByteTarget, Execution, ExecutionPhase, Limits, NoEffects, Outcome, Vm};

const FUEL: u64 = 64_000_000;
const PERSISTENT_ALLOCATIONS: usize = 1;

fn execute(vm: &mut Vm<'_, Program, NoEffects>, entry: ProcedureId, expected: i128) -> Execution {
    let result = vm.execute(entry, vec![]);
    assert!(
        matches!(&result.outcome, Outcome::Complete(values)
        if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == expected)),
        "mixed runtime benchmark must complete: {result:?}"
    );
    assert_eq!(
        vm.memory().allocation_count(),
        PERSISTENT_ALLOCATIONS,
        "only default context may persist after the scalar result"
    );
    result
}

fn report(name: &str, count: usize, source_bytes: usize, result: &Execution, warmups: usize) {
    eprintln!(
        "JAI_BENCH_WORKLOAD {{\"name\":\"{name}\",\"size\":{count},\"source_bytes\":{source_bytes},\"phase\":\"runtime\",\"fuel\":{FUEL},\"steps\":{},\"calls\":{},\"expected_outcome\":\"complete\",\"live_allocations\":{PERSISTENT_ALLOCATIONS},\"warmup_calls\":{warmups}}}",
        result.statistics.steps, result.statistics.calls
    );
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_runtime_cold(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let target = target(BitcodeOptimization::O0);
    let program = fixture.program(&target);
    let byte_target = ByteTarget::from(&target.build_target().unwrap());
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry")
    };
    let run = || {
        let mut vm = Vm::new_with_execution_phase(
            &program,
            NoEffects,
            Limits {
                fuel: FUEL,
                ..Default::default()
            },
            byte_target,
            ExecutionPhase::Runtime,
        )
        .unwrap();
        execute(&mut vm, entry, fixture.expected(3))
    };
    report("mixed_runtime_cold", count, fixture.source.len(), &run(), 0);
    bencher.counter(ItemsCount::new(count)).bench_local(run);
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_runtime_reuse(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let target = target(BitcodeOptimization::O0);
    let program = fixture.program(&target);
    let byte_target = ByteTarget::from(&target.build_target().unwrap());
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry")
    };
    let mut vm = Vm::new_with_execution_phase(
        &program,
        NoEffects,
        Limits {
            fuel: FUEL,
            ..Default::default()
        },
        byte_target,
        ExecutionPhase::Runtime,
    )
    .unwrap();
    let mut result = execute(&mut vm, entry, fixture.expected(3));
    for _ in 1..40 {
        result = execute(&mut vm, entry, fixture.expected(3));
    }
    report(
        "mixed_runtime_reuse",
        count,
        fixture.source.len(),
        &result,
        40,
    );
    bencher
        .counter(ItemsCount::new(count))
        .bench_local(|| execute(&mut vm, entry, fixture.expected(3)));
}
