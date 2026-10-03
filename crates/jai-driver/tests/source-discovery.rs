//! Independently authored source exercises genuine semantic dependency decisions.
use jai_driver::{
    CompilerSession, DiscoveryEffectPolicy, EffectReplayCache, ReplayEffects,
    SemanticDiscoveryOptions, discover_graph_with_session,
};
use jai_modules::{BootstrapOptions, GraphOptions, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

const MAIN: &str = "/jai-discovery/main.jai";

fn target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    }
}

#[test]
fn actual_vm_and_typed_string_guards_resume_file_and_lexical_dependencies() {
    for source in [
        "#if #run choose() { #load \"chosen.jai\"; } else { #load \"missing.jai\"; } choose::()->bool{return true;} main::()->int{return ANSWER;}",
        "TEXT::\"Hello\"; #if TEXT==\"Hello\" { #load \"chosen.jai\"; } else { #load \"missing.jai\"; } main::()->int{return ANSWER;}",
        "choose::()->bool{return true;} main::()->int{ Enabled::#run choose(); #if Enabled { flags::#import \"Flags\"; } else { flags::#import \"Missing\"; } return flags.ANSWER; }",
        "choose::()->bool{return true;} main::()->int{ #if #run choose() { flags::#import \"Flags\"; } else { flags::#import \"Missing\"; } later::#import \"Later\"; return flags.ANSWER; }",
        "flags::#import \"WrongFlags\"; choose::()->bool{return true;} main::()->int{ #if #run choose() { flags::#import \"Flags\"; } #if flags.ENABLED { #import \"Later\"; } else { #import \"Missing\"; } return flags.ANSWER; }",
        "ENABLED::false; choose::()->bool{return true;} main::()->int{ #if #run choose() { #import \"Flags\"; } #if ENABLED { #import \"Later\"; } else { #import \"Missing\"; } return ANSWER; }",
        "ENABLED::false; choose::()->bool{return true;} main::()->int{ #if #run choose() { using flags::#import \"Flags\"; } #if ENABLED { #import \"Later\"; } else { #import \"Missing\"; } return ANSWER; }",
    ] {
        let mut overlay = SourceOverlay::new();
        for (path, text) in [
            (MAIN, source),
            ("/jai-discovery/chosen.jai", "ANSWER::42;"),
            (
                "/jai-discovery/modules/Flags/module.jai",
                "ANSWER::42; ENABLED::true;",
            ),
            (
                "/jai-discovery/modules/WrongFlags/module.jai",
                "ENABLED::false;",
            ),
            ("/jai-discovery/modules/Later/module.jai", "OTHER::9;"),
        ] {
            overlay
                .insert(Path::new(path), text.as_bytes().to_vec())
                .unwrap();
        }
        let options = GraphOptions {
            import_dirs: vec!["/jai-discovery/modules".into()],
        };
        let mut session = CompilerSession::new();
        let mut replay = EffectReplayCache::default();
        let graph = discover_graph_with_session(
            Path::new(MAIN),
            SemanticDiscoveryOptions {
                graph: options.clone(),
                bootstrap: BootstrapOptions::disabled(),
                target: target(),
                workspace: session.root(),
                limits: Limits::default(),
                effect_policy: DiscoveryEffectPolicy::CompilerSession,
            },
            &overlay,
            &mut session,
            &mut replay,
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        assert!(!graph.source_condition_selections().is_empty());
        assert!(
            graph
                .sources()
                .records()
                .iter()
                .all(|source| !source.path().to_string_lossy().contains("missing"))
        );
        let options = jai_sema::ResolveOptions {
            target: Some(target()),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &options.import_dirs,
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
        let Outcome::Complete(values) = execution.outcome else {
            panic!("{execution:?}")
        };
        assert!(
            matches!(values.as_slice(), [Value::Int(value)] if value.value()==42),
            "{source}: {values:?}"
        );
    }
}

#[test]
fn runtime_guard_and_failed_run_keep_original_source_diagnostics() {
    for (source, message) in [
        (
            "main::()->int{ enabled:=true; #if enabled { #import \"Flags\"; } return 42; }",
            "requires compile-time values",
        ),
        (
            "#if #run fail() { #load \"chosen.jai\"; } fail::()->bool{ n:=1/0; return true; } main::()->int{return 42;}",
            "ZeroDivisor",
        ),
    ] {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(Path::new(MAIN), source.as_bytes().to_vec())
            .unwrap();
        let mut session = CompilerSession::new();
        let mut replay = EffectReplayCache::default();
        let error = discover_graph_with_session(
            Path::new(MAIN),
            SemanticDiscoveryOptions {
                graph: GraphOptions::default(),
                bootstrap: BootstrapOptions::disabled(),
                target: target(),
                workspace: session.root(),
                limits: Limits::default(),
                effect_policy: DiscoveryEffectPolicy::Disabled,
            },
            &overlay,
            &mut session,
            &mut replay,
        )
        .unwrap_err();
        assert!(error.to_string().contains(message), "{error:?}");
        assert!(error.to_string().contains(MAIN), "{error:?}");
        assert!(replay.is_empty());
    }
}

#[test]
fn scalar_graph_choices_still_validate_all_guard_operands_in_the_source_scope() {
    for (source, message) in [
        (
            "main::()->int{ enabled:=true; #if false&&enabled { #import \"Missing\"; } return 42; }",
            "requires compile-time values",
        ),
        (
            "#if false&&absent() { #load \"missing.jai\"; } main::()->int{return 42;}",
            "absent",
        ),
        (
            "choose::()->bool{return true;} #if false&&choose() { #load \"missing.jai\"; } main::()->int{return 42;}",
            "requires compile-time values",
        ),
    ] {
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(Path::new(MAIN), source.as_bytes().to_vec())
            .unwrap();
        let mut session = CompilerSession::new();
        let mut replay = EffectReplayCache::default();
        let graph = discover_graph_with_session(
            Path::new(MAIN),
            SemanticDiscoveryOptions {
                graph: GraphOptions::default(),
                bootstrap: BootstrapOptions::disabled(),
                target: target(),
                workspace: session.root(),
                limits: Limits::default(),
                effect_policy: DiscoveryEffectPolicy::Disabled,
            },
            &overlay,
            &mut session,
            &mut replay,
        );
        let graph = match graph {
            Ok(graph) => graph,
            Err(error) => {
                assert!(error.to_string().contains(message), "{error:?}");
                assert!(replay.is_empty());
                continue;
            }
        };
        assert!(
            graph
                .source_condition_selections()
                .iter()
                .all(|selection| selection.origin() == jai_modules::SourceConditionOrigin::Scalar)
        );
        let error = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                target: Some(target()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
        assert!(replay.is_empty());
    }
}

#[test]
fn actual_generic_substitutions_keep_import_choices_independent_and_unused_templates_dormant() {
    for source in [
        "choose::($Enabled:bool)->int{ #if Enabled { flags::#import \"First\"; } else { flags::#import \"Second\"; } return flags.ANSWER; } main::()->int{return choose(true)+choose(false);} unused::($Enabled:bool)->int{ #assert false \"unused assertion\"; #if Enabled { #import \"Missing\"; } return unavailable; }",
        "identity::(enabled:bool)->bool{return enabled;} choose::($Enabled:bool)->int{ #if #run identity(Enabled) { flags::#import \"First\"; } else { flags::#import \"Second\"; } return flags.ANSWER; } main::()->int{return choose(true)+choose(false);}",
        "choose::(x:$T)->int{ #if T==u8 { flags::#import \"First\"; } else { flags::#import \"Second\"; } return flags.ANSWER; } main::()->int{return choose(cast(u8)1)+choose(cast(s32)1);}",
        "choose::($Mode:string)->int{ #if Mode==\"first\" { flags::#import \"First\"; } else { flags::#import \"Second\"; } return flags.ANSWER; } main::()->int{return choose(\"first\")+choose(\"second\");}",
        "choose::($value:int)->int{ flags::#import \"Args\"(X=value); return flags.ANSWER; } main::()->int{return choose(20)+choose(22);}",
        "Mode::enum u32 #specified{FIRST::1;SECOND::2;} choose::($value:Mode)->int{ #if value==.FIRST { flags::#import \"First\"; } else { flags::#import \"Second\"; } return flags.ANSWER; } main::()->int{return choose(.FIRST)+choose(.SECOND);}",
    ] {
        let mut overlay = SourceOverlay::new();
        for (path, text) in [
            (MAIN, source),
            ("/jai-discovery/modules/First/module.jai", "ANSWER::20;"),
            ("/jai-discovery/modules/Second/module.jai", "ANSWER::22;"),
            (
                "/jai-discovery/modules/Args/module.jai",
                "#module_parameters(X:int); ANSWER::X;",
            ),
        ] {
            overlay
                .insert(Path::new(path), text.as_bytes().to_vec())
                .unwrap();
        }
        let graph_options = GraphOptions {
            import_dirs: vec!["/jai-discovery/modules".into()],
        };
        let mut session = CompilerSession::new();
        let mut replay = EffectReplayCache::default();
        let graph = discover_graph_with_session(
            Path::new(MAIN),
            SemanticDiscoveryOptions {
                graph: graph_options.clone(),
                bootstrap: BootstrapOptions::disabled(),
                target: target(),
                workspace: session.root(),
                limits: Limits::default(),
                effect_policy: DiscoveryEffectPolicy::Disabled,
            },
            &overlay,
            &mut session,
            &mut replay,
        )
        .unwrap();
        assert_eq!(graph.source_specializations().len(), 2, "{source}");
        assert!(
            graph
                .sources()
                .records()
                .iter()
                .all(|record| !record.path().to_string_lossy().contains("Missing"))
        );
        let options = jai_sema::ResolveOptions {
            target: Some(target()),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &graph_options.import_dirs,
                session.root(),
            )),
            ..Default::default()
        };
        let program =
            jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
        let execution = jai_vm::execute(&program, Limits::default());
        let Outcome::Complete(values) = execution.outcome else {
            panic!("{execution:?}");
        };
        assert!(
            matches!(values.as_slice(), [Value::Int(value)] if value.value()==42),
            "{source}: {values:?}"
        );
    }
}
