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
    let Declaration::Inferred {
        initializer, ..
    } = &global.declaration
    else {
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
    let PlaceKind::Index {
        index, ..
    } = &literal.fields[2].target.kind
    else {
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

#[test]
fn bare_braces_are_contextual_values_and_statement_braces_remain_blocks() {
    let (expression, _) = initializer("value := {x=20, y=22};");
    assert!(matches!(
        expression.kind,
        ExpressionKind::StructLiteral(jai_syntax::StructLiteral {
            ty: None,
            ..
        })
    ));
    let mut sources = SourceMap::default();
    let id = sources.insert("blocks.jai".into(), "main::(){ { value:=42; } }".into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!();
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!();
    };
    assert!(matches!(
        procedure.body[0].kind,
        jai_syntax::StatementKind::Block(_)
    ));
}

#[test]
fn selected_import_and_nested_record_using_retain_original_source_owners() {
    let text = "using,except(hidden) API::#import \"dependency\"; Owner::struct { using,only(x) child:Child; using Values::enum { A::1; B::2; } using,except(y) child; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("using.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let [
        FileItem::Import(import),
        FileItem::Using {
            directive, ..
        },
        FileItem::Declaration(declaration),
    ] = file.items()
    else {
        panic!("namespace and using request are separate real owners");
    };
    assert!(!import.using);
    assert!(matches!(
        directive.selection,
        jai_syntax::UsingSelection::Except(_)
    ));
    assert_eq!(
        directive.span.text(text),
        "using,except(hidden) API::#import \"dependency\";"
    );
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!();
    };
    let [
        jai_syntax::RecordMember::Field(field),
        jai_syntax::RecordMember::Enum(enumeration),
        jai_syntax::RecordMember::Using(declared),
        jai_syntax::RecordMember::Using(bare),
    ] = record.members.as_slice()
    else {
        panic!("original ordered children retained");
    };
    assert!(field.using);
    assert!(matches!(
        field.using_selection,
        jai_syntax::UsingSelection::Only(_)
    ));
    assert!(matches!(declared.target.kind,ExpressionKind::Name(name) if name == enumeration.name));
    assert_eq!(bare.target.span.text(text), "child");
    assert!(matches!(
        bare.selection,
        jai_syntax::UsingSelection::Except(_)
    ));
}

#[test]
fn physical_using_fields_preserve_conversion_overlay_and_selection_qualifiers() {
    let text = "Owner::struct{using #as info:Base;#as using other:Base;using,only(x) #as selected:Base;using #overlay(info) alias:Base;using,except(y) namespace;}";
    let mut sources = SourceMap::default();
    let id = sources.insert("field-prefix-composition.jai".into(), text.into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(source) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &source.kind else {
        panic!()
    };
    let fields: Vec<_> = record.fields().collect();
    assert_eq!(fields.len(), 4);
    for field in &fields[..3] {
        assert!(field.using);
        assert_eq!(field.conversion, jai_syntax::FieldConversion::Implicit);
    }
    assert!(matches!(
        fields[2].using_selection,
        jai_syntax::UsingSelection::Only(_)
    ));
    assert!(
        fields[3]
            .attributes
            .iter()
            .any(|attribute| matches!(attribute, jai_syntax::FieldAttribute::Placement(_)))
    );
    assert!(
        matches!(&record.members[4],jai_syntax::RecordMember::Using(directive)
        if matches!(directive.selection,jai_syntax::UsingSelection::Except(_)))
    );
    assert_eq!(fields[0].span.text(text), "info:Base;");
}
#[test]
fn duplicate_conversion_qualifiers_are_rejected_by_the_real_field_parser() {
    for text in [
        "Owner::struct{using #as #as info:Base;}",
        "Owner::struct{using,only(x) #as #as info:Base;}",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad-field-prefix.jai".into(), text.into());
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert!(error.message.contains("duplicate #as"), "{error:?}");
    }
}
