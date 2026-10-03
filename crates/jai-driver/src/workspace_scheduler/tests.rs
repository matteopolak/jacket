use super::*;
use jai_types::{Architecture, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout};
use jai_vm::{CompilerEffects, CompilerRequest, CompilerResponse, EffectOutcome};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "jai-workspace-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn scheduler(&self) -> WorkspaceScheduler {
        WorkspaceScheduler::new(&self.0.join("main.jai"), GraphOptions::default(), options())
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn options() -> SchedulerOptions {
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
    }
}
fn stage(session: &mut CompilerSession, request: CompilerRequest) -> CompilerResponse {
    session.begin();
    let response = match session.request(request) {
        EffectOutcome::Ready(response) => response,
        other => panic!("{other:?}"),
    };
    session.finish(true).unwrap();
    response
}
fn checked(build: &ScheduledBuild, id: WorkspaceId) -> (&CompilationUnit, &jai_sema::Library) {
    match &build.workspace(id).unwrap().output {
        WorkspaceOutput::Checked {
            unit,
            library,
            ..
        } => (unit, library),
        WorkspaceOutput::AwaitingInputs => panic!("expected checked graph"),
    }
}

#[test]
fn run_generated_source_is_resolved_and_rebuild_does_not_duplicate_effects() {
    let fixture = Fixture::new(
        "add_build_string :: (data: string, w: s64) #compiler;\n#run add_build_string(\"answer :: 42;\", -1);\nmain :: () -> int { return answer; }",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    let (unit, library) = checked(&build, session.root());
    assert_eq!(unit.sources().len(), 2);
    assert_eq!(library.procedures().len(), 1);
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    let executed = vm.execute(library.procedures()[0].id, vec![]);
    assert!(
        matches!(executed.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
    assert_eq!(scheduler.replay_cache().len(), 1);
    let again = scheduler.resolve(&mut session).unwrap();
    assert_eq!(again.passes, 1);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn source_created_workspace_keeps_own_current_identity_and_graph() {
    let fixture = Fixture::new(
        "compiler_create_workspace :: (name: string) -> s64 #compiler;\nadd_build_string :: (data: string, w: s64) #compiler;\nrecipe :: () { w := compiler_create_workspace(\"child\"); add_build_string(\"get_current_workspace :: () -> s64 #compiler; child_id :: #run get_current_workspace(); child :: () -> s64 { return child_id; }\", w); }\n#run recipe();\nmain :: () {}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.workspaces.len(), 2);
    let child = build
        .workspaces
        .iter()
        .find(|workspace| workspace.id != session.root())
        .unwrap();
    let (unit, library) = checked(&build, child.id);
    assert_eq!(library.procedures().len(), 1);
    assert!(
        unit.sources()
            .iter()
            .any(|source| source.text().contains("child_id"))
    );
    assert_eq!(session.workspaces().len(), 2);
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    let executed = vm.execute(library.procedures()[0].id, vec![]);
    assert_eq!(
        executed.outcome,
        jai_vm::Outcome::Complete(vec![jai_vm::Value::Int(
            jai_types::Integer::checked(jai_types::IntegerType::S64, i128::from(child.id.get()))
                .unwrap()
        )])
    );
}

#[test]
fn committed_file_inputs_load_real_scopes_and_duplicate_paths_deduplicate() {
    let fixture = Fixture::new("main :: () -> int { return answer(); }");
    fs::write(
        fixture.0.join("extra.jai"),
        "value :: 42; answer :: () -> int { return value; }",
    )
    .unwrap();
    let mut session = CompilerSession::new();
    let root = session.root();
    for path in ["extra.jai", "./extra.jai"] {
        stage(
            &mut session,
            CompilerRequest::AddSourceFile {
                workspace: root,
                path: path.into(),
            },
        );
    }
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    let (unit, library) = checked(&build, root);
    assert_eq!(unit.sources().len(), 2);
    assert_eq!(library.procedures().len(), 2);
}

#[test]
fn located_file_input_resolves_against_its_recipe_directory() {
    let fixture = Fixture::new("main :: () -> int { return extra; }");
    fs::create_dir_all(fixture.0.join("recipe")).unwrap();
    fs::write(fixture.0.join("recipe/extra.jai"), "extra :: 9;").unwrap();
    let mut session = CompilerSession::new();
    let root = session.root();
    stage(
        &mut session,
        CompilerRequest::AddSourceFileAt {
            workspace: root,
            path: "extra.jai".into(),
            location: jai_vm::SourceLocation {
                path: "recipe/build.jai".into(),
                line: 2,
                column: 3,
            },
        },
    );
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    let (unit, _) = checked(&build, root);
    assert!(
        unit.sources()
            .iter()
            .any(|source| source.path().ends_with("recipe/extra.jai"))
    );
}

#[test]
fn generated_parse_and_semantic_errors_retain_their_virtual_source() {
    for source in [
        "bad :: () { return +; }",
        "bad :: () -> int { return missing; }",
    ] {
        let fixture = Fixture::new("main :: () {}");
        let mut session = CompilerSession::new();
        let root = session.root();
        stage(
            &mut session,
            CompilerRequest::AddSource {
                workspace: root,
                source: source.into(),
            },
        );
        let error = fixture
            .scheduler()
            .resolve(&mut session)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains(".jai-generated-"), "{error}");
    }
}

#[test]
fn physical_edits_require_a_fresh_session_and_unknown_target_is_rejected() {
    let fixture = Fixture::new("main :: () {}");
    let mut session = CompilerSession::new();
    let mut scheduler = fixture.scheduler();
    scheduler.resolve(&mut session).unwrap();
    fs::write(fixture.0.join("main.jai"), "main :: () { x := 1; }").unwrap();
    assert!(matches!(
        scheduler.resolve(&mut session),
        Err(SchedulerError::PhysicalSourceChanged(_))
    ));
    let mut session = CompilerSession::new();
    let root = session.root();
    stage(
        &mut session,
        CompilerRequest::SetBuildOption {
            workspace: root,
            option: jai_vm::BuildOption::Target(
                TargetTriple::parse("aarch64-apple-darwin").unwrap(),
            ),
        },
    );
    assert!(matches!(
        fixture.scheduler().resolve(&mut session),
        Err(SchedulerError::UnsupportedTarget { .. })
    ));
}

#[test]
fn empty_children_are_explicitly_awaiting_inputs_and_scheduler_is_bounded() {
    let fixture = Fixture::new("main :: () {}");
    let mut session = CompilerSession::new();
    let child = match stage(
        &mut session,
        CompilerRequest::CreateWorkspace {
            name: "empty".into(),
        },
    ) {
        CompilerResponse::Workspace(id) => id,
        _ => panic!(),
    };
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert!(matches!(
        build.workspace(child).unwrap().output,
        WorkspaceOutput::AwaitingInputs
    ));
    let mut configuration = options();
    configuration.limits.workspaces = 1;
    let mut scheduler = WorkspaceScheduler::new(
        &fixture.0.join("main.jai"),
        GraphOptions::default(),
        configuration,
    )
    .unwrap();
    assert!(matches!(
        scheduler.resolve(&mut session),
        Err(SchedulerError::Limit("workspaces"))
    ));
}

#[test]
fn recursively_created_workspace_recipes_stop_at_the_round_limit() {
    let fixture = Fixture::new("#load \"recipe.jai\"; #run spawn(); main :: () {}");
    fs::write(fixture.0.join("recipe.jai"), "compiler_create_workspace :: (name: string) -> s64 #compiler; add_build_file :: (filename: string, w: s64) #compiler; add_build_string :: (data: string, w: s64) #compiler; spawn :: () { w := compiler_create_workspace(\"child\"); add_build_file(\"recipe.jai\", w); add_build_string(\"#run spawn();\", w); }").unwrap();
    let mut configuration = options();
    configuration.limits.passes = 2;
    let mut scheduler = WorkspaceScheduler::new(
        &fixture.0.join("main.jai"),
        GraphOptions::default(),
        configuration,
    )
    .unwrap();
    let mut session = CompilerSession::new();
    assert!(matches!(
        scheduler.resolve(&mut session),
        Err(SchedulerError::Limit("passes"))
    ));
    assert_eq!(session.workspaces().len(), 3);
}

#[test]
fn source_run_fatal_report_rolls_back_earlier_generated_input() {
    let fixture = Fixture::new(
        "add_build_string :: (data: string, w: s64) #compiler; compiler_report :: (message: string) #compiler; recipe :: () { add_build_string(\"answer :: 42;\", -1); compiler_report(\"recipe failed\"); } #run recipe(); main :: () {}",
    );
    let mut session = CompilerSession::new();
    let error = fixture
        .scheduler()
        .resolve(&mut session)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("recipe failed"), "{error}");
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
}

#[test]
fn source_output_and_runtime_policy_commit_and_drive_bootstrap_parameters() {
    let fixture = Fixture::new(
        "Output_Type :: enum u8 { NO_OUTPUT :: 0; EXECUTABLE :: 1; DYNAMIC_LIBRARY :: 2; STATIC_LIBRARY :: 3; OBJECT_FILE :: 4; } Runtime_Mode :: enum u8 { AUTO :: 0; ENTRY_POINT_AND_INIT :: 1; ONLY_INIT :: 2; OMIT :: 3; } Backtrace :: enum u8 { OFF :: 0; ON :: 1; } Build_Options :: struct { output_type: Output_Type = .EXECUTABLE; runtime_support_definitions: Runtime_Mode = .AUTO; backtrace_on_crash: Backtrace = .ON; } set_build_options :: (options: Build_Options, w: s64 = -1) #compiler; #run set_build_options(Build_Options.{output_type = .OBJECT_FILE, runtime_support_definitions = .ONLY_INIT, backtrace_on_crash = .OFF}); main :: () {}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    let settings = session.workspace(session.root()).unwrap().settings();
    assert_eq!(settings.output_kind, jai_types::BuildOutputKind::Object);
    assert_eq!(
        settings.runtime_support,
        jai_types::RuntimeSupportMode::InitializationOnly
    );
    assert_eq!(
        settings.backtrace_on_crash,
        jai_types::BacktraceOnCrash::Off
    );
    assert_eq!(
        settings.runtime_support_parameters(),
        jai_modules::RuntimeSupportParameters {
            define_system_entry_point: false,
            define_initialization: true,
            enable_backtrace_on_crash: false,
            temporary_storage_size: 32768,
        }
    );
}

#[test]
fn source_output_is_committed_once_across_generated_graph_rebuilds() {
    let fixture = Fixture::new(
        r#"write_string :: (s: string, to_standard_error := false) #no_context #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () { write_string("hello\0world"); write_string("error", true); add_build_string("answer :: 42;", -1); }
#run recipe();
main :: () -> int { return answer; }"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    assert_eq!(
        session.take_outputs(),
        vec![
            crate::compiler_effects::CompilerOutput {
                stream: jai_vm::CompilerOutputStream::StandardOutput,
                bytes: b"hello\0world".to_vec()
            },
            crate::compiler_effects::CompilerOutput {
                stream: jai_vm::CompilerOutputStream::StandardError,
                bytes: b"error".to_vec()
            },
        ]
    );
    scheduler.resolve(&mut session).unwrap();
    assert!(session.take_outputs().is_empty());
}

#[test]
fn source_output_rolls_back_when_later_compiler_report_aborts() {
    let fixture = Fixture::new(
        r#"write_string :: (s: string, to_standard_error := false) #no_context #compiler;
compiler_report :: (message: string) #compiler;
recipe :: () { write_string("unpublished"); compiler_report("stop output"); }
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("source recipe must fail");
    assert!(error.to_string().contains("stop output"), "{error}");
    assert!(session.take_outputs().is_empty());
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn checked_variadic_string_output_uses_actual_slice_storage() {
    let fixture = Fixture::new(
        r#"write_strings :: (strings: ..string, to_standard_error := false) #no_context #compiler;
recipe :: () { write_strings("one", "two", to_standard_error = true); }
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    scheduler.resolve(&mut session).unwrap();
    assert_eq!(
        session.take_outputs(),
        vec![crate::compiler_effects::CompilerOutput {
            stream: jai_vm::CompilerOutputStream::StandardError,
            bytes: b"onetwo".to_vec(),
        }]
    );
}

#[test]
fn compiler_report_omitted_location_preserves_real_call_site() {
    let fixture = Fixture::new(
        r#"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }
Report :: enum u8 { ERROR; ERROR_CONTINUABLE; WARNING; INFO; }
compiler_report :: (message: string, loc: Source_Code_Location = #caller_location, mode: Report = .WARNING) #compiler;
#run compiler_report("located warning");
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    scheduler.resolve(&mut session).unwrap();
    let messages = session.take_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].level, jai_vm::MessageLevel::Warning);
    assert_eq!(messages[0].text, "located warning");
    let location = messages[0].location.as_ref().unwrap();
    assert_eq!(
        location.path,
        fixture.0.join("main.jai").canonicalize().unwrap()
    );
    assert_eq!(location.line, 4);
    assert_eq!(location.column, 6);
}

#[test]
fn compiler_code_null_suffix_adds_root_source_and_preserves_location() {
    let fixture = Fixture::new(
        r#"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }
add_build_string :: (data: string, w: s64, code := #code,null, loc := #caller_location) #compiler;
#run add_build_string("answer :: 42;", -1);
main :: () -> int { return answer; }"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    let inputs = session.workspace(session.root()).unwrap().inputs();
    assert_eq!(inputs.len(), 1);
    match &inputs[0] {
        crate::BuildInput::SourceAt {
            source,
            location,
        } => {
            assert_eq!(source, "answer :: 42;");
            assert_eq!(
                location.path,
                fixture.0.join("main.jai").canonicalize().unwrap()
            );
            assert_eq!(location.line, 3);
            assert_eq!(location.column, 6);
        }
        other => panic!("expected located generated root source: {other:?}"),
    }
    let (_, library) = checked(&build, session.root());
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    assert!(
        matches!(vm.execute(library.procedures()[0].id, vec![]).outcome,
        jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn captured_compiler_code_scope_rejects_without_staging_root_source() {
    let fixture = Fixture::new(
        r#"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }
add_build_string :: (data: string, w: s64, code := #code,null, loc := #caller_location) #compiler;
#run add_build_string("wrong_scope :: 42;", -1, #code { value := 1; });
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("source recipe must fail");
    assert!(
        error
            .to_string()
            .contains("captured Code scope is not implemented"),
        "{error}"
    );
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn source_set_build_options_keeps_implicit_caller_location_after_commit() {
    let fixture = Fixture::new(
        r#"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }
Output_Type :: enum u8 { NO_OUTPUT; EXECUTABLE; DYNAMIC_LIBRARY; STATIC_LIBRARY; OBJECT_FILE; }
Build_Options :: struct { output_type: Output_Type = .EXECUTABLE; }
set_build_options :: (options: Build_Options, w: s64 = -1, loc := #caller_location) #compiler;
#run set_build_options(Build_Options.{output_type = .OBJECT_FILE});
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    let workspace = session.workspace(session.root()).unwrap();
    assert_eq!(
        workspace.settings().output_kind,
        jai_types::BuildOutputKind::Object
    );
    let location = workspace
        .option_origin(crate::BuildSetting::OutputKind)
        .unwrap();
    assert_eq!(
        location.path,
        fixture.0.join("main.jai").canonicalize().unwrap()
    );
    assert_eq!(location.line, 5);
    assert_eq!(location.column, 6);
}

#[test]
fn source_workspace_status_recovers_its_continuable_error() {
    let fixture = Fixture::new(
        r#"Source_Code_Location :: struct { fully_pathed_filename: string; line_number: s64; character_number: s64; }
Report :: enum u8 { ERROR; ERROR_CONTINUABLE; WARNING; INFO; }
Workspace_Status :: enum u8 { OK; FAILED; }
compiler_report :: (message: string, loc := #caller_location, mode: Report = .ERROR_CONTINUABLE) #compiler;
compiler_set_workspace_status :: (status: Workspace_Status, w: s64 = -1) #compiler;
recipe :: () { compiler_report("recovered error"); compiler_set_workspace_status(.OK); }
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    scheduler.resolve(&mut session).unwrap();
    assert!(session.error().is_none());
    let messages = session.take_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].level, jai_vm::MessageLevel::Error);
    assert_eq!(messages[0].text, "recovered error");
}

#[test]
fn source_workspace_status_can_deliberately_fail_the_build() {
    let fixture = Fixture::new(
        r#"Workspace_Status :: enum u8 { OK; FAILED; }
compiler_set_workspace_status :: (status: Workspace_Status, w: s64 = -1) #compiler;
#run compiler_set_workspace_status(.FAILED);
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("failed workspace must fail build");
    assert!(error.to_string().contains("workspace"), "{error}");
    assert!(session.error().is_some());
}

#[test]
fn failed_child_can_recover_in_its_recipe_before_final_build_status() {
    let fixture = Fixture::new(
        r#"Workspace_Status :: enum u8 { OK; FAILED; }
compiler_create_workspace :: (name: string) -> s64 #compiler;
compiler_set_workspace_status :: (status: Workspace_Status, w: s64 = -1) #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("recovering child");
    add_build_string("Workspace_Status :: enum u8 { OK; FAILED; } compiler_set_workspace_status :: (status: Workspace_Status, w: s64 = -1) #compiler; #run compiler_set_workspace_status(.OK); child :: () -> int { return 42; }", child);
    compiler_set_workspace_status(.FAILED, child);
}
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    assert_eq!(build.workspaces.len(), 2);
    assert!(session.error().is_none());
    assert!(
        session
            .workspaces()
            .all(|workspace| workspace.status() == jai_vm::WorkspaceStatus::Ok)
    );
}

#[test]
fn compiler_debug_break_is_a_real_trap_and_rolls_back_output() {
    let fixture = Fixture::new(
        r#"write_string :: (s: string, to_standard_error := false) #no_context #compiler;
compile_time_debug_break :: () #no_context #compiler;
recipe :: () { write_string("unpublished"); compile_time_debug_break(); }
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("debug break must trap");
    assert!(error.to_string().contains("trap"), "{error}");
    assert!(session.take_outputs().is_empty());
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn source_create_and_destroy_skips_unparseable_child_inputs() {
    let fixture = Fixture::new(
        r#"compiler_create_workspace :: (name: string) -> s64 #compiler;
compiler_destroy_workspace :: (w: s64) #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("retired child");
    add_build_string("this is intentionally invalid source", child);
    compiler_destroy_workspace(child);
}
#run recipe();
main :: () -> int { return 42; }"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 1);
    assert_eq!(build.workspaces.len(), 1);
    assert_eq!(session.workspaces().len(), 1);
}

#[test]
fn root_retirement_keeps_generated_child_checked_and_drops_root_diagnostics() {
    let fixture = Fixture::new(
        r#"compiler_create_workspace :: (name: string) -> s64 #compiler;
compiler_destroy_workspace :: (w: s64) #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("surviving child");
    compiler_destroy_workspace(-1);
    add_build_string("main :: () -> int { return 42; }", child);
}
#run recipe();
main :: () -> int { return retired_root_dependency; }"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let root = session.root();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 2);
    assert_eq!(build.workspaces.len(), 1);
    assert!(build.workspace(root).is_none());
    assert!(session.is_destroyed(root));
    let child = build.workspaces[0].id;
    let (_, library) = checked(&build, child);
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    let outcome = vm.execute(library.procedures()[0].id, vec![]).outcome;
    assert!(
        matches!(outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn retiring_all_workspaces_returns_an_empty_successful_plan() {
    let fixture = Fixture::new(
        "compiler_destroy_workspace :: (w: s64) #compiler; #run compiler_destroy_workspace(-1); main :: () {}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert!(build.workspaces.is_empty());
    assert!(session.is_destroyed(session.root()));
    assert_eq!(session.workspaces().len(), 0);
    assert!(
        scheduler
            .resolve(&mut session)
            .unwrap()
            .workspaces
            .is_empty()
    );
}

#[test]
fn retirement_cancels_an_existing_failed_child_before_its_recipe_runs() {
    let mut session = CompilerSession::new();
    let CompilerResponse::Workspace(child) = stage(
        &mut session,
        CompilerRequest::CreateWorkspace {
            name: "failed child".into(),
        },
    ) else {
        panic!("expected child workspace")
    };
    stage(&mut session, CompilerRequest::AddSource { workspace: child, source: "compiler_report :: (message: string) #compiler; #run compiler_report(\"must not execute\");".into() });
    stage(
        &mut session,
        CompilerRequest::SetWorkspaceStatus {
            workspace: child,
            status: jai_vm::WorkspaceStatus::Failed,
        },
    );
    let fixture = Fixture::new(&format!(
        "compiler_destroy_workspace :: (w: s64) #compiler; #run compiler_destroy_workspace({}); main :: () {{}}",
        child.get()
    ));
    let mut scheduler = fixture.scheduler();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.workspaces.len(), 1);
    assert!(session.is_destroyed(child));
    assert!(session.error().is_none());
    assert!(session.take_messages().is_empty());
}

#[test]
fn child_self_retirement_leaves_no_stale_checked_plan() {
    let fixture = Fixture::new(
        r#"compiler_create_workspace :: (name: string) -> s64 #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("self retiring");
    add_build_string("compiler_destroy_workspace :: (w: s64) #compiler; #run compiler_destroy_workspace(-1); main :: () -> int { return retired_dependency; }", child);
}
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(build.passes, 3);
    assert_eq!(build.workspaces.len(), 1);
    assert_eq!(build.workspaces[0].id, session.root());
    assert_eq!(session.workspaces().len(), 1);
}

#[test]
fn source_stale_handle_request_rolls_back_the_whole_retirement() {
    let fixture = Fixture::new(
        r#"compiler_create_workspace :: (name: string) -> s64 #compiler;
compiler_destroy_workspace :: (w: s64) #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("retired");
    compiler_destroy_workspace(child);
    add_build_string("main :: () {}", child);
}
#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("stale handle must fail");
    assert!(error.to_string().contains("destroyed"), "{error}");
    assert_eq!(session.workspaces().len(), 1);
    assert!(scheduler.replay_cache().is_empty());
}

#[test]
fn source_workspace_name_reads_staged_and_current_child_names() {
    let fixture = Fixture::new(
        r#"compiler_create_workspace :: (name: string) -> s64 #compiler;
get_name :: (w: s64 = -1) -> string #compiler;
write_string :: (s: string, to_standard_error := false) #no_context #compiler;
add_build_string :: (data: string, w: s64) #compiler;
recipe :: () {
    child := compiler_create_workspace("child π");
    write_string(get_name(child));
    add_build_string("get_name :: (w: s64 = -1) -> string #compiler; name :: #run get_name(); child :: () -> bool { return name == \"child π\"; }", child);
}

#run recipe();
main :: () {}"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    let child = build
        .workspaces
        .iter()
        .find(|workspace| workspace.id != session.root())
        .unwrap()
        .id;
    let (_, library) = checked(&build, child);
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(library.procedures()[0].id, vec![]).outcome,
        jai_vm::Outcome::Complete(vec![jai_vm::Value::Bool(true)])
    );
    assert_eq!(session.take_outputs()[0].bytes, "child π".as_bytes());
}

#[test]
fn source_version_info_writes_checked_fields_and_accepts_null() {
    let fixture = Fixture::new(
        r#"Version_Info :: struct { major: s32; minor: s32; micro: s32; }
compiler_get_version_info :: (version_info_return: *Version_Info) -> string #compiler;
inspect :: () -> bool {
    info: Version_Info;
    text := compiler_get_version_info(*info);
    without_pointer := compiler_get_version_info(null);
    return text == without_pointer && info.major == 0 && info.minor == 1 && info.micro == 0;
}
valid :: #run inspect();
main :: () -> bool { return valid; }"#,
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    let (_, library) = checked(&build, session.root());
    let main = library.procedures().last().unwrap().id;
    let mut vm = jai_vm::Vm::new(library, jai_vm::NoEffects, Limits::default()).unwrap();
    assert_eq!(
        vm.execute(main, vec![]).outcome,
        jai_vm::Outcome::Complete(vec![jai_vm::Value::Bool(true)])
    );
}

#[test]
fn source_version_info_rejects_incompatible_nominal_fields() {
    for fields in [
        "minor: s32; major: s32; micro: s32;",
        "major: s64; minor: s32; micro: s32;",
    ] {
        let fixture = Fixture::new(&format!(
            "Version_Info :: struct {{ {fields} }}\ncompiler_get_version_info :: (p: *Version_Info) -> string #compiler;\nmain :: () {{}}"
        ));
        let mut scheduler = fixture.scheduler();
        let mut session = CompilerSession::new();
        let error = scheduler
            .resolve(&mut session)
            .err()
            .expect("invalid source schema must reject");
        assert!(
            error
                .to_string()
                .contains("Version_Info requires exactly s32"),
            "{error}"
        );
        assert!(session.take_outputs().is_empty());
    }
}
