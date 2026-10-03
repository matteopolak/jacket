//! Generated initializer fulfillment uses the real workspace source scheduler.
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
            "jai-generated-global-initializers-{}-{}",
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
        panic!("generated initializer fixture did not complete");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn original_run_supplies_the_called_global_initializer_source_once() {
    let fixture = Fixture::new(
        "value:int=#run generated();add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"generated::()->int{return 42;}\",-1);main::()->int{return value;}",
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
fn ordinary_global_call_keeps_the_original_initializer_job() {
    let fixture = Fixture::new(
        "value:int=generated();add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"generated::()->int{return 42;}\",-1);main::()->int{return value;}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn selected_initializer_body_waits_without_starting_a_blocking_vm_journal() {
    let fixture = Fixture::new(
        "read::()->int{return generated();}value:int=#run read();add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"generated::()->int{return 42;}\",-1);main::()->int{return value;}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 1);
}

#[test]
fn separate_initializers_keep_source_order_across_two_actual_publications() {
    let fixture = Fixture::new(
        "first:int=#run generated_first();second:int=#run generated_second();add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"generated_first::()->int{return 40;}\",-1);#run add_build_string(\"generated_second::()->int{return 2;}\",-1);main::()->int{return first+second;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let build = scheduler.resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
    assert_eq!(build.passes, 3);
    assert_eq!(session.workspace(session.root()).unwrap().inputs().len(), 2);
    assert_eq!(scheduler.replay_cache().len(), 2);
}

#[test]
fn a_source_producer_may_read_an_earlier_genuinely_published_global() {
    let fixture = Fixture::new(
        "ready:int=40;value:int=#run generated();add_build_string::(data:string,w:s64)#compiler;#run {if ready!=40 {zero:int=0;bad:=1/zero;}add_build_string(\"generated::()->int{return 42;}\",-1);}main::()->int{return value;}",
    );
    let mut session = CompilerSession::new();
    let build = fixture.scheduler().resolve(&mut session).unwrap();
    assert_eq!(execute_main(&build, &session), 42);
}

#[test]
fn an_exhausted_source_prefix_reports_the_actual_lookup_and_drops_private_inputs() {
    let fixture = Fixture::new(
        "value:int=#run missing();add_build_string::(data:string,w:s64)#compiler;#run add_build_string(\"unrelated::()->int{return 42;}\",-1);main::()->int{return value;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = match scheduler.resolve(&mut session) {
        Err(error) => error,
        Ok(_) => panic!("an unrelated generated declaration cannot satisfy the actual lookup"),
    };
    assert!(
        error.to_string().contains("unknown name 'missing'"),
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
fn a_failing_selected_source_producer_cannot_publish_its_initializer_or_effects() {
    let fixture = Fixture::new(
        "value:int=#run generated();add_build_string::(data:string,w:s64)#compiler;write_string::(data:string,to_standard_error:bool)#no_context #compiler;make_zero::()->int #no_context{return 0;}#run {write_string(\"private\",false);add_build_string(\"generated::()->int{return 42;}\",-1);zero:=make_zero();bad:=1/zero;}main::()->int{return value;}",
    );
    let mut scheduler = fixture.scheduler();
    let mut session = CompilerSession::new();
    let error = match scheduler.resolve(&mut session) {
        Err(error) => error,
        Ok(_) => panic!("the original source producer must fail"),
    };
    let jai_driver::SchedulerError::Driver(jai_driver::Error::Located {
        source,
        path,
        diagnostic,
        ..
    }) = &error
    else {
        panic!("the actual source VM arithmetic failure must remain located: {error:?}")
    };
    assert_eq!(
        diagnostic.message,
        jai_vm::Error::Arithmetic(jai_vm::ArithmeticError::ZeroDivisor).to_string(),
    );
    assert_eq!(diagnostic.source, Some(*source));
    assert_eq!(path, &fs::canonicalize(fixture.0.join("main.jai")).unwrap());
    let unit = jai_driver::CompilationUnit::load(&fixture.0.join("main.jai")).unwrap();
    assert_eq!(diagnostic.span, unit.graph().runs()[0].syntax.location.span);
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
