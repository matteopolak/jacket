//! Real generic instances across workspace programs, without editor sketches.
use jai_language_server::{Position, TextChange};
mod common;
use common::{uri_of, virtual_session};
use std::path::Path;

const GENERIC: &str = "identity :: (x: $T) -> T { return x; }\nunused :: (x:$T) { identity(x); }\n";

#[test]
fn closed_workspace_callers_outside_the_library_tree_are_merged_and_cached() {
    let files = [
        (
            "/workspace/jacket.toml",
            "[project]\nentry=[\"a.jai\",\"b.jai\",\"c.jai\",\"d.jai\"]\nimport_path=[\"/shared\"]\n",
        ),
        (
            "/workspace/a.jai",
            "L :: #import \"Lib\"; main :: () { x:u8; L.identity(x); L.identity(x); }",
        ),
        (
            "/workspace/b.jai",
            "L :: #import \"Lib\"; main :: () { x:u16; L.identity(x); }",
        ),
        (
            "/workspace/c.jai",
            "L :: #import \"Lib\"; main :: () { x:u32; L.identity(x); }",
        ),
        (
            "/workspace/d.jai",
            "L :: #import \"Lib\"; main :: () { x:u64; L.identity(x); }",
        ),
        ("/shared/Lib/module.jai", GENERIC),
    ];
    let mut s = virtual_session(&files, &["/shared"], None);
    s.set_workspace_folders(vec!["/workspace".into()]);
    let uri = uri_of(Path::new("/shared/Lib/module.jai"));
    s.open(uri.clone(), 1, GENERIC.into()).unwrap();
    let lenses = s.code_lenses(&uri).unwrap();
    let at = lenses[0].range.start;
    assert_eq!(
        s.polymorphs(&uri, at),
        ["T = u16", "T = u32", "T = u64", "T = u8"]
    );
    assert_eq!(lenses[1].command.title, "no polymorphs yet");
    let before = s.compile_counts();
    assert!(before.0 >= 4);
    for _ in 0..3 {
        assert_eq!(s.polymorphs(&uri, at).len(), 4);
    }
    assert_eq!(
        before,
        s.compile_counts(),
        "summaries must survive compiler eviction"
    );
    let caller = uri_of(Path::new("/workspace/a.jai"));
    s.open(caller.clone(), 1, files[1].1.into()).unwrap();
    s.polymorphs(&uri, at);
    let before = s.compile_counts().0;
    s.change(
        &caller,
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: "L :: #import \"Lib\"; main :: () { x:s32; L.identity(x); }".into(),
        }],
    )
    .unwrap();
    assert_eq!(
        s.polymorphs(&uri, at),
        ["T = s32", "T = u16", "T = u32", "T = u64"]
    );
    assert_eq!(
        s.compile_counts().0,
        before + 1,
        "only the changed caller program recompiles"
    );
}

#[test]
fn sketch_instances_and_rejected_overloads_do_not_count() {
    let text = "pick :: (x:$T) -> T { return x; }\npick :: (x:u32) -> u32 { return x; }\nwrap1 :: (x:$T) {pick(x);}\nwrap2 :: (x:$T) {pick(x);}\nmain :: () {x:u32; pick(x);}";
    let mut s = virtual_session(&[("/app/main.jai", text)], &[], None);
    let uri = uri_of(Path::new("/app/main.jai"));
    s.open(uri.clone(), 1, text.into()).unwrap();
    let lenses = s.code_lenses(&uri).unwrap();
    assert_eq!(lenses[0].command.title, "no polymorphs yet");
}

#[test]
fn nominal_types_with_the_same_name_are_not_deduplicated_by_display() {
    let files = [
        (
            "/app/jacket.toml",
            "[project]\nentry=[\"a.jai\",\"b.jai\",\"c.jai\"]",
        ),
        (
            "/app/a.jai",
            "L :: #import \"Lib\"; T :: struct {x:u32;} main :: () {x:T; L.identity(x);}",
        ),
        (
            "/app/b.jai",
            "L :: #import \"Lib\"; T :: struct {x:u32;} main :: () {x:T; L.identity(x);}",
        ),
        (
            "/app/c.jai",
            "L :: #import \"Lib\"; T :: struct {x:u64;} main :: () {x:T; L.identity(x);}",
        ),
        ("/shared/Lib/module.jai", GENERIC),
    ];
    let mut s = virtual_session(&files, &["/shared"], None);
    s.set_workspace_folders(vec!["/app".into()]);
    let uri = uri_of(Path::new("/shared/Lib/module.jai"));
    s.open(uri.clone(), 1, GENERIC.into()).unwrap();
    let values = s.polymorphs(
        &uri,
        Position {
            line: 0,
            character: 0,
        },
    );
    assert_eq!(
        values.len(),
        3,
        "these are three real nominal types: {values:?}"
    );
    for path in ["a.jai", "b.jai", "c.jai"] {
        assert!(
            values
                .iter()
                .any(|v| v.starts_with("T = T [") && v.contains(path)),
            "{values:?}"
        );
    }
}

#[test]
fn baked_values_and_many_instantiations_are_not_truncated() {
    let lib = "baked :: ($N:s64) -> s64 {return N;}";
    let calls = (0..48)
        .map(|n| format!("L.baked({n});"))
        .collect::<String>();
    let main = format!("L :: #import \"Lib\"; main :: () {{{calls}}}");
    let mut s = virtual_session(
        &[("/app/main.jai", &main), ("/shared/Lib/module.jai", lib)],
        &["/shared"],
        None,
    );
    s.set_workspace_folders(vec!["/app".into()]);
    let uri = uri_of(Path::new("/shared/Lib/module.jai"));
    s.open(uri.clone(), 1, lib.into()).unwrap();
    let values = s.polymorphs(
        &uri,
        Position {
            line: 0,
            character: 0,
        },
    );
    assert_eq!(values.len(), 48);
    assert!(values.contains(&"N = 47".into()));
    assert!(
        s.code_lenses(&uri).unwrap()[0]
            .command
            .title
            .starts_with("48 polymorphs:")
    );
}

#[test]
fn partial_analysis_is_not_reported_as_zero_known_instances() {
    let text = "id :: (x:$T)->T {return x;} main :: () { missing(); }";
    let mut s = virtual_session(&[("/app/main.jai", text)], &[], None);
    let uri = uri_of(Path::new("/app/main.jai"));
    s.open(uri.clone(), 1, text.into()).unwrap();
    assert!(
        s.code_lenses(&uri).unwrap()[0]
            .command
            .title
            .contains("analysis incomplete")
    );
}

#[test]
fn shared_nominal_types_deduplicate_across_entry_programs() {
    let lib = "T :: struct {x:u32;} identity :: (x:$T)->T {return x;}";
    let main = "L :: #import \"Lib\"; main :: () {x:L.T; L.identity(x);}";
    let mut s = virtual_session(
        &[
            ("/app/jacket.toml", "[project]\nentry=[\"a.jai\",\"b.jai\"]"),
            ("/app/a.jai", main),
            ("/app/b.jai", main),
            ("/shared/Lib/module.jai", lib),
        ],
        &["/shared"],
        None,
    );
    s.set_workspace_folders(vec!["/app".into()]);
    let uri = uri_of(Path::new("/shared/Lib/module.jai"));
    s.open(uri.clone(), 1, lib.into()).unwrap();
    let lens = s.code_lenses(&uri).unwrap();
    assert_eq!(s.polymorphs(&uri, lens[0].range.start), ["T = T"]);
}

#[test]
fn stdlib_instances_include_closed_callers() {
    let dir = common::project(
        "poly-stdlib",
        &[(
            "main.jai",
            "#import \"Basic\"; main :: () {a:[..]u32; array_add(*a,1); b:[..]s64; array_add(*b,2);}",
        )],
    );
    let mut s = common::session();
    s.set_workspace_folders(vec![dir.clone()]);
    let path = common::stdlib().join("internal/basic/arrays.jai");
    // Find the declaration in our own stdlib, independent of its implementation file name.
    let source = std::fs::read_to_string(&path).unwrap();
    let uri = uri_of(&path);
    s.open(uri.clone(), 1, source.clone()).unwrap();
    let lenses = s.code_lenses(&uri).unwrap();
    let declaration = source
        .find("array_add :: (array: *[..] $T, item: T)")
        .unwrap();
    let line = source[..declaration]
        .bytes()
        .filter(|b| *b == b'\n')
        .count() as u32;
    let row = lenses
        .iter()
        .find(|l| l.range.start.line == line)
        .expect("array_add item overload");
    let values = s.polymorphs(&uri, row.range.start);
    assert!(
        values.contains(&"T = u32".into()) && values.contains(&"T = s64".into()),
        "{values:?}"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn all_child_workspaces_of_a_build_contribute() {
    let lib = GENERIC;
    let build = "#import \"Compiler\"; #run { set_build_options_dc(.{do_output=false}); for 0..1 { w:=compiler_create_workspace(\"child\"); o:=get_build_options(w); o.output_type=.NO_OUTPUT; set_build_options(o,w); add_build_string(\"generated_marker :: 42;\",w); if it==0 add_build_file(\"a.jai\",w); else add_build_file(\"b.jai\",w); } }";
    let a =
        "L :: #import \"Lib\"; main :: () {#assert generated_marker==42; x:u32; L.identity(x);}";
    let b =
        "L :: #import \"Lib\"; main :: () {#assert generated_marker==42; x:u64; L.identity(x);}";
    let dir = common::project(
        "poly-build",
        &[
            ("z-build.jai", build),
            (
                "jacket.toml",
                "[project]\nentry=[\"a.jai\",\"b.jai\",\"z-build.jai\"]",
            ),
            ("a.jai", a),
            ("b.jai", b),
            ("modules/Lib/module.jai", lib),
        ],
    );
    let mut s = jai_language_server::Session::with_environment(
        jai_language_server::Limits::default(),
        common::modules_environment(None),
    );
    s.set_workspace_folders(vec![dir.clone()]);
    let uri = uri_of(&dir.join("modules/Lib/module.jai"));
    s.open(uri.clone(), 1, lib.into()).unwrap();
    let lenses = s.code_lenses(&uri).unwrap();
    let values = s.polymorphs(&uri, lenses[0].range.start);
    assert_eq!(values, ["T = u32", "T = u64"], "{lenses:?}");
    assert!(
        !lenses[0].command.title.contains("incomplete"),
        "{lenses:?}"
    );
    let before = s.compile_counts();
    s.code_lenses(&uri).unwrap();
    assert_eq!(s.compile_counts(), before);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn nested_calls_preserve_parameterized_module_type_identity() {
    let lib = "#module_parameters (FLAG := true); T :: struct {x:u32;} identity :: (x:$T)->T {return x;} wrap :: (x:$T)->T {return identity(x);}";
    let main = "A :: #import \"Lib\"; B :: #import \"Lib\"(FLAG=false); main :: () {a:A.T; b:B.T; A.wrap(a); B.wrap(b);}";
    let mut s = virtual_session(
        &[("/app/main.jai", main), ("/shared/Lib/module.jai", lib)],
        &["/shared"],
        None,
    );
    s.set_workspace_folders(vec!["/app".into()]);
    let uri = uri_of(Path::new("/shared/Lib/module.jai"));
    s.open(uri.clone(), 1, lib.into()).unwrap();
    let lenses = s.code_lenses(&uri).unwrap();
    assert_eq!(lenses.len(), 2);
    for lens in lenses {
        let values = s.polymorphs(&uri, lens.range.start);
        assert_eq!(
            values.len(),
            2,
            "nested calls are real, separate module instances: {values:?}"
        );
        assert!(
            values.iter().all(|v| v.contains("/shared/Lib/module.jai")),
            "{values:?}"
        );
        assert!(
            values.iter().any(|v| v.contains("FLAG = false")),
            "{values:?}"
        );
    }
}
