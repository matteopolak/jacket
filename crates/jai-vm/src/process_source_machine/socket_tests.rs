use super::*;
use crate::process_abi::ProcessSocketTypes;
use jai_types::RecordKind;

fn en(ty: TypeId, repr: IntegerType, bits: i128) -> Value {
    Value::Enum {
        ty,
        value: Integer::checked(repr, bits).unwrap(),
    }
}
impl Fixture {
    fn socket_types(&self) -> ProcessSocketTypes {
        self.proof(ProcessAbiOperation::SocketPair)
            .nominals()
            .socket
            .unwrap()
    }
    fn socket_pair(&mut self) -> [i32; 2] {
        let socket = self.socket_types();
        let buffer = self.pair_buffer();
        assert_eq!(
            scalar(
                self.call(
                    ProcessAbiOperation::SocketPair,
                    &[
                        s32(1),
                        en(socket.socket_kind, IntegerType::U32, 1),
                        en(socket.protocol, IntegerType::U32, 0),
                        Value::Pointer(buffer.clone())
                    ]
                )
                .unwrap()
            ),
            0
        );
        let value = self.memory.load(&self.types, &buffer).unwrap();
        let Value::Array { elements, .. } = value.semantic() else {
            panic!("socket pair array")
        };
        [
            fd_number(&elements[0]).unwrap(),
            fd_number(&elements[1]).unwrap(),
        ]
    }
    fn socket_length(&self) -> IntegerType {
        if self.target.operating_system == OperatingSystem::MacOS {
            IntegerType::U32
        } else {
            IntegerType::U64
        }
    }
    fn control(&mut self, descriptors: &[i32]) -> (Pointer, usize) {
        let socket = self.socket_types();
        let s32_type = self.types.scalar(ScalarType::Int(IntegerType::S32));
        let array = self
            .types
            .fixed_array(s32_type, descriptors.len() as u64)
            .unwrap();
        let control = self.types.reserve_record(RecordKind::Struct);
        self.types
            .define_record(control, [socket.control_header, array])
            .unwrap();
        let mac = self.target.operating_system == OperatingSystem::MacOS;
        let header_bytes = if mac { 12 } else { 16 };
        let alignment = if mac { 4 } else { 8 };
        let length = header_bytes + 4 * descriptors.len();
        let capacity = (length + alignment - 1) & !(alignment - 1);
        let root = self
            .memory
            .allocate(
                &self.types,
                control,
                Some(Value::Record {
                    ty: control,
                    fields: vec![
                        Value::Record {
                            ty: socket.control_header,
                            fields: vec![
                                int(self.socket_length(), length as i128),
                                s32(if mac { 0xffff } else { 1 }),
                                s32(1),
                            ],
                        },
                        Value::Array {
                            ty: array,
                            elements: descriptors.iter().map(|fd| s32(*fd)).collect(),
                        },
                    ],
                }),
            )
            .unwrap();
        let void = self
            .memory
            .cast_pointer(&self.types, &root, self.types.void(), CastMode::Unchecked)
            .unwrap();
        (void, capacity)
    }
    fn message(
        &mut self,
        vectors: &[(Pointer, usize)],
        control: Pointer,
        capacity: usize,
    ) -> Pointer {
        let socket = self.socket_types();
        let void = Pointer::null(self.types.void());
        let array = self
            .types
            .fixed_array(socket.io_vector, vectors.len() as u64)
            .unwrap();
        let root = self
            .memory
            .allocate(
                &self.types,
                array,
                Some(Value::Array {
                    ty: array,
                    elements: vectors
                        .iter()
                        .map(|(pointer, count)| Value::Record {
                            ty: socket.io_vector,
                            fields: vec![Value::Pointer(pointer.clone()), u64(*count)],
                        })
                        .collect(),
                }),
            )
            .unwrap();
        let data = self.memory.sequence_data(&self.types, &root).unwrap();
        self.memory
            .allocate(
                &self.types,
                socket.message_header,
                Some(Value::Record {
                    ty: socket.message_header,
                    fields: vec![
                        Value::Pointer(void),
                        int(IntegerType::U32, 0),
                        Value::Pointer(data),
                        int(
                            if self.target.operating_system == OperatingSystem::MacOS {
                                IntegerType::S32
                            } else {
                                IntegerType::U64
                            },
                            vectors.len() as i128,
                        ),
                        Value::Pointer(control),
                        int(self.socket_length(), capacity as i128),
                        s32(123),
                    ],
                }),
            )
            .unwrap()
    }
    fn socket_call(
        &mut self,
        op: ProcessAbiOperation,
        fd: i32,
        header: Pointer,
    ) -> Result<ProcessCallOutcome, Error> {
        let socket = self.socket_types();
        self.call(
            op,
            &[
                s32(fd),
                Value::Pointer(header),
                en(socket.message_flags, IntegerType::S32, 0),
            ],
        )
    }
    fn message_field(&self, header: &Pointer, index: usize) -> Value {
        self.memory
            .load(
                &self.types,
                &self.memory.field(&self.types, header, index).unwrap(),
            )
            .unwrap()
    }
    fn received_fds(&self, control: &Pointer, count: usize) -> Vec<i32> {
        let ty = self.types.scalar(ScalarType::Int(IntegerType::S32));
        let offset = if self.target.operating_system == OperatingSystem::MacOS {
            12
        } else {
            16
        };
        (0..count)
            .map(|index| {
                let byte = self.types.scalar(ScalarType::Int(IntegerType::U8));
                let pointer = self
                    .memory
                    .cast_pointer(&self.types, control, byte, CastMode::Unchecked)
                    .unwrap();
                let pointer = self
                    .memory
                    .offset(&self.types, &pointer, (offset + index * 4) as isize)
                    .unwrap();
                let pointer = self
                    .memory
                    .cast_pointer(&self.types, &pointer, ty, CastMode::Unchecked)
                    .unwrap();
                fd_number(&self.memory.load(&self.types, &pointer).unwrap()).unwrap()
            })
            .collect()
    }
    fn output_buffer(&mut self, count: usize) -> Pointer {
        let byte = self.types.scalar(ScalarType::Int(IntegerType::U8));
        let array = self.types.fixed_array(byte, count as u64).unwrap();
        let pointer = self.memory.allocate(&self.types, array, None).unwrap();
        self.memory
            .cast_pointer(
                &self.types,
                &pointer,
                self.types.void(),
                CastMode::Unchecked,
            )
            .unwrap()
    }
}

#[test]
fn inspected_macos_and_linux_source_socket_protocol_transfers_three_live_pipes_and_ack() {
    for os in [OperatingSystem::MacOS, OperatingSystem::Linux] {
        let mut fixture = Fixture::new_for(os.clone());
        let parent = fixture.branch.current();
        let sockets = fixture.socket_pair();
        let parent_branch = fixture.branch.clone();
        let child = fixture.world.fork(parent).unwrap().child;
        let child_memory = fixture.memory.fork_private_branch(usize::MAX).unwrap();
        let parent_memory = std::mem::replace(&mut fixture.memory, child_memory);
        fixture.branch = parent_branch.for_child(child, &fixture.world).unwrap();
        assert_eq!(
            scalar(
                fixture
                    .call(ProcessAbiOperation::Close, &[s32(sockets[0])])
                    .unwrap()
            ),
            0
        );
        let pipes = [fixture.pipe(), fixture.pipe(), fixture.pipe()];
        let rights: Vec<_> = pipes.iter().map(|pipe| pipe[0]).collect();
        let payload = fixture.bytes(&[123]);
        let (control, capacity) = fixture.control(&rights);
        let send = fixture.message(&[(payload, 1)], control, capacity);
        assert_eq!(
            scalar(
                fixture
                    .socket_call(ProcessAbiOperation::SendMsg, sockets[1], send)
                    .unwrap()
            ),
            1
        );
        let ack = fixture.output_buffer(1);
        assert!(matches!(
            fixture
                .call(
                    ProcessAbiOperation::Read,
                    &[s32(sockets[1]), Value::Pointer(ack.clone()), u64(1)]
                )
                .unwrap(),
            ProcessCallOutcome::Pending(_)
        ));
        let child_memory = std::mem::replace(&mut fixture.memory, parent_memory);
        fixture.branch = parent_branch.clone();
        fixture
            .call(ProcessAbiOperation::Close, &[s32(sockets[1])])
            .unwrap();
        let output = fixture.output_buffer(4);
        let (control, capacity) = fixture.control(&[-1, -1, -1]);
        let recv = fixture.message(&[(output.clone(), 4)], control.clone(), capacity);
        assert_eq!(
            scalar(
                fixture
                    .socket_call(ProcessAbiOperation::RecvMsg, sockets[0], recv.clone())
                    .unwrap()
            ),
            1
        );
        assert_eq!(
            fixture
                .memory
                .host_read_bytes(&fixture.types, &output, 1)
                .unwrap(),
            [123]
        );
        assert_eq!(fixture.message_field(&recv, 6), s32(0));
        let received = fixture.received_fds(&control, 3);
        assert_ne!(received, rights);
        let ack_bytes = fixture.bytes(&[42]);
        assert_eq!(
            scalar(
                fixture
                    .call(
                        ProcessAbiOperation::Write,
                        &[s32(sockets[0]), Value::Pointer(ack_bytes), u64(1)]
                    )
                    .unwrap()
            ),
            1
        );
        let parent_memory = std::mem::replace(&mut fixture.memory, child_memory);
        fixture.branch = parent_branch.for_child(child, &fixture.world).unwrap();
        assert_eq!(
            scalar(
                fixture
                    .call(
                        ProcessAbiOperation::Read,
                        &[s32(sockets[1]), Value::Pointer(ack.clone()), u64(1)]
                    )
                    .unwrap()
            ),
            1
        );
        assert_eq!(
            fixture
                .memory
                .host_read_bytes(&fixture.types, &ack, 1)
                .unwrap(),
            [42]
        );
        for fd in &rights {
            fixture
                .call(ProcessAbiOperation::Close, &[s32(*fd)])
                .unwrap();
        }
        let bytes = fixture.bytes(b"child stdout");
        fixture
            .call(
                ProcessAbiOperation::Write,
                &[s32(pipes[1][1]), Value::Pointer(bytes), u64(12)],
            )
            .unwrap();
        fixture.world.exit(child, 0).unwrap();
        fixture.memory = parent_memory;
        fixture.branch = parent_branch;
        let bytes = fixture.output_buffer(12);
        assert_eq!(
            scalar(
                fixture
                    .call(
                        ProcessAbiOperation::Read,
                        &[s32(received[1]), Value::Pointer(bytes.clone()), u64(12)]
                    )
                    .unwrap()
            ),
            12
        );
        assert_eq!(
            fixture
                .memory
                .host_read_bytes(&fixture.types, &bytes, 12)
                .unwrap(),
            b"child stdout"
        );
        assert_eq!(
            scalar(
                fixture
                    .call(
                        ProcessAbiOperation::Read,
                        &[s32(received[1]), Value::Pointer(bytes), u64(1)]
                    )
                    .unwrap()
            ),
            0
        );
        for fd in received.into_iter().chain([sockets[0]]) {
            fixture
                .call(ProcessAbiOperation::Close, &[s32(fd)])
                .unwrap();
        }
        assert_eq!(
            fixture.world.wait(parent, child, false).unwrap(),
            WaitOutcome::Reaped(ProcessTermination::Exited(0))
        );
        fixture.world.require_quiescent(parent).unwrap();
    }
}

#[test]
fn partial_stream_recv_transfers_rights_once_and_initializes_only_received_payload() {
    let mut fixture = Fixture::new();
    let sockets = fixture.socket_pair();
    let pipe = fixture.pipe();
    let bytes = fixture.bytes(b"abc");
    let (control, capacity) = fixture.control(&[pipe[0]]);
    let send = fixture.message(&[(bytes, 3)], control, capacity);
    fixture
        .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
        .unwrap();
    fixture
        .call(ProcessAbiOperation::Close, &[s32(pipe[0])])
        .unwrap();
    fixture
        .call(ProcessAbiOperation::Close, &[s32(pipe[1])])
        .unwrap();
    let first = fixture.output_buffer(3);
    let (control, capacity) = fixture.control(&[-1]);
    let recv = fixture.message(&[(first.clone(), 1)], control.clone(), capacity);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv)
                .unwrap()
        ),
        1
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &first, 1)
            .unwrap(),
        b"a"
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &first, 3)
            .unwrap_err(),
        Error::Uninitialized
    );
    let received = fixture.received_fds(&control, 1)[0];
    let trailing = fixture.output_buffer(2);
    let recv = fixture.message(&[(trailing.clone(), 2)], control.clone(), capacity);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
                .unwrap()
        ),
        2
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &trailing, 2)
            .unwrap(),
        b"bc"
    );
    assert_eq!(fixture.message_field(&recv, 5), int(IntegerType::U32, 0));
    assert_eq!(fixture.received_fds(&control, 1), [received]);
    assert_eq!(fixture.world.retained_bytes(), 0);
    assert_eq!(
        scalar(
            fixture
                .call(
                    ProcessAbiOperation::Read,
                    &[
                        s32(received),
                        Value::Pointer(Pointer::null(fixture.types.void())),
                        u64(0)
                    ]
                )
                .unwrap()
        ),
        0
    );
}

#[test]
fn control_truncation_installs_only_fitting_rights_and_closes_discarded_references() {
    for os in [OperatingSystem::MacOS, OperatingSystem::Linux] {
        let mut fixture = Fixture::new_for(os.clone());
        let root = fixture.branch.current();
        let sockets = fixture.socket_pair();
        let pipes = [fixture.pipe(), fixture.pipe()];
        let data = fixture.bytes(&[1]);
        let (control, capacity) = fixture.control(&[pipes[0][0], pipes[1][0]]);
        let send = fixture.message(&[(data, 1)], control, capacity);
        fixture
            .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
            .unwrap();
        for fd in pipes.into_iter().flatten() {
            fixture
                .call(ProcessAbiOperation::Close, &[s32(fd)])
                .unwrap();
        }
        let output = fixture.output_buffer(1);
        let (control, _) = fixture.control(&[-1]);
        let capacity = if os == OperatingSystem::MacOS { 16 } else { 20 };
        let recv = fixture.message(&[(output, 1)], control.clone(), capacity);
        assert_eq!(
            scalar(
                fixture
                    .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
                    .unwrap()
            ),
            1
        );
        assert_eq!(
            fixture.message_field(&recv, 6),
            s32(if os == OperatingSystem::MacOS {
                0x20
            } else {
                0x8
            })
        );
        assert_eq!(
            fixture.message_field(&recv, 5),
            int(fixture.socket_length(), capacity as i128)
        );
        let fd = fixture.received_fds(&control, 1)[0];
        for fd in [fd, sockets[0], sockets[1]] {
            fixture
                .call(ProcessAbiOperation::Close, &[s32(fd)])
                .unwrap();
        }
        fixture.world.require_quiescent(root).unwrap();
    }
}

#[test]
fn inherited_socket_shutdown_wr_forces_eof_and_zero_length_buffers_are_not_accessed() {
    let mut fixture = Fixture::new();
    let root = fixture.branch.current();
    let sockets = fixture.socket_pair();
    let inherited = fixture.world.fork(root).unwrap().child;
    let socket = fixture.socket_types();
    fixture
        .call(
            ProcessAbiOperation::Shutdown,
            &[
                s32(sockets[0]),
                en(socket.shutdown_kind, IntegerType::U32, 1),
            ],
        )
        .unwrap();
    let null = Pointer::null(fixture.types.void());
    let recv = fixture.message(&[(null.clone(), 0)], null.clone(), 0);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv)
                .unwrap()
        ),
        0
    );
    let output = fixture.output_buffer(1);
    let recv = fixture.message(&[(output, 1)], null.clone(), 0);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv)
                .unwrap()
        ),
        0
    );
    let bytes = fixture.bytes(&[1]);
    let send = fixture.message(&[(bytes, 1)], null, 0);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
                .unwrap()
        ),
        -1
    );
    assert_eq!(fixture.errno().1, 32);
    fixture.world.exit(inherited, 0).unwrap();
}

#[test]
fn pending_recv_readonly_destinations_and_late_fuel_failure_preserve_memory_and_rights() {
    let mut fixture = Fixture::new();
    let sockets = fixture.socket_pair();
    let pipe = fixture.pipe();
    let output = fixture.output_buffer(2);
    let (control, capacity) = fixture.control(&[-1]);
    let recv = fixture.message(&[(output.clone(), 2)], control.clone(), capacity);
    assert!(matches!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
            .unwrap(),
        ProcessCallOutcome::Pending(_)
    ));
    let cells = fixture.memory.value_cells();
    assert!(matches!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
            .unwrap(),
        ProcessCallOutcome::Pending(_)
    ));
    assert_eq!(fixture.memory.value_cells(), cells);
    assert_eq!(fixture.message_field(&recv, 6), s32(123));
    let payload = fixture.bytes(b"x");
    let (send_control, send_capacity) = fixture.control(&[pipe[0]]);
    let send = fixture.message(&[(payload, 1)], send_control, send_capacity);
    fixture
        .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
        .unwrap();
    let readonly = fixture.output_buffer(1);
    fixture.memory.freeze(&readonly).unwrap();
    let readonly_recv = fixture.message(
        &[(output.clone(), 1), (readonly, 1)],
        control.clone(),
        capacity,
    );
    assert_eq!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], readonly_recv)
            .unwrap_err(),
        Error::ReadOnlyStorage
    );
    assert_eq!(fixture.world.retained_bytes(), 1);
    let snapshot_cost = (fixture.memory.snapshot_work_cost().unwrap() as u64) * 2;
    let mut copied = false;
    let proof = fixture.proof(ProcessAbiOperation::RecvMsg);
    let flags = fixture.socket_types().message_flags;
    let result = proof.invoke_process(
        &[
            s32(sockets[1]),
            Value::Pointer(recv.clone()),
            en(flags, IntegerType::S32, 0),
        ],
        &mut fixture.memory,
        &fixture.types,
        &fixture.target,
        &mut fixture.branch,
        &mut fixture.world,
        &mut |cost| {
            if copied && cost == 128 {
                return Err(Error::Limit(LimitKind::Fuel));
            }
            if cost == snapshot_cost {
                copied = true;
            }
            Ok(())
        },
    );
    assert!(
        copied,
        "regression must reject after the private memory candidate was admitted"
    );
    assert_eq!(result.unwrap_err(), Error::Limit(LimitKind::Fuel));
    assert_eq!(fixture.world.retained_bytes(), 1);
    assert_eq!(fixture.message_field(&recv, 6), s32(123));
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 1)
            .unwrap_err(),
        Error::Uninitialized
    );
    assert!(fixture.branch.errno.is_empty());
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv)
                .unwrap()
        ),
        1
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 1)
            .unwrap(),
        b"x"
    );
    assert!(
        fixture
            .world
            .descriptor(
                fixture.branch.current(),
                fixture.received_fds(&control, 1)[0]
            )
            .is_ok()
    );
}

#[test]
fn recv_transient_memory_and_world_retention_is_admitted_before_private_copy() {
    let mut fixture = Fixture::new();
    fixture.memory = Memory::new(Limits {
        value_cells: 1024,
        ..Default::default()
    });
    let sockets = fixture.socket_pair();
    let payload = fixture.bytes(b"x");
    let null = Pointer::null(fixture.types.void());
    let send = fixture.message(&[(payload, 1)], null.clone(), 0);
    fixture
        .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
        .unwrap();
    let output = fixture.bytes(&vec![0; 512]);
    let recv = fixture.message(&[(output.clone(), 512)], null, 0);
    let before = fixture.memory.value_cells();
    assert!(before < fixture.memory.value_cell_limit());
    assert!(
        before * 2 + fixture.world.work_cost().unwrap() as usize * 2
            > fixture.memory.value_cell_limit()
    );
    assert_eq!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
            .unwrap_err(),
        Error::Limit(LimitKind::ValueCells)
    );
    assert_eq!(fixture.world.retained_bytes(), 1);
    assert_eq!(fixture.memory.value_cells(), before);
    assert_eq!(fixture.message_field(&recv, 6), s32(123));
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 1)
            .unwrap(),
        [0]
    );
    assert!(fixture.branch.errno.is_empty());
}

#[test]
fn socket_pointer_provenance_malformed_ancillary_and_unknown_rights_do_not_send() {
    let mut fixture = Fixture::new();
    let sockets = fixture.socket_pair();
    let byte = fixture.types.scalar(ScalarType::Int(IntegerType::U8));
    let pointer_type = fixture.types.pointer(byte).unwrap();
    let target = fixture.bytes(b"pointed data");
    let target = fixture
        .memory
        .cast_pointer(&fixture.types, &target, byte, CastMode::Unchecked)
        .unwrap();
    let pointer = fixture
        .memory
        .allocate(&fixture.types, pointer_type, Some(Value::Pointer(target)))
        .unwrap();
    let pointer = fixture
        .memory
        .cast_pointer(
            &fixture.types,
            &pointer,
            fixture.types.void(),
            CastMode::Unchecked,
        )
        .unwrap();
    let null = Pointer::null(fixture.types.void());
    let send = fixture.message(&[(pointer, 8)], null, 0);
    assert!(
        fixture
            .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
            .is_err()
    );
    assert_eq!(fixture.world.retained_bytes(), 0);
    let bytes = fixture.bytes(b"x");
    let (control, capacity) = fixture.control(&[-1]);
    let send = fixture.message(&[(bytes.clone(), 1)], control.clone(), capacity);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
                .unwrap()
        ),
        -1
    );
    assert_eq!(fixture.errno().1, 9);
    let header = fixture
        .memory
        .cast_pointer(
            &fixture.types,
            &control,
            fixture.socket_types().control_header,
            CastMode::Unchecked,
        )
        .unwrap();
    let length = fixture.memory.field(&fixture.types, &header, 0).unwrap();
    fixture
        .memory
        .store(&fixture.types, &length, int(IntegerType::U32, 13))
        .unwrap();
    let send = fixture.message(&[(bytes, 1)], control, capacity);
    assert!(
        fixture
            .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
            .is_err()
    );
    assert_eq!(fixture.errno().1, 9);
    assert_eq!(fixture.world.retained_bytes(), 0);
}

#[test]
fn scatter_gather_copies_complete_stream_counts_without_initializing_trailing_destinations() {
    let mut fixture = Fixture::new();
    let sockets = fixture.socket_pair();
    let first = fixture.bytes(b"ab");
    let second = fixture.bytes(b"cd");
    let null = Pointer::null(fixture.types.void());
    let send = fixture.message(&[(first, 2), (second, 2)], null.clone(), 0);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
                .unwrap()
        ),
        4
    );
    let first = fixture.output_buffer(1);
    let second = fixture.output_buffer(5);
    let recv = fixture.message(&[(first.clone(), 1), (second.clone(), 5)], null, 0);
    assert_eq!(
        scalar(
            fixture
                .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv)
                .unwrap()
        ),
        4
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &first, 1)
            .unwrap(),
        b"a"
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &second, 3)
            .unwrap(),
        b"bcd"
    );
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &second, 5)
            .unwrap_err(),
        Error::Uninitialized
    );
}

#[test]
fn rights_count_budget_and_wrong_target_control_layout_never_create_transport_data() {
    let mut fixture = Fixture::new();
    fixture.world = VirtualProcesses::new(crate::virtual_process::ProcessLimits {
        transferred_descriptors: 1,
        ..Default::default()
    });
    fixture.branch = ProcessBranchState::new(fixture.world.create_root().unwrap());
    let sockets = fixture.socket_pair();
    let pipes = [fixture.pipe(), fixture.pipe()];
    let bytes = fixture.bytes(b"x");
    let (control, capacity) = fixture.control(&[pipes[0][0], pipes[1][0]]);
    let send = fixture.message(&[(bytes, 1)], control, capacity);
    assert!(
        fixture
            .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send.clone())
            .is_err()
    );
    assert_eq!(fixture.world.retained_bytes(), 0);
    assert!(fixture.branch.errno.is_empty());
    let original = fixture.proof(ProcessAbiOperation::SendMsg);
    let mut nominals = original.nominals();
    let wrong = fixture.types.reserve_record(RecordKind::Struct);
    fixture
        .types
        .define_record(
            wrong,
            [
                fixture.types.scalar(ScalarType::Int(IntegerType::U64)),
                fixture.types.scalar(ScalarType::Int(IntegerType::S32)),
                fixture.types.scalar(ScalarType::Int(IntegerType::S32)),
            ],
        )
        .unwrap();
    nominals.socket.as_mut().unwrap().control_header = wrong;
    let library = ForeignLibrary {
        id: original.library(),
        kind: ForeignLibraryKind::System {
            name: "libc".into(),
        },
        options: Default::default(),
    };
    let authority = crate::process_abi::ProcessAuthority::from_verified_source(
        fixture.target.clone(),
        library.clone(),
        nominals,
        [(
            original.procedure(),
            ProcessAbiOperation::SendMsg,
            original.signature(),
        )],
        &fixture.types,
    )
    .unwrap();
    let proof = authority
        .bind(
            &ProcedurePrototype {
                id: original.procedure(),
                signature: original.signature(),
                origin: PrototypeOrigin::Foreign {
                    symbol: "sendmsg".into(),
                    library: Some(library),
                },
            },
            &fixture.target,
            &fixture.types,
        )
        .unwrap();
    let flags = fixture.socket_types().message_flags;
    assert!(matches!(
        proof.invoke_process(
            &[
                s32(sockets[0]),
                Value::Pointer(send),
                en(flags, IntegerType::S32, 0)
            ],
            &mut fixture.memory,
            &fixture.types,
            &fixture.target,
            &mut fixture.branch,
            &mut fixture.world,
            &mut |_| Ok(())
        ),
        Err(Error::InvalidIr(_))
    ));
    assert_eq!(fixture.world.retained_bytes(), 0);
}

#[test]
fn uninitialized_receive_root_materialization_is_admitted_before_any_byte_image_copy() {
    let mut fixture = Fixture::new();
    fixture.memory = Memory::new(Limits {
        value_cells: 1024,
        ..Default::default()
    });
    let sockets = fixture.socket_pair();
    let output = fixture.output_buffer(920);
    let null = Pointer::null(fixture.types.void());
    let recv = fixture.message(&[(output.clone(), 920)], null.clone(), 0);
    // Warm bounded typed layout metadata while the actual transport is empty.
    assert!(matches!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
            .unwrap(),
        ProcessCallOutcome::Pending(_)
    ));
    let payload = fixture.bytes(b"x");
    let send = fixture.message(&[(payload, 1)], null, 0);
    fixture
        .socket_call(ProcessAbiOperation::SendMsg, sockets[0], send)
        .unwrap();
    let before = fixture.memory.value_cells();
    assert!(
        before * 2 + fixture.world.work_cost().unwrap() as usize * 2
            < fixture.memory.value_cell_limit(),
        "old retained memory alone must fit"
    );
    assert_eq!(
        fixture
            .socket_call(ProcessAbiOperation::RecvMsg, sockets[1], recv.clone())
            .unwrap_err(),
        Error::Limit(LimitKind::ValueCells)
    );
    assert_eq!(fixture.world.retained_bytes(), 1);
    assert_eq!(fixture.memory.value_cells(), before);
    assert_eq!(fixture.message_field(&recv, 6), s32(123));
    assert_eq!(
        fixture
            .memory
            .host_read_bytes(&fixture.types, &output, 1)
            .unwrap_err(),
        Error::Uninitialized
    );
    assert!(fixture.branch.errno.is_empty());
}
