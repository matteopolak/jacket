//! References to and renames of struct fields and enum members.
use jai_language_server::{DocumentUri, Environment, Limits, Location, Position, Session};
use std::path::PathBuf;

fn environment() -> Environment {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn session() -> Session {
    Session::with_environment(Limits::default(), environment())
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///lsp-members-test/main.jai").unwrap()
}

fn at(text: &str, marker: &str, n: usize, offset: usize) -> Position {
    let byte = text.match_indices(marker).nth(n).unwrap().0 + offset;
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().len() as u32,
    }
}

const PROGRAM: &str = r#"Vec :: struct {
    x: float;
    y: float;
}

Other :: struct {
    x: int;
}

Base :: struct {
    id: int;
}

Player :: struct {
    using base: Base;
    pos: Vec;
}

Kind :: enum {
    ONE;
    TWO;
}

Pick :: enum { ONE; }

main :: () {
    v := Vec.{ x = 1, y = 2 };
    o := Other.{ x = 3 };
    p: Player;
    q := *p;
    p.pos.x = v.x + 1;
    q.id = 4;
    q.base.id += p.id;
    k := Kind.ONE;
    if k == {
        case .TWO; v.y = 0;
        case .ONE; o.x = 0;
    }
    c: Pick = .ONE;
}
"#;

fn starts(refs: &[Location]) -> Vec<Position> {
    refs.iter().map(|r| r.range.start).collect()
}

#[test]
fn field_references_follow_the_struct_not_the_name() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let refs = s
        .references(&uri(), at(PROGRAM, "x: float", 0, 0), true)
        .unwrap();
    assert_eq!(
        starts(&refs),
        [
            at(PROGRAM, "x: float", 0, 0),
            at(PROGRAM, "x = 1", 0, 0),
            at(PROGRAM, "x = v.x", 0, 0),
            at(PROGRAM, "x + 1", 0, 0),
        ],
        "{refs:?}"
    );
    // From a use; without the declaration.
    let uses = s
        .references(&uri(), at(PROGRAM, "x + 1", 0, 0), false)
        .unwrap();
    assert_eq!(uses.len(), 3, "{uses:?}");
    // The other struct's `x`.
    let other = s
        .references(&uri(), at(PROGRAM, "x: int", 0, 0), true)
        .unwrap();
    assert_eq!(
        starts(&other),
        [
            at(PROGRAM, "x: int", 0, 0),
            at(PROGRAM, "x = 3", 0, 0),
            at(PROGRAM, "x = 0", 0, 0)
        ],
        "{other:?}"
    );
}

#[test]
fn fields_through_pointers_and_using() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let refs = s
        .references(&uri(), at(PROGRAM, "id: int", 0, 0), true)
        .unwrap();
    assert_eq!(
        starts(&refs),
        [
            at(PROGRAM, "id: int", 0, 0),
            at(PROGRAM, "id = 4", 0, 0),
            at(PROGRAM, "id += ", 0, 0),
            at(PROGRAM, "id;\n    k", 0, 0),
        ],
        "{refs:?}"
    );
}

#[test]
fn rename_a_field() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let position = at(PROGRAM, "x = 1", 0, 0);
    s.prepare_rename(&uri(), position).unwrap().unwrap();
    let edits = s.rename(&uri(), position, "px").unwrap().unwrap();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].1.len(), 4, "{edits:?}");
    assert!(edits[0].1.iter().all(|e| e.new_text == "px"));
}

#[test]
fn enum_member_references_and_rename() {
    let mut s = session();
    s.open(uri(), 1, PROGRAM.into()).unwrap();
    let refs = s
        .references(&uri(), at(PROGRAM, "Kind.ONE", 0, 5), true)
        .unwrap();
    assert_eq!(
        starts(&refs),
        [
            at(PROGRAM, "ONE;\n    TWO", 0, 0),
            at(PROGRAM, "Kind.ONE", 0, 5),
            at(PROGRAM, ".ONE; o", 0, 1),
        ],
        "{refs:?}"
    );
    let edits = s
        .rename(&uri(), at(PROGRAM, ".TWO;", 0, 1), "SECOND")
        .unwrap()
        .unwrap();
    assert_eq!(edits[0].1.len(), 2, "{edits:?}");
    // The other enum's `ONE`.
    let pick = s
        .references(&uri(), at(PROGRAM, "ONE; }", 0, 0), true)
        .unwrap();
    assert_eq!(pick.len(), 2, "{pick:?}");
}

#[test]
fn references_reach_project_files_that_are_not_open() {
    let dir = std::env::temp_dir().join(format!("jailsp-members-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let util = "Point :: struct { x: int; }\n";
    let main = "#load \"util.jai\";\nmain :: () { p: Point; p.x = 1; }\n";
    std::fs::write(dir.join("util.jai"), util).unwrap();
    std::fs::write(dir.join("main.jai"), main).unwrap();
    std::fs::write(dir.join("jai.toml"), "build_files = [\"main.jai\"]\n").unwrap();
    let util_uri = DocumentUri::parse(&format!("file://{}/util.jai", dir.display())).unwrap();
    let mut s = session();
    s.open(util_uri.clone(), 1, util.into()).unwrap();
    let refs = s.references(&util_uri, at(util, "x:", 0, 0), true).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(refs.len(), 2, "{refs:?}");
    assert!(
        refs.iter().any(|r| r.uri.ends_with("/main.jai")),
        "{refs:?}"
    );
}

#[test]
fn members_of_modules_cannot_be_renamed() {
    let mut s = session();
    let text = "#import \"Basic\";\nmain :: () { a: [..] int; a.count = 0; }\n";
    s.open(uri(), 1, text.into()).unwrap();
    assert!(
        s.prepare_rename(&uri(), at(text, "count", 0, 0))
            .unwrap()
            .is_none()
    );
}

const GOTO: &str = r#"Color :: enum { RED; GREEN; }

Inner :: struct {
    name: string;
    color: Color;
}

Stuff :: struct {
    stuff_a: int;
    struct {
        anon_b: int;
        anon_c: int;
    }
}

main :: () {
    stuff: Stuff;
    stuff.stuff_a = 1;
    stuff.anon_c = 2;
    items: [2] Inner;
    for m: items {
        print_name(m.name);
    }
    for * p: items {
        p.name = "x";
    }
    c := Color.RED;
}

print_name :: (s: string) {}
"#;

fn definition(s: &Session, text: &str, marker: &str, offset: usize) -> Vec<Position> {
    s.definition(&uri(), at(text, marker, 0, offset))
        .unwrap()
        .iter()
        .map(|l| l.range.start)
        .collect()
}

#[test]
fn definition_of_struct_members() {
    let mut s = session();
    s.open(uri(), 1, GOTO.into()).unwrap();
    assert_eq!(
        definition(&s, GOTO, "stuff_a = 1", 0),
        [at(GOTO, "stuff_a: int", 0, 0)]
    );
    // A member of the anonymous struct inside `Stuff`.
    assert_eq!(
        definition(&s, GOTO, "anon_c = 2", 0),
        [at(GOTO, "anon_c: int", 0, 0)]
    );
    // Through a `for` variable, by value and by pointer.
    assert_eq!(
        definition(&s, GOTO, "name);", 0),
        [at(GOTO, "name: string", 0, 0)]
    );
    assert_eq!(
        definition(&s, GOTO, "name = \"x\"", 0),
        [at(GOTO, "name: string", 0, 0)]
    );
    // References and rename work from those uses too.
    let refs = s
        .references(&uri(), at(GOTO, "name);", 0, 0), true)
        .unwrap();
    assert_eq!(refs.len(), 3, "{refs:?}");
    let refs = s
        .references(&uri(), at(GOTO, "anon_c = 2", 0, 0), true)
        .unwrap();
    assert_eq!(refs.len(), 2, "{refs:?}");
    s.prepare_rename(&uri(), at(GOTO, "anon_c = 2", 0, 0))
        .unwrap()
        .unwrap();
}

#[test]
fn definition_of_enum_members() {
    let mut s = session();
    s.open(uri(), 1, GOTO.into()).unwrap();
    assert_eq!(
        definition(&s, GOTO, "Color.RED", 6),
        [at(GOTO, "RED; GREEN", 0, 0)]
    );
    let refs = s
        .references(&uri(), at(GOTO, "Color.RED", 0, 6), true)
        .unwrap();
    assert_eq!(refs.len(), 2, "{refs:?}");
}
