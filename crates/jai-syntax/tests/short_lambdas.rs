use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, RecordMember};

fn parse(text: &str) -> jai_syntax::ParsedFile {
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("short-lambdas.jai".into(), text.into());
    jai_syntax::parse_file(
        sources.get(source).unwrap(),
        &mut jai_source::Symbols::default(),
    )
    .unwrap()
}

#[test]
fn named_short_lambda_retains_parameter_and_result_source_spans() {
    let text = "HashKey :: (key) => get_hash(key);";
    let file = parse(text);
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!()
    };
    let ExpressionKind::ShortLambda(lambda) = &constant.initializer.kind else {
        panic!()
    };
    assert_eq!(lambda.parameters.len(), 1);
    assert!(lambda.parameters[0].ty.is_none());
    assert_eq!(
        &text[lambda.parameters[0].span.start..lambda.parameters[0].span.end],
        "key"
    );
    assert_eq!(
        &text[lambda.body.span.start..lambda.body.span.end],
        "get_hash(key)"
    );
}

#[test]
fn record_member_short_lambdas_keep_constant_declaration_identity() {
    let file = parse(
        "HashMap :: struct { HashKey :: (key) => get_hash(key); CompareKeys :: (a, b) => a == b; }",
    );
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    assert!(record.members.iter().all(|member| matches!(member, RecordMember::Constant(value) if matches!(value.initializer.kind,ExpressionKind::ShortLambda(_)))));
}

#[test]
fn zero_and_typed_parameters_do_not_confuse_parenthesized_expressions() {
    parse(
        "answer :: () => 42; typed :: (value: u8) => value; main :: () -> int { f: (int)->int = (value) => (value + (1 + 2)); return f((39)); }",
    );
}

#[test]
fn bare_parameter_lambda_preserves_modern_library_syntax() {
    let text = "negate :: value => -value; main :: () -> int { return ((x) => x + 2)(40); }";
    let file = parse(text);
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!()
    };
    let ExpressionKind::ShortLambda(lambda) = &constant.initializer.kind else {
        panic!()
    };
    assert_eq!(
        &text[lambda.parameters[0].span.start..lambda.parameters[0].span.end],
        "value"
    );
}

#[test]
fn block_lambda_retains_original_statements_without_an_expression_placeholder() {
    let file = parse("shutdown :: p => { free(p); }; main :: () {} ");
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!()
    };
    let ExpressionKind::ShortLambda(lambda) = &constant.initializer.kind else {
        panic!()
    };
    let jai_syntax::ShortLambdaBodyKind::Block(statements) = &lambda.body.kind else {
        panic!()
    };
    assert_eq!(statements.len(), 1);
    assert!(matches!(
        statements[0].kind,
        jai_syntax::StatementKind::Expression(_)
    ));
}
