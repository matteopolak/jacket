use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::path::Path;

fn run(source: &str) -> i64 {
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/context-source-binding/main.jai");
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    let program = jai_sema::resolve_graph(&graph)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    match outcome {
        jai_vm::Outcome::Complete(values) => match values.as_slice() {
            [jai_vm::Value::Int(value)] => value.value() as i64,
            _ => panic!("unexpected values {values:?}"),
        },
        outcome => panic!("{outcome:?}"),
    }
}

#[test]
fn global_context_is_ordinary_storage_even_without_implicit_context() {
    assert_eq!(
        run(
            "Context::struct{value:int=40;} context:Context; main::()->int #no_context{context.value+=2;return context.value;}"
        ),
        42
    );
}

#[test]
fn a_local_context_binding_shadows_the_global_and_implicit_records() {
    assert_eq!(
        run(
            "#add_context value:int=1; Context::struct{value:int=40;} context:Context; main::()->int{context:Context;context.value+=2;return context.value;}"
        ),
        42
    );
}

#[test]
fn implicit_context_remains_available_when_no_source_binding_exists() {
    assert_eq!(
        run("#add_context value:int=40; main::()->int{context.value+=2;return context.value;}"),
        42
    );
}

#[test]
fn imported_files_do_not_acquire_the_applications_source_context_binding() {
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/context-source-binding/main.jai");
    overlay.insert(path, b"other::#import \"Other\"; #add_context value:int=40; AppContext::struct{value:int=1;} context:AppContext; main::()->int{return other.read()+context.value+1;}".to_vec()).unwrap();
    overlay
        .insert(
            Path::new("/context-source-binding/Other/module.jai"),
            b"read::()->int{return context.value;}".to_vec(),
        )
        .unwrap();
    let graph = ModuleGraph::load_with_provider(
        path,
        GraphOptions {
            import_dirs: vec!["/context-source-binding".into()],
        },
        &overlay,
    )
    .unwrap();
    let program = jai_sema::resolve_graph(&graph)
        .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())));
    assert_ne!(
        program.library().globals()[0].ty(),
        program.context().unwrap().record_type
    );
    assert!(
        matches!(jai_vm::execute(&program,jai_vm::Limits::default()).outcome,jai_vm::Outcome::Complete(values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42))
    );
}
