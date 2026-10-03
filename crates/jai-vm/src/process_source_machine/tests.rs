use super::*;
use crate::{Limits, Number};
use jai_ir::{
    ForeignLibrary, ForeignLibraryId, ForeignLibraryKind, ProcedureId, ProcedurePrototype,
    PrototypeOrigin,
};
use jai_source::Identities;
use jai_types::{
    Architecture, ByteOrder, CallingConvention, CastMode, ContextMode, DistinctKind, LayoutPolicy,
    ProcedureType, ScalarType, TypeRegistry, Variadic,
};

struct Fixture {
    types: TypeRegistry,
    target: BuildTarget,
    memory: Memory,
    world: VirtualProcesses,
    branch: ProcessBranchState,
    procedures: Vec<ProcessAbiProcedure>,
    charged: u64,
}
impl Fixture {
    fn new() -> Self {
        Self::new_for(OperatingSystem::MacOS)
    }
    fn new_for(operating_system: OperatingSystem) -> Self {
        let mut types = TypeRegistry::new();
        let s32 = types.scalar(ScalarType::Int(IntegerType::S32));
        let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
        let u64 = types.scalar(ScalarType::Int(IntegerType::U64));
        let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
        let byte = types.pointer(u8).unwrap();
        let argv = types.pointer(byte).unwrap();
        let void = types.pointer(types.void()).unwrap();
        let array = types.fixed_array(s32, 2).unwrap();
        let pair = types.pointer(array).unwrap();
        let status = types.pointer(s32).unwrap();
        let error = types.reserve_distinct(DistinctKind::IsA);
        types.define_distinct(error, s32).unwrap();
        let errno = types.pointer(error).unwrap();
        let u32 = types.scalar(ScalarType::Int(IntegerType::U32));
        let mut nominal = |repr| {
            let ty = types.reserve_enum(repr);
            types.define_enum(ty, []).unwrap();
            ty
        };
        let socket_kind = nominal(IntegerType::U32);
        let protocol = nominal(IntegerType::U32);
        let message_flags = nominal(IntegerType::S32);
        let shutdown_kind = nominal(IntegerType::U32);
        let io_vector = types.reserve_record(jai_types::RecordKind::Struct);
        types.define_record(io_vector, [void, u64]).unwrap();
        let vector_pointer = types.pointer(io_vector).unwrap();
        let mac = operating_system == OperatingSystem::MacOS;
        let length = if mac {
            u32
        } else {
            u64
        };
        let iov_count = if mac {
            s32
        } else {
            u64
        };
        let control_header = types.reserve_record(jai_types::RecordKind::Struct);
        types
            .define_record(control_header, [length, s32, s32])
            .unwrap();
        let message_header = types.reserve_record(jai_types::RecordKind::Struct);
        types
            .define_record(
                message_header,
                [void, u32, vector_pointer, iov_count, void, length, s32],
            )
            .unwrap();
        let message_pointer = types.pointer(message_header).unwrap();
        let socket = crate::process_abi::ProcessSocketTypes {
            socket_kind,
            protocol,
            message_flags,
            shutdown_kind,
            message_header,
            control_header,
            io_vector,
        };

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
        let mut procedures = Vec::new();
        for (index, (op, symbol, parameters, result)) in [
            (ProcessAbiOperation::Pipe, "pipe", vec![pair], s32),
            (ProcessAbiOperation::Close, "close", vec![s32], s32),
            (ProcessAbiOperation::Read, "read", vec![s32, void, u64], s64),
            (
                ProcessAbiOperation::Write,
                "write",
                vec![s32, void, u64],
                s64,
            ),
            (ProcessAbiOperation::Dup2, "dup2", vec![s32, s32], s32),
            (ProcessAbiOperation::GetPid, "getpid", vec![], s32),
            (ProcessAbiOperation::GetParentPid, "getppid", vec![], s32),
            (ProcessAbiOperation::ErrnoLocation, "__error", vec![], errno),
            (ProcessAbiOperation::Fork, "fork", vec![], s32),
            (
                ProcessAbiOperation::WaitPid,
                "waitpid",
                vec![s32, status, s32],
                s32,
            ),
            (ProcessAbiOperation::Exit, "_exit", vec![s32], types.void()),
            (ProcessAbiOperation::ExecVp, "execvp", vec![byte, argv], s32),
            (
                ProcessAbiOperation::SocketPair,
                "socketpair",
                vec![s32, socket_kind, protocol, pair],
                s32,
            ),
            (
                ProcessAbiOperation::SendMsg,
                "sendmsg",
                vec![s32, message_pointer, message_flags],
                s64,
            ),
            (
                ProcessAbiOperation::RecvMsg,
                "recvmsg",
                vec![s32, message_pointer, message_flags],
                s64,
            ),
            (
                ProcessAbiOperation::Shutdown,
                "shutdown",
                vec![s32, shutdown_kind],
                s32,
            ),
            (ProcessAbiOperation::Fcntl, "fcntl", vec![s32, s32], s32),
        ]
        .into_iter()
        .enumerate()
        {
            let signature = types
                .procedure(ProcedureType {
                    parameters: parameters.into(),
                    results: if result == types.void() {
                        Vec::new().into()
                    } else {
                        [result].into()
                    },
                    return_abi: jai_types::ForeignReturnAbi::Natural,
                    convention: CallingConvention::C,
                    context: ContextMode::None,
                    variadic: if op == ProcessAbiOperation::Fcntl {
                        Variadic::C {
                            fixed_parameters: 2,
                        }
                    } else {
                        Variadic::None
                    },
                })
                .unwrap();
            let id = ProcedureId::new(index);
            let auth = crate::process_abi::ProcessAuthority::from_verified_source(
                target.clone(),
                library.clone(),
                crate::process_abi::ProcessAbiNominals {
                    error_code: error,
                    socket: Some(socket),
                },
                [(id, op, signature)],
                &types,
            )
            .unwrap();
            procedures.push(
                auth.bind(
                    &ProcedurePrototype {
                        id,
                        signature,
                        origin: PrototypeOrigin::Foreign {
                            symbol: if symbol == "__error" && !mac {
                                "__errno_location".into()
                            } else {
                                symbol.into()
                            },
                            library: Some(library.clone()),
                        },
                    },
                    &target,
                    &types,
                )
                .unwrap(),
            );
        }
        let mut world = VirtualProcesses::new(Default::default());
        let root = world.create_root().unwrap();
        Self {
            types,
            target,
            memory: Memory::new(Limits::default()),
            world,
            branch: ProcessBranchState::new(root),
            procedures,
            charged: 0,
        }
    }
    fn proof(&self, op: ProcessAbiOperation) -> ProcessAbiProcedure {
        self.procedures
            .iter()
            .find(|proof| proof.operation() == op)
            .unwrap()
            .clone()
    }
    fn call(
        &mut self,
        op: ProcessAbiOperation,
        args: &[Value],
    ) -> Result<ProcessCallOutcome, Error> {
        self.proof(op).invoke_process(
            args,
            &mut self.memory,
            &self.types,
            &self.target,
            &mut self.branch,
            &mut self.world,
            &mut |cost| {
                self.charged += cost;
                Ok(())
            },
        )
    }
    fn pair_buffer(&mut self) -> Pointer {
        let s32 = self.types.scalar(ScalarType::Int(IntegerType::S32));
        let pair = self.types.fixed_array(s32, 2).unwrap();
        self.memory.allocate(&self.types, pair, None).unwrap()
    }
    fn bytes(&mut self, bytes: &[u8]) -> Pointer {
        let u8 = self.types.scalar(ScalarType::Int(IntegerType::U8));
        let array = self.types.fixed_array(u8, bytes.len() as u64).unwrap();
        let root = self
            .memory
            .allocate(
                &self.types,
                array,
                Some(Value::Array {
                    ty: array,
                    elements: bytes
                        .iter()
                        .map(|byte| int(IntegerType::U8, i128::from(*byte)))
                        .collect(),
                }),
            )
            .unwrap();
        self.memory
            .cast_pointer(&self.types, &root, self.types.void(), CastMode::Unchecked)
            .unwrap()
    }
    fn pipe(&mut self) -> [i32; 2] {
        let buffer = self.pair_buffer();
        assert_eq!(
            scalar(
                self.call(ProcessAbiOperation::Pipe, &[Value::Pointer(buffer.clone())])
                    .unwrap()
            ),
            0
        );
        let Value::Array {
            elements, ..
        } = self.memory.load(&self.types, &buffer).unwrap()
        else {
            panic!("pipe ABI must store actual array")
        };
        [
            fd_number(&elements[0]).unwrap(),
            fd_number(&elements[1]).unwrap(),
        ]
    }
    fn errno(&mut self) -> (Pointer, i128) {
        let ProcessCallOutcome::Values(values) =
            self.call(ProcessAbiOperation::ErrnoLocation, &[]).unwrap()
        else {
            panic!("errno pointer")
        };
        let pointer = values[0].pointer().unwrap().clone();
        let Value::Distinct {
            value, ..
        } = self.memory.load(&self.types, &pointer).unwrap()
        else {
            panic!("actual error nominal")
        };
        (pointer, value.integer().unwrap().value())
    }
}
fn s32(number: i32) -> Value {
    int(IntegerType::S32, i128::from(number))
}
fn u64(number: usize) -> Value {
    int(IntegerType::U64, number as i128)
}
fn scalar(outcome: ProcessCallOutcome) -> i128 {
    let ProcessCallOutcome::Values(values) = outcome else {
        panic!("expected actual values")
    };
    assert_eq!(values.len(), 1);
    values[0].integer().unwrap().value()
}

#[test]
fn typed_pipe_read_write_dup_close_pending_and_eof_preserve_real_memory() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let input = fixture.bytes(b"hello");
    let output = fixture.bytes(b".....");
    let read_args = [s32(reader), Value::Pointer(output.clone()), u64(5)];
    let ProcessCallOutcome::Pending(event) =
        fixture.call(ProcessAbiOperation::Read, &read_args).unwrap()
    else {
        panic!("empty open pipe must block")
    };
    assert!(!fixture.world.event_ready(event).unwrap());
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 5)
            .unwrap(),
        b"....."
    );
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Write,
                    &[s32(writer), Value::Pointer(input), u64(5)]
                )
                .unwrap()
        ),
        5
    );
    assert!(fixture.world.event_ready(event).unwrap());
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::Read, &read_args).unwrap()),
        5
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 5)
            .unwrap(),
        b"hello"
    );
    assert_eq!(
        scalar(
            fixture
                .call(ProcessAbiOperation::Dup2, &[s32(writer), s32(90)])
                .unwrap()
        ),
        90
    );
    fixture
        .call(ProcessAbiOperation::Close, &[s32(writer)])
        .unwrap();
    assert!(matches!(
        fixture.call(ProcessAbiOperation::Read, &read_args).unwrap(),
        ProcessCallOutcome::Pending(_)
    ));
    fixture
        .call(ProcessAbiOperation::Close, &[s32(90)])
        .unwrap();
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::Read, &read_args).unwrap()),
        0
    );
    fixture
        .call(ProcessAbiOperation::Close, &[s32(reader)])
        .unwrap();
    fixture
        .world
        .require_quiescent(fixture.branch.current())
        .unwrap();
    assert!(fixture.charged > 0);
}

#[test]
fn virtual_errno_is_a_mutable_stable_nominal_cell_and_errors_are_recoverable() {
    let mut fixture = Fixture::new();
    assert_eq!(
        scalar(
            fixture
                .call(ProcessAbiOperation::Close, &[s32(-1)])
                .unwrap()
        ),
        -1
    );
    let (errno, value) = fixture.errno();
    assert_eq!(value, 9);
    fixture
        .memory
        .store(&fixture.types, &errno, errno_value(errno.pointee(), 123))
        .unwrap();
    assert_eq!(fixture.errno(), (errno.clone(), 123));
    let [reader, writer] = fixture.pipe();
    let out = fixture.bytes(b".");
    let fd = fixture
        .world
        .descriptor(fixture.branch.current(), reader)
        .unwrap();
    fixture.world.set_nonblocking(fd, true).unwrap();
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Read,
                    &[s32(reader), Value::Pointer(out.clone()), u64(1)]
                )
                .unwrap()
        ),
        -1
    );
    assert_eq!(fixture.errno(), (errno.clone(), 35));
    fixture
        .call(ProcessAbiOperation::Close, &[s32(reader)])
        .unwrap();
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Write,
                    &[s32(writer), Value::Pointer(out), u64(1)]
                )
                .unwrap()
        ),
        -1
    );
    assert_eq!(fixture.errno(), (errno, 32));
}

#[test]
fn fuel_failure_readonly_and_wrong_signature_leave_pipe_world_unmodified() {
    let mut fixture = Fixture::new();
    let buffer = fixture.pair_buffer();
    let proof = fixture.proof(ProcessAbiOperation::Pipe);
    let cells = fixture.memory.value_cells();
    let cost = fixture.world.work_cost().unwrap();
    assert_eq!(
        proof
            .invoke_process(
                &[Value::Pointer(buffer.clone())],
                &mut fixture.memory,
                &fixture.types,
                &fixture.target,
                &mut fixture.branch,
                &mut fixture.world,
                &mut |_| Err(Error::Limit(LimitKind::Fuel))
            )
            .unwrap_err(),
        Error::Limit(LimitKind::Fuel)
    );
    assert_eq!(fixture.world.work_cost().unwrap(), cost);
    assert_eq!(fixture.memory.value_cells(), cells);
    fixture.memory.freeze(&buffer).unwrap();
    assert_eq!(
        fixture
            .call(ProcessAbiOperation::Pipe, &[Value::Pointer(buffer)])
            .unwrap_err(),
        Error::ReadOnlyStorage
    );
    fixture
        .world
        .require_quiescent(fixture.branch.current())
        .unwrap();
    assert!(fixture.call(ProcessAbiOperation::Close, &[u64(3)]).is_err());
    assert!(fixture.call(ProcessAbiOperation::Close, &[]).is_err());
}

#[test]
fn no_address_integer_or_pointer_provenance_can_be_sent_as_process_bytes() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let plain = fixture.bytes(b"x");
    let tagged = Value::AddressInteger(Number::address(
        Integer::checked(IntegerType::S32, writer as i128).unwrap(),
        crate::AddressProvenance::Pointer(plain.clone()),
    ));
    assert!(matches!(
        fixture.call(ProcessAbiOperation::Close, &[tagged]),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    let byte = fixture.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_type = fixture.types.pointer(byte).unwrap();
    let live_pointer = fixture
        .memory
        .cast_pointer(&fixture.types, &plain, byte, CastMode::Unchecked)
        .unwrap();
    let storage = fixture
        .memory
        .allocate(
            &fixture.types,
            pointer_type,
            Some(Value::Pointer(live_pointer)),
        )
        .unwrap();
    let bytes = fixture
        .memory
        .cast_pointer(
            &fixture.types,
            &storage,
            fixture.types.void(),
            CastMode::Unchecked,
        )
        .unwrap();
    assert!(matches!(
        fixture.call(
            ProcessAbiOperation::Write,
            &[s32(writer), Value::Pointer(bytes), u64(8)]
        ),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert_eq!(
        fixture
            .world
            .readiness(
                fixture
                    .world
                    .descriptor(fixture.branch.current(), reader)
                    .unwrap()
            )
            .unwrap()
            .bytes,
        0
    );
}

#[test]
fn fork_is_sealed_control_and_child_pid_parent_ids_follow_real_ledger_relation() {
    let mut fixture = Fixture::new();
    let current = fixture.branch.current();
    let ProcessCallOutcome::ForkControl(control) =
        fixture.call(ProcessAbiOperation::Fork, &[]).unwrap()
    else {
        panic!("fork must not return a made-up scalar")
    };
    assert_eq!(control.parent(), current);
    assert_eq!(control.procedure().operation(), ProcessAbiOperation::Fork);
    fixture.world.require_quiescent(current).unwrap();
    assert!(
        fixture
            .call(ProcessAbiOperation::GetParentPid, &[])
            .is_err()
    );
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::GetPid, &[]).unwrap()),
        current.abi_value().unwrap() as i128
    );
    let child = fixture.world.fork(current).unwrap().child;
    let unrelated = fixture.world.create_root().unwrap();
    assert!(fixture.branch.for_child(unrelated, &fixture.world).is_err());
    fixture.branch = fixture.branch.for_child(child, &fixture.world).unwrap();
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::GetPid, &[]).unwrap()),
        child.abi_value().unwrap() as i128
    );
    assert_eq!(
        scalar(
            fixture
                .call(ProcessAbiOperation::GetParentPid, &[])
                .unwrap()
        ),
        current.abi_value().unwrap() as i128
    );
}

#[test]
fn failed_read_does_not_consume_data_and_budget_does_not_become_errno() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let input = fixture.bytes(b"abc");
    let output = fixture.bytes(b"...");
    fixture
        .call(
            ProcessAbiOperation::Write,
            &[s32(writer), Value::Pointer(input), u64(3)],
        )
        .unwrap();
    fixture.memory.freeze(&output).unwrap();
    assert_eq!(
        fixture
            .call(
                ProcessAbiOperation::Read,
                &[s32(reader), Value::Pointer(output), u64(3)]
            )
            .unwrap_err(),
        Error::ReadOnlyStorage
    );
    assert_eq!(
        fixture
            .world
            .readiness(
                fixture
                    .world
                    .descriptor(fixture.branch.current(), reader)
                    .unwrap()
            )
            .unwrap()
            .bytes,
        3
    );
    let valid = fixture.bytes(b"...");
    assert!(matches!(
        fixture.call(
            ProcessAbiOperation::Read,
            &[
                s32(reader),
                Value::Pointer(valid),
                u64(fixture.world.limits().transfer_bytes + 1)
            ]
        ),
        Err(Error::EffectRejected(_))
    ));
    assert!(fixture.branch.errno.is_empty());
}

#[test]
fn zero_count_read_and_write_do_not_touch_null_buffers_or_block() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let null = Value::Pointer(Pointer::null(fixture.types.void()));
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Read,
                    &[s32(reader), null.clone(), u64(0)]
                )
                .unwrap()
        ),
        0
    );
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Write,
                    &[s32(writer), null.clone(), u64(0)]
                )
                .unwrap()
        ),
        0
    );
    fixture
        .call(ProcessAbiOperation::Close, &[s32(reader)])
        .unwrap();
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Write,
                    &[s32(writer), null.clone(), u64(0)]
                )
                .unwrap()
        ),
        0
    );
    assert!(fixture.branch.errno.is_empty());
    assert_eq!(
        scalar(
            fixture
                .call(ProcessAbiOperation::Read, &[s32(-1), null, u64(0)])
                .unwrap()
        ),
        -1
    );
    assert_eq!(fixture.errno().1, 9);
}

#[test]
fn partial_read_initializes_only_received_bytes_in_uninitialized_source_buffer() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let input = fixture.bytes(b"ab");
    let byte = fixture.types.scalar(ScalarType::Int(IntegerType::U8));
    let array = fixture.types.fixed_array(byte, 4).unwrap();
    let root = fixture
        .memory
        .allocate(&fixture.types, array, None)
        .unwrap();
    let output = fixture
        .memory
        .cast_pointer(
            &fixture.types,
            &root,
            fixture.types.void(),
            CastMode::Unchecked,
        )
        .unwrap();
    fixture
        .call(
            ProcessAbiOperation::Write,
            &[s32(writer), Value::Pointer(input), u64(2)],
        )
        .unwrap();
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Read,
                    &[s32(reader), Value::Pointer(output.clone()), u64(4)]
                )
                .unwrap()
        ),
        2
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 2)
            .unwrap(),
        b"ab"
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 4)
            .unwrap_err(),
        Error::Uninitialized
    );
}

#[test]
fn signed_or_address_derived_counts_never_become_errno_or_transport_mutation() {
    let mut fixture = Fixture::new();
    let [reader, writer] = fixture.pipe();
    let buffer = fixture.bytes(b"x");
    let tagged = Value::AddressInteger(Number::address(
        Integer::checked(IntegerType::U64, 1).unwrap(),
        crate::AddressProvenance::Pointer(buffer.clone()),
    ));
    assert!(matches!(
        fixture.call(
            ProcessAbiOperation::Write,
            &[s32(writer), Value::Pointer(buffer.clone()), tagged]
        ),
        Err(Error::UnsupportedPointerOperation(_))
    ));
    assert!(matches!(
        fixture.call(
            ProcessAbiOperation::Read,
            &[
                s32(reader),
                Value::Pointer(buffer),
                int(IntegerType::S64, -1)
            ]
        ),
        Err(Error::TypeMismatch { .. })
    ));
    assert!(fixture.branch.errno.is_empty());
    assert_eq!(fixture.world.retained_bytes(), 0);
}

#[test]
fn exit_emits_control_without_reaping_or_running_source_cleanup() {
    let mut fixture = Fixture::new();
    let current = fixture.branch.current();
    let ProcessCallOutcome::ExitControl(control) =
        fixture.call(ProcessAbiOperation::Exit, &[s32(-1)]).unwrap()
    else {
        panic!("_exit requires control replacement")
    };
    assert_eq!(control.procedure().operation(), ProcessAbiOperation::Exit);
    assert_eq!(control.process(), current);
    assert_eq!(control.status(), -1);
    fixture.world.validate_running(current).unwrap();
    assert_eq!(
        encode_wait_status(ProcessTermination::Exited(255)).unwrap(),
        0xff00
    );
    assert!(encode_wait_status(ProcessTermination::Signaled).is_err());
    assert!(encode_wait_status(ProcessTermination::TimedOut).is_err());
}

#[test]
fn waitpid_pending_nohang_status_encoding_null_status_and_single_reap_are_actual() {
    let mut fixture = Fixture::new();
    let parent = fixture.branch.current();
    let child = fixture.world.fork(parent).unwrap().child;
    let pid = child.abi_value().unwrap();
    let s32_type = fixture.types.scalar(ScalarType::Int(IntegerType::S32));
    let status = fixture
        .memory
        .allocate(&fixture.types, s32_type, Some(s32(123)))
        .unwrap();
    let args = [s32(pid), Value::Pointer(status.clone()), s32(0)];
    let ProcessCallOutcome::Pending(event) =
        fixture.call(ProcessAbiOperation::WaitPid, &args).unwrap()
    else {
        panic!("live child must block real wait")
    };
    assert!(!fixture.world.event_ready(event).unwrap());
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::WaitPid,
                    &[s32(pid), Value::Pointer(status.clone()), s32(1)]
                )
                .unwrap()
        ),
        0
    );
    assert_eq!(
        fixture.memory.load(&fixture.types, &status).unwrap(),
        s32(123)
    );
    fixture.world.exit(child, -1).unwrap();
    assert!(fixture.world.event_ready(event).unwrap());
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::WaitPid, &args).unwrap()),
        pid as i128
    );
    assert_eq!(
        fixture.memory.load(&fixture.types, &status).unwrap(),
        s32(0xff00)
    );
    assert_eq!(
        scalar(fixture.call(ProcessAbiOperation::WaitPid, &args).unwrap()),
        -1
    );
    assert_eq!(fixture.errno().1, 10);
    let child = fixture.world.fork(parent).unwrap().child;
    fixture.world.exit(child, 7).unwrap();
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::WaitPid,
                    &[
                        s32(child.abi_value().unwrap()),
                        Value::Pointer(Pointer::null(s32_type)),
                        s32(0)
                    ]
                )
                .unwrap()
        ),
        child.abi_value().unwrap() as i128
    );
}

#[test]
fn waitpid_memory_or_signal_failure_cannot_reap_or_publish_status() {
    let mut fixture = Fixture::new();
    let parent = fixture.branch.current();
    let child = fixture.world.fork(parent).unwrap().child;
    let s32_type = fixture.types.scalar(ScalarType::Int(IntegerType::S32));
    let status = fixture
        .memory
        .allocate(&fixture.types, s32_type, Some(s32(321)))
        .unwrap();
    fixture.memory.freeze(&status).unwrap();
    fixture.world.exit(child, 2).unwrap();
    let args = [
        s32(child.abi_value().unwrap()),
        Value::Pointer(status.clone()),
        s32(0),
    ];
    assert_eq!(
        fixture
            .call(ProcessAbiOperation::WaitPid, &args)
            .unwrap_err(),
        Error::ReadOnlyStorage
    );
    assert_eq!(
        fixture.world.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(2))
    );
    let child = fixture.world.fork(parent).unwrap().child;
    let key = HostRequestKey::allocate();
    struct Deferred(HostRequestKey);
    impl crate::host_effects::HostEffects for Deferred {
        fn begin(&mut self, _: crate::SourceOrigin) -> Result<(), crate::host_effects::HostError> {
            Ok(())
        }
        fn request(
            &mut self,
            _: crate::host_effects::HostRequest,
        ) -> crate::host_effects::HostOutcome {
            crate::host_effects::HostOutcome::Pending(self.0)
        }
        fn finish(&mut self, _: bool) -> Result<(), crate::host_effects::HostError> {
            Ok(())
        }
    }
    let invocation = crate::host_effects::ProgramInvocation {
        program: crate::host_effects::ProgramId::allocate(),
        arguments: crate::host_effects::ProcessArguments::new([]).unwrap(),
        working_root: crate::host_effects::FileRootId::allocate(),
    };
    fixture
        .world
        .exec(child, invocation, &mut Deferred(key))
        .unwrap();
    fixture
        .world
        .complete_exec(
            child,
            key,
            crate::host_effects::HostOutcome::Ready(crate::host_effects::HostResponse::Process(
                crate::host_effects::ProcessOutput {
                    termination: ProcessTermination::Signaled,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                },
            )),
        )
        .unwrap();
    let mutable = fixture
        .memory
        .allocate(&fixture.types, s32_type, Some(s32(321)))
        .unwrap();
    assert!(
        fixture
            .call(
                ProcessAbiOperation::WaitPid,
                &[
                    s32(child.abi_value().unwrap()),
                    Value::Pointer(mutable.clone()),
                    s32(0)
                ]
            )
            .is_err()
    );
    assert_eq!(
        fixture.memory.load(&fixture.types, &mutable).unwrap(),
        s32(321)
    );
    assert_eq!(
        fixture.world.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Signaled)
    );
}

#[path = "exec_tests.rs"]
mod exec_tests;

#[path = "socket_tests.rs"]
mod socket_tests;

#[path = "flags_tests.rs"]
mod flags_tests;
