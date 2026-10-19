//! A parameter is renamed in the `#modify` block that reads it, as in the body.
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r#"positive :: ($n: int) -> int #modify {
    return n > 0, "n must be positive";
} {
    return n;
}

twice :: ($$k: int) -> int #modify {
    return k < 100, "too large";
} {
    return k * 2;
}

main :: () {
    positive(3);
    twice(4);
}
"#;

#[test]
fn a_parameter_is_renamed_in_its_modify_block() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    for (name, needle) in [("n", "n: int"), ("k", "k: int")] {
        let at = at_nth(PROGRAM, needle, 0, 0);
        let edits = s.rename(&uri(), at, "m").unwrap().expect("rename");
        let lines: Vec<u32> = edits[0].1.iter().map(|e| e.range.start.line).collect();
        let expected: &[u32] = if name == "n" {
            &[0, 1, 3]
        } else {
            &[6, 7, 9]
        };
        assert_eq!(lines, expected, "{name}");
    }
}
