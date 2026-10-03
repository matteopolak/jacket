use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    Declaration, ExpressionKind, FileDeclarationKind, FileItem, StatementKind, TypeSyntax,
};

#[test]
fn anonymous_type_initializers_preserve_bodies_and_declaration_boundaries() {
    let text = "record_type := struct { x:int; y:int; }\nunion_type := union { a:int; b:int; };\nenum_type := enum { A; B; }\nflags_type := enum_flags u32 { A; B; };\nmain::(){ local := struct { next:*#this; } after:=42; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("anonymous-type-values.jai".into(), text.into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    assert_eq!(file.items().len(), 5);
    for (index, is_record) in [true, true, false, false].into_iter().enumerate() {
        let FileItem::Declaration(item) = &file.items()[index] else {
            panic!()
        };
        let FileDeclarationKind::Global(global) = &item.kind else {
            panic!()
        };
        let Declaration::Inferred {
            initializer, ..
        } = &global.declaration
        else {
            panic!()
        };
        assert!(matches!(
            (&initializer.kind, is_record),
            (ExpressionKind::Type(TypeSyntax::InlineRecord(_)), true)
                | (ExpressionKind::Type(TypeSyntax::InlineEnum(_)), false)
        ));
    }
    let FileItem::Declaration(item) = &file.items()[4] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &item.kind else {
        panic!()
    };
    assert_eq!(main.body.len(), 2);
    assert!(matches!(
        &main.body[0].kind,
        StatementKind::Declare(Declaration::Inferred {
            initializer: jai_syntax::Expression {
                kind: ExpressionKind::Type(TypeSyntax::InlineRecord(_)),
                ..
            },
            ..
        })
    ));
    assert_eq!(
        main.body[0].span.text(text),
        "local := struct { next:*#this; }"
    );
    assert_eq!(main.body[1].span.text(text), "after:=42;");
}

#[test]
fn only_a_direct_anonymous_type_body_terminates_an_initializer() {
    for text in [
        "value := #type int next:int;",
        "value := *struct { x:int; } next:int;",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("type-value-boundary.jai".into(), text.into());
        assert!(jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
    }
    let error = jai_syntax::parse("main::(){value:=struct{x:int;};}").unwrap_err();
    assert_eq!(error.message, "type expressions require type resolution");
}
