use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, RecordMember, StatementKind};

#[test]
fn local_record_defaults_and_method_default_literals_keep_the_enclosing_block() {
    for text in [
        "main::()->int{R::struct{value:int=7; value=21; read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
        "main::()->int{R::struct{value:=#run seed();seed::()->int{return 21;}read::(input:R=R.{}) ->int{return input.value;}} return R.read()+R.read();}",
    ] {
        let mut sources = SourceMap::default();
        let source = sources.insert("local-record-boundary.jai".into(), text.into());
        let parsed = jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default())
            .unwrap_or_else(|error| panic!("{}", error.render(&sources)));
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!("expected the enclosing procedure")
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!("expected the enclosing procedure")
        };
        assert_eq!(procedure.body.len(), 2);
        let StatementKind::Record(record) = &procedure.body[0].kind else {
            panic!("expected the local record")
        };
        assert!(
            record
                .members
                .iter()
                .any(|member| matches!(member, RecordMember::Procedure(_)))
        );
        assert!(matches!(procedure.body[1].kind, StatementKind::Return(_)));
        assert_eq!(
            procedure.body[1].span.text(text),
            "return R.read()+R.read();"
        );
    }
}
