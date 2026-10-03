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
                    results: [
                        if matches!(
                            operation,
                            ProcessAbiOperation::Read | ProcessAbiOperation::Write
                        ) {
                            long
                        } else {
                            integer
                        },
                    ]
                    .into(),
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
fn scalar(outcome: ProcessCallOutcome) -> i128 {
    let ProcessCallOutcome::Values(values) = outcome else {
        panic!("expected scalar process result")
    };
    values[0].integer().unwrap().value()
}

#[test]
fn process_state_is_lazy_and_getpid_keeps_its_virtual_identity() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    assert!(vm.processes.is_none());
    let proof = &fixture.proofs[0];
    let first = scalar(
        vm.invoke_process_available(proof.procedure(), proof, &[])
            .unwrap(),
    );
    let state = vm.processes.as_ref().unwrap();
    assert_eq!(
        first,
        i128::from(state.branch.current().abi_value().unwrap())
    );
    assert_eq!(
        state.cells(),
        usize::try_from(state.world.work_cost().unwrap()).unwrap()
            + state.branch.retained_metadata_cells()
            + 1
    );
    assert!(vm.statistics.steps > state.work_cost());
    let second = scalar(
        vm.invoke_process_available(proof.procedure(), proof, &[])
            .unwrap(),
    );
    assert_eq!(first, second);
}

#[test]
fn mismatched_receipt_identity_or_target_does_not_initialize_process_state() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    assert!(matches!(
        vm.invoke_process_available(ProcedureId::new(1), &fixture.proofs[0], &[]),
        Err(Halt::Failed(Error::InvalidIr(_)))
    ));
    assert!(vm.processes.is_none());
    let target = crate::ByteTarget {
        endian: crate::byte_memory::Endian::Big,
        ..crate::ByteTarget::default()
    };
    let mut vm = Vm::new_with_target(&fixture, NoEffects, Limits::default(), target).unwrap();
    assert!(matches!(
        vm.invoke_process_available(ProcedureId::new(0), &fixture.proofs[0], &[]),
        Err(Halt::Failed(Error::InvalidIr(_)))
    ));
    assert!(vm.processes.is_none());
}

#[test]
fn rejected_fuel_precedes_virtual_root_creation() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(
        &fixture,
        NoEffects,
        Limits {
            fuel: 0,
            ..Limits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        vm.invoke_process_available(ProcedureId::new(0), &fixture.proofs[0], &[]),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    assert!(vm.processes.is_none());
}

#[test]
fn close_failure_writes_typed_errno_and_refreshes_retained_metadata() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let proof = &fixture.proofs[1];
    let bad_fd = Value::Int(Integer::wrapping(jai_types::IntegerType::S32, 123));
    assert_eq!(
        scalar(
            vm.invoke_process_available(proof.procedure(), proof, &[bad_fd])
                .unwrap()
        ),
        -1
    );
    let state = vm.processes.as_ref().unwrap();
    assert!(state.branch.retained_metadata_cells() > 0);
    assert_eq!(
        state.work_cost(),
        state.world.work_cost().unwrap() + state.branch.retained_metadata_cells() as u64 + 1
    );
}

#[test]
fn fork_keeps_a_control_token_until_the_scheduler_splits_continuations() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let proof = &fixture.proofs[2];
    let ProcessCallOutcome::ForkControl(control) = vm
        .invoke_process_available(proof.procedure(), proof, &[])
        .unwrap()
    else {
        panic!("fork became a scalar result")
    };
    assert_eq!(control.procedure().procedure(), proof.procedure());
    assert_eq!(
        control.parent(),
        vm.processes.as_ref().unwrap().branch.current()
    );
}

#[test]
fn scalar_calls_transfer_the_same_process_identity_with_vm_state() {
    let fixture = Fixture::new();
    let id = fixture.proofs[0].procedure();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let first = vm.execute(id, vec![]).outcome;
    assert!(matches!(&first, crate::Outcome::Complete(values) if values.len() == 1));
    let state = vm.into_state();
    let mut vm = Vm::with_state(&fixture, NoEffects, Limits::default(), state).unwrap();
    assert_eq!(vm.execute(id, vec![]).outcome, first);
}

#[test]
fn unused_process_capabilities_do_not_require_process_metadata_capacity() {
    let fixture = Fixture::new();
    let mut probe = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let (empty_owner, _, _) = probe.test_snapshot_costs();
    let mut vm = Vm::new(
        &fixture,
        NoEffects,
        Limits {
            // Actual empty live/rollback owners plus one source scalar.
            value_cells: 2 * empty_owner + 1,
            ..Limits::default()
        },
    )
    .unwrap();
    assert_eq!(
        vm.evaluate(&ValueExpr::Int(IntExpr::constant(Integer::wrapping(
            jai_types::IntegerType::S64,
            5
        ))))
        .outcome,
        crate::Outcome::Complete(vec![Value::Int(Integer::wrapping(
            jai_types::IntegerType::S64,
            5
        ))])
    );
    assert!(vm.processes.is_none());
}

#[test]
fn pipe_read_keeps_a_pending_event_until_the_same_descriptor_has_bytes() {
    let fixture = Fixture::new();
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let pipe = &fixture.proofs[3];
    let pair_type = fixture
        .types
        .procedure_definition(pipe.signature())
        .unwrap()
        .parameters[0];
    let TypeKind::Pointer(pair_type) = fixture.types.kind(pair_type).unwrap() else {
        unreachable!()
    };
    let pair = vm
        .memory
        .allocate(&fixture.types, *pair_type, None)
        .unwrap();
    assert_eq!(
        scalar(
            vm.invoke_process_available(pipe.procedure(), pipe, &[Value::Pointer(pair.clone())])
                .unwrap()
        ),
        0
    );
    let pair = vm.memory.load(&fixture.types, &pair).unwrap();
    let Value::Array { elements, .. } = pair.semantic() else {
        panic!("expected descriptor pair")
    };
    let byte = fixture
        .types
        .scalar(ScalarType::Int(jai_types::IntegerType::U8));
    let source = vm
        .memory
        .allocate(
            &fixture.types,
            byte,
            Some(Value::Int(Integer::wrapping(
                jai_types::IntegerType::U8,
                77,
            ))),
        )
        .unwrap();
    let destination = vm.memory.allocate(&fixture.types, byte, None).unwrap();
    let read = &fixture.proofs[4];
    let pointer_type = fixture
        .types
        .procedure_definition(read.signature())
        .unwrap()
        .parameters[1];
    let TypeKind::Pointer(void) = fixture.types.kind(pointer_type).unwrap() else {
        unreachable!()
    };
    let to_void = |memory: &Memory, pointer: &Pointer| {
        memory
            .cast_pointer(
                &fixture.types,
                pointer,
                *void,
                jai_types::CastMode::Unchecked,
            )
            .unwrap()
    };
    let count = Value::Int(Integer::wrapping(jai_types::IntegerType::U64, 1));
    let read_arguments = [
        elements[0].clone(),
        Value::Pointer(to_void(&vm.memory, &destination)),
        count.clone(),
    ];
    let ProcessCallOutcome::Pending(event) = vm
        .invoke_process_available(read.procedure(), read, &read_arguments)
        .unwrap()
    else {
        panic!("empty pipe returned a fake scalar result")
    };
    let crate::virtual_process::ProcessEvent::Readable(descriptor) = event else {
        panic!("wrong process dependency")
    };
    assert_eq!(
        descriptor.process(),
        vm.processes.as_ref().unwrap().branch.current()
    );
    assert_eq!(
        descriptor.number(),
        elements[0].integer().unwrap().value() as i32
    );
    let write = &fixture.proofs[5];
    let write_arguments = [
        elements[1].clone(),
        Value::Pointer(to_void(&vm.memory, &source)),
        count,
    ];
    assert_eq!(
        scalar(
            vm.invoke_process_available(write.procedure(), write, &write_arguments)
                .unwrap()
        ),
        1
    );
    assert_eq!(
        scalar(
            vm.invoke_process_available(read.procedure(), read, &read_arguments)
                .unwrap()
        ),
        1
    );
    assert_eq!(
        vm.memory
            .load(&fixture.types, &destination)
            .unwrap()
            .integer()
            .unwrap()
            .value(),
        77
    );
}
