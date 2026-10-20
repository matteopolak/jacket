//! Cases the typing-session checker found (docs/tools/lsp-fuzzing.md), reduced by hand: answers
//! while someone types must not depend on which requests came before, on a half-typed neighbour,
//! or on a lexical edit that has since been completed.
#[allow(dead_code)]
#[path = "../examples/support/typing_files.rs"]
mod typing_files;

use jai_language_server::{Limits, Position, Range, SemanticTokenKind, Session, TextChange};
use std::path::Path;
mod common;
use common::{position, uri};

fn session() -> Session {
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../stdlib")
        .canonicalize()
        .unwrap();
    Session::with_environment(Limits::default(), typing_files::native_environment(stdlib))
}

fn open(text: &str) -> Session {
    let mut session = session();
    session.open(uri(), 1, text.into()).unwrap();
    session
}

/// Insert `typed` at byte `at` of the document as an editor would: one character per change.
fn type_in(session: &mut Session, text: &mut String, at: usize, typed: &str, version: &mut i32) {
    for (offset, c) in typed.char_indices() {
        let p = position(text, at + offset);
        *version += 1;
        session
            .change(
                &uri(),
                *version,
                &[TextChange {
                    range: Some(Range {
                        start: p,
                        end: p,
                    }),
                    range_length: None,
                    text: c.to_string(),
                }],
            )
            .unwrap();
        text.insert(at + offset, c);
        // Requests between keystrokes, as an editor makes them.
        let _ = session.diagnostics(&uri());
        let _ = session.semantic_tokens(&uri());
    }
}

fn kind_at(session: &Session, line: u32, character: u32) -> Option<SemanticTokenKind> {
    session
        .semantic_tokens(&uri())
        .unwrap()
        .into_iter()
        .find(|t| t.position.line == line && t.position.character == character)
        .map(|t| t.kind)
}

fn messages(session: &Session) -> Vec<String> {
    session
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .map(|d| format!("{:?} {}", d.range, d.message))
        .collect()
}

fn on_big_stack(work: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(1 << 29)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn a_string_closed_after_typing_leaves_no_lexical_damage() {
    on_big_stack(|| {
        let mut text = "main :: () {\n    \n}\n".to_string();
        let mut session = open(&text);
        let mut version = 1;
        let at = text.find("    \n").unwrap() + 4;
        type_in(&mut session, &mut text, at, "x := \"abc", &mut version);
        let close = at + "x := \"abc".len();
        type_in(&mut session, &mut text, close, "\";", &mut version);
        let fresh = open(&text);
        assert_eq!(messages(&session), messages(&fresh), "{text}");
        assert!(
            !messages(&session)
                .iter()
                .any(|m| m.contains("Incomplete lexical edit")),
            "{:?}",
            messages(&session)
        );
    });
}

#[test]
fn an_unfinished_cast_at_the_end_is_reported_once() {
    on_big_stack(|| {
        let session = open("x := cast(\n {\n __t");
        let all = messages(&session);
        let mut unique = all.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(all.len(), unique.len(), "{all:#?}");
    });
}

#[test]
fn a_callee_has_the_same_color_whether_or_not_completion_ran() {
    on_big_stack(|| {
        let text = "#import \"Basic\";\nmain :: () {\n    print(\"hi\\n\");\n}\n";
        let call = position(text, text.find("print").unwrap());
        let open_paren = position(text, text.find("print(").unwrap() + "print(".len());
        let plain = open(text);
        let before = kind_at(&plain, call.line, call.character);
        let asked = open(text);
        let _ = asked.signature_help(&uri(), open_paren);
        let _ = asked.completion(&uri(), open_paren);
        assert_eq!(kind_at(&asked, call.line, call.character), before);
        assert!(before.is_some());
    });
}

#[test]
fn a_module_alias_is_a_namespace_before_and_after_it_is_hovered() {
    on_big_stack(|| {
        let text = "B :: #import \"Basic\";\nmain :: () {\n    B.print(\"hi\\n\");\n}\n\n#run {\n    B.print(\"run\\n\");\n}\n";
        let session = open(text);
        for (line, character) in [(2, 4), (6, 4)] {
            let first = kind_at(&session, line, character);
            let _ = session.hover(
                &uri(),
                Position {
                    line,
                    character,
                },
            );
            let again = kind_at(&session, line, character);
            assert_eq!(first, again, "line {line}");
            assert_eq!(first, Some(SemanticTokenKind::Namespace), "line {line}");
        }
        // Reading the tokens twice gives the same answer too.
        assert_eq!(
            session.semantic_tokens(&uri()).unwrap(),
            session.semantic_tokens(&uri()).unwrap()
        );
    });
}

#[test]
fn a_statement_at_file_scope_does_not_hide_the_directives_after_it() {
    on_big_stack(|| {
        let text = "P :: #import \"Preload\";\n#assert P.Allocator == Allocator;\n\nfor 0..3 {}\n";
        let session = open(text);
        assert_eq!(
            kind_at(&session, 1, 8),
            Some(SemanticTokenKind::Namespace),
            "the alias in the assertion"
        );
        assert_eq!(kind_at(&session, 1, 10), Some(SemanticTokenKind::Type));
        assert!(
            messages(&session)
                .iter()
                .any(|m| m.contains("not allowed at file scope")),
            "{:?}",
            messages(&session)
        );
    });
}

#[test]
fn an_unfinished_call_of_a_type_has_the_same_color_whether_or_not_signature_help_ran() {
    on_big_stack(|| {
        let text = "Shared :: enum u32 {\n    IMPORTED :: 3;\n}\n\nShared(1, \n    x := 1";
        let at = Position {
            line: 4,
            character: 0,
        };
        let plain = open(text);
        let before = kind_at(&plain, at.line, at.character);
        let asked = open(text);
        let _ = asked.signature_help(
            &uri(),
            Position {
                line: 4,
                character: 7,
            },
        );
        assert_eq!(kind_at(&asked, at.line, at.character), before);
        assert_eq!(before, Some(SemanticTokenKind::Type));
    });
}

#[test]
fn a_called_name_in_a_body_that_cannot_be_checked_is_answered_without_a_token_request_first() {
    // `broken` has a parameter of an unknown type, so its body is never checked; the callee
    // `helper` is resolved on demand, by whichever request first needs it. Hover and definition
    // must not depend on the semantic-token request having been the one.
    let source = "helper :: (x: s64) -> s64 { return x; }\nbroken :: (a: Missing) { helper(1); }\nmain :: () { broken(0); }\n";
    let at = Position {
        line: 1,
        character: 28,
    };
    let session = open(source);
    let hover = session.hover(&uri(), at).unwrap().expect("a hover");
    assert!(
        !hover.contents.value.contains("Source syntax declaration"),
        "{}",
        hover.contents.value
    );
    assert!(
        hover.contents.value.contains("helper"),
        "{}",
        hover.contents.value
    );
    let session = open(source);
    let definition = session.definition(&uri(), at).unwrap();
    assert_eq!(definition.len(), 1);
    assert_eq!(definition[0].range.start.line, 0);
}

#[test]
fn a_struct_parameter_is_a_type_whether_or_not_the_struct_is_instantiated() {
    // Removing the only `Node(int)` must not turn the `T`s of the struct into variables, drop
    // their definition or degrade the field hovers.
    let declaration = "Node :: struct(T: Type) {\n    next: *Node(T);\n    value: T = 19;\n}\n\n";
    let used = format!(
        "{declaration}main :: () -> int {{\n    node: Node(int);\n    return node.value;\n}}\n"
    );
    let unused = format!("{declaration}main :: () -> int {{\n    return 1;\n}}\n");
    for source in [used, unused] {
        let session = open(&source);
        for (line, character) in [(0, 15), (1, 16), (2, 11)] {
            assert_eq!(
                kind_at(&session, line, character),
                Some(SemanticTokenKind::Type),
                "{line}:{character} in {source}"
            );
        }
        for (line, character) in [(1, 16), (2, 11)] {
            let at = Position {
                line,
                character,
            };
            let definition = session.definition(&uri(), at).unwrap();
            assert_eq!(definition.len(), 1, "{line}:{character} in {source}");
            assert_eq!(
                (
                    definition[0].range.start.line,
                    definition[0].range.start.character
                ),
                (0, 15),
                "{line}:{character} in {source}"
            );
            assert!(session.hover(&uri(), at).unwrap().is_some());
        }
        let field = session
            .hover(
                &uri(),
                Position {
                    line: 2,
                    character: 4,
                },
            )
            .unwrap()
            .expect("a hover");
        assert!(
            !field.contents.value.contains("Source syntax declaration"),
            "{}",
            field.contents.value
        );
    }
}

#[test]
fn a_loader_follows_the_recovered_view_of_a_loaded_file_that_says_so() {
    // Deleting the end of a block comment would make it swallow the rest of the loaded file,
    // and a fresh session opened on that text sees `Q` gone. While the comment is being edited
    // the session leaves it out instead, so what the loader uses of the file stays known, and
    // the loaded file reports the omission, since the loader has no diagnostic that could.
    let lib = "/* note */\nQ :: struct { value: int; }\n";
    let main = "#load \"lib.jai\";\nmain :: () { q: Q; q.value = 1; }\n";
    let mut session =
        common::virtual_session(&[("/p/main.jai", main), ("/p/lib.jai", lib)], &[], None);
    let (main_uri, lib_uri) = (
        common::file_uri("/p/main.jai"),
        common::file_uri("/p/lib.jai"),
    );
    session.open(main_uri.clone(), 1, main.into()).unwrap();
    session.open(lib_uri.clone(), 1, lib.into()).unwrap();
    let start = Position {
        line: 0,
        character: 7,
    };
    let end = Position {
        line: 0,
        character: 10,
    };
    session
        .change(
            &lib_uri,
            2,
            &[TextChange {
                range: Some(Range {
                    start,
                    end,
                }),
                range_length: None,
                text: String::new(),
            }],
        )
        .unwrap();
    let omitted: Vec<_> = session
        .diagnostics(&lib_uri)
        .unwrap()
        .into_iter()
        .map(|d| d.message)
        .collect();
    assert!(
        omitted
            .iter()
            .any(|m| m.starts_with("Incomplete lexical edit")),
        "{omitted:?}"
    );
    let loader: Vec<_> = session
        .diagnostics(&main_uri)
        .unwrap()
        .into_iter()
        .map(|d| d.message)
        .collect();
    assert!(loader.is_empty(), "{loader:?}");
}

#[test]
fn a_struct_argument_that_is_not_a_type_is_reported_at_the_use() {
    // `Node(1)` was reported on the parameter `T: Type` of the struct, far from what was typed,
    // so the struct's own line gained an error while a call was being written.
    let source =
        "Node :: struct(T: Type) {\n    value: T;\n}\n\nmain :: () {\n    node: Node(1);\n}\n";
    let session = open(source);
    let reported: Vec<_> = session
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.message.contains("expected a type"))
        .collect();
    assert!(!reported.is_empty());
    assert!(
        reported.iter().all(|d| d.range.start.line == 5),
        "{reported:?}"
    );
}

#[test]
fn a_field_default_has_the_same_color_whether_or_not_the_struct_is_used() {
    let declaration = "Explicit :: struct {\n    descriptor: Type = void;\n}\n";
    for main in [
        "main :: () {\n    e: Explicit;\n}\n",
        "main :: () {\n    x :\n    e: Explicit;\n}\n",
    ] {
        let source = format!("{declaration}{main}");
        assert_eq!(
            kind_at(&open(&source), 1, 23),
            Some(SemanticTokenKind::Type),
            "{source}"
        );
    }
    // As in compilation, a default nothing makes a value with is not reported.
    let unused = "Bad :: struct {\n    descriptor: Type = 3;\n}\nmain :: () {\n}\n";
    assert_eq!(messages(&open(unused)), Vec::<String>::new());
}

#[test]
fn a_constant_whose_run_fails_is_still_a_constant_where_it_is_used() {
    on_big_stack(|| {
        for body in ["return 1;", "assert(false); return 1;"] {
            let source = format!(
                "#import \"Basic\";\nf :: () -> int {{ {body} }}\nregistered :: #run f();\nalias :: registered;\nmain :: () {{ print(\"%\", alias); }}\n"
            );
            let session = open(&source);
            let token = session
                .semantic_tokens(&uri())
                .unwrap()
                .into_iter()
                .find(|t| t.position.line == 3 && t.position.character == 9)
                .expect("a token for `registered`");
            assert_eq!(
                (token.kind, token.readonly),
                (SemanticTokenKind::Variable, true),
                "{source}"
            );
            let definition = session
                .definition(
                    &uri(),
                    Position {
                        line: 3,
                        character: 9,
                    },
                )
                .unwrap();
            assert_eq!(
                definition
                    .iter()
                    .map(|l| l.range.start.line)
                    .collect::<Vec<_>>(),
                [2],
                "{source}"
            );
        }
    });
}

#[test]
fn a_member_of_a_constant_whose_run_fails_is_still_a_field() {
    on_big_stack(|| {
        let source = "#import \"Basic\";\nPair :: struct { data: int; }\nf :: () -> Pair { assert(false); return .{}; }\nregistered :: #run f();\nalias :: registered;\nmain :: () { print(\"%\", alias.data); }\n";
        let session = open(source);
        let token = session
            .semantic_tokens(&uri())
            .unwrap()
            .into_iter()
            .find(|t| t.position.line == 5 && t.position.character == 30)
            .expect("a token for `data`");
        assert_eq!(token.kind, SemanticTokenKind::Property);
        for line in [3, 4] {
            let at = Position {
                line,
                character: 0,
            };
            let hover = session.hover(&uri(), at).unwrap().expect("a hover");
            assert!(
                hover.contents.value.contains(":: Pair"),
                "{}",
                hover.contents.value
            );
        }
    });
}

#[test]
fn the_branches_of_an_if_whose_condition_fails_keep_their_tokens() {
    on_big_stack(|| {
        let source = "Pair :: struct { tag: int; }\nmain :: () {\n    p: Pair;\n    if missing == 1 {\n        p.tag = 2;\n    } else {\n        p.tag = 3;\n    }\n}\n";
        let session = open(source);
        let tokens = session.semantic_tokens(&uri()).unwrap();
        for line in [4, 6] {
            let token = tokens
                .iter()
                .find(|t| t.position.line == line && t.position.character == 10)
                .expect("a token for `tag`");
            assert_eq!(token.kind, SemanticTokenKind::Property, "line {line}");
        }
    });
}

#[test]
fn an_instance_nothing_else_uses_hovers_with_its_fields_the_first_time() {
    on_big_stack(|| {
        let source = "Box :: struct (T: Type) { item: T; extra: int; }\n#assert Box(u8) != Box(s8);\nmain :: () {}\n";
        let session = open(source);
        let at = Position {
            line: 1,
            character: 9,
        };
        let first = session.hover(&uri(), at).unwrap().expect("a hover");
        let second = session.hover(&uri(), at).unwrap().expect("a hover");
        assert!(
            first.contents.value.contains("item: u8;"),
            "{}",
            first.contents.value
        );
        assert_eq!(first.contents.value, second.contents.value);
    });
}

#[test]
fn a_binder_keeps_its_layout_while_its_instance_body_is_half_typed() {
    on_big_stack(|| {
        let source = "Set :: struct { count: u32; items: [4] u64; }\nadd :: (fd: u64, set: *$T) {\n    x := 1;\n    _\n}\nmain :: () { s: Set; add(1, *s); }\n";
        let session = open(source);
        let at = Position {
            line: 1,
            character: 25,
        };
        let hover = session.hover(&uri(), at).unwrap().expect("a hover");
        assert!(
            hover.contents.value.contains("size 40, align 8"),
            "{}",
            hover.contents.value
        );
    });
}

#[test]
fn a_field_of_a_constant_whose_run_fails_keeps_its_layout_and_members() {
    on_big_stack(|| {
        let source = "Point :: struct { n: int; tags: [] string; }\nf :: () -> Point { assert(false); return .{}; }\nbaked :: #run f();\nmain :: () { x := baked.tags.count; }\n";
        let session = open(source);
        assert_eq!(kind_at(&session, 3, 29), Some(SemanticTokenKind::Property));
        let at = Position {
            line: 3,
            character: 24,
        };
        let hover = session.hover(&uri(), at).unwrap().expect("a hover");
        assert!(
            hover.contents.value.contains("offset 8, size 16"),
            "{}",
            hover.contents.value
        );
    });
}

#[test]
fn a_polymorph_parameter_hovers_with_every_instance_type_in_a_fixed_order() {
    on_big_stack(|| {
        for calls in ["f(cast(s8) 1); f(null);", "f(null); f(cast(s8) 1);"] {
            let source = format!("f :: (value: $T) {{ x := value; }}\nmain :: () {{ {calls} }}\n");
            let session = open(&source);
            let at = Position {
                line: 0,
                character: 7,
            };
            let hover = session.hover(&uri(), at).unwrap().expect("a hover");
            assert_eq!(hover.contents.value, "value: *void\nvalue: s8", "{source}");
        }
    });
}

#[test]
fn a_member_in_an_uncalled_polymorph_with_a_baked_polymorphic_parameter_is_a_field() {
    on_big_stack(|| {
        let source = "App :: struct { marker: int; }\npost :: (app: *App, $handler: $H) {\n    handler();\n    app.marker = 1;\n}\nmain :: () {}\n";
        let session = open(source);
        eprintln!("TOK {:?}", session.semantic_tokens(&uri()).unwrap());
        eprintln!(
            "HOV {:?}",
            session.hover(
                &uri(),
                Position {
                    line: 3,
                    character: 9
                }
            )
        );
        assert_eq!(kind_at(&session, 3, 8), Some(SemanticTokenKind::Property));
    });
}
