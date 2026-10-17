//! Format strings are found through the callee's `@PrintLike` note, not its name: the `%`
//! directives of the literal are format specifiers, hovering one names its argument, and a
//! count that does not match the arguments is a warning.
use jai_language_server::{
    DiagnosticCode, DiagnosticSeverity, DocumentUri, Environment, Limits, Position,
    SemanticTokenKind, Session,
};
use std::path::PathBuf;

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join(jaic::PRELOAD_ENTRY));
            options
        }),
    }
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-print-like/main.jai").unwrap()
}

fn at(text: &str, marker: &str, offset: usize) -> Position {
    let byte = text.find(marker).unwrap_or_else(|| panic!("no {marker:?}")) + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

/// `shout` is marked, `plain` has the same signature but no note. Neither forwards its
/// parameters, so only the note tells them apart.
const SOURCE: &str = "#import \"Basic\";\n\
    shout :: (what: string, parts: ..Any) -> int { return parts.count; } @PrintLike\n\
    plain :: (what: string, parts: ..Any) -> int { return parts.count; }\n\
    main :: () {\n\
    \x20   a := shout(\"x % y %\", 1, 2);\n\
    \x20   b := shout(\"only %\", 1, 2);\n\
    \x20   c := plain(\"p % q %\", 1);\n\
    \x20   print(\"lib %\\n\", a + b + c);\n\
    \x20   print(\"two %%\\n\", 1);\n\
    }\n";

fn specifiers(s: &Session, text: &str) -> Vec<(Position, u32)> {
    let _ = text;
    s.semantic_tokens(&uri())
        .unwrap()
        .into_iter()
        .filter(|t| t.kind == SemanticTokenKind::FormatSpecifier)
        .map(|t| (t.position, t.length))
        .collect()
}

#[test]
fn directives_follow_the_print_like_note_not_the_name() {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(uri(), 1, SOURCE.into()).unwrap();
    let found = specifiers(&s, SOURCE);
    let mark = |line_marker: &str, nth: usize| {
        let base = SOURCE.find(line_marker).unwrap();
        let rel = SOURCE[base..]
            .match_indices('%')
            .nth(nth)
            .map(|(i, _)| i)
            .unwrap();
        at(SOURCE, line_marker, rel)
    };
    // The two `%` of the noted `shout` and the one of the library's `print` (itself noted).
    assert!(
        found.contains(&(mark("shout(\"x % y %\"", 0), 1)),
        "{found:?}"
    );
    assert!(
        found.contains(&(mark("shout(\"x % y %\"", 1), 1)),
        "{found:?}"
    );
    assert!(found.contains(&(mark("print(\"lib %", 0), 1)), "{found:?}");
    // `plain` has no note: its literal is just a string.
    assert!(
        !found.contains(&(mark("plain(\"p % q %\"", 0), 1)),
        "{found:?}"
    );
    // `%%` is two directives in a row, as `Basic` reads it.
    assert!(found.contains(&(mark("print(\"two %%", 0), 1)), "{found:?}");
    assert!(found.contains(&(mark("print(\"two %%", 1), 1)), "{found:?}");
}

#[test]
fn count_mismatches_warn_only_for_print_like_callees() {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(uri(), 1, SOURCE.into()).unwrap();
    let warnings: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.code == DiagnosticCode::Check && d.severity == DiagnosticSeverity::Warning)
        .collect();
    let lines: Vec<u32> = warnings.iter().map(|d| d.range.start.line).collect();
    // `shout("x % y %", 1, 2)` is right; `shout("only %", 1, 2)` and the library's
    // `print("two %%", 1)` are not; `plain` is not a formatter.
    assert!(lines.contains(&5), "{warnings:?}");
    assert!(lines.contains(&8), "{warnings:?}");
    assert!(!lines.contains(&4), "{warnings:?}");
    assert!(!lines.contains(&6), "{warnings:?}");
    assert!(
        warnings
            .iter()
            .any(|d| d.range.start.line == 5 && d.message.contains("`shout`")),
        "{warnings:?}"
    );
}

#[test]
fn hovering_a_directive_names_its_argument() {
    let mut s = Session::with_environment(Limits::default(), environment());
    s.open(uri(), 1, SOURCE.into()).unwrap();
    let base = SOURCE.find("shout(\"x % y %\"").unwrap();
    let second = base + SOURCE[base..].match_indices('%').nth(1).unwrap().0;
    let prefix = &SOURCE[..second];
    let position = Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    };
    let hover = s
        .hover(&uri(), position)
        .unwrap()
        .expect("hover")
        .contents
        .value;
    assert!(hover.contains('2'), "{hover}");
    // And a directive of an unmarked callee has no format hover.
    let base = SOURCE.find("plain(\"p % q %\"").unwrap();
    let prefix = &SOURCE[..base + 9];
    let position = Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    };
    let hover = s.hover(&uri(), position).unwrap();
    assert!(
        hover.is_none_or(|h| !h.contents.value.contains("No format arguments")
            && !h.contents.value.contains("missing argument")),
        "plain callee must not get a format hover"
    );
}

#[test]
fn without_a_compile_the_documents_own_declarations_decide() {
    let mut s = Session::new(Limits::default());
    let text = "shout :: (what: string, parts: ..Any) { } @PrintLike\n\
        plain :: (what: string, parts: ..Any) { }\n\
        main :: () {\n    shout(\"a % b\", 1);\n    plain(\"a % b\", 1);\n}\n";
    s.open(uri(), 1, text.into()).unwrap();
    let found = specifiers(&s, text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0.line, 3);
}
