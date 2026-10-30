//! Fields of a struct whose body compile-time code writes (`#insert #run`) per parameter value,
//! the shape a typed query row takes: one type per query text, its fields named by the query.
use jai_language_server::{DocumentUri, Position, Session};
use test_support::lsp::{at_offset, open_file, project};

const PROGRAM: &str = r#"#import "Basic";
fields_for :: (sql: string) -> string {
    if sql == "SELECT id, name FROM users"  return "id: s64; name: string;";
    return "n: s64;";
}
Row :: struct ($sql: string) { #insert #run fields_for(sql); }
Query :: struct ($sql: string) { rows: [..] Row(sql); }
query :: ($sql: string) -> Query(sql) { q: Query(sql); return q; }

main :: () {
    users := query("SELECT id, name FROM users");
    for users.rows  print("%\n", it.na);
    first := users.rows[0];
    print("%\n", first.id);
}
"#;

fn labels(s: &mut Session, uri: &DocumentUri, at: Position) -> Vec<String> {
    s.completion(uri, at)
        .unwrap()
        .items
        .into_iter()
        .map(|i| i.label)
        .collect()
}

#[test]
fn generated_fields_complete_and_hover() {
    let dir = project("generated-fields", &[("main.jai", PROGRAM)]);
    let (mut s, uri, text) = open_file(&dir, "main.jai");

    let after_it = labels(&mut s, &uri, at_offset(&text, "it.na", 3));
    assert!(
        after_it.contains(&"id".to_string()) && after_it.contains(&"name".to_string()),
        "{after_it:?}"
    );

    let after_first = labels(&mut s, &uri, at_offset(&text, "first.id", 6));
    assert!(after_first.contains(&"name".to_string()), "{after_first:?}");

    let hover = s.hover(&uri, at_offset(&text, "first.id", 6)).unwrap();
    let shown = format!("{hover:?}");
    assert!(shown.contains("s64"), "{shown}");
}
