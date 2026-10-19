//! Renames take any name the lexer reads as one identifier, non-ASCII characters included.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = "main :: () {
    größe := 1;
    größe += 2;
}
";

#[test]
fn a_name_with_non_ascii_characters_renames_both_ways() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let at = at_nth(PROGRAM, "größe", 0, 0);
    for name in ["→size", "ß", "size·2"] {
        let edits = s.rename(&uri(), at, name).unwrap().expect("rename");
        assert_eq!(edits[0].1.len(), 2, "{name}: {edits:?}");
    }
    for name in ["2size", "a b", "a-b", "if"] {
        assert!(s.rename(&uri(), at, name).is_err(), "{name}");
    }
}
