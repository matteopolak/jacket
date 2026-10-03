//! Independently authored examples of syntax used by pinned Focus and Jaison.
use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CompileTimeBody, CompileTimeRun, ExpressionKind, FileDeclarationKind, FileItem, RecordMember,
    StatementKind, parse_file,
};

#[test]
fn block_run_initializers_end_declarations_without_a_semicolon() {
    let source = "VERSION::#run->string {return \"v\";} NEXT::#run->int {return 41;}; main::()->int { local:=#run->int {return 1;} return NEXT+local; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("block-run-declaration.jai".into(), source.into());
    let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    assert_eq!(parsed.items().len(), 3);
    let FileItem::Declaration(first) = &parsed.items()[0] else {
        panic!("expected constant");
    };
    assert_eq!(
        first.location.span.text(source),
        "VERSION::#run->string {return \"v\";}"
    );
    let FileDeclarationKind::Constant(value) = &first.kind else {
        panic!("expected constant");
    };
    assert!(matches!(
        value.initializer.kind,
        ExpressionKind::CompileTime(CompileTimeRun {
            body: CompileTimeBody::Procedure { .. },
            ..
        })
    ));
}

#[test]
fn ordinary_or_composed_initializers_still_require_their_semicolon() {
    for source in [
        "VALUE::1 NEXT::2;",
        "VALUE::#run number() NEXT::2;",
        "VALUE::#run->int {return 1;}+2 NEXT::2;",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("missing-terminator.jai".into(), source.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.message, "expected ';' after declaration");
        assert_eq!(error.location.span.text(source), "NEXT");
    }
}

#[test]
fn parenthesized_dereference_keeps_pointer_operand_precedence_and_source_span() {
    let source = "main::(){ value:=ifx (.*)(cast(*bool)data) 17 else 25; next:=(.*)cast(*int)data+1; ((.*)cast(*int)data)=42; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("prefix-dereference.jai".into(), source.into());
    let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(procedure) = &parsed.items()[0] else {
        panic!("expected procedure");
    };
    let FileDeclarationKind::Procedure(procedure) = &procedure.kind else {
        panic!("expected procedure");
    };
    let StatementKind::Declare(jai_syntax::Declaration::Inferred { initializer, .. }) =
        &procedure.body[0].kind
    else {
        panic!("expected initializer");
    };
    let ExpressionKind::Conditional(conditional) = &initializer.kind else {
        panic!("expected conditional");
    };
    assert!(matches!(
        conditional.condition.kind,
        ExpressionKind::Dereference(_)
    ));
    assert_eq!(
        conditional.condition.span.text(source),
        "(.*)(cast(*bool)data)"
    );
    let StatementKind::Declare(jai_syntax::Declaration::Inferred { initializer, .. }) =
        &procedure.body[1].kind
    else {
        panic!("expected initializer");
    };
    assert!(
        matches!(&initializer.kind,ExpressionKind::Binary(_,left,_) if matches!(left.kind,ExpressionKind::Dereference(_)))
    );
}

#[test]
fn dereference_operator_without_an_operand_is_a_source_error() {
    let source = "main::(){value:=(.*);}";
    let mut sources = SourceMap::default();
    let id = sources.insert("missing-pointer.jai".into(), source.into());
    let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
    assert_eq!(error.location.span.text(source), ";");
}

#[test]
fn record_assertion_messages_share_argument_syntax_and_keep_source_nodes() {
    let source = "Record::struct { #assert true \"ready\"; #assert(size_of(Record)==8, REASON); value:int; }";
    let mut sources = SourceMap::default();
    let id = sources.insert("record-assert-message.jai".into(), source.into());
    let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!("expected record");
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!("expected record");
    };
    let [
        RecordMember::Assert {
            message: Some(message),
            span,
            ..
        },
        RecordMember::Assert {
            message: Some(named),
            ..
        },
        RecordMember::Field(_),
    ] = record.members.as_slice()
    else {
        panic!("expected ordered assertions and field");
    };
    assert!(matches!(message.kind, ExpressionKind::String(_)));
    assert_eq!(span.text(source), "#assert true \"ready\";");
    assert_eq!(named.span.text(source), "REASON");
}
