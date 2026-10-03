use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions};
use std::path::Path;

#[derive(Default)]
struct Effects {
    ready: bool,
    requests: usize,
    writes: Vec<Vec<u8>>,
    staged: Vec<Vec<u8>>,
    finishes: Vec<bool>,
    origins: Vec<jai_vm::SourceOrigin>,
}
impl jai_vm::CompilerEffects for Effects {
    fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
        self.origins.push(origin);
    }
    fn begin(&mut self) {
        assert!(self.staged.is_empty());
    }
    fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        match request {
            jai_vm::CompilerRequest::CreateWorkspace {
                ..
            } => {
                self.requests += 1;
                jai_vm::EffectOutcome::Pending(jai_vm::EffectKey(42))
            }
            jai_vm::CompilerRequest::WriteOutput {
                bytes, ..
            } => {
                self.staged.push(bytes);
                jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Unit)
            }
            other => panic!("{other:?}"),
        }
    }
    fn poll_request(
        &mut self,
        request: &jai_vm::CompilerRequest,
        key: jai_vm::EffectKey,
    ) -> jai_vm::EffectOutcome {
        assert!(matches!(
            request,
            jai_vm::CompilerRequest::CreateWorkspace { .. }
        ));
        assert_eq!(key, jai_vm::EffectKey(42));
        if self.ready {
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        } else {
            jai_vm::EffectOutcome::Pending(key)
        }
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.finishes.push(commit);
        if commit {
            self.writes.append(&mut self.staged);
        } else {
            self.staged.clear();
        }
        Ok(())
    }
}

fn options(graph: &ModuleGraph) -> ResolveOptions {
    ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    }
}

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-headers/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap()
}

fn execute(graph: &ModuleGraph, library: jai_ir::Library) -> i128 {
    let declaration = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "main")
        .unwrap();
    let entry = library.procedure(declaration.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    match jai_vm::execute(&program, Default::default()).outcome {
        jai_vm::Outcome::Complete(values) => values[0].integer().unwrap().value(),
        outcome => panic!("{outcome:?}"),
    }
}

#[test]
fn field_runs_complete_before_the_original_global_initializer() {
    let graph = graph(
        r#"
        Record::struct { seed::()->int { return 42; } value:int=#run seed(); }
        global:Record;
        main::()->int { return global.value; }
    "#,
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => assert_eq!(execute(&graph, *library), 42),
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    }
}

#[test]
fn field_runs_complete_before_ordinary_parameter_defaults() {
    let graph = graph(
        r#"
        Record::struct { seed::()->int { return 42; } value:int=#run seed(); }
        read::(value:Record=Record.{}) -> int { return value.value; }
        main::()->int { return read(); }
    "#,
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => assert_eq!(execute(&graph, *library), 42),
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    }
}

#[test]
fn synchronous_resolution_uses_the_same_header_readiness_stage() {
    let graph = graph(
        r#"
        Record::struct { seed::()->int { return 42; } value:int=#run seed(); }
        global:Record;
        read::(value:Record=Record.{}) -> int { return value.value; }
        main::()->int { return global.value+read()-42; }
    "#,
    );
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("VM failed")
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn header_prerequisites_use_the_actual_typed_sequence_constant() {
    let graph = graph(
        r#"
        values:[2]int:.[20,22];
        Record::struct { seed::()->int {return values[0]+values[1];} value:int=#run seed(); }
        global:Record;
        main::()->int {return global.value;}
    "#,
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    match session.drive(&mut jai_vm::NoEffects) {
        LibraryReadiness::Complete(library) => assert_eq!(execute(&graph, *library), 42),
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    }
}

#[test]
fn header_prerequisites_do_not_execute_unrelated_body_or_root_recipes() {
    let graph = graph(
        r#"
        write_string::(s:string,to_standard_error:bool) #no_context #compiler;
        Record::struct {
            seed::()->int { write_string("field",false); return 42; }
            value:int=#run seed();
        }
        global:Record;
        unrelated::(){ #run write_string("body",false); }
        #run write_string("root",false);
        main::()->int { return global.value; }
    "#,
    );
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    match session.drive(&mut effects) {
        LibraryReadiness::Complete(library) => assert_eq!(execute(&graph, *library), 42),
        LibraryReadiness::Pending(pending) => panic!("{pending:?}"),
        LibraryReadiness::Failed(error) => panic!("{error:?}"),
    }
    assert_eq!(effects.writes.len(), 3);
    assert_eq!(effects.writes[0], b"field");
    assert!(effects.writes[1..].contains(&b"body".to_vec()));
    assert!(effects.writes[1..].contains(&b"root".to_vec()));
}

fn pending_field_graph() -> ModuleGraph {
    graph(
        r#"
        compiler_create_workspace::(name:string)->s64 #compiler;
        write_string::(s:string,to_standard_error:bool) #no_context #compiler;
        Record::struct {
            seed::()->int {
                write_string("field before",false);
                receipt:=compiler_create_workspace("field child");
                write_string("field after",false);
                return receipt;
            }
            value:int=#run,stallable seed();
        }
        global:Record;
        read::(value:Record=Record.{}) -> int {return value.value;}
        main::()->int {return global.value+read()-42;}
    "#,
    )
}

#[test]
fn pending_field_default_resumes_before_publishing_globals_and_parameter_defaults() {
    let graph = pending_field_graph();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let mut effects = Effects::default();
    let LibraryReadiness::Pending(pending) = session.drive(&mut effects) else {
        panic!("real field effect must park before global/header recipes")
    };
    assert!(
        pending
            .dependencies
            .contains(&jai_vm::Dependency::Effect(jai_vm::EffectKey(42)))
    );
    assert_eq!(effects.requests, 1);
    assert!(effects.writes.is_empty());
    effects.ready = true;
    let LibraryReadiness::Complete(library) = session.drive(&mut effects) else {
        panic!("same field recipe must resume")
    };
    assert_eq!(execute(&graph, *library), 42);
    assert_eq!(effects.requests, 1);
    assert_eq!(
        effects.writes,
        [b"field before".to_vec(), b"field after".to_vec()]
    );
}

#[test]
fn cancellation_discards_the_pending_field_checkpoint_before_any_global_publication() {
    let graph = pending_field_graph();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Pending(_)
    ));
    let origin = effects.origins.last().unwrap().clone();
    session.cancel(&mut effects).unwrap();
    assert_eq!(effects.requests, 1);
    assert!(effects.writes.is_empty());
    assert!(effects.staged.is_empty());
    assert_eq!(effects.origins.last(), Some(&origin));
    assert_eq!(effects.finishes.last(), Some(&false));
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Failed(_)
    ));
}
