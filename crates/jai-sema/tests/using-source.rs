//! Genuine parsed directives retain source bindings and mutable field places.
use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn execute(source: &str, library: &str) {
    let mut overlay = SourceOverlay::new();
    for (path, text) in [
        ("/using-source/main.jai", source),
        ("/using-source/library.jai", library),
    ] {
        overlay
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = GraphDiscovery::new(
        Path::new("/using-source/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap();
    for _ in 0..16 {
        if discovery.advance().unwrap().is_complete() {
            break;
        }
        let requests = discovery.pending_using_requests();
        assert!(!requests.is_empty());
        let outcome = jai_sema::resolve_discovery_using(
            discovery.graph(),
            &requests,
            &jai_sema::ResolveOptions::default(),
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        assert!(outcome.pending.is_empty(), "{:?}", outcome.pending);
        for (id, decision) in outcome.decisions {
            discovery.resolve_using(id, decision).unwrap();
        }
    }
    let graph = discovery.into_graph().unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn lexical_enum_using_resolves_actual_nominal_members() {
    execute(
        "Choice::enum{ ANSWER::42; } main::()->int{ using Choice; return cast(int) ANSWER; }",
        "",
    );
}

#[test]
fn imported_using_enum_keeps_namespace_type_aliases_available_during_preparation() {
    execute(
        "Lib::#import,file \"library.jai\"; Level::Lib.Width; main::()->int{value:Level=42;return cast(int)value;}",
        "Width::u64; using Flags::enum_flags Width{FIRST::1;}",
    );
}

#[test]
fn imported_using_enum_keeps_anonymous_type_aliases_available_during_preparation() {
    execute(
        "#import,file \"library.jai\"; Level::Width; main::()->int{value:Level=42;return cast(int)value;}",
        "Width::u64; using Flags::enum_flags Width{FIRST::1;}",
    );
}

#[test]
fn computed_namespace_filters_and_map_publish_real_declarations() {
    execute(
        "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"value\"]; } using,only(names()) Lib; main::()->int{return value;}",
        "value::42; omitted::5;",
    );
    execute(
        "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"omitted\"]; } using,except(names()) Lib; main::()->int{return value;}",
        "value::42; omitted::5;",
    );
    execute(
        "Lib::#import,file \"library.jai\"; mapper::(names:[]string){names[0]=\"renamed\";} using,map(mapper) Lib; main::()->int{return renamed;}",
        "value::42;",
    );
}

#[test]
fn mapped_place_alias_mutates_original_field() {
    execute(
        "Record::struct{original:int;} mapper::(names:[]string){names[0]=\"renamed\";} main::()->int{record:Record; using,map(mapper) record; renamed=42; return record.original;}",
        "",
    );
}

#[test]
fn declaration_using_resolves_original_file_and_lexical_enum_children() {
    execute(
        "using Choice::enum{ ANSWER::42; } main::()->int{return cast(int) ANSWER;}",
        "",
    );
    execute(
        "main::()->int{using Choice::enum{ ANSWER::42; } return cast(int) ANSWER;}",
        "",
    );
}

#[test]
fn declaration_using_checks_real_runtime_child_and_preserves_its_place() {
    execute(
        "Record::struct{value:int;} make::()->Record{return .{41};} main::()->int{using record:=make(); value+=1; return record.value;}",
        "",
    );
}

#[test]
fn file_storage_using_updates_original_global_and_nested_field_paths() {
    execute(
        "Pair::struct{value:int=1;} using state:Pair; main::()->int{value+=41;return state.value;}",
        "",
    );
    execute(
        "Inner::struct{value:int=1;} Outer::struct{inner:Inner;} state:Outer; using state.inner; main::()->int{value+=41;return state.inner.value;}",
        "",
    );
}

#[test]
fn exported_storage_using_preserves_defining_global_across_module_boundaries() {
    execute(
        "Lib::#import,file \"library.jai\"; main::()->int{Lib.value+=41;return Lib.read();}",
        "Pair::struct{value:int=1;} #scope_file state:Pair; #scope_export using state; read::()->int{return state.value;}",
    );
}
