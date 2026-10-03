use jai_source::{SourceMap, Symbols};
use jai_syntax::{FieldBinding, FileDeclarationKind, FileItem, TypeSyntax, parse_file};

#[test]
fn record_self_type_fields_retain_the_original_unresolved_owner_request() {
    let text = "Node::struct{next:*#this; siblings:[]*#this;} Generic::struct(T:Type){next:*#this; value:T;}";
    let mut sources = SourceMap::default();
    let source = sources.insert("self-type-fields.jai".into(), text.into());
    let parsed = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    for item in parsed.items() {
        let FileItem::Declaration(declaration) = item else {
            panic!()
        };
        let FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!()
        };
        let field = record.fields().next().unwrap();
        let FieldBinding::Explicit {
            ty: TypeSyntax::Pointer(pointee),
            ..
        } = &field.binding
        else {
            panic!()
        };
        assert!(matches!(pointee.as_ref(), TypeSyntax::This));
        assert_eq!(field.span.text(text), "next:*#this;");
    }
}
