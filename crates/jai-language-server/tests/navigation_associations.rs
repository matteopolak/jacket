//! Independent compiler-owner reductions, checkpoint 36ea131e; strict authored source oracles.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Range, Session, TextChange};
use jaic::sema::{Options, VirtualFs};
use std::rc::Rc;

fn check(source: &str, targets: &[(u32, u32, u32, u32)]) {
    let uri = DocumentUri::parse("file:///navigation-associations/main.jai").unwrap();
    let mut session = Session::with_environment(
        Limits::default(),
        Environment {
            fs: Rc::new(VirtualFs::default()),
            options: Box::new(|_| {
                let mut o = Options::host();
                o.runtime_support = false;
                o.stack_trace = false;
                o
            }),
        },
    );
    session.open(uri.clone(), 1, source.into()).unwrap();
    for version in 1..=3 {
        if version > 1 {
            let suffix = "\n// 😀 UTF-16 edit\nunrelated :: (";
            let end = position(source);
            let (range, text) = if version == 2 {
                (
                    Range {
                        start: end,
                        end,
                    },
                    suffix.to_owned(),
                )
            } else {
                (
                    Range {
                        start: end,
                        end: position(&format!("{source}{suffix}")),
                    },
                    String::new(),
                )
            };
            session
                .change(
                    &uri,
                    version,
                    &[TextChange {
                        range: Some(range),
                        range_length: None,
                        text,
                    }],
                )
                .unwrap();
        }
        let errors: Vec<_> = session
            .diagnostics(&uri)
            .unwrap()
            .into_iter()
            .filter(|d| d.severity == jai_language_server::DiagnosticSeverity::Error)
            .collect();
        if version != 2 {
            assert!(errors.is_empty(), "{errors:?}");
        } else {
            assert!(
                !errors.is_empty(),
                "the incomplete suffix must remain an error"
            );
        }
        assert!(
            errors
                .iter()
                .all(|d| d.range.start.line >= source.lines().count() as u32),
            "{errors:?}"
        );
        for &(line, character, target_line, target_character) in targets {
            let position = Position {
                line,
                character,
            };
            assert!(
                session.hover(&uri, position).unwrap().is_some(),
                "hover {position:?}, version {version}"
            );
            let definitions = session.definition(&uri, position).unwrap();
            assert_eq!(
                definitions.len(),
                1,
                "{position:?}, version {version}: {definitions:?}"
            );
            assert_eq!(definitions[0].uri, uri.as_str());
            assert_eq!(
                definitions[0].range.start,
                Position {
                    line: target_line,
                    character: target_character
                }
            );
        }
    }
}

fn position(prefix: &str) -> Position {
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}

fn check_names(source: &str, targets: &[(&str, &str)]) {
    let targets: Vec<_> = targets
        .iter()
        .map(|&(usage, declaration)| {
            let at = position(&source[..source.find(usage).unwrap()]);
            let target = position(&source[..source.find(declaration).unwrap()]);
            (at.line, at.character, target.line, target.character)
        })
        .collect();
    check(source, &targets);
}

#[test]
fn dependent_and_baked_struct_arguments_use_selected_parameter_declarations() {
    check_names(
        "Container :: struct (T: Type, callback: (value: T) -> T) { item: T; }\nidentity :: (actual: s64) -> s64 { return actual; }\nFixed :: #bake_arguments Container(T = s64);\nAlias :: Fixed;\nfirst: Container(T = s64, callback = identity);\nsecond: Alias(callback = identity);\nmain :: () {}\n",
        &[
            ("callback = identity);\nsecond", "callback:"),
            ("callback = identity);\nmain", "callback:"),
        ],
    );
}

#[test]
fn nested_and_zero_overrides_keep_each_selected_member_source() {
    check_names(
        "Base :: struct { value: s64; }\nMiddle :: struct { using #as base: Base; }\nDerived :: struct {\nusing #as middle: Middle;\nmiddle.base.value = 0;\n}\nPromoted :: struct { using #as parent: Base; value = 1; }\nmain :: () { a: Derived; b: Promoted; }\n",
        &[
            ("middle.base", "middle: Middle"),
            ("base.value", "base: Base"),
            ("value = 0", "value: s64"),
            ("value = 1", "value: s64"),
        ],
    );
}

#[test]
fn structural_parameter_types_do_not_merge_authored_names() {
    check_names(
        "First :: #type (left: s64) -> (answer: s64);\nSecond :: #type (right: s64) -> (result: s64);\nmain :: () { first: First; second: Second; }\n",
        &[
            ("left:", "left:"),
            ("right:", "right:"),
            ("answer:", "answer:"),
            ("result:", "result:"),
        ],
    );
}

#[test]
fn procedure_type_parameters() {
    check(
        r#"Callback :: #type (left: s64, right: s64) -> s64;
Record :: struct { callback: #type (item: s64) -> s64; }
main :: () { record: Record; callback: Callback; }
"#,
        &[(0, 19, 0, 19), (0, 30, 0, 30), (1, 36, 1, 36)],
    );
}

#[test]
fn named_generic_callback() {
    check(
        r#"Box :: struct (callback: (value: s64) -> s64) { value: s64; }
identity :: (actual: s64) -> s64 { return actual; }
box: Box(callback = identity);
main :: () {}
"#,
        &[(2, 9, 0, 15)],
    );
}

#[test]
fn inherited_member_default() {
    check(
        r#"Base :: struct { value: s64; }
Derived :: struct {
    using #as base: Base;
    base.value = 7;
}
main :: () { record: Derived; result := record.value; }
"#,
        &[(3, 9, 0, 17), (5, 47, 0, 17)],
    );
}
