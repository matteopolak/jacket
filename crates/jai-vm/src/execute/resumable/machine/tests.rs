use super::*;
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeRegistry, Variadic};

#[test]
fn readiness_retry_preserves_the_exact_retained_range_charge() {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(0);
    let library = ProgramBuilder::new(types.freeze().unwrap())
        .procedures(vec![Procedure {
            id,
            signature,
            parameters: vec![],
            locals: vec![],
            body: Block {
                statements: vec![],
                flow: Flow::FallsThrough,
            },
            cleanups: vec![],
        }])
        .finish_library()
        .unwrap();
    let limits = Limits::default();
    let plan = compile_checked_procedure(library.checked_procedure(id).unwrap(), limits).unwrap();
    let vm = Vm::new(&library, crate::NoEffects, limits).unwrap();
    let mut machine = Machine::expression(plan.code.clone());
    machine
        .push(
            &vm,
            Some(plan.code),
            0,
            Action::RangeNext(Box::new(RangeState {
                id: LoopId::new(0),
                pointer: None,
                ty: jai_types::IntegerType::S64,
                end: Some(Number::plain(Integer::wrapping(
                    jai_types::IntegerType::S64,
                    42,
                ))),
                direction: jai_types::Direction::Forward,
                body: plan.body,
            })),
            0,
        )
        .unwrap();
    let admitted = machine.retained;
    assert!(admitted > 1);
    for _ in 0..32 {
        let task = machine.tasks.pop().unwrap();
        machine.retained -= task.cells;
        machine.retain_task(&vm, task).unwrap();
        assert_eq!(machine.retained, admitted);
        assert_eq!(machine.tasks.len(), 1);
    }
}
