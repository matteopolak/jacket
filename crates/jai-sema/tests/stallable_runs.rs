use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use jai_vm::{CompilerEffects, CompilerRequest, Dependency, EffectKey, EffectOutcome};
use std::path::Path;

#[derive(Default)]
struct Effects {
    ready: bool,
    service: bool,
    begins: usize,
    requests: usize,
    suspends: usize,
    resumes: usize,
    finishes: Vec<bool>,
    origins: Vec<jai_vm::SourceOrigin>,
}
impl CompilerEffects for Effects {
    fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
        self.origins.push(origin);
    }
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        self.requests += 1;
        EffectOutcome::Pending(EffectKey(42))
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        assert_eq!(key, EffectKey(42));
        if self.ready {
            EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        } else {
            EffectOutcome::Pending(key)
        }
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        self.suspends += 1;
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        self.resumes += 1;
        Ok(())
    }
    fn service_pending(&mut self, dependencies: &[Dependency]) -> Result<bool, jai_vm::Error> {
        if self.service && dependencies.contains(&Dependency::Effect(EffectKey(42))) {
            self.ready = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.finishes.push(commit);
        Ok(())
    }
}

fn graph(expression: &str) -> ModuleGraph {
    let source = format!(
        "compiler_create_workspace::(name:string)->s64 #compiler; calls:int; recipe::()->int{{calls+=1;receipt:=compiler_create_workspace(\"child\");return calls+receipt;}} answer::{expression};main::()->int{{return answer;}}"
    );
    let path = Path::new("/own-stallable-runs/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.into_bytes()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}
fn options(graph: &ModuleGraph) -> ResolveOptions {
    ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..ResolveOptions::default()
    }
}
fn execute(graph: &ModuleGraph, library: jai_ir::Library) -> i128 {
    let main = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "main")
        .unwrap();
    let id = library.procedure(main.id()).unwrap().id;
    let program = library.into_program(jai_ir::EntryPoint::Int(id)).unwrap();
    match jai_vm::execute(&program, jai_vm::Limits::default()).outcome {
        jai_vm::Outcome::Complete(values) => match &values[..] {
            [jai_vm::Value::Int(value)] => value.value(),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
}

#[test]
fn pending_session_resumes_exact_call_and_anonymous_frames() {
    for expression in [
        "#run,stallable recipe()",
        "#run,stallable ->int {calls+=1;receipt:=compiler_create_workspace(\"child\");return calls+receipt;}",
    ] {
        let graph = graph(expression);
        let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
        let mut effects = Effects::default();
        match session.drive(&mut effects) {
            LibraryReadiness::Pending(pending) => assert!(
                pending
                    .dependencies
                    .contains(&Dependency::Effect(EffectKey(42))),
                "{pending:?}"
            ),
            LibraryReadiness::Failed(error) => panic!("{error:?}"),
            LibraryReadiness::Complete(_) => panic!("pending effect cannot complete"),
        }
        assert_eq!(effects.requests, 1);
        assert!(effects.finishes.is_empty(), "{:?}", effects.finishes);
        effects.ready = true;
        let library = match session.drive(&mut effects) {
            LibraryReadiness::Complete(library) => library,
            LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
            LibraryReadiness::Failed(error) => panic!("{error:?}"),
        };
        assert_eq!(execute(&graph, *library), 43);
        assert_eq!(effects.requests, 1);
        assert_eq!(effects.begins, 1);
        assert_eq!(effects.finishes, [true]);
        assert!(effects.resumes > 0);
    }
}

#[test]
fn synchronous_service_resumes_before_publication() {
    let graph = graph("#run,stallable recipe()");
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let mut effects = Effects {
        service: true,
        ..Effects::default()
    };
    let library = match session.drive(&mut effects) {
        LibraryReadiness::Complete(library) => library,
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    };
    assert_eq!(execute(&graph, *library), 43);
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.begins, 1);
    assert_eq!(effects.finishes, [true]);
}

#[test]
fn cancellation_retires_the_parked_checkpoint_without_reexecution() {
    let graph = graph("#run,stallable recipe()");
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Pending(_)
    ));
    let origin = effects.origins.last().unwrap().clone();
    session.cancel(&mut effects).unwrap();
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes, [false]);
    assert_eq!(effects.origins.last(), Some(&origin));
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Failed(_)
    ));
}

#[test]
fn unflagged_external_wait_rolls_back_and_reports_its_policy() {
    let graph = graph("#run recipe()");
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let mut effects = Effects::default();
    match session.drive(&mut effects) {
        LibraryReadiness::Failed(error) => {
            assert!(error.message.contains("#run,stallable"), "{error:?}")
        }
        _ => panic!("an unflagged external wait cannot remain parked"),
    }
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes.last(), Some(&false));
    assert_eq!(effects.suspends, 0);
}
