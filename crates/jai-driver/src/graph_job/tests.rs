use super::*;
use jai_modules::{BootstrapOptions, GraphOptions};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(child: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-owned-graph-job-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("main.jai");
        let api = include_str!("../../../../tests/fixtures/compiler-message-api.jai");
        let child_source = if child.is_empty() {
            String::new()
        } else {
            format!("add_build_string(\"{child}\", child);")
        };
        fs::write(
            &path,
            format!(
                r#"{api}
            calls:int;
            choose::()->bool {{
                calls+=1;
                write_string("guard before");
                child:=compiler_create_workspace("guard child");
                {child_source}
                compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
                message:=compiler_wait_for_message();
                phase:=cast(*Message_Phase) message;
                write_string("guard after");
                compiler_end_intercept(child);
                return calls==1 && phase.phase==.ALL_SOURCE_CODE_PARSED;
            }}
            #if #run,stallable choose() {{ #load "selected.jai"; }} else {{ #load "absent.jai"; }}
            main::()->int {{return ANSWER;}}
        "#
            ),
        )
        .unwrap();
        fs::write(directory.join("selected.jai"), "ANSWER::42;").unwrap();
        Self(path)
    }
    fn target() -> BuildTarget {
        BuildTarget {
            operating_system: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            byte_order: ByteOrder::Little,
            layout: LayoutPolicy::lp64(),
        }
    }
    fn job(&self, compiler: CompilerSession) -> PreparedGraphJob {
        let options = SemanticDiscoveryOptions {
            graph: GraphOptions::default(),
            bootstrap: BootstrapOptions::disabled(),
            target: Self::target(),
            workspace: compiler.root(),
            limits: Default::default(),
            effect_policy: DiscoveryEffectPolicy::CompilerSession,
        };
        PreparedGraphJob::new(
            self.0.clone(),
            options,
            jai_modules::Filesystem,
            compiler,
            EffectReplayCache::default(),
        )
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
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(self.0.parent().unwrap());
    }
}

#[test]
fn owned_graph_guard_resumes_before_loading_only_the_selected_file() {
    let fixture = Fixture::new("child::()->int{return 42;}");
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    assert!(matches!(job.poll().unwrap(), GraphJobProgress::Pending(_)));
    assert_eq!(job.suspended_origins().len(), 1);
    assert_eq!(original.workspaces().count(), 1);
    assert!(original.take_outputs().is_empty());
    assert!(job.service_pending(&mut fixture.scheduler()).unwrap());
    let GraphJobProgress::Complete(mut result) = job.poll().unwrap() else {
        panic!("same graph guard must complete")
    };
    assert!(
        result
            .unit
            .graph()
            .sources()
            .records()
            .iter()
            .all(|source| !source.path().ends_with("absent.jai"))
    );
    assert_eq!(result.compiler.workspaces().count(), 2);
    let output: Vec<_> = result
        .compiler
        .take_outputs()
        .into_iter()
        .map(|output| output.bytes)
        .collect();
    assert_eq!(output, [b"guard before".to_vec(), b"guard after".to_vec()]);
    assert_eq!(result.replay.suspended_jobs(), 0);
    let program = result
        .unit
        .resolve_discovered_with_session(
            LayoutPolicy::lp64(),
            &mut result.compiler,
            &mut result.replay,
            DiscoveryEffectPolicy::CompilerSession,
        )
        .unwrap();
    assert!(
        result.compiler.take_outputs().is_empty(),
        "final binding must reuse the committed source guard"
    );
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("VM failed")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    assert!(job.poll().is_err());
}

#[test]
fn cancel_owned_graph_guard_discards_the_exact_checkpoint_and_private_inputs() {
    let fixture = Fixture::new("");
    let mut original = CompilerSession::new();
    let mut job = fixture.job(original.clone());
    let GraphJobProgress::Pending(first) = job.poll().unwrap() else {
        panic!("empty child cannot produce readiness")
    };
    let GraphJobProgress::Pending(second) = job.poll().unwrap() else {
        panic!("guard must stay pending")
    };
    assert_eq!(first.dependencies, second.dependencies);
    assert_eq!(job.suspended_origins().len(), 1);
    let state = Rc::clone(&job.state);
    drop(job);
    let mut state = state.borrow_mut();
    assert!(state.cancellation_error.is_none());
    let journal = state.journal.as_mut().unwrap();
    assert_eq!(journal.replay.suspended_jobs(), 0);
    assert_eq!(journal.replay.len(), 0);
    assert_eq!(journal.compiler.workspaces().count(), 1);
    assert!(journal.compiler.take_outputs().is_empty());
    assert!(original.take_outputs().is_empty());
}

#[test]
fn workspace_scheduler_services_a_real_guard_before_body_binding() {
    let fixture = Fixture::new("child::()->int{return 42;}");
    let mut compiler = CompilerSession::new();
    let mut scheduler = fixture.scheduler();
    let build = scheduler.resolve(&mut compiler).unwrap();
    assert_eq!(build.workspaces.len(), 2);
    let output: Vec<_> = compiler
        .take_outputs()
        .into_iter()
        .map(|output| output.bytes)
        .collect();
    assert_eq!(output, [b"guard before".to_vec(), b"guard after".to_vec()]);
    assert_eq!(scheduler.replay_cache().suspended_jobs(), 0);
}

#[test]
fn workspace_scheduler_retains_pending_discovery_and_cancels_without_graph_advance() {
    let fixture = Fixture::new("");
    let mut compiler = CompilerSession::new();
    let mut scheduler = fixture.scheduler();
    let crate::SchedulerReadiness::Pending(first) =
        scheduler.resolve_resumable(&mut compiler).unwrap()
    else {
        panic!("empty child must leave a live guard")
    };
    let crate::SchedulerReadiness::Pending(second) =
        scheduler.resolve_resumable(&mut compiler).unwrap()
    else {
        panic!("same guard remains live")
    };
    assert_eq!(first.source.dependencies, second.source.dependencies);
    assert!(compiler.take_outputs().is_empty());
    assert_eq!(compiler.workspaces().count(), 1);
    scheduler.cancel_pending().unwrap();
    assert_eq!(scheduler.replay_cache().suspended_jobs(), 0);
}
