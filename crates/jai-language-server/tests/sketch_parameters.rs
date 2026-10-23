//! Go to definition on the type parameters of polymorphs nothing instantiates.
mod common;
use common::{at, at_last, open, uri};

/// `open` instantiates `Box` with its own `$T` while it is sketched; the `T` in `Box`'s body
/// still names `Box`'s parameter.
#[test]
fn a_struct_parameter_is_its_own_even_when_another_sketch_instantiates_it() {
    let program = "Box :: struct ($T: Type) { v: T; }
open :: (box: Box($T)) -> T { return box.v; }
main :: () {}
";
    let s = open(program);
    let field = s.definition(&uri(), at(program, "T; }")).unwrap();
    assert_eq!(field.len(), 1, "{field:?}");
    assert_eq!(field[0].range.start.line, 0, "{field:?}");
    let result = s.definition(&uri(), at_last(program, "T {")).unwrap();
    assert_eq!(result.len(), 1, "{result:?}");
    assert_eq!(result[0].range.start, at(program, "T)) ->"), "{result:?}");
}
