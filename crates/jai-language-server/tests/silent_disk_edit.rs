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
