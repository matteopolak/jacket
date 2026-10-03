use super::*;
use jai_types::{
    Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-owned-source-job-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("main.jai");
        fs::write(&path, source).unwrap();
        Self(path)
    }

    fn target() -> BuildTarget {
        BuildTarget {
            operating_system: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            byte_order: ByteOrder::Little,
            layout: LayoutPolicy::new(
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
            .unwrap(),
        }
    }

    fn scheduler(&self) -> crate::WorkspaceScheduler {
        crate::WorkspaceScheduler::new(
            &self.0,
            Default::default(),
            crate::SchedulerOptions {
                target: Self::target(),
                target_triple: jai_vm::TargetTriple::parse("x86_64-unknown-linux").unwrap(),
                compile_time_limits: Default::default(),
                limits: Default::default(),
                replay_limits: Default::default(),
            },
        )
        .unwrap()
    }

    fn job(&self, compiler: CompilerSession) -> PreparedWorkspaceJob {
        self.job_with_replay(compiler, EffectReplayCache::default())
    }
    fn job_with_replay(
        &self,
        compiler: CompilerSession,
        replay: EffectReplayCache,
    ) -> PreparedWorkspaceJob {
        let target = Self::target();
        let unit =
            CompilationUnit::load_with_target(&self.0, Default::default(), target.clone()).unwrap();
        let options = ResolveOptions {
            target: Some(target),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                unit.graph(),
                &[],
                compiler.root(),
            )),
            ..Default::default()
        };
        PreparedWorkspaceJob::new(unit, options, compiler, replay)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

#[test]
fn owned_future_publishes_the_actual_library_and_private_source_journal() {
    let fixture = Fixture::new(
        "write_string :: (s: string, to_standard_error := false) #no_context #compiler; #run write_string(\"owned once\"); seed :: () -> int { return 42; } answer :: #run seed(); main :: () -> int { return answer; }",
    );
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    let WorkspaceJobProgress::Complete(mut result) = job.poll().unwrap() else {
        panic!("ready authored source must complete")
    };
    assert!(original.take_outputs().is_empty());
    assert_eq!(result.compiler.take_outputs()[0].bytes, b"owned once");
    assert_eq!(result.replay.len(), 1);
    assert!(job.poll().is_err());
    drop(job);
    let entry = jai_sema::select_entry(result.unit.graph(), &result.library)
        .unwrap()
        .unwrap();
    let program = result.library.into_program(entry).unwrap();
    let execution = jai_vm::execute(&program, Default::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42)),
        "{execution:?}"
    );
}

#[test]
fn cancel_before_poll_does_not_run_source_or_publish_journals() {
    let fixture = Fixture::new(
        "write_string :: (s: string, to_standard_error := false) #no_context #compiler; #run write_string(\"must not run\"); main :: () {}",
    );
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    job.cancel().unwrap();
    job.cancel().unwrap();
    assert!(job.poll().is_err());
    assert!(job.suspended_origins().is_empty());
    assert!(original.take_outputs().is_empty());
}

#[test]
fn real_semantic_failure_preserves_source_diagnostics_and_drops_private_effects() {
    let fixture = Fixture::new(
        "write_string :: (s: string, to_standard_error := false) #no_context #compiler; #run write_string(\"private\"); main :: () -> int { return missing; }",
    );
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    let WorkspaceJobProgress::Failed(error) = job.poll().unwrap() else {
        panic!("actual missing source name must fail")
    };
    let canonical = fs::canonicalize(&fixture.0).unwrap();
    assert!(matches!(error, Error::Located { ref path, .. } if path == &canonical));
    assert!(error.to_string().contains("missing"));
    assert!(original.take_outputs().is_empty());
    assert!(job.suspended_origins().is_empty());
    assert!(job.poll().is_err());
}

fn stallable_fixture() -> Fixture {
    let api = include_str!("../../../../tests/fixtures/compiler-message-api.jai");
    Fixture::new(&format!(
        r#"{api}
#run,stallable {{
    count := 0;
    count += 1;
    write_string("before wait");
    child := compiler_create_workspace("retained child");
    add_build_string("write_string :: (s: string, to_standard_error := false) #no_context #compiler; #run write_string(\"child once\"); child :: () -> int {{ return 42; }}", child);
    compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
    message := compiler_wait_for_message();
    phase := cast(*Message_Phase) message;
    if count != 1 {{ write_string("bad replay"); }}
    if phase.phase != .ALL_SOURCE_CODE_PARSED {{ write_string("bad phase"); }}
    write_string("after wait");
    compiler_end_intercept(child);
}}
main :: () -> int {{ return 42; }}
"#,
    ))
}

#[test]
fn actual_stallable_source_resumes_the_owned_job_after_real_child_readiness() {
    let fixture = stallable_fixture();
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    let WorkspaceJobProgress::Pending(pending) = job.poll().unwrap() else {
        panic!("actual source wait must park")
    };
    assert!(matches!(
        pending.dependencies.as_slice(),
        [jai_vm::Dependency::Effect(_)]
    ));
    assert_eq!(job.suspended_origins().len(), 1);
    assert_eq!(original.workspaces().count(), 1);
    assert!(original.take_outputs().is_empty());
    let mut scheduler = fixture.scheduler();
    assert!(job.service_pending(&mut scheduler).unwrap());
    let WorkspaceJobProgress::SourceRebuild(rebuild) = job.poll().unwrap() else {
        panic!("completed real prefix exposes its actual child source configuration")
    };
    let WorkspaceSourceRebuild {
        unit: _,
        compiler,
        replay,
    } = *rebuild;
    drop(job);
    let mut rebound = fixture.job_with_replay(compiler, replay);
    let WorkspaceJobProgress::Complete(mut result) = rebound.poll().unwrap() else {
        panic!("source-owned replay reaches Full without repeating the prior prefix")
    };
    assert_eq!(result.compiler.workspaces().count(), 2);
    let output: Vec<_> = result
        .compiler
        .take_outputs()
        .into_iter()
        .map(|output| output.bytes)
        .collect();
    assert_eq!(
        output,
        [
            b"before wait".to_vec(),
            b"child once".to_vec(),
            b"after wait".to_vec()
        ]
    );
    assert_eq!(result.replay.len(), 2);
    assert_eq!(result.replay.suspended_jobs(), 0);
    assert!(original.take_outputs().is_empty());
}

#[test]
fn cancelling_actual_stallable_source_drops_its_exact_private_checkpoint() {
    let fixture = stallable_fixture();
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    assert!(matches!(
        job.poll().unwrap(),
        WorkspaceJobProgress::Pending(_)
    ));
    let state = Rc::clone(&job.state);
    drop(job);
    let state = state.borrow();
    assert!(state.cancellation_error.is_none());
    let journal = state.journal.as_ref().unwrap();
    assert_eq!(journal.replay.suspended_jobs(), 0);
    assert!(journal.replay.is_empty());
    assert_eq!(journal.compiler.workspaces().count(), 1);
    assert!(original.take_outputs().is_empty());
}
