use jai_syntax::{FieldBinding, FileDeclarationKind, FileItem, TypeSyntax};
fn parse(source: &str) -> jai_syntax::ParsedFile {
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert("inline.jai".into(), source.into());
    jai_syntax::parse_file(
        sources.get(id).unwrap(),
        &mut jai_source::Symbols::default(),
    )
    .unwrap()
}

#[test]
fn anonymous_enum_annotation_preserves_defaults_and_surrounding_terminators() {
    let source = "Options :: struct { output: enum u8 { OMIT; EXECUTABLE; } = .EXECUTABLE; flags: enum_flags u16 #specified { READ :: 1; WRITE :: 2; }; using common: struct { enabled: bool = true; } }";
    let parsed = parse(source);
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    let fields = record.fields().collect::<Vec<_>>();
    let FieldBinding::Explicit {
        ty: TypeSyntax::InlineEnum(enumeration),
        initializer: Some(initializer),
    } = &fields[0].binding
    else {
        panic!()
    };
    assert_eq!(
        enumeration.span.text(source),
        "enum u8 { OMIT; EXECUTABLE; }"
    );
    assert_eq!(initializer.span.text(source), ".EXECUTABLE");
    assert_eq!(enumeration.members.len(), 2);
    let FieldBinding::Explicit {
        ty: TypeSyntax::InlineEnum(enumeration),
        initializer: None,
    } = &fields[1].binding
    else {
        panic!()
    };
    assert_eq!(enumeration.kind, jai_syntax::EnumKind::Flags);
    assert!(enumeration.specified);
    assert_eq!(fields.len(), 3);
}

#[test]
fn anonymous_enums_can_nest_under_arrays_and_pointers() {
    let parsed =
        parse("Owner :: struct { choices: [2] enum u8 { ZERO; ONE; }; ptr: *enum { ZERO; }; }");
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    let FieldBinding::Explicit {
        ty: TypeSyntax::FixedArray {
            element, ..
        },
        ..
    } = &record.fields().next().unwrap().binding
    else {
        panic!()
    };
    assert!(matches!(element.as_ref(), TypeSyntax::InlineEnum(_)));
}
