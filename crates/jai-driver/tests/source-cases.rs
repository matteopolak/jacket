//! Independently authored cases exercise the real typed discovery scheduler.
use jai_driver::{
    CompilerSession, DiscoveryEffectPolicy, EffectReplayCache, ReplayEffects,
    SemanticDiscoveryOptions, discover_graph_with_session,
};
use jai_modules::{BootstrapOptions, GraphOptions, SourceConditionOrigin, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn execute(source: &str) -> jai_modules::ModuleGraph {
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let mut overlay = SourceOverlay::new();
    for (path, text) in [
        ("/own-driver-cases/main.jai", source),
        ("/own-driver-cases/chosen.jai", "ANSWER::42;"),
        (
            "/own-driver-cases/modules/First/module.jai",
            "ANSWER::19;ENABLED::true;",
        ),
        ("/own-driver-cases/modules/Second/module.jai", "ANSWER::23;"),
        (
            "/own-driver-cases/modules/Shadow/module.jai",
            "ENABLED::false;",
        ),
        ("/own-driver-cases/modules/Later/module.jai", "ANSWER::42;"),
    ] {
        overlay
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let graph_options = GraphOptions {
        import_dirs: vec!["/own-driver-cases/modules".into()],
    };
    let mut session = CompilerSession::new();
    let mut replay = EffectReplayCache::default();
    let graph = discover_graph_with_session(
        Path::new("/own-driver-cases/main.jai"),
        SemanticDiscoveryOptions {
            graph: graph_options.clone(),
            bootstrap: BootstrapOptions::disabled(),
            target: target.clone(),
            workspace: session.root(),
            limits: Limits::default(),
            effect_policy: DiscoveryEffectPolicy::CompilerSession,
        },
        &overlay,
        &mut session,
        &mut replay,
    )
    .unwrap_or_else(|error| panic!("{source}: {error:?}"));
    assert!(!graph.source_cases().is_empty());
    assert!(
        graph
            .sources()
            .records()
            .iter()
            .all(|record| !record.path().to_string_lossy().contains("Missing"))
    );
    let options = jai_sema::ResolveOptions {
        target: Some(target),
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &graph_options.import_dirs,
            session.root(),
        )),
        ..Default::default()
    };
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &options,
        &mut ReplayEffects::new(&mut session, &mut replay),
    )
    .unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{source}: {execution:?}"
    );
    graph
}

#[test]
fn file_and_lexical_cases_use_real_vm_selectors_and_active_dependencies() {
    for source in [
        "Operating_System_Tag::enum{WINDOWS;MACOS;LINUX;}#if OS == {case .WINDOWS;#load \"Missing.jai\";case .MACOS;#load \"Missing.jai\";case .LINUX;#load \"chosen.jai\";}main::()->int{return ANSWER;}",
        "choose::()->int{return 2;} #if #run choose() == {case 1;#load \"Missing.jai\";case 2;#load \"chosen.jai\";}main::()->int{return ANSWER;}",
        "Tag::enum{OTHER;CHOSEN;} selected::Tag.CHOSEN; #if selected == {case .OTHER;#load \"Missing.jai\";case .CHOSEN;#load \"chosen.jai\";}main::()->int{return ANSWER;}",
        "flags::#import \"Shadow\";choose::()->int{return 1;}main::()->int{#if #run choose() == {case 1;flags::#import \"First\";case;#import \"Missing\";}#if flags.ENABLED{answer::#import \"Later\";}else{#import \"Missing\";}return answer.ANSWER;}",
    ] {
        let graph = execute(source);
        assert!(
            graph
                .source_cases()
                .iter()
                .any(|case| case.origin == SourceConditionOrigin::Semantic)
        );
    }
}

#[test]
fn one_original_case_table_selects_independent_source_specializations() {
    let graph = execute(
        "choose::($N:int)->int{#if N == {case 1;flags::#import \"First\";case 2;flags::#import \"Second\";case;#import \"Missing\";}return flags.ANSWER;}main::()->int{return choose(1)+choose(2);}",
    );
    assert_eq!(graph.source_specializations().len(), 2);
    let specialized = graph
        .source_cases()
        .iter()
        .filter(|case| case.specialization.is_some())
        .collect::<Vec<_>>();
    assert_eq!(specialized.len(), 2);
    assert_ne!(specialized[0].specialization, specialized[1].specialization);
    assert_ne!(specialized[0].choice, specialized[1].choice);
}
