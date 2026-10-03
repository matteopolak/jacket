//! Public driver lifecycle retains real source execution before graph advancement.
use jai_driver::{
    DiscoveryEffectPolicy, DiscoveryQuery, PreparedGraphDiscoverySession, SemanticDiscoveryOptions,
};
use jai_modules::{BootstrapOptions, GraphDiscovery, GraphOptions, SourceOverlay};
use jai_sema::{DiscoveryReadiness, PreparedDiscoveryOutcome};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{CompilerEffects, CompilerRequest, CompilerResponse, EffectKey, EffectOutcome};
use std::path::Path;

#[derive(Default)]
struct Effects {
    ready: bool,
    requests: usize,
    begins: usize,
    finishes: Vec<bool>,
}
impl CompilerEffects for Effects {
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        self.requests += 1;
        EffectOutcome::Pending(EffectKey(73))
    }
    fn poll_request(&mut self, _: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert_eq!(key, EffectKey(73));
        if self.ready {
            EffectOutcome::Ready(CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(73).unwrap(),
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
        self.finishes.push(commit);
        Ok(())
    }
}
fn options(policy: DiscoveryEffectPolicy) -> SemanticDiscoveryOptions {
    SemanticDiscoveryOptions {
        graph: GraphOptions::default(),
        bootstrap: BootstrapOptions::disabled(),
        target: BuildTarget {
            operating_system: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            layout: LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        },
        workspace: jai_vm::WorkspaceId::from_raw(1).unwrap(),
        limits: Default::default(),
        effect_policy: policy,
    }
}
fn overlay() -> SourceOverlay {
    let mut overlay = SourceOverlay::new();
    overlay.insert(Path::new("/driver-guard/main.jai"), br#"
        compiler_create_workspace::(name:string)->s64 #compiler;
        calls:int;
        choose::()->bool { calls+=1; result:=compiler_create_workspace("child"); return calls==1 && result==73; }
        #if #run,stallable choose() { #load "selected.jai"; } else { #load "absent.jai"; }
        main::()->int{return ANSWER;}
    "#.to_vec()).unwrap();
    overlay
        .insert(
            Path::new("/driver-guard/selected.jai"),
            b"ANSWER::42;".to_vec(),
        )
        .unwrap();
    overlay
}

#[test]
fn public_driver_two_drive_guard_publishes_after_releasing_its_graph_borrow() {
    let overlay = overlay();
    let options = options(DiscoveryEffectPolicy::CompilerSession);
    let mut graph = GraphDiscovery::with_target(
        Path::new("/driver-guard/main.jai"),
        options.graph.clone(),
        &overlay,
        options.target.clone(),
    )
    .unwrap();
    graph.advance().unwrap();
    let mut effects = Effects::default();
    let outcome = {
        let mut preparation =
            PreparedGraphDiscoverySession::new(&graph, &options, DiscoveryQuery::Conditions)
                .unwrap();
        match preparation.drive(&mut effects) {
            DiscoveryReadiness::Pending(_) => {}
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("the external observation cannot yet choose a branch"),
        }
        assert_eq!(effects.requests, 1);
        assert!(effects.finishes.is_empty());
        effects.ready = true;
        match preparation.drive(&mut effects) {
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Conditions(outcome)) => outcome,
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("ready source guard must finish"),
        }
    };
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.begins, 1);
    assert_eq!(effects.finishes, [true]);
    assert_eq!(outcome.decisions.len(), 1);
    assert!(outcome.decisions[0].1);
    for (id, selected) in outcome.decisions {
        graph.select_condition(id, selected).unwrap();
    }
    assert!(graph.advance().unwrap().is_complete());
    let graph = graph.into_graph().unwrap();
    assert!(
        graph
            .sources()
            .records()
            .iter()
            .all(|source| !source.path().ends_with("absent.jai"))
    );
    let resolve = jai_sema::ResolveOptions {
        target: Some(options.target.clone()),
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &options.graph.import_dirs,
            options.workspace,
        )),
        ..Default::default()
    };
    let program = jai_sema::resolve_graph_with_options(&graph, &resolve, &mut effects).unwrap();
    assert_eq!(effects.requests, 1);
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("VM failed");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn driver_cancellation_retires_exact_pending_transaction_without_advancing_graph() {
    let overlay = overlay();
    let options = options(DiscoveryEffectPolicy::CompilerSession);
    let mut graph = GraphDiscovery::with_target(
        Path::new("/driver-guard/main.jai"),
        options.graph.clone(),
        &overlay,
        options.target.clone(),
    )
    .unwrap();
    graph.advance().unwrap();
    let mut effects = Effects::default();
    {
        let mut preparation =
            PreparedGraphDiscoverySession::new(&graph, &options, DiscoveryQuery::Conditions)
                .unwrap();
        assert!(matches!(
            preparation.drive(&mut effects),
            DiscoveryReadiness::Pending(_)
        ));
        preparation.cancel(&mut effects).unwrap();
        assert!(matches!(
            preparation.drive(&mut effects),
            DiscoveryReadiness::Failed(_)
        ));
    }
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes, [false]);
    assert!(graph.graph().source_condition_selections().is_empty());
    assert!(!graph.advance().unwrap().is_complete());
}

#[test]
fn pure_check_policy_cannot_delegate_to_the_supplied_effects_owner() {
    let overlay = overlay();
    let options = options(DiscoveryEffectPolicy::Disabled);
    let mut graph = GraphDiscovery::with_target(
        Path::new("/driver-guard/main.jai"),
        options.graph.clone(),
        &overlay,
        options.target.clone(),
    )
    .unwrap();
    graph.advance().unwrap();
    let mut effects = Effects::default();
    let mut preparation =
        PreparedGraphDiscoverySession::new(&graph, &options, DiscoveryQuery::Conditions).unwrap();
    match preparation.drive(&mut effects) {
        DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Conditions(outcome)) => {
            assert!(outcome.decisions.is_empty());
            assert!(!outcome.pending.is_empty());
        }
        DiscoveryReadiness::Failed(_) => {}
        _ => panic!(
            "disabled compiler effects cannot produce a guard decision or park a transaction"
        ),
    }
    assert_eq!(effects.requests, 0);
    assert_eq!(effects.begins, 0);
}
