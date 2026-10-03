use jai_source::{SourceMap, Symbols};
use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, StatementKind, UnaryOp};

#[test]
fn is_constant_keyword_enters_the_existing_call_ast_with_original_source_ranges() {
    let text = "main::(value:int){#if !is_constant(value) #asm { value === c; }}";
    let mut sources = SourceMap::default();
    let id = sources.insert("constant-query.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(item) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &item.kind else {
        panic!()
    };
    let StatementKind::CompileTimeIf {
        condition,
        then_body,
        ..
    } = &main.body[0].kind
    else {
        panic!()
    };
    let ExpressionKind::Unary(UnaryOp::LogicalNot, call) = &condition.kind else {
        panic!()
    };
    let ExpressionKind::Call(name, arguments) = &call.kind else {
        panic!()
    };
    assert_eq!(symbols.name(*name), "is_constant");
    assert_eq!(call.span.text(text), "is_constant(value)");
    assert_eq!(condition.span.text(text), "!is_constant(value)");
    assert_eq!(arguments.len(), 1);
    assert_eq!(arguments[0].value.span.text(text), "value");
    assert!(matches!(then_body[0].kind, StatementKind::Simd(_)));
}
