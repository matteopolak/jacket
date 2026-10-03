//! Original initializer lookups retain genuine source prerequisites and rollback.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions, SourcePrefixReadiness};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-initializer-lookup/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
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
#[derive(Default)]
struct Effects {
    ready: bool,
    requests: usize,
    staged: Vec<Vec<u8>>,
    writes: Vec<Vec<u8>>,
    finishes: Vec<bool>,
}
impl jai_vm::CompilerEffects for Effects {
    fn begin(&mut self) {
        assert!(
            self.staged.is_empty(),
            "a fresh transaction cannot own prior staged output"
        );
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
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.finishes.push(commit);
        if commit {
            self.writes.append(&mut self.staged);
        } else {
            self.staged.clear();
        }
        Ok(())
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
}

fn waiting_graph() -> ModuleGraph {
    graph(
        "value:int=#run generated(); compiler_create_workspace::(name:string)->s64 #compiler; #run,stallable compiler_create_workspace(\"child\"); main::()->int{return value;}",
    )
}
fn assert_lookup_wait(graph: &ModuleGraph, wait: &jai_sema::LibraryPending) {
    let value = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "value")
        .unwrap();
    let demand = wait
        .source
        .expect("original initializer lookup remains beside its real producer wait");
    assert_eq!(demand.lookup_consumer(), Some(value.id()));
    assert_eq!(
        graph.symbols().name(demand.lookup_root().unwrap()),
        "generated"
    );
    assert!(demand.placeholder().is_none());
    assert!(demand.procedure_default().is_none());
    assert_eq!(demand.location().source, value.location().source);
    assert!(demand.location().span.start >= value.location().span.start);
    assert!(demand.location().span.end <= value.location().span.end);
    assert!(
        wait.dependencies
            .contains(&jai_vm::Dependency::Effect(jai_vm::EffectKey(42))),
        "{wait:?}"
    );
}

#[test]
fn initializer_lookup_retains_actual_consumer_beside_the_real_producer_effect() {
    let graph = waiting_graph();
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    for _ in 0..2 {
        let SourcePrefixReadiness::Pending(wait) = session.drive_source_prefix(&mut effects) else {
            panic!("the admitted real source producer must await its actual effect")
        };
        assert_lookup_wait(&graph, &wait);
    }
    assert_eq!(effects.requests, 1);
    session.cancel(&mut effects).unwrap();
    assert_eq!(effects.finishes, [false]);
    assert!(effects.writes.is_empty());
}

#[test]
fn exhausted_original_runs_preserve_the_initializer_lookup_as_a_hard_failure() {
    let graph = graph("value:int=#run generated(); #run 42; main::()->int{return value;}");
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    assert!(matches!(
        session.drive_source_prefix(&mut jai_vm::NoEffects),
        SourcePrefixReadiness::Ready
    ));
    let SourcePrefixReadiness::Failed(error) = session.drive_source_prefix(&mut jai_vm::NoEffects)
    else {
        panic!("no original source producer remains to satisfy this actual lookup")
    };
    assert!(
        error.message.contains("unknown name 'generated'"),
        "{error:?}"
    );
    let value = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "value")
        .unwrap();
    assert_eq!(error.location.source, value.location().source);
    assert!(error.location.span.start >= value.location().span.start);
    assert!(error.location.span.end <= value.location().span.end);
}

#[test]
fn direct_driving_does_not_claim_to_have_consumed_a_source_publication_round() {
    let graph = waiting_graph();
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let LibraryReadiness::Failed(error) = session.drive(&mut effects) else {
        panic!("the source publication controller must request the explicit prefix")
    };
    assert!(
        error.message.contains("unknown name 'generated'"),
        "{error:?}"
    );
    assert_eq!(effects.requests, 0);
    assert!(effects.finishes.is_empty());
}

#[test]
fn successful_builtin_lookup_alternative_does_not_turn_a_type_error_into_source_readiness() {
    for initializer in ["bool", "#run bool"] {
        let source = format!(
            "value:int={initializer}; compiler_create_workspace::(name:string)->s64 #compiler; #run,stallable compiler_create_workspace(\"child\"); main::()->int{{return value;}}"
        );
        let graph = graph(&source);
        let mut effects = Effects::default();
        let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
        let SourcePrefixReadiness::Failed(error) = session.drive_source_prefix(&mut effects) else {
            panic!("the resolved builtin type cannot become a generated-name demand")
        };
        assert!(
            error.location.span.text(&source).contains("bool"),
            "{error:?}"
        );
        assert!(!error.message.contains("unknown name"), "{error:?}");
        assert_eq!(effects.requests, 0);
        assert!(effects.finishes.is_empty());
    }
}
