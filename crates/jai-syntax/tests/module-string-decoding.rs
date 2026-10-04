use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileItem, ModuleArgumentValue};
fn parse(text: &str) -> Result<jai_syntax::ParsedFile, jai_source::LocatedDiagnostic> {
    let mut sources = SourceMap::default();
    let id = sources.insert("module-strings.jai".into(), text.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default())
}
#[test]
fn quoted_source_paths_and_arguments_share_actual_unicode_and_quote_decoding() {
    let source = r##"#import,string "answer::()->string{return \"λ\";}"; A::#import,file "lib/\u03bb.jai"(Name="quoted \"λ\"");"##;
    let parsed = parse(source).unwrap();
    let FileItem::Import(first) = &parsed.items()[0] else {
        panic!()
    };
    assert_eq!(first.target, "answer::()->string{return \"λ\";}");
    let FileItem::Import(second) = &parsed.items()[1] else {
        panic!()
    };
    assert_eq!(second.target, "lib/λ.jai");
    let ModuleArgumentValue::String(value) = &second.arguments.instance.as_ref().unwrap()[0].value
    else {
        panic!()
    };
    assert_eq!(value, "quoted \"λ\"");
}
#[test]
fn invalid_utf8_and_invalid_escape_keep_the_original_literal_span() {
    for literal in [r#""\xff""#, r#""\q""#] {
        let source = format!("#import,string {literal};");
        let error = parse(&source).unwrap_err();
        assert_eq!(error.location.span.text(&source), literal);
        assert!(
            error.message.contains("UTF-8") || error.message.contains("escape"),
            "{}",
            error.message
        );
    }
}
#[test]
fn quoted_and_here_string_module_sources_retain_equivalent_utf8_text() {
    let source = r##"#import,string "value::\"λ\";\n"; #import,string #string END
value::"λ";
END
;"##;
    let parsed = parse(source).unwrap();
    let FileItem::Import(first) = &parsed.items()[0] else {
        panic!()
    };
    let FileItem::Import(second) = &parsed.items()[1] else {
        panic!()
    };
    assert_eq!(first.target, second.target);
}
