use super::*;
use crate::ReplayEffects;
use jai_types::{Architecture, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout};
use jai_vm::{
    CompilerEffects, CompilerRequest, CompilerResponse, EffectKey, EffectOutcome, InterceptFlags,
};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-retained-child-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("main.jai"), "main :: () {}").unwrap();
        Self(path)
    }
    fn scheduler(&self) -> WorkspaceScheduler {
        let layout = LayoutPolicy::new(
            ScalarLayout::new(8, 8),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 8),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
            ScalarLayout::new(1, 1),
        )
        .unwrap();
        WorkspaceScheduler::new(
            &self.0.join("main.jai"),
            GraphOptions::default(),
            SchedulerOptions {
                target: BuildTarget {
                    operating_system: OperatingSystem::Linux,
                    architecture: Architecture::X86_64,
                    layout,
                    byte_order: ByteOrder::Little,
                },
                target_triple: TargetTriple::parse("x86_64-unknown-linux").unwrap(),
                compile_time_limits: Limits::default(),
                limits: SchedulerLimits::default(),
                replay_limits: ReplayLimits::default(),
            },
        )
        .unwrap()
    }
    fn with_source(source: &str) -> Self {
        let fixture = Self::new();
        fs::write(fixture.0.join("main.jai"), source).unwrap();
        fixture
    }
}

fn root_wait_source(no_output: bool) -> String {
    let api = include_str!("../../../../tests/fixtures/compiler-message-api.jai");
    let policy = if no_output {
        "add_build_string(\"child :: () -> int { return 42; }\", child); set_build_options(Build_Options.{output_type = .NO_OUTPUT}, child);"
    } else {
        ""
    };
    format!(
        r#"{api}
Output_Type :: enum u8 {{ NO_OUTPUT; EXECUTABLE; DYNAMIC_LIBRARY; STATIC_LIBRARY; OBJECT_FILE; }}
Build_Options :: struct {{ output_type: Output_Type = .EXECUTABLE; }}
set_build_options :: (options: Build_Options, w: s64 = -1) #compiler;
#run,stallable {{
    write_string("root before");
    child := compiler_create_workspace("root child");
    {policy}
    compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
    while true {{
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {{ break; }}
    }}
    write_string("root after");
    compiler_end_intercept(child);
}}
main :: () -> int {{ return 42; }}
"#
    )
}

#[test]
fn root_scheduler_retains_a_real_pending_source_job_across_drives_and_cancels_it() {
    let fixture = Fixture::with_source(&root_wait_source(false));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let SchedulerReadiness::Pending(first) = scheduler.resolve_resumable(&mut session).unwrap()
    else {
        panic!("empty child has no event yet")
    };
    let origin = scheduler
        .build
        .as_ref()
        .unwrap()
        .state
        .active
        .as_ref()
        .unwrap()
        .job
        .suspended_origins()[0]
        .clone();
    let SchedulerReadiness::Pending(second) = scheduler.resolve_resumable(&mut session).unwrap()
    else {
        panic!("the same issued wait remains pending")
    };
    assert_eq!(first.source.dependencies, second.source.dependencies);
    assert_eq!(
        scheduler
            .build
            .as_ref()
            .unwrap()
            .state
            .active
            .as_ref()
            .unwrap()
            .job
            .suspended_origins()[0],
        origin
    );
    assert_eq!(session.workspaces().count(), 1);
    assert!(session.take_outputs().is_empty());
    assert!(scheduler.replay_cache().is_empty());
    scheduler.cancel_pending().unwrap();
    assert!(scheduler.build.is_none());
    assert_eq!(session.workspaces().count(), 1);
    assert!(session.take_outputs().is_empty());
}

#[test]
fn root_scheduler_finishes_real_no_output_child_and_resumes_the_waiting_source_once() {
    let fixture = Fixture::with_source(&root_wait_source(true));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let SchedulerReadiness::Complete(build) = scheduler.resolve_resumable(&mut session).unwrap()
    else {
        panic!("real NO_OUTPUT child finishes its configured job")
    };
    assert_eq!(build.workspaces.len(), 2);
    assert_eq!(build.passes, 2);
    assert!(
        build
            .workspaces
            .iter()
            .all(|workspace| matches!(workspace.output, WorkspaceOutput::Checked { .. }))
    );
    let output: Vec<_> = session
        .take_outputs()
        .into_iter()
        .map(|output| output.bytes)
        .collect();
    assert_eq!(output, [b"root before".to_vec(), b"root after".to_vec()]);
    assert_eq!(scheduler.replay_cache().len(), 1);
    assert!(scheduler.build.is_none());
}

#[test]
fn root_scheduler_rejects_committed_session_changes_while_a_source_job_is_pending() {
    let fixture = Fixture::with_source(&root_wait_source(false));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    assert!(matches!(
        scheduler.resolve_resumable(&mut session).unwrap(),
        SchedulerReadiness::Pending(_)
    ));
    session.begin();
    let root = session.root();
    ready(session.request(CompilerRequest::SetBuildOption {
        workspace: root,
        option: jai_vm::BuildOption::OutputPath("new-original-output".into()),
    }));
    session.finish(true).unwrap();
    assert!(matches!(
        scheduler.resolve_resumable(&mut session),
        Err(SchedulerError::ChangedPendingSession)
    ));
    assert!(scheduler.build.is_none());
    assert_eq!(session.workspaces().count(), 1);
    assert_eq!(
        session
            .workspace(root)
            .unwrap()
            .settings()
            .output_path
            .as_deref(),
        Some(Path::new("new-original-output"))
    );
    assert!(session.take_outputs().is_empty());
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn ready(outcome: EffectOutcome) -> CompilerResponse {
    let EffectOutcome::Ready(response) = outcome else {
        panic!("{outcome:?}")
    };
    response
}
fn park_child(
    scheduler: &mut WorkspaceScheduler,
    session: &mut CompilerSession,
    source: &str,
) -> (SourceOrigin, WorkspaceId, EffectKey) {
    park_child_with_output(scheduler, session, source, None)
}

fn park_child_with_output(
    scheduler: &mut WorkspaceScheduler,
    session: &mut CompilerSession,
    source: &str,
    output: Option<jai_types::BuildOutputKind>,
) -> (SourceOrigin, WorkspaceId, EffectKey) {
    let origin = SourceOrigin {
        workspace: session.root(),
        path: scheduler.root.clone(),
        start: 0,
        end: 12,
        body_hash: 1,
        body: b"actual parent".to_vec(),
        specialization: vec![],
    };
    let mut effects = ReplayEffects::new(session, &mut scheduler.replay);
    effects.set_source_origin(origin.clone());
    effects.begin();
    let CompilerResponse::Workspace(child) =
        ready(effects.request(CompilerRequest::CreateWorkspace {
            name: "actual child".into(),
        }))
    else {
        panic!("child identity expected")
    };
    if !source.is_empty() {
        ready(effects.request(CompilerRequest::AddSource {
            workspace: child,
            source: source.into(),
        }));
    }
    if let Some(output) = output {
        ready(effects.request(CompilerRequest::SetBuildOption {
            workspace: child,
            option: jai_vm::BuildOption::OutputKind(output),
        }));
    }
    ready(effects.request(CompilerRequest::BeginIntercept {
        workspace: child,
        flags: InterceptFlags::SKIP_ALL,
    }));
    let EffectOutcome::Pending(key) = effects.request(CompilerRequest::WaitForMessage) else {
        panic!("child work has not run")
    };
    effects.suspend().unwrap();
    (origin, child, key)
}

#[test]
fn explicit_no_output_completes_only_after_the_actual_source_recipe_fixed_point() {
    let fixture = Fixture::new();
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let source = "write_string :: (s: string, to_standard_error := false) #no_context #compiler; add_build_string :: (s: string, w: s64) #compiler; #run write_string(\"no output recipe\"); #run add_build_string(\"answer :: 42;\", -1); child :: () -> int { return answer; }";
    let (origin, child, key) = park_child_with_output(
        &mut scheduler,
        &mut session,
        source,
        Some(jai_types::BuildOutputKind::None),
    );
    assert!(scheduler.service_suspended(&mut session, &origin).unwrap());
    assert!(session.workspace(child).is_none());
    assert!(session.take_outputs().is_empty());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin);
        effects.resume().unwrap();
        assert_eq!(
            effects.poll_request(&CompilerRequest::WaitForMessage, key),
            EffectOutcome::Ready(CompilerResponse::Message(CompilerEvent::Phase {
                workspace: child,
                phase: CompilerPhase::SourceParsed
            }))
        );
        assert_eq!(
            ready(effects.request(CompilerRequest::WaitForMessage)),
            CompilerResponse::Message(CompilerEvent::Phase {
                workspace: child,
                phase: CompilerPhase::Typechecked {
                    pending_count: 0
                }
            })
        );
        assert_eq!(
            ready(effects.request(CompilerRequest::WaitForMessage)),
            CompilerResponse::Message(CompilerEvent::Complete {
                workspace: child,
                error: jai_vm::CompilerCompletion::None
            })
        );
        ready(effects.request(CompilerRequest::EndIntercept {
            workspace: child,
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(session.workspace(child).unwrap().inputs().len(), 2);
    assert_eq!(session.take_outputs()[0].bytes, b"no output recipe");
}

#[test]
fn actual_child_source_readiness_and_effects_remain_private_until_parent_publication() {
    let fixture = Fixture::new();
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let source = "write_string :: (s: string, to_standard_error := false) #no_context #compiler; #run write_string(\"child once\"); child :: () -> int { return 42; }";
    let (origin, child, key) = park_child(&mut scheduler, &mut session, source);
    assert!(scheduler.service_suspended(&mut session, &origin).unwrap());
    assert!(session.workspace(child).is_none());
    assert!(session.take_outputs().is_empty());
    assert!(scheduler.replay_cache().is_empty());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin);
        effects.resume().unwrap();
        assert_eq!(
            effects.poll_request(&CompilerRequest::WaitForMessage, key),
            EffectOutcome::Ready(CompilerResponse::Message(CompilerEvent::Phase {
                workspace: child,
                phase: CompilerPhase::SourceParsed
            }))
        );
        assert_eq!(
            ready(effects.request(CompilerRequest::WaitForMessage)),
            CompilerResponse::Message(CompilerEvent::Phase {
                workspace: child,
                phase: CompilerPhase::Typechecked {
                    pending_count: 0
                }
            })
        );
        ready(effects.request(CompilerRequest::EndIntercept {
            workspace: child,
        }));
        effects.finish(true).unwrap();
    }
    assert_eq!(scheduler.replay_cache().len(), 2);
    assert_eq!(session.take_outputs()[0].bytes, b"child once");
    let build = scheduler.resolve(&mut session).unwrap();
    assert!(matches!(
        build.workspace(child).unwrap().output,
        WorkspaceOutput::Checked { .. }
    ));
    assert!(session.take_outputs().is_empty());
}

#[test]
fn child_generated_source_reaches_its_real_fixed_point_in_private_staging() {
    let fixture = Fixture::new();
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let source = "add_build_string :: (s: string, w: s64) #compiler; #run add_build_string(\"answer :: 42;\", -1); child :: () -> int { return answer; }";
    let (origin, child, _) = park_child(&mut scheduler, &mut session, source);
    assert!(scheduler.service_suspended(&mut session, &origin).unwrap());
    let preview = scheduler
        .replay
        .preview_suspended(&origin, &session)
        .unwrap();
    assert_eq!(preview.workspace(child).unwrap().inputs().len(), 2);
    assert!(session.workspace(child).is_none());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin);
        effects.finish(false).unwrap();
    }
    assert_eq!(session.workspaces().len(), 1);
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn a_checked_library_never_fabricates_successful_native_completion() {
    let fixture = Fixture::new();
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let (origin, child, first_key) = park_child(
        &mut scheduler,
        &mut session,
        "child :: () -> int { return 42; }",
    );
    assert!(scheduler.service_suspended(&mut session, &origin).unwrap());
    let second_key;
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin.clone());
        effects.resume().unwrap();
        ready(effects.poll_request(&CompilerRequest::WaitForMessage, first_key));
        ready(effects.request(CompilerRequest::WaitForMessage));
        let EffectOutcome::Pending(key) = effects.request(CompilerRequest::WaitForMessage) else {
            panic!("native completion has not happened")
        };
        second_key = key;
        effects.suspend().unwrap();
    }
    assert!(!scheduler.service_suspended(&mut session, &origin).unwrap());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin);
        effects.resume().unwrap();
        assert_eq!(
            effects.poll_request(&CompilerRequest::WaitForMessage, second_key),
            EffectOutcome::Pending(second_key)
        );
        effects.finish(false).unwrap();
    }
    assert!(session.workspace(child).is_none());
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn an_empty_child_waits_and_actual_failed_child_emits_failure_before_parent_rollback() {
    for source in ["", "child :: () -> int { return missing; }"] {
        let fixture = Fixture::new();
        let mut scheduler = fixture.scheduler();
        let mut session = CompilerSession::new();
        let (origin, child, key) = park_child(&mut scheduler, &mut session, source);
        let outcome = scheduler.service_suspended(&mut session, &origin);
        if source.is_empty() {
            assert!(!outcome.unwrap());
        } else {
            assert!(outcome.unwrap());
            let preview = scheduler
                .replay
                .preview_suspended(&origin, &session)
                .unwrap();
            assert_eq!(
                preview.workspace(child).unwrap().status(),
                jai_vm::WorkspaceStatus::Failed
            );
            assert!(preview.error().unwrap().text.contains("missing"));
        }
        {
            let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
            effects.set_source_origin(origin);
            if !source.is_empty() {
                effects.resume().unwrap();
                assert_eq!(
                    effects.poll_request(&CompilerRequest::WaitForMessage, key),
                    EffectOutcome::Ready(CompilerResponse::Message(CompilerEvent::Phase {
                        workspace: child,
                        phase: CompilerPhase::SourceParsed,
                    }))
                );
                assert_eq!(
                    ready(effects.request(CompilerRequest::WaitForMessage)),
                    CompilerResponse::Message(CompilerEvent::Complete {
                        workspace: child,
                        error: jai_vm::CompilerCompletion::CompilationFailed,
                    })
                );
            }
            effects.finish(false).unwrap();
        }
        assert!(session.workspace(child).is_none());
        assert!(scheduler.replay_cache().is_empty());
    }
}

#[test]
fn cancelled_actual_child_produces_shutdown_without_running_its_source() {
    let fixture = Fixture::new();
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let (origin, child, key) = park_child(
        &mut scheduler,
        &mut session,
        "this source should never parse;",
    );
    let mut preview = scheduler
        .replay
        .preview_suspended(&origin, &session)
        .unwrap();
    preview.begin();
    ready(preview.request(CompilerRequest::DestroyWorkspace {
        workspace: child,
    }));
    preview.finish(true).unwrap();
    scheduler
        .replay
        .prepare_suspended_preview(&origin, preview)
        .unwrap();
    assert!(scheduler.service_suspended(&mut session, &origin).unwrap());
    {
        let mut effects = ReplayEffects::new(&mut session, &mut scheduler.replay);
        effects.set_source_origin(origin);
        effects.resume().unwrap();
        assert_eq!(
            effects.poll_request(&CompilerRequest::WaitForMessage, key),
            EffectOutcome::Ready(CompilerResponse::Message(CompilerEvent::Complete {
                workspace: child,
                error: jai_vm::CompilerCompletion::CompilerShutdown
            }))
        );
        ready(effects.request(CompilerRequest::EndIntercept {
            workspace: child,
        }));
        effects.finish(true).unwrap();
    }
    assert!(session.is_destroyed(child));
    assert_eq!(session.workspaces().len(), 1);
}
