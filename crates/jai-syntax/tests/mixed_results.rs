use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, ResultTargetBinding, StatementKind};
#[test]
fn source_marks_existing_and_new_result_destinations_explicitly() {
    let mut sources = SourceMap::default();
    let id = sources.insert(
        "mixed.jai".into(),
        "main :: () { old=,fresh := pair(); fresh,old= :s64 = pair(); success:,plugins_to_create:,args = parse_plugin_arguments(args); }".into(),
    );
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let StatementKind::MixedResults {
        bindings,
        ty: None,
        ..
    } = &procedure.body[0].kind
    else {
        panic!()
    };
    assert!(matches!(bindings[0], ResultTargetBinding::Existing(_)));
    assert!(matches!(bindings[1], ResultTargetBinding::New { .. }));
    let StatementKind::MixedResults {
        bindings,
        ty: Some(_),
        ..
    } = &procedure.body[1].kind
    else {
        panic!()
    };
    assert!(matches!(bindings[0], ResultTargetBinding::New { .. }));
    assert!(matches!(bindings[1], ResultTargetBinding::Existing(_)));
    let StatementKind::MixedResults {
        bindings,
        ty: None,
        ..
    } = &procedure.body[2].kind
    else {
        panic!()
    };
    assert!(matches!(bindings[0], ResultTargetBinding::New { .. }));
    assert!(matches!(bindings[1], ResultTargetBinding::New { .. }));
    assert!(matches!(bindings[2], ResultTargetBinding::Existing(_)));
}
