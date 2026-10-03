//! Compare cold and warmed unsigned string indexing without hidden backing growth.
use super::*;
const PERSISTENT_ALLOCATIONS: usize = 2;

fn program(count: usize) -> Program {
    source_program(format!(
        "main::()->int{{text:=\"A\";sum:int=0;for i:0..{count}-1{{index:=cast(u64)i&0;sum+=cast(int)text[index];}}return sum;}}"
    ))
}
fn limits() -> Limits {
    Limits {
        fuel: 4_000_000,
        ..Default::default()
    }
}

#[divan::bench(args = [64, 1024, 4096])]
fn unsigned_string_index_cold(bencher: Bencher, count: usize) {
    let program = program(count);
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected int entry")
    };
    let expected = Integer::checked(IntegerType::S64, 65 * count as i128).unwrap();
    let execute = || {
        let mut vm = Vm::new(&program, NoEffects, limits()).unwrap();
        let execution = completed_execute(&mut vm, entry, expected);
        assert_eq!(
            vm.memory().allocation_count(),
            PERSISTENT_ALLOCATIONS,
            "only implicit context and immutable literal backing may persist"
        );
        execution
    };
    let preflight = execute();
    report_workload(
        "unsigned_string_index_cold",
        count,
        limits().fuel,
        &preflight,
        "complete",
        PERSISTENT_ALLOCATIONS,
        0,
    );
    bencher.counter(ItemsCount::new(count)).bench_local(execute);
}

#[divan::bench(args = [64, 1024, 4096])]
fn unsigned_string_index_reuse(bencher: Bencher, count: usize) {
    let program = program(count);
    let EntryPoint::Int(entry) = program.entry() else {
        panic!("expected int entry")
    };
    let expected = Integer::checked(IntegerType::S64, 65 * count as i128).unwrap();
    let mut vm = Vm::new(&program, NoEffects, limits()).unwrap();
    let mut preflight = checked_execute(&mut vm, entry, expected, PERSISTENT_ALLOCATIONS);
    for _ in 1..40 {
        preflight = checked_execute(&mut vm, entry, expected, PERSISTENT_ALLOCATIONS);
    }
    report_workload(
        "unsigned_string_index_reuse",
        count,
        limits().fuel,
        &preflight,
        "complete",
        PERSISTENT_ALLOCATIONS,
        40,
    );
    bencher
        .counter(ItemsCount::new(count))
        .bench_local(|| checked_execute(&mut vm, entry, expected, PERSISTENT_ALLOCATIONS));
}
