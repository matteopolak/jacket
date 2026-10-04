use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CodeBody, Declaration, ExpressionKind, FileDeclarationKind, FileItem, ParsedFile, StatementKind,
};

fn parsed(text: &str) -> (ParsedFile, Symbols) {
    let mut sources = SourceMap::default();
    let id = sources.insert("termination.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    (file, symbols)
}

fn body(file: &ParsedFile) -> &[jai_syntax::Statement] {
    let FileItem::Declaration(declaration) = file.items().last().unwrap() else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    &procedure.body
}

#[test]
fn empty_statements_retain_ranges_and_single_statement_control_flow() {
    let text = "main::(){; if true ; while false ; {;} defer {;};; return;}";
    let (file, _) = parsed(text);
    let statements = body(&file);
    assert!(matches!(statements[0].kind, StatementKind::Empty));
    assert_eq!(statements[0].span.text(text), ";");
    let StatementKind::If(_, yes, _) = &statements[1].kind else {
        panic!()
    };
    assert!(matches!(
        yes.as_slice(),
        [jai_syntax::Statement {
            kind: StatementKind::Empty,
            ..
        }]
    ));
    let StatementKind::While(_, loop_body) = &statements[2].kind else {
        panic!()
    };
    assert!(matches!(
        loop_body.as_slice(),
        [jai_syntax::Statement {
            kind: StatementKind::Empty,
            ..
        }]
    ));
    let StatementKind::Defer(deferred) = &statements[4].kind else {
        panic!()
    };
    assert!(matches!(deferred[0].kind, StatementKind::Empty));
    assert!(matches!(statements[5].kind, StatementKind::Empty));
}

#[test]
fn closed_callable_code_and_aggregate_values_retain_their_actual_bodies() {
    let text = "main::(){ callback:=()->int{return 42;} quoted::#code{missing();} union{a:u64;b:float64;} using enum u8{A::1;B;} return;}";
    let (file, _) = parsed(text);
    let statements = body(&file);
    let StatementKind::Declare(Declaration::Inferred {
        initializer, ..
    }) = &statements[0].kind
    else {
        panic!()
    };
    assert!(matches!(
        initializer.kind,
        ExpressionKind::AnonymousProcedure(_)
    ));
    let StatementKind::Constant(code) = &statements[1].kind else {
        panic!()
    };
    let ExpressionKind::Code(CodeBody::Block(quoted)) = &code.initializer.kind else {
        panic!()
    };
    assert_eq!(quoted.len(), 1);
    assert_eq!(quoted[0].span.text(text), "missing();");
    let StatementKind::Expression(aggregate) = &statements[2].kind else {
        panic!()
    };
    let ExpressionKind::Type(jai_syntax::TypeSyntax::InlineRecord(record)) = &aggregate.kind else {
        panic!()
    };
    assert_eq!(record.fields().count(), 2);
    assert!(matches!(statements[3].kind, StatementKind::Using(_)));
}

#[test]
fn here_string_assignment_preserves_delimiter_text_and_following_statement() {
    let text = "main::(){text:=\"old\";text=#string END\nsemi; /* literal */\nEND\nreturn;}";
    let (file, _) = parsed(text);
    let StatementKind::Assign(_, value) = &body(&file)[1].kind else {
        panic!()
    };
    let ExpressionKind::HereString(literal) = &value.kind else {
        panic!()
    };
    assert_eq!(literal.bytes, b"semi; /* literal */\n");
    assert_eq!(
        value.span.text(text),
        "#string END\nsemi; /* literal */\nEND"
    );
    assert!(matches!(body(&file)[2].kind, StatementKind::Return(None)));
}

#[test]
fn compile_time_field_assignment_keeps_its_returning_producer() {
    let text = "main::(){state.value=#run -> int{return 42;} state.other=1;}";
    let (file, _) = parsed(text);
    let StatementKind::AssignPlace {
        value, ..
    } = &body(&file)[0].kind
    else {
        panic!()
    };
    assert!(matches!(value.kind, ExpressionKind::CompileTime(_)));
    assert!(matches!(
        body(&file)[1].kind,
        StatementKind::AssignPlace { .. }
    ));
}

#[test]
fn extra_enum_separators_do_not_create_or_renumber_members() {
    let (file, symbols) = parsed("E::enum u8{;A::7;;B;;;C;} main::(){} ");
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Enum(enumeration) = &declaration.kind else {
        panic!()
    };
    assert_eq!(enumeration.members.len(), 3);
    let second = jai_syntax::EnumMemberSyntax::new(&enumeration.members)
        .nth(1)
        .unwrap();
    assert_eq!(symbols.name(second.name), "B");
    assert!(second.initializer.is_none());
}

#[test]
fn ordinary_and_wrapped_values_still_require_semicolons() {
    for text in [
        "main::(){x:=1 return;}",
        "main::(){x:=1;x=2 return;}",
        "main::(){call() return;}",
        "main::(){x:=(()->int{return 1;}) return;}",
        "main::(){x:=accept(#code{}) return;}",
        "main::(){text:=(#string END\nx\nEND\n) return;}",
        "main::(){text:=\"a\"+#string END\nx\nEND\nreturn;}",
        "main::(){quote::#code 42 return;}",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("invalid-termination.jai".into(), text.into());
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert!(error.message.contains("';'"), "{text}: {error:?}");
        assert!(error.location.span.end <= text.len());
    }
}
