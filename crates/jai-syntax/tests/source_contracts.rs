use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem};
use jai_types::{CallingConvention, ContextMode};
#[test]
fn bare_jai_pack_retains_trailing_defaults_and_context() {
    let mut sources = SourceMap::default();
    let id = sources.insert("source-contract.jai".into(), "join :: (values:..string, separator:=\"\", before_first:=false, after_last:=false)->string #foreign;".into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.kind else {
        panic!()
    };
    assert_eq!(prototype.convention, CallingConvention::Jai);
    assert_eq!(prototype.context, ContextMode::Implicit);
    assert_eq!(prototype.parameters.len(), 4);
    assert!(prototype.parameters[0].variadic);
}
#[test]
fn native_c_metadata_keeps_c_ellipsis_last() {
    for suffix in [
        "#c_call #foreign",
        "#foreign Native",
        "#foreign \"native_join\"",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert(
            "native-c.jai".into(),
            format!("join :: (values:..string, tail:=false)->string {suffix};"),
        );
        let error =
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert!(
            error.message.contains("C variadic parameter must be last"),
            "{error:?}"
        );
    }
}
