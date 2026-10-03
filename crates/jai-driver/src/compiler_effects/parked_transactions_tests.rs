use super::*;

fn ready(session: &mut CompilerSession, request: CompilerRequest) -> CompilerResponse {
    match session.request(request) {
        EffectOutcome::Ready(response) => response,
        outcome => panic!("{outcome:?}"),
    }
}
fn child(session: &mut CompilerSession) -> WorkspaceId {
    let CompilerResponse::Workspace(id) = ready(
        session,
        CompilerRequest::CreateWorkspace {
            name: "staged child".into(),
        },
    ) else {
        panic!("workspace creation must return its actual identity")
    };
    ready(
        session,
        CompilerRequest::AddSource {
            workspace: id,
            source: "main :: () -> int { return 42; }".into(),
        },
    );
    id
}
fn write(session: &mut CompilerSession, text: &str) {
    ready(
        session,
        CompilerRequest::WriteOutput {
            stream: CompilerOutputStream::StandardOutput,
            bytes: text.as_bytes().to_vec(),
        },
    );
}

#[test]
fn park_preview_and_resume_retain_child_identity_and_unpublished_output() {
    let mut session = CompilerSession::new();
    session.begin();
    let id = child(&mut session);
    write(&mut session, "before wait");
    let parked = session.suspend_transaction().unwrap();
    let job = parked.id();
    assert_eq!(parked.source_workspace(), session.root());
    assert!(session.workspace(id).is_none());
    assert!(session.take_outputs().is_empty());
    let preview = session.preview_transaction(&parked).unwrap();
    assert_eq!(preview.workspace(id).unwrap().inputs().len(), 1);
    assert_eq!(preview.workspace(id).unwrap().name(), "staged child");
    assert_eq!(preview.outputs[0].bytes, b"before wait");
    assert!(session.workspace(id).is_none());
    session.resume_transaction(parked).unwrap();
    write(&mut session, "after wait");
    let parked = session.suspend_transaction().unwrap();
    assert_eq!(parked.id(), job);
    session.resume_transaction(parked).unwrap();
    session.finish(true).unwrap();
    assert_eq!(session.workspace(id).unwrap().inputs().len(), 1);
    let output = session.take_outputs();
    assert_eq!(output.len(), 2);
    assert_eq!(output[0].bytes, b"before wait");
    assert_eq!(output[1].bytes, b"after wait");
}

#[test]
fn completed_child_preview_mutations_commit_only_with_the_waiting_parent() {
    for commit in [false, true] {
        let mut session = CompilerSession::new();
        session.begin();
        let id = child(&mut session);
        write(&mut session, "parent before");
        let parked = session.suspend_transaction().unwrap();
        let mut preview = session.preview_transaction(&parked).unwrap();
        preview.begin_from_workspace(id);
        ready(
            &mut preview,
            CompilerRequest::AddSource {
                workspace: id,
                source: "child_generated :: 17;".into(),
            },
        );
        write(&mut preview, "child output");
        let grandchild = child(&mut preview);
        preview.finish(true).unwrap();
        session
            .resume_transaction_with_preview(parked, preview)
            .unwrap();
        assert!(session.workspace(id).is_none());
        assert!(session.workspace(grandchild).is_none());
        assert!(session.take_outputs().is_empty());
        let CompilerResponse::WorkspaceName(name) = ready(
            &mut session,
            CompilerRequest::GetWorkspaceName { workspace: id },
        ) else {
            panic!("resumed parent must see its actual staged child")
        };
        assert_eq!(name, "staged child");
        write(&mut session, "parent after");
        session.finish(commit).unwrap();
        if commit {
            assert_eq!(session.workspace(id).unwrap().inputs().len(), 2);
            assert!(session.workspace(grandchild).is_some());
            let output = session.take_outputs();
            assert_eq!(output.len(), 3);
            assert_eq!(output[0].bytes, b"parent before");
            assert_eq!(output[1].bytes, b"child output");
            assert_eq!(output[2].bytes, b"parent after");
        } else {
            assert_eq!(session.workspaces().len(), 1);
            assert!(session.take_outputs().is_empty());
        }
    }
}

#[test]
fn unrelated_committed_mutation_invalidates_the_parked_snapshot() {
    let mut session = CompilerSession::new();
    session.begin();
    let id = child(&mut session);
    let parked = session.suspend_transaction().unwrap();
    session.begin();
    let root = session.root();
    ready(
        &mut session,
        CompilerRequest::AddSource {
            workspace: root,
            source: "another_run :: 1;".into(),
        },
    );
    session.finish(true).unwrap();
    assert!(matches!(
        session.resume_transaction(parked),
        Err(CompilerTransactionError::ChangedSession)
    ));
    assert!(session.workspace(id).is_none());
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn parked_transaction_cannot_cross_sessions_or_adopt_an_unrelated_preview() {
    let mut session = CompilerSession::new();
    session.begin();
    let id = child(&mut session);
    let parked = session.suspend_transaction().unwrap();
    let mut other = CompilerSession::new();
    assert!(matches!(
        other.resume_transaction(parked),
        Err(CompilerTransactionError::WrongSession)
    ));
    assert!(session.workspace(id).is_none());
    session.begin();
    child(&mut session);
    let parked = session.suspend_transaction().unwrap();
    let unrelated = session.clone();
    assert!(matches!(
        session.resume_transaction_with_preview(parked, unrelated),
        Err(CompilerTransactionError::WrongPreview)
    ));
    assert_eq!(session.workspaces().len(), 1);
}

#[test]
fn failed_preview_and_cancelled_parent_publish_no_staged_effects() {
    let mut session = CompilerSession::new();
    session.begin();
    let id = child(&mut session);
    write(&mut session, "never published");
    let parked = session.suspend_transaction().unwrap();
    drop(parked);
    assert!(session.workspace(id).is_none());
    assert!(session.take_outputs().is_empty());
    session.begin();
    let id = child(&mut session);
    let parked = session.suspend_transaction().unwrap();
    let mut preview = session.preview_transaction(&parked).unwrap();
    preview.begin_from_workspace(id);
    assert!(matches!(
        session.resume_transaction_with_preview(parked, preview),
        Err(CompilerTransactionError::UnfinishedPreview)
    ));
    assert!(session.workspace(id).is_none());
}

#[test]
fn resumed_preview_counts_child_output_against_the_same_session_budget() {
    let mut session = CompilerSession::with_output_limit(5);
    session.begin();
    write(&mut session, "abc");
    let parked = session.suspend_transaction().unwrap();
    let mut preview = session.preview_transaction(&parked).unwrap();
    preview.begin();
    write(&mut preview, "de");
    preview.finish(true).unwrap();
    session
        .resume_transaction_with_preview(parked, preview)
        .unwrap();
    assert!(matches!(
        session.request(CompilerRequest::WriteOutput {
            stream: CompilerOutputStream::StandardOutput,
            bytes: vec![b'f'],
        }),
        EffectOutcome::Rejected(_)
    ));
    session.finish(false).unwrap();
    assert!(session.take_outputs().is_empty());
}
