use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    Declaration, DeclarationAttribute, ExternalDataSource, FileDeclarationKind, FileItem,
    StatementKind, TypeSyntax,
};

#[test]
fn global_and_local_external_storage_preserve_types_attributes_providers_and_origins() {
    let text = r#"libc::#system_library "c"; State::struct { value:int; }
counter:u32 #align 16 #elsewhere libc "native_\x63ounter";
main::(){ state:State #elsewhere; }"#;
    let mut sources = SourceMap::default();
    let id = sources.insert("external-data.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(global) = &file.items()[2] else {
        panic!()
    };
    let FileDeclarationKind::Global(storage) = &global.kind else {
        panic!()
    };
    let Declaration::External {
        name,
        ty,
        binding,
        attributes,
    } = &storage.declaration
    else {
        panic!()
    };
    assert_eq!(symbols.name(*name), "counter");
    assert!(ty.as_scalar().is_some());
    assert!(matches!(
        attributes.as_slice(),
        [DeclarationAttribute::Alignment(_)]
    ));
    assert_eq!(storage.declaration.attributes().len(), 1);
    let ExternalDataSource::Library(library) = &binding.source else {
        panic!()
    };
    assert_eq!(symbols.name(library.root), "libc");
    assert_eq!(binding.symbol.as_deref(), Some("native_counter"));
    assert_eq!(
        binding.span.text(text),
        r#"#elsewhere libc "native_\x63ounter""#
    );
    assert_eq!(global.location.source, id);
    assert_eq!(
        global.location.span.text(text),
        r#"counter:u32 #align 16 #elsewhere libc "native_\x63ounter";"#
    );
    let FileItem::Declaration(main) = &file.items()[3] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &main.kind else {
        panic!()
    };
    let StatementKind::Declare(Declaration::External {
        name,
        ty,
        binding,
        attributes,
    }) = &main.body[0].kind
    else {
        panic!()
    };
    assert_eq!(symbols.name(*name), "state");
    assert!(matches!(ty, TypeSyntax::Named(_)));
    assert!(matches!(binding.source, ExternalDataSource::Program));
    assert!(binding.symbol.is_none());
    assert!(attributes.is_empty());
    assert_eq!(main.body[0].span.text(text), "state:State #elsewhere;");
}

#[test]
fn external_storage_never_acquires_an_initializer_or_loses_its_terminator() {
    for (text, spelling, message) in [
        (
            "counter:int #elsewhere = 42;",
            "=",
            "external data declaration cannot have an initializer",
        ),
        (
            "counter:int #elsewhere libc #elsewhere;",
            "#elsewhere",
            "duplicate external data binding",
        ),
        (
            "counter:int #elsewhere next:int;",
            ":",
            "expected ';' after declaration",
        ),
        (
            r#"counter:int #elsewhere "alias";"#,
            r#""alias""#,
            "an external symbol alias requires a library",
        ),
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad-external-data.jai".into(), text.into());
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.location.span.text(text), spelling);
        assert_eq!(error.message, message);
    }
    let text = "counter:int #elsewhere; main::(){}";
    let error = jai_syntax::parse(text).unwrap_err();
    assert_eq!(error.span.text(text), "#elsewhere");
    assert_eq!(
        error.message,
        "external data requires checked storage binding"
    );
}
