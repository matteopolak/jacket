use jai_lexer::{Kind, lex};
use jai_syntax::{FieldConversion, field_prefix};

#[test]
fn both_source_orders_retain_separate_conversion_and_using_metadata() {
    for source in ["using #as info: Type_Info;", "#as using base: Document;"] {
        let tokens = lex(source).unwrap();
        let mut at = 0;
        let prefix = field_prefix(&tokens, &mut at).unwrap();
        assert!(prefix.using);
        assert_eq!(prefix.conversion, FieldConversion::Implicit);
        assert_eq!(prefix.conversion_span.unwrap().text(source), "#as");
        assert_eq!(tokens[at].kind, Kind::Ident);
        assert_eq!(at, 2);
    }
}

#[test]
fn ordinary_using_does_not_imply_conversion_and_as_does_not_imply_using() {
    for (source, using, conversion) in [
        ("using info: Type_Info;", true, FieldConversion::None),
        ("#as base: Document;", false, FieldConversion::Implicit),
        ("value: int;", false, FieldConversion::None),
    ] {
        let tokens = lex(source).unwrap();
        let mut at = 0;
        let prefix = field_prefix(&tokens, &mut at).unwrap();
        assert_eq!(prefix.using, using);
        assert_eq!(prefix.conversion, conversion);
        assert_eq!(tokens[at].kind, Kind::Ident);
    }
}

#[test]
fn duplicate_qualifiers_report_the_repeated_token() {
    for (source, duplicate) in [
        ("using using info: Type_Info;", "using"),
        ("#as using #as base: Document;", "#as"),
    ] {
        let tokens = lex(source).unwrap();
        let mut at = 0;
        let error = field_prefix(&tokens, &mut at).unwrap_err();
        assert_eq!(error.span, tokens[at].span);
        assert_eq!(error.span.text(source), duplicate);
        assert!(error.message.starts_with("duplicate"));
    }
}

#[test]
fn compiler_reflection_headers_keep_their_source_offsets() {
    let source = include_str!("../../../prelude/reflection.jai");
    let tokens = lex(source).unwrap();
    let mut count = 0;
    for (index, pair) in tokens.windows(2).enumerate() {
        if pair[0].span.text(source) == "using" && pair[1].span.text(source) == "#as" {
            let mut at = index;
            let prefix = field_prefix(&tokens, &mut at).unwrap();
            assert!(prefix.using);
            assert_eq!(prefix.conversion, FieldConversion::Implicit);
            assert_eq!(prefix.conversion_span.unwrap(), pair[1].span);
            assert_eq!(tokens[at].span.text(source), "info");
            count += 1;
        }
    }
    assert_eq!(count, 9);
}

#[test]
fn every_compiler_prelude_fragment_parses_with_runtime_declarations() {
    let mut sources = jai_source::SourceMap::default();
    let mut symbols = jai_source::Symbols::default();
    for (path, source) in [
        ("module.jai", include_str!("../../../stdlib/prelude/module.jai")),
        (
            "platform.jai",
            include_str!("../../../prelude/platform.jai"),
        ),
        (
            "reflection.jai",
            include_str!("../../../prelude/reflection.jai"),
        ),
        (
            "allocation.jai",
            include_str!("../../../prelude/allocation.jai"),
        ),
        (
            "diagnostics.jai",
            include_str!("../../../prelude/diagnostics.jai"),
        ),
        (
            "runtime-storage.jai",
            include_str!("../../../prelude/runtime-storage.jai"),
        ),
        (
            "intrinsics.jai",
            include_str!("../../../prelude/intrinsics.jai"),
        ),
        ("context.jai", include_str!("../../../prelude/context.jai")),
    ] {
        let id = sources.insert(format!("prelude/{path}").into(), source.into());
        let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        assert_eq!(file.source(), id);
        assert_eq!(sources.get(id).unwrap().text(), source);
    }
    assert!(symbols.find("FIRST_ADD_CONTEXT").is_some());
    assert!(symbols.find("compare_and_swap").is_some());
}
