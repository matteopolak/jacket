//! Selected early defaults retain genuine source prerequisites, values, and rollback.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{LibraryReadiness, PreparedLibrarySession, ResolveOptions, SourcePrefixReadiness};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/prepared-source-defaults/main.jai");
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
fn finish_prefix(
    session: &mut PreparedLibrarySession<'_>,
    effects: &mut dyn jai_vm::CompilerEffects,
) {
    for _ in 0..16 {
        match session.drive_source_prefix(effects) {
            SourcePrefixReadiness::Ready => {}
            SourcePrefixReadiness::Complete => return,
            SourcePrefixReadiness::Pending(wait) => panic!("{wait:?}"),
            SourcePrefixReadiness::Failed(error) => panic!("{error:?}"),
        }
    }
    panic!("actual selected source prefix must reach a finite checkpoint")
}

#[test]
fn early_selected_false_default_completes_before_the_unrelated_placeholder() {
    let graph = graph(
        "#placeholder Later; Alias::#type *Later; \
         read::(flag:=false)->int #no_context {if flag return 0;return 42;} \
         #run read(); main::()->int{return 42;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    finish_prefix(&mut session, &mut jai_vm::NoEffects);
    let LibraryReadiness::Pending(wait) = session.drive(&mut jai_vm::NoEffects) else {
        panic!("the selected real default cannot fill an unrelated placeholder")
    };
    assert!(wait.dependencies.is_empty());
    assert_eq!(
        wait.source.unwrap().placeholder(),
        Some(graph.placeholders()[0].id())
    );
    session.cancel(&mut jai_vm::NoEffects).unwrap();
}

#[test]
fn early_selected_run_default_uses_its_checked_body_and_skips_unselected_defaults() {
    let graph = graph(
        "#placeholder Later; Alias::#type *Later; \
         choose::()->bool #no_context{return false;} \
         read::(flag:bool=#run choose())->int #no_context {if flag return 0;return 42;} \
         fail::()->int #no_context{return 1/0;} \
         unrelated::(value:int=#run fail())->int #no_context{return value;} \
         #run read(); main::()->int{return 42;}",
    );
    let mut session = PreparedLibrarySession::new(&graph, &ResolveOptions::default()).unwrap();
    finish_prefix(&mut session, &mut jai_vm::NoEffects);
    session.cancel(&mut jai_vm::NoEffects).unwrap();
}

#[derive(Default)]
struct Effects {
    ready: bool,
    begins: usize,
    suspensions: usize,
    resumes: usize,
    serviced_dependencies: Vec<Vec<jai_vm::Dependency>>,
    requests: usize,
    issued_writes: usize,
    staged: Vec<Vec<u8>>,
    writes: Vec<Vec<u8>>,
    finishes: Vec<bool>,
}
impl jai_vm::CompilerEffects for Effects {
    fn begin(&mut self) {
        self.begins += 1;
        assert!(
            self.staged.is_empty(),
            "a fresh transaction cannot own prior staged output"
        );
    }
    fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
        match request {
            jai_vm::CompilerRequest::CreateWorkspace { .. } => {
                self.requests += 1;
                jai_vm::EffectOutcome::Pending(jai_vm::EffectKey(42))
            }
            jai_vm::CompilerRequest::WriteOutput { bytes, .. } => {
                self.issued_writes += 1;
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
        self.suspensions += 1;
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        self.resumes += 1;
        Ok(())
    }
    fn service_pending(
        &mut self,
        dependencies: &[jai_vm::Dependency],
    ) -> Result<bool, jai_vm::Error> {
        self.serviced_dependencies.push(dependencies.to_vec());
        Ok(false)
    }
}
fn pending_graph() -> ModuleGraph {
    graph(
        "#placeholder Later; Alias::#type *Later; \
         write_string::(s:string,to_standard_error:bool) #no_context #compiler; \
         compiler_create_workspace::(name:string)->s64 #compiler; \
         choose::()->bool {write_string(\"before\",false);receipt:=compiler_create_workspace(\"child\");write_string(\"after\",false);return false;} \
         read::(flag:bool=#run,stallable choose())->int #no_context {if flag return 0;return 42;} \
         #run read(); main::()->int{return 42;}",
    )
}
fn assert_default_wait(graph: &ModuleGraph, wait: &jai_sema::LibraryPending) {
    assert!(
        wait.dependencies
            .contains(&jai_vm::Dependency::Effect(jai_vm::EffectKey(42))),
        "{wait:?}"
    );
    let read = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "read")
        .unwrap();
    let source = wait
        .source
        .expect("real source default retains its identity beside VM dependencies");
    assert_eq!(source.procedure_default(), Some((read.id(), 0)));
    let jai_syntax::FileDeclarationKind::Procedure(procedure) = &read.syntax().kind else {
        panic!("source procedure")
    };
    let original = match &procedure.parameters[0].binding {
        jai_syntax::ParameterBinding::Defaulted { expression, .. }
        | jai_syntax::ParameterBinding::DefaultedType { expression, .. } => expression,
        jai_syntax::ParameterBinding::Required(_)
        | jai_syntax::ParameterBinding::RequiredType(_) => panic!("original source default"),
    };
    assert_eq!(source.location().source, read.location().source);
    assert_eq!(source.location().span, original.span);
}

#[test]
fn selected_default_retains_the_same_effect_request_and_source_across_resume() {
    let graph = pending_graph();
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    for _ in 0..2 {
        let SourcePrefixReadiness::Pending(wait) = session.drive_source_prefix(&mut effects) else {
            panic!("selected default must await its real effect")
        };
        assert_default_wait(&graph, &wait);
    }
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.staged, [b"before".to_vec()]);
    assert!(effects.writes.is_empty());
    effects.ready = true;
    finish_prefix(&mut session, &mut effects);
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.writes, [b"before".to_vec(), b"after".to_vec()]);
    session.cancel(&mut effects).unwrap();
}

#[test]
fn cancelling_a_selected_default_rolls_back_its_actual_pending_run() {
    let graph = pending_graph();
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let SourcePrefixReadiness::Pending(wait) = session.drive_source_prefix(&mut effects) else {
        panic!("selected default must await its real effect")
    };
    assert_default_wait(&graph, &wait);
    session.cancel(&mut effects).unwrap();
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes, [false]);
    assert!(effects.writes.is_empty());
    assert!(effects.staged.is_empty());
    assert!(matches!(
        session.drive_source_prefix(&mut effects),
        SourcePrefixReadiness::Failed(_)
    ));
}

#[test]
fn a_failed_selected_default_does_not_commit_its_source_effects() {
    let graph = graph(
        "#placeholder Later; Alias::#type *Later; \
         write_string::(s:string,to_standard_error:bool) #no_context #compiler; \
         make_zero::()->int #no_context{return 0;} \
         choose::()->bool {write_string(\"private\",false);zero:=make_zero();return 1/zero==0;} \
         read::(flag:bool=#run choose())->int #no_context {if flag return 0;return 42;} \
         #run read(); main::()->int{return 42;}",
    );
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let SourcePrefixReadiness::Failed(error) = session.drive_source_prefix(&mut effects) else {
        panic!("the original selected default must fail")
    };
    assert_eq!(
        effects.issued_writes, 1,
        "the failing VM must issue its original write"
    );
    assert_eq!(effects.begins, 1);
    assert!(!effects.serviced_dependencies.is_empty());
    let mut procedures = std::collections::HashSet::new();
    for dependencies in &effects.serviced_dependencies {
        assert!(!dependencies.is_empty());
        for dependency in dependencies {
            let jai_vm::Dependency::Procedure(procedure) = dependency else {
                panic!(
                    "this original default waits only on its reached source procedures: {dependency:?}"
                )
            };
            procedures.insert(*procedure);
        }
    }
    assert_eq!(
        procedures.len(),
        2,
        "choose and make_zero have distinct actual emitted owners"
    );
    assert_eq!(effects.suspensions, effects.serviced_dependencies.len());
    assert_eq!(effects.resumes, effects.suspensions);
    assert_eq!(
        error.message,
        jai_vm::Error::Arithmetic(jai_vm::ArithmeticError::ZeroDivisor).to_string(),
    );
    let read = graph
        .declarations()
        .iter()
        .find(|source| graph.symbols().name(source.name()) == "read")
        .unwrap();
    let jai_syntax::FileDeclarationKind::Procedure(procedure) = &read.syntax().kind else {
        panic!("the original selected default belongs to a source procedure")
    };
    let original = match &procedure.parameters[0].binding {
        jai_syntax::ParameterBinding::Defaulted { expression, .. }
        | jai_syntax::ParameterBinding::DefaultedType { expression, .. } => expression,
        jai_syntax::ParameterBinding::Required(_)
        | jai_syntax::ParameterBinding::RequiredType(_) => panic!("original source default"),
    };
    assert_eq!(error.location.source, read.location().source);
    assert_eq!(error.location.span, original.span);
    assert_eq!(effects.finishes, [false]);
    assert!(effects.writes.is_empty());
    assert!(effects.staged.is_empty());
}

#[test]
fn an_ordinary_default_does_not_authorize_an_external_effect_wait() {
    let graph = graph(
        "#placeholder Later; Alias::#type *Later; \
         write_string::(s:string,to_standard_error:bool) #no_context #compiler; \
         compiler_create_workspace::(name:string)->s64 #compiler; \
         choose::()->bool {write_string(\"private\",false);receipt:=compiler_create_workspace(\"child\");return false;} \
         read::(flag:bool=#run choose())->int #no_context {if flag return 0;return 42;} \
         #run read(); main::()->int{return 42;}",
    );
    let mut effects = Effects::default();
    let mut session = PreparedLibrarySession::new(&graph, &options(&graph)).unwrap();
    let SourcePrefixReadiness::Failed(error) = session.drive_source_prefix(&mut effects) else {
        panic!("an ordinary source default cannot authorize an external wait")
    };
    assert_eq!(effects.issued_writes, 1);
    assert_eq!(effects.requests, 1);
    assert!(
        effects
            .serviced_dependencies
            .iter()
            .flatten()
            .all(|dependency| {
                !matches!(
                    dependency,
                    jai_vm::Dependency::Effect(_) | jai_vm::Dependency::Host(_)
                )
            })
    );
    assert_eq!(effects.finishes, [false]);
    assert!(effects.writes.is_empty());
    assert!(effects.staged.is_empty());
    assert_eq!(
        error.message,
        "compiler or host dependencies require #run,stallable"
    );
    assert!(matches!(
        session.drive_source_prefix(&mut effects),
        SourcePrefixReadiness::Failed(_)
    ));
}
