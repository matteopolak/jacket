use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, StatementKind};

#[test]
fn named_expansion_and_exported_declarations_retain_real_source_ranges() {
    let text = "walk :: (source: Container, body: Code, flags: For_Flags) #expand { `it := source.value; `it_index := 0; #insert body; } main :: () { for < * :walk value,index: source { consume(value,index); } }";
    let mut sources = SourceMap::default();
    let id = sources.insert("custom-iteration.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!("macro declaration");
    };
    let FileDeclarationKind::Procedure(macro_) = &declaration.kind else {
        panic!("macro procedure");
    };
    assert_eq!(macro_.body[0].span.text(text), "`it := source.value;");
    let StatementKind::CallerExport(declaration) = &macro_.body[0].kind else {
        panic!("export");
    };
    assert_eq!(declaration.span.text(text), "it := source.value;");
    let FileItem::Declaration(declaration) = &parsed.items()[1] else {
        panic!("main");
    };
    let FileDeclarationKind::Procedure(main) = &declaration.kind else {
        panic!("main procedure");
    };
    let StatementKind::ArrayLoop(loop_) = &main.body[0].kind else {
        panic!("custom loop");
    };
    assert_eq!(symbols.name(loop_.expansion.as_ref().unwrap().root), "walk");
    assert_eq!(symbols.name(loop_.iterator), "value");
    assert_eq!(symbols.name(loop_.index.unwrap()), "index");
    assert_eq!(loop_.direction, jai_syntax::Direction::Reverse);
    assert!(loop_.by_pointer);
}

#[test]
fn exports_reject_non_declaration_statements_at_their_source_range() {
    let text = "main :: () { `consume(); }";
    let mut sources = SourceMap::default();
    let id = sources.insert("export.jai".into(), text.into());
    let error =
        jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
    assert_eq!(
        error.message,
        "a caller export requires a declaration, defer, or return"
    );
    assert_eq!(error.location.span.text(text), "consume();");
}

#[test]
fn direct_iterator_exports_and_constant_modifiers_are_retained() {
    let text = "walk :: (source: Container, body: Code, flags: For_Flags) #expand { for *=(false) <=(true) `it, `it_index: source.values { #insert body; } }";
    let mut sources = SourceMap::default();
    let id = sources.insert("direct-exports.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!("macro");
    };
    let FileDeclarationKind::Procedure(macro_) = &declaration.kind else {
        panic!("procedure");
    };
    let StatementKind::ArrayLoop(loop_) = &macro_.body[0].kind else {
        panic!("array loop");
    };
    assert!(loop_.iterator_export);
    assert!(loop_.index_export);
    assert_eq!(
        loop_.pointer_control.as_ref().unwrap().span.text(text),
        "(false)"
    );
    assert_eq!(
        loop_.reverse_control.as_ref().unwrap().span.text(text),
        "(true)"
    );
}

#[test]
fn exported_while_binding_retains_the_marked_name_and_real_initializer() {
    let text = "walk::(body:Code)#expand{while `table_while_loop:=remaining {#insert body;} while ordinary:=remaining {}}";
    let mut sources = SourceMap::default();
    let id = sources.insert("exported-while.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!("macro");
    };
    let FileDeclarationKind::Procedure(macro_) = &declaration.kind else {
        panic!("procedure");
    };
    let StatementKind::While(
        jai_syntax::WhileCondition::Binding {
            name,
            export_span,
            initializer,
        },
        _,
    ) = &macro_.body[0].kind
    else {
        panic!("exported while binding");
    };
    assert_eq!(symbols.name(*name), "table_while_loop");
    assert_eq!(export_span.unwrap().text(text), "`table_while_loop");
    assert_eq!(initializer.span.text(text), "remaining");
    assert_eq!(
        macro_.body[0].span.text(text),
        "while `table_while_loop:=remaining {#insert body;}"
    );
    let StatementKind::While(jai_syntax::WhileCondition::Binding { export_span, .. }, _) =
        &macro_.body[1].kind
    else {
        panic!("ordinary while binding");
    };
    assert_eq!(*export_span, None);
}
