//! A struct's supertypes are the types it embeds with `using`; its subtypes embed it.
use jai_language_server::{DocumentUri, Position, Session, TypeHierarchyItem};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
Entity :: struct { id: int; }
Named :: struct { name: string; }
Monster :: struct {
    #as using entity: Entity;
    using named: Named;
    health: int;
}
Dragon :: struct {
    #as using monster: Monster;
    wings: int;
}
Rock :: struct { weight: int; }
main :: () {
    d: Dragon;
}
";

fn open() -> (Session, DocumentUri) {
    let mut session = lean_session();
    let uri = file_uri("/types/main.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    (session, uri)
}

fn at(line: u32, character: u32) -> Position {
    Position {
        line,
        character,
    }
}

fn item(session: &Session, uri: &DocumentUri, position: Position) -> TypeHierarchyItem {
    let mut items = session.prepare_type_hierarchy(uri, position).unwrap();
    assert_eq!(items.len(), 1, "{items:?}");
    items.remove(0)
}

fn names(items: &[TypeHierarchyItem]) -> Vec<&str> {
    items.iter().map(|i| i.name.as_str()).collect()
}

#[test]
fn a_declaration_and_a_use_prepare_the_same_item() {
    let (session, uri) = open();
    let declared = item(&session, &uri, at(2, 2));
    let used = item(&session, &uri, at(8, 28));
    assert_eq!(declared, used);
    assert_eq!(declared.name, "Monster");
    assert_eq!(declared.selection_range.start, at(2, 0));
}

#[test]
fn supertypes_are_the_embedded_types() {
    let (session, uri) = open();
    let monster = item(&session, &uri, at(2, 2));
    let supers = session.supertypes(&monster).unwrap();
    assert_eq!(names(&supers), ["Entity", "Named"]);
    assert!(session.supertypes(&supers[0]).unwrap().is_empty());
}

#[test]
fn subtypes_are_the_types_that_embed_it() {
    let (session, uri) = open();
    let entity = item(&session, &uri, at(0, 1));
    let subs = session.subtypes(&entity).unwrap();
    assert_eq!(names(&subs), ["Monster"]);
    assert_eq!(names(&session.subtypes(&subs[0]).unwrap()), ["Dragon"]);
}

#[test]
fn a_struct_that_embeds_nothing_has_no_hierarchy() {
    let (session, uri) = open();
    let rock = item(&session, &uri, at(11, 1));
    assert!(session.supertypes(&rock).unwrap().is_empty());
    assert!(session.subtypes(&rock).unwrap().is_empty());
}

#[test]
fn only_structs_prepare_an_item() {
    let (session, uri) = open();
    assert!(
        session
            .prepare_type_hierarchy(&uri, at(12, 2))
            .unwrap()
            .is_empty()
    );
}
