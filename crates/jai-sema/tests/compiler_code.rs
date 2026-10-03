//! These authored fixtures run through the actual source binder and retained VM.
use jai_modules::{GraphDiscovery, GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{
    DiscoveryReadiness, LibraryReadiness, PreparedDiscoveryOutcome, PreparedDiscoveryRequests,
    PreparedDiscoverySession, PreparedLibrarySession, ResolveOptions,
};
use jai_types::TypeView;
use jai_vm::{CompilerEffects, CompilerRequest, CompilerResponse, EffectKey, EffectOutcome};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/compiler-code-source/main.jai");
    let mut sources = SourceOverlay::new();
    sources.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &sources).unwrap()
}

fn program(graph: &ModuleGraph, effects: &mut impl CompilerEffects) -> jai_ir::Program {
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut session = PreparedLibrarySession::new(graph, &options).unwrap();
    let library = match session.drive(effects) {
        LibraryReadiness::Complete(library) => library,
        LibraryReadiness::Pending(pending) => {
            panic!("source compiler Code is pending: {pending:?}")
        }
        LibraryReadiness::Failed(error) => panic!("source compiler Code failed: {error:?}"),
    };
    let main = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap()
}

fn answer(program: &jai_ir::Program) -> i128 {
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(program, Default::default()).outcome
    else {
        panic!("inserted compiler quotation must execute")
    };
    values[0].integer().unwrap().value()
}

#[test]
fn named_code_producer_has_source_identity_and_no_native_signature() {
    let graph = graph(
        "produce::()->Code{n:=41;if n==41{return #code n+1;}else{return #code 0;}} main::()->int{return #insert #run produce();}",
    );
    let program = program(&graph, &mut jai_vm::NoEffects);
    assert_eq!(answer(&program), 42);
    assert_eq!(program.procedures().len(), 1);
    for ty in program
        .procedures()
        .iter()
        .map(|procedure| procedure.signature)
    {
        let jai_types::TypeKind::Procedure(signature) = program.types().kind(ty).unwrap() else {
            panic!("native procedure signature")
        };
        assert!(
            program
                .types()
                .procedure_type(*signature)
                .unwrap()
                .results
                .iter()
                .all(|&ty| !matches!(program.types().kind(ty), Ok(jai_types::TypeKind::Code)))
        );
    }
}

#[test]
fn imported_code_producer_keeps_its_original_file_and_source_scope() {
    let mut sources = SourceOverlay::new();
    let main = Path::new("/compiler-code-source/main.jai");
    sources.insert(main, b"Maker::#import,file \"maker.jai\";main::()->int{return #insert #run Maker.make_code();}".to_vec()).unwrap();
    sources
        .insert(
            Path::new("/compiler-code-source/maker.jai"),
            b"OFFSET::40;make_code::()->Code{return #code OFFSET+2;}".to_vec(),
        )
        .unwrap();
    let graph = ModuleGraph::load_with_provider(main, GraphOptions::default(), &sources).unwrap();
    assert_eq!(answer(&program(&graph, &mut jai_vm::NoEffects)), 42);
}

#[test]
fn anonymous_short_form_retains_actual_enclosing_runtime_place() {
    let graph = graph("main::()->int{x:=3;#insert ->Code{return #code x=(x*10)+3495;}return x;}");
    assert_eq!(answer(&program(&graph, &mut jai_vm::NoEffects)), 3525);
}

#[test]
fn compiler_slots_are_distinct_under_shadowing_and_lexical_retirement() {
    let graph = graph(
        "main::()->int{return #insert #run ->Code{n:=40;{n:=100;n=n+1;}n=n+2;return #code n;};}",
    );
    assert_eq!(answer(&program(&graph, &mut jai_vm::NoEffects)), 42);
}

#[derive(Default)]
struct Effects {
    ready: bool,
    begins: usize,
    requests: usize,
    polls: usize,
    commits: usize,
    aborts: usize,
}
impl CompilerEffects for Effects {
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        self.requests += 1;
        if self.ready {
            EffectOutcome::Ready(CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(41).unwrap(),
            ))
        } else {
            EffectOutcome::Pending(EffectKey(73))
        }
    }
    fn poll_request(&mut self, request: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        assert_eq!(key, EffectKey(73));
        self.polls += 1;
        if self.ready {
            EffectOutcome::Ready(CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(41).unwrap(),
            ))
        } else {
            EffectOutcome::Pending(key)
        }
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        if commit {
            self.commits += 1;
        } else {
            self.aborts += 1;
        }
        Ok(())
    }
}

#[test]
fn suspension_keeps_native_result_compiler_slots_and_the_single_transaction() {
    let graph = graph(
        "compiler_create_workspace::(name:string)->s64 #compiler; counter:int=0;bump::()->int{counter+=1;return counter;} produce::()->Code{n:=bump();workspace:=compiler_create_workspace(\"once\");return #code n+cast(int)workspace;}main::()->int{return #insert #run,stallable produce();}",
    );
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut session = PreparedLibrarySession::new(&graph, &options).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Pending(_)
    ));
    assert_eq!(
        (effects.begins, effects.requests, effects.commits),
        (1, 1, 0)
    );
    effects.ready = true;
    let LibraryReadiness::Complete(library) = session.drive(&mut effects) else {
        panic!("same source continuation must resume")
    };
    let main = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "main")
        .unwrap();
    let entry = library.procedure(main.id()).unwrap().id;
    let program = library
        .into_program(jai_ir::EntryPoint::Int(entry))
        .unwrap();
    assert_eq!(
        answer(&program),
        42,
        "native bump must execute once across suspension"
    );
    assert_eq!(
        (
            effects.begins,
            effects.requests,
            effects.commits,
            effects.aborts
        ),
        (1, 1, 1, 0)
    );
}

#[test]
fn runtime_result_destination_is_rejected_before_any_compiler_effect() {
    let graph = graph(
        "compiler_create_workspace::(name:string)->s64 #compiler; answer:int:#run ->Code{workspace:=compiler_create_workspace(\"never\");return #code 42;}; main::()->int{return answer;}",
    );
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut session = PreparedLibrarySession::new(&graph, &options).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Failed(_)
    ));
    assert_eq!(
        (effects.begins, effects.requests, effects.commits),
        (0, 0, 0)
    );
}

#[test]
fn canceling_the_source_session_aborts_the_same_compiler_transaction() {
    let graph = graph(
        "compiler_create_workspace::(name:string)->s64 #compiler;produce::()->Code{workspace:=compiler_create_workspace(\"cancel\");return #code cast(int)workspace;}main::()->int{return #insert #run,stallable produce();}",
    );
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut session = PreparedLibrarySession::new(&graph, &options).unwrap();
    let mut effects = Effects::default();
    assert!(matches!(
        session.drive(&mut effects),
        LibraryReadiness::Pending(_)
    ));
    session.cancel(&mut effects).unwrap();
    assert_eq!(
        (
            effects.begins,
            effects.requests,
            effects.commits,
            effects.aborts
        ),
        (1, 1, 0, 1)
    );
}

#[test]
fn overlapping_source_copies_share_one_session_retention_allowance() {
    let payload = "a".repeat(270_000);
    let graph = graph(&format!(
        "compiler_create_workspace::(name:string)->s64 #compiler;produce::()->Code{{workspace:=compiler_create_workspace(\"retention\");return #code \"{payload}\";}}main::()->int{{return #insert #run,stallable produce();}}"
    ));
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut session = PreparedLibrarySession::new(&graph, &options).unwrap();
    let mut effects = Effects {
        ready: true,
        ..Default::default()
    };
    let LibraryReadiness::Failed(error) = session.drive(&mut effects) else {
        panic!("four individually small retained copies must exceed the shared source allowance");
    };
    assert!(error.message.contains("payload byte"), "{error}");
    assert_eq!(
        (
            effects.begins,
            effects.requests,
            effects.commits,
            effects.aborts
        ),
        (1, 1, 0, 1)
    );
}

#[test]
fn declaration_admission_rejects_duplicate_before_compiler_commit() {
    let path = Path::new("/compiler-code-source/main.jai");
    let mut sources = SourceOverlay::new();
    sources.insert(path, b"compiler_create_workspace::(name:string)->s64 #compiler;produce::()->Code{workspace:=compiler_create_workspace(\"aborted\");return #code ANSWER::42;}ANSWER::1;#insert #run,stallable produce();main::()->int{return ANSWER;}".to_vec()).unwrap();
    let mut graph = GraphDiscovery::new(path, GraphOptions::default(), &sources).unwrap();
    assert!(!graph.advance().unwrap().is_complete());
    let requests: Vec<_> = graph.pending_insertion_requests().cloned().collect();
    assert_eq!(requests.len(), 1, "original insertion frontier: {graph:?}");
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            graph.graph(),
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    };
    let mut effects = Effects {
        ready: true,
        ..Default::default()
    };
    {
        let mut session = PreparedDiscoverySession::with_insertion_admission(
            graph.graph(),
            &options,
            PreparedDiscoveryRequests::Insertions(requests),
            Box::new(|request, code| graph.admit_insertion(request, code)),
        )
        .unwrap();
        match session.drive(&mut effects) {
            DiscoveryReadiness::Failed(error) => {
                assert!(error.message.contains("ANSWER"), "{error}")
            }
            DiscoveryReadiness::Pending(pending) => {
                panic!("duplicate declaration pending: {pending:?}")
            }
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Insertions(outcome)) => {
                panic!("duplicate declaration was admitted: {outcome:?}")
            }
            DiscoveryReadiness::Complete(_) => panic!("wrong discovery outcome"),
        }
    }
    assert_eq!(
        (
            effects.begins,
            effects.requests,
            effects.commits,
            effects.aborts
        ),
        (1, 1, 0, 1)
    );
    assert!(graph.graph().insertion_publications().is_empty());
}

#[test]
fn declaration_publication_uses_the_sealed_original_graph_admission() {
    let path = Path::new("/compiler-code-source/main.jai");
    let mut sources = SourceOverlay::new();
    sources.insert(path, b"produce::()->Code{n:=41;return #code ANSWER::n+1;}#insert #run produce();main::()->int{return ANSWER;}".to_vec()).unwrap();
    let mut graph = GraphDiscovery::new(path, GraphOptions::default(), &sources).unwrap();
    assert!(!graph.advance().unwrap().is_complete());
    let requests = graph.pending_insertion_requests().cloned().collect();
    let outcome = {
        let mut session = PreparedDiscoverySession::with_insertion_admission(
            graph.graph(),
            &Default::default(),
            PreparedDiscoveryRequests::Insertions(requests),
            Box::new(|request, code| graph.admit_insertion(request, code)),
        )
        .unwrap();
        match session.drive(&mut jai_vm::NoEffects) {
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Insertions(outcome)) => outcome,
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("closed compiler Code insertion must complete"),
        }
    };
    assert_eq!(outcome.decisions.len(), 1);
    let decision = outcome.decisions.into_iter().next().unwrap();
    let transaction = graph
        .prepare_insertion_admitted(decision.request, decision.admission)
        .unwrap();
    graph.commit_insertion(transaction).unwrap();
    assert!(graph.advance().unwrap().is_complete());
    let graph = graph.into_graph().unwrap();
    assert_eq!(answer(&program(&graph, &mut jai_vm::NoEffects)), 42);
}
