use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem};

#[test]
fn context_is_a_source_name_without_losing_implicit_context_expressions() {
    let mut sources = SourceMap::default();
    let source = sources.insert(
        "context-binding.jai".into(),
        "Context::struct{value:int;} context:Context; main::(){value:=context.value;}".into(),
    );
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[1] else {
        panic!("global declaration");
    };
    let FileDeclarationKind::Global(global) = &declaration.kind else {
        panic!("global context storage");
    };
    assert_eq!(symbols.name(global.declaration.name()), "context");
    let FileItem::Declaration(declaration) = &file.items()[2] else {
        panic!("main declaration");
    };
    let FileDeclarationKind::Procedure(main) = &declaration.kind else {
        panic!("main body");
    };
    assert!(!main.body.is_empty());
}

#[test]
fn unrelated_reserved_keywords_do_not_become_declaration_names() {
    let mut sources = SourceMap::default();
    let source = sources.insert("reserved.jai".into(), "return:int;".into());
    assert!(jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).is_err());
}

#[test]
#[ignore = "requires the separately supplied reference and pinned upstream source corpus"]
fn unchanged_open_jai_basic_parses_its_context_global() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/upstream/withlang-dev--open-jai/modules/Basic/module.jai");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut sources = SourceMap::default();
    let source = sources.insert(path, text);
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols)
        .unwrap_or_else(|error| panic!("{}", error.render(&sources)));
    assert!(parsed.items().iter().any(|item| match item {
        FileItem::Declaration(declaration) => matches!(&declaration.kind, FileDeclarationKind::Global(global) if symbols.name(global.declaration.name())=="context"),
        _ => false,
    }));
}
