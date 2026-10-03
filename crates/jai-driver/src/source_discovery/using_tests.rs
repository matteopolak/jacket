use super::*;
use jai_modules::SourceOverlay;
use jai_types::{Architecture, ByteOrder, LayoutPolicy, OperatingSystem, ScalarLayout};

fn target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        byte_order: ByteOrder::Little,
        layout: LayoutPolicy::new(
            ScalarLayout::new(8, 8),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 8),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 8)],
            ScalarLayout::new(1, 1),
        )
        .unwrap(),
    }
}
fn actual_graph(source: &str, library: &str) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    for (path, text) in [
        ("/authored-using/main.jai", source),
        ("/authored-using/library.jai", library),
        ("/authored-using/good.jai", "verified_branch :: 1;"),
    ] {
        overlay
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut session = CompilerSession::new();
    discover_graph_with_session(
        Path::new("/authored-using/main.jai"),
        SemanticDiscoveryOptions {
            graph: GraphOptions::default(),
            bootstrap: BootstrapOptions::disabled(),
            target: target(),
            workspace: session.root(),
            limits: Limits::default(),
            effect_policy: DiscoveryEffectPolicy::CompilerSession,
        },
        &overlay,
        &mut session,
        &mut EffectReplayCache::default(),
    )
    .unwrap()
}
fn execute42(graph: &ModuleGraph) {
    let program = jai_sema::resolve_graph_with_options(
        graph,
        &jai_sema::ResolveOptions {
            target: Some(target()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)),
        "{execution:?}"
    );
}

#[test]
fn actual_driver_discovery_publishes_computed_only_except_and_map_names() {
    for (source, library) in [
        (
            "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"value\"]; } using,only(names()) Lib; main::()->int{return value;}",
            "value::42; omitted::5;",
        ),
        (
            "Lib::#import,file \"library.jai\"; names::()->[]string{return .[\"omitted\"]; } using,except(names()) Lib; main::()->int{return value;}",
            "value::42; omitted::5;",
        ),
        (
            "Lib::#import,file \"library.jai\"; mapper::(names:[]string){names[0]=\"renamed\";} using,map(mapper) Lib; main::()->int{return renamed;}",
            "value::42;",
        ),
    ] {
        execute42(&actual_graph(source, library));
    }
}

#[test]
fn lexical_promoted_name_controls_its_real_import_without_global_fallback() {
    let graph = actual_graph(
        "Lib::#import,file \"library.jai\"; Enabled::false; main::()->int{using Lib; #if Enabled { #import,file \"good.jai\"; } else { #import,file \"missing.jai\"; } return answer;}",
        "Enabled::true; answer::42;",
    );
    assert!(
        graph
            .sources()
            .records()
            .iter()
            .any(|source| source.path().ends_with("good.jai"))
    );
    assert!(
        !graph
            .sources()
            .records()
            .iter()
            .any(|source| source.path().ends_with("missing.jai"))
    );
    execute42(&graph);
}

#[test]
fn mapped_mutable_alias_updates_the_actual_original_place() {
    let graph = actual_graph(
        "Record::struct{original:int;} mapper::(names:[]string){names[0]=\"renamed\";} main::()->int{record:Record; using,map(mapper) record; renamed=42; return record.original;}",
        "",
    );
    execute42(&graph);
}
