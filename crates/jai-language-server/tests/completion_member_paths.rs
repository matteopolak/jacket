//! Member completion through struct constants, parameters and `using` member paths.
use test_support::lsp::{cursor, file_uri, lean_session};

fn labels(source: &str) -> Vec<String> {
    let (text, at) = cursor(source);
    let mut session = lean_session();
    let uri = file_uri("/member-paths/main.jai");
    session.open(uri.clone(), 1, text).unwrap();
    let list = session.completion(&uri, at).unwrap();
    list.items.into_iter().map(|i| i.label).collect()
}

fn offers(source: &str, name: &str) {
    let labels = labels(source);
    assert!(labels.contains(&name.to_string()), "{name} in {labels:?}");
}

#[test]
fn struct_parameters_through_a_value() {
    offers(
        "Grid :: struct(W: int, H: int) { cells: [W * H] u8; }\nmain :: () { g: Grid(2, 3); x := g.|W; }",
        "H",
    );
}

#[test]
fn struct_constants_through_a_value() {
    offers(
        "Table :: struct { SIZE :: 8; n: int; }\nmain :: () { t: Table; x := t.|n; }",
        "SIZE",
    );
}

#[test]
fn struct_parameters_through_an_instance_type() {
    offers(
        "Box :: struct(T: Type) { v: T; }\nmain :: () { x := Box(int).|T; }",
        "T",
    );
}

#[test]
fn constants_of_a_using_member_through_the_type() {
    offers(
        "Inner :: struct { n: int; K :: 3; }\nPlain :: struct { using i: Inner; }\nmain :: () { x := Plain.|K; }",
        "K",
    );
}

#[test]
fn a_field_named_through_its_struct() {
    offers(
        "Map :: struct { Load :: 70; }\nAnim :: struct { joints: Map; }\nmain :: () { x := Anim.joints.|Load; }",
        "Load",
    );
}

#[test]
fn a_using_member_path_reaches_the_nested_members() {
    let source = "Desc :: struct { width: int; height: int; depth: int; }\n\
                  Tex :: struct { desc: Desc; }\n\
                  Tex2D :: struct { texture: Tex; using,only(width, height) texture.desc; }\n\
                  main :: () { t: Tex2D; x := t.|width; }";
    let labels = labels(source);
    assert!(labels.contains(&"height".to_string()), "{labels:?}");
    assert!(!labels.contains(&"depth".to_string()), "{labels:?}");
}

#[test]
fn a_member_path_override_in_a_struct_body() {
    offers(
        "Base :: struct { id: int; g: int; }\nDerived :: struct { using #as base: Base; base.|id = 7; }\nmain :: () { d: Derived; }",
        "g",
    );
}

#[test]
fn a_parameter_of_a_polymorphic_struct_in_the_return_type() {
    offers(
        "Holder :: struct(Elem: Type) { storage: *Elem; }\nfirst :: (holder: Holder) -> *holder.|Elem { return holder.storage; }\nmain :: () { h: Holder(int); p := first(h); }",
        "Elem",
    );
}

#[test]
fn a_restricted_polymorphic_type_in_the_return_type() {
    offers(
        "Box :: struct(T: Type) { v: T; }\nvalue :: (b: $B/Box) -> B.|T { return b.v; }\nmain :: () { b: Box(int); x := value(b); }",
        "T",
    );
}

#[test]
fn a_parameter_in_a_later_parameter_type() {
    offers(
        "Cache :: struct(T: Type) { Proc :: #type (*T) -> int; proc: Proc; }\ninit :: (cache: *$C/Cache, proc: cache.|Proc) { cache.proc = proc; }\nmain :: () { c: Cache(int); }",
        "Proc",
    );
}
