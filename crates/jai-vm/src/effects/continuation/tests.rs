use super::*;
use crate::LimitKind;
use crate::host_effects::{FileRootId, HostPath, HostRequestKey};

struct Effects {
    compiler_requests: usize,
    compiler_polls: usize,
    host_requests: usize,
    host_polls: usize,
    compiler_prefix: CompilerResponse,
    host_prefix: HostResponse,
    host_key: HostRequestKey,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            compiler_requests: 0,
            compiler_polls: 0,
            host_requests: 0,
            host_polls: 0,
            compiler_prefix: CompilerResponse::WorkspaceName("retained".into()),
            host_prefix: HostResponse::FileBytes(vec![1, 2, 3]),
            host_key: HostRequestKey::allocate(),
        }
    }
}
impl CompilerEffects for Effects {
    fn begin(&mut self) {
    }
    fn finish(&mut self, _: bool) -> Result<(), Error> {
        Ok(())
    }
    fn request(&mut self, _: CompilerRequest) -> EffectOutcome {
        self.compiler_requests += 1;
        if self.compiler_requests == 1 {
            EffectOutcome::Ready(self.compiler_prefix.clone())
        } else {
            EffectOutcome::Pending(EffectKey(9))
        }
    }
    fn poll_request(&mut self, _: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert_eq!(key, EffectKey(9));
        self.compiler_polls += 1;
        if self.compiler_polls == 1 {
            EffectOutcome::Pending(key)
        } else {
            EffectOutcome::Ready(CompilerResponse::Unit)
        }
    }
    fn host_request(&mut self, _: HostRequest) -> HostOutcome {
        self.host_requests += 1;
        if self.host_requests == 1 {
            HostOutcome::Ready(self.host_prefix.clone())
        } else {
            HostOutcome::Pending(self.host_key)
        }
    }
    fn poll_host_request(&mut self, key: HostRequestKey) -> HostOutcome {
        assert_eq!(key, self.host_key);
        self.host_polls += 1;
        if self.host_polls == 1 {
            HostOutcome::Pending(key)
        } else {
            HostOutcome::Ready(HostResponse::WriteStaged)
        }
    }
}
fn compiler_request() -> CompilerRequest {
    CompilerRequest::GetWorkspaceName {
        workspace: WorkspaceId::from_raw(1).unwrap(),
    }
}
fn host_request() -> HostRequest {
    HostRequest::ReadEntireFile(HostPath::new(FileRootId::allocate(), "data").unwrap())
}

#[test]
fn compiler_ready_prefix_is_copied_without_rerequest_and_only_leaf_is_polled() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(1000, 1000).unwrap();
    let prefix = compiler_request();
    let leaf = CompilerRequest::WaitForMessage;
    assert!(matches!(
        effects.request(prefix.clone()),
        EffectOutcome::Ready(_)
    ));
    assert_eq!(
        effects.request(leaf.clone()),
        EffectOutcome::Pending(EffectKey(9))
    );
    effects.take_work();
    for attempt in 0..2 {
        effects.retry_leaf(1000).unwrap();
        assert_eq!(
            effects.request(prefix.clone()),
            EffectOutcome::Ready(CompilerResponse::WorkspaceName("retained".into()))
        );
        let result = effects.request(leaf.clone());
        if attempt == 0 {
            assert_eq!(result, EffectOutcome::Pending(EffectKey(9)));
        } else {
            assert_eq!(result, EffectOutcome::Ready(CompilerResponse::Unit));
        }
        assert!(
            effects.take_work()
                >= compiler_response_cells(&CompilerResponse::WorkspaceName("retained".into()))
        );
    }
    assert_eq!(effects.inner().compiler_requests, 2);
    assert_eq!(effects.inner().compiler_polls, 2);
    assert!(effects.take_failure().is_none());
    effects.end_leaf().unwrap();
}

#[test]
fn host_ready_prefix_is_copied_without_rerequest_and_only_leaf_is_polled() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(1000, 1000).unwrap();
    let prefix = host_request();
    let leaf = HostRequest::WriteEntireFile {
        path: match &prefix {
            HostRequest::ReadEntireFile(path) => path.clone(),
            _ => unreachable!(),
        },
        bytes: vec![7],
    };
    assert_eq!(
        effects.host_request(prefix.clone()),
        HostOutcome::Ready(HostResponse::FileBytes(vec![1, 2, 3]))
    );
    let key = effects.inner().host_key;
    assert_eq!(
        effects.host_request(leaf.clone()),
        HostOutcome::Pending(key)
    );
    effects.take_work();
    for attempt in 0..2 {
        effects.retry_leaf(1000).unwrap();
        assert_eq!(
            effects.host_request(prefix.clone()),
            HostOutcome::Ready(HostResponse::FileBytes(vec![1, 2, 3]))
        );
        let result = effects.host_request(leaf.clone());
        if attempt == 0 {
            assert_eq!(result, HostOutcome::Pending(key));
        } else {
            assert_eq!(result, HostOutcome::Ready(HostResponse::WriteStaged));
        }
        assert!(
            effects.take_work() >= host_response_cells(&HostResponse::FileBytes(vec![1, 2, 3]))
        );
    }
    assert_eq!(effects.inner().host_requests, 2);
    assert_eq!(effects.inner().host_polls, 2);
    effects.end_leaf().unwrap();
}

#[test]
fn changed_compiler_stream_has_a_structured_failure_before_new_request() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(1000, 1000).unwrap();
    effects.request(compiler_request());
    effects.retry_leaf(1000).unwrap();
    assert!(matches!(
        effects.request(CompilerRequest::WaitForMessage),
        EffectOutcome::Rejected(_)
    ));
    assert_eq!(
        effects.take_failure(),
        Some(Error::InvalidIr(
            "resumed compiler adapter changed its request stream"
        ))
    );
    assert_eq!(effects.inner().compiler_requests, 1);
}

#[test]
fn changed_host_stream_has_a_structured_failure_before_poll() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(1000, 1000).unwrap();
    let request = host_request();
    effects.host_request(request.clone());
    effects.host_request(request);
    effects.retry_leaf(1000).unwrap();
    effects.host_request(host_request());
    assert_eq!(
        effects.take_failure(),
        Some(Error::InvalidIr(
            "resumed host adapter changed its request stream"
        ))
    );
    assert_eq!(effects.inner().host_requests, 2);
    assert_eq!(effects.inner().host_polls, 0);
}

#[test]
fn compiler_request_cell_failure_precedes_provider_request() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(3, 1000).unwrap();
    assert!(matches!(
        effects.request(compiler_request()),
        EffectOutcome::Rejected(_)
    ));
    assert_eq!(
        effects.take_failure(),
        Some(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(effects.inner().compiler_requests, 0);
    assert_eq!(effects.take_work(), 0);
}

#[test]
fn host_request_fuel_failure_precedes_provider_request() {
    let mut effects = JournalEffects::new(Effects::default());
    effects.begin_leaf(1000, 0).unwrap();
    assert!(matches!(
        effects.host_request(host_request()),
        HostOutcome::Rejected(_)
    ));
    assert_eq!(effects.take_failure(), Some(Error::Limit(LimitKind::Fuel)));
    assert_eq!(effects.inner().host_requests, 0);
    assert_eq!(effects.take_work(), 0);
}

#[test]
fn oversized_response_has_structured_cells_failure_before_retained_clone() {
    let inner = Effects {
        compiler_prefix: CompilerResponse::WorkspaceName("x".repeat(100)),
        ..Effects::default()
    };
    let mut effects = JournalEffects::new(inner);
    effects.begin_leaf(150, 1000).unwrap();
    assert!(matches!(
        effects.request(compiler_request()),
        EffectOutcome::Rejected(_)
    ));
    assert_eq!(
        effects.take_failure(),
        Some(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(effects.inner().compiler_requests, 1);
    assert!(effects.journal.as_ref().unwrap().records.is_empty());
}

#[test]
fn replay_response_clone_fuel_is_checked_before_clone() {
    let mut effects = JournalEffects::new(Effects::default());
    let request = compiler_request();
    effects.begin_leaf(1000, 1000).unwrap();
    effects.request(request.clone());
    effects.take_work();
    let comparison = compiler_request_cells(&request) * 2;
    effects.retry_leaf(comparison as u64).unwrap();
    assert!(matches!(
        effects.request(request),
        EffectOutcome::Rejected(_)
    ));
    assert_eq!(effects.take_failure(), Some(Error::Limit(LimitKind::Fuel)));
    assert_eq!(effects.take_work(), comparison);
    assert_eq!(effects.inner().compiler_requests, 1);
    assert_eq!(effects.journal.as_ref().unwrap().cursor, 0);
}

#[test]
fn cached_host_response_clone_has_a_transient_cell_limit() {
    let mut effects = JournalEffects::new(Effects::default());
    let request = host_request();
    effects.begin_leaf(1000, 1000).unwrap();
    effects.host_request(request.clone());
    effects.take_work();
    let journal = effects.journal.as_mut().unwrap();
    journal.limit = journal.cells
        + host_request_cells(&request)
        + host_response_cells(&HostResponse::FileBytes(vec![1, 2, 3]))
        - 1;
    effects.retry_leaf(1000).unwrap();
    assert!(matches!(
        effects.host_request(request),
        HostOutcome::Rejected(_)
    ));
    assert_eq!(
        effects.take_failure(),
        Some(Error::Limit(LimitKind::ValueCells))
    );
    assert_eq!(effects.inner().host_requests, 1);
    assert_eq!(effects.journal.as_ref().unwrap().cursor, 0);
}

#[test]
fn fixed_typed_launch_and_intercept_metadata_are_counted() {
    use crate::host_effects::{ProcessHostPlatform, ProcessLaunchFailure};
    let failure = ProcessLaunchFailure::from_os_error(ProcessHostPlatform::Linux, 2).unwrap();
    assert_eq!(
        host_response_cells(&HostResponse::ProcessLaunchFailed(failure)),
        3
    );
    assert_eq!(
        compiler_response_cells(&CompilerResponse::Message(CompilerEvent::Phase {
            workspace: WorkspaceId::from_raw(1).unwrap(),
            phase: CompilerPhase::Typechecked {
                pending_count: 200
            },
        })),
        4
    );
    assert_eq!(
        compiler_request_cells(&CompilerRequest::BeginIntercept {
            workspace: WorkspaceId::from_raw(1).unwrap(),
            flags: InterceptFlags::SKIP_ALL,
        }),
        3
    );
}
