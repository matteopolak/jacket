use jai_modules::{
    GraphError, GraphOptions, ModuleBuiltin, ModuleGraph, ModuleType, ParameterValue, SourceOverlay,
};
use std::path::Path;

fn build_graph(application: &str, module: &str) -> Result<ModuleGraph, GraphError> {
    let mut sources = SourceOverlay::new();
    for (path, text) in [
        ("/jai-type-params/main.jai", application),
        ("/jai-type-params/library.jai", module),
    ] {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/jai-type-params/main.jai"),
        GraphOptions::default(),
        &sources,
    )
}
#[test]
fn typed_defaults_and_dependent_values_use_structural_type_identity() {
    let graph = build_graph(
        "Alias::u8; A::#import,file \"library.jai\"(T=Alias,N=255); B::#import,file \"library.jai\"(T=u8,N=255); C::#import,file \"library.jai\"(T=[]u8,N=1);",
        "#module_parameters(T:Type=int, N:int=3);",
    ).unwrap();
    assert_eq!(
        graph.modules().len(),
        3,
        "aliases resolve before request deduplication"
    );
    assert!(
        graph
            .parameters()
            .iter()
            .any(|p| matches!(p.value, ParameterValue::Type(ModuleType::Slice(_))))
    );
    let dependent = build_graph(
        "A::#import,file \"library.jai\"(T=u8,N=255);",
        "#module_parameters(T:Type=int,N:T=3); #if N == 255 { answer::42; }",
    )
    .unwrap();
    assert!(dependent.parameters().iter().any(|p| matches!(&p.value, ParameterValue::Scalar(jai_eval::Value::Int(value)) if value.value()==255 && value.ty()==jai_types::IntegerType::U8)));
    let error = build_graph(
        "A::#import,file \"library.jai\"(T=u8,N=256);",
        "#module_parameters(T:Type=int,N:T=3);",
    )
    .unwrap_err();
    assert!(matches!(error, GraphError::Located { .. }));
}

#[test]
fn typed_constants_use_bound_module_types_and_nominal_enum_annotations() {
    let graph = build_graph("Lib::#import,file \"library.jai\"(T=u8);", "#module_parameters(T:Type=int); count:T:2; Kind::enum{A;B;} chosen:Kind:.B; #if count==2 && chosen==Kind.B {value::42;} else {#load \"missing.jai\";}").unwrap();
    let selected = graph.symbols().find("value").unwrap();
    assert!(matches!(
        graph.modules()[1].exports().get(&selected),
        Some(jai_modules::Binding::Declaration(_))
    ));
    let error = build_graph(
        "Lib::#import,file \"library.jai\"(T=u8);",
        "#module_parameters(T:Type=int); count:T:256; #if count==2 {value::42;}",
    )
    .unwrap_err();
    assert!(error.to_string().contains("out of range"), "{error}");
}

#[test]
fn deferred_float_argument_errors_keep_the_defining_source() {
    let mut sources = SourceOverlay::new();
    for (path, text) in [
        (
            "/jai-type-params/main.jai",
            "#load \"values.jai\"; Alias::Bad+0.0; A::#import,file \"library.jai\"(X=Alias);",
        ),
        (
            "/jai-type-params/values.jai",
            "Bad::ifx 1 / 0 == 0 then 1.0 else 2.0;",
        ),
        (
            "/jai-type-params/library.jai",
            "#module_parameters(X:float64=0.0);",
        ),
    ] {
        sources
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let error = ModuleGraph::load_with_provider(
        Path::new("/jai-type-params/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap_err();
    assert!(error.to_string().contains("values.jai"), "{error}");
    assert!(error.to_string().contains("zero divisor"), "{error}");

    sources
        .insert(
            Path::new("/jai-type-params/values.jai"),
            b"Bad::1e39;".to_vec(),
        )
        .unwrap();
    sources
        .insert(
            Path::new("/jai-type-params/library.jai"),
            b"#module_parameters(X:float32=0.0);".to_vec(),
        )
        .unwrap();
    let error = ModuleGraph::load_with_provider(
        Path::new("/jai-type-params/main.jai"),
        GraphOptions::default(),
        &sources,
    )
    .unwrap_err();
    assert!(error.to_string().contains("library.jai"), "{error}");
    assert!(
        error.to_string().contains("decimal literal overflows F32"),
        "{error}"
    );
}
#[test]
fn nominal_type_requests_keep_distinct_source_declarations() {
    let graph = build_graph("X::struct{} Y::struct{} A::#import,file \"library.jai\"(T=X); B::#import,file \"library.jai\"(T=Y);", "#module_parameters(T:Type=int);").unwrap();
    let types: Vec<_> = graph
        .parameters()
        .iter()
        .filter_map(|p| match &p.value {
            ParameterValue::Type(ty) => Some(ty),
            _ => None,
        })
        .collect();
    assert_ne!(types[0], types[1]);
    assert!(matches!(types[0], ModuleType::Declaration(_)));
}
const INTERFACE: &str = "#module_parameters()(REPLACEMENT_INTERFACE: $I/interface Memory_Debugger_Interface = Memory_Debugger_Interface) { Memory_Debugger_Interface::struct { check_alloc::inline (memory:*void,size:s64) {} check_free::inline (memory:*void) {} } }";
#[test]
fn modern_interface_header_checks_signatures_and_binds_type_variable() {
    let graph = build_graph("Replacement::struct { check_alloc::(memory:*void,size:s64){} check_free::(memory:*void){} extra:int; } A::#import,file \"library.jai\"()(REPLACEMENT_INTERFACE=Replacement);", INTERFACE).unwrap();
    assert!(
        graph.parameters().iter().any(|p| p.program_wide
            && matches!(p.value, ParameterValue::Type(ModuleType::Declaration(_))))
    );
    let module = &graph.modules()[1];
    let symbol = graph.symbols().find("I").unwrap();
    assert!(matches!(
        graph.lookup(
            module.entry(),
            &jai_syntax::NamePath {
                root: symbol,
                members: vec![]
            }
        ),
        Ok(jai_modules::Binding::Parameter(_))
    ));
    for (replacement, message) in [
        (
            "Replacement::struct { check_free::(memory:*void){} }",
            "missing member 'check_alloc'",
        ),
        (
            "Replacement::struct { check_alloc::(memory:*void,size:u8){} check_free::(memory:*void){} }",
            "incompatible member 'check_alloc'",
        ),
    ] {
        let error=build_graph(&format!("{replacement} A::#import,file \"library.jai\"()(REPLACEMENT_INTERFACE=Replacement);"),INTERFACE).unwrap_err();
        assert!(matches!(error, GraphError::Located { .. }));
        assert!(error.to_string().contains(message), "{error}");
    }
}
#[test]
fn structural_default_is_not_an_unresolved_request() {
    let graph = build_graph(
        "A::#import,file \"library.jai\";",
        "#module_parameters(T:Type=[]u8, Alias:Type=$T);",
    )
    .unwrap();
    assert_eq!(graph.parameters()[0].value, graph.parameters()[1].value);
    assert_eq!(
        graph.parameters()[0].value,
        ParameterValue::Type(ModuleType::Slice(Box::new(ModuleType::Builtin(
            ModuleBuiltin::Scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8))
        ))))
    );
}

#[test]
fn program_type_defaults_rebind_local_sentinels_and_compare_type_identity() {
    let graph = build_graph(
        "A::#import,file \"library.jai\"; B::#import,file \"library.jai\"(Instance=2);",
        "#module_parameters(Instance:=1)(Selected:Type=Sentinel){Sentinel::struct{}} #if Selected == Sentinel && int == s64 { selected::42; } else { incorrect::0; }",
    ).unwrap();
    let values: Vec<_> = graph
        .parameters()
        .iter()
        .filter_map(|parameter| {
            if !parameter.program_wide {
                return None;
            }
            let ParameterValue::Type(ModuleType::Declaration(id)) = parameter.value else {
                panic!()
            };
            Some(id)
        })
        .collect();
    assert_eq!(values.len(), 2);
    assert_ne!(
        values[0], values[1],
        "each private sentinel owns a nominal identity"
    );
    let root = graph.module(graph.root()).unwrap().entry();
    for instance in ["A", "B"] {
        let path = jai_syntax::NamePath {
            root: graph.symbols().find(instance).unwrap(),
            members: vec![graph.symbols().find("selected").unwrap()],
        };
        assert!(matches!(
            graph.lookup(root, &path),
            Ok(jai_modules::Binding::Declaration(_))
        ));
    }
}

#[test]
fn record_constructor_arguments_defer_semantics_and_runtime_calls_remain_unsupported() {
    let error = build_graph(
        "Box::struct(T:Type){value:T;} A::#import,file \"library.jai\"(T=Box(u8));",
        "#module_parameters(T:Type=int);",
    )
    .unwrap_err();
    assert!(matches!(error, GraphError::Pending { .. }), "{error}");
    assert!(
        error
            .to_string()
            .contains("semantic nominal specialization"),
        "{error}"
    );
    let error = build_graph(
        "make::()->int{return 1;} A::#import,file \"library.jai\"(T=make());",
        "#module_parameters(T:Type=int);",
    )
    .unwrap_err();
    assert!(matches!(error, GraphError::Unsupported { .. }), "{error}");
}

#[test]
fn procedure_type_keys_normalize_c_variadic_marker_and_preserve_abi() {
    let graph = build_graph(
        "A::#import,file \"library.jai\";",
        "#module_parameters(T:Type=#type (format:*u8, args:..Any)->int #c_call);",
    )
    .unwrap();
    let ParameterValue::Type(ModuleType::Procedure(signature)) = &graph.parameters()[0].value
    else {
        panic!()
    };
    assert_eq!(signature.parameters.len(), 1);
    assert_eq!(signature.convention, jai_types::CallingConvention::C);
    assert_eq!(
        signature.variadic,
        jai_modules::ModuleVariadic::C {
            fixed_parameters: 1
        }
    );
    assert_eq!(signature.results.len(), 1);
}

#[test]
fn supplied_program_types_keep_the_callers_nominal_identity_across_instances() {
    let mut source = SourceOverlay::new();
    for (path, text) in [
        (
            "/jai-type-params/main.jai",
            "#load \"common.jai\"; A::#import,file \"library.jai\"()(Selected=Point); B::#import,file \"library.jai\"(Instance=2);",
        ),
        (
            "/jai-type-params/library.jai",
            "#module_parameters(Instance:=1)(Selected:Type=int); #load \"common.jai\"; #if Selected == Point { incorrect::0; } else { foreign::42; }",
        ),
        ("/jai-type-params/common.jai", "Point::struct{value:int;}"),
    ] {
        source
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let graph = ModuleGraph::load_with_provider(
        Path::new("/jai-type-params/main.jai"),
        GraphOptions::default(),
        &source,
    )
    .unwrap();
    let root = graph.module(graph.root()).unwrap().entry();
    let point = graph.symbols().find("Point").unwrap();
    let jai_modules::Binding::Declaration(caller) = graph
        .lookup(
            root,
            &jai_syntax::NamePath {
                root: point,
                members: vec![],
            },
        )
        .unwrap()
    else {
        panic!()
    };
    let configured: Vec<_> = graph
        .parameters()
        .iter()
        .filter(|p| p.program_wide)
        .collect();
    assert_eq!(configured.len(), 2);
    for parameter in configured {
        assert_eq!(
            parameter.value,
            ParameterValue::Type(ModuleType::Declaration(caller))
        );
    }
    for instance in ["A", "B"] {
        let name = graph.symbols().find(instance).unwrap();
        let local = graph
            .lookup(
                root,
                &jai_syntax::NamePath {
                    root: name,
                    members: vec![point],
                },
            )
            .unwrap();
        assert_ne!(local, jai_modules::Binding::Declaration(caller));
        assert!(
            graph
                .lookup(
                    root,
                    &jai_syntax::NamePath {
                        root: name,
                        members: vec![graph.symbols().find("foreign").unwrap()]
                    }
                )
                .is_ok()
        );
    }
}

#[test]
fn procedure_type_void_result_uses_the_empty_result_identity() {
    let graph = build_graph(
        "A::#import,file \"library.jai\"(T=#type ()->void); B::#import,file \"library.jai\"(T=#type ());",
        "#module_parameters(T:Type=#type ());",
    ).unwrap();
    assert_eq!(
        graph.modules().len(),
        2,
        "void and omitted results share a source type key"
    );
    let ParameterValue::Type(ModuleType::Procedure(signature)) = &graph.parameters()[0].value
    else {
        panic!()
    };
    assert!(signature.results.is_empty());
}
