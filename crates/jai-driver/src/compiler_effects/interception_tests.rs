use super::*;

fn ready(outcome: EffectOutcome) -> CompilerResponse {
    match outcome {
        EffectOutcome::Ready(response) => response,
        other => panic!("{other:?}"),
    }
}
fn subscribe(session: &mut CompilerSession) -> WorkspaceId {
    session.begin();
    let CompilerResponse::Workspace(child) =
        ready(session.request(CompilerRequest::CreateWorkspace {
            name: "genuine child input".into(),
        }))
    else {
        panic!("workspace identity expected")
    };
    ready(session.request(CompilerRequest::AddSource {
        workspace: child,
        source: "main :: () -> int { return 42; }".into(),
    }));
    ready(session.request(CompilerRequest::BeginIntercept {
        workspace: child,
        flags: InterceptFlags::SKIP_ALL,
    }));
    child
}

#[test]
fn wait_ticket_and_filtered_events_survive_actual_transaction_parking() {
    let mut session = CompilerSession::new();
    let child = subscribe(&mut session);
    let EffectOutcome::Pending(key) = session.request(CompilerRequest::WaitForMessage) else {
        panic!("no job work has happened, so no event can be ready")
    };
    let mut parked = session.suspend_transaction().unwrap();
    assert_eq!(parked.intercepted_workspaces().collect::<Vec<_>>(), [child]);
    assert_eq!(
        parked.publish_event(CompilerEvent::Phase {
            workspace: session.root(),
            phase: CompilerPhase::SourceParsed,
        }),
        Err(CompilerEventError::UnsubscribedWorkspace)
    );
    let parsed = CompilerEvent::Phase {
        workspace: child,
        phase: CompilerPhase::SourceParsed,
    };
    parked.publish_event(parsed.clone()).unwrap();
    let preview = session.preview_transaction(&parked).unwrap();
    assert_eq!(preview.workspace(child).unwrap().inputs().len(), 1);
    session
        .resume_transaction_with_preview(parked, preview)
        .unwrap();
    assert!(matches!(
        session.poll_request(&CompilerRequest::WaitForMessage, EffectKey(key.0 + 1)),
        EffectOutcome::Rejected(_)
    ));
    assert_eq!(
        session.poll_request(&CompilerRequest::WaitForMessage, key),
        EffectOutcome::Ready(CompilerResponse::Message(parsed))
    );
    let EffectOutcome::Pending(next) = session.request(CompilerRequest::WaitForMessage) else {
        panic!("receiving parse readiness does not fabricate another event")
    };
    assert_ne!(next, key);
    let mut parked = session.suspend_transaction().unwrap();
    parked
        .publish_event(CompilerEvent::Phase {
            workspace: child,
            phase: CompilerPhase::Typechecked { pending_count: 0 },
        })
        .unwrap();
    session.resume_transaction(parked).unwrap();
    assert!(matches!(
        session.poll_request(&CompilerRequest::WaitForMessage, next),
        EffectOutcome::Ready(CompilerResponse::Message(CompilerEvent::Phase {
            phase: CompilerPhase::Typechecked { pending_count: 0 },
            ..
        }))
    ));
    session.finish(false).unwrap();
    assert!(session.workspace(child).is_none());
}

#[test]
fn unread_events_are_removed_when_their_subscription_ends() {
    let mut session = CompilerSession::new();
    let child = subscribe(&mut session);
    let mut parked = session.suspend_transaction().unwrap();
    parked
        .publish_event(CompilerEvent::Phase {
            workspace: child,
            phase: CompilerPhase::SourceParsed,
        })
        .unwrap();
    session.resume_transaction(parked).unwrap();
    ready(session.request(CompilerRequest::EndIntercept { workspace: child }));
    assert!(matches!(
        session.request(CompilerRequest::WaitForMessage),
        EffectOutcome::Rejected(_)
    ));
    session.finish(false).unwrap();
}

#[test]
fn unsupported_ast_and_performance_requests_fail_explicitly() {
    for flags in [
        InterceptFlags::NONE,
        InterceptFlags::from_bits(0x103f).unwrap(),
    ] {
        let mut session = CompilerSession::new();
        let root = session.root();
        session.begin();
        assert!(matches!(
            session.request(CompilerRequest::BeginIntercept {
                workspace: root,
                flags
            }),
            EffectOutcome::Rejected(_)
        ));
        session.finish(false).unwrap();
    }
}

#[test]
fn event_queue_and_source_pending_count_are_bounded() {
    let mut session = CompilerSession::new();
    let child = subscribe(&mut session);
    let mut parked = session.suspend_transaction().unwrap();
    assert_eq!(
        parked.publish_event(CompilerEvent::Phase {
            workspace: child,
            phase: CompilerPhase::Typechecked {
                pending_count: i32::MAX as u32 + 1
            },
        }),
        Err(CompilerEventError::InvalidPendingCount)
    );
    for _ in 0..MAX_QUEUED_EVENTS {
        parked
            .publish_event(CompilerEvent::Phase {
                workspace: child,
                phase: CompilerPhase::SourceParsed,
            })
            .unwrap();
    }
    assert_eq!(
        parked.publish_event(CompilerEvent::Phase {
            workspace: child,
            phase: CompilerPhase::SourceParsed,
        }),
        Err(CompilerEventError::QueueLimit)
    );
    drop(parked);
    assert!(session.workspace(child).is_none());
}

#[test]
fn retired_and_foreign_workspace_handles_cannot_create_subscriptions() {
    for foreign in [false, true] {
        let mut session = CompilerSession::new();
        let workspace = if foreign {
            CompilerSession::new().root()
        } else {
            session.root()
        };
        session.begin();
        if !foreign {
            ready(session.request(CompilerRequest::DestroyWorkspace { workspace }));
        }
        assert!(matches!(
            session.request(CompilerRequest::BeginIntercept {
                workspace,
                flags: InterceptFlags::SKIP_ALL,
            }),
            EffectOutcome::Rejected(_)
        ));
        session.finish(false).unwrap();
        assert!(session.workspace(session.root()).is_some());
    }
}

#[test]
fn a_pending_wait_cannot_be_reported_as_a_completed_transaction() {
    let mut session = CompilerSession::new();
    let child = subscribe(&mut session);
    assert!(matches!(
        session.request(CompilerRequest::WaitForMessage),
        EffectOutcome::Pending(_)
    ));
    assert!(session.finish(true).is_err());
    assert!(session.workspace(child).is_none());
}
