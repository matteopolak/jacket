use jai_source::{SourceMap, Symbols};
use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, StatementKind};
fn parse(text: &str) -> jai_syntax::ParsedFile {
    let mut map = SourceMap::default();
    let id = map.insert("anonymous.jai".into(), text.into());
    jai_syntax::parse_file(map.get(id).unwrap(), &mut Symbols::default()).unwrap()
}
#[test]
fn real_full_source_body_and_header() {
    let file = parse(
        "main::()->s32 { f := (value:s32=40)->(answer:s32=42) #no_context { return; }; return f(); }",
    );
    let FileItem::Declaration(d) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(p) = &d.kind else {
        panic!()
    };
    let StatementKind::Declare(declaration) = &p.body[0].kind else {
        panic!("{:?}", p.body[0])
    };
    let e = match declaration {
        jai_syntax::Declaration::Inferred {
            initializer, ..
        } => initializer,
        _ => panic!(),
    };
    assert!(matches!(e.kind, ExpressionKind::AnonymousProcedure(_)));
    let ExpressionKind::AnonymousProcedure(a) = &e.kind else {
        panic!()
    };
    assert_eq!(a.parameters.len(), 1);
    assert_eq!(a.results.len(), 1);
    assert_eq!(a.body.len(), 1);
    assert!(a.span.start > p.span.start);
}
#[test]
fn bodyless_prototype_shares_only_callable_header() {
    let file = parse("foreign::(value:s32=40)->s32 #c_call #foreign;");
    let FileItem::Declaration(d) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::ProcedurePrototype(p) = &d.kind else {
        panic!()
    };
    assert_eq!(p.header.parameters.len(), 1);
    assert_eq!(p.header.results.len(), 1);
}
#[test]
fn return_and_immediate_call_remain_expressions() {
    parse(
        "Callback::()->s32; factory::()->Callback { return ()->s32 { return 42; }; } main::()->s32 {return (() -> s32 {return 42;})();}",
    );
}
