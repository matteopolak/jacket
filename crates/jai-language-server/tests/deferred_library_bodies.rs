//! The editor check leaves the bodies of non-generic stdlib procedures it cannot observe
//! unchecked (`Session::defer_library_bodies`). Nothing the user sees may differ.
use jai_language_server::{DocumentUri, Position, Session, Stage};
use test_support::lsp::{session, uri};

/// Programs that reach the stdlib in different ways.
const PROGRAMS: &[&str] = &[
    // Calls, a stdlib type, formatting.
    "#import \"Basic\";\nmain :: () {\n    xs: [..] int;\n    array_add(*xs, 1);\n    for xs print(\"%\\n\", it);\n    s := tprint(\"%\", xs.count);\n    print(s);\n}\n",
    // A user error next to stdlib calls.
    "#import \"Basic\";\nmain :: () {\n    xs: [..] int;\n    array_add(*xs, \"text\");\n    n: int = xs.count;\n    print(\"%\\n\", n);\n}\n",
    // A lint that follows a stdlib callee (a pointer kept across an append).
    "#import \"Basic\";\nmain :: () {\n    xs: [..] int;\n    array_add(*xs, 1);\n    p := *xs[0];\n    array_add(*xs, 2);\n    print(\"%\\n\", p.*);\n}\n",
    // Compile-time code that runs a stdlib procedure.
    "#import \"Basic\";\nmake :: () -> int {\n    xs: [..] int;\n    array_add(*xs, 4);\n    array_add(*xs, 5);\n    return xs.count;\n}\nN :: #run make();\n#assert N == 2;\nmain :: () { print(\"%\\n\", N); }\n",
    // Math, strings and a procedure value.
    "#import \"Basic\";\n#import \"Math\";\n#import \"String\";\nmain :: () {\n    r := sqrt(4.0);\n    print(\"%\\n\", r);\n    t := to_lower_copy(\"ABC\");\n    print(\"%\\n\", t);\n}\n",
    // A body using a name another body declares through `#insert,scope`.
    include_str!("../../../tests/compiler/regressions/body-waits-for-inserted-declaration.jai"),
    // A struct laid out while `using` a global pointer to a struct that holds it.
    include_str!("../../../tests/compiler/regressions/using-global-of-own-struct.jai"),
    // Polymorphs, baked arguments and polymorphic structs.
    include_str!("../../../tests/compiler/regressions/rules-polymorphism.jai"),
    // A metaprogram that sees the stdlib bodies its workspace checks.
    include_str!("../../../tests/compiler/metaprograms/intercepted-stdlib-body-from-string.jai"),
];

fn open(defer: bool, text: &str) -> Session {
    let mut s = session();

    s.defer_library_bodies(defer);
    s.open(
        DocumentUri::parse("file:///lsp-test/jacket.toml").unwrap(),
        1,
        "[lint.rules]\npointer_lifetime = \"warn\"\npointer_into_resizable = \"warn\"\n".into(),
    )
    .unwrap();
    s.open(uri(), 1, text.into()).unwrap();
    s
}

/// What the editor answers at the start of every word.
fn answers(s: &Session, text: &str) -> String {
    let mut out = format!("{:?}\n", s.diagnostics(&uri()));
    let lines: Vec<&str> = text.lines().collect();

    for (line, content) in lines.iter().enumerate() {
        let mut word_start = None;

        for (at, c) in content
            .char_indices()
            .chain(std::iter::once((content.len(), ' ')))
        {
            if c.is_alphanumeric() || c == '_' {
                word_start.get_or_insert(at);
            } else if let Some(start) = word_start.take() {
                let position = Position {
                    line: line as u32,
                    character: start as u32,
                };

                out += &format!(
                    "{line}:{start} {:?} {:?}\n",
                    s.hover(&uri(), position),
                    s.definition(&uri(), position)
                );
            }
        }
    }

    out
}

#[test]
fn deferring_changes_no_answer() {
    for (n, text) in PROGRAMS.iter().enumerate() {
        let with = answers(&open(true, text), text);
        let without = answers(&open(false, text), text);

        assert_eq!(with, without, "program {n}");
    }
}

#[test]
fn compile_time_code_still_runs_stdlib_procedures() {
    let text = PROGRAMS[3];
    let s = open(true, text);

    s.staged_unless_interrupted(Stage::Compile).unwrap();
    // The run asked for the bodies it calls, before any lint did.
    assert!(s.deferred_library_bodies().1 > 0);
    let diagnostics = s.diagnostics(&uri()).unwrap();

    assert!(
        diagnostics
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error),
        "{diagnostics:?}"
    );
}

#[test]
fn lints_follow_stdlib_callees() {
    let s = open(true, PROGRAMS[2]);
    let diagnostics = s.diagnostics(&uri()).unwrap();

    assert!(
        diagnostics
            .iter()
            .any(|d| matches!(d.code, jai_language_server::DiagnosticCode::Lint(_))),
        "{diagnostics:?}"
    );
}

#[test]
fn the_compile_leaves_stdlib_bodies_unchecked_until_something_asks() {
    let on = open(true, PROGRAMS[0]);
    let off = open(false, PROGRAMS[0]);

    on.staged_unless_interrupted(Stage::Compile).unwrap();
    off.staged_unless_interrupted(Stage::Compile).unwrap();
    let (deferred, promoted) = on.deferred_library_bodies();

    assert!(deferred > 0, "{deferred} bodies deferred");
    assert_eq!(promoted, 0, "nothing needed one yet");
    assert_eq!(off.deferred_library_bodies(), (0, 0));
    // The lints read what the code calls: they ask for every body.
    on.staged_unless_interrupted(Stage::Full).unwrap();
    let (left, promoted) = on.deferred_library_bodies();

    assert_eq!(left, 0);
    assert!(promoted >= deferred, "{promoted} of {deferred}");
}

#[test]
fn a_library_file_open_in_the_editor_is_checked_whole() {
    let mut s = session();

    s.defer_library_bodies(true);
    let path = test_support::lsp::stdlib().join("Basic/module.jai");
    let text = std::fs::read_to_string(&path).unwrap();
    let library = test_support::lsp::uri_of(&path);

    s.open(library.clone(), 1, text).unwrap();
    s.staged_unless_interrupted(Stage::Compile).unwrap();
    assert_eq!(s.deferred_library_bodies().0, 0);
}
