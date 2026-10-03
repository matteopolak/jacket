//! No-write field recipes retain shape without manufacturing initializer values.
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::Path;

fn graph(source: &str) -> ModuleGraph {
    let path = Path::new("/jai-local-no-write/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap()
}

fn options() -> ResolveOptions {
    ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..ResolveOptions::default()
    }
}

fn run(source: &str) -> i128 {
    let program = resolve_graph_with_options(&graph(source), &options(), &mut NoEffects).unwrap();
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    let Outcome::Complete(values) = outcome else {
        panic!("no-write fixture did not complete: {outcome:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

#[test]
fn unused_local_no_write_record_keeps_its_valid_shape() {
    assert_eq!(
        run("main::()->int{Unused::struct{value:int=---;}return 42;}"),
        42
    );
}

#[test]
fn whole_record_no_write_declaration_never_requests_field_defaults() {
    assert_eq!(
        run(
            "main::()->int{R::struct{value:int=---;}value:R=---;value.value=42;return value.value;}"
        ),
        42
    );
}

#[test]
fn selected_overrides_over_no_write_storage_remain_unconstructed() {
    assert_eq!(
        run(r#"
            main::()->int {
                Base::struct{value:int=7;}
                R::struct {
                    using base:Base=---;
                    #if ENABLED {base.value=21;base.value=42;}
                    else {base.missing=MISSING;}
                    ENABLED::true;
                }
                value:R=---;
                value.base.value=42;
                return value.base.value;
            }
        "#),
        42
    );
}

#[test]
fn implicit_construction_reports_the_original_no_write_recipe() {
    for source in [
        "main::(){R::struct{value:int=---;}value:R;}",
        "main::(){Base::struct{value:int=7;}R::struct{using base:Base=---;base.value=42;}value:R;}",
    ] {
        let error =
            resolve_graph_with_options(&graph(source), &options(), &mut NoEffects).unwrap_err();
        assert!(
            error.message.contains("construction") && error.message.contains("uninitialized"),
            "unexpected no-write diagnostic: {error}"
        );
        assert!(!error.message.contains("pending"), "{error}");
        assert_eq!(error.location.span.text(source), "---");
    }
}
