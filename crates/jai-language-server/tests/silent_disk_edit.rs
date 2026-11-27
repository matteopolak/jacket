//! A file changed on disk without any notification (a client without a watcher, a tool that
//! rewrites in place) must not leave answers that mix cached offsets with the new text.
use test_support::lsp::{project, uri_of, workspace_session};

#[test]
fn workspace_symbols_of_a_closed_file_survive_a_silent_edit() {
    let dir = project(
        "silent-edit",
        &[
            ("main.jai", "main :: () {}\n"),
            ("util.jai", "wanted_name :: () {}\n"),
        ],
    );
    let mut s = workspace_session(&dir);
    let main = uri_of(&dir.join("main.jai"));
    s.open(main, 1, "main :: () {}\n".into()).unwrap();
    let before = s.workspace_symbols("wanted_name");
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].location.range.start.line, 0);
    // Rewritten behind the server's back: the declaration moves down three lines.
    std::fs::write(
        dir.join("util.jai"),
        "// one\n// two\n// three\nwanted_name :: () {}\n",
    )
    .unwrap();
    let after = s.workspace_symbols("wanted_name");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].location.range.start.line, 3, "{after:?}");
    assert_eq!(after[0].location.range.start.character, 0);
}

/// An open document the parser gives up on (nested past its budget) lists the declarations of
/// its last text that parsed, at the place they are in the text now.
#[test]
fn workspace_symbols_of_an_open_file_that_does_not_parse_are_where_they_are_now() {
    use jai_language_server::TextChange;
    let dir = project("unparsed-open", &[("main.jai", "main :: () {}\n")]);
    let mut s = workspace_session(&dir);
    let main = uri_of(&dir.join("main.jai"));
    let text = "main :: () {}\nwanted_name :: () {}\n";
    s.open(main.clone(), 1, text.into()).unwrap();
    // Three lines added above, the last of them too deeply nested to parse.
    let edit = TextChange {
        range: None,
        range_length: None,
        text: format!("// one\n// two\ndeep :: {};\n{text}", "(".repeat(200)),
    };
    s.change(&main, 2, &[edit]).unwrap();
    let found = s.workspace_symbols("wanted_name");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location.range.start.line, 4, "{found:?}");
}
