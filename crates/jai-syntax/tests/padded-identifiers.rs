use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    BuiltinType, ExpressionKind, FieldBinding, FileDeclarationKind, FileItem, TypeSyntax,
};

#[test]
fn compiler_report_fields_keep_padding_spans_and_canonical_names() {
    let text = "Report :: struct { time\\        _report: i\\ nt; polymorph\\   _report: int; bytecode\\    _report: int; } hel\\ lo :: 5; same :: hello;";
    let mut sources = SourceMap::default();
    let id = sources.insert("compiler-report-padding.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    let fields: Vec<_> = record.fields().collect();
    assert_eq!(symbols.name(fields[0].name), "time_report");
    assert_eq!(symbols.name(fields[1].name), "polymorph_report");
    assert_eq!(symbols.name(fields[2].name), "bytecode_report");
    assert_eq!(fields[0].span.text(text), "time\\        _report: i\\ nt;");
    assert!(matches!(
        fields[0].binding,
        FieldBinding::Explicit {
            ty: TypeSyntax::Builtin(BuiltinType::Scalar(_)),
            ..
        }
    ));
    let FileItem::Declaration(hello) = &parsed.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Constant(hello) = &hello.kind else {
        panic!()
    };
    let FileItem::Declaration(same) = &parsed.items()[2] else {
        panic!()
    };
    let FileDeclarationKind::Constant(same) = &same.kind else {
        panic!()
    };
    assert!(matches!(same.initializer.kind, ExpressionKind::Name(name) if name == hello.name));
    assert_eq!(symbols.name(hello.name), "hello");
}

#[test]
fn newline_after_a_padding_escape_does_not_merge_declaration_names() {
    let text = "first\\\nsecond :: 5;";
    let mut sources = SourceMap::default();
    let id = sources.insert("invalid-padding.jai".into(), text.into());
    let error =
        jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
    assert_eq!(error.location.span.text(text), "second");
}
