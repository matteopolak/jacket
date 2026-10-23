//! Go to definition on members reached through `using` paths and on declarations that
//! `#insert`ed text generates.
mod common;
use common::{cursor, file_uri, lean_session};
use jai_language_server::Location;

fn definition(source: &str) -> (Vec<Location>, String) {
    let (text, at) = cursor(source);
    let mut session = lean_session();
    let uri = file_uri("/definition-generated/main.jai");
    session.open(uri.clone(), 1, text.clone()).unwrap();
    (session.definition(&uri, at).unwrap(), text)
}

/// The text of the line `location` starts on.
fn line_of(location: &Location, text: &str) -> String {
    text.lines()
        .nth(location.range.start.line as usize)
        .unwrap()
        .to_owned()
}

#[test]
fn a_member_reached_through_a_using_path() {
    let (found, text) = definition(
        "Desc :: struct { width: int; height: int; }\n\
         Tex :: struct { desc: Desc; }\n\
         Tex2D :: struct { texture: Tex; using,only(width, height) texture.desc; }\n\
         main :: () { t: Tex2D; t.|height = 7; }",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        line_of(&found[0], &text).starts_with("Desc ::"),
        "{found:?}"
    );
}

#[test]
fn a_field_an_insert_in_a_struct_body_generates() {
    let (found, text) = definition(
        "Fields :: struct {\n\
             #insert \"alpha: int; beta: int;\";\n\
         }\n\
         main :: () { f: Fields; f.|alpha = 3; }",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(line_of(&found[0], &text).contains("#insert"), "{found:?}");
}

#[test]
fn a_procedure_a_file_scope_insert_generates() {
    let (found, text) = definition(
        "generate :: () -> string { return \"update :: (x: int) {}\"; }\n\
         #insert #run generate();\n\
         main :: () { |update(1); }",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(line_of(&found[0], &text).contains("#insert"), "{found:?}");
}
