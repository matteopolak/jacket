//! Authored source reservations never stand in for a value, type, or procedure.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn fixture_graph(application: &str, library: Option<&str>) -> ModuleGraph {
    let path = Path::new("/jai-source-placeholders/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay
        .insert(path, application.as_bytes().to_vec())
        .unwrap();
    if let Some(library) = library {
        overlay
            .insert(
                Path::new("/jai-source-placeholders/library.jai"),
                library.as_bytes().to_vec(),
            )
            .unwrap();
    }
    ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap()
}

fn run(graph: &ModuleGraph) -> i128 {
    let program = jai_sema::resolve_graph(graph).unwrap();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("placeholder fixture did not complete: {outcome:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn real_constant_record_and_procedure_fillers_keep_their_actual_identities() {
    let source = r#"
        #placeholder VALUE;
        #placeholder Box;
        #placeholder answer;
        VALUE::40;
        Box::struct{value:int=VALUE;}
        answer::()->int{value:Box;return value.value+2;}
        main::()->int{return answer();}
    "#;
    let graph = fixture_graph(source, None);
    assert_eq!(graph.declarations().len(), 4);
    let mut names: Vec<_> = graph
        .declarations()
        .iter()
        .map(|declaration| graph.symbols().name(declaration.name()))
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["Box", "VALUE", "answer", "main"]);
    assert_eq!(run(&graph), 42);
}

#[test]
fn fulfilled_overload_reservation_contains_only_real_callable_declarations() {
    let graph = fixture_graph(
        "#placeholder choose;choose::(value:int)->int{return value;}choose::(value:bool)->int{return 0;}main::()->int{return choose(42);}",
        None,
    );
    assert_eq!(graph.declarations().len(), 3);
    let program = jai_sema::resolve_graph(&graph).unwrap();
    assert_eq!(program.procedures().len(), 3);
    assert_eq!(run(&graph), 42);
}

#[test]
fn actual_filler_kind_is_checked_after_the_name_reservation_is_fulfilled() {
    for (source, role) in [
        ("#placeholder R;R::42;main::(){value:R;}", "type"),
        (
            "#placeholder answer;answer::42;main::()->int{return answer();}",
            "procedure",
        ),
    ] {
        let graph = fixture_graph(source, None);
        let error = jai_sema::resolve_graph(&graph).unwrap_err();
        assert!(!error.message.contains("unfilled"), "{error}");
        assert!(error.message.contains(role), "{error}");
    }
}

#[test]
fn unused_reservation_and_lexical_shadow_create_no_runtime_artifacts() {
    for source in [
        "#placeholder UNUSED;main::()->int{return 42;}",
        "#placeholder answer;main::()->int{answer::42;return answer;}",
    ] {
        let graph = fixture_graph(source, None);
        assert_eq!(graph.declarations().len(), 1);
        let program = jai_sema::resolve_graph(&graph).unwrap();
        assert_eq!(program.procedures().len(), 1);
        assert!(program.globals().is_empty());
        assert_eq!(run(&graph), 42);
    }
}

#[test]
fn unfilled_value_type_and_callable_demands_report_the_original_use() {
    for (source, name, demand) in [
        (
            "#placeholder VALUE;main::()->int{return VALUE;}",
            "VALUE",
            "VALUE",
        ),
        ("#placeholder R;main::(){value:R;}", "R", "value:R"),
        (
            "#placeholder answer;main::()->int{return answer();}",
            "answer",
            "answer",
        ),
    ] {
        let graph = fixture_graph(source, None);
        let error = jai_sema::resolve_graph(&graph).unwrap_err();
        assert!(
            error.message.contains("unfilled") && error.message.contains("#placeholder"),
            "{error}"
        );
        assert!(error.message.contains(name), "{error}");
        assert_eq!(error.location.span.start, source.rfind(demand).unwrap());
        assert_eq!(
            graph.sources().get(error.location.source).unwrap().path(),
            Path::new("/jai-source-placeholders/main.jai")
        );
    }
}

#[test]
fn marker_named_like_a_builtin_cannot_fall_back_to_the_builtin_type_value() {
    let source = "#placeholder int;main::()->int{return size_of(int);}";
    let graph = fixture_graph(source, None);
    let options = jai_sema::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    let error =
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap_err();
    assert!(error.message.contains("#placeholder"), "{error}");
    assert_eq!(error.location.span.text(source), "int");
}

#[test]
fn imported_marker_demand_preserves_the_export_and_private_boundaries() {
    let application = "Lib::#import,file \"library.jai\";main::()->int{return Lib.VALUE;}";
    let graph = fixture_graph(application, Some("#placeholder VALUE;"));
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("#placeholder"), "{error}");
    assert_eq!(error.location.span.text(application), "Lib.VALUE");

    for library in [
        "#scope_module;#placeholder VALUE;",
        "#placeholder VALUE;#scope_module;VALUE::42;",
    ] {
        let graph = fixture_graph(application, Some(library));
        let error = jai_sema::resolve_graph(&graph).unwrap_err();
        assert!(error.message.contains("private"), "{error}");
    }
}

#[test]
fn available_imported_callables_do_not_demand_unrelated_exported_markers() {
    for application in [
        "Lib::#import,file \"library.jai\";main::()->int{return Lib.seed();}",
        "main::()->int{#import,file \"library.jai\";return seed();}",
        "main::()->int{using Lib::#import,file \"library.jai\";return Lib.seed();}",
    ] {
        let graph = fixture_graph(
            application,
            Some("#placeholder LATER;seed::()->int{return 42;}"),
        );
        assert_eq!(run(&graph), 42);
    }
}

#[test]
fn scoped_placeholder_imports_preserve_selected_demand_and_local_shadowing() {
    let application = "main::()->int{#import,file \"library.jai\";return VALUE;}";
    let graph = fixture_graph(application, Some("#placeholder VALUE;"));
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("#placeholder"), "{error}");
    assert_eq!(error.location.span.text(application), "VALUE");

    let graph = fixture_graph(
        "main::()->int{#import,file \"library.jai\";VALUE::2;return seed()+VALUE;}",
        Some("#placeholder VALUE;seed::()->int{return 40;}"),
    );
    assert_eq!(run(&graph), 42);
}

#[test]
fn inactive_marker_does_not_reserve_or_materialize_a_name() {
    let graph = fixture_graph(
        "#if false{#placeholder VALUE;}VALUE::42;main::()->int{return VALUE;}",
        None,
    );
    assert_eq!(graph.declarations().len(), 2);
    assert_eq!(run(&graph), 42);
}

#[test]
fn selected_marker_before_a_pre_registered_filler_resolves_the_real_binding() {
    let graph = fixture_graph(
        "#if true{#placeholder VALUE;}VALUE::42;main::()->int{return VALUE;}",
        None,
    );
    assert_eq!(graph.declarations().len(), 2);
    assert_eq!(run(&graph), 42);
}
