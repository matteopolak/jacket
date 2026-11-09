//! A finding covers the complete syntax it is about, without trailing whitespace or comments.
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
        let crlf = text.contains("\r\n");
        assert_eq!(
            &text[lints[0].start..lints[0].end],
            expected.replace(
                '\n',
                if crlf {
                    "\r\n"
                } else {
                    "\n"
                }
            ),
            "{rule}"
        );
    }
}

/// Removing a no-op takes its whole line only when nothing else is on it.
#[test]
fn self_assignment_fix_keeps_what_shares_the_line() {
    let root = TempDir::new("self-assignment-fix");
    let file = root.join("main.jai");
    for (body, fixed) in [
        ("    x = x;\n", ""),
        ("    x = x;\r\n", ""),
        ("    x = x; // keep\n", "     // keep\n"),
        ("    /* keep */ x = x;\n", "    /* keep */ \n"),
        ("    x = x; x += 1;\n", "     x += 1;\n"),
    ] {
        let text = format!("main :: () {{\n    x := 1;\n{body}}}\n");
        std::fs::write(&file, &text).unwrap();
        let (lints, _) = lint(&file, &[("self_assignment", Level::Warn)]);
        assert_eq!(lints.len(), 1, "{lints:?}");
        let mut actual = text.clone();
        for edit in lints[0].fix.as_ref().unwrap().edits.iter().rev() {
            actual.replace_range(edit.start..edit.end, &edit.text);
        }
        assert_eq!(actual, format!("main :: () {{\n    x := 1;\n{fixed}}}\n"));
    }
}
