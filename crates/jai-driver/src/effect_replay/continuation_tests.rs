use super::*;

fn origin(session: &CompilerSession, body: &[u8]) -> SourceOrigin {
    SourceOrigin {
        workspace: session.root(),
        path: "/independently-authored/recipe.jai".into(),
        start: 0,
        end: body.len(),
        body_hash: 1,
        body: body.into(),
        specialization: vec![],
    }
}
fn ready(outcome: EffectOutcome) -> CompilerResponse {
    match outcome {
        EffectOutcome::Ready(response) => response,
        other => panic!("{other:?}"),
    }
}
fn create() -> CompilerRequest {
    CompilerRequest::CreateWorkspace {
        name: "actual staged child".into(),
    }
}
fn source(workspace: jai_vm::WorkspaceId, source: &str) -> CompilerRequest {
    CompilerRequest::AddSource {
        workspace,
        source: source.into(),
    }
}
fn write(text: &str) -> CompilerRequest {
    CompilerRequest::WriteOutput {
        stream: jai_vm::CompilerOutputStream::StandardOutput,
        bytes: text.as_bytes().to_vec(),
    }
}

#[test]
fn recording_parks_by_exact_origin_and_preserves_the_committed_replay_stream() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"create actual child; wait; continue");
    let mut cache = EffectReplayCache::default();
    let child;
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) = ready(effects.request(create())) else {
            panic!("workspace identity expected")
        };
        child = id;
        ready(effects.request(source(child, "main :: () -> int { return 42; }")));
        ready(effects.request(write("parent before")));
        effects.suspend().unwrap();
    }
    assert_eq!(cache.suspended_jobs(), 1);
    assert!(cache.suspended_job(&origin).is_some());
    assert!(session.workspace(child).is_none());
    assert!(session.take_outputs().is_empty());
    let mut preview = cache.preview_suspended(&origin, &session).unwrap();
    preview.begin();
    ready(preview.request(source(child, "child_ready :: 1;")));
    ready(preview.request(write("actual child output")));
    preview.finish(true).unwrap();
    cache.prepare_suspended_preview(&origin, preview).unwrap();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        let mut wrong = origin.clone();
        wrong.specialization = b"another source specialization".to_vec();
        effects.set_source_origin(wrong);
        assert!(effects.resume().is_err());
        effects.set_source_origin(origin.clone());
        effects.resume().unwrap();
        ready(effects.request(source(child, "parent_after_wait :: 2;")));
        ready(effects.request(write("parent after")));
        effects.finish(true).unwrap();
    }
    assert_eq!(cache.suspended_jobs(), 0);
    assert_eq!(cache.len(), 1);
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 3);
    let output = session.take_outputs();
    assert_eq!(output.len(), 3);
    assert_eq!(output[0].bytes, b"parent before");
    assert_eq!(output[1].bytes, b"actual child output");
    assert_eq!(output[2].bytes, b"parent after");
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin);
        effects.begin();
        assert_eq!(
            ready(effects.request(create())),
            CompilerResponse::Workspace(child)
        );
        ready(effects.request(source(child, "main :: () -> int { return 42; }")));
        ready(effects.request(write("parent before")));
        ready(effects.request(source(child, "parent_after_wait :: 2;")));
        ready(effects.request(write("parent after")));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 3);
    assert!(session.take_outputs().is_empty());
}

#[test]
fn cancellation_releases_the_parked_host_state_and_replay_budget() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"cancelled recipe");
    let mut cache = EffectReplayCache::default();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        ready(effects.request(create()));
        ready(effects.request(write("never published")));
        effects.suspend().unwrap();
    }
    assert!(cache.suspended_bytes > 0);
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin);
        assert!(effects.finish(true).is_err());
        effects.finish(false).unwrap();
    }
    assert_eq!(cache.suspended_jobs(), 0);
    assert_eq!(cache.suspended_bytes, 0);
    assert!(cache.is_empty());
    assert_eq!(session.workspaces().len(), 1);
    assert!(session.take_outputs().is_empty());
}

#[test]
fn replay_suspension_preserves_the_next_request_position_without_host_staging() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"already committed recipe");
    let root = session.root();
    let mut cache = EffectReplayCache::default();
    for first in [true, false] {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        ready(effects.request(source(root, "first :: 1;")));
        if !first {
            effects.suspend().unwrap();
            assert!(effects.cache.suspended_job(&origin).is_none());
            effects.resume().unwrap();
        }
        ready(effects.request(source(root, "second :: 2;")));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspace(root).unwrap().inputs().len(), 2);
    assert_eq!(cache.len(), 1);
}

#[test]
fn fresh_begin_cannot_restart_a_suspended_source_job() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"do not restart this recipe");
    let mut cache = EffectReplayCache::default();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        ready(effects.request(create()));
        effects.suspend().unwrap();
    }
    let mut effects = ReplayEffects::new(&mut session, &mut cache);
    effects.set_source_origin(origin);
    effects.begin();
    assert!(matches!(
        effects.request(create()),
        EffectOutcome::Rejected(_)
    ));
    effects.finish(false).unwrap();
    assert_eq!(effects.session.workspaces().len(), 1);
    assert_eq!(effects.cache.suspended_jobs(), 0);
}

#[test]
fn suspended_recordings_share_the_completed_replay_count_budget() {
    let mut session = CompilerSession::new();
    let first = origin(&session, b"first source job");
    let second = origin(&session, b"second source job");
    let mut cache = EffectReplayCache::new(ReplayLimits {
        runs: 1,
        ..ReplayLimits::default()
    });
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(first.clone());
        effects.begin();
        ready(effects.request(create()));
        effects.suspend().unwrap();
    }
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(second);
        effects.begin();
        ready(effects.request(create()));
        assert!(effects.suspend().is_err());
        effects.finish(false).unwrap();
        assert_eq!(effects.cache.suspended_jobs(), 1);
        effects.set_source_origin(first);
        effects.finish(false).unwrap();
    }
    assert_eq!(session.workspaces().len(), 1);
    assert_eq!(cache.suspended_jobs(), 0);
}

#[test]
fn an_exact_pending_wait_slot_is_polled_once_and_replayed_as_the_same_event() {
    let mut session = CompilerSession::new();
    let origin = origin(&session, b"subscribe; wait for actual phase; finish");
    let mut cache = EffectReplayCache::default();
    let (child, key);
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) = ready(effects.request(create())) else {
            panic!("actual child identity expected")
        };
        child = id;
        ready(effects.request(CompilerRequest::BeginIntercept {
            workspace: child,
            flags: jai_vm::InterceptFlags::SKIP_ALL,
        }));
        let EffectOutcome::Pending(wait) = effects.request(CompilerRequest::WaitForMessage) else {
            panic!("no compiler work has produced an event")
        };
        key = wait;
        effects.suspend().unwrap();
    }
    assert_eq!(cache.suspended_workspaces(&origin), [child]);
    let event = CompilerEvent::Phase {
        workspace: child,
        phase: jai_vm::CompilerPhase::Typechecked { pending_count: 0 },
    };
    cache
        .publish_suspended_event(&origin, event.clone())
        .unwrap();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin.clone());
        effects.resume().unwrap();
        assert_eq!(
            effects.poll_request(&CompilerRequest::WaitForMessage, key),
            EffectOutcome::Ready(CompilerResponse::Message(event.clone()))
        );
        ready(effects.request(CompilerRequest::EndIntercept { workspace: child }));
        effects.finish(true).unwrap();
    }
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(origin);
        effects.begin();
        assert_eq!(
            ready(effects.request(create())),
            CompilerResponse::Workspace(child)
        );
        ready(effects.request(CompilerRequest::BeginIntercept {
            workspace: child,
            flags: jai_vm::InterceptFlags::SKIP_ALL,
        }));
        assert_eq!(
            ready(effects.request(CompilerRequest::WaitForMessage)),
            CompilerResponse::Message(event)
        );
        ready(effects.request(CompilerRequest::EndIntercept { workspace: child }));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspaces().len(), 2);
}

#[test]
fn actual_child_traces_publish_with_the_parent_and_replay_without_duplicate_effects() {
    let mut session = CompilerSession::new();
    let parent = origin(&session, b"create child; retain exact continuation; finish");
    let mut cache = EffectReplayCache::default();
    let child;
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) = ready(effects.request(create())) else {
            panic!("workspace identity expected")
        };
        child = id;
        ready(effects.request(source(child, "main :: () -> int { return 42; }")));
        effects.suspend().unwrap();
    }
    let mut child_origin = origin(&session, b"actual child compile-time recipe");
    child_origin.workspace = child;
    let mut preview = cache.preview_suspended(&parent, &session).unwrap();
    let mut branch = cache.fork_suspended_replay(&parent).unwrap();
    {
        let mut effects = ReplayEffects::new(&mut preview, &mut branch);
        effects.set_source_origin(child_origin.clone());
        effects.begin();
        ready(effects.request(source(child, "child_ready :: 1;")));
        ready(effects.request(write("child published once")));
        effects.finish(true).unwrap();
    }
    cache
        .prepare_suspended_preview_with_replay(&parent, preview, branch)
        .unwrap();
    assert!(cache.is_empty());
    assert!(session.workspace(child).is_none());
    assert!(session.take_outputs().is_empty());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent.clone());
        effects.resume().unwrap();
        ready(effects.request(write("parent published once")));
        effects.suspend().unwrap();
    }
    let branch = cache.fork_suspended_replay(&parent).unwrap();
    assert_eq!(branch.len(), 1);
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent.clone());
        effects.resume().unwrap();
        effects.finish(true).unwrap();
    }
    assert_eq!(cache.len(), 2);
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 2);
    assert_eq!(session.take_outputs().len(), 2);
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent);
        effects.begin();
        assert_eq!(
            ready(effects.request(create())),
            CompilerResponse::Workspace(child)
        );
        ready(effects.request(source(child, "main :: () -> int { return 42; }")));
        ready(effects.request(write("parent published once")));
        effects.finish(true).unwrap();
        effects.set_source_origin(child_origin);
        effects.begin();
        ready(effects.request(source(child, "child_ready :: 1;")));
        ready(effects.request(write("child published once")));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 2);
    assert!(session.take_outputs().is_empty());
}

#[test]
fn parent_cancellation_discards_actual_child_traces_and_private_outputs() {
    let mut session = CompilerSession::new();
    let parent = origin(&session, b"cancel after actual child recipe");
    let mut cache = EffectReplayCache::default();
    let child;
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) = ready(effects.request(create())) else {
            panic!("workspace identity expected")
        };
        child = id;
        effects.suspend().unwrap();
    }
    let mut preview = cache.preview_suspended(&parent, &session).unwrap();
    let mut branch = cache.fork_suspended_replay(&parent).unwrap();
    let mut child_origin = origin(&session, b"child trace must disappear");
    child_origin.workspace = child;
    {
        let mut effects = ReplayEffects::new(&mut preview, &mut branch);
        effects.set_source_origin(child_origin);
        effects.begin();
        ready(effects.request(write("discard private child output")));
        effects.finish(true).unwrap();
    }
    cache
        .prepare_suspended_preview_with_replay(&parent, preview, branch)
        .unwrap();
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent);
        effects.finish(false).unwrap();
    }
    assert_eq!(cache.suspended_bytes, 0);
    assert!(cache.is_empty());
    assert_eq!(session.workspaces().len(), 1);
    assert!(session.take_outputs().is_empty());
}

#[test]
fn child_preview_replay_authority_and_shared_budget_are_validated_before_adoption() {
    let mut session = CompilerSession::new();
    let parent = origin(&session, b"bounded child preview");
    let mut cache = EffectReplayCache::new(ReplayLimits {
        runs: 1,
        ..ReplayLimits::default()
    });
    let child;
    {
        let mut effects = ReplayEffects::new(&mut session, &mut cache);
        effects.set_source_origin(parent.clone());
        effects.begin();
        let CompilerResponse::Workspace(id) = ready(effects.request(create())) else {
            panic!("workspace identity expected")
        };
        child = id;
        effects.suspend().unwrap();
    }
    let foreign = EffectReplayCache::default().fork_committed();
    let preview = cache.preview_suspended(&parent, &session).unwrap();
    assert!(
        cache
            .prepare_suspended_preview_with_replay(&parent, preview, foreign)
            .is_err()
    );
    let mut preview = cache.preview_suspended(&parent, &session).unwrap();
    let mut branch = cache.fork_suspended_replay(&parent).unwrap();
    let mut child_origin = origin(&session, b"bounded child trace");
    child_origin.workspace = child;
    {
        let mut effects = ReplayEffects::new(&mut preview, &mut branch);
        effects.set_source_origin(child_origin);
        effects.begin();
        ready(effects.request(write("too many combined source recipes")));
        effects.finish(true).unwrap();
    }
    assert!(
        cache
            .prepare_suspended_preview_with_replay(&parent, preview, branch)
            .is_err()
    );
    assert!(cache.is_empty());
    assert_eq!(cache.suspended_jobs(), 1);
    assert!(session.workspace(child).is_none());
    assert!(session.take_outputs().is_empty());
}
