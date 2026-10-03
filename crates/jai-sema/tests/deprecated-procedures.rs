use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn resolve(source: &str) -> jai_ir::Program {
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/deprecated-source/main.jai");
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    jai_sema::resolve_graph(&graph)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())))
}

#[test]
fn selected_references_warn_with_advice_and_both_original_source_sites() {
    let program = resolve(
        "old::()->int #deprecated \"use fresh\" #no_debug{return 42;} unused::()->int #deprecated{return 1;} main::()->int{return old();}",
    );
    let warnings = program.library().source_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].message().contains("old"));
    assert!(warnings[0].message().contains("use fresh"));
    assert_eq!(warnings[0].location().text(), "old()");
    assert_eq!(
        warnings[0].notes()[0].location.text(),
        "#deprecated \"use fresh\""
    );
    assert!(
        matches!(jai_vm::execute(&program,jai_vm::Limits::default()).outcome,jai_vm::Outcome::Complete(values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42))
    );
}

#[test]
fn calls_from_a_deprecated_definition_only_warn_at_the_outside_reference() {
    let program = resolve(
        "old::()->int #deprecated{return 42;} wrapper::()->int #deprecated{return old();} main::()->int{return wrapper();}",
    );
    let warnings = program.library().source_warnings();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].message().contains("wrapper"));
}

#[test]
fn selected_generic_and_local_procedures_keep_their_original_annotations() {
    let program = resolve(
        "old::(arg:$T)->T #deprecated{return arg;} main::()->int{local::(arg:int)->int #deprecated{return arg;}return local(old(42));}",
    );
    assert_eq!(program.library().source_warnings().len(), 2);
}

#[test]
fn marked_macro_body_suppresses_nested_calls_but_caller_arguments_still_warn() {
    let program = resolve(
        "old::()->int #deprecated{return 42;} quiet::(arg:int) #expand #deprecated{value:=old();} main::()->int{quiet(old());return 42;}",
    );
    let warnings = program.library().source_warnings();
    assert_eq!(warnings.len(), 2);
    assert!(
        warnings
            .iter()
            .any(|warning| warning.message().contains("quiet"))
    );
    assert!(
        warnings
            .iter()
            .any(|warning| warning.message().contains("old"))
    );
}

#[test]
fn rejected_overload_candidates_do_not_warn_and_addresses_keep_original_targets() {
    let clean = resolve(
        "pick::(arg:bool)->int #deprecated{return 1;} pick::(arg:int)->int{return arg;} main::()->int{return pick(42);}",
    );
    assert!(clean.library().source_warnings().is_empty());
    let selected = resolve(
        "old::(arg:int)->int #deprecated{return arg;} alias::old; main::()->int{callback:=old;return alias(callback(42));}",
    );
    assert_eq!(selected.library().source_warnings().len(), 2);
    let sites: Vec<_> = selected
        .library()
        .source_warnings()
        .iter()
        .map(|warning| warning.location().text())
        .collect();
    assert!(
        sites.contains(&"old"),
        "the actual procedure address reference must warn"
    );
    assert!(
        sites.contains(&"alias(callback(42))"),
        "the callable alias use must warn at its call site"
    );
    assert!(
        selected
            .library()
            .source_warnings()
            .iter()
            .all(|warning| warning.message().contains("'old'"))
    );
}
