//! Default recipes read live typed storage instead of snapshotting mutable state.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn check(source: &str, files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    let path = Path::new("/jai-runtime-defaults/main.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, source.as_bytes().to_vec()).unwrap();
    for (name, source) in files {
        provider
            .insert(
                Path::new(&format!("/jai-runtime-defaults/{name}")),
                source.as_bytes().to_vec(),
            )
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        path,
        GraphOptions {
            import_dirs: vec!["/jai-runtime-defaults".into()],
        },
        &provider,
    )
    .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph(&graph).map_err(|error| error.render(graph.sources()))
}
fn complete(source: &str, files: &[(&str, &str)]) {
    let program = check(source, files).unwrap();
    let result = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    assert!(
        matches!(&result, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42)),
        "{result:?}"
    );
}
#[test]
fn global_aggregate_defaults_keep_live_pointer_and_callable_values_through_aliases() {
    complete(include_str!("fixtures/runtime-default-global.jai"), &[]);
}
#[test]
fn omitted_context_defaults_read_the_active_context_on_every_call() {
    complete(include_str!("fixtures/runtime-default-context.jai"), &[]);
}
#[test]
fn generated_context_overrides_establish_context_before_loading_omitted_defaults() {
    complete(
        include_str!("fixtures/runtime-default-context-override.jai"),
        &[],
    );
}
#[test]
fn imported_default_reads_its_defining_global_despite_caller_shadows() {
    complete(
        "other::#import\"Other\"; value:int=1; main::()->int{other.set(42);value:int=9;return other.read();}",
        &[(
            "Other/module.jai",
            "#scope_export; value:int; set::(next:int){value=next;} read::(next:=value)->int{return next;}",
        )],
    );
}
#[test]
fn source_argument_effects_precede_the_omitted_global_read() {
    complete(
        "value:int; tick::()->int{value=42;return 0;} read::(supplied:int,next:=value)->int{return next+supplied;} main::()->int{return read(tick());}",
        &[],
    );
}
#[test]
fn generic_aggregate_default_is_deferred_after_matching() {
    complete(
        "Allocator::struct{marker:int;} Context::struct{allocator:Allocator;} context:Context; read::(value:$T,allocator:=context.allocator)->int{return allocator.marker;} main::()->int{context.allocator.marker=40;if read(0)!=40 return 1;context.allocator.marker=42;return read(true);}",
        &[],
    );
}
#[test]
fn generic_context_override_uses_compact_runtime_slots_after_baked_formals() {
    complete(
        "#add_context number:int; read::($before:int,prefix:int,$after:int,selected:int=context.number)->int{return prefix+selected;} main::()->int{return read(before=100,prefix=2,after=200,,number=40);}",
        &[],
    );
    complete(
        "#add_context number:int; read::($before:int,prefix:int,$after:int,selected:int=context.number)->int{return prefix+selected;} main::()->int{return read(100,2,200,,number=40);}",
        &[],
    );
}
#[test]
fn context_override_defaults_follow_discarded_formals_and_forwarded_jai_packs() {
    complete(
        "#add_context number:int; read::(#discard ignored:int,prefix:int,values:..int,selected:int=context.number)->int{if values.count!=2 return 1;return prefix+selected;} main::()->int{values:[2]int=.[1,2];return read(100,2,..values,,number=40);}",
        &[],
    );
}
#[test]
fn explicit_context_arguments_keep_caller_context_evaluation() {
    complete(
        "#add_context number:int=40; read::(selected:int=context.number)->int{return selected;} main::()->int{return read(context.number,,number=2)+context.number-38;}",
        &[],
    );
}
#[test]
fn a_bodyless_foreign_prototype_preserves_its_definition_scoped_aggregate_default() {
    let path = Path::new("/jai-runtime-defaults/prototype.jai");
    let mut provider = SourceOverlay::new();
    provider.insert(path, b"Allocator::struct{marker:int;} Context::struct{allocator:Allocator;} context:Context; alloc::(size:s64,allocator:=context.allocator)->*void #foreign;".to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &provider).unwrap();
    let library = jai_sema::resolve_library(&graph)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
    assert_eq!(library.prototypes().len(), 1);
    assert_eq!(
        library
            .types()
            .procedure_definition(library.prototypes()[0].signature)
            .unwrap()
            .parameters
            .len(),
        2
    );
}
#[test]
fn invalid_members_and_different_nominal_defaults_are_source_errors() {
    for source in [
        "Allocator::struct{marker:int;} context:Allocator; read::(value:=context.missing)->int{return value;} main::()->int{return read();}",
        "Left::struct{marker:int;} Right::struct{marker:int;} value:Left; read::(arg:Right=value)->int{return arg.marker;} main::()->int{return read();}",
    ] {
        assert!(check(source, &[]).is_err());
    }
}
