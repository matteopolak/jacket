//! Exercise the central VM boundary, including actual promoted C-vararg tails.
use super::*;
use crate::{
    NoEffects,
    process_abi::{ProcessAbiNominals, ProcessAuthority},
};
use jai_source::Identities;
use jai_types::{
    Architecture, BuildTarget, ByteOrder, CallingConvention, ContextMode, DistinctKind,
    IntegerType, LayoutPolicy, OperatingSystem, ScalarType, TypeRegistry, Types, Variadic,
};
use std::collections::HashMap;

struct Fixture {
    types: Types,
    signatures: HashMap<ProcedureId, TypeId>,
    proofs: Vec<ProcessAbiProcedure>,
}
impl Fixture {
    fn new(operating_system: OperatingSystem) -> Self {
        let mut types = TypeRegistry::new();
        let integer = types.scalar(ScalarType::Int(IntegerType::S32));
        let pair = types.fixed_array(integer, 2).unwrap();
        let pair = types.pointer(pair).unwrap();
        let error_code = types.reserve_distinct(DistinctKind::IsA);
        types.define_distinct(error_code, integer).unwrap();
        let target = BuildTarget {
            operating_system,
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
        for (index, (operation, symbol, parameters, variadic)) in [
            (
                ProcessAbiOperation::Pipe,
                "pipe",
                vec![pair],
                Variadic::None,
            ),
            (
                ProcessAbiOperation::Fcntl,
                "fcntl",
                vec![integer, integer],
                Variadic::C {
                    fixed_parameters: 2,
                },
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let id = ProcedureId::new(index);
            let signature = types
                .procedure(ProcedureType {
                    parameters: parameters.into(),
                    results: [integer].into(),
                    convention: CallingConvention::C,
                    context: ContextMode::None,
                    variadic,
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
fn integer(value: i32) -> Value {
    Value::Int(jai_types::Integer::wrapping(
        IntegerType::S32,
        i128::from(value),
    ))
}
fn scalar(outcome: ProcessCallOutcome) -> i32 {
    let ProcessCallOutcome::Values(values) = outcome else {
        panic!("expected scalar")
    };
    i32::try_from(values[0].integer().unwrap().value()).unwrap()
}
fn pipe(vm: &mut Vm<'_, Fixture, NoEffects>, fixture: &Fixture) -> [i32; 2] {
    let proof = &fixture.proofs[0];
    let signature = fixture
        .types
        .procedure_definition(proof.signature())
        .unwrap();
    let TypeKind::Pointer(pair_type) = fixture.types.kind(signature.parameters[0]).unwrap() else {
        panic!()
    };
    let pair = vm
        .memory
        .allocate(&fixture.types, *pair_type, None)
        .unwrap();
    assert_eq!(
        scalar(
            vm.invoke_process_available(proof.procedure(), proof, &[Value::Pointer(pair.clone())])
                .unwrap()
        ),
        0
    );
    let result = vm.memory.load(&fixture.types, &pair).unwrap();
    let Value::Array { elements, .. } = result.semantic() else {
        panic!()
    };
    [
        elements[0].integer().unwrap().value() as i32,
        elements[1].integer().unwrap().value() as i32,
    ]
}
fn flags(
    vm: &mut Vm<'_, Fixture, NoEffects>,
    fixture: &Fixture,
    fd: i32,
    command: i32,
    value: Option<i32>,
) -> i32 {
    let proof = &fixture.proofs[1];
    let mut arguments = vec![integer(fd), integer(command)];
    if let Some(value) = value {
        arguments.push(integer(value));
    }
    scalar(
        vm.invoke_process_available(proof.procedure(), proof, &arguments)
            .unwrap(),
    )
}

#[test]
fn central_fcntl_getters_and_promoted_setters_preserve_actual_descriptor_flags() {
    for os in [OperatingSystem::MacOS, OperatingSystem::Linux] {
        let nonblock = if os == OperatingSystem::MacOS {
            4
        } else {
            0x800
        };
        let fixture = Fixture::new(os);
        let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
        let [read, write] = pipe(&mut vm, &fixture);
        assert_eq!(flags(&mut vm, &fixture, read, 1, None), 0);
        assert_eq!(flags(&mut vm, &fixture, read, 2, Some(1)), 0);
        assert_eq!(flags(&mut vm, &fixture, read, 1, None), 1);
        let old = flags(&mut vm, &fixture, write, 3, None);
        assert_eq!(old, 1);
        assert_eq!(flags(&mut vm, &fixture, write, 4, Some(old | nonblock)), 0);
        assert_eq!(flags(&mut vm, &fixture, write, 3, None), 1 | nonblock);
        assert_eq!(flags(&mut vm, &fixture, write, 4, Some(old)), 0);
        assert_eq!(flags(&mut vm, &fixture, write, 3, None), old);
    }
}

#[test]
fn central_fcntl_rejects_bad_tail_or_command_before_ledger_creation() {
    let fixture = Fixture::new(OperatingSystem::MacOS);
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let proof = &fixture.proofs[1];
    for arguments in [
        vec![],
        vec![integer(0)],
        vec![integer(0), integer(2)],
        vec![integer(0), integer(1), integer(0)],
        vec![integer(0), integer(99)],
        vec![
            integer(0),
            integer(2),
            Value::Int(jai_types::Integer::wrapping(IntegerType::S64, 1)),
        ],
    ] {
        assert!(
            vm.invoke_process_available(proof.procedure(), proof, &arguments)
                .is_err()
        );
        assert!(vm.processes.is_none());
    }
}

#[test]
fn central_fcntl_fuel_rejection_preserves_existing_flag_state() {
    let fixture = Fixture::new(OperatingSystem::MacOS);
    let mut vm = Vm::new(&fixture, NoEffects, Limits::default()).unwrap();
    let [read, _] = pipe(&mut vm, &fixture);
    let proof = &fixture.proofs[1];
    vm.limits.fuel = vm.statistics.steps;
    assert!(matches!(
        vm.invoke_process_available(
            proof.procedure(),
            proof,
            &[integer(read), integer(2), integer(1)]
        ),
        Err(Halt::Failed(Error::Limit(LimitKind::Fuel)))
    ));
    let state = vm.processes.as_ref().unwrap();
    let fd = state
        .world
        .descriptor(state.branch.current(), read)
        .unwrap();
    assert!(!state.world.close_on_exec(fd).unwrap());
}
