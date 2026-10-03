use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    Declaration, ExpressionKind, FileDeclarationKind, FileItem, PlaceKind, TypeSyntax,
};

fn initializer(text: &str) -> (jai_syntax::Expression, Symbols) {
    let mut sources = SourceMap::default();
    let id = sources.insert("aggregate-targets.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Global(global) = &declaration.kind else {
        panic!()
    };
    let Declaration::Inferred { initializer, .. } = &global.declaration else {
        panic!()
    };
    (initializer.clone(), symbols)
}

#[test]
fn generic_targets_and_relative_member_index_paths_retain_source_order() {
    let text = "value := Namespace.Message(Payload,N=2).{method=\"ready\",data._u64=xx tid,color_formats[next()]=.RGBA32UInt};";
    let (expression, symbols) = initializer(text);
    let ExpressionKind::StructLiteral(literal) = &expression.kind else {
        panic!()
    };
    let TypeSyntax::Application(application) = literal.ty.as_ref().unwrap() else {
        panic!()
    };
    let TypeSyntax::Named(path) = application.base.as_ref() else {
        panic!()
    };
    assert_eq!(symbols.name(path.root), "Namespace");
    assert_eq!(symbols.name(path.members[0]), "Message");
    assert_eq!(
        application.span.text(text),
        "Namespace.Message(Payload,N=2)"
    );
    assert_eq!(application.arguments.len(), 2);
    assert_eq!(symbols.name(application.arguments[1].name.unwrap()), "N");
    assert_eq!(literal.fields.len(), 3);
    assert_eq!(literal.fields[1].target.span.text(text), "data._u64");
    assert!(matches!(
        literal.fields[1].target.kind,
        PlaceKind::Qualified(_)
    ));
    assert_eq!(literal.fields[1].span.text(text), "data._u64=xx tid");
    let PlaceKind::Index { index, .. } = &literal.fields[2].target.kind else {
        panic!()
    };
    assert_eq!(index.span.text(text), "next()");
    assert_eq!(literal.fields[2].value.span.text(text), ".RGBA32UInt");
}

#[test]
fn structural_and_positional_targets_preserve_actual_type_syntax() {
    let (expression, _) = initializer("value := ([]arr.T).{count=2,data=pointer};");
    let ExpressionKind::StructLiteral(literal) = expression.kind else {
        panic!()
    };
    assert!(matches!(literal.ty, Some(TypeSyntax::Slice(_))));
    let (expression, _) = initializer("value := Container(T).{1,2};");
    let ExpressionKind::PositionalStructLiteral(literal) = expression.kind else {
        panic!()
    };
    assert!(matches!(literal.ty, Some(TypeSyntax::Application(_))));
    assert_eq!(literal.values.len(), 2);
    let (expression, _) = initializer("value := .{items[0].member=7};");
    let ExpressionKind::StructLiteral(literal) = expression.kind else {
        panic!()
    };
    assert!(literal.ty.is_none());
    assert!(matches!(
        literal.fields[0].target.kind,
        PlaceKind::Member { .. }
    ));
}

#[test]
fn runtime_roots_and_mixed_field_forms_are_located_errors() {
    for text in [
        "value := .{call()=1};",
        "value := .{pointer.*=1};",
        "value := .{call().member=1};",
        "value := .{1,data.member=2};",
        "value := .{data.member=2,1};",
        "value := factory()[0].{member=2};",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad-aggregate-targets.jai".into(), text.into());
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert!(!error.location.span.text(text).is_empty());
    }
}
