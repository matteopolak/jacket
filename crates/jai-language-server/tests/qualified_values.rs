//! Independent language/IDE regression: module-qualified declarations retain identity.
//! No replaced-stdlib implementation or convenience API is the test oracle.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Session, TextChange};
use jaic::sema::{Options, VirtualFs};
use std::{path::PathBuf, rc::Rc};

const MODULE: &str = "Item :: struct { field: int; }\nseed :: 41;\nscalar :: seed + 1;\naggregate :: Item.{scalar};\nstate: Item;\nNumber :: s32;\ncallback :: (status: s32) {}\n#scope_file\nsecret :: 99;\n";
const SOURCE: &str = "Data :: #import \"Values\";\nAlias :: Data;\nmain :: () {\n    first := Data.scalar;\n    second := Alias.aggregate;\n    third := Data.state;\n    number: Data.Number;\n    result := second.field;\n    Data.callback(1);\n}\n";

#[test]
fn failed_identifier_does_not_become_a_literal_members_definition() {
    let uri = DocumentUri::parse("file:///qualified-values/main.jai").unwrap();
    let source = "bad :: missing;\nmain :: () { failed := bad; length := \"abc\".count; }\n";
    let mut s = session();
    s.open(uri.clone(), 1, source.into()).unwrap();
    assert!(s.diagnostics(&uri).unwrap().iter().any(|d| {
        d.severity == jai_language_server::DiagnosticSeverity::Error
            && d.message.contains("missing")
    }));
    assert!(
        s.definition(&uri, at(source, "count;")).unwrap().is_empty(),
        "a builtin string count has no source declaration; a failed entity must not leak"
    );
}

fn at(text: &str, needle: &str) -> Position {
    let prefix = &text[..text.find(needle).unwrap()];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}

#[test]
fn qualified_private_name_remains_an_error_without_a_fabricated_target() {
    let uri = DocumentUri::parse("file:///qualified-values/main.jai").unwrap();
    let source = SOURCE.replace("Data.scalar", "Data.secret");
    let mut s = session();
    s.open(uri.clone(), 1, source.clone()).unwrap();
    assert!(s.diagnostics(&uri).unwrap().iter().any(|d| {
        d.severity == jai_language_server::DiagnosticSeverity::Error && d.message.contains("secret")
    }));
    assert!(
        s.definition(&uri, at(&source, "secret;"))
            .unwrap()
            .is_empty()
    );
    let definitions = s.definition(&uri, at(&source, "aggregate;")).unwrap();
    assert_eq!(definitions.len(), 1, "{definitions:?}");
    assert_eq!(definitions[0].range.start, at(MODULE, "aggregate ::"));
}

fn session() -> Session {
    let mut fs = VirtualFs::default();
    fs.insert(
        "/qualified-values/modules/Values/module.jai",
        MODULE.as_bytes(),
    );
    fs.insert(
        "/qualified-values/modules/Other/module.jai",
        "scalar :: 7;\n".as_bytes(),
    );
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(fs),
            options: Box::new(|_| {
                let mut options = Options::host();
                // An independent compiler/IDE regression. The native investigation
                // separately uses production runtime/bootstrap and genuine Basic.
                options.runtime_support = false;
                options.stack_trace = false;
                options.import_paths = vec![PathBuf::from("/qualified-values/modules")];
                options
            }),
        },
    )
}

#[test]
fn same_named_values_in_different_modules_follow_the_actual_lookup() {
    let uri = DocumentUri::parse("file:///qualified-values/main.jai").unwrap();
    let source = "Left :: #import \"Values\";\nRight :: #import \"Other\";\nmain :: () { first := Left.scalar; second := Right.scalar; }\n";
    let mut s = session();
    s.open(uri.clone(), 1, source.into()).unwrap();
    for (usage, path, declaration) in [
        ("scalar; second", "Values", at(MODULE, "scalar ::")),
        (
            "scalar; }",
            "Other",
            Position {
                line: 0,
                character: 0,
            },
        ),
    ] {
        let definitions = s.definition(&uri, at(source, usage)).unwrap();
        assert_eq!(definitions.len(), 1, "{usage}: {definitions:?}");
        assert_eq!(
            definitions[0].uri,
            format!("file:///qualified-values/modules/{path}/module.jai")
        );
        assert_eq!(definitions[0].range.start, declaration);
    }
}

fn assert_definitions(s: &Session, uri: &DocumentUri, source: &str) {
    for (usage, declaration) in [
        ("scalar;", "scalar ::"),
        ("aggregate;", "aggregate ::"),
        ("state;", "state:"),
        ("Number;", "Number ::"),
        ("field;", "field:"),
        ("callback(1)", "callback ::"),
    ] {
        let definitions = s.definition(uri, at(source, usage)).unwrap();
        assert_eq!(definitions.len(), 1, "{usage}: {definitions:?}");
        assert_eq!(
            definitions[0].uri, "file:///qualified-values/modules/Values/module.jai",
            "{usage}"
        );
        assert_eq!(
            definitions[0].range.start,
            at(MODULE, declaration),
            "{usage}"
        );
        let hover = s.hover(uri, at(source, usage)).unwrap().unwrap();
        assert!(!hover.contents.value.contains("Source syntax declaration"));
    }
}

#[test]
fn qualified_scalar_aggregate_variable_and_type_alias_keep_their_declarations() {
    let uri = DocumentUri::parse("file:///qualified-values/main.jai").unwrap();
    let mut s = session();
    s.open(uri.clone(), 1, SOURCE.into()).unwrap();
    assert!(
        s.diagnostics(&uri)
            .unwrap()
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error),
        "{:?}",
        s.diagnostics(&uri)
    );
    assert_definitions(&s, &uri, SOURCE);
    let damaged = SOURCE.replace("    first", "    broken := (\n    first");
    s.change(
        &uri,
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: damaged.clone(),
        }],
    )
    .unwrap();
    assert_definitions(&s, &uri, &damaged);
    s.change(
        &uri,
        3,
        &[TextChange {
            range: None,
            range_length: None,
            text: SOURCE.into(),
        }],
    )
    .unwrap();
    assert_definitions(&s, &uri, SOURCE);
}
