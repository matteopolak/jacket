//! Rename and prepare-rename: which names can be renamed and which uses the edits reach.
//!
//! Cases that differ only in the program, the name under the cursor and the number of edits are
//! rows of [`EDIT_COUNTS`]; those that check where the edits land are rows of [`EDIT_LINES`].
//! The rest need more than one step and stay as tests of their own.
use jai_language_server::Session;
use test_support::lsp::{at_nth, open, session, uri};

/// The text edits of renaming the name at `needle` (cursor `offset` bytes in) to `to`.
fn rename(
    program: &str,
    needle: &str,
    offset: usize,
    to: &str,
) -> Vec<jai_language_server::TextEdit> {
    let s = open(program);
    let at = at_nth(program, needle, 0, offset);
    s.rename(&uri(), at, to)
        .unwrap()
        .expect("rename")
        .remove(0)
        .1
}

/// Program, text the cursor is placed at, the new name, how many places are edited.
const EDIT_COUNTS: &[(&str, &str, &str, usize)] = &[
    // A local named only inside `type_of` in a constant context is a use of it.
    (
        "main :: () {\n    b := 2.5;\n    #assert type_of(b) == float64;\n    #assert type_of(b) != float32;\n}\n",
        "b :=",
        "n",
        3,
    ),
    // A parameter of a polymorphic struct is renamed where the body uses it.
    (STRUCT_PARAMS, "H: int", "N", 2),
    (STRUCT_PARAMS, "W: int", "N", 2),
    (STRUCT_PARAMS, "T: Type", "N", 2),
    // A parameter of a polymorphic procedure reaches the uses in its body.
    (
        "twice :: (x: $T) -> type_of(x) {\n    return x + x;\n}\n\nmain :: () {\n    y := twice(21);\n}\n",
        "x: $T",
        "n",
        4,
    ),
    // A parameter typed with a bare polymorphic struct is renamed in the body.
    (
        "Base :: struct ($Reading: bool) {\n    depth: int;\n}\n\ndepth_of :: (r: Base) -> int {\n    return r.depth;\n}\n\nmain :: () {\n    w: Base(true);\n    x := depth_of(w);\n}\n",
        "r: Base",
        "n",
        2,
    ),
    // Renames take any name the lexer reads as one identifier, non-ASCII characters included.
    (UNICODE, "größe", "→size", 2),
    (UNICODE, "größe", "ß", 2),
    (UNICODE, "größe", "size·2", 2),
    // The names of a `#place` or `#overlay` anchor are uses: a field of the struct, a member
    // reached through it, and a member promoted from a `using` field.
    (ANCHORS, "base: Header;\n    #overlay", "b", 2),
    (ANCHORS, "base: Header;\n    #place", "b", 2),
    (ANCHORS, "table: *u8", "t", 3),
    (ANCHORS, "tag: u32", "t", 2),
];

const ANCHORS: &str = "Header :: struct {\n    tag: u32;\n    table: *u8;\n}\n\nObject :: struct {\n    pad: u64;\n    using base: Header;\n    #overlay(base.table) typed: *u32;\n}\n\nPlaced :: struct {\n    pad: u64;\n    base: Header;\n    #place base.table;\n    other: *u32;\n}\n\nPromoted :: struct {\n    using header: Header;\n    #overlay(tag) small: u32;\n}\n";

const STRUCT_PARAMS: &str = "Grid :: struct (T: Type, W: int, H: int) {\n    cells: [H][W] T;\n}\n\nmain :: () {\n    g: Grid(float32, 2, 3);\n}\n";

const UNICODE: &str = "main :: () {\n    größe := 1;\n    größe += 2;\n}\n";

#[test]
fn edits_reach_every_use() {
    for (program, needle, to, count) in EDIT_COUNTS {
        let edits = rename(program, needle, 0, to);
        assert_eq!(edits.len(), *count, "{needle} -> {to}: {edits:?}");
    }
}

const MODIFY: &str = r#"positive :: ($n: int) -> int #modify {
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

const OVERLOADS: &str = r#"G :: struct { n: int; }
pick :: (it: $T) -> string { return "poly"; }
pick :: (g: G) -> string { return "G"; }
pick :: (x: s64) -> string { return "s64"; }

main :: () {
    g: G;
    a := pick(g);
    i: s64 = 3;
    b := pick(i);
    c := pick(2.5);
}
"#;

const STRUCT_CONSTANTS: &str = r"Map :: struct {
    LIMIT :: 70;
}

Box :: struct {
    m: Map;
}

main :: () {
    a := Map.LIMIT;
    b := Box.m.LIMIT;
    m: Map;
    c := m.LIMIT;
}
";

const ANONYMOUS: &str = r"using _ :: struct {
    LIMIT :: 70;
}

main :: () {
    a := LIMIT;
    b := LIMIT + 1;
}
";

const BAKED: &str = r"Grid :: struct (T: Type, W: int) {
    cells: [W] T;
}

first :: (g: Grid($T, $W)) -> T {
    sum: T;
    return sum;
}

first_f :: #bake_constants first(T = float32);

main :: () {
    small: Grid(float32, 2);
    x := first_f(small);
}
";

/// Program, text the cursor is placed at, the cursor's offset into it, the new name, the line of
/// each edit in the order the server returns them.
const EDIT_LINES: &[(&str, &str, usize, &str, &[u32])] = &[
    // A parameter is renamed in the `#modify` block that reads it, as in the body.
    (MODIFY, "n: int", 0, "m", &[0, 1, 3]),
    (MODIFY, "k: int", 0, "m", &[6, 7, 9]),
    // Renaming one overload renames the whole set: every declaration and every call.
    (OVERLOADS, "pick :: (g", 0, "CAP", &[1, 2, 3, 7, 9, 10]),
    // A constant declared in a struct is found through the type, a field's type or an instance.
    (STRUCT_CONSTANTS, "LIMIT ::", 0, "CAP", &[1, 9, 10, 12]),
    (ANONYMOUS, "LIMIT ::", 0, "CAP", &[1, 5, 6]),
    // `$T`, `-> T`, `sum: T` and `T = float32`, not the struct's own `T`.
    (BAKED, "$T", 1, "E", &[4, 4, 5, 9]),
];

#[test]
fn edits_land_on_the_right_lines() {
    for (program, needle, offset, to, expected) in EDIT_LINES {
        let lines: Vec<u32> = rename(program, needle, *offset, to)
            .iter()
            .map(|e| e.range.start.line)
            .collect();
        assert_eq!(lines, *expected, "{needle} -> {to}");
    }
}

#[test]
fn a_constant_of_an_anonymous_using_struct_has_hover_on_its_uses() {
    let mut s = session();
    s.open(uri(), 1, ANONYMOUS.into()).unwrap();
    let at = at_nth(ANONYMOUS, "LIMIT;", 0, 0);
    let hover = s.hover(&uri(), at).unwrap().expect("hover");
    assert!(hover.contents.value.contains("LIMIT"), "{hover:?}");
}

#[test]
fn names_that_are_not_one_identifier_are_refused() {
    let mut s = session();
    s.open(uri(), 1, UNICODE.into()).unwrap();
    let at = at_nth(UNICODE, "größe", 0, 0);
    for name in ["2size", "a b", "a-b", "if"] {
        assert!(s.rename(&uri(), at, name).is_err(), "{name}");
    }
}

const LOOPS: &str = r"main :: () {
    xs: [3] int;
    for xs print(it);
    for o: xs print(o);
}

print :: (n: int) {}
";

/// A loop's implicit `it` has no declaration to rewrite, and a named loop variable is declared
/// where it is written, not at the first matching letters of the loop.
#[test]
fn only_a_written_loop_variable_can_be_renamed() {
    let mut s = session();
    s.open(uri(), 1, LOOPS.into()).unwrap();
    let implicit = at_nth(LOOPS, "(it)", 0, 1);
    assert!(s.prepare_rename(&uri(), implicit).unwrap().is_none());
    let written = at_nth(LOOPS, "o: xs", 0, 0);
    let edits = s.rename(&uri(), written, "n").unwrap().expect("rename");
    let starts: Vec<_> = edits[0].1.iter().map(|e| e.range.start).collect();
    assert_eq!(starts, [written, at_nth(LOOPS, "(o)", 0, 1)]);
}

const OUTSIDE: &str = r#"#import "Basic";

Point :: struct {
    x: int;
}

main :: () {
    p: Point;
    p.x = 3;
    print("%\n", p.x);
}
"#;

/// A name the project does not declare (a built-in type, a procedure of an imported module)
/// cannot be renamed: the edit would break every use, or rewrite a module the project does not own.
#[test]
fn only_names_the_project_declares_can_be_renamed() {
    let mut s: Session = session();
    s.open(uri(), 1, OUTSIDE.into()).unwrap();
    for (needle, width) in [("int;", 3), ("print(", 5), ("Basic", 5)] {
        let at = at_nth(OUTSIDE, needle, 0, 1);
        assert!(
            s.prepare_rename(&uri(), at).unwrap().is_none(),
            "`{}` can be renamed",
            &needle[..width]
        );
        assert!(s.rename(&uri(), at, "other").unwrap().is_none());
    }
    // The project's own declarations still can.
    for needle in ["Point;", "p: Point", "x: int"] {
        let at = at_nth(OUTSIDE, needle, 0, 1);
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
