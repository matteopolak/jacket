//! Go to definition on a note leads to the compile-time code that read it: here a build
//! metaprogram that intercepts the workspace compiling `main.jai` and looks at the notes of
//! its procedure headers.
use jai_language_server::Session;
use test_support::lsp::{open_file, position, project};

const FIRST: &str = r#"#import "Basic";
#import "Compiler";
#run build();
build :: () {
    set_build_options_dc(.{do_output = false});
    w := compiler_create_workspace("app");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_file("main.jai", w);
    seen := 0;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .TYPECHECKED {
            typechecked := cast(*Message_Typechecked) message;
            for item: typechecked.procedure_headers {
                for note: item.expression.notes {
                    if note.text == "Traced" print("traced %\n", item.expression.name);
                    if note.text.count == 5 && note.text[0] == #char "S" print("short %\n", item.expression.name);
                    if note.text == "Traced" seen += 1;
                }
            }
        }
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}
"#;

const MAIN: &str = "traced :: () {} @Traced\nsized :: () {} @Sized\nignored :: () {} @Unrelated\n\
                    main :: () { traced(); sized(); ignored(); }\n";

/// The (line, character) of each place `definition` gives for the note at `marker`, in
/// `first.jai` only.
fn readers(
    s: &Session,
    uri: &jai_language_server::DocumentUri,
    text: &str,
    marker: &str,
) -> Vec<u32> {
    let at = text.find(marker).unwrap() + 1;
    let mut lines: Vec<u32> = s
        .definition(uri, position(text, at))
        .unwrap()
        .into_iter()
        .filter(|l| l.uri.as_str().ends_with("first.jai"))
        .map(|l| l.range.start.line)
        .collect();
    lines.sort_unstable();
    lines
}

/// The zero-based line of `marker` in `FIRST`.
fn line_of(marker: &str) -> u32 {
    FIRST[..FIRST.find(marker).unwrap()].matches('\n').count() as u32
}

#[test]
fn a_note_leads_to_the_code_that_read_it() {
    let dir = project("note-readers", &[("first.jai", FIRST), ("main.jai", MAIN)]);
    let (s, uri, text) = open_file(&dir, "main.jai");
    // Two `==` compared `@Traced` equal: both are listed. The byte load read `@Sized`.
    assert_eq!(
        readers(&s, &uri, &text, "@Traced"),
        vec![
            line_of("if note.text == \"Traced\" print"),
            line_of("if note.text == \"Traced\" seen")
        ]
    );
    assert_eq!(
        readers(&s, &uri, &text, "@Sized"),
        vec![line_of("if note.text.count == 5")]
    );
    // `@Unrelated` has another length than either: no code looked at its text.
    assert_eq!(readers(&s, &uri, &text, "@Unrelated"), Vec::<u32>::new());
}
