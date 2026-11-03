//! The outline nests declarations under scope sections and `#if` blocks, and shows the
//! directives of a file.
use jai_language_server::{DocumentSymbol, Range, SymbolKind};
use test_support::lsp::{file_uri, lean_session};

const SOURCE: &str = "\
#load \"other.jai\";
Point :: struct {
    x: float;
    y: float;
}
Mode :: enum { slow; fast; }
#if OS == .MACOS {
    on_mac :: () {}
} else {
    elsewhere :: () {}
}
#run on_mac();
#placeholder Hook;
#scope_file
helper :: () {}
#scope_module
shared :: 1;
#scope_export
main :: () {}
";

fn symbols() -> Vec<DocumentSymbol> {
    let mut session = lean_session();
    let uri = file_uri("/outline/main.jai");
    session.open(uri.clone(), 1, SOURCE.into()).unwrap();
    session.document_symbols(&uri).unwrap()
}

fn names(list: &[DocumentSymbol]) -> Vec<&str> {
    list.iter().map(|s| s.name.as_str()).collect()
}

fn find<'a>(list: &'a [DocumentSymbol], prefix: &str) -> &'a DocumentSymbol {
    list.iter()
        .find(|s| s.name.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} in {:?}", names(list)))
}

fn within(inner: Range, outer: Range) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn check_ranges(list: &[DocumentSymbol], parent: Option<Range>) {
    for symbol in list {
        assert!(
            within(symbol.selection_range, symbol.range),
            "{}: selection inside range",
            symbol.name
        );
        if let Some(parent) = parent {
            assert!(
                within(symbol.range, parent),
                "{} inside its parent",
                symbol.name
            );
        }
        check_ranges(&symbol.children, Some(symbol.range));
    }
}

#[test]
fn the_top_level_lists_declarations_and_directives_in_order() {
    let top = symbols();
    let names = names(&top);
    let order = [
        "#load",
        "Point",
        "Mode",
        "#if",
        "#run",
        "Hook",
        "#scope_file",
        "#scope_module",
        "#scope_export",
    ];
    let mut at = 0;
    for expected in order {
        let found = names[at..]
            .iter()
            .position(|n| n.starts_with(expected))
            .unwrap_or_else(|| panic!("{expected} after {at} in {names:?}"));
        at += found + 1;
    }
    assert!(!names.contains(&"helper"), "{names:?}");
}

#[test]
fn types_hold_their_fields_and_members() {
    let top = symbols();
    let point = find(&top, "Point");
    assert_eq!(point.kind, SymbolKind::Struct);
    assert_eq!(names(&point.children), ["x", "y"]);
    assert_eq!(find(&top, "Mode").children.len(), 2);
}

#[test]
fn an_if_holds_its_branches_and_their_declarations() {
    let top = symbols();
    let branch = find(&top, "#if OS == .MACOS");
    assert_eq!(branch.kind, SymbolKind::Boolean);
    assert_eq!(names(&branch.children), ["on_mac", "#else"]);
    let other = find(&branch.children, "#else");
    assert_eq!(names(&other.children), ["elsewhere"]);
}

#[test]
fn scope_sections_hold_what_they_cover() {
    let top = symbols();
    assert_eq!(names(&find(&top, "#scope_file").children), ["helper"]);
    assert_eq!(names(&find(&top, "#scope_module").children), ["shared"]);
    assert_eq!(names(&find(&top, "#scope_export").children), ["main"]);
}

#[test]
fn kinds_suit_the_directives() {
    let top = symbols();
    assert_eq!(find(&top, "#load").kind, SymbolKind::File);
    assert_eq!(find(&top, "#run").kind, SymbolKind::Event);
    assert_eq!(find(&top, "#scope_file").kind, SymbolKind::Namespace);
    assert_eq!(find(&top, "Hook").detail, "#placeholder");
}

#[test]
fn every_selection_lies_in_its_range_and_every_child_in_its_parent() {
    check_ranges(&symbols(), None);
}
