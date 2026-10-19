//! A name the project does not declare (a built-in type, a procedure of an imported module) cannot
//! be renamed: the edit would break every use, or rewrite a module the project does not own.
use jai_language_server::Session;
mod common;
use common::{at_nth, session, uri};

const PROGRAM: &str = r#"#import "Basic";

Point :: struct {
    x: int;
}

main :: () {
    p: Point;
    p.x = 3;
    print("%\n", p.x);
}
"#;

#[test]
fn only_names_the_project_declares_can_be_renamed() {
    let mut s: Session = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    for (needle, width) in [("int;", 3), ("print(", 5), ("Basic", 5)] {
        let at = at_nth(PROGRAM, needle, 0, 1);
        assert!(
            s.prepare_rename(&uri(), at).unwrap().is_none(),
            "`{}` can be renamed",
            &needle[..width]
        );
        assert!(s.rename(&uri(), at, "other").unwrap().is_none());
    }
    // The project's own declarations still can.
    for needle in ["Point;", "p: Point", "x: int"] {
        let at = at_nth(PROGRAM, needle, 0, 1);
        assert!(s.prepare_rename(&uri(), at).unwrap().is_some(), "{needle}");
    }
}

#[test]
fn a_parameter_bound_to_a_built_in_type_does_not_make_it_renamable() {
    // `T` is declared at `Box(s64)`'s instance with the value `s64`.
    let source = "Box :: struct (T: Type) {\n    v: s64;\n}\n\nmain :: () {\n    b: Box(s64);\n}\n";
    let mut s = session();
    s.open(uri(), 1, source.into()).unwrap();
    assert!(
        s.prepare_rename(&uri(), at_nth(source, "s64;", 0, 1))
            .unwrap()
            .is_none()
    );
}
