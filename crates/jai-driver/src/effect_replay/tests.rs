use super::*;
use jai_vm::{BitcodeOptimization, BuildOption};

fn origin(session: &CompilerSession, body: &[u8]) -> SourceOrigin {
    SourceOrigin {
        workspace: session.root(),
        path: "/build/main.jai".into(),
        start: 0,
        end: body.len(),
        body_hash: 1,
        body: body.into(),
        specialization: vec![],
    }
}
fn ready(outcome: EffectOutcome) -> CompilerResponse {
    match outcome {
        EffectOutcome::Ready(value) => value,
        other => panic!("{other:?}"),
    }
}

#[test]
fn originated_child_errors_and_status_recovery_are_not_repeated_by_replay() {
    let mut session = CompilerSession::new();
    session.begin();
    let CompilerResponse::Workspace(child) =
        ready(session.request(CompilerRequest::CreateWorkspace {
            name: "child".into(),
        }))
    else {
        panic!("expected a workspace")
    };
    session.finish(true).unwrap();
    let root = session.root();
    let mut child_origin = origin(&session, b"child reports");
    child_origin.workspace = child;
    let mut cache = EffectReplayCache::default();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(child_origin.clone());
        effects.begin();
        ready(effects.request(CompilerRequest::Message {
            level: jai_vm::MessageLevel::Error,
            text: "child failed".into(),
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(
        session.workspace(root).unwrap().status(),
        jai_vm::WorkspaceStatus::Ok
    );
    assert_eq!(
        session.workspace(child).unwrap().status(),
        jai_vm::WorkspaceStatus::Failed
    );
    session.take_messages();
    session.begin();
    ready(session.request(CompilerRequest::SetWorkspaceStatus {
        workspace: child,
        status: jai_vm::WorkspaceStatus::Ok,
    }));
    session.finish(true).unwrap();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(child_origin);
        effects.begin();
        ready(effects.request(CompilerRequest::Message {
            level: jai_vm::MessageLevel::Error,
            text: "child failed".into(),
        }));
        effects.finish(true).unwrap();
    }
    assert!(session.error().is_none());
    assert!(session.take_messages().is_empty());
    assert_eq!(
        session.workspace(child).unwrap().status(),
        jai_vm::WorkspaceStatus::Ok
    );
}

#[test]
fn repeated_sources_and_workspace_creation_reuse_committed_results() {
    let mut session = CompilerSession::new();
    let root = session.root();
    let origin = origin(&session, b"same run");
    let mut cache = EffectReplayCache::default();
    let mut child = None;
    for _ in 0..3 {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        let result = ready(effects.request(CompilerRequest::CreateWorkspace {
            name: "child".into(),
        }));
        if let Some(expected) = &child {
            assert_eq!(expected, &result);
        } else {
            child = Some(result);
        }
        ready(effects.request(CompilerRequest::AddSource {
            workspace: root,
            source: "answer :: 42;".into(),
        }));
        ready(effects.request(CompilerRequest::Message {
            level: jai_vm::MessageLevel::Info,
            text: "created input".into(),
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspaces().len(), 2);
    assert_eq!(session.workspace(root).unwrap().inputs().len(), 1);
    assert_eq!(cache.len(), 1);
    assert_eq!(session.take_messages().len(), 1);
}

#[test]
fn replay_keeps_historical_create_destroy_without_reallocating_or_mutating() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"create then destroy");
    let mut cache = EffectReplayCache::default();
    let mut child = None;
    for _ in 0..2 {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) =
            ready(effects.request(CompilerRequest::CreateWorkspace {
                name: "temporary".into(),
            }))
        else {
            panic!("expected child identity")
        };
        assert_eq!(*child.get_or_insert(id), id);
        ready(effects.request(CompilerRequest::DestroyWorkspace {
            workspace: id,
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspaces().len(), 1);
    assert!(session.is_destroyed(child.unwrap()));
    assert_eq!(cache.len(), 1);
    session.begin();
    assert!(matches!(
        session.request(CompilerRequest::GetBuildOptions {
            workspace: child.unwrap()
        }),
        EffectOutcome::Rejected(_)
    ));
    session.finish(false).unwrap();
}

#[test]
fn chronological_settings_are_replayed_even_after_later_transactions() {
    let mut session = CompilerSession::new();
    let root = session.root();
    let origin = origin(&session, b"options run");
    let mut cache = EffectReplayCache::default();
    let read = CompilerRequest::GetBuildOptions {
        workspace: root,
    };
    let write = CompilerRequest::SetBuildOption {
        workspace: root,
        option: BuildOption::BitcodeOptimization(BitcodeOptimization::O3),
    };
    let mut first = vec![];
    for pass in 0..2 {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        let responses = vec![
            ready(effects.request(read.clone())),
            ready(effects.request(write.clone())),
            ready(effects.request(read.clone())),
        ];
        effects.finish(true).unwrap();
        if pass == 0 {
            first = responses;
        } else {
            assert_eq!(responses, first);
        }
    }
    assert_ne!(first[0], first[2]);
}

#[test]
fn missing_changed_and_extra_requests_fail_without_new_mutations() {
    let mut session = CompilerSession::new();
    let root = session.root();
    let origin = origin(&session, b"run");
    let mut cache = EffectReplayCache::default();
    let request = CompilerRequest::AddSource {
        workspace: root,
        source: "a :: 1;".into(),
    };
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        ready(effects.request(request.clone()));
        effects.finish(true).unwrap();
    }
    for mode in 0..3 {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        if mode == 1 {
            assert!(matches!(
                effects.request(CompilerRequest::CreateWorkspace {
                    name: "changed".into()
                }),
                EffectOutcome::Rejected(_)
            ));
        }
        if mode == 2 {
            ready(effects.request(request.clone()));
            assert!(matches!(
                effects.request(request.clone()),
                EffectOutcome::Rejected(_)
            ));
        }
        assert!(effects.finish(true).is_err());
    }
    assert_eq!(session.workspaces().len(), 1);
    assert_eq!(session.workspace(root).unwrap().inputs().len(), 1);
}

#[test]
fn exact_body_bytes_distinguish_edits_even_when_hashes_collide() {
    let mut session = CompilerSession::new();
    let root = session.root();
    let mut cache = EffectReplayCache::default();
    for body in [b"run a".as_slice(), b"run b".as_slice()] {
        let origin = origin(&session, body);
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin);
        effects.begin();
        ready(effects.request(CompilerRequest::AddSource {
            workspace: root,
            source: String::from_utf8(body.into()).unwrap(),
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(cache.len(), 2);
    assert_eq!(session.workspace(root).unwrap().inputs().len(), 2);
}

#[test]
fn failed_and_over_limit_transactions_never_enter_cache_or_commit() {
    let mut session = CompilerSession::new();
    let root = session.root();
    let origin = origin(&session, b"run");
    let mut cache = EffectReplayCache::new(ReplayLimits {
        runs: 0,
        ..Default::default()
    });
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        ready(effects.request(CompilerRequest::AddSource {
            workspace: root,
            source: "a :: 1;".into(),
        }));
        assert!(effects.finish(true).is_err());
    }
    assert!(session.workspace(root).unwrap().inputs().is_empty());
    assert!(cache.is_empty());
    cache = EffectReplayCache::default();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin);
        effects.begin();
        ready(effects.request(CompilerRequest::CreateWorkspace {
            name: "discard".into(),
        }));
        effects.finish(false).unwrap();
    }
    assert_eq!(session.workspaces().len(), 1);
    assert!(cache.is_empty());
}
