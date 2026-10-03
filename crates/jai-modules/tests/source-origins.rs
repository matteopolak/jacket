//! Effects distinguish source module environments without using allocation IDs.
use jai_modules::{GraphDiscovery, GraphOptions, ModuleGraph, SourceOriginError, SourceOverlay};
use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
use std::path::Path;

fn inputs(root: &str, library: &str) -> SourceOverlay {
    let mut inputs = SourceOverlay::new();
    for (path, source) in [
        ("/origins/main.jai", root),
        ("/origins/library.jai", library),
    ] {
        inputs
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    inputs
}
fn graph(inputs: &SourceOverlay) -> ModuleGraph {
    ModuleGraph::load_with_provider(
        Path::new("/origins/main.jai"),
        GraphOptions::default(),
        inputs,
    )
    .unwrap()
}
fn imported(graph: &ModuleGraph, index: usize) -> Vec<u8> {
    graph
        .module_environment_origin(
            graph
                .module(graph.imports()[index].module())
                .unwrap()
                .entry(),
        )
        .unwrap()
}

#[test]
fn values_and_ordered_requests_remain_distinct_across_fresh_graphs() {
    let inputs = inputs(
        "A::#import,file \"library.jai\"(X=1,Y=2); B::#import,file \"library.jai\"(Y=2,X=1); C::#import,file \"library.jai\"(X=2,Y=2); D::#import,file \"library.jai\"(X=1,Y=2);",
        "#module_parameters(X:int=1,Y:int=2);",
    );
    let first = graph(&inputs);
    let second = graph(&inputs);
    assert_eq!(imported(&first, 0), imported(&second, 0));
    assert_eq!(imported(&first, 0), imported(&first, 3));
    assert_ne!(imported(&first, 0), imported(&first, 1));
    assert_ne!(imported(&first, 0), imported(&first, 2));
}

#[test]
fn absent_and_explicit_empty_requests_have_independent_origins() {
    let inputs = inputs(
        "A::#import,file \"library.jai\"; B::#import,file \"library.jai\"();",
        "#module_parameters(X:int=1);",
    );
    let graph = graph(&inputs);
    assert_ne!(imported(&graph, 0), imported(&graph, 1));
}

#[test]
fn nominal_parameter_environment_cycles_use_stable_traversal_references() {
    let inputs = inputs(
        "A::#import,file \"library.jai\";",
        "#module_parameters(T:Type=Node); Node::struct{value:int;}",
    );
    let first = graph(&inputs);
    let second = graph(&inputs);
    assert_eq!(imported(&first, 0), imported(&second, 0));
    assert!(imported(&first, 0).len() < 4096);
}

#[test]
fn implicit_target_facts_change_even_an_unparameterized_environment() {
    let inputs = inputs("A::#import,file \"library.jai\";", "Value::1;");
    let load = |operating_system| {
        ModuleGraph::load_with_target(
            Path::new("/origins/main.jai"),
            GraphOptions::default(),
            &inputs,
            BuildTarget {
                operating_system,
                architecture: Architecture::X86_64,
                layout: LayoutPolicy::lp64(),
                byte_order: ByteOrder::Little,
            },
        )
        .unwrap()
    };
    assert_ne!(
        imported(&load(OperatingSystem::Linux), 0),
        imported(&load(OperatingSystem::Windows), 0)
    );
}

#[test]
fn incomplete_parameter_headers_never_publish_an_environment() {
    let inputs = inputs(
        "A::#import,file \"library.jai\";",
        "Box::struct(U:Type){value:U;} #module_parameters(T:Type=Box(u8));",
    );
    let mut discovery = GraphDiscovery::new(
        Path::new("/origins/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let entry = graph.modules()[1].discovered_entry().unwrap();
    assert_eq!(
        graph.module_environment_origin(entry),
        Err(SourceOriginError::IncompleteModule)
    );
}

#[test]
fn program_settings_affect_other_defining_modules() {
    let load = |setting: bool| {
        let mut sources = inputs(
            &format!(
                "Config::#import,file \"settings.jai\"()(DEBUG={setting}); A::#import,file \"library.jai\";"
            ),
            "Value::1;",
        );
        sources
            .insert(
                Path::new("/origins/settings.jai"),
                b"#module_parameters()(DEBUG:bool=false);".to_vec(),
            )
            .unwrap();
        graph(&sources)
    };
    assert_ne!(imported(&load(false), 1), imported(&load(true), 1));
}

#[test]
fn weak_decimal_request_keys_remain_exact_after_equal_float_rounding() {
    let inputs = inputs(
        "A::#import,file \"library.jai\"(F=0.1); B::#import,file \"library.jai\"(F=0.10000000000000000000000001);",
        "#module_parameters(F:float32=0.0);",
    );
    let graph = graph(&inputs);
    let values = graph
        .parameters()
        .iter()
        .map(|parameter| &parameter.value)
        .collect::<Vec<_>>();
    assert_eq!(values[0], values[1]);
    assert_ne!(imported(&graph, 0), imported(&graph, 1));
}

#[test]
fn contextual_enum_requests_encode_only_after_actual_nominal_binding() {
    let inputs = inputs(
        "A::#import,file \"library.jai\"(K=.A); B::#import,file \"library.jai\"(K=.B);",
        "#module_parameters(K:Kind=.A){Kind::enum{A;B;}};",
    );
    let graph = graph(&inputs);
    assert_ne!(imported(&graph, 0), imported(&graph, 1));
}

#[test]
fn unrelated_appended_declarations_preserve_environment_replay_identity() {
    let root = "A::#import,file \"library.jai\";";
    let library = "#module_parameters(T:Type=Node); Node::struct{value:int;}";
    let before = graph(&inputs(root, library));
    let after = graph(&inputs(
        &format!("{root} Unused::19;"),
        &format!("{library} Unused::23;"),
    ));
    assert_eq!(
        before
            .module_environment_origin(before.files()[0].id())
            .unwrap(),
        after
            .module_environment_origin(after.files()[0].id())
            .unwrap()
    );
    assert_eq!(imported(&before, 0), imported(&after, 0));
    // A referenced nominal declaration remains part of the relevant origin.
    let changed = graph(&inputs(
        root,
        "#module_parameters(T:Type=Node); Node::struct{value:u16;}",
    ));
    assert_ne!(imported(&before, 0), imported(&changed, 0));
}

fn insertion_origins(value: jai_modules::SourceCaptureValue, suppress_debug: bool) -> Vec<Vec<u8>> {
    insertion_origins_with_binding(value, suppress_debug, false)
}

fn insertion_origins_with_binding(
    value: jai_modules::SourceCaptureValue,
    suppress_debug: bool,
    capture_expansion: bool,
) -> Vec<Vec<u8>> {
    use jai_modules::{DeclarationInsertionCode, SourceCaptureValue};
    use jai_syntax::{
        CodeBody, ExpressionKind, FileDeclaration, FileDeclarationKind, FileItem, StatementKind,
        Visibility,
    };
    let inputs = inputs(
        "QUOTE::#code{generated::()->int{return CAPTURE;}}; #load \"first.jai\"; #load \"second.jai\";",
        "",
    );
    let mut inputs = inputs;
    for path in ["/origins/first.jai", "/origins/second.jai"] {
        inputs
            .insert(Path::new(path), b"#scope_file; #insert QUOTE;".to_vec())
            .unwrap();
    }
    let mut discovery = GraphDiscovery::new(
        Path::new("/origins/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    let quote = discovery
        .graph()
        .declarations()
        .iter()
        .find(|declaration| discovery.graph().symbols().name(declaration.name()) == "QUOTE")
        .unwrap();
    let FileDeclarationKind::Constant(constant) = &quote.syntax().kind else {
        panic!()
    };
    let ExpressionKind::Code(CodeBody::Block(statements)) = &constant.initializer.kind else {
        panic!()
    };
    let items = statements
        .iter()
        .map(|statement| {
            let StatementKind::Procedure(procedure) = &statement.kind else {
                panic!()
            };
            FileItem::Declaration(FileDeclaration {
                program_export: None,
                visibility: Visibility::File,
                kind: FileDeclarationKind::Procedure(procedure.as_ref().clone()),
                location: jai_source::SourceSpan {
                    source: quote.location().source,
                    span: statement.span,
                },
            })
        })
        .collect();
    let code = DeclarationInsertionCode {
        file: quote.file(),
        source_file: quote.file(),
        location: jai_source::SourceSpan {
            source: quote.location().source,
            span: constant.initializer.span,
        },
        items,
        bindings: vec![],
        values: vec![(discovery.graph().symbols().find("CAPTURE").unwrap(), value)],
        checks: Default::default(),
        debug: if suppress_debug {
            jai_types::DebugPolicy::Suppress
        } else {
            jai_types::DebugPolicy::Emit
        },
        origins: vec![],
    };
    // Captures are source facts; arbitrary byte strings are valid even though
    // this helper's quoted procedure is never semantically executed.
    assert!(matches!(
        code.values[0].1,
        SourceCaptureValue::Scalar(_) | SourceCaptureValue::String(_) | SourceCaptureValue::Type(_)
    ));
    let requests = discovery
        .pending_insertion_requests()
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 2);
    let mut files = vec![];
    let mut generated = None;
    for request in requests {
        let mut code = code.clone();
        if capture_expansion && let Some(declaration) = generated {
            let name = code.values[0].0;
            code.values.clear();
            code.bindings
                .push((name, jai_modules::Binding::Declaration(declaration)));
        }
        let receipt = discovery.prepare_insertion(request.id, code).unwrap();
        let publication = discovery.commit_insertion(receipt).unwrap();
        generated = Some(publication.declarations[0]);
        files.push(publication.file);
    }
    assert!(discovery.advance().unwrap().is_complete());
    files
        .into_iter()
        .map(|file| discovery.graph().module_environment_origin(file).unwrap())
        .collect()
}

#[test]
fn repeated_quote_destinations_have_distinct_stable_environment_receipts() {
    use jai_modules::SourceCaptureValue as C;
    let first = insertion_origins(C::Scalar(jai_eval::Value::Literal(42)), false);
    let rebuilt = insertion_origins(C::Scalar(jai_eval::Value::Literal(42)), false);
    assert_ne!(first[0], first[1]);
    assert_eq!(first, rebuilt);
    let changed = insertion_origins(C::Scalar(jai_eval::Value::Literal(43)), false);
    assert_ne!(first[0], changed[0]);
    let policy = insertion_origins(C::Scalar(jai_eval::Value::Literal(42)), true);
    assert_ne!(first[0], policy[0]);
}

#[test]
fn insertion_environment_receipts_preserve_raw_bytes_and_structural_types() {
    use jai_modules::{ModuleBuiltin, ModuleType, SourceCaptureValue as C};
    let raw = insertion_origins(C::String(vec![0xff, 0, b'x'].into_boxed_slice()), false);
    assert_eq!(
        raw,
        insertion_origins(C::String(vec![0xff, 0, b'x'].into_boxed_slice()), false)
    );
    assert_ne!(
        raw[0],
        insertion_origins(
            C::String(vec![0xef, 0xbf, 0xbd, 0, b'x'].into_boxed_slice()),
            false
        )[0]
    );
    assert_ne!(
        raw[0],
        insertion_origins(C::String(vec![0xff, b'x'].into_boxed_slice()), false)[0]
    );
    let ty = |scalar| {
        C::Type(ModuleType::Pointer(Box::new(ModuleType::Builtin(
            ModuleBuiltin::Scalar(scalar),
        ))))
    };
    assert_ne!(
        insertion_origins(
            ty(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
            false
        )[0],
        insertion_origins(
            ty(jai_types::ScalarType::Int(jai_types::IntegerType::U16)),
            false
        )[0]
    );
}

#[test]
fn captured_inserted_declarations_include_their_own_expansion_environment() {
    use jai_modules::SourceCaptureValue as C;
    let first = insertion_origins_with_binding(C::Scalar(jai_eval::Value::Literal(7)), false, true);
    let rebuilt =
        insertion_origins_with_binding(C::Scalar(jai_eval::Value::Literal(7)), false, true);
    let changed =
        insertion_origins_with_binding(C::Scalar(jai_eval::Value::Literal(8)), false, true);
    assert_eq!(first, rebuilt);
    // The second insertion captures only the first generated declaration.
    // Its source span/body are identical, but that declaration's own capture differs.
    assert_ne!(first[1], changed[1]);
}

#[test]
fn captured_weak_decimal_receipts_preserve_exact_unrounded_values() {
    fn weak(spelling: &str) -> jai_eval::Value {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert("/weak.jai".into(), format!("W::{spelling};"));
        let parsed = jai_syntax::parse_file(
            sources.get(source).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let jai_syntax::FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let jai_syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!()
        };
        jai_eval::evaluate(&constant.initializer, |_, span| {
            Err(jai_source::Diagnostic::new(span, "unexpected lookup"))
        })
        .unwrap()
    }
    use jai_modules::SourceCaptureValue as C;
    let first = weak("0.1");
    let second = weak("0.10000000000000000000000001");
    let jai_eval::Value::WeakFloat(a) = &first else {
        panic!()
    };
    let jai_eval::Value::WeakFloat(b) = &second else {
        panic!()
    };
    assert_eq!(
        a.round(jai_types::FloatType::F32, jai_source::Span::default())
            .unwrap(),
        b.round(jai_types::FloatType::F32, jai_source::Span::default())
            .unwrap()
    );
    assert_ne!(
        insertion_origins(C::Scalar(first), false)[0],
        insertion_origins(C::Scalar(second), false)[0]
    );
}
