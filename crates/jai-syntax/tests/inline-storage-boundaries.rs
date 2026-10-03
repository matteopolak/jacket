use jai_source::{SourceMap, Symbols};
use jai_syntax::{Declaration, FileDeclarationKind, FileItem, StatementKind, TypeSyntax};

#[test]
fn inline_record_storage_bodies_terminate_before_the_next_global_or_local_declaration() {
    let text = "settings_info: struct { count:int; Entry::struct { value:int; } }\nplugins:[..]int; main::(){ local:struct { value:int; } next:=42; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("inline-storage.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    assert_eq!(file.items().len(), 3);
    let FileItem::Declaration(settings) = &file.items()[0] else {
        panic!()
    };
    assert_eq!(
        settings.location.span.text(text),
        "settings_info: struct { count:int; Entry::struct { value:int; } }"
    );
    let FileDeclarationKind::Global(settings) = &settings.kind else {
        panic!()
    };
    assert!(matches!(
        &settings.declaration,
        Declaration::UnresolvedExplicit {
            ty: TypeSyntax::InlineRecord(_),
            initializer: None,
            ..
        }
    ));
    let FileItem::Declaration(main) = &file.items()[2] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &main.kind else {
        panic!()
    };
    assert_eq!(main.body.len(), 2);
    assert!(matches!(
        &main.body[0].kind,
        StatementKind::Declare(Declaration::UnresolvedExplicit {
            ty: TypeSyntax::InlineRecord(_),
            initializer: None,
            ..
        })
    ));
    assert_eq!(main.body[0].span.text(text), "local:struct { value:int; }");
    assert_eq!(main.body[1].span.text(text), "next:=42;");
}

#[test]
fn inline_record_initializers_and_ordinary_types_still_require_declaration_terminators() {
    for text in [
        "value:struct { field:int; } = .{} next:int;",
        "value:int next:int;",
        "value:*struct { field:int; } next:int;",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad-inline-storage.jai".into(), text.into());
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.message, "expected ';' after declaration");
        assert_eq!(error.location.span.text(text), "next");
    }
}
