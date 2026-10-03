use super::*;

fn ready(session: &mut CompilerSession, request: CompilerRequest) -> CompilerResponse {
    match session.request(request) {
        EffectOutcome::Ready(response) => response,
        other => panic!("{other:?}"),
    }
}
fn create(session: &mut CompilerSession) -> WorkspaceId {
    match ready(
        session,
        CompilerRequest::CreateWorkspace {
            name: "child".into(),
        },
    ) {
        CompilerResponse::Workspace(id) => id,
        other => panic!("{other:?}"),
    }
}
fn destroy(session: &mut CompilerSession, id: WorkspaceId) {
    assert_eq!(
        ready(session, CompilerRequest::DestroyWorkspace { workspace: id }),
        CompilerResponse::Unit
    );
}

#[test]
fn destruction_is_atomic_and_committed_identities_never_return() {
    let mut session = CompilerSession::new();
    session.begin();
    let child = create(&mut session);
    session.finish(true).unwrap();
    session.begin();
    destroy(&mut session, child);
    assert!(session.workspace(child).is_some());
    assert!(!session.is_destroyed(child));
    session.finish(false).unwrap();
    assert!(session.workspace(child).is_some());
    assert!(!session.is_destroyed(child));
    session.begin();
    destroy(&mut session, child);
    session.finish(true).unwrap();
    assert!(session.workspace(child).is_none());
    assert!(session.is_destroyed(child));
    session.begin();
    let replacement = create(&mut session);
    assert_ne!(replacement, child);
    session.finish(true).unwrap();
}

#[test]
fn creation_inputs_and_destruction_can_share_one_transaction() {
    let mut session = CompilerSession::new();
    session.begin();
    let child = create(&mut session);
    ready(
        &mut session,
        CompilerRequest::AddSource {
            workspace: child,
            source: "never compiled".into(),
        },
    );
    ready(
        &mut session,
        CompilerRequest::SetWorkspaceStatus {
            workspace: child,
            status: WorkspaceStatus::Failed,
        },
    );
    destroy(&mut session, child);
    session.finish(true).unwrap();
    assert_eq!(session.workspaces().len(), 1);
    assert!(session.is_destroyed(child));
    assert!(session.error().is_none());
}

#[test]
fn every_targeted_operation_rejects_staged_and_committed_tombstones() {
    let mut session = CompilerSession::new();
    session.begin();
    let child = create(&mut session);
    session.finish(true).unwrap();
    let operations = [
        CompilerRequest::DestroyWorkspace { workspace: child },
        CompilerRequest::GetBuildOptions { workspace: child },
        CompilerRequest::GetWorkspaceName { workspace: child },
        CompilerRequest::AddSource {
            workspace: child,
            source: "discard".into(),
        },
        CompilerRequest::SetWorkspaceStatus {
            workspace: child,
            status: WorkspaceStatus::Ok,
        },
        CompilerRequest::SetBuildOption {
            workspace: child,
            option: BuildOption::Optimize(true),
        },
    ];
    for operation in &operations {
        session.begin();
        destroy(&mut session, child);
        assert!(
            matches!(session.request(operation.clone()), EffectOutcome::Rejected(reason) if reason.contains("destroyed"))
        );
        assert!(session.finish(true).is_err());
        assert!(session.workspace(child).is_some());
        assert!(!session.is_destroyed(child));
    }
    session.begin();
    destroy(&mut session, child);
    session.finish(true).unwrap();
    for operation in operations {
        session.begin();
        assert!(
            matches!(session.request(operation), EffectOutcome::Rejected(reason) if reason.contains("destroyed"))
        );
        assert!(session.finish(true).is_err());
    }
}

#[test]
fn retiring_root_preserves_unrelated_work_and_session_diagnostics() {
    let mut session = CompilerSession::new();
    let root = session.root();
    session.begin();
    destroy(&mut session, root);
    let child = create(&mut session);
    ready(
        &mut session,
        CompilerRequest::AddSource {
            workspace: child,
            source: "main :: () {}".into(),
        },
    );
    ready(
        &mut session,
        CompilerRequest::Message {
            level: MessageLevel::Error,
            text: "retired recipe diagnostic".into(),
        },
    );
    ready(
        &mut session,
        CompilerRequest::WriteOutput {
            stream: CompilerOutputStream::StandardOutput,
            bytes: vec![0, 42],
        },
    );
    session.finish(true).unwrap();
    assert!(session.is_destroyed(root));
    assert_eq!(session.root(), root);
    assert!(session.error().is_none());
    assert_eq!(session.take_messages()[0].text, "retired recipe diagnostic");
    assert_eq!(session.take_outputs()[0].bytes, vec![0, 42]);
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 1);
    session.begin_from_workspace(child);
    ready(
        &mut session,
        CompilerRequest::SetWorkspaceStatus {
            workspace: child,
            status: WorkspaceStatus::Failed,
        },
    );
    session.finish(true).unwrap();
    assert!(session.error().is_some());
}

#[test]
fn fatal_reports_roll_back_root_retirement() {
    let mut session = CompilerSession::new();
    let root = session.root();
    session.begin();
    destroy(&mut session, root);
    ready(
        &mut session,
        CompilerRequest::Report {
            level: MessageLevel::Error,
            continuation: ReportContinuation::Stop,
            location: SourceLocation {
                path: "/build/main.jai".into(),
                line: 4,
                column: 2,
            },
            text: "stop".into(),
        },
    );
    assert!(matches!(
        session.finish(true),
        Err(jai_vm::Error::CompilerDiagnostic { .. })
    ));
    assert!(session.workspace(root).is_some());
    assert!(!session.is_destroyed(root));
    assert!(session.take_messages().is_empty());
}

#[test]
fn unknown_handles_reject_and_destroyed_failures_do_not_survive() {
    let mut session = CompilerSession::new();
    let foreign = CompilerSession::new().root();
    session.begin();
    let child = create(&mut session);
    assert!(matches!(
        session.request(CompilerRequest::DestroyWorkspace { workspace: foreign }),
        EffectOutcome::Rejected(_)
    ));
    assert!(session.finish(true).is_err());
    assert!(session.workspace(child).is_none());
    assert!(!session.is_destroyed(foreign));
    session.begin();
    let child = create(&mut session);
    ready(
        &mut session,
        CompilerRequest::SetWorkspaceStatus {
            workspace: child,
            status: WorkspaceStatus::Failed,
        },
    );
    session.finish(true).unwrap();
    assert!(session.error().is_some());
    session.begin();
    destroy(&mut session, child);
    session.finish(true).unwrap();
    assert!(session.error().is_none());
}

#[test]
fn workspace_names_read_real_transaction_visible_identity() {
    let mut session = CompilerSession::new();
    let root = session.root();
    session.begin();
    let CompilerResponse::Workspace(child) = ready(
        &mut session,
        CompilerRequest::CreateWorkspace {
            name: "child π".into(),
        },
    ) else {
        panic!("expected child workspace")
    };
    assert_eq!(
        ready(
            &mut session,
            CompilerRequest::GetWorkspaceName { workspace: child }
        ),
        CompilerResponse::WorkspaceName("child π".into())
    );
    assert_eq!(
        ready(
            &mut session,
            CompilerRequest::GetWorkspaceName { workspace: root }
        ),
        CompilerResponse::WorkspaceName(String::new())
    );
    session.finish(true).unwrap();
    session.begin();
    assert_eq!(
        ready(
            &mut session,
            CompilerRequest::GetWorkspaceName { workspace: child }
        ),
        CompilerResponse::WorkspaceName("child π".into())
    );
    session.finish(true).unwrap();
}
