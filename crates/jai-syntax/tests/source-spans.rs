use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CodeBody, ExpressionKind, FileDeclarationKind, FileItem, ParsedFile, Procedure, StatementKind,
};

fn procedure(file: &ParsedFile, item: usize) -> &Procedure {
    let FileItem::Declaration(declaration) = &file.items()[item] else {
        panic!("expected procedure declaration")
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!("expected procedure")
    };
    procedure
}

#[test]
fn complete_statement_ranges_include_keywords_names_braces_and_terminators() {
    let text = "\u{feff}// é before byte offsets\r\nmain :: () {\r\n value: int;\r\n return;\r\n { value += 1; }\r\n if true then value = 2; else return;\r\n}";
    let mut sources = SourceMap::default();
    let source = sources.insert("ranges.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let body = &procedure(&file, 0).body;
    assert_eq!(
        body.iter()
            .map(|statement| statement.span.text(text))
            .collect::<Vec<_>>(),
        [
            "value: int;",
            "return;",
            "{ value += 1; }",
            "if true then value = 2; else return;"
        ]
    );
    let StatementKind::Block(block) = &body[2].kind else {
        panic!("expected block")
    };
    assert_eq!(block[0].span.text(text), "value += 1;");
    let StatementKind::If(_, yes, no) = &body[3].kind else {
        panic!("expected conditional")
    };
    assert_eq!(yes[0].span.text(text), "value = 2;");
    assert_eq!(no[0].span.text(text), "return;");
    assert_eq!(file.location(no[0].span).source, source);
    assert_eq!(
        file.clone().location(no[0].clone().span),
        file.location(no[0].span)
    );
}

#[test]
fn declaration_place_and_control_paths_share_one_statement_range_contract() {
    for statement in [
        "x := 1;",
        "N :: 42;",
        "x = 2;",
        "x *= 3;",
        "point.x = 4;",
        "array[0] += 1;",
        "x,y := pair();",
        "x,y = 1,2;",
        "return x,y;",
        "consume(1);",
        "if true { return; } else { consume(1); }",
        "if 1 == { case 1; consume(1); case; return; }",
        "#if ENABLED { chosen(); } else { rejected(); }",
        "#no_aoc { value += 1; }",
        "while ready := true { continue ready; }",
        "for i: 1..3 { break i; }",
        "for value: values { remove value; }",
        "defer consume(1);",
        "push_context context { consume(1); }",
        "#insert quoted;",
        "local :: () { return; }",
        "external :: () #foreign lib;",
        "lib :: #system_library \"c\";",
        "Point :: struct { x: int; }",
        "Mode :: enum { On; Off; }",
        "Alias :: int;",
        "#add_context field: int;",
    ] {
        let text = format!("// prefix\nmain :: () {{ {statement} }}");
        let mut sources = SourceMap::default();
        let source = sources.insert("paths.jai".into(), text.clone());
        let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default())
            .unwrap_or_else(|error| panic!("{statement}: {error}"));
        let body = &procedure(&file, 0).body;
        assert_eq!(body.len(), 1, "{statement}");
        assert_eq!(body[0].span.text(&text), statement);
    }
}

#[test]
fn quoted_statement_and_block_clones_retain_original_payload_ranges() {
    let text = "// original source\nquoted :: #code if true { return 7; } else return 8; body :: #code { value := 3; consume(value); };";
    let mut sources = SourceMap::default();
    let source = sources.insert("quote.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!("expected declaration")
    };
    let FileDeclarationKind::Constant(quoted) = &declaration.kind else {
        panic!("expected constant")
    };
    let ExpressionKind::Code(CodeBody::Statement(statement)) = &quoted.initializer.kind else {
        panic!("expected quoted statement")
    };
    let cloned = statement.clone();
    assert_eq!(
        cloned.span.text(text),
        "if true { return 7; } else return 8;"
    );
    let StatementKind::If(_, yes, no) = &cloned.kind else {
        panic!("expected conditional")
    };
    assert_eq!(yes[0].span.text(text), "return 7;");
    assert_eq!(no[0].span.text(text), "return 8;");
    assert_eq!(file.location(cloned.span).source, source);
    let FileItem::Declaration(declaration) = &file.items()[1] else {
        panic!("expected declaration")
    };
    let FileDeclarationKind::Constant(quoted) = &declaration.kind else {
        panic!("expected constant")
    };
    let ExpressionKind::Code(CodeBody::Block(body)) = &quoted.initializer.kind else {
        panic!("expected quoted block")
    };
    assert_eq!(
        body.clone()
            .iter()
            .map(|statement| statement.span.text(text))
            .collect::<Vec<_>>(),
        ["value := 3;", "consume(value);"]
    );
}

#[test]
fn scalar_adapter_preserves_the_same_source_ranges() {
    let text = "main :: () { local := 3; if true return local; }";
    let module = jai_syntax::parse(text).unwrap();
    let mut sources = SourceMap::default();
    let source = sources.insert("scalar.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    for (scalar, full) in module.procedures()[0]
        .body
        .iter()
        .zip(&procedure(&file, 0).body)
    {
        assert_eq!(scalar.span, full.span);
    }
}

#[test]
fn context_and_remove_quotations_use_the_statement_parser_range() {
    for payload in ["remove value;", "push_context context { return; }"] {
        let terminator = if payload.ends_with(';') {
            ""
        } else {
            ";"
        };
        let text = format!("// quote origin\nquoted :: #code {payload}{terminator}");
        let mut sources = SourceMap::default();
        let source = sources.insert("control-quote.jai".into(), text.clone());
        let file =
            jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("expected declaration")
        };
        let FileDeclarationKind::Constant(quoted) = &declaration.kind else {
            panic!("expected constant")
        };
        let ExpressionKind::Code(CodeBody::Statement(statement)) = &quoted.initializer.kind else {
            panic!("expected statement quotation")
        };
        assert_eq!(statement.clone().span.text(&text), payload);
    }
}

#[test]
fn missing_terminators_report_the_unconsumed_token() {
    let text = "main :: () { value := 3 return; }";
    let mut sources = SourceMap::default();
    let source = sources.insert("missing.jai".into(), text.into());
    let error =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap_err();
    assert_eq!(error.location.source, source);
    assert_eq!(error.location.span.text(text), "return");
}

#[test]
fn parentheses_remain_in_expression_ranges_before_binary_and_postfix_joins() {
    for expression in ["(1/0)>2", "((1 + 2))", "(value).field", "(callback)(3)"] {
        let text = format!("main :: () {{ result := {expression}; }}");
        let mut sources = SourceMap::default();
        let source = sources.insert("expression-ranges.jai".into(), text.clone());
        let file =
            jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        let StatementKind::Declare(jai_syntax::Declaration::Inferred {
            initializer, ..
        }) = &procedure(&file, 0).body[0].kind
        else {
            panic!("expected inferred declaration")
        };
        assert_eq!(initializer.span.text(&text), expression);
    }
}
