//! Generic module headers use source keys across discovery and one final registry.
use jai_driver::{
    CompilerSession, DiscoveryEffectPolicy, EffectReplayCache, SemanticDiscoveryOptions,
    discover_graph_with_session,
};
use jai_modules::{
    BootstrapOptions, GraphOptions, ModuleGraph, ModuleType, ParameterValue, SourceOverlay,
};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn target() -> BuildTarget {
    BuildTarget {
        operating_system: OperatingSystem::Linux,
        architecture: Architecture::X86_64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    }
}
fn discover(main: &str, library: &str) -> Result<ModuleGraph, jai_driver::Error> {
    discover_sources(&sources(main, library))
}
fn sources(main: &str, library: &str) -> SourceOverlay {
    let mut source = SourceOverlay::new();
    for (path, text) in [
        ("/generic-modules/main.jai", main),
        ("/generic-modules/library.jai", library),
    ] {
        source
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    source
}
fn discover_sources(source: &SourceOverlay) -> Result<ModuleGraph, jai_driver::Error> {
    let mut session = CompilerSession::new();
    let mut replay = EffectReplayCache::default();
    discover_graph_with_session(
        Path::new("/generic-modules/main.jai"),
        SemanticDiscoveryOptions {
            graph: GraphOptions::default(),
            bootstrap: BootstrapOptions::disabled(),
            target: target(),
            workspace: session.root(),
            limits: Limits::default(),
            effect_policy: DiscoveryEffectPolicy::Disabled,
        },
        source,
        &mut session,
        &mut replay,
    )
}
fn execute(graph: &ModuleGraph) {
    let program = jai_sema::resolve_graph(graph).unwrap();
    let execution = jai_vm::execute(&program, Limits::default());
    assert!(
        matches!(execution.outcome, Outcome::Complete(ref values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{execution:?}"
    );
}
#[test]
fn generic_type_arguments_normalize_formal_order_and_preserve_instance_identity() {
    let graph = discover(
        "Box::struct(T:Type,N:int=1){value:[N]T;} A::#import,file \"library.jai\"(T=Box(u8,2)); B::#import,file \"library.jai\"(T=Box(N=2,T=u8)); C::#import,file \"library.jai\"(T=Box(u16,2)); main::()->int{value:Box(u8,2); value.value[0]=21; value.value[1]=21; return A.read(value);}",
        "#module_parameters(T:Type=int); read::(value:T)->int{return cast(int)value.value[0]+cast(int)value.value[1];}",
    ).unwrap();
    assert_eq!(
        graph.modules().len(),
        3,
        "named generic argument order normalizes before module request identity"
    );
    assert!(graph.parameters().iter().all(|parameter| matches!(
        parameter.value,
        ParameterValue::Type(ModuleType::Application { .. })
    )));
    execute(&graph);
}
#[test]
fn generic_type_default_keeps_defining_module_template_identity() {
    let graph = discover(
        "Lib::#import,file \"library.jai\"; main::()->int{return Lib.read(Lib.make());}",
        "#module_parameters(T:Type=Box(u8)){Box::struct(U:Type){value:U;}} make::()->T{value:T; value.value=42; return value;} read::(value:T)->int{return cast(int)value.value;}",
    ).unwrap();
    execute(&graph);
}
#[test]
fn inherited_interface_fields_are_checked_after_semantic_member_expansion() {
    let graph = discover(
        "Base::struct{value:int;} Replacement::struct{using base:Base;} Lib::#import,file \"library.jai\"()(R=Replacement); main::()->int{value:Replacement; value.value=42; return Lib.read(value);}",
        "#module_parameters()(R:$I/interface Required=Required){Required::struct{value:int;}} read::(value:$I)->int{return value.value;}",
    ).unwrap();
    execute(&graph);
}
#[test]
fn generic_interface_uses_concrete_field_types_in_the_canonical_registry() {
    let graph = discover(
        "Replacement::struct(T:Type){value:T;} Lib::#import,file \"library.jai\"()(R=Replacement(u8)); main::()->int{value:Replacement(u8); value.value=42; return Lib.read(value);}",
        "#module_parameters()(R:$I/interface Required(u8)=Required(u8)){Required::struct(T:Type){value:T;}} read::(value:$I)->int{return cast(int)value.value;}",
    ).unwrap();
    execute(&graph);
    let error = discover(
        "Replacement::struct(T:Type){value:T;} Lib::#import,file \"library.jai\"()(R=Replacement(u16));",
        "#module_parameters()(R:$I/interface Required(u8)=Required(u8)){Required::struct(T:Type){value:T;}}",
    ).unwrap_err();
    assert!(
        error.to_string().contains("incompatible member 'value'"),
        "{error}"
    );
}

#[test]
fn generic_program_type_defaults_rebind_only_the_same_module_source_template() {
    let graph = discover(
        "A::#import,file \"library.jai\"; B::#import,file \"library.jai\"(Instance=2); main::()->int{return A.read(A.make())+B.read(B.make());}",
        "#module_parameters(Instance:=1)(T:Type=Box(u8)){Box::struct(U:Type){value:U;}} make::()->T{value:T; value.value=21; return value;} read::(value:T)->int{return cast(int)value.value;}",
    ).unwrap();
    let templates: Vec<_> = graph
        .parameters()
        .iter()
        .filter_map(|parameter| match &parameter.value {
            ParameterValue::Type(ModuleType::Application { template, .. })
                if parameter.program_wide =>
            {
                Some(*template)
            }
            _ => None,
        })
        .collect();
    assert_eq!(templates.len(), 2);
    assert_ne!(
        templates[0], templates[1],
        "private module templates keep independent nominal identities"
    );
    execute(&graph);
}

#[test]
fn direct_type_query_argument_is_resolved_by_the_canonical_semantic_query() {
    let graph = discover(
        "Sample::struct{value:u8;} sample:Sample; Lib::#import,file \"library.jai\"(T=type_of(sample.value)); main::()->int{return Lib.read(42);}",
        "#module_parameters(T:Type=int); read::(value:T)->int{return cast(int)value;}",
    ).unwrap();
    assert_eq!(
        graph.parameters()[0].value,
        ParameterValue::Type(ModuleType::Builtin(jai_modules::ModuleBuiltin::Scalar(
            jai_types::ScalarType::Int(jai_types::IntegerType::U8)
        )))
    );
    execute(&graph);
}

#[test]
fn type_queries_keep_private_loaded_definitions_and_reject_annotation_cycles() {
    let mut sources = sources(
        "Other::struct{value:u16;} sample:Other; #load \"types.jai\"; Lib::#import,file \"library.jai\"(T=Alias); main::()->int{return Lib.read(42);}",
        "#module_parameters(T:Type=int); read::(value:T)->int{return cast(int)value;}",
    );
    sources.insert(Path::new("/generic-modules/types.jai"), b"#scope_file; Sample::struct{value:u8;} sample:Sample; #scope_export; Alias::type_of(sample.value);".to_vec()).unwrap();
    let graph = discover_sources(&sources).unwrap();
    assert_eq!(
        graph.parameters()[0].value,
        ParameterValue::Type(ModuleType::Builtin(jai_modules::ModuleBuiltin::Scalar(
            jai_types::ScalarType::Int(jai_types::IntegerType::U8)
        )))
    );
    execute(&graph);
    let error = discover("a:type_of(b); b:type_of(a); Lib::#import,file \"library.jai\"(T=type_of(a)); main::()->int{return 42;}", "#module_parameters(T:Type=int);").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("annotation value type dependency"),
        "{error}"
    );
}

#[test]
fn typed_module_parameters_contextualize_constant_array_extents() {
    let graph = discover("Lib::#import,file \"library.jai\"(T=u8); main::()->int{return Lib.sum();}", "#module_parameters(T:Type=int); count:T:2; Values::[count]int; sum::()->int{v:Values=.[20,22];return v[0]+v[1];}").unwrap();
    execute(&graph);
}

#[test]
fn transparent_module_type_aliases_contextualize_counts_in_the_defining_instance() {
    let graph = discover("count::3; A::#import,file \"library.jai\"(T=u8); B::#import,file \"library.jai\"(T=u16); main::()->int{return A.sum()+B.sum();}", "#module_parameters(T:Type=int); Element::T; count:Element:2; Values::[count]int; sum::()->int{v:Values=.[10,11];return v[0]+v[1];}").unwrap();
    execute(&graph);
}

#[test]
fn nominal_module_restrictions_retain_the_supplied_derived_type() {
    let graph = discover("Base::struct{value:int;} Middle::struct{using base:Base;} Actual::struct{using middle:Middle;extra:int;} Lib::#import,file \"library.jai\"(BaseType=Base,R=Actual); main::()->int{item:Actual;item.value=40;item.extra=2;return Lib.read(item);}", "#module_parameters(BaseType:Type=int,R:$I/BaseType=BaseType); read::(item:$I)->int{return item.value+item.extra;}").unwrap();
    execute(&graph);
    for actual in [
        "Actual::struct{value:int;extra:int;}",
        "Actual::struct{base:Base;extra:int;}",
        "Actual::struct{#as base:Base;extra:int;}",
    ] {
        let error = discover(&format!("Base::struct{{value:int;}} {actual} Lib::#import,file \"library.jai\"(BaseType=Base,R=Actual); main::()->int{{return 42;}}"), "#module_parameters(BaseType:Type=int,R:$I/BaseType=BaseType);").unwrap_err();
        assert!(error.to_string().contains("nominal restriction"), "{error}");
        assert!(error.to_string().contains("library.jai:"), "{error}");
    }
    let error = discover("Base::struct{value:int;} Left::struct{using base:Base;} Right::struct{using base:Base;} Actual::struct{using left:Left;using right:Right;} Lib::#import,file \"library.jai\"(BaseType=Base,R=Actual); main::()->int{return 42;}", "#module_parameters(BaseType:Type=int,R:$I/BaseType=BaseType);").unwrap_err();
    assert!(error.to_string().contains("ambiguous"), "{error}");
}

#[test]
fn nominal_module_restrictions_use_normalized_generic_ancestry() {
    let graph = discover("Base::struct(T:Type){value:T;} Actual::struct(T:Type){using base:Base(T);extra:int;} Lib::#import,file \"library.jai\"(BaseType=Base(u8),R=Actual(u8)); main::()->int{item:Actual(u8);item.value=40;item.extra=2;return Lib.read(item);}", "#module_parameters(BaseType:Type=int,R:$I/BaseType=BaseType); read::(item:$I)->int{return cast(int)item.value+item.extra;}").unwrap();
    execute(&graph);
    let error = discover("Base::struct(T:Type){value:T;} Actual::struct(T:Type){using base:Base(T);extra:int;} Lib::#import,file \"library.jai\"(BaseType=Base(u16),R=Actual(u8)); main::()->int{return 42;}", "#module_parameters(BaseType:Type=int,R:$I/BaseType=BaseType);").unwrap_err();
    assert!(error.to_string().contains("nominal restriction"), "{error}");
}

#[test]
fn nominal_program_defaults_rebind_and_prove_each_defining_instance() {
    let graph = discover("A::#import,file \"library.jai\"(X=1); B::#import,file \"library.jai\"(X=2); main::()->int{return A.read(A.make())+B.read(B.make());}", "#module_parameters(X:int=1)(BaseType:Type=Base,R:$I/BaseType=Child){Base::struct{value:int;} Child::struct{using base:Base;extra:int;}} make::()->$I{item:$I;item.value=20;item.extra=1;return item;} read::(item:$I)->int{return item.value+item.extra;}").unwrap();
    let replacements: Vec<_> = graph
        .parameters()
        .iter()
        .filter(|parameter| graph.symbols().name(parameter.name) == "R")
        .map(|parameter| &parameter.value)
        .collect();
    assert_eq!(replacements.len(), 2);
    assert_ne!(
        replacements[0], replacements[1],
        "each source instance retains its own nominal identity"
    );
    execute(&graph);
}
