use jai_source::{SourceMap, Symbols};
use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, StatementKind};
#[test]
fn one_initializer_retains_all_constant_result_names() {
    let mut sources = SourceMap::default();
    let id = sources.insert("constant-results.jai".into(), "facts :: ()->bool,bool,int {return true,true,32;} main :: ()->int {IS_INTEGER,SIGNED,BITS :: #run facts(); return BITS;}".into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let StatementKind::ConstantResults(group) = &procedure.body[0].kind else {
        panic!("expected constant result group")
    };
    assert_eq!(group.names.len(), 3);
    assert!(matches!(
        group.initializer.kind,
        ExpressionKind::CompileTime(_)
    ));
    let names = group
        .names
        .iter()
        .map(|(name, _)| symbols.name(*name))
        .collect::<Vec<_>>();
    assert_eq!(names, ["IS_INTEGER", "SIGNED", "BITS"]);
}
#[test]
fn constant_result_bindings_cannot_be_existing_places() {
    let mut sources = SourceMap::default();
    let id = sources.insert(
        "invalid-constant-results.jai".into(),
        "main :: () {old.field,new :: #run facts();}".into(),
    );
    let error =
        jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
    assert!(error.message.contains("identifier bindings"), "{error:?}");
}
