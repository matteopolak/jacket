//! Generated declaration fulfillment uses the real workspace source scheduler.
use jai_driver::{
    CompilerSession, ReplayLimits, ScheduledBuild, SchedulerLimits, SchedulerOptions,
    WorkspaceOutput, WorkspaceScheduler,
};
use jai_modules::GraphOptions;
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{Limits, Outcome, TargetTriple, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "jai-source-placeholders-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }

    fn scheduler(&self) -> WorkspaceScheduler {
        WorkspaceScheduler::new(
            &self.0.join("main.jai"),
            GraphOptions::default(),
            SchedulerOptions {
                target: BuildTarget {
                    operating_system: OperatingSystem::Linux,
                    architecture: Architecture::X86_64,
                    layout: LayoutPolicy::lp64(),
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
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn execute_main(build: &ScheduledBuild, session: &CompilerSession) -> i128 {
    let WorkspaceOutput::Checked {
        unit,
        library,
        ..
    } = &build.workspace(session.root()).unwrap().output
    else {
        panic!("generated source did not produce a checked workspace");
    };
    let declaration = unit
        .graph()
        .declarations()
        .iter()
        .find(|declaration| unit.graph().symbols().name(declaration.name()) == "main")
        .unwrap();
    let main = library.procedure(declaration.id()).unwrap().id;
    let mut vm = jai_vm::Vm::new(library.as_ref(), jai_vm::NoEffects, Limits::default()).unwrap();
    let execution = vm.execute(main, vec![]);
    let Outcome::Complete(values) = execution.outcome else {
        panic!("generated placeholder fixture did not complete");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn file_run_fills_a_constant_once_and_rebuild_preserves_the_real_binding() {
    let fixture = Fixture::new(
        "#placeholder ANSWER;add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"ANSWER::42;\",-1);main::()->int{return ANSWER;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(build.passes, 2);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
    assert_eq!(scheduler.replay_cache().len(), 1);
    let again = scheduler.resolve(&mut session).unwrap();
    assert_eq!(execute_main(&again, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn unavailable_global_type_waits_for_its_real_generated_record_definition() {
    let fixture = Fixture::new(
        "#placeholder Generated;value:Generated;add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"Generated::struct{answer:int=42;}\",-1);main::()->int{return value.answer;}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn unavailable_procedure_header_waits_without_inventing_parameter_types() {
    let fixture = Fixture::new(
        "#placeholder Generated;answer::(value:Generated=.{})->int{return value.answer;}add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"Generated::struct{answer:int=42;}\",-1);main::()->int{return answer();}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn a_pointer_alias_preserves_the_unavailable_nominal_until_real_fulfillment() {
    let fixture = Fixture::new(
        "#placeholder Generated;Alias::#type *Generated;add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"Generated::struct{answer:int=42;}\",-1);main::()->int{value:Generated;pointer:Alias=*value;return pointer.answer;}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn missing_fulfillment_returns_a_demand_error_at_a_stable_source_fixed_point() {
    let fixture = Fixture::new("#placeholder MISSING;main::()->int{return MISSING;}");
    let mut session = CompilerSession::new();
    let error = match fixture.scheduler().resolve(&mut session) {
        Ok(_) => panic!("unfilled reservation cannot provide a runtime result"),
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(
        message.contains("#placeholder") && message.contains("unfilled"),
        "{message}"
    );
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
}

#[test]
fn selected_ordinary_default_precedes_a_real_generated_record_round() {
    let fixture = Fixture::new(
        "#placeholder Generated; Alias::#type *Generated; \
         add_build_string::(data:string,w:s64)#compiler; \
         produce::(to_standard_error:=false) {if to_standard_error return;add_build_string(\"Generated::struct{answer:int=42;}\",-1);} \
         #run produce(); main::()->int{value:Generated;pointer:Alias=*value;return pointer.answer;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(build.passes, 2);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
    assert_eq!(scheduler.replay_cache().len(), 1);
    let again = scheduler.resolve(&mut session).unwrap();
    assert_eq!(execute_main(&again, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn failed_early_default_cannot_publish_its_generated_source_or_replay_receipt() {
    let fixture = Fixture::new(
        "#placeholder Generated; Alias::#type *Generated; \
         add_build_string::(data:string,w:s64)#compiler; \
         make_zero::()->int #no_context{return 0;} \
         choose::()->bool {add_build_string(\"Generated::struct{answer:int=42;}\",-1);zero:=make_zero();return 1/zero==0;} \
         produce::(flag:bool=#run choose()) {} #run produce(); main::()->int{return 42;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = scheduler
        .resolve(&mut session)
        .err()
        .expect("the genuine selected default must fail");
    let jai_driver::SchedulerError::Driver(jai_driver::Error::Located {
        source,
        path,
        diagnostic,
        ..
    }) = &error
    else {
        panic!("the original selected default arithmetic failure must remain located: {error:?}")
    };
    assert_eq!(
        diagnostic.message,
        jai_vm::Error::Arithmetic(jai_vm::ArithmeticError::ZeroDivisor).to_string(),
    );
    assert_eq!(diagnostic.source, Some(*source));
    assert_eq!(path, &fs::canonicalize(fixture.0.join("main.jai")).unwrap());
    let unit = jai_driver::CompilationUnit::load(&fixture.0.join("main.jai")).unwrap();
    let original = unit
        .graph()
        .declarations()
        .iter()
        .find(|source| unit.graph().symbols().name(source.name()) == "produce")
        .unwrap();
    let jai_syntax::FileDeclarationKind::Procedure(procedure) = &original.syntax().kind else {
        panic!("the default retains its original source procedure")
    };
    let default = match &procedure.parameters[0].binding {
        jai_syntax::ParameterBinding::Defaulted {
            expression, ..
        }
        | jai_syntax::ParameterBinding::DefaultedType {
            expression, ..
        } => expression,
        jai_syntax::ParameterBinding::Required(_)
        | jai_syntax::ParameterBinding::RequiredType(_) => panic!("original source default"),
    };
    assert_eq!(diagnostic.span, default.span);
    assert!(
        session
            .workspace(session.root())
            .unwrap()
            .inputs()
            .is_empty()
    );
    assert!(session.take_outputs().is_empty());
    assert!(scheduler.replay_cache().is_empty());
}
