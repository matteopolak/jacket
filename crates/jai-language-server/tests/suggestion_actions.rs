//! Every suggestion of a compile error or warning becomes a quick fix, and the notes and help
//! of the diagnostic reach the editor. See docs/compiler/language-server.md.
use jai_language_server::{CodeAction, DiagnosticCode, DiagnosticSeverity, Range};
mod common;
use common::{at_nth, session, uri};

/// The quick fixes the editor offers at `marker` (its first occurrence) in `text`.
fn quick_fixes(s: &jai_language_server::Session, text: &str, marker: &str) -> Vec<CodeAction> {
    let cursor = at_nth(text, marker, 0, 0);
    s.code_actions(
        &uri(),
        Range {
            start: cursor,
            end: cursor,
        },
    )
    .unwrap()
    .into_iter()
    .filter(|a| a.kind == Some("quickfix"))
    .collect()
}

#[test]
fn a_second_error_gets_its_own_quick_fixes() {
    // Two procedures that fail independently: the compile reports both.
    let text = "first :: () {\n    print(\"a\\n\");\n}\nsecond :: () {\n    x := log(2.0);\n}\nmain :: () {}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let errors = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check && d.severity == DiagnosticSeverity::Error)
        .count();
    assert!(errors >= 2, "expected two check errors");
    let first = quick_fixes(&s, text, "print");
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first[0].title, "Add `#import \"Basic\";`");
    assert!(first[0].is_preferred);
    // `log` is declared by more than one module: each is offered, none preferred.
    let second = quick_fixes(&s, text, "log");
    let titles: Vec<&str> = second.iter().map(|a| a.title.as_str()).collect();
    assert_eq!(
        titles,
        ["Add `#import \"Basic\";`", "Add `#import \"Math\";`"],
        "{second:?}"
    );
    assert!(second.iter().all(|a| !a.is_preferred));
}

#[test]
fn a_warning_gets_quick_fixes() {
    let text = "main :: () {\n    return;\n    x := 1;\n    y := 2;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let warnings: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Warning && d.code == DiagnosticCode::Check)
        .collect();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].message.contains("never reached"));
    // The linter has a fix of its own for the unused variable there.
    let fixes: Vec<_> = quick_fixes(&s, text, "x := 1")
        .into_iter()
        .filter(|a| {
            a.diagnostics
                .iter()
                .all(|d| d.code == DiagnosticCode::Check)
        })
        .collect();
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Remove the statements that can not run");
    // A guess about what the author meant, so it is never the preferred fix.
    assert!(!fixes[0].is_preferred);
    assert_eq!(fixes[0].diagnostics, warnings);
    let (target, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(target, uri().as_str());
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "");
    assert_eq!(edits[0].range.start, at_nth(text, "x := 1", 0, 0));
    assert_eq!(
        edits[0].range.end,
        at_nth(text, "y := 2;", 0, "y := 2;".len())
    );
}

#[test]
fn a_name_a_namespaced_import_exports_gets_quick_fixes_that_use_the_import() {
    let text = "M :: #import \"Math\";\nmain :: () {\n    x := sqrt(2.0);\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let fixes = quick_fixes(&s, text, "sqrt");
    let titles: Vec<&str> = fixes.iter().map(|a| a.title.as_str()).collect();
    // No second `#import "Math"`: the module is imported, under `M`.
    assert_eq!(titles, ["Write `M.sqrt`", "Add `using M;`"], "{fixes:?}");
    assert!(fixes[0].is_preferred);
    assert!(!fixes[1].is_preferred);
    let (target, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(target, uri().as_str());
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "M.");
    assert_eq!(edits[0].range.start, at_nth(text, "sqrt", 0, 0));
    assert_eq!(edits[0].range.end, edits[0].range.start);
    let (_, edits) = fixes[1].edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "\nusing M;");
    assert_eq!(
        edits[0].range.start,
        at_nth(text, "\"Math\";", 0, "\"Math\";".len())
    );
}

#[test]
fn an_operator_found_through_its_type_module_offers_the_explicit_call() {
    let text =
        "Math :: #import \"Math\";\nmain :: () {\n    u, v: Math.Vector2;\n    w := u + v;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let warning = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .find(|d| d.severity == DiagnosticSeverity::Warning && d.message.contains("`operator +`"))
        .expect("a warning for the namespaced operator");
    assert!(
        warning.message.contains("help: or add `using Math;`"),
        "{warning:?}"
    );
    let fixes = quick_fixes(&s, text, "u + v");
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Call `Math.operator +` explicitly");
    assert!(fixes[0].is_preferred);
    let (_, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "Math.operator +(u, v)");
}

#[test]
fn help_and_notes_reach_the_editor_in_the_message() {
    let text = "main :: () {\n    return;\n    x := 1;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let diagnostics = s.diagnostics(&uri()).unwrap();
    let warning = diagnostics
        .iter()
        .find(|d| d.severity == DiagnosticSeverity::Warning)
        .unwrap();
    // A suggestion's title and a plain help line, each as a `help:` line.
    assert_in_order(
        &warning.message,
        &[
            "never reached",
            "\nhelp: remove the statements that can not run",
            "\nhelp: remove it, or the `return` before it",
        ],
    );
}

#[test]
fn a_located_note_is_related_information() {
    let text = "main :: () {\n    x := 1;\n    x := 2;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let diagnostics = s.diagnostics(&uri()).unwrap();
    let redeclared = diagnostics
        .iter()
        .find(|d| d.code == DiagnosticCode::Check && !d.related.is_empty())
        .unwrap_or_else(|| panic!("a note with a place: {diagnostics:?}"));
    let note = &redeclared.related[0];
    assert_eq!(note.uri, uri().as_str());
    assert_eq!(note.range.start, at_nth(text, "x := 1", 0, 0));
    // A note with a place is not repeated in the message.
    assert!(
        !redeclared.message.contains(&note.message),
        "{redeclared:?}"
    );
}

#[track_caller]
fn assert_in_order(text: &str, needles: &[&str]) {
    let mut rest = text;
    for needle in needles {
        match rest.find(needle) {
            Some(at) => rest = &rest[at + needle.len()..],
            None => panic!("expected {needle:?} (in this order) in:\n{text}"),
        }
    }
}

#[test]
fn a_conversion_error_offers_the_cast_as_a_quick_fix() {
    let text = "main :: () {\n    a: s64 = 3;\n    b: s32 = a;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let fixes = quick_fixes(&s, text, "a;");
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Convert with `cast(s32)`");
    // A cast can drop bits: the editor does not apply it by itself.
    assert!(!fixes[0].is_preferred);
    let (_, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "cast(s32) a");
    assert_eq!(edits[0].range.start, at_nth(text, "a;", 0, 0));
}

#[test]
fn an_enum_value_out_of_range_offers_to_widen_the_enum() {
    let text = "E :: enum u8 { A :: 300; }\nmain :: () {}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let fixes = quick_fixes(&s, text, "300");
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Widen the enum to `u16`");
    // The widened type is a guess about what the author wants.
    assert!(!fixes[0].is_preferred);
    let (target, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(target, uri().as_str());
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, "u16");
    assert_eq!(edits[0].range.start, at_nth(text, "u8", 0, 0));
    assert_eq!(edits[0].range.end, at_nth(text, "u8", 0, 2));
}

#[test]
fn a_syntax_error_offers_its_suggestion_as_a_quick_fix() {
    let text = "main :: () {\n    x := 1\n    y := 2;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let parse_errors: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Parser)
        .collect();
    assert_eq!(parse_errors.len(), 1, "{parse_errors:?}");
    // The error blames the last token of the line, the `1`.
    let fixes = quick_fixes(&s, text, "1\n");
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Insert `;`");
    // The statement ends there for certain, so the editor may apply it on its own.
    assert!(fixes[0].is_preferred);
    assert_eq!(fixes[0].diagnostics, parse_errors);
    let (target, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(target, uri().as_str());
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].new_text, ";");
    let after_one = at_nth(text, "1\n", 0, 1);
    assert_eq!(edits[0].range.start, after_one);
    assert_eq!(edits[0].range.end, after_one);
}

#[test]
fn a_lexical_error_offers_its_suggestion_as_a_quick_fix() {
    let text = "main :: () {\n    s := \"a\\qb\";\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let fixes = quick_fixes(&s, text, "\\q");
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Escape the backslash");
    assert!(!fixes[0].is_preferred);
    let (_, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "\\\\q");
}

#[test]
fn a_format_argument_warning_offers_to_remove_the_extra_argument() {
    let text = "#import \"Basic\";\nmain :: () {\n    print(\"done\\n\", 1);\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let fixes: Vec<_> = quick_fixes(&s, text, "print(")
        .into_iter()
        .filter(|a| {
            a.diagnostics
                .iter()
                .all(|d| d.code == DiagnosticCode::Check)
        })
        .collect();
    assert_eq!(fixes.len(), 1, "{fixes:?}");
    assert_eq!(fixes[0].title, "Remove the extra argument");
    assert!(!fixes[0].is_preferred);
    let (_, edits) = fixes[0].edit.as_ref().unwrap();
    assert_eq!(edits[0].new_text, "");
    assert_eq!(edits[0].range.start, at_nth(text, ", 1", 0, 0));
}
