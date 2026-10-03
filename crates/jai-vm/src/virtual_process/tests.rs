use super::*;
use crate::{
    SourceOrigin,
    host_effects::{FileRootId, ProcessArguments, ProgramId},
};

fn ledger() -> (VirtualProcesses, ProcessId) {
    let mut state = VirtualProcesses::new(ProcessLimits::default());
    let root = state.create_root().unwrap();
    (state, root)
}
fn bytes(outcome: ProcessIo<Vec<u8>>) -> Vec<u8> {
    match outcome {
        ProcessIo::Ready(bytes) => bytes,
        ProcessIo::Pending(_) => panic!("unexpected pending read"),
    }
}
fn message(outcome: ProcessIo<ReceivedMessage>) -> ReceivedMessage {
    match outcome {
        ProcessIo::Ready(message) => message,
        ProcessIo::Pending(_) => panic!("unexpected pending message"),
    }
}

#[test]
fn fork_copies_slots_but_shared_writers_produce_real_eof() {
    let (mut state, parent) = ledger();
    let [reader, writer] = state.pipe(parent).unwrap();
    assert_eq!(reader.number(), 3);
    let child = state.fork(parent).unwrap().child;
    let child_writer = state.descriptor(child, writer.number()).unwrap();
    state.close(writer).unwrap();
    let ProcessIo::Pending(event) = state.read(reader, 2).unwrap() else {
        panic!("inherited writer must prevent EOF")
    };
    assert!(!state.event_ready(event).unwrap());
    state.write(child_writer, b"abc").unwrap();
    assert!(state.event_ready(event).unwrap());
    assert_eq!(bytes(state.read(reader, 2).unwrap()), b"ab");
    state.close(child_writer).unwrap();
    assert_eq!(bytes(state.read(reader, 2).unwrap()), b"c");
    assert!(bytes(state.read(reader, 2).unwrap()).is_empty());
    state.exit(child, 7).unwrap();
    assert_eq!(
        state.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(7))
    );
    assert_eq!(
        state.wait(parent, child, false),
        Err(ProcessError::AlreadyReaped)
    );
    state.close(reader).unwrap();
    state.require_quiescent(parent).unwrap();
}

#[test]
fn socket_shutdown_bypasses_inherited_aliases_and_preserves_buffered_data() {
    let (mut state, parent) = ledger();
    let [left, right] = state.socketpair(parent).unwrap();
    let child = state.fork(parent).unwrap().child;
    state.write(left, b"x").unwrap();
    state.shutdown_write(left).unwrap();
    assert_eq!(
        state.write(state.descriptor(child, left.number()).unwrap(), b"y"),
        Err(ProcessError::BrokenPipe)
    );
    assert_eq!(bytes(state.read(right, 1).unwrap()), b"x");
    assert!(bytes(state.read(right, 1).unwrap()).is_empty());
    state.write(right, b"reverse").unwrap();
    assert_eq!(bytes(state.read(left, 7).unwrap()), b"reverse");
}

#[test]
fn rights_hold_open_description_until_received_even_after_sender_closes() {
    let (mut state, parent) = ledger();
    let [left, right] = state.socketpair(parent).unwrap();
    let child = state.fork(parent).unwrap().child;
    state.close(right).unwrap();
    state
        .close(state.descriptor(child, left.number()).unwrap())
        .unwrap();
    let child_socket = state.descriptor(child, right.number()).unwrap();
    let [reader, writer] = state.pipe(child).unwrap();
    state.send_rights(child_socket, b"ACK", &[reader]).unwrap();
    state.close(reader).unwrap();
    state.write(writer, b"payload").unwrap();
    let received = message(state.recv_rights(left, 1, 1).unwrap());
    assert_eq!(received.bytes, b"A");
    assert_eq!(received.descriptors.len(), 1);
    assert_eq!(received.descriptors[0].process(), parent);
    assert_eq!(
        bytes(state.read(received.descriptors[0], 7).unwrap()),
        b"payload"
    );
    let remainder = message(state.recv_rights(left, 8, 1).unwrap());
    assert_eq!(remainder.bytes, b"CK");
    assert!(remainder.descriptors.is_empty());
    state.close(writer).unwrap();
    assert!(bytes(state.read(received.descriptors[0], 1).unwrap()).is_empty());
}

#[test]
fn rights_truncation_and_wrong_process_do_not_mutate_transport() {
    let (mut state, parent) = ledger();
    let [left, right] = state.socketpair(parent).unwrap();
    let [reader, writer] = state.pipe(parent).unwrap();
    let child = state.fork(parent).unwrap().child;
    let child_reader = state.descriptor(child, reader.number()).unwrap();
    assert_eq!(
        state.send_rights(left, b"x", &[child_reader]),
        Err(ProcessError::WrongProcess)
    );
    state.send_rights(left, b"x", &[reader, writer]).unwrap();
    assert_eq!(
        state.recv_rights(right, 1, 1),
        Err(ProcessError::Unsupported("truncated SCM_RIGHTS"))
    );
    assert_eq!(state.readiness(right).unwrap().bytes, 1);
    assert_eq!(
        message(state.recv_rights(right, 1, 2).unwrap())
            .descriptors
            .len(),
        2
    );
}

#[test]
fn plain_read_discards_ancillary_refs_and_close_drops_unread_payload() {
    let (mut state, parent) = ledger();
    let [left, right] = state.socketpair(parent).unwrap();
    let [reader, writer] = state.pipe(parent).unwrap();
    state.send_rights(left, b"x", &[writer]).unwrap();
    state.close(writer).unwrap();
    assert!(matches!(
        state.read(reader, 1).unwrap(),
        ProcessIo::Pending(_)
    ));
    assert_eq!(bytes(state.read(right, 1).unwrap()), b"x");
    assert!(bytes(state.read(reader, 1).unwrap()).is_empty());
    state.write(left, b"unread").unwrap();
    state.close(right).unwrap();
    assert_eq!(state.retained_bytes(), 0);
    assert_eq!(
        state.write(left, b"no-reader"),
        Err(ProcessError::BrokenPipe)
    );
}

#[test]
fn dup_flags_generations_and_reset_do_not_regrant_stale_tokens() {
    let (mut state, parent) = ledger();
    let [reader, writer] = state.pipe(parent).unwrap();
    state.set_close_on_exec(writer, true).unwrap();
    state.set_nonblocking(reader, true).unwrap();
    let duplicate = state.dup2(reader, 20).unwrap();
    assert!(state.nonblocking(duplicate).unwrap());
    assert!(!state.close_on_exec(duplicate).unwrap());
    assert_eq!(state.read(duplicate, 1), Err(ProcessError::WouldBlock));
    state.close(duplicate).unwrap();
    let replacement = state.dup2(writer, 20).unwrap();
    assert_eq!(
        state.nonblocking(duplicate),
        Err(ProcessError::UnknownDescriptor)
    );
    assert_ne!(duplicate, replacement);
    let snapshot = state.clone();
    state.reset();
    let new_root = state.create_root().unwrap();
    assert_ne!(parent, new_root);
    assert_eq!(state.read(reader, 1), Err(ProcessError::UnknownProcess));
    state = snapshot;
    assert_eq!(state.read(reader, 1), Err(ProcessError::WouldBlock));
}

struct Deferred {
    key: HostRequestKey,
    requests: Vec<HostRequest>,
}
impl HostEffects for Deferred {
    fn begin(&mut self, _: SourceOrigin) -> Result<(), HostError> {
        Ok(())
    }
    fn request(&mut self, request: HostRequest) -> HostOutcome {
        self.requests.push(request);
        HostOutcome::Pending(self.key)
    }
    fn finish(&mut self, _: bool) -> Result<(), HostError> {
        Ok(())
    }
}
fn invocation() -> ProgramInvocation {
    ProgramInvocation {
        program: ProgramId::allocate(),
        working_root: FileRootId::allocate(),
        arguments: ProcessArguments::new(["a b;$(never shell)".into()]).unwrap(),
    }
}

#[test]
fn source_equivalent_descriptor_handshake_exec_eof_output_and_nonzero_reap() {
    let (mut state, parent) = ledger();
    let [parent_socket, original_child_socket] = state.socketpair(parent).unwrap();
    let child = state.fork(parent).unwrap().child;
    state.close(original_child_socket).unwrap();
    state
        .close(state.descriptor(child, parent_socket.number()).unwrap())
        .unwrap();
    let child_socket = state
        .descriptor(child, original_child_socket.number())
        .unwrap();
    let [status_reader, status_writer] = state.pipe(child).unwrap();
    state.set_close_on_exec(status_writer, true).unwrap();
    state.dup2(child_socket, 0).unwrap();
    let [output_reader, output_writer] = state.pipe(child).unwrap();
    let [error_reader, error_writer] = state.pipe(child).unwrap();
    state.dup2(output_writer, 1).unwrap();
    state.dup2(error_writer, 2).unwrap();
    state.close(output_writer).unwrap();
    state.close(error_writer).unwrap();
    state
        .send_rights(
            child_socket,
            &[123],
            &[status_reader, output_reader, error_reader],
        )
        .unwrap();
    let ProcessIo::Pending(ack) = state.read(child_socket, 1).unwrap() else {
        panic!("child must wait for ACK")
    };
    let received = message(state.recv_rights(parent_socket, 4, 3).unwrap());
    assert_eq!(received.bytes, [123]);
    let [status, stdout, stderr]: [FileDescriptor; 3] = received.descriptors.try_into().unwrap();
    state.write(parent_socket, b"!").unwrap();
    assert!(state.event_ready(ack).unwrap());
    assert_eq!(bytes(state.read(child_socket, 1).unwrap()), b"!");
    for fd in [status_reader, output_reader, error_reader] {
        state.close(fd).unwrap();
    }
    let ProcessIo::Pending(status_event) = state.read(status, 4).unwrap() else {
        panic!("exec has not occurred")
    };
    let key = HostRequestKey::allocate();
    let invocation = invocation();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    assert_eq!(
        state.exec(child, invocation.clone(), &mut effects).unwrap(),
        ExecOutcome::Pending(key)
    );
    assert_eq!(effects.requests, [HostRequest::RunProgram(invocation)]);
    assert!(!state.event_ready(status_event).unwrap());
    assert_eq!(
        state.wait(parent, child, true).unwrap(),
        WaitOutcome::StillRunning
    );
    let output = ProcessOutput {
        termination: ProcessTermination::Exited(13),
        stdout: b"hello".to_vec(),
        stderr: b"problem".to_vec(),
    };
    let pending_snapshot = state.clone();
    assert_eq!(
        state
            .complete_exec(
                child,
                key,
                HostOutcome::Ready(HostResponse::Process(output.clone()))
            )
            .unwrap(),
        ExecOutcome::Replaced
    );
    assert!(state.event_ready(status_event).unwrap());
    assert!(bytes(state.read(status, 4).unwrap()).is_empty());
    assert_eq!(bytes(state.read(stdout, 3).unwrap()), b"hel");
    assert_eq!(bytes(state.read(stdout, 9).unwrap()), b"lo");
    assert!(bytes(state.read(stdout, 9).unwrap()).is_empty());
    assert_eq!(bytes(state.read(stderr, 9).unwrap()), b"problem");
    assert_eq!(
        state.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(13))
    );
    for fd in [parent_socket, status, stdout, stderr] {
        state.close(fd).unwrap();
    }
    state.require_quiescent(parent).unwrap();
    assert_eq!(state.retained_bytes(), 0);
    // Whole-transaction rollback restores IPC and pending host keys. Delivering
    // the provider's retained observation again performs no second request.
    state = pending_snapshot;
    assert_eq!(state.pending_requests().collect::<Vec<_>>(), [(child, key)]);
    assert_eq!(state.retained_bytes(), 0);
    state
        .complete_exec(
            child,
            key,
            HostOutcome::Ready(HostResponse::Process(output)),
        )
        .unwrap();
    assert_eq!(effects.requests.len(), 1);
    assert!(bytes(state.read(status, 4).unwrap()).is_empty());
    assert_eq!(bytes(state.read(stdout, 9).unwrap()), b"hello");
    assert_eq!(bytes(state.read(stderr, 9).unwrap()), b"problem");
    assert_eq!(
        state.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(13))
    );
    for fd in [parent_socket, status, stdout, stderr] {
        state.close(fd).unwrap();
    }
    state.require_quiescent(parent).unwrap();
}

#[test]
fn rejected_exec_preserves_status_writer_without_granting_errno() {
    let (mut state, parent) = ledger();
    let child = state.fork(parent).unwrap().child;
    let [reader, writer] = state.pipe(child).unwrap();
    state.set_close_on_exec(writer, true).unwrap();
    let key = HostRequestKey::allocate();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    state.exec(child, invocation(), &mut effects).unwrap();
    assert_eq!(
        state.complete_exec(
            child,
            HostRequestKey::allocate(),
            HostOutcome::Rejected(HostError::Unavailable)
        ),
        Err(ProcessError::NotRunning)
    );
    assert_eq!(
        state
            .complete_exec(child, key, HostOutcome::Rejected(HostError::Unavailable))
            .unwrap(),
        ExecOutcome::Rejected(HostError::Unavailable)
    );
    state.write(writer, b"failure").unwrap();
    assert_eq!(bytes(state.read(reader, 9).unwrap()), b"failure");
    state.exit(child, -1).unwrap();
    assert_eq!(
        state.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(255))
    );
}

#[test]
fn observed_launch_errno_resumes_child_without_closing_exec_descriptors() {
    let (mut state, parent) = ledger();
    let child = state.fork(parent).unwrap().child;
    let [reader, writer] = state.pipe(child).unwrap();
    state.set_close_on_exec(writer, true).unwrap();
    let key = HostRequestKey::allocate();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    state.exec(child, invocation(), &mut effects).unwrap();
    let failure =
        ProcessLaunchFailure::from_os_error(crate::host_effects::ProcessHostPlatform::MacOS, 2)
            .unwrap();
    assert!(failure.matches_platform(&jai_types::OperatingSystem::MacOS));
    assert!(!failure.matches_platform(&jai_types::OperatingSystem::Linux));
    assert_eq!(failure.errno(), 2);
    assert_eq!(
        state
            .complete_exec(
                child,
                key,
                HostOutcome::Ready(HostResponse::ProcessLaunchFailed(failure))
            )
            .unwrap(),
        ExecOutcome::Failed(failure)
    );
    assert!(state.close_on_exec(writer).unwrap());
    assert_eq!(
        state.wait(parent, child, true).unwrap(),
        WaitOutcome::StillRunning
    );
    state.write(writer, &failure.errno().to_le_bytes()).unwrap();
    assert_eq!(
        bytes(state.read(reader, 4).unwrap()),
        failure.errno().to_le_bytes()
    );
    state.exit(child, -1).unwrap();
    assert_eq!(
        state.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(255))
    );
    state.require_quiescent(parent).unwrap();
    assert!(
        ProcessLaunchFailure::from_os_error(crate::host_effects::ProcessHostPlatform::MacOS, 0)
            .is_err()
    );
}

#[test]
fn resource_limits_and_output_failure_are_atomic() {
    let mut state = VirtualProcesses::new(ProcessLimits {
        processes: 2,
        descriptors: 4,
        buffered_bytes: 2,
        transfer_bytes: 2,
        queued_messages: 1,
        transferred_descriptors: 1,
    });
    let root = state.create_root().unwrap();
    let [reader, writer] = state.pipe(root).unwrap();
    let child = state.fork(root).unwrap().child;
    assert_eq!(state.fork(root), Err(ProcessError::Budget("processes")));
    assert_eq!(state.pipe(root), Err(ProcessError::Budget("descriptors")));
    state.write(writer, b"ok").unwrap();
    assert_eq!(
        state.write(writer, b"x"),
        Err(ProcessError::Budget("queued transport"))
    );
    assert_eq!(bytes(state.read(reader, 2).unwrap()), b"ok");
    state
        .close(state.descriptor(child, reader.number()).unwrap())
        .unwrap();
    state
        .dup2(state.descriptor(child, writer.number()).unwrap(), 1)
        .unwrap();
    let output = ProcessOutput {
        termination: ProcessTermination::TimedOut,
        stdout: b"big".to_vec(),
        stderr: Vec::new(),
    };
    let key = HostRequestKey::allocate();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    state.exec(child, invocation(), &mut effects).unwrap();
    assert_eq!(
        state.complete_exec(
            child,
            key,
            HostOutcome::Ready(HostResponse::Process(output))
        ),
        Err(ProcessError::Budget("queued transport"))
    );
    assert_eq!(state.retained_bytes(), 0);
    assert_eq!(
        state.wait(root, child, true).unwrap(),
        WaitOutcome::StillRunning
    );
}

#[test]
fn cancellation_collects_queued_socket_reference_cycles() {
    let (mut state, root) = ledger();
    let [left, right] = state.socketpair(root).unwrap();
    state.send_rights(left, b"cycle", &[right]).unwrap();
    state.close(left).unwrap();
    state.close(right).unwrap();
    assert_eq!(
        state.require_quiescent(root),
        Err(ProcessError::LiveResources)
    );
    state.reset();
    assert_eq!(state.retained_bytes(), 0);
    assert_eq!(state.readiness(right), Err(ProcessError::UnknownProcess));
}

#[test]
fn pending_host_exec_rejects_stdin_instead_of_silently_discarding_it() {
    let (mut state, parent) = ledger();
    let [parent_socket, original_child_socket] = state.socketpair(parent).unwrap();
    let child = state.fork(parent).unwrap().child;
    state.close(original_child_socket).unwrap();
    state
        .close(state.descriptor(child, parent_socket.number()).unwrap())
        .unwrap();
    let child_socket = state
        .descriptor(child, original_child_socket.number())
        .unwrap();
    let stdin = state.dup2(child_socket, 0).unwrap();
    state.write(parent_socket, b"before exec").unwrap();
    let key = HostRequestKey::allocate();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    assert_eq!(
        state.exec(child, invocation(), &mut effects),
        Err(ProcessError::Unsupported(
            "host runner does not accept process stdin"
        ))
    );
    assert!(effects.requests.is_empty());
    assert_eq!(bytes(state.read(stdin, 20).unwrap()), b"before exec");
    state.exec(child, invocation(), &mut effects).unwrap();
    assert_eq!(state.pending_requests().collect::<Vec<_>>(), [(child, key)]);
    assert_eq!(
        state.write(parent_socket, b"after exec"),
        Err(ProcessError::Unsupported(
            "host runner does not accept process stdin"
        ))
    );
    assert_eq!(state.retained_bytes(), 0);
    state
        .complete_exec(child, key, HostOutcome::Rejected(HostError::Unavailable))
        .unwrap();
    assert!(state.pending_requests().next().is_none());
    state.write(parent_socket, b"source retry").unwrap();
    assert_eq!(bytes(state.read(stdin, 20).unwrap()), b"source retry");
}

#[test]
fn empty_descriptor_table_does_not_allow_commit_of_pending_root_exec() {
    let (mut state, root) = ledger();
    let key = HostRequestKey::allocate();
    let mut effects = Deferred {
        key,
        requests: Vec::new(),
    };
    state.exec(root, invocation(), &mut effects).unwrap();
    assert_eq!(
        state.require_quiescent(root),
        Err(ProcessError::LiveResources)
    );
    state
        .complete_exec(
            root,
            key,
            HostOutcome::Ready(HostResponse::Process(ProcessOutput {
                termination: ProcessTermination::Exited(0),
                stdout: Vec::new(),
                stderr: Vec::new(),
            })),
        )
        .unwrap();
    state.require_quiescent(root).unwrap();
}

#[test]
fn deep_ancillary_reference_chains_close_without_recursive_stack_growth() {
    let (mut state, root) = ledger();
    let pairs: Vec<_> = (0..600).map(|_| state.socketpair(root).unwrap()).collect();
    for pair in pairs.windows(2) {
        state.send_rights(pair[0][0], b"x", &[pair[1][1]]).unwrap();
    }
    state.write(pairs.last().unwrap()[0], b"x").unwrap();
    for pair in pairs.iter().skip(1) {
        state.close(pair[1]).unwrap();
    }
    for pair in &pairs {
        state.close(pair[0]).unwrap();
    }
    assert_eq!(state.retained_bytes(), 600);
    state.close(pairs[0][1]).unwrap();
    assert_eq!(state.retained_bytes(), 0);
    state.require_quiescent(root).unwrap();
}

#[test]
fn partial_messages_release_large_backing_allocations() {
    let (mut state, root) = ledger();
    let payload = vec![1; 256 * 1024];
    let mut pairs = Vec::new();
    for _ in 0..32 {
        let [reader, writer] = state.pipe(root).unwrap();
        state.write(writer, &payload).unwrap();
        assert_eq!(
            bytes(state.read(reader, payload.len() - 1).unwrap()).len(),
            payload.len() - 1
        );
        pairs.push([reader, writer]);
    }
    assert_eq!(state.retained_bytes(), 32);
    let retained_capacity: usize = state
        .channels
        .values()
        .flat_map(|channel| &channel.messages)
        .map(|message| message.bytes.capacity())
        .sum();
    assert!(retained_capacity <= 64);
    for [reader, writer] in pairs {
        assert_eq!(bytes(state.read(reader, 1).unwrap()), [1]);
        state.close(writer).unwrap();
        state.close(reader).unwrap();
    }
    state.require_quiescent(root).unwrap();
}

#[test]
fn fork_descriptor_forecast_is_readonly_and_counts_live_parent_slots() {
    let (mut state, parent) = ledger();
    let [reader, writer] = state.pipe(parent).unwrap();
    state.dup2(reader, 10).unwrap();
    state.write(writer, b"queued").unwrap();
    let before = format!("{state:?}");
    let ids: Vec<_> = state.processes.keys().copied().collect();
    let original_work = state.work_cost().unwrap();
    assert_eq!(state.fork_descriptor_cells(parent), Ok(3));
    assert_eq!(state.fork_descriptor_cells(parent), Ok(3));
    assert_eq!(format!("{state:?}"), before);
    assert_eq!(state.processes.keys().copied().collect::<Vec<_>>(), ids);
    assert_eq!(state.work_cost().unwrap(), original_work);
    state.close(writer).unwrap();
    let parent_slots = state.fork_descriptor_cells(parent).unwrap();
    assert_eq!(parent_slots, 2);
    let original_work = state.work_cost().unwrap();
    let mut candidate = state.clone();
    let pair = candidate.fork(parent).unwrap();
    assert!(candidate.work_cost().unwrap() <= original_work + parent_slots as u64 + 1);
    assert_eq!(
        candidate.fork_descriptor_cells(pair.child),
        Ok(parent_slots)
    );
    assert_eq!(state.processes.len(), 1);
    assert_eq!(bytes(state.read(reader, 6).unwrap()), b"queued");
}

#[test]
fn fork_descriptor_forecast_rejects_unknown_nonrunning_and_full_process_table() {
    let (mut state, parent) = ledger();
    let before = format!("{state:?}");
    assert_eq!(
        state.fork_descriptor_cells(ProcessId(0)),
        Err(ProcessError::UnknownProcess)
    );
    assert_eq!(format!("{state:?}"), before);
    let child = state.fork(parent).unwrap().child;
    state.exit(child, 0).unwrap();
    let before = format!("{state:?}");
    assert_eq!(
        state.fork_descriptor_cells(child),
        Err(ProcessError::NotRunning)
    );
    assert_eq!(format!("{state:?}"), before);
    let mut full = VirtualProcesses::new(ProcessLimits {
        processes: 1,
        ..Default::default()
    });
    let root = full.create_root().unwrap();
    let before = format!("{full:?}");
    assert_eq!(
        full.fork_descriptor_cells(root),
        Err(ProcessError::Budget("processes"))
    );
    assert_eq!(format!("{full:?}"), before);
}

#[test]
fn fork_descriptor_forecast_counts_before_global_descriptor_quota_rejection() {
    let mut state = VirtualProcesses::new(ProcessLimits {
        descriptors: 2,
        ..Default::default()
    });
    let parent = state.create_root().unwrap();
    state.pipe(parent).unwrap();
    let before = format!("{state:?}");
    let work = state.work_cost().unwrap();
    assert_eq!(state.fork_descriptor_cells(parent), Ok(2));
    assert_eq!(state.fork(parent), Err(ProcessError::Budget("descriptors")));
    assert_eq!(format!("{state:?}"), before);
    assert_eq!(state.work_cost().unwrap(), work);
}
