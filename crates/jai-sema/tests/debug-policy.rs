use jai_ir::{DebugPolicy, Program};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use std::path::Path;

fn resolve(source: &str) -> Program {
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/jai-debug-policy/main.jai");
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
    resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap_or_else(|error| panic!("{}", error.render(graph.sources())))
}

#[test]
fn marked_procedures_keep_source_identity_but_have_no_emitted_locations() {
    let program = resolve(
        "hidden::(arg:int)->int #no_debug { value:=arg+1; defer value+=1; return value; } main::()->int{return hidden(41);}",
    );
    let debug = program.library().debug_sources().unwrap();
    let hidden = debug
        .procedures()
        .find_map(|(id, source)| (source.name == "hidden").then_some(id))
        .unwrap();
    let main = debug
        .procedures()
        .find_map(|(id, source)| (source.name == "main").then_some(id))
        .unwrap();
    assert_eq!(debug.procedure_policy(hidden), DebugPolicy::Suppress);
    assert_eq!(debug.procedure_policy(main), DebugPolicy::Emit);
    assert_eq!(
        debug.procedure(hidden).unwrap().location.path(),
        Path::new("/jai-debug-policy/main.jai")
    );
    assert!(
        debug
            .blocks()
            .all(|(path, _)| path.root.procedure() != hidden)
    );
    assert!(
        debug
            .statements()
            .all(|(path, _)| path.root.procedure() != hidden)
    );
    assert!(debug.locals().all(|(local, _)| local.procedure() != hidden));
    assert!(
        debug
            .statements()
            .any(|(path, _)| path.root.procedure() == main)
    );
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42))
    );
}

#[test]
fn marked_local_and_specialized_procedures_use_the_same_policy() {
    let program = resolve(
        "hidden::(arg:$T)->T #no_debug{return arg;} main::()->int{local::(arg:int)->int #no_debug{return arg+1;} return local(hidden(41));}",
    );
    let debug = program.library().debug_sources().unwrap();
    for name in ["hidden", "local"] {
        let ids: Vec<_> = debug
            .procedures()
            .filter_map(|(id, source)| (source.name == name).then_some(id))
            .collect();
        assert!(!ids.is_empty(), "missing source body for {name}");
        for id in ids {
            assert_eq!(debug.procedure_policy(id), DebugPolicy::Suppress);
            assert!(
                debug
                    .statements()
                    .all(|(path, _)| path.root.procedure() != id)
            );
        }
    }
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==42))
    );
}

#[test]
fn suppressed_debug_information_preserves_caller_location_values() {
    let program = resolve(
        "Source_Code_Location::struct{fully_pathed_filename:string;line_number:s64;character_number:s64;}\nread::(loc:=#caller_location)->int #no_debug{return loc.line_number;}\nmain::()->int #no_debug {\n return read();\n}",
    );
    let debug = program.library().debug_sources().unwrap();
    assert!(debug.statements().next().is_none());
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value()==4))
    );
}
