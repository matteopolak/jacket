use super::*;
use crate::host_effects::{
    FileRootId, HostError, HostOutcome, HostRequest, HostResponse, ProcessArguments,
    ProcessHostPlatform, ProcessLaunchFailure, ProcessOutput, ProgramId, ProgramInvocation,
    ReviewedProgramGrant, ReviewedPrograms,
};
use std::ffi::OsString;

struct Observations {
    outcome: HostOutcome,
    requests: Vec<HostRequest>,
}
impl crate::CompilerEffects for Observations {
    fn begin(&mut self) {
        panic!("helper cannot begin outer transaction");
    }
    fn request(&mut self, _: crate::CompilerRequest) -> crate::EffectOutcome {
        panic!("not a compiler request");
    }
    fn host_request(&mut self, request: HostRequest) -> HostOutcome {
        self.requests.push(request);
        self.outcome.clone()
    }
    fn finish(&mut self, _: bool) -> Result<(), Error> {
        panic!("helper cannot finish outer transaction");
    }
}
fn observed(outcome: HostOutcome) -> Observations {
    Observations {
        outcome,
        requests: Vec::new(),
    }
}
fn output(bytes: &[u8]) -> HostOutcome {
    HostOutcome::Ready(HostResponse::Process(ProcessOutput {
        termination: ProcessTermination::Exited(7),
        stdout: bytes.into(),
        stderr: Vec::new(),
    }))
}
fn scope(executable: OsString, argv: &[OsString]) -> (ProcessExecScope, ProgramInvocation) {
    scope_limits(executable, argv, Default::default())
}
fn scope_limits(
    executable: OsString,
    argv: &[OsString],
    limits: ProcessExecLimits,
) -> (ProcessExecScope, ProgramInvocation) {
    let invocation = ProgramInvocation {
        program: ProgramId::allocate(),
        arguments: ProcessArguments::new(argv[1..].iter().cloned()).unwrap(),
        working_root: FileRootId::allocate(),
    };
    let grant = ReviewedProgramGrant::from_registered(
        executable,
        argv[0].clone(),
        invocation.clone(),
        "/own-fixture".into(),
    )
    .unwrap();
    let programs = ReviewedPrograms::new(vec![grant], Default::default()).unwrap();
    assert!(programs.work_cost() >= argv.len() as u64);
    (
        ProcessExecScope::new(programs, "/own-fixture".into(), limits).unwrap(),
        invocation,
    )
}
impl Fixture {
    fn exec_args(&mut self, executable: &[u8], argv: &[&[u8]]) -> [Value; 2] {
        let byte = self.types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = self.types.pointer(byte).unwrap();
        let mut cstring = |bytes: &[u8]| {
            let mut terminated = bytes.to_vec();
            terminated.push(0);
            let root = self.bytes(&terminated);
            self.memory
                .cast_pointer(&self.types, &root, byte, CastMode::Unchecked)
                .unwrap()
        };
        let file = cstring(executable);
        let mut values: Vec<_> = argv
            .iter()
            .map(|bytes| Value::Pointer(cstring(bytes)))
            .collect();
        values.push(Value::Pointer(Pointer::null(byte)));
        let array = self
            .types
            .fixed_array(pointer, values.len() as u64)
            .unwrap();
        let root = self
            .memory
            .allocate(
                &self.types,
                array,
                Some(Value::Array {
                    ty: array,
                    elements: values,
                }),
            )
            .unwrap();
        [
            Value::Pointer(file),
            Value::Pointer(self.memory.sequence_data(&self.types, &root).unwrap()),
        ]
    }
    fn exec(
        &mut self,
        args: &[Value],
        scope: &ProcessExecScope,
        effects: &mut Observations,
    ) -> Result<ProcessCallOutcome, Error> {
        self.proof(ProcessAbiOperation::ExecVp).invoke_exec(
            args,
            &mut self.memory,
            &self.types,
            &self.target,
            &mut self.branch,
            &mut self.world,
            scope,
            effects,
            &mut |cost| {
                self.charged += cost;
                Ok(())
            },
        )
    }
    fn complete(
        &mut self,
        control: &ProcessExecControl,
        observation: HostOutcome,
    ) -> Result<ProcessCallOutcome, Error> {
        self.proof(ProcessAbiOperation::ExecVp)
            .complete_process_exec(
                control,
                observation,
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
}
fn control(outcome: ProcessCallOutcome) -> ProcessExecControl {
    let ProcessCallOutcome::ExecControl(control) = outcome else {
        panic!("exec cannot fake a scalar success");
    };
    control
}

#[test]
fn typed_exec_preserves_non_utf8_and_argument_boundaries_then_replaces_child_once() {
    use std::os::unix::ffi::OsStringExt;
    let mut fixture = Fixture::new();
    let parent = fixture.branch.current();
    let [read, write] = fixture.pipe();
    let parent_read = fixture.world.descriptor(parent, read).unwrap();
    let child = fixture.world.fork(parent).unwrap().child;
    fixture
        .world
        .close(fixture.world.descriptor(parent, write).unwrap())
        .unwrap();
    fixture
        .world
        .close(fixture.world.descriptor(child, read).unwrap())
        .unwrap();
    let writer = fixture.world.descriptor(child, write).unwrap();
    fixture.world.dup2(writer, 1).unwrap();
    fixture.world.close(writer).unwrap();
    fixture.branch = fixture.branch.for_child(child, &fixture.world).unwrap();
    let argv = [
        OsString::from("reviewed-argv-zero"),
        OsString::from_vec(vec![255, b' ', b';']),
        OsString::from("$(literal)"),
    ];
    let (scope, invocation) = scope("fixture".into(), &argv);
    let args = fixture.exec_args(
        b"fixture",
        &[b"reviewed-argv-zero", &[255, b' ', b';'], b"$(literal)"],
    );
    let key = HostRequestKey::allocate();
    let mut effects = observed(HostOutcome::Pending(key));
    let pending = control(fixture.exec(&args, &scope, &mut effects).unwrap());
    assert_eq!(pending.stage(), ProcessExecStage::HostPending(key));
    assert_eq!(effects.requests, [HostRequest::RunProgram(invocation)]);
    assert_eq!(
        control(
            fixture
                .complete(&pending, HostOutcome::Pending(key))
                .unwrap()
        )
        .stage(),
        pending.stage()
    );
    assert!(
        fixture
            .complete(&pending, HostOutcome::Pending(HostRequestKey::allocate()))
            .is_err()
    );
    let replaced = control(fixture.complete(&pending, output(b"child bytes")).unwrap());
    assert_eq!(replaced.stage(), ProcessExecStage::Replaced);
    assert_eq!(effects.requests.len(), 1);
    assert!(fixture.complete(&pending, output(b"duplicate")).is_err());
    assert!(fixture.complete(&replaced, output(b"duplicate")).is_err());
    assert_eq!(
        fixture.world.read(parent_read, 20).unwrap(),
        ProcessIo::Ready(b"child bytes".to_vec())
    );
    assert_eq!(
        fixture.world.read(parent_read, 20).unwrap(),
        ProcessIo::Ready(Vec::new())
    );
    assert_eq!(
        fixture.world.wait(parent, child, false).unwrap(),
        WaitOutcome::Reaped(ProcessTermination::Exited(7))
    );
}
#[test]
fn observed_launch_errno_preserves_cloexec_descriptors_and_pending_failure_resumes_source() {
    let mut fixture = Fixture::new();
    let [read, _] = fixture.pipe();
    let fd = fixture
        .world
        .descriptor(fixture.branch.current(), read)
        .unwrap();
    fixture.world.set_close_on_exec(fd, true).unwrap();
    let (scope, _) = scope("fixture".into(), &["fixture".into()]);
    let args = fixture.exec_args(b"fixture", &[b"fixture"]);
    let failure = ProcessLaunchFailure::from_os_error(ProcessHostPlatform::MacOS, 2).unwrap();
    let mut effects = observed(HostOutcome::Ready(HostResponse::ProcessLaunchFailed(
        failure,
    )));
    assert_eq!(
        scalar(fixture.exec(&args, &scope, &mut effects).unwrap()),
        -1
    );
    assert_eq!(fixture.errno().1, 2);
    fixture
        .world
        .validate_running(fixture.branch.current())
        .unwrap();
    fixture
        .world
        .descriptor(fixture.branch.current(), read)
        .unwrap();
    let key = HostRequestKey::allocate();
    effects.outcome = HostOutcome::Pending(key);
    let pending = control(fixture.exec(&args, &scope, &mut effects).unwrap());
    assert_eq!(
        scalar(
            fixture
                .complete(
                    &pending,
                    HostOutcome::Ready(HostResponse::ProcessLaunchFailed(failure))
                )
                .unwrap()
        ),
        -1
    );
    fixture
        .world
        .validate_running(fixture.branch.current())
        .unwrap();
    fixture
        .world
        .descriptor(fixture.branch.current(), read)
        .unwrap();
    effects.outcome = HostOutcome::Ready(HostResponse::ProcessLaunchFailed(
        ProcessLaunchFailure::from_os_error(ProcessHostPlatform::Linux, 2).unwrap(),
    ));
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    assert_eq!(fixture.errno().1, 2);
}
#[test]
fn exec_policy_denial_and_unreviewed_argv_are_fatal_without_errno_or_state_mutation() {
    let mut fixture = Fixture::new();
    let (scope, _) = scope("fixture".into(), &["fixture".into(), "exact".into()]);
    let args = fixture.exec_args(b"fixture", &[b"fixture", b"changed"]);
    let mut effects = observed(output(b"ignored"));
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    assert!(effects.requests.is_empty());
    assert!(fixture.branch.errno.is_empty());
    let args = fixture.exec_args(b"fixture", &[b"fixture", b"exact"]);
    effects.outcome = HostOutcome::Rejected(HostError::Denied("unreviewed provider"));
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    assert!(fixture.branch.errno.is_empty());
    fixture
        .world
        .validate_running(fixture.branch.current())
        .unwrap();
    assert!(fixture.call(ProcessAbiOperation::ExecVp, &args).is_err());
}
#[test]
fn exec_byte_and_argument_budgets_and_unterminated_typed_strings_never_submit() {
    let mut fixture = Fixture::new();
    let (mut scope, _) = scope("fixture".into(), &["fixture".into()]);
    let mut args = fixture.exec_args(b"fixture", &[b"fixture"]);
    let byte = fixture.types.scalar(ScalarType::Int(IntegerType::U8));
    let unterminated = fixture.bytes(b"unterminated");
    args[0] = Value::Pointer(
        fixture
            .memory
            .cast_pointer(&fixture.types, &unterminated, byte, CastMode::Unchecked)
            .unwrap(),
    );
    let mut effects = observed(output(b"ignored"));
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    let args = fixture.exec_args(b"fixture", &[b"fixture", b"extra"]);
    scope = scope_limits(
        "fixture".into(),
        &["fixture".into()],
        ProcessExecLimits {
            arguments: 1,
            bytes: 1024,
        },
    )
    .0;
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    scope = scope_limits(
        "fixture".into(),
        &["fixture".into()],
        ProcessExecLimits {
            arguments: 4,
            bytes: 4,
        },
    )
    .0;
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    assert!(effects.requests.is_empty());
    assert!(fixture.branch.errno.is_empty());
}
#[test]
fn actual_output_charge_rejects_before_replacement_and_pending_completion_remains_retryable() {
    let mut fixture = Fixture::new();
    let [_, writer] = fixture.pipe();
    let writer = fixture
        .world
        .descriptor(fixture.branch.current(), writer)
        .unwrap();
    fixture.world.dup2(writer, 1).unwrap();
    fixture.world.close(writer).unwrap();
    let (scope, _) = scope("fixture".into(), &["fixture".into()]);
    let args = fixture.exec_args(b"fixture", &[b"fixture"]);
    let mut effects = observed(output(&vec![b'x'; 8192]));
    let proof = fixture.proof(ProcessAbiOperation::ExecVp);
    let result = proof.invoke_exec(
        &args,
        &mut fixture.memory,
        &fixture.types,
        &fixture.target,
        &mut fixture.branch,
        &mut fixture.world,
        &scope,
        &mut effects,
        &mut |cost| {
            if cost >= 16384 {
                Err(Error::Limit(LimitKind::Fuel))
            } else {
                Ok(())
            }
        },
    );
    assert_eq!(result.unwrap_err(), Error::Limit(LimitKind::Fuel));
    fixture
        .world
        .validate_running(fixture.branch.current())
        .unwrap();
    assert_eq!(effects.requests.len(), 1);
    let key = HostRequestKey::allocate();
    effects.outcome = HostOutcome::Pending(key);
    let pending = control(fixture.exec(&args, &scope, &mut effects).unwrap());
    let result = proof.complete_process_exec(
        &pending,
        output(b"bytes"),
        &mut fixture.memory,
        &fixture.types,
        &fixture.target,
        &mut fixture.branch,
        &mut fixture.world,
        &mut |_| Err(Error::Limit(LimitKind::Fuel)),
    );
    assert_eq!(result.unwrap_err(), Error::Limit(LimitKind::Fuel));
    assert_eq!(
        control(fixture.complete(&pending, output(b"bytes")).unwrap()).stage(),
        ProcessExecStage::Replaced
    );
    assert_eq!(effects.requests.len(), 2);
}

#[test]
fn raw_integer_argv_addresses_cannot_reconstruct_typed_pointer_authority() {
    let mut fixture = Fixture::new();
    let (scope, _) = scope("fixture".into(), &["fixture".into()]);
    let mut args = fixture.exec_args(b"fixture", &[b"fixture"]);
    let u64_type = fixture.types.scalar(ScalarType::Int(IntegerType::U64));
    let array = fixture.types.fixed_array(u64_type, 2).unwrap();
    let raw = fixture
        .memory
        .allocate(
            &fixture.types,
            array,
            Some(Value::Array {
                ty: array,
                elements: vec![u64(0x12345678), u64(0)],
            }),
        )
        .unwrap();
    let byte = fixture.types.scalar(ScalarType::Int(IntegerType::U8));
    let byte_pointer = fixture.types.pointer(byte).unwrap();
    args[1] = Value::Pointer(
        fixture
            .memory
            .cast_pointer(&fixture.types, &raw, byte_pointer, CastMode::Unchecked)
            .unwrap(),
    );
    let mut effects = observed(output(b"ignored"));
    assert!(fixture.exec(&args, &scope, &mut effects).is_err());
    assert!(effects.requests.is_empty());
    assert!(fixture.branch.errno.is_empty());
    fixture
        .world
        .validate_running(fixture.branch.current())
        .unwrap();
}
