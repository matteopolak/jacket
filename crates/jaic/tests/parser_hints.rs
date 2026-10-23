//! Syntax errors carry the edit that fixes them when it is clear: a missing `;`, a missing `}`,
//! an escape that is not one, a comment or here-string left open. Each test applies the edit
//! and compiles the result to show it was the fix.
use jaic::sema::{Compiler, Options, VirtualFs};
use jaic::source::{Applicability, Diagnostic};
use std::{path::Path, rc::Rc};

fn compile(source: &str) -> Result<(), Box<Diagnostic>> {
    let mut fs = VirtualFs::default();
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.compile_program(Path::new("/main.jai"))
}

fn error_of(source: &str) -> Diagnostic {
    match compile(source) {
        Err(d) => *d,
        Ok(()) => panic!("expected an error in:\n{source}"),
    }
}

/// The source with the edits of suggestion `index` applied.
fn applied(source: &str, d: &Diagnostic, index: usize) -> String {
    let mut edits: Vec<_> = d.suggestions()[index]
        .replacements()
        .map(|(span, text)| (span.start as usize, span.end as usize, text.to_string()))
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = source.to_string();
    for (start, end, text) in edits {
        out.replace_range(start..end, &text);
    }
    out
}

/// Fix `source`: the message begins with `message`, the first suggestion has `applicability`,
/// and applying it gives a program that compiles. Returns the fixed source.
#[track_caller]
fn fixes(source: &str, message: &str, applicability: Applicability) -> String {
    let d = error_of(source);
    assert!(d.message.starts_with(message), "{}", d.message);
    assert_eq!(d.suggestions()[0].applicability, applicability);
    let fixed = applied(source, &d, 0);
    if let Err(e) = compile(&fixed) {
        panic!("the fix leaves an error: {}\n{fixed}", e.message);
    }
    fixed
}

#[test]
fn a_missing_semicolon_before_a_new_line_is_inserted_after_the_expression() {
    let source = "main :: () {\n    x := 1\n    y := 2;\n}\n";
    let fixed = fixes(
        source,
        "expected `;` after statement",
        Applicability::MachineApplicable,
    );
    assert_eq!(fixed, "main :: () {\n    x := 1;\n    y := 2;\n}\n");
}

#[test]
fn a_missing_semicolon_before_a_closing_brace_is_machine_applicable() {
    let source = "main :: () {\n    x := 1 }\n";
    let fixed = fixes(source, "expected `;`", Applicability::MachineApplicable);
    assert_eq!(fixed, "main :: () {\n    x := 1; }\n");
}

#[test]
fn a_missing_semicolon_in_the_middle_of_a_line_is_a_guess() {
    let source = "main :: () {\n    x := 1 y := 2;\n}\n";
    fixes(source, "expected `;`", Applicability::MaybeIncorrect);
}

#[test]
fn a_missing_semicolon_after_case_values_is_inserted() {
    let source =
        "main :: () {\n    x := 1;\n    if x == {\n        case 1\n            x = 2;\n    }\n}\n";
    let d = error_of(source);
    assert!(d.message.contains("after the case values"), "{}", d.message);
    let fixed = applied(source, &d, 0);
    assert!(fixed.contains("case 1;"), "{fixed}");
    compile(&fixed).unwrap_or_else(|e| panic!("{}", e.message));
}

#[test]
fn an_unclosed_block_points_at_its_brace_and_closes_at_the_end() {
    let source = "main :: () {\n    x := 1;\n";
    let d = error_of(source);
    assert!(d.message.starts_with("expected `}` to close the block"));
    assert_eq!(d.notes.len(), 1);
    assert_eq!(d.notes[0].1, "the block opens here");
    assert_eq!(
        &source[d.notes[0].0.start as usize..d.notes[0].0.end as usize],
        "{"
    );
    let fixed = applied(source, &d, 0);
    assert_eq!(fixed, "main :: () {\n    x := 1;\n}\n");
    compile(&fixed).unwrap_or_else(|e| panic!("{}", e.message));
}

#[test]
fn an_unknown_escape_is_fixed_by_doubling_the_backslash() {
    let source = "main :: () {\n    s := \"a\\qb\";\n}\n";
    let fixed = fixes(
        source,
        "unknown escape sequence",
        Applicability::MaybeIncorrect,
    );
    assert_eq!(fixed, "main :: () {\n    s := \"a\\\\qb\";\n}\n");
}

#[test]
fn an_unterminated_block_comment_is_closed_at_the_end() {
    let source = "main :: () {\n}\n/* the rest\n";
    let fixed = fixes(
        source,
        "unterminated block comment",
        Applicability::MaybeIncorrect,
    );
    assert!(fixed.ends_with("/* the rest\n*/\n"), "{fixed}");
}

#[test]
fn an_unterminated_here_string_gets_its_terminator() {
    let source = "main :: () {\n    s := #string END\nhello\n";
    let d = error_of(source);
    assert!(
        d.message.starts_with("unterminated here-string"),
        "{}",
        d.message
    );
    let fixed = applied(source, &d, 0);
    assert_eq!(fixed, "main :: () {\n    s := #string END\nhello\nEND\n");
    // The terminator is the whole fix: the string now ends, and the rest of `main` follows.
    let closed = format!("{fixed};\n}}\n");
    compile(&closed).unwrap_or_else(|e| panic!("{}", e.message));
}
