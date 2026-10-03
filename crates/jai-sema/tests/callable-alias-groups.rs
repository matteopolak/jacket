//! Overload aliases keep the target identity, defaults and defining scope.
use jai_modules::{Binding, GraphDiscovery, GraphOptions, ModuleGraph, SourceOverlay};
use jai_syntax::{FileDeclarationKind, NamePath};
use jai_vm::{Limits, Outcome};
use std::path::Path;

fn execute(application: &str, library: Option<&str>, procedures: usize) {
    let mut sources = SourceOverlay::new();
    sources
        .insert(
            Path::new("/callable-aliases/main.jai"),
            application.as_bytes().to_vec(),
        )
        .unwrap();
    if let Some(library) = library {
        sources
            .insert(
                Path::new("/callable-aliases/library.jai"),
                library.as_bytes().to_vec(),
            )
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/callable-aliases/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    assert_eq!(
        program.procedures().len(),
        procedures,
        "alias minted a wrapper procedure"
    );
    let result = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(result.outcome, Outcome::Complete(ref values) if values[0].integer().is_ok_and(|value| value.value() == 42)),
        "{result:?}"
    );
}

#[test]
fn forward_alias_joins_overloads_with_target_named_defaults() {
    execute(
        "pick::later;later::(value:int=19)->int{return value;}pick::(value:bool)->int{return ifx value then 23 else 0;}main::()->int{return pick()+pick(value=true);}",
        None,
        3,
    );
}

#[test]
fn imported_alias_group_keeps_private_default_and_target_identity() {
    execute(
        "main::()->int{Library::#import,file \"library.jai\";return Library.pick()+Library.pick(value=true);}",
        Some(
            "#scope_file DEFAULT::19;hidden::(value:int=DEFAULT)->int{return value;}forward::hidden;#scope_export pick::forward;pick::(value:bool)->int{return ifx value then 23 else 0;}",
        ),
        3,
    );
}

#[test]
fn alias_group_specializes_the_original_generic_target_once() {
    execute(
        "pick::identity;identity::(value:$T)->T{return value;}pick::(value:bool)->int{return ifx value then 23 else 0;}main::()->int{left:int=20;right:int=22;return pick(value=left)+pick(value=right);}",
        None,
        3,
    );
}

#[test]
fn compile_time_guard_sees_alias_members_before_module_finalization() {
    let mut sources = SourceOverlay::new();
    let path = Path::new("/callable-aliases/main.jai");
    sources
        .insert(
            path,
            b"pick::(value:int)->bool{return false;}pick::alias;alias::(value:bool)->bool{return true;}#if #run pick(true){answer::42;}else{answer::0;}main::()->int{return answer;}".to_vec(),
        )
        .unwrap();
    let mut discovery = GraphDiscovery::new(path, GraphOptions::default(), &sources).unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let conditions = discovery.pending_conditions().cloned().collect::<Vec<_>>();
    assert_eq!(conditions.len(), 1);
    let outcome = jai_sema::resolve_discovery_conditions(
        discovery.graph(),
        &conditions,
        &Default::default(),
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    assert!(outcome.pending.is_empty(), "{:?}", outcome.pending);
    let [(id, true)] = outcome.decisions.as_slice() else {
        panic!(
            "partial overload family selected the wrong source target: {:?}",
            outcome.decisions
        )
    };
    discovery.select_condition(*id, true).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(result.outcome, Outcome::Complete(ref values) if values[0].integer().is_ok_and(|value| value.value() == 42)),
        "{result:?}"
    );
}

#[test]
fn unchanged_basic_print_alias_has_canonical_graph_members() {
    let source =
    if !source.is_file() {
        eprintln!("reference Basic/Print.jai is absent; source graph check skipped");
        return;
    }
    let mut discovery =
        GraphDiscovery::new(&source, GraphOptions::default(), &jai_modules::Filesystem).unwrap();
    // Standalone Print source can still await its real Preload using inputs.
    // Its registered alias family must already have honest target membership.
    let _status = discovery.advance().unwrap();
    let graph = discovery.graph();
    let alias = graph
        .declarations()
        .iter()
        .find(|declaration| {
            graph.symbols().name(declaration.name()) == "print"
                && matches!(declaration.syntax().kind, FileDeclarationKind::Constant(_))
        })
        .expect("actual source alias must remain in the declaration arena");
    let Binding::OverloadSet(group) = graph
        .lookup(
            alias.file(),
            &NamePath {
                root: alias.name(),
                members: vec![],
            },
        )
        .unwrap()
    else {
        panic!("actual print alias did not join the source overload group")
    };
    let members = graph.overload_set(group).unwrap().declarations();
    assert_eq!(members.len(), 2);
    let names = members
        .iter()
        .map(|id| {
            let declaration = graph.declaration(*id).unwrap();
            assert!(matches!(
                declaration.syntax().kind,
                FileDeclarationKind::Procedure(_)
            ));
            graph.symbols().name(declaration.name())
        })
        .collect::<Vec<_>>();
    assert!(names.contains(&"print") && names.contains(&"print_to_builder"));
}
