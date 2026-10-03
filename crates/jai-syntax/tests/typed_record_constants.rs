use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, RecordMember};

#[test]
fn typed_record_constants_are_not_storage_fields() {
    let mut sources = SourceMap::default();
    let id = sources.insert(
        "typed-record-constants.jai".into(),
        "EGL::struct { TRUE:s32:1; NONE:Word:41; value:s32=9; }".into(),
    );
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    assert_eq!(record.members.len(), 3);
    for member in &record.members[..2] {
        let RecordMember::Constant(constant) = member else {
            panic!()
        };
        assert!(constant.ty.is_some());
    }
    assert!(matches!(record.members[2], RecordMember::Field(_)));
}
