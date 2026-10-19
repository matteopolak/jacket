use super::*;

fn doc(text: &str, name: &str) -> Option<String> {
    declaration(text, text.find(name).unwrap())
}

#[test]
fn line_comments_directly_above_are_the_docs() {
    let text = "x :: 1;\n\n// First.\n//  Indented *md*.\n//\n// Last.\nf :: () {}\n";
    assert_eq!(
        doc(text, "f ::").unwrap(),
        "First.\n Indented *md*.\n\nLast."
    );
    let gap = "// Not docs.\n\nf :: () {}\n";
    assert_eq!(doc(gap, "f ::"), None);
    let rule = "// ------\nf :: () {}\n";
    assert_eq!(doc(rule, "f ::"), None);
}

#[test]
fn block_comments_are_docs() {
    let text = "/* Adds.\n * Twice.\n */\nf :: () {}\n";
    assert_eq!(doc(text, "f ::").unwrap(), "Adds.\nTwice.");
    let one = "/* One line. */\nf :: () {}\n";
    assert_eq!(doc(one, "f ::").unwrap(), "One line.");
}

#[test]
fn fields_and_members_take_trailing_comments() {
    let text = "P :: struct {\n    x: int; // the x\n    // the y\n    y: int;\n}\n";
    assert_eq!(doc(text, "x:").unwrap(), "the x");
    assert_eq!(doc(text, "y:").unwrap(), "the y");
    assert_eq!(doc("f :: () { // not docs\n}\n", "f ::"), None);
}

#[test]
fn parameters_use_their_own_comments_or_the_list() {
    let text = "\
// Does it.
// - a: the first
// - `b` — the second
//   continued
f :: (a: int, b: int, /* c */ c: int) {}
g :: (
// the left
left: int,
right: int, // the right
) {}
";
    let at = |n: &str| text.find(n).unwrap();
    assert_eq!(declaration(text, at("a: int")).unwrap(), "the first");
    assert_eq!(
        declaration(text, at("b: int")).unwrap(),
        "the second\ncontinued"
    );
    assert_eq!(declaration(text, at("left")).unwrap(), "the left");
    assert_eq!(declaration(text, at("right")).unwrap(), "the right");
    let open = header_open(text, at("g ::")).unwrap();
    assert_eq!(
        parameters(text, open),
        [
            ("left".to_owned(), "the left".to_owned()),
            ("right".to_owned(), "the right".to_owned())
        ]
    );
}

#[test]
fn locals_are_not_parameters() {
    let text = "// Doc.\nf :: (a: int) {\n    local := 1;\n}\n";
    assert_eq!(enclosing_header(text, text.find("local").unwrap()), None);
    assert!(enclosing_header(text, text.find("a:").unwrap()).is_some());
}

#[test]
fn links_in_prose() {
    let names = |s: &str| -> Vec<Vec<String>> { links(s).into_iter().map(|l| l.names).collect() };
    assert_eq!(
        names("see [print] and [`Basic.print`]"),
        [vec!["print"], vec!["Basic", "print"]]
    );
    assert_eq!(
        names("[the text](Thing.x) `[code]` a[i] [0] [url](http://x)"),
        [vec!["Thing", "x"]]
    );
    assert_eq!(names("![img](x) [a b]"), Vec::<Vec<String>>::new());
}

#[test]
fn fenced_code_is_the_jai_blocks_of_comments() {
    let text = "// Use it:\n//\n// ```\n// x := f(1);\n// ```\n// ```c\n// int x;\n// ```\n/// ```jai\n/// y := 2;\n/// ```\nf :: (a: int) {}\n";
    let code: Vec<&str> = fenced_code(text)
        .into_iter()
        .map(|(s, e)| &text[s..e])
        .collect();
    assert_eq!(code, [" x := f(1);", " y := 2;"]);
}

#[test]
fn markdown_tags_fences_and_links() {
    let resolve = |names: &[String]| (names == ["print"]).then(|| "file:///a.jai#L3".to_owned());
    let doc = "See [`print`] and [x].\n\n```\nlet [print]\n```\n\n```c\nint x;\n```";
    assert_eq!(
        markdown(doc, &resolve),
        "See [`print`](file:///a.jai#L3) and [x].\n\n```jai\nlet [print]\n```\n\n```c\nint x;\n```"
    );
}

#[test]
fn targeted_links_include_their_closing_parenthesis() {
    let resolve = |names: &[String]| (names == ["print"]).then(|| "file:///a.jai#L3".to_owned());
    assert_eq!(
        markdown("a [the text](print) b", &resolve),
        "a [the text](file:///a.jai#L3) b"
    );
    let found = links("a [`x`](Basic.print) b");
    assert_eq!(
        &"a [`x`](Basic.print) b"[found[0].whole.0..found[0].whole.1],
        "[`x`](Basic.print)"
    );
    assert_eq!(
        &"a [`x`](Basic.print) b"[found[0].path.0..found[0].path.1],
        "Basic.print"
    );
}

#[test]
fn link_paths_exclude_code_marks() {
    let line = "[ `Basic.print()` ] [print()]";
    let found = links(line);
    let path = |i: usize| &line[found[i].path.0..found[i].path.1];
    assert_eq!((path(0), path(1)), ("Basic.print", "print"));
}

#[test]
fn meant_links_are_told_from_prose() {
    let text = "// [print] [`print`] [TODO] [x] [Type.field] [a](b) [ref] and [guide][ref]\n// [ref]: https://example.com\nf :: () {}\n// [print]\nx = 1;\n// [print]\nenum_member,\n";
    let found: Vec<(String, bool, bool)> = sites(text)
        .into_iter()
        .map(|s| {
            (
                text[s.whole.0..s.whole.1].to_owned(),
                s.meant,
                s.on_declaration,
            )
        })
        .collect();
    let meant = |what: &str| found.iter().find(|f| f.0 == what).map(|f| f.1);
    assert_eq!(meant("[print]"), Some(true));
    assert_eq!(meant("[`print`]"), Some(true));
    assert_eq!(meant("[TODO]"), Some(false));
    assert_eq!(meant("[x]"), Some(false));
    assert_eq!(meant("[Type.field]"), Some(true));
    assert_eq!(meant("[a](b)"), Some(true));
    assert_eq!(meant("[ref]"), Some(false));
    // Above a statement, then above an enum member.
    assert!(!found[found.len() - 2].2);
    assert!(found[found.len() - 1].2);
}

#[test]
fn stale_offsets_answer_nothing() {
    // Completion data and indexes carry offsets from before an edit: past the end of the text
    // now, or inside a multibyte character.
    let text = "// docs\nf :: (a: int) {}\n\u{e9}";
    let inside = text.len() - 1;
    for at in [text.len() + 100, inside] {
        assert_eq!(declaration(text, at), None);
        assert_eq!(above(text, at), None);
        assert_eq!(trailing(text, at), None);
        assert_eq!(enclosing_header(text, at), None);
        assert_eq!(header_open(text, at), None);
        assert!(parameters(text, at).is_empty());
        assert!(site_at(text, at).is_none());
        assert!(signature_line(text, at).is_empty());
    }
}

#[test]
fn declaration_lines() {
    for yes in [
        "f :: () {",
        "x := 1;",
        "x: int;",
        "a, b: int;",
        "RED,",
        "using s: S;",
        "$T: Type",
    ] {
        assert!(declaration_line(yes), "{yes}");
    }
    for no in ["x = 1;", "f(x);", "return a;", "case X:", "}", "if a {"] {
        assert!(!declaration_line(no), "{no}");
    }
}

#[test]
fn sites_cover_runs_blocks_and_trailing_comments() {
    let text = "// Uses [helper].\nf :: () {}\nx: int; // see [Mode.IDLE]\n\n// stray [nothing]\n\ng :: 1;\n";
    let found = sites(text);
    let labels: Vec<&str> = found.iter().map(|s| &text[s.label.0..s.label.1]).collect();
    assert_eq!(labels, ["helper", "Mode.IDLE"]);
    assert_eq!(found[0].scope_at, text.find("f ::").unwrap());
    assert!(site_at(text, text.find("helper").unwrap() + 2).is_some());
    assert!(site_at(text, text.find("stray").unwrap()).is_none());
}

#[test]
fn markdown_escapes_and_block_comment_links_map_back_to_the_file() {
    let found = links("not \\[a link\\] but [one]");
    assert_eq!(found.len(), 1);
    let text = "/* Block with [helper].\n * More [Mode.IDLE].\n */\nf :: () {}\n";
    let labels: Vec<&str> = sites(text)
        .iter()
        .map(|s| &text[s.label.0..s.label.1])
        .collect();
    assert_eq!(labels, ["helper", "Mode.IDLE"]);
}
