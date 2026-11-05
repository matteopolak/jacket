//! Statement-removal edits and their shared CLI previews.
use super::*;

/// Removing a no-op owns a whole line only when no comment or other code shares it.
#[test]
fn self_assignment_fix_preserves_surroundings() {
    let root = TempDir::new("self-assignment-fix");
    let file = root.join("main.jai");
    for (body, fixed) in [
        ("    x = x;\n", ""),
        ("    x = x;\r\n", ""),
        ("    x = x; // retain\n", "     // retain\n"),
        ("    /* retain */ x = x;\n", "    /* retain */ \n"),
        ("    x = x; x += 1;\n", "     x += 1;\n"),
    ] {
        let text = format!("main :: () {{\n    x := 1;\n{body}}}\n");
        std::fs::write(&file, &text).unwrap();
        let (lints, _) = lint(&file, &[("self_assignment", Level::Warn)]);
        assert_eq!(lints.len(), 1, "{lints:?}");
        let fix = lints[0].fix.as_ref().unwrap();
        let mut actual = text.clone();
        for edit in fix.edits.iter().rev() {
            actual.replace_range(edit.start..edit.end, &edit.text);
        }
        assert_eq!(actual, format!("main :: () {{\n    x := 1;\n{fixed}}}\n"));
        if fixed.is_empty() {
            assert_eq!(
                jaic::render::changed_lines(
                    &text,
                    &jaic::source::LineIndex::new(&text),
                    &fix.edits
                        .iter()
                        .map(|e| (e.start, e.end, e.text.clone()))
                        .collect::<Vec<_>>()
                ),
                vec![("3".into(), '-', "    x = x;".into())]
            );
        }
    }
}

#[test]
fn duplicate_condition_preview_only_removes_dead_branch() {
    let file = repo().join("tests/lint/duplicate_condition/example.jai");
    let (lints, text) = lint(&file, &[("duplicate_condition", Level::Warn)]);
    assert_eq!(lints.len(), 1);
    let fix = lints[0].fix.as_ref().unwrap();
    let mut fixed = text.clone();
    for e in fix.edits.iter().rev() {
        fixed.replace_range(e.start..e.end, &e.text);
    }
    assert_eq!(
        fixed,
        text.replace("    else if key == 1 print(\"uno\\n\");\n", "")
    );
    assert_eq!(
        jaic::render::changed_lines(
            &text,
            &jaic::source::LineIndex::new(&text),
            &fix.edits
                .iter()
                .map(|e| (e.start, e.end, e.text.clone()))
                .collect::<Vec<_>>()
        ),
        vec![(
            "6".into(),
            '-',
            "    else if key == 1 print(\"uno\\n\");".into()
        )]
    );
}

#[test]
fn duplicate_condition_fix_preserves_live_branches_and_comments() {
    let root = TempDir::new("duplicate-condition-fix");
    let file = root.join("main.jai");
    for separator in ["\n    ", "\r\n    ", " "] {
        let source = format!(
            "main :: () {{\n    x := 1;\n    if x == 1 x += 1;{separator}else if x == 1 x += 2;{separator}else x += 3;\n}}\n"
        );
        std::fs::write(&file, &source).unwrap();
        let (lints, _) = lint(&file, &[("duplicate_condition", Level::Warn)]);
        assert_eq!(lints.len(), 1, "{lints:?}");
        let fix = lints[0].fix.as_ref().unwrap();
        let mut actual = source.clone();
        for edit in fix.edits.iter().rev() {
            actual.replace_range(edit.start..edit.end, &edit.text);
        }
        assert_eq!(
            actual,
            source.replace(&format!("{separator}else if x == 1 x += 2;"), "")
        );
    }
    let source = "main :: () { x := 1; if x == 1 x += 1; /* retain */ else if x == 1 x += 2; }\n";
    std::fs::write(&file, source).unwrap();
    let (lints, _) = lint(&file, &[("duplicate_condition", Level::Warn)]);
    assert_eq!(lints.len(), 1);
    assert!(
        lints[0].fix.is_none(),
        "a branch removal must not erase its comment"
    );
}
