//! Closed authored cycle fixtures; no original source or host effects.
use jai_modules::{Binding, DependencyKind, GraphError, GraphOptions, ModuleGraph, SourceOverlay};
use jai_syntax::NamePath;
use std::path::Path;

fn graph(files: &[(&str, &str)]) -> Result<ModuleGraph, GraphError> {
    let mut sources = SourceOverlay::new();
    for (name, source) in files {
        sources
            .insert(
                Path::new(&format!("/cycles/{name}")),
                source.as_bytes().to_vec(),
            )
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/cycles/main.jai"),
        GraphOptions {
            import_dirs: vec!["/cycles".into()],
        },
        &sources,
    )
}

fn lookup(graph: &ModuleGraph, names: &[&str]) -> Binding {
    let names = names
        .iter()
        .map(|name| graph.symbols().find(name).unwrap())
        .collect::<Vec<_>>();
    graph
        .lookup(
            graph.module(graph.root()).unwrap().entry(),
            &NamePath {
                root: names[0],
                members: names[1..].to_vec(),
            },
        )
        .unwrap()
}

#[test]
fn namespace_backedges_keep_one_module_and_original_nominal_declarations() {
    let graph = graph(&[
        ("main.jai", "A::#import \"A\"; B::#import \"B\";"),
        ("A.jai", "B::#import \"B\"; Left::struct {link:*B.Right;}"),
        ("B.jai", "A::#import \"A\"; Right::struct {link:*A.Left;}"),
    ])
    .unwrap();
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.files().len(), 3);
    assert_eq!(lookup(&graph, &["A"]), lookup(&graph, &["B", "A"]));
    assert_eq!(lookup(&graph, &["B"]), lookup(&graph, &["A", "B"]));
    assert_eq!(
        lookup(&graph, &["A", "Left"]),
        lookup(&graph, &["B", "A", "Left"])
    );
    assert_ne!(
        lookup(&graph, &["A", "Left"]),
        lookup(&graph, &["B", "Right"])
    );
}

#[test]
fn late_loaded_exports_propagate_and_diamond_imports_are_idempotent() {
    let graph = graph(&[
        (
            "main.jai",
            "A::#import \"A\"; B::#import \"B\"; #import \"A\"; #import \"B\";",
        ),
        ("A.jai", "#import \"B\"; #load \"late.jai\";"),
        ("B.jai", "#import \"A\"; value_b::2;"),
        ("late.jai", "value_a::40;"),
    ])
    .unwrap();
    for name in ["value_a", "value_b"] {
        let binding = lookup(&graph, &[name]);
        assert_eq!(binding, lookup(&graph, &["A", name]));
        assert_eq!(binding, lookup(&graph, &["B", name]));
    }
    assert_eq!(graph.declarations().len(), 2);
    assert_eq!(graph.imports().len(), 6);
}

#[test]
fn cycle_overloads_reuse_exact_original_member_ids() {
    let graph = graph(&[
        ("main.jai", "A::#import \"A\"; B::#import \"B\";"),
        ("A.jai", "#import \"B\"; pick::(x:int)->int{return x;}"),
        ("B.jai", "#import \"A\"; pick::(x:bool)->bool{return x;}"),
    ])
    .unwrap();
    let a = lookup(&graph, &["A", "pick"]);
    assert_eq!(a, lookup(&graph, &["B", "pick"]));
    let Binding::OverloadSet(set) = a else {
        panic!("expected original overload group")
    };
    let members = graph.overload_set(set).unwrap().declarations();
    assert_eq!(members.len(), 2);
    assert_ne!(members[0], members[1]);
    assert_eq!(graph.declarations().len(), 2);
}

#[test]
fn file_private_import_backedges_do_not_reexport_the_namespace() {
    let graph = graph(&[
        ("main.jai", "A::#import \"A\"; B::#import \"B\";"),
        (
            "A.jai",
            "#scope_file B::#import \"B\"; #scope_export left::1;",
        ),
        (
            "B.jai",
            "#scope_file A::#import \"A\"; #scope_export right::2;",
        ),
    ])
    .unwrap();
    let Binding::Module(a) = lookup(&graph, &["A"]) else {
        panic!()
    };
    let b = graph.symbols().find("B").unwrap();
    assert!(!graph.module(a).unwrap().exports().contains_key(&b));
    assert_eq!(graph.imports().len(), 4);
}

#[test]
fn same_parameter_request_backedge_reuses_the_reserved_instance() {
    let graph = graph(&[
        ("main.jai", "A::#import \"A\"(n=cast(s64)7);"),
        (
            "A.jai",
            "#module_parameters(n:s64); B::#import \"B\"(n=n); left::n;",
        ),
        (
            "B.jai",
            "#module_parameters(n:s64); A::#import \"A\"(n=n); right::n;",
        ),
    ])
    .unwrap();
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(lookup(&graph, &["A"]), lookup(&graph, &["A", "B", "A"]));
}

#[test]
fn changing_parameter_recursion_is_still_a_located_cycle_error() {
    let error = graph(&[
        ("main.jai", "A::#import \"A\"(n=0);"),
        ("A.jai", "#module_parameters(n:s64); B::#import \"B\"(n=n);"),
        (
            "B.jai",
            "#module_parameters(n:s64); A::#import \"A\"(n=n+1);",
        ),
    ])
    .unwrap_err();
    assert!(matches!(
        &error,
        GraphError::Cycle {
            kind: DependencyKind::Import,
            location: Some(_),
            ..
        }
    ));
    assert!(error.to_string().contains("B.jai:1:"));
}

#[test]
fn genuine_nominal_collision_across_a_cycle_remains_an_error() {
    let error = graph(&[
        ("main.jai", "#import \"A\";"),
        ("A.jai", "#import \"B\"; Position::struct {line:int;}"),
        ("B.jai", "#import \"A\"; Position::struct {line:int;}"),
    ])
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("conflicting declaration or import 'Position'"));
    assert!(message.contains("A.jai:1:"));
    assert!(message.contains("B.jai:1:"));
}

#[test]
fn active_source_load_cycles_remain_errors() {
    let error = graph(&[
        ("main.jai", "#load \"other.jai\";"),
        ("other.jai", "#load \"main.jai\";"),
    ])
    .unwrap_err();
    assert!(matches!(
        error,
        GraphError::Cycle {
            kind: DependencyKind::Load,
            location: Some(_),
            ..
        }
    ));
}

#[test]
fn active_backedge_cannot_reconfigure_program_parameters() {
    let error = graph(&[(
        "main.jai",
        "#module_parameters () (enabled:=false); Root::#import,file \"main.jai\"()(enabled=true);",
    )])
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("before the module begins dependency expansion")
    );
}

#[test]
fn cyclic_namespace_dependency_waits_for_a_later_loaded_condition_value() {
    let graph = graph(&[
        ("main.jai", "A::#import \"A\";"),
        ("A.jai", "B::#import \"B\"; #load \"late.jai\";"),
        (
            "B.jai",
            "A::#import \"A\"; #if A.enabled { #load \"selected.jai\"; }",
        ),
        ("late.jai", "enabled::true;"),
        ("selected.jai", "selected_value::42;"),
    ])
    .unwrap();
    let Binding::Declaration(id) = lookup(&graph, &["A", "B", "selected_value"]) else {
        panic!()
    };
    let source = graph
        .sources()
        .get(graph.declaration(id).unwrap().location().source)
        .unwrap();
    assert_eq!(source.path(), Path::new("/cycles/selected.jai"));
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.files().len(), 5);
}

#[test]
fn completed_other_instance_is_reusable_while_the_same_source_is_active() {
    let graph = graph(&[
        ("main.jai", "Zero::#import \"A\"(n=cast(s64)0); One::#import \"A\"(n=cast(s64)1);"),
        ("A.jai", "#module_parameters(n:s64); #if n==1 { Earlier::#import \"A\"(n=cast(s64)0); } value::n;"),
    ]).unwrap();
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(
        lookup(&graph, &["Zero"]),
        lookup(&graph, &["One", "Earlier"])
    );
    assert_ne!(
        lookup(&graph, &["Zero", "value"]),
        lookup(&graph, &["One", "value"])
    );
}
