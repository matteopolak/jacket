use super::*;
use crate::host_io::FileCompilerSession;
use jai_vm::WorkspaceId;
use std::ffi::OsString;
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-host-test-{}-{:?}",
            std::process::id(),
            HostRequestKey::allocate()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn origin() -> SourceOrigin {
    SourceOrigin {
        workspace: WorkspaceId::from_raw(1).unwrap(),
        path: "self-authored.jai".into(),
        start: 0,
        end: 1,
        body_hash: 0,
        body: b"{}".to_vec(),
        specialization: vec![],
    }
}
fn ready(outcome: HostOutcome) -> HostResponse {
    match outcome {
        HostOutcome::Ready(value) => value,
        other => panic!("{other:?}"),
    }
}
#[test]
fn writes_overlay_rollback_and_atomic_commit_replay() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    let path = HostPath::new(root, "generated.jai").unwrap();
    let write = HostRequest::WriteEntireFile {
        path: path.clone(),
        bytes: b"main :: () {}".to_vec(),
    };
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(write.clone())),
        HostResponse::WriteStaged
    );
    assert!(!temp.0.join("generated.jai").exists());
    assert_eq!(
        ready(host.request(HostRequest::ReadEntireFile(path.clone()))),
        HostResponse::FileBytes(b"main :: () {}".to_vec())
    );
    host.finish(false).unwrap();
    assert!(!temp.0.join("generated.jai").exists());
    host.begin(origin()).unwrap();
    ready(host.request(write.clone()));
    ready(host.request(HostRequest::ReadEntireFile(path.clone())));
    host.finish(true).unwrap();
    assert_eq!(
        fs::read(temp.0.join("generated.jai")).unwrap(),
        b"main :: () {}"
    );
    fs::write(temp.0.join("generated.jai"), "external update").unwrap();
    host.begin(origin()).unwrap();
    ready(host.request(write));
    ready(host.request(HostRequest::ReadEntireFile(path)));
    host.finish(true).unwrap();
    assert_eq!(
        fs::read(temp.0.join("generated.jai")).unwrap(),
        b"external update"
    );
    host.retire(&origin()).unwrap();
    assert_eq!(host.retained_bytes, 0);
}
#[test]
fn capabilities_readonly_path_and_budget_fail_closed() {
    let temp = Temp::new();
    fs::write(temp.0.join("input"), vec![1; 513]).unwrap();
    let mut host = HostIo::new(HostLimits {
        bytes: 512,
        ..HostLimits::default()
    });
    let root = host.register_root(&temp.0, false).unwrap();
    assert!(HostPath::new(root, "../escape").is_err());
    assert!(HostPath::new(root, "/absolute").is_err());
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "input").unwrap()
        )),
        HostOutcome::Rejected(HostError::Budget(_))
    ));
    assert!(host.finish(true).is_err());
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(HostRequest::WriteEntireFile {
            path: HostPath::new(root, "new").unwrap(),
            bytes: vec![]
        }),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(FileRootId::allocate(), "input").unwrap()
        )),
        HostOutcome::Rejected(HostError::UnknownCapability)
    );
    host.finish(false).unwrap();
}
#[test]
fn default_denial_and_argument_boundaries() {
    let mut host = NoHostEffects;
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(FileRootId::allocate(), "input").unwrap()
        )),
        HostOutcome::Rejected(HostError::Unavailable)
    );
    assert!(ProcessArguments::new([OsString::from("bad\0argument")]).is_err());
    let args = ProcessArguments::new([OsString::from("one two"), OsString::from("$(do-not-run)")])
        .unwrap();
    assert_eq!(args.as_slice().len(), 2);
}
#[cfg(unix)]
#[test]
fn symlinks_never_escape_root() {
    let temp = Temp::new();
    let outside = Temp::new();
    fs::write(outside.0.join("input"), "outside").unwrap();
    std::os::unix::fs::symlink(&outside.0, temp.0.join("escape")).unwrap();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "escape/input").unwrap()
        )),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    assert_eq!(fs::read(outside.0.join("input")).unwrap(), b"outside");
}
#[test]
fn multiple_file_publication_is_precisely_unsupported() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    host.begin(origin()).unwrap();
    for name in ["one", "two"] {
        ready(host.request(HostRequest::WriteEntireFile {
            path: HostPath::new(root, name).unwrap(),
            bytes: vec![1],
        }));
    }
    assert!(matches!(host.finish(true), Err(HostError::Denied(_))));
    assert!(!temp.0.join("one").exists());
    assert!(!temp.0.join("two").exists());
}
#[test]
fn original_input_protection_preserves_binary_reads_but_denies_writes_and_execution() {
    let temp = Temp::new();
    let original = temp.0.join("original");
    fs::create_dir(&original).unwrap();
    // Self-authored inert binary data; it is never executed or linked.
    let bytes = b"\x7fELF\0static fixture";
    fs::write(original.join("tool"), bytes).unwrap();
    let mut host = HostIo::default();
    host.protect_original_inputs(&original).unwrap();
    let root = host.register_root(&temp.0, true).unwrap();
    assert!(matches!(
        host.register_read_only_program(
            &original.join("tool"),
            ProcessArguments::new([]).unwrap(),
            root
        ),
        Err(HostError::Denied(_))
    ));
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "original/tool").unwrap()
        ))),
        HostResponse::FileBytes(bytes.to_vec())
    );
    assert!(matches!(
        host.request(HostRequest::WriteEntireFile {
            path: HostPath::new(root, "original/tool").unwrap(),
            bytes: b"changed".to_vec()
        }),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    assert_eq!(fs::read(original.join("tool")).unwrap(), bytes);
    let mut other = origin();
    other.specialization.push(13);
    host.begin(other).unwrap();
    ready(host.request(HostRequest::WriteEntireFile {
        path: HostPath::new(root, "own-output").unwrap(),
        bytes: b"own".to_vec(),
    }));
    host.finish(true).unwrap();
    assert_eq!(fs::read(temp.0.join("own-output")).unwrap(), b"own");
    assert!(host.protect_original_inputs(&temp.0).is_err());
}
#[test]
fn reviewed_executable_fingerprint_rejects_changed_bytes_without_launch() {
    let temp = Temp::new();
    let path = temp.0.join("inert-reviewed-fixture");
    fs::write(&path, b"self-authored inert bytes").unwrap();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(vec![]).unwrap();
    let program = host
        .register_read_only_program(&path, arguments.clone(), root)
        .unwrap();
    let invocation = ProgramInvocation {
        program,
        arguments,
        working_root: root,
    };
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(HostRequest::RunProgram(invocation)) else {
        panic!("expected pending reviewed request");
    };
    host.finish(false).unwrap();
    fs::write(&path, b"different self-authored inert bytes").unwrap();
    host.service(key).unwrap();
    assert_eq!(
        host.completion(&origin(), key).unwrap(),
        HostOutcome::Rejected(HostError::Denied("reviewed executable bytes changed"))
    );
    assert!(host.service(key).is_err());
}
#[test]
fn executable_fingerprint_size_is_bounded_before_registration() {
    let temp = Temp::new();
    let path = temp.0.join("oversized-inert-fixture");
    fs::write(&path, [0u8; 2048]).unwrap();
    let mut host = HostIo::new(HostLimits {
        bytes: 1024,
        ..HostLimits::default()
    });
    let root = host.register_root(&temp.0, false).unwrap();
    assert_eq!(
        host.register_read_only_program(&path, ProcessArguments::new(vec![]).unwrap(), root),
        Err(HostError::Budget("executable fingerprint bytes"))
    );
    assert!(host.programs.is_empty());
}
#[test]
fn trusted_native_inventory_denies_same_bytes_outside_protected_paths() {
    let temp = Temp::new();
    let original = temp.0.join("original-inert-fixture");
    let copy = temp.0.join("copied-inert-fixture");
    let bytes = b"self-authored inert native-inventory fixture";
    fs::write(&original, bytes).unwrap();
    fs::write(&copy, bytes).unwrap();
    let fingerprint: [u8; 32] = Sha256::digest(bytes).into();
    let mut host = HostIo::default();
    host.protect_original_inputs(&original).unwrap();
    host.protect_original_native_fingerprint(fingerprint)
        .unwrap();
    let root = host.register_root(&temp.0, false).unwrap();
    assert_eq!(
        host.register_read_only_program(&copy, ProcessArguments::new([]).unwrap(), root),
        Err(HostError::Denied(
            "original native bytes cannot be a reviewed executable"
        ))
    );
    assert!(host.programs.is_empty());
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "copied-inert-fixture").unwrap()
        ))),
        HostResponse::FileBytes(bytes.to_vec())
    );
    host.finish(true).unwrap();
    assert!(host.protect_original_native_fingerprint([1; 32]).is_err());
}
#[cfg(unix)]
#[test]
fn inspected_installed_printf_is_pending_and_replayed_without_shell() {
    // Fixed OS program and fixed self-authored args; never upstream project commands.
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments =
        ProcessArguments::new(["%s".into(), "literal $(shell); two words".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/printf"), arguments.clone(), root)
        .unwrap();
    let request = HostRequest::RunProgram(ProgramInvocation {
        program,
        arguments,
        working_root: root,
    });
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("expected pending");
    };
    assert_eq!(
        host.completion(&origin(), key).unwrap(),
        HostOutcome::Pending(key)
    );
    let mut other_origin = origin();
    other_origin.specialization.push(1);
    assert!(matches!(
        host.completion(&other_origin, key),
        Err(HostError::Denied(_))
    ));
    assert!(host.retire(&origin()).is_err());
    host.finish(false).unwrap();
    host.service(key).unwrap();
    assert!(host.service(key).is_err());
    host.begin(origin()).unwrap();
    let HostResponse::Process(output) = ready(host.request(request.clone())) else {
        panic!("expected process output");
    };
    assert_eq!(output.termination, ProcessTermination::Exited(0));
    assert_eq!(output.stdout, b"literal $(shell); two words");
    assert!(output.stderr.is_empty());
    assert_eq!(
        host.completion(&origin(), key).unwrap(),
        HostOutcome::Ready(HostResponse::Process(output.clone()))
    );
    host.finish(true).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(ready(host.request(request)), HostResponse::Process(output));
    host.finish(true).unwrap();
    host.retire(&origin()).unwrap();
    assert!(host.completion(&origin(), key).is_err());
    assert!(host.tickets.is_empty());
    assert_eq!(host.retained_bytes, 0);
}
#[cfg(unix)]
#[test]
fn unreviewed_process_arguments_and_output_budget_rejected() {
    let temp = Temp::new();
    let mut host = HostIo::new(HostLimits {
        process_output_bytes: 2,
        ..HostLimits::default()
    });
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["%s".into(), "too-long".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/printf"), arguments.clone(), root)
        .unwrap();
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(HostRequest::RunProgram(ProgramInvocation {
            program,
            arguments: ProcessArguments::new(["changed".into()]).unwrap(),
            working_root: root
        })),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    host.begin(origin()).unwrap();
    let request = HostRequest::RunProgram(ProgramInvocation {
        program,
        arguments,
        working_root: root,
    });
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("pending");
    };
    host.finish(false).unwrap();
    host.service(key).unwrap();
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(request),
        HostOutcome::Rejected(HostError::Budget(_))
    ));
    assert!(host.finish(true).is_err());
}
#[cfg(unix)]
#[test]
fn inspected_sleep_timeout_reaps_child() {
    // Fixed OS sleep is reviewed here solely to verify the provider deadline.
    let temp = Temp::new();
    let mut host = HostIo::new(HostLimits {
        process_timeout: Duration::from_millis(5),
        ..HostLimits::default()
    });
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["1".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/bin/sleep"), arguments.clone(), root)
        .unwrap();
    let request = HostRequest::RunProgram(ProgramInvocation {
        program,
        arguments,
        working_root: root,
    });
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("pending");
    };
    host.finish(false).unwrap();
    host.service(key).unwrap();
    host.begin(origin()).unwrap();
    let HostResponse::Process(output) = ready(host.request(request)) else {
        panic!("process");
    };
    assert_eq!(output.termination, ProcessTermination::TimedOut);
    host.finish(true).unwrap();
}

#[cfg(unix)]
#[test]
fn cancelled_readiness_and_nonzero_exit_are_typed() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new([]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/false"), arguments.clone(), root)
        .unwrap();
    let request = HostRequest::RunProgram(ProgramInvocation {
        program,
        arguments,
        working_root: root,
    });
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("pending");
    };
    assert!(host.service(key).is_err());
    host.finish(false).unwrap();
    host.cancel(key).unwrap();
    assert!(matches!(
        host.completion(&origin(), key).unwrap(),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(request.clone()),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    host.finish(false).unwrap();
    host.retire(&origin()).unwrap();
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("pending");
    };
    host.finish(false).unwrap();
    host.service(key).unwrap();
    host.begin(origin()).unwrap();
    let HostResponse::Process(output) = ready(host.request(request)) else {
        panic!("process");
    };
    assert_eq!(output.termination, ProcessTermination::Exited(1));
    host.finish(true).unwrap();
}

#[test]
fn parked_host_overlay_preserves_cursor_and_can_be_discarded_without_publication() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    let path = HostPath::new(root, "output").unwrap();
    host.begin(origin()).unwrap();
    ready(host.request(HostRequest::WriteEntireFile {
        path: path.clone(),
        bytes: b"staged".to_vec(),
    }));
    let parked = host.suspend_transaction().unwrap();
    assert_eq!(parked.origin(), &origin());
    assert!(host.begin(origin()).is_err());
    assert!(host.retire(&origin()).is_err());
    assert!(HostIo::default().validate_suspended(&parked).is_err());
    let mut other = origin();
    other.specialization.push(2);
    host.begin(other).unwrap();
    host.finish(false).unwrap();
    host.resume_transaction(parked).unwrap();
    assert_eq!(
        ready(host.request(HostRequest::ReadEntireFile(path))),
        HostResponse::FileBytes(b"staged".to_vec())
    );
    let parked = host.suspend_transaction().unwrap();
    drop(parked);
    assert!(!temp.0.join("output").exists());
    host.retire(&origin()).unwrap();
    assert!(host.parked.is_empty());
    assert_eq!(host.retained_bytes, 0);
}

#[cfg(unix)]
#[test]
fn parked_process_completion_finishes_without_reissuing_the_request() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["%s".into(), "parked".into()]).unwrap();
    let program = host
        .register_read_only_program_with_argument_zero(
            Path::new("/usr/bin/printf"),
            "printf".into(),
            arguments.clone(),
            root,
        )
        .unwrap();
    let scope = ReviewedPrograms::new(
        vec![
            host.reviewed_program_grant(program, "printf".into())
                .unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let argv = vec!["printf".into(), "%s".into(), "parked".into()];
    let invocation = scope
        .resolve(
            std::ffi::OsStr::new("printf"),
            &argv,
            &temp.0.canonicalize().unwrap(),
        )
        .unwrap();
    assert_eq!(invocation.program, program);
    assert_eq!(invocation.arguments, arguments);
    assert!(
        host.register_read_only_program_with_argument_zero(
            Path::new("/usr/bin/printf"),
            "bad\0argument".into(),
            ProcessArguments::new([]).unwrap(),
            root
        )
        .is_err()
    );
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(HostRequest::RunProgram(invocation)) else {
        panic!("pending");
    };
    let parked = host.suspend_transaction().unwrap();
    assert!(host.service(key).is_err());
    host.service_suspended(&parked, key).unwrap();
    let HostResponse::Process(output) = ready(host.completion(&origin(), key).unwrap()) else {
        panic!("process");
    };
    assert_eq!(output.stdout, b"parked");
    host.resume_transaction(parked).unwrap();
    host.finish(true).unwrap();
    host.retire(&origin()).unwrap();
    let mut cancelled = origin();
    cancelled.specialization.push(12);
    host.begin(cancelled.clone()).unwrap();
    let HostOutcome::Pending(cancelled_key) =
        host.request(HostRequest::RunProgram(ProgramInvocation {
            program,
            arguments: ProcessArguments::new(["%s".into(), "parked".into()]).unwrap(),
            working_root: root,
        }))
    else {
        panic!("pending");
    };
    let parked = host.suspend_transaction().unwrap();
    host.cancel_suspended(parked).unwrap();
    assert!(matches!(
        host.completion(&cancelled, cancelled_key).unwrap(),
        HostOutcome::Rejected(HostError::Denied(_))
    ));
    assert!(host.service(cancelled_key).is_err());
    host.retire(&cancelled).unwrap();
}
#[cfg(unix)]
#[test]
fn file_session_polling_matches_parked_origin_without_reissuing_effects() {
    use jai_vm::CompilerEffects;
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["%s".into(), "completion".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/printf"), arguments.clone(), root)
        .unwrap();
    let compiler = crate::CompilerSession::new();
    let mut source = origin();
    source.workspace = compiler.root();
    let mut effects = FileCompilerSession::with_root(compiler, host, root, Path::new("")).unwrap();
    effects.set_source_origin(source.clone());
    assert!(matches!(
        effects.poll_host_request(HostRequestKey::allocate()),
        HostOutcome::Rejected(_)
    ));
    effects.begin();
    let HostOutcome::Pending(key) =
        effects.host_request(HostRequest::RunProgram(ProgramInvocation {
            program,
            arguments,
            working_root: root,
        }))
    else {
        panic!("pending reviewed request");
    };
    effects.suspend().unwrap();
    assert_eq!(effects.poll_host_request(key), HostOutcome::Pending(key));
    let mut other = source.clone();
    other.specialization.push(3);
    effects.set_source_origin(other);
    assert!(matches!(
        effects.poll_host_request(key),
        HostOutcome::Rejected(_)
    ));
    effects.set_source_origin(source.clone());
    effects.service_suspended(&source, key).unwrap();
    let HostOutcome::Ready(HostResponse::Process(output)) = effects.poll_host_request(key) else {
        panic!("serviced process response");
    };
    assert_eq!(output.stdout, b"completion");
    effects.resume().unwrap();
    assert_eq!(effects.host().transaction.as_ref().unwrap().cursor, 1);
    assert!(matches!(
        effects.poll_host_request(key),
        HostOutcome::Ready(_)
    ));
    effects.finish(true).unwrap();
    assert!(matches!(
        effects.poll_host_request(key),
        HostOutcome::Rejected(_)
    ));
}

#[cfg(unix)]
#[test]
fn file_session_services_only_validated_parked_host_dependencies() {
    use jai_vm::{CompilerEffects, Dependency};
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["%s".into(), "scheduler".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/printf"), arguments.clone(), root)
        .unwrap();
    let compiler = crate::CompilerSession::new();
    let mut source = origin();
    source.workspace = compiler.root();
    let mut effects = FileCompilerSession::with_root(compiler, host, root, Path::new("")).unwrap();
    effects.set_source_origin(source.clone());
    effects.begin();
    let HostOutcome::Pending(key) =
        effects.host_request(HostRequest::RunProgram(ProgramInvocation {
            program,
            arguments,
            working_root: root,
        }))
    else {
        panic!("pending reviewed request");
    };
    assert!(effects.service_pending(&[Dependency::Host(key)]).is_err());
    effects.suspend().unwrap();
    let mut other = source.clone();
    other.specialization.push(13);
    effects.set_source_origin(other);
    assert!(effects.service_pending(&[Dependency::Host(key)]).is_err());
    effects.set_source_origin(source.clone());
    assert!(
        effects
            .service_pending(&[
                Dependency::Host(key),
                Dependency::Host(HostRequestKey::allocate()),
            ])
            .is_err()
    );
    assert_eq!(effects.poll_host_request(key), HostOutcome::Pending(key));
    assert!(
        effects
            .service_pending(&[Dependency::Host(key), Dependency::Host(key)])
            .unwrap()
    );
    let HostOutcome::Ready(HostResponse::Process(output)) = effects.poll_host_request(key) else {
        panic!("actual serviced observation");
    };
    assert_eq!(output.stdout, b"scheduler");
    assert!(effects.service_pending(&[Dependency::Host(key)]).unwrap());
    assert_eq!(
        effects.poll_host_request(key),
        HostOutcome::Ready(HostResponse::Process(output))
    );
    effects.resume().unwrap();
    assert_eq!(effects.host().transaction.as_ref().unwrap().cursor, 1);
    effects.finish(true).unwrap();
    assert!(effects.service_pending(&[Dependency::Host(key)]).is_err());
}

#[cfg(unix)]
#[test]
fn actual_installed_program_spawn_failure_retains_os_errno_as_data() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let arguments = ProcessArguments::new(["%s".into(), "not launched".into()]).unwrap();
    let program = host
        .register_read_only_program(Path::new("/usr/bin/printf"), arguments.clone(), root)
        .unwrap();
    let request = HostRequest::RunProgram(ProgramInvocation {
        program,
        arguments,
        working_root: root,
    });
    host.begin(origin()).unwrap();
    let HostOutcome::Pending(key) = host.request(request.clone()) else {
        panic!("pending");
    };
    let parked = host.suspend_transaction().unwrap();
    // Remove only this self-authored empty working directory. No program bytes
    // are changed, and the OS cannot start the reviewed program in this cwd.
    fs::remove_dir(&temp.0).unwrap();
    let expected_errno = fs::metadata(&temp.0).unwrap_err().raw_os_error().unwrap();
    host.service_suspended(&parked, key).unwrap();
    let HostResponse::ProcessLaunchFailed(failure) =
        ready(host.completion(&origin(), key).unwrap())
    else {
        panic!("actual OS launch failure");
    };
    assert_eq!(failure.errno(), expected_errno);
    assert_eq!(
        failure.platform(),
        if cfg!(target_os = "macos") {
            ProcessHostPlatform::MacOS
        } else if cfg!(target_os = "linux") {
            ProcessHostPlatform::Linux
        } else {
            ProcessHostPlatform::Other
        }
    );
    host.resume_transaction(parked).unwrap();
    host.finish(true).unwrap();
    fs::create_dir(&temp.0).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(request)),
        HostResponse::ProcessLaunchFailed(failure)
    );
    host.finish(true).unwrap();
    host.retire(&origin()).unwrap();
}

#[test]
fn cached_request_divergence_and_global_observation_budget_fail_closed() {
    let temp = Temp::new();
    fs::write(temp.0.join("one"), "one").unwrap();
    fs::write(temp.0.join("two"), "two").unwrap();
    let mut host = HostIo::new(HostLimits {
        requests: 1,
        ..HostLimits::default()
    });
    let root = host.register_root(&temp.0, false).unwrap();
    assert!(host.register_root(&temp.0, false).is_err());
    host.begin(origin()).unwrap();
    ready(host.request(HostRequest::ReadEntireFile(
        HostPath::new(root, "one").unwrap(),
    )));
    host.finish(false).unwrap();
    host.begin(origin()).unwrap();
    assert!(matches!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "two").unwrap()
        )),
        HostOutcome::Rejected(HostError::Transaction(_))
    ));
    host.finish(false).unwrap();
    let mut other = origin();
    other.body.push(1);
    host.begin(other).unwrap();
    assert!(matches!(
        host.request(HostRequest::ReadEntireFile(
            HostPath::new(root, "two").unwrap()
        )),
        HostOutcome::Rejected(HostError::Budget(_))
    ));
    host.finish(false).unwrap();
}

#[test]
fn even_empty_committed_origins_have_bounded_lifetimes() {
    let mut host = HostIo::new(HostLimits {
        requests: 1,
        ..HostLimits::default()
    });
    host.begin(origin()).unwrap();
    host.finish(true).unwrap();
    let mut other = origin();
    other.body.push(1);
    host.begin(other).unwrap();
    assert!(matches!(host.finish(true), Err(HostError::Budget(_))));
    host.retire(&origin()).unwrap();
    assert_eq!(host.retained_bytes, 0);
}

#[test]
fn source_open_failures_are_observations_but_capability_rejections_remain_fatal() {
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, false).unwrap();
    let request =
        HostRequest::ReadFileForOpen(HostPath::new(root, "missing-parent/missing-file").unwrap());
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(request.clone())),
        HostResponse::FileOpen(FileOpenObservation::Failed(FileOpenFailure::NotFound))
    );
    host.finish(true).unwrap();
    fs::create_dir(temp.0.join("missing-parent")).unwrap();
    fs::write(temp.0.join("missing-parent/missing-file"), "later input").unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        ready(host.request(request)),
        HostResponse::FileOpen(FileOpenObservation::Failed(FileOpenFailure::NotFound))
    );
    host.finish(true).unwrap();
    host.retire(&origin()).unwrap();
    host.begin(origin()).unwrap();
    assert_eq!(
        host.request(HostRequest::ReadFileForOpen(
            HostPath::new(FileRootId::allocate(), "file").unwrap()
        )),
        HostOutcome::Rejected(HostError::UnknownCapability)
    );
    assert!(host.finish(true).is_err());
}
#[test]
fn compiler_rejection_prevents_host_publication_and_valid_commit_publishes_both() {
    use jai_vm::{CompilerEffects, CompilerRequest, EffectOutcome};
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    let path = HostPath::new(root, "generated.jai").unwrap();
    let compiler = crate::CompilerSession::new();
    let workspace = compiler.root();
    let mut effects = FileCompilerSession::with_root(compiler, host, root, Path::new("")).unwrap();
    let mut source = origin();
    source.workspace = workspace;
    effects.set_source_origin(source.clone());
    CompilerEffects::begin(&mut effects);
    assert!(matches!(
        effects.host_request(HostRequest::WriteEntireFile {
            path: path.clone(),
            bytes: b"generated".to_vec()
        }),
        HostOutcome::Ready(HostResponse::WriteStaged)
    ));
    let foreign = crate::CompilerSession::new().root();
    assert!(matches!(
        CompilerEffects::request(
            &mut effects,
            CompilerRequest::AddSource {
                workspace: foreign,
                source: "wrong workspace".into()
            }
        ),
        EffectOutcome::Rejected(_)
    ));
    assert!(CompilerEffects::finish(&mut effects, true).is_err());
    assert!(!temp.0.join("generated.jai").exists());
    effects.set_source_origin(source);
    CompilerEffects::begin(&mut effects);
    assert!(matches!(
        effects.host_request(HostRequest::WriteEntireFile {
            path,
            bytes: b"generated".to_vec()
        }),
        HostOutcome::Ready(HostResponse::WriteStaged)
    ));
    assert!(matches!(
        CompilerEffects::request(
            &mut effects,
            CompilerRequest::AddSource {
                workspace,
                source: "main :: () {}".into()
            }
        ),
        EffectOutcome::Ready(_)
    ));
    CompilerEffects::finish(&mut effects, true).unwrap();
    assert_eq!(
        fs::read(temp.0.join("generated.jai")).unwrap(),
        b"generated"
    );
    assert_eq!(
        effects
            .compiler()
            .workspace(workspace)
            .unwrap()
            .inputs()
            .len(),
        1
    );
}
#[test]
fn file_session_parks_both_journals_and_commits_or_cancels_the_same_source() {
    use jai_vm::{CompilerEffects, CompilerRequest, EffectOutcome};
    let temp = Temp::new();
    let mut host = HostIo::default();
    let root = host.register_root(&temp.0, true).unwrap();
    let compiler = crate::CompilerSession::new();
    let workspace = compiler.root();
    let mut effects = FileCompilerSession::with_root(compiler, host, root, Path::new("")).unwrap();
    let mut source = origin();
    source.workspace = workspace;
    effects.set_source_origin(source.clone());
    CompilerEffects::begin(&mut effects);
    ready(effects.host_request(HostRequest::WriteEntireFile {
        path: HostPath::new(root, "committed").unwrap(),
        bytes: b"committed".to_vec(),
    }));
    assert!(matches!(
        effects.request(CompilerRequest::AddSource {
            workspace,
            source: "First :: 1;".into()
        }),
        EffectOutcome::Ready(_)
    ));
    effects.suspend().unwrap();
    assert!(!temp.0.join("committed").exists());
    assert!(
        effects
            .compiler()
            .workspace(workspace)
            .unwrap()
            .inputs()
            .is_empty()
    );
    let mut wrong = source.clone();
    wrong.specialization.push(7);
    effects.set_source_origin(wrong);
    assert!(effects.resume().is_err());
    effects.set_source_origin(source.clone());
    effects.resume().unwrap();
    effects.finish(true).unwrap();
    assert_eq!(fs::read(temp.0.join("committed")).unwrap(), b"committed");
    assert_eq!(
        effects
            .compiler()
            .workspace(workspace)
            .unwrap()
            .inputs()
            .len(),
        1
    );
    source.specialization.push(9);
    effects.set_source_origin(source.clone());
    CompilerEffects::begin(&mut effects);
    ready(effects.host_request(HostRequest::WriteEntireFile {
        path: HostPath::new(root, "cancelled").unwrap(),
        bytes: b"cancelled".to_vec(),
    }));
    assert!(matches!(
        effects.request(CompilerRequest::AddSource {
            workspace,
            source: "Second :: 2;".into()
        }),
        EffectOutcome::Ready(_)
    ));
    effects.suspend().unwrap();
    // A later committed host/compiler job must not stop cancellation of the
    // original private staging or be rolled back with it.
    effects.compiler_mut().begin();
    assert!(matches!(
        effects.compiler_mut().request(CompilerRequest::AddSource {
            workspace,
            source: "External :: 3;".into(),
        }),
        EffectOutcome::Ready(_)
    ));
    effects.compiler_mut().finish(true).unwrap();
    effects.finish(false).unwrap();
    assert!(!temp.0.join("cancelled").exists());
    assert_eq!(
        effects
            .compiler()
            .workspace(workspace)
            .unwrap()
            .inputs()
            .len(),
        2
    );
    effects.host_mut().retire(&source).unwrap();
}
#[test]
fn file_path_scope_resolves_absolute_and_relative_paths_without_widening_root() {
    let root = FileRootId::allocate();
    let scope = FilePathScope::new(root, "/own-fixture", "nested").unwrap();
    assert_eq!(
        scope.resolve(Path::new("../input")).unwrap(),
        HostPath::new(root, "input").unwrap()
    );
    assert_eq!(
        scope
            .resolve(Path::new("/own-fixture/nested/../input"))
            .unwrap(),
        HostPath::new(root, "input").unwrap()
    );
    assert!(scope.resolve(Path::new("../../outside")).is_err());
    assert!(scope.resolve(Path::new("/other/input")).is_err());
}
