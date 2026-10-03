use super::*;
use crate::{
    NoEffects,
    process_abi::{ProcessAbiNominals, ProcessAuthority},
};
use jai_source::Identities;
use jai_types::{
    Architecture, BuildTarget, ByteOrder, CallingConvention, ContextMode, DistinctKind,
    LayoutPolicy, OperatingSystem, ScalarType, TypeRegistry, Types, Variadic,
};
use std::collections::HashMap;

struct Fixture {
    types: Types,
    signatures: HashMap<ProcedureId, TypeId>,
    proofs: Vec<ProcessAbiProcedure>,
}
impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(jai_types::IntegerType::S32));
        let long = types.scalar(ScalarType::Int(jai_types::IntegerType::S64));
        let count = types.scalar(ScalarType::Int(jai_types::IntegerType::U64));
        let pair = types.fixed_array(integer, 2).unwrap();
        let pair = types.pointer(pair).unwrap();
        let bytes = types.pointer(types.void()).unwrap();
        let error_code = types.reserve_distinct(DistinctKind::IsA);
        types.define_distinct(error_code, integer).unwrap();
        let target = BuildTarget {
            operating_system: OperatingSystem::MacOS,
            architecture: Architecture::Arm64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        };
        let library = ForeignLibrary {
            id: ForeignLibraryId::new(Identities::default().declaration()),
            kind: ForeignLibraryKind::System {
                name: "libc".into(),
            },
            options: Default::default(),
        };
        let mut proofs = Vec::new();
        let mut signatures = HashMap::new();
        for (index, (operation, symbol, parameters)) in [
            (ProcessAbiOperation::GetPid, "getpid", vec![]),
            (ProcessAbiOperation::Close, "close", vec![integer]),
            (ProcessAbiOperation::Fork, "fork", vec![]),
            (ProcessAbiOperation::Pipe, "pipe", vec![pair]),
            (ProcessAbiOperation::Exit, "_exit", vec![integer]),
            (
                ProcessAbiOperation::Read,
                "read",
                vec![integer, bytes, count],
            ),
            (
                ProcessAbiOperation::Write,
                "write",
                vec![integer, bytes, count],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let id = ProcedureId::new(index);
            let signature = types
                .procedure(ProcedureType {
                    parameters: parameters.into(),
                    results: if operation == ProcessAbiOperation::Exit {
                        Box::new([])
                    } else {
                        [
                            if matches!(
                                operation,
                                ProcessAbiOperation::Read | ProcessAbiOperation::Write
                            ) {
                                long
                            } else {
                                integer
                            },
                        ]
                        .into()
                    },
                    return_abi: jai_types::ForeignReturnAbi::Natural,
                    convention: CallingConvention::C,
                    context: ContextMode::None,
                    variadic: Variadic::None,
                })
                .unwrap();
            let authority = ProcessAuthority::from_verified_source(
                target.clone(),
                library.clone(),
                ProcessAbiNominals {
                    error_code,
                    socket: None,
                },
                [(id, operation, signature)],
                &types,
            )
            .unwrap();
            proofs.push(
                authority
                    .bind(
                        &ProcedurePrototype {
                            id,
                            signature,
                            origin: PrototypeOrigin::Foreign {
                                symbol: symbol.into(),
                                library: Some(library.clone()),
                            },
                        },
                        &target,
                        &types,
                    )
                    .unwrap(),
            );
            signatures.insert(id, signature);
        }
        Self {
            types: types.freeze().unwrap(),
            signatures,
            proofs,
        }
    }
}
impl ProcedureProvider for Fixture {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.signatures
    }
    fn procedure(&self, id: ProcedureId) -> ProcedureAvailability<'_> {
        self.proofs
            .get(id.index())
            .map_or(ProcedureAvailability::Missing, |proof| {
                ProcedureAvailability::ProcessAbi(proof.clone())
            })
    }
}

fn stopped_fork(fixture: &Fixture) -> (Vm<'_, Fixture, NoEffects>, Machine) {
    let mut vm = Vm::new(fixture, NoEffects, Limits::default()).unwrap();
    let mut machine = Machine::procedure(ProcedureId::new(2), vec![]);
    assert_eq!(machine.drive(&mut vm).unwrap(), DriveStatus::ProcessControl);
    assert!(!vm.effects.has_journal());
    assert!(matches!(
        machine.process_control(),
        Some(ProcessControl::Fork(_))
    ));
    (vm, machine)
}

#[test]
fn actual_pair_results_resume_the_same_call_once_in_both_private_machines() {
    let fixture = Fixture::new();
    let (mut vm, mut parent_machine) = stopped_fork(&fixture);
    let calls = vm.statistics.calls;
    let steps = vm.statistics.steps;
    assert_eq!(
        parent_machine.drive(&mut vm).unwrap(),
        DriveStatus::ProcessControl
    );
    assert_eq!((vm.statistics.calls, vm.statistics.steps), (calls, steps));
    let mut child_machine = parent_machine
        .fork_private(parent_machine.fork_work_cost().unwrap())
        .unwrap();
    let mut world = vm.processes.as_ref().unwrap().world.clone();
    let parent = vm.processes.as_ref().unwrap().branch.clone();
    let pair = world.fork(parent.current()).unwrap();
    let child = parent.for_child(pair.child, &world).unwrap();
    let prepared_parent = parent_machine
        .prepare_fork_result(&mut vm, &world, &parent, pair)
        .unwrap();
    let prepared_child = child_machine
        .prepare_fork_result(&mut vm, &world, &child, pair)
        .unwrap();
    parent_machine
        .commit_process_result(&vm, &world, &parent, prepared_parent)
        .unwrap();
    child_machine
        .commit_process_result(&vm, &world, &child, prepared_child)
        .unwrap();
    assert_eq!(
        parent_machine.drive(&mut vm).unwrap(),
        DriveStatus::Complete
    );
    assert_eq!(child_machine.drive(&mut vm).unwrap(), DriveStatus::Complete);
    assert_eq!(
        parent_machine.values().unwrap()[0]
            .integer()
            .unwrap()
            .value(),
        i128::from(pair.child.abi_value().unwrap())
    );
    assert_eq!(
        child_machine.values().unwrap()[0]
            .integer()
            .unwrap()
            .value(),
        0
    );
    assert_eq!(vm.statistics.calls, calls);
    assert!(!vm.effects.has_journal());
    assert!(
        parent_machine
            .prepare_fork_result(&mut vm, &world, &parent, pair)
            .is_err()
    );
}

#[test]
fn scalar_fork_result_runs_the_waiting_checked_expression_without_call_replay() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let ty = jai_types::IntegerType::S32;
    let call = IntExpr::new(
        ty,
        IntExprKind::Call(Call::new(ProcedureId::new(2), vec![])),
    );
    let expression = ValueExpr::Int(IntExpr::new(ty, IntExprKind::Complement(Box::new(call))));
    let places = Places::default();
    let checked = jai_ir::verify_expression(
        &fixture.types,
        &expression,
        &fixture.signatures,
        &[],
        &places,
    )
    .unwrap();
    let code = plan::compile_checked_expression(checked, vm.limits).unwrap();
    let mut parent_machine = Machine::expression(code);
    assert_eq!(
        parent_machine.drive(&mut vm).unwrap(),
        DriveStatus::ProcessControl
    );
    let mut child_machine = parent_machine
        .fork_private(parent_machine.fork_work_cost().unwrap())
        .unwrap();
    let mut world = vm.processes.as_ref().unwrap().world.clone();
    let parent = vm.processes.as_ref().unwrap().branch.clone();
    let pair = world.fork(parent.current()).unwrap();
    let child = parent.for_child(pair.child, &world).unwrap();
    let prepared_parent = parent_machine
        .prepare_fork_result(&mut vm, &world, &parent, pair)
        .unwrap();
    let prepared_child = child_machine
        .prepare_fork_result(&mut vm, &world, &child, pair)
        .unwrap();
    parent_machine
        .commit_process_result(&vm, &world, &parent, prepared_parent)
        .unwrap();
    child_machine
        .commit_process_result(&vm, &world, &child, prepared_child)
        .unwrap();
    assert_eq!(
        parent_machine.drive(&mut vm).unwrap(),
        DriveStatus::Complete
    );
    assert_eq!(child_machine.drive(&mut vm).unwrap(), DriveStatus::Complete);
    assert_eq!(
        parent_machine.values().unwrap()[0]
            .integer()
            .unwrap()
            .value(),
        !i128::from(pair.child.abi_value().unwrap())
    );
    assert_eq!(
        child_machine.values().unwrap()[0]
            .integer()
            .unwrap()
            .value(),
        -1
    );
    assert_eq!(vm.statistics.calls, 1);
    assert!(!vm.effects.has_journal());
}

#[test]
fn mismatched_pair_and_cross_branch_commit_preserve_the_stop() {
    let fixture = Fixture::new();
    let (mut vm, mut machine) = stopped_fork(&fixture);
    let mut world = vm.processes.as_ref().unwrap().world.clone();
    let parent = vm.processes.as_ref().unwrap().branch.clone();
    let unrelated = world.create_root().unwrap();
    let wrong_pair = world.fork(unrelated).unwrap();
    let retained = machine.retained_cells();
    assert!(
        machine
            .prepare_fork_result(&mut vm, &world, &parent, wrong_pair)
            .is_err()
    );
    assert_eq!(machine.retained_cells(), retained);
    assert!(matches!(
        machine.process_control(),
        Some(ProcessControl::Fork(_))
    ));
    let pair = world.fork(parent.current()).unwrap();
    let child = parent.for_child(pair.child, &world).unwrap();
    let prepared = machine
        .prepare_fork_result(&mut vm, &world, &parent, pair)
        .unwrap();
    // Preparation reserves the actual operand backing; a rejected commit must
    // preserve that prepared owner as well as the still-stopped source progress.
    let retained = machine.retained_cells();
    assert!(
        machine
            .commit_process_result(&vm, &world, &child, prepared)
            .is_err()
    );
    assert_eq!(machine.retained_cells(), retained);
    assert!(matches!(
        machine.process_control(),
        Some(ProcessControl::Fork(_))
    ));
}

#[test]
fn fork_result_budget_failure_preserves_the_stop_before_operand_reservation() {
    let fixture = Fixture::new();
    let (mut vm, mut machine) = stopped_fork(&fixture);
    let mut world = vm.processes.as_ref().unwrap().world.clone();
    let parent = vm.processes.as_ref().unwrap().branch.clone();
    let pair = world.fork(parent.current()).unwrap();
    let retained = machine.retained_cells();
    let capacity = machine.operands.capacity();
    let limits = vm.limits;
    vm.limits.fuel = vm.statistics.steps;
    assert!(matches!(
        machine.prepare_fork_result(&mut vm, &world, &parent, pair),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert_eq!(machine.retained_cells(), retained);
    assert_eq!(machine.operands.capacity(), capacity);
    vm.limits = limits;
    vm.limits.value_cells = 1;
    assert!(matches!(
        machine.prepare_fork_result(&mut vm, &world, &parent, pair),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert_eq!(machine.retained_cells(), retained);
    assert_eq!(machine.operands.capacity(), capacity);
    assert!(matches!(
        machine.process_control(),
        Some(ProcessControl::Fork(_))
    ));
}

#[test]
fn process_stop_capacity_is_admitted_before_lazy_world_creation() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(
        &fixture,
        NoEffects,
        Limits {
            value_cells: 10,
            ..Limits::default()
        },
    )
    .unwrap();
    let mut machine = Machine::procedure(ProcedureId::new(2), vec![]);
    assert!(matches!(
        machine.drive(&mut vm),
        Err(Halt::Failed(Error::Limit(LimitKind::ValueCells)))
    ));
    assert!(vm.processes.is_none());
    assert!(!vm.effects.has_journal());
}

#[test]
fn process_pending_retains_typed_arguments_without_a_compiler_journal_leaf() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let pipe = &fixture.proofs[3];
    let pipe_signature = fixture
        .types
        .procedure_definition(pipe.signature())
        .unwrap();
    let TypeKind::Pointer(pair_ty) = *fixture.types.kind(pipe_signature.parameters[0]).unwrap()
    else {
        panic!("pipe output is not a pointer")
    };
    vm.prepare_layout(pair_ty).unwrap();
    let pair_pointer = vm.memory.allocate(&fixture.types, pair_ty, None).unwrap();
    vm.invoke_process_available(
        pipe.procedure(),
        pipe,
        &[Value::Pointer(pair_pointer.clone())],
    )
    .unwrap();
    let reader_pointer = vm.memory.index(&fixture.types, &pair_pointer, 0).unwrap();
    let writer_pointer = vm.memory.index(&fixture.types, &pair_pointer, 1).unwrap();
    let reader = vm.memory.load(&fixture.types, &reader_pointer).unwrap();
    let writer = vm
        .memory
        .load(&fixture.types, &writer_pointer)
        .unwrap()
        .integer()
        .unwrap()
        .value() as i32;
    let read = &fixture.proofs[5];
    let read_signature = fixture
        .types
        .procedure_definition(read.signature())
        .unwrap();
    let count_ty = read_signature.parameters[2];
    let TypeKind::Pointer(void) = *fixture.types.kind(read_signature.parameters[1]).unwrap() else {
        panic!("read buffer is not a pointer")
    };
    let backing = vm
        .memory
        .allocate(
            &fixture.types,
            count_ty,
            Some(Value::Int(
                Integer::checked(jai_types::IntegerType::U64, 0).unwrap(),
            )),
        )
        .unwrap();
    let buffer = vm
        .memory
        .cast_pointer(&fixture.types, &backing, void, jai_types::CastMode::Checked)
        .unwrap();
    let arguments = vec![
        reader,
        Value::Pointer(buffer),
        Value::Int(Integer::checked(jai_types::IntegerType::U64, 1).unwrap()),
    ];
    let mut machine = Machine::procedure(read.procedure(), arguments.clone());
    let Err(Halt::Pending(Dependency::Process(event))) = machine.drive(&mut vm) else {
        panic!("read did not retain genuine process readiness")
    };
    let Action::Invoke {
        arguments: retained,
        charged,
        retry_leaf,
        ..
    } = &machine.tasks.last().unwrap().action
    else {
        panic!("pending invocation lost its owned operands")
    };
    assert_eq!(retained, &arguments);
    assert!(*charged);
    assert!(!*retry_leaf);
    assert!(!vm.effects.has_journal());
    let calls = vm.statistics.calls;
    let state = vm.processes.as_mut().unwrap();
    let writer = state
        .world
        .descriptor(state.branch.current(), writer)
        .unwrap();
    state.world.write(writer, b"K").unwrap();
    assert!(state.world.event_ready(event).unwrap());
    state.refresh(vm.limits).unwrap();
    assert_eq!(machine.drive(&mut vm).unwrap(), DriveStatus::Complete);
    assert_eq!(machine.values().unwrap()[0].integer().unwrap().value(), 1);
    assert_eq!(vm.statistics.calls, calls);
    assert_eq!(
        vm.memory
            .load(&fixture.types, &backing)
            .unwrap()
            .integer()
            .unwrap()
            .bits(),
        u64::from(b'K')
    );
    assert!(!vm.effects.has_journal());
}

#[test]
fn exit_retires_source_progress_without_returning_or_dispatching_downstream_tasks() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let status = Value::Int(Integer::checked(jai_types::IntegerType::S32, 7).unwrap());
    let mut machine = Machine::procedure(ProcedureId::new(4), vec![status]);
    assert_eq!(machine.drive(&mut vm).unwrap(), DriveStatus::ProcessControl);
    let Some(ProcessControl::Exit(control)) = machine.process_control() else {
        panic!("missing actual exit receipt")
    };
    let (process, status) = (control.process(), control.status());
    vm.processes
        .as_mut()
        .unwrap()
        .world
        .exit(process, status)
        .unwrap();
    let branch = vm.processes.as_ref().unwrap().branch.clone();
    machine.retire_source_control(&mut vm, &branch).unwrap();
    assert!(machine.tasks.is_empty());
    assert!(machine.operands.is_empty());
    assert!(machine.values().is_none());
    assert_eq!(machine.retained_cells(), 0);
    assert_eq!(machine.drive(&mut vm).unwrap(), DriveStatus::Retired);
    assert!(machine.fork_private(usize::MAX).is_err());
    assert!(!vm.effects.has_journal());
}
