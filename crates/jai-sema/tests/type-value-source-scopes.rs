//! Pure type queries preserve definition scopes and builtin-name shadows.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    let mut sources = SourceOverlay::new();
    for (path, text) in files {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/type-scopes/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}

fn run(files: &[(&str, &str)]) {
    let program = compile(files).unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(result.outcome, Outcome::Complete(ref values)
        if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{result:?}"
    );
}

#[test]
fn builtin_spelling_cannot_replace_generic_type_or_value_bindings() {
    for source in [
        "measure::($s32:Type)->int{return size_of(s32)+41;} main::()->int{return measure(u8);}",
        "answer::($s32:int)->int{return s32;} main::()->int{return answer(42);}",
        "main::()->int{s32::u8;return size_of(s32)+41;}",
    ] {
        run(&[("/type-scopes/main.jai", source)]);
    }
}

#[test]
fn selected_global_annotation_resolves_in_its_defining_file() {
    run(&[
        (
            "/type-scopes/main.jai",
            "Lib::#import,file \"library.jai\"; Tag::struct{wrong:bool;} make::()->type_of(Lib.item){return .{value=42};} main::()->int{return cast(int)make().value;}",
        ),
        (
            "/type-scopes/library.jai",
            "Tag::struct{value:u8;} item:Tag;",
        ),
    ]);
}

#[test]
fn selected_global_annotation_failure_keeps_its_original_source() {
    let error = compile(&[
        (
            "/type-scopes/main.jai",
            "Lib::#import,file \"library.jai\"; make::()->type_of(Lib.item){return .{};} main::(){}",
        ),
        ("/type-scopes/library.jai", "\nitem:Missing;"),
    ])
    .unwrap_err();
    assert!(error.contains("/type-scopes/library.jai:2:"), "{error}");
    assert!(error.contains("Missing"), "{error}");
}
