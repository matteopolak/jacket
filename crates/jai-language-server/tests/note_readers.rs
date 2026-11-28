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

const TYPE_INFO: &str = r#"#import "Basic";
Config :: struct @Saved {
    port: int @json ;
    host: string @toml ;
}
#run {
    info := type_info(Config);
    for info.notes if it == "Saved" print("saved\n");
    for member: info.members {
        for note: member.notes if note == "json" print("json %\n", member.name);
    }
}
main :: () {}
"#;

/// The zero-based lines in `main.jai` that `definition` gives for the note at `marker`, each once.
fn readers_here(
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
        .filter(|l| l.uri.as_str().ends_with("main.jai"))
        .map(|l| l.range.start.line)
        .filter(|&line| line != text[..at].matches('\n').count() as u32)
        .collect();

    lines.sort_unstable();
    lines.dedup();
    lines
}

#[test]
fn notes_read_through_type_info_lead_to_their_readers() {
    let dir = project("note-readers-type-info", &[("main.jai", TYPE_INFO)]);
    let (s, uri, text) = open_file(&dir, "main.jai");
    let line = |marker: &str| {
        TYPE_INFO[..TYPE_INFO.find(marker).unwrap()]
            .matches('\n')
            .count() as u32
    };

    assert_eq!(
        readers_here(&s, &uri, &text, "@json"),
        vec![line("for note: member.notes")]
    );
    assert_eq!(
        readers_here(&s, &uri, &text, "@Saved"),
        vec![line("for info.notes")]
    );
    // `== "json"` compared `toml` and found it different: not a read.
    assert_eq!(readers_here(&s, &uri, &text, "@toml"), Vec::<u32>::new());
}

/// A note with arguments, found by its name: the prefix test reads it, whatever the arguments.
const WITH_ARGUMENTS: &str = r#"#import "Basic";
#import "String";
Config :: struct {
    port: int; @json(rename = "Port")
    host: string; @toml(rename = "Host")
}
#run {
    for member: type_info(Config).members {
        for note: member.notes if begins_with(note, "json(") print("%\n", note);
    }
}
main :: () {}
"#;

#[test]
fn a_note_with_arguments_is_read_by_a_test_of_its_name() {
    let dir = project("note-readers-arguments", &[("main.jai", WITH_ARGUMENTS)]);
    let (s, uri, text) = open_file(&dir, "main.jai");
    let line = |marker: &str| {
        WITH_ARGUMENTS[..WITH_ARGUMENTS.find(marker).unwrap()]
            .matches('\n')
            .count() as u32
    };

    assert_eq!(
        readers_here(&s, &uri, &text, "@json"),
        vec![line("for note: member.notes")]
    );
    assert_eq!(readers_here(&s, &uri, &text, "@toml"), Vec::<u32>::new());
}
