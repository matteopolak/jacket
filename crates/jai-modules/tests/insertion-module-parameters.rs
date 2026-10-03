//! Inserted dependency arguments use the quote's genuine captured source facts.
use jai_modules::{
    DeclarationInsertionCode, EnumParameter, GraphDiscovery, GraphOptions, ModuleType,
    ParameterValue, SourceCaptureValue, SourceOverlay,
};
use jai_source::SourceSpan;
use jai_syntax::{
    CodeBody, ExpressionKind, FileDeclaration, FileDeclarationKind, FileItem, StatementKind,
    Visibility,
};
use jai_types::{Integer, IntegerType};
use std::path::Path;

#[test]
fn captured_scalar_type_and_enum_bind_import_parameters_without_destination_fallback() {
    let mut inputs = SourceOverlay::default();
    for (path, source) in [
        (
            "/capture/main.jai",
            "#scope_file; VALUE::99; Kind::enum { A; B; } QUOTE::#code { generated::() { Selected::#import,file \"library.jai\"(N=VALUE,T=TYPE,K=ENUM); } }; #insert QUOTE;",
        ),
        (
            "/capture/library.jai",
            "#module_parameters(N:int,T:Type,K:T);",
        ),
    ] {
        inputs
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = GraphDiscovery::new(
        Path::new("/capture/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    discovery.advance().unwrap();
    let graph = discovery.graph();
    let quote = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "QUOTE")
        .unwrap();
    let enumeration = graph
        .declarations()
        .iter()
        .find(|declaration| graph.symbols().name(declaration.name()) == "Kind")
        .unwrap()
        .id();
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
                location: SourceSpan {
                    source: quote.location().source,
                    span: statement.span,
                },
            })
        })
        .collect();
    let enum_value = EnumParameter {
        declaration: enumeration,
        value: Integer::wrapping(IntegerType::S64, 1),
    };
    let code = DeclarationInsertionCode {
        file: quote.file(),
        source_file: quote.file(),
        location: SourceSpan {
            source: quote.location().source,
            span: constant.initializer.span,
        },
        items,
        bindings: vec![],
        values: vec![
            (
                graph.symbols().find("VALUE").unwrap(),
                SourceCaptureValue::Scalar(jai_eval::Value::Literal(7)),
            ),
            (
                graph.symbols().find("TYPE").unwrap(),
                SourceCaptureValue::Type(ModuleType::Declaration(enumeration)),
            ),
            (
                graph.symbols().find("ENUM").unwrap(),
                SourceCaptureValue::Enumeration(enum_value),
            ),
        ],
        checks: Default::default(),
        debug: Default::default(),
        origins: vec![],
    };
    let request = discovery.pending_insertion_requests().next().unwrap().id;
    let receipt = discovery.prepare_insertion(request, code).unwrap();
    discovery.commit_insertion(receipt).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let parameters = discovery.graph().parameters();
    assert_eq!(parameters.len(), 3);
    assert_eq!(
        parameters[0].value,
        ParameterValue::Scalar(jai_eval::Value::Int(Integer::wrapping(IntegerType::S64, 7)))
    );
    assert_eq!(
        parameters[1].value,
        ParameterValue::Type(ModuleType::Declaration(enumeration))
    );
    assert_eq!(parameters[2].value, ParameterValue::Enumeration(enum_value));
}
