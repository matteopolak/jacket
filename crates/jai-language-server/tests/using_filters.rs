//! Filtered imports keep their alias and explain contextual using modifiers.
use jai_language_server::{DiagnosticSeverity, Limits, MarkupKind, Session};
use test_support::lsp::{at, file_uri, position, virtual_session};

const DOG: &str = "foo :: 1;\nbar :: 2;\nbaz :: 3;\nkept :: 4;\n";

#[test]
fn named_except_import_preserves_qualified_access_and_selector_navigation() {
    let mut s = virtual_session(
        &[("/filters/modules/Dog/module.jai", DOG)],
        &["/filters/modules"],
        None,
    );
    let uri = file_uri("/filters/main.jai");
    let text = "using,except(foo, bar, baz) Dog :: #import \"Dog\";\nmain :: () { result := kept + Dog.foo + Dog.bar + Dog.baz; }\n";
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != DiagnosticSeverity::Error)
    );
    let items = s.completion(&uri, at(text, " result")).unwrap().items;
    assert!(items.iter().any(|i| i.label == "kept"));
    assert!(items.iter().any(|i| i.label == "Dog"));
    for (name, marker, declaration) in [
        ("foo", "foo,", "foo ::"),
        ("bar", "bar,", "bar ::"),
        ("baz", "baz)", "baz ::"),
    ] {
        assert!(!items.iter().any(|i| i.label == name), "{name}");
        let p = at(text, marker);
        let hover = s.hover(&uri, p).unwrap().expect(name);
        assert!(hover.contents.value.contains(name));
        let targets = s.definition(&uri, p).unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].uri, "file:///filters/modules/Dog/module.jai");
        assert_eq!(targets[0].range.start, at(DOG, declaration));
    }
}

#[test]
fn using_filter_modifiers_have_contextual_hover_without_semantic_analysis() {
    for (name, suffix, description) in [
        ("only", "(foo) value;", "selected names"),
        (
            "except",
            "(foo, bar, baz) Dog :: #import \"Dog\";",
            "except the selected names",
        ),
        ("map", "(new = old) value;", "new names"),
        ("except", " .[\"foo\"] value;", "except the selected names"),
        ("except", "", "except the selected names"),
    ] {
        let text = format!("using /* 🐕 */ ,\n {name}{suffix}");
        let mut s = Session::new(Limits::default());
        let uri = file_uri("/filters/main.jai");
        s.open(uri.clone(), 1, text.clone()).unwrap();
        let offset = text.find(name).unwrap();
        for kind in [MarkupKind::PlainText, MarkupKind::Markdown] {
            let hover = s
                .hover_as(&uri, position(&text, offset + 2), kind)
                .unwrap()
                .expect(name);
            assert!(
                hover.contents.value.contains(description),
                "{}",
                hover.contents.value
            );
            assert_eq!(hover.range.start, position(&text, offset));
            assert_eq!(hover.range.end, position(&text, offset + name.len()));
        }
    }
}

#[test]
fn filter_words_remain_ordinary_identifiers_outside_using_modifiers() {
    let uri = file_uri("/filters/main.jai");
    let mut s = Session::new(Limits::default());
    let text = "except :: 42; only :: 5; map :: 6;\nmain :: () { n := except + only + map; s := \"using,except(foo)\"; }\n// using,except(foo)\n";
    s.open(uri.clone(), 1, text.into()).unwrap();
    for marker in ["except +", "only +", "map;"] {
        let hover = s.hover(&uri, at(text, marker)).unwrap().expect(marker);
        assert!(!hover.contents.value.contains("Using filter"));
        assert!(!s.definition(&uri, at(text, marker)).unwrap().is_empty());
    }
    for offset in [
        text.find("using,except").unwrap(),
        text.rfind("using,except").unwrap(),
    ] {
        assert!(
            s.hover(&uri, position(text, offset + "using,ex".len()))
                .unwrap()
                .is_none()
        );
    }
}
