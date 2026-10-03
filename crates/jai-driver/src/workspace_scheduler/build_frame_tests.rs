use super::*;
use jai_types::{Architecture, ByteOrder, LayoutPolicy, OperatingSystem};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-private-build-frame-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn target() -> BuildTarget {
        BuildTarget {
            operating_system: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        }
    }
    fn scheduler(&self) -> WorkspaceScheduler {
        WorkspaceScheduler::new(
            &self.0.join("main.jai"),
            GraphOptions::default(),
            SchedulerOptions {
                target: Self::target(),
                target_triple: TargetTriple::parse("x86_64-unknown-linux").unwrap(),
                compile_time_limits: Limits::default(),
                limits: SchedulerLimits::default(),
                replay_limits: ReplayLimits::default(),
            },
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn typed_source_rounds(body: &str) -> String {
    format!(
        r#"
write_string :: (s: string, to_standard_error := false) #no_context #compiler;
add_build_string :: (s: string, w: s64) #compiler;
#run {{
    write_string("first", false);
    add_build_string("Generated :: struct {{ value: int; }} message :: \"ready\";", -1);
}}
#run {{
    checked: Generated = .{{value = 21}};
    if checked.value != 21 {{ write_string("bad typed value", false); }}
    write_string(message, false);
    add_build_string("answer :: 42;", -1);
}}
main :: () -> int {{ {body} }}
"#
    )
}
fn output(session: &mut CompilerSession) -> Vec<Vec<u8>> {
    session
        .take_outputs()
        .into_iter()
        .map(|output| output.bytes)
        .collect()
}

#[test]
fn genuine_source_prefix_rebuilds_before_a_later_original_typed_run() {
    let fixture = Fixture::new(&typed_source_rounds("return answer;"));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 3);
    let WorkspaceOutput::Checked { unit, library, .. } = build
        .workspaces
        .into_iter()
        .find(|workspace| workspace.id == session.root())
        .unwrap()
        .output
    else {
        panic!("root is checked")
    };
    assert_eq!(unit.sources().len(), 3);
    let entry = jai_sema::select_entry(unit.graph(), &library)
        .unwrap()
        .unwrap();
    let execution = jai_vm::execute(&library.into_program(entry).unwrap(), Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values)
        if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
    assert_eq!(output(&mut session), [b"first".to_vec(), b"ready".to_vec()]);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 2);
    assert_eq!(scheduler.replay_cache().len(), 2);
    scheduler.resolve(&mut session).unwrap();
    assert!(output(&mut session).is_empty());
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 2);
}

#[test]
fn genuine_later_full_failure_discards_earlier_typed_source_rounds() {
    let source = typed_source_rounds("return missing;");
    let missing_start = source.find("return missing;").unwrap() + "return ".len();
    let fixture = Fixture::new(&source);
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("actual missing source name fails");
    let SchedulerError::Driver(Error::Located {
        path, diagnostic, ..
    }) = &error
    else {
        panic!("expected the actual main-body source diagnostic: {error}")
    };
    assert_eq!(path, &fs::canonicalize(fixture.0.join("main.jai")).unwrap());
    assert_eq!(diagnostic.message, "unknown name 'missing'");
    assert_eq!(
        diagnostic.span,
        jai_source::Span::new(missing_start, missing_start + "missing".len())
    );
    assert!(output(&mut session).is_empty());
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
    assert_eq!(session.workspaces().len(), 1);
    assert_eq!(
        session.workspace(session.root()).unwrap().status(),
        jai_vm::WorkspaceStatus::Failed
    );
    assert!(session.error().unwrap().text.contains("missing"));
    assert!(scheduler.replay_cache().is_empty());
    assert!(scheduler.build.is_none());
    assert!(scheduler.children.borrow().is_empty());
}

#[test]
fn plain_driver_session_resolution_discards_completed_run_before_later_body_failure() {
    let fixture = Fixture::new(
        r#"write_string :: (s: string, to_standard_error := false) #no_context #compiler;
#run write_string("private", false); main :: () -> int { return missing; }"#,
    );
    let unit = CompilationUnit::load_with_target(
        &fixture.0.join("main.jai"),
        GraphOptions::default(),
        Fixture::target(),
    )
    .unwrap();
    let mut session = CompilerSession::new();
    let error = unit
        .resolve_library_with_session(LayoutPolicy::lp64(), &mut session)
        .err()
        .expect("actual semantic failure");
    let Error::Located { diagnostic, .. } = &error else {
        panic!("expected the actual later body diagnostic: {error}")
    };
    assert_eq!(diagnostic.message, "unknown name 'missing'");
    assert!(output(&mut session).is_empty());
    assert_eq!(
        session.workspace(session.root()).unwrap().status(),
        jai_vm::WorkspaceStatus::Failed
    );
}

#[test]
fn actual_parent_handles_failed_child_without_observing_any_child_round_effects() {
    let api = include_str!("../../../../tests/fixtures/compiler-message-api.jai");
    let fixture = Fixture::new(&format!(
        r#"{api}
compiler_destroy_workspace :: (w: s64) #compiler;
#run,stallable {{
    child := compiler_create_workspace("failed child");
    add_build_string("write_string :: (s: string, to_standard_error := false) #no_context #compiler; add_build_string :: (s: string, w: s64) #compiler; #run {{ write_string(\"private child\", false); add_build_string(\"Generated :: struct {{ value: int; }}\", -1); }} child :: () -> int {{ return missing; }}", child);
    compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
    while true {{
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE {{
            completed := cast(*Message_Complete) message;
            if completed.error_code != .COMPILATION_FAILED {{ write_string("unexpected receipt", false); }}
            break;
        }}
    }}
    compiler_end_intercept(child);
    compiler_destroy_workspace(child);
    write_string("handled", false);
}}
main :: () -> int {{ return 42; }}"#
    ));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    scheduler.resolve(&mut session).unwrap();
    assert_eq!(output(&mut session), [b"handled".to_vec()]);
    assert_eq!(session.workspaces().len(), 1);
    assert!(session.error().is_none());
    assert_eq!(scheduler.replay_cache().len(), 1);
    assert!(scheduler.children.borrow().is_empty());
}

#[test]
fn repeated_real_pending_keeps_prior_typed_source_round_inside_the_frame() {
    let api = include_str!("../../../../tests/fixtures/compiler-message-api.jai");
    let fixture = Fixture::new(&format!(
        r#"{api}
#run {{ write_string("private first", false); add_build_string("Generated :: struct {{ value: int; }}", -1); }}
#run,stallable {{
    value: Generated = .{{value = 42}};
    if value.value != 42 {{ write_string("bad pending typed value", false); }}
    child := compiler_create_workspace("empty child");
    compiler_begin_intercept(child, Intercept_Flags.SKIP_ALL);
    compiler_wait_for_message();
    compiler_end_intercept(child);
}}
main :: () -> int {{ return 42; }}"#
    ));
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let SchedulerReadiness::Pending(first) = scheduler.resolve_resumable(&mut session).unwrap()
    else {
        panic!("empty child waits")
    };
    let SchedulerReadiness::Pending(second) = scheduler.resolve_resumable(&mut session).unwrap()
    else {
        panic!("same actual checkpoint waits")
    };
    assert_eq!(first.source.dependencies, second.source.dependencies);
    assert_eq!(scheduler.build.as_ref().unwrap().state.pass, 2);
    assert!(output(&mut session).is_empty());
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
    assert_eq!(session.workspaces().len(), 1);
    assert!(scheduler.replay_cache().is_empty());
    scheduler.cancel_pending().unwrap();
    assert!(scheduler.children.borrow().is_empty());
    assert!(output(&mut session).is_empty());
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
}
