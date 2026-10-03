use jai_modules::{Binding, GraphError, GraphOptions, ModuleGraph, ParameterValue, SourceOverlay};
use jai_syntax::NamePath;
use jai_types::{Architecture, BuildTarget, ByteOrder, OperatingSystem};
use std::path::Path;
fn provider(files: &[(&str, &str)]) -> SourceOverlay {
    let mut provider = SourceOverlay::new();
    for (path, text) in files {
        provider
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    provider
}
fn binding(graph: &ModuleGraph, names: &[&str]) -> Binding {
    let file = graph.module(graph.root()).unwrap().entry();
    graph
        .lookup(
            file,
            &NamePath {
                root: graph.symbols().find(names[0]).unwrap(),
                members: names[1..]
                    .iter()
                    .map(|name| graph.symbols().find(name).unwrap())
                    .collect(),
            },
        )
        .unwrap()
}
#[test]
fn modern_private_enum_defaults_select_dependencies_and_keep_nominal_identity() {
    let source = provider(&[
        (
            "/jai-enum/main.jai",
            "A :: #import,file \"renderer.jai\"; B :: #import,file \"renderer.jai\"(Import_Mode=.Main); C :: #import,file \"renderer.jai\"(Import_Mode=.Main);",
        ),
        (
            "/jai-enum/renderer.jai",
            "#module_parameters(Import_Mode := ImportMode.Foreign, Debug := true) { ImportMode :: enum u8 { Foreign; Main; } } #if Import_Mode == .Foreign && Debug { #load \"foreign.jai\"; } else { #load \"main_api.jai\"; }",
        ),
        ("/jai-enum/foreign.jai", "foreign_api :: 1;"),
        ("/jai-enum/main_api.jai", "main_api :: 2;"),
    ]);
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-enum/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    assert!(matches!(
        binding(&graph, &["A", "foreign_api"]),
        Binding::Declaration(_)
    ));
    assert!(matches!(
        binding(&graph, &["B", "main_api"]),
        Binding::Declaration(_)
    ));
    assert_eq!(binding(&graph, &["B"]), binding(&graph, &["C"]));
    let mode = graph.symbols().find("Import_Mode").unwrap();
    let values: Vec<_> = graph
        .parameters()
        .iter()
        .filter(|p| p.name == mode)
        .map(|p| match p.value {
            ParameterValue::Enumeration(value) => value,
            _ => panic!("enum parameter is fully bound"),
        })
        .collect();
    assert_eq!(values.len(), 2);
    assert_ne!(values[0].declaration, values[1].declaration);
    assert_eq!(values[0].value.value(), 0);
    assert_eq!(values[1].value.value(), 1);
    assert!(
        !graph
            .parameters()
            .iter()
            .any(|p| matches!(p.value, ParameterValue::ContextualMember(_)))
    );
}
#[test]
fn explicit_enum_parameters_reject_other_nominals_even_with_equal_bits() {
    let source = provider(&[
        (
            "/jai-enum/main.jai",
            "Other :: enum u8 { Main; } A :: #import,file \"renderer.jai\"(Mode=Other.Main);",
        ),
        (
            "/jai-enum/renderer.jai",
            "#module_parameters(Mode: ModeTag = ModeTag.Main) { ModeTag :: enum u8 { Main; } }",
        ),
    ]);
    let error = ModuleGraph::load_with_provider(
        Path::new("/jai-enum/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap_err();
    assert!(matches!(error, GraphError::Located { .. }));
    assert!(
        error.to_string().contains("different nominal enum type"),
        "{error}"
    );
}
#[test]
fn explicit_target_facts_resolve_source_values_without_host_or_numeric_guesses() {
    let source = provider(&[
        (
            "/jai-target/main.jai",
            "Operating_System_Tag :: enum u32 { MACOS :: 99; LINUX :: 17; } CPU_Tag :: enum u32 { ARM64 :: 31; X64 :: 61; } #if BUILD_OS == .LINUX && CPU == .X64 { #load \"linux.jai\"; } else { #load \"missing.jai\"; }",
        ),
        ("/jai-target/linux.jai", "selected :: 42;"),
    ]);
    let target = BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: jai_types::LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let graph = ModuleGraph::load_with_target(
        Path::new("/jai-target/main.jai"),
        GraphOptions::default(),
        &source,
        target.clone(),
    )
    .unwrap();
    assert_eq!(graph.target(), Some(&target));
    assert!(matches!(
        binding(&graph, &["selected"]),
        Binding::Declaration(_)
    ));
    assert!(
        ModuleGraph::load_with_provider(
            Path::new("/jai-target/main.jai"),
            GraphOptions::default(),
            &source
        )
        .is_err()
    );
}

#[test]
fn target_tags_use_the_bootstrap_file_before_root_publication_and_respect_shadowing() {
    let sources = provider(&[
        (
            "/jai-target/main.jai",
            "OS::3; #if OS == 3 { selected::42; } else { #load \"missing.jai\"; }",
        ),
        (
            "/jai-target/preload.jai",
            "Operating_System_Tag::enum u32 {LINUX::17;WINDOWS::31;} CPU_Tag::enum u32 {X64::61;} #if BUILD_OS == .LINUX && CPU == .X64 {ready::42;} else {#load \"missing.jai\";}",
        ),
    ]);
    let graph = ModuleGraph::load_with_bootstrap(
        Path::new("/jai-target/main.jai"),
        GraphOptions::default(),
        jai_modules::PreludeSource::File("/jai-target/preload.jai".into()),
        &sources,
        Some(BuildTarget {
            operating_system: OperatingSystem::Linux,
            architecture: Architecture::X86_64,
            layout: jai_types::LayoutPolicy::lp64(),
            byte_order: ByteOrder::Little,
        }),
    )
    .unwrap();
    assert!(matches!(
        binding(&graph, &["selected"]),
        Binding::Declaration(_)
    ));
    assert!(matches!(
        binding(&graph, &["ready"]),
        Binding::Declaration(_)
    ));
}

#[test]
fn program_enum_values_rebind_the_same_source_definition_for_each_instance() {
    let source = provider(&[
        (
            "/jai-program-enum/main.jai",
            "A :: #import,file \"renderer.jai\"()(Mode=.Main); B :: #import,file \"renderer.jai\"(Instance=2);",
        ),
        (
            "/jai-program-enum/renderer.jai",
            "#module_parameters(Instance := 1)(Mode := ModeTag.Foreign) { ModeTag :: enum u8 { Foreign; Main; } } #if Mode == .Main { answer :: 42; } else { wrong :: 0; }",
        ),
    ]);
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-program-enum/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    assert!(matches!(
        binding(&graph, &["A", "answer"]),
        Binding::Declaration(_)
    ));
    assert!(matches!(
        binding(&graph, &["B", "answer"]),
        Binding::Declaration(_)
    ));
    let mode = graph.symbols().find("Mode").unwrap();
    let values: Vec<_> = graph
        .parameters()
        .iter()
        .filter(|p| p.name == mode)
        .map(|p| {
            assert!(p.program_wide);
            let ParameterValue::Enumeration(value) = p.value else {
                panic!()
            };
            value
        })
        .collect();
    assert_eq!(values.len(), 2);
    assert_ne!(values[0].declaration, values[1].declaration);
    assert_eq!(values[0].value, values[1].value);
}
#[test]
fn weak_float_arguments_round_at_parameter_width_and_keep_exact_request_identity() {
    let source = provider(&[
        (
            "/jai-weak-params/main.jai",
            "DEC :: 0.1000000000000000000001; A :: #import,file \"wide.jai\"(Amount=DEC); B :: #import,file \"wide.jai\"(Amount=DEC); C :: #import,file \"wide.jai\"(Amount=0.1000000000000000000002);",
        ),
        (
            "/jai-weak-params/wide.jai",
            "#module_parameters(Amount: float64 = 0.0); answer :: 1;",
        ),
    ]);
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-weak-params/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    assert_eq!(binding(&graph, &["A"]), binding(&graph, &["B"]));
    assert_ne!(binding(&graph, &["A"]), binding(&graph, &["C"]));
    let amount = graph.symbols().find("Amount").unwrap();
    for parameter in graph.parameters().iter().filter(|p| p.name == amount) {
        assert_eq!(
            parameter.value,
            ParameterValue::Scalar(jai_eval::Value::Float(jai_types::FloatValue::from_f64(0.1)))
        );
    }
}

#[test]
fn flags_masks_select_dependencies_with_nominal_representation_and_weak_zero() {
    let source = provider(&[
        ("/jai-enum/main.jai", "A::#import,file \"renderer.jai\";"),
        (
            "/jai-enum/renderer.jai",
            "#module_parameters(Mask:=Bits.A|Bits.B){Bits::enum_flags u8{A;B;}} #if Mask & ~.A == .B && 0 != Mask & .A && Mask & .A ^ .B == .A | .B && Mask != ~.A { #load \"selected.jai\"; } else { #load \"missing.jai\"; }",
        ),
        ("/jai-enum/selected.jai", "selected::42;"),
    ]);
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-enum/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    assert!(matches!(
        binding(&graph, &["A", "selected"]),
        Binding::Declaration(_)
    ));
    let ParameterValue::Enumeration(mask) = graph.parameters()[0].value else {
        panic!()
    };
    assert_eq!(mask.value.bits(), 3);
    assert_eq!(mask.value.ty(), jai_types::IntegerType::U8);
}
#[test]
fn flags_zero_exception_rejects_strong_integers_nonzero_and_other_nominals() {
    for expression in [
        "Mask == cast(u8)0",
        "Mask == 1",
        "Mask == Other.A",
        "Mask & Other.A == 0",
        "Mode == 0",
        "Mode | .A == .A",
    ] {
        let source = provider(&[
            ("/jai-enum/main.jai", "A::#import,file \"renderer.jai\";"),
            (
                "/jai-enum/renderer.jai",
                &format!(
                    "#module_parameters(Mask:=Bits.A,Mode:=ModeTag.A){{Bits::enum_flags u8{{A;B;}} Other::enum_flags u8{{A;B;}} ModeTag::enum u8{{A;B;}}}} #if {expression} {{ selected::42; }}"
                ),
            ),
        ]);
        let error = ModuleGraph::load_with_provider(
            Path::new("/jai-enum/main.jai"),
            GraphOptions::default(),
            &source,
        )
        .unwrap_err();
        assert!(
            matches!(error, GraphError::Located { .. }),
            "{expression}: {error}"
        );
    }
}
