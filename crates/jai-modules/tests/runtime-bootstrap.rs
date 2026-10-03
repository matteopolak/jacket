use jai_modules::{
    Binding, BootstrapOptions, GraphOptions, LookupError, ModuleGraph, ParameterValue,
    PreludeSource, RuntimeSupportOptions, RuntimeSupportParameters, RuntimeSupportSource,
    SourceOverlay,
};
use jai_syntax::NamePath;
use std::path::Path;

const RUNTIME: &str = "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool); Context_Base :: struct { value: int; } runtime_helper :: 42; #if preload_helper == 3 { from_preload :: true; } #if DEFINE_SYSTEM_ENTRY_POINT { chosen_entry :: true; } else { chosen_entry :: false; } #if DEFINE_INITIALIZATION { chosen_init :: true; } else { chosen_init :: false; } #if ENABLE_BACKTRACE_ON_CRASH { chosen_backtrace :: true; } else { chosen_backtrace :: false; } #scope_module; runtime_private :: 9;";

fn load(entry: &str, parameters: RuntimeSupportParameters) -> ModuleGraph {
    load_with_main(entry, parameters, "A :: #import \"A\"; main :: () {}")
}

fn load_with_main(entry: &str, parameters: RuntimeSupportParameters, main: &str) -> ModuleGraph {
    let mut provider = SourceOverlay::new();
    for (path, source) in [
        ("/jai-runtime/main.jai", main),
        ("/jai-runtime/modules/A.jai", "value :: runtime_helper;"),
        ("/jai-runtime/modules/module.jai", "preload_helper :: 3;"),
        ("/jai-runtime/modules/Runtime_Support.jai", RUNTIME),
    ] {
        provider
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_bootstrap_options(
        Path::new(entry),
        GraphOptions {
            import_dirs: vec!["/jai-runtime/modules".into()],
        },
        BootstrapOptions {
            prelude: PreludeSource::Search,
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters,
            }),
        },
        &provider,
        None,
    )
    .unwrap()
}

#[test]
fn explicit_runtime_import_reuses_only_the_same_ordered_argument_request() {
    let graph = load_with_main(
        "/jai-runtime/main.jai",
        RuntimeSupportParameters {
            define_system_entry_point: true,
            define_initialization: true,
            enable_backtrace_on_crash: false,
        },
        "Same :: #import \"Runtime_Support\"(DEFINE_SYSTEM_ENTRY_POINT=true, DEFINE_INITIALIZATION=true, ENABLE_BACKTRACE_ON_CRASH=false); Different :: #import \"Runtime_Support\"(ENABLE_BACKTRACE_ON_CRASH=false, DEFINE_INITIALIZATION=true, DEFINE_SYSTEM_ENTRY_POINT=true); main :: () {}",
    );
    let file = graph.module(graph.root()).unwrap().entry();
    assert_eq!(
        lookup(&graph, file, "Same", &[]),
        Ok(Binding::Module(graph.runtime_support().unwrap()))
    );
    let Binding::Module(different) = lookup(&graph, file, "Different", &[]).unwrap() else {
        panic!("expected distinct explicit runtime instance");
    };
    assert_ne!(Some(different), graph.runtime_support());
    // Source text is parsed once even when request order creates a second instance.
    assert_eq!(graph.sources().records().len(), 3);
    assert_ne!(
        lookup(&graph, file, "Same", &["Context_Base"]),
        lookup(&graph, file, "Different", &["Context_Base"])
    );
}
fn lookup(
    graph: &ModuleGraph,
    file: jai_modules::FileInstanceId,
    root: &str,
    members: &[&str],
) -> Result<Binding, LookupError> {
    graph.lookup(
        file,
        &NamePath {
            root: graph.symbols().find(root).unwrap(),
            members: members
                .iter()
                .map(|name| graph.symbols().find(name).unwrap())
                .collect(),
        },
    )
}

#[test]
fn actual_parameter_names_bind_explicit_typed_build_policy_and_preserve_module_identity() {
    let parameters = RuntimeSupportParameters {
        define_system_entry_point: true,
        define_initialization: true,
        enable_backtrace_on_crash: false,
    };
    let graph = load("/jai-runtime/main.jai", parameters);
    let runtime = graph.runtime_support().unwrap();
    assert_ne!(Some(runtime), graph.prelude());
    assert_ne!(runtime, graph.root());
    let supplied = graph
        .parameters()
        .iter()
        .filter(|parameter| parameter.module == runtime)
        .map(|parameter| (graph.symbols().name(parameter.name), &parameter.value))
        .collect::<Vec<_>>();
    assert_eq!(supplied.len(), 3);
    for (name, expected) in [
        ("DEFINE_SYSTEM_ENTRY_POINT", true),
        ("DEFINE_INITIALIZATION", true),
        ("ENABLE_BACKTRACE_ON_CRASH", false),
    ] {
        assert!(supplied.iter().any(|(actual, value)| *actual == name
            && **value == ParameterValue::Scalar(jai_eval::Value::Bool(expected))));
    }
    let root = graph.module(graph.root()).unwrap().entry();
    let canonical = lookup(&graph, root, "runtime_helper", &[]).unwrap();
    let Binding::Module(module) = lookup(&graph, root, "A", &[]).unwrap() else {
        panic!("expected module");
    };
    let file = graph.module(module).unwrap().entry();
    assert_eq!(lookup(&graph, file, "runtime_helper", &[]), Ok(canonical));
    assert!(matches!(
        lookup(&graph, file, "runtime_private", &[]),
        Err(LookupError::UnknownName(_))
    ));
    assert!(matches!(
        lookup(&graph, root, "A", &["runtime_helper"]),
        Err(LookupError::UnknownMember { .. })
    ));
}

#[test]
fn checking_runtime_support_itself_does_not_duplicate_source_declarations() {
    let graph = load(
        "/jai-runtime/modules/Runtime_Support.jai",
        RuntimeSupportParameters {
            define_system_entry_point: false,
            define_initialization: true,
            enable_backtrace_on_crash: false,
        },
    );
    assert_eq!(graph.runtime_support(), Some(graph.root()));
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.sources().records().len(), 2);
    assert_eq!(graph.parameters().len(), 3);
}

#[test]
fn checking_preload_itself_publishes_its_scope_before_loading_runtime_support() {
    let graph = load(
        "/jai-runtime/modules/module.jai",
        RuntimeSupportParameters {
            define_system_entry_point: false,
            define_initialization: true,
            enable_backtrace_on_crash: false,
        },
    );
    assert_eq!(graph.prelude(), Some(graph.root()));
    assert_eq!(graph.modules().len(), 2);
    assert_eq!(graph.sources().records().len(), 2);
    let runtime = graph.module(graph.runtime_support().unwrap()).unwrap();
    assert!(lookup(&graph, runtime.entry(), "from_preload", &[]).is_ok());
}

#[test]
fn runtime_support_requires_preload_instead_of_inventing_its_types() {
    let mut provider = SourceOverlay::new();
    provider
        .insert(
            Path::new("/jai-runtime/main.jai"),
            b"main :: () {}".to_vec(),
        )
        .unwrap();
    let error = ModuleGraph::load_with_bootstrap_options(
        Path::new("/jai-runtime/main.jai"),
        GraphOptions::default(),
        BootstrapOptions {
            prelude: PreludeSource::Disabled,
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: false,
                    enable_backtrace_on_crash: false,
                },
            }),
        },
        &provider,
        None,
    )
    .unwrap_err();
    assert!(error.to_string().contains("requires actual Preload source"));
}

#[test]
fn runtime_target_arguments_use_preload_before_the_application_file_is_published() {
    let mut provider = SourceOverlay::new();
    for (path, source) in [
        ("/jai-bootstrap-target/main.jai", "main :: () {}"),
        (
            "/jai-bootstrap-target/modules/module.jai",
            "Operating_System_Tag :: enum u32 { LINUX :: 17; MACOS :: 99; } CPU_Tag :: enum u32 { X64 :: 61; ARM64 :: 31; }",
        ),
        (
            "/jai-bootstrap-target/modules/Runtime_Support.jai",
            "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT: bool, DEFINE_INITIALIZATION: bool, ENABLE_BACKTRACE_ON_CRASH: bool); Selected :: #import \"Selected\"(Selected_OS=OS, Selected_CPU=CPU);",
        ),
        (
            "/jai-bootstrap-target/modules/Selected.jai",
            "#module_parameters(Selected_OS: Operating_System_Tag, Selected_CPU: CPU_Tag); #if Selected_OS == .LINUX && Selected_CPU == .X64 { value :: 42; } else { #load \"missing.jai\"; }",
        ),
    ] {
        provider
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_bootstrap_options(
        Path::new("/jai-bootstrap-target/main.jai"),
        GraphOptions {
            import_dirs: vec!["/jai-bootstrap-target/modules".into()],
        },
        BootstrapOptions {
            prelude: PreludeSource::Search,
            runtime_support: Some(RuntimeSupportOptions {
                source: RuntimeSupportSource::Search,
                parameters: RuntimeSupportParameters {
                    define_system_entry_point: false,
                    define_initialization: true,
                    enable_backtrace_on_crash: false,
                },
            }),
        },
        &provider,
        Some(jai_types::BuildTarget {
            operating_system: jai_types::OperatingSystem::Linux,
            architecture: jai_types::Architecture::X86_64,
            layout: jai_types::LayoutPolicy::lp64(),
            byte_order: jai_types::ByteOrder::Little,
        }),
    )
    .unwrap();
    let preload = graph.module(graph.prelude().unwrap()).unwrap().entry();
    for (name, enum_name, expected) in [
        ("Selected_OS", "Operating_System_Tag", 17),
        ("Selected_CPU", "CPU_Tag", 61),
    ] {
        let Binding::Declaration(declaration) = lookup(&graph, preload, enum_name, &[]).unwrap()
        else {
            panic!("source tag must retain its enum declaration");
        };
        let value = graph
            .parameters()
            .iter()
            .find(|parameter| graph.symbols().name(parameter.name) == name)
            .unwrap();
        let ParameterValue::Enumeration(value) = &value.value else {
            panic!("target argument must retain source enum identity");
        };
        assert_eq!(value.declaration, declaration);
        assert_eq!(
            value.value,
            jai_types::Integer::checked(jai_types::IntegerType::U32, expected).unwrap()
        );
    }
}
