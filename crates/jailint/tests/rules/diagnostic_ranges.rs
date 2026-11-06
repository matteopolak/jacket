//! Diagnostic extents are chosen by the rule's expression or statement role.
use super::*;

#[test]
fn diagnostic_ranges_match_complete_syntax_without_trailing_trivia() {
    for (rule, expected) in [
        ("neg_cmp_float", "!(x < limit)"),
        ("manual_array_find", "for names"),
        ("almost_swapped", "a = b;\n    b = a;"),
    ] {
        let file = repo().join(format!("tests/lint/{rule}/example.jai"));
        let (lints, text) = lint(&file, &[(rule, Level::Warn)]);
        assert_eq!(lints.len(), 1, "{rule}");
        assert_eq!(
            &text[lints[0].start..lints[0].end],
            expected.replace(
                '\n',
                if text.contains("\r\n") {
                    "\r\n"
                } else {
                    "\n"
                }
            ),
            "{rule}"
        );
    }
}

#[test]
fn manual_index_counter_preview_keeps_source_order_and_neutral_context() {
    let file = repo().join("tests/lint/manual_index_counter/example.jai");
    let (lints, text) = lint(&file, &[("manual_index_counter", Level::Warn)]);
    assert_eq!(lints.len(), 1);
    let edits = lints[0]
        .fix
        .as_ref()
        .unwrap()
        .edits
        .iter()
        .map(|e| (e.start, e.end, e.text.clone()))
        .collect::<Vec<_>>();
    let actual = jaic::render::changed_lines(&text, &jaic::source::LineIndex::new(&text), &edits);
    assert_eq!(
        actual,
        vec![
            ("3".into(), ' ', "show :: (names: [] string) {".into()),
            ("4".into(), '-', "    n := 0;".into()),
            ("5".into(), ' ', String::new()),
            ("6".into(), ' ', "    for names {".into()),
            (
                "6".into(),
                '~',
                "        print(\"% %\\n\", it_index, it);".into()
            ),
            ("8".into(), '-', "        n += 1;".into()),
            ("9".into(), ' ', "    }".into()),
        ]
    );
}
