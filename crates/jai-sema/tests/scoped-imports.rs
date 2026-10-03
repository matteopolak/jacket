//! Source-only fixtures exercise lexical modules and their canonical callbacks.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn graph(application: &str, library: &str) -> ModuleGraph {
    let mut provider = SourceOverlay::new();
    for (path, source) in [
        ("/lexical-imports/main.jai", application),
        ("/lexical-imports/library.jai", library),
    ] {
        provider
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/lexical-imports/main.jai"),
        GraphOptions::default(),
        &provider,
    )
    .unwrap()
}

fn execute(application: &str, library: &str) {
    let program = jai_sema::resolve_graph(&graph(application, library)).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn namespace_alias_stays_in_defining_procedure_and_calls_module_body() {
    execute(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.answer(); }",
        "answer :: () -> int { return 42; }",
    );
    let error = jai_sema::resolve_graph(&graph("first :: () { Lib :: #import,file \"library.jai\"; } main :: () -> int { return Lib.answer(); }", "answer :: () -> int { return 42; }")).unwrap_err();
    assert!(error.message.contains("Lib"), "{error}");
}

#[test]
fn nested_alias_shadow_uses_selected_module_without_changing_outer_names() {
    execute(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; result := Lib.answer(); { Lib :: 21; result += Lib; } return result; }",
        "answer :: () -> int { return 21; }",
    );
}

#[test]
fn using_exports_keep_local_declaration_shadowing() {
    execute(
        "main :: () -> int { using Lib :: #import,file \"library.jai\"; answer :: 21; return Lib.answer() + answer; }",
        "answer :: () -> int { return 21; }",
    );
    execute(
        "main :: () -> int { #import,file \"library.jai\"; return answer(); }",
        "answer :: () -> int { return 42; }",
    );
}

#[test]
fn exported_callback_keeps_private_defining_procedure_identity() {
    execute(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.answer() + Lib.callback(); }",
        "#scope_module; private_answer :: () -> int { return 21; } #scope_export; callback :: private_answer; answer :: () -> int { return callback(); }",
    );
    let error = jai_sema::resolve_graph(&graph(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.private_answer(); }",
        "#scope_module; private_answer :: () -> int { return 42; }",
    ))
    .unwrap_err();
    assert!(error.message.contains("private"), "{error}");
}

#[test]
fn selected_local_branch_ignores_missing_module_and_preserves_import_scope() {
    execute(
        "main :: () -> int { #if false { Missing :: #import \"Absent\"; } else { Lib :: #import,file \"library.jai\"; return Lib.answer(); } }",
        "answer :: () -> int { return 42; }",
    );
}

#[test]
fn repeated_requests_share_canonical_module_globals() {
    execute(
        "first :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.next(); } main :: () -> int { Lib :: #import,file \"./library.jai\"; return first() + Lib.next(); }",
        "value: int = 19; next :: () -> int { value += 2; return value - 1; }",
    );
}

#[test]
fn imported_global_places_keep_their_own_module_storage() {
    execute(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; Lib.value += 2; return Lib.value; }",
        "value: int = 40;",
    );
}

#[test]
fn imported_callback_storage_calls_its_private_source_target() {
    execute(
        "main :: () -> int { Lib :: #import,file \"library.jai\"; return Lib.callback(); }",
        "#scope_module; helper :: () -> int { return 42; } #scope_export; callback: () -> int = helper;",
    );
}

#[test]
fn using_import_retains_generic_and_overload_declaration_identity() {
    execute(
        "main :: () -> int { #import,file \"library.jai\"; return identity(21) + choose(21); }",
        "identity :: (value: $T) -> T { return value; } choose :: (value: int) -> int { return value; } choose :: (value: bool) -> int { return 0; }",
    );
}

#[test]
fn scoped_record_templates_keep_imported_type_and_value_arguments_canonical() {
    execute(
        "main :: () -> int { using Lib :: #import,file \"library.jai\"; first:Box(Lib.Item,Lib.Count); second:Lib.Box(Item,Count)=first; return first.value+second.value; }",
        "Item :: int; Count :: 21; Box :: struct(T:Type,Fill:T) { value:T=Fill; }",
    );
}

#[test]
fn block_import_names_do_not_escape_their_lexical_body() {
    let graph = graph(
        "main :: () -> int { { Lib :: #import,file \"library.jai\"; Lib.answer(); } return Lib.answer(); }",
        "answer :: () -> int { return 42; }",
    );
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("Lib"), "{error}");
    assert_eq!(
        graph.sources().get(error.location.source).unwrap().path(),
        Path::new("/lexical-imports/main.jai")
    );
}
