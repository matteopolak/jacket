//! Original standalone IDE fixtures, independent of any library implementation.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Session};

fn session() -> Session {
    Session::with_environment(
        Limits::default(),
        Environment {
            fs: std::rc::Rc::new(jaic::sema::NativeFs),
            options: Box::new(|_| {
                let mut options = jaic::sema::Options::host();
                options.runtime_support = false;
                options.stack_trace = false;
                options
            }),
        },
    )
}

fn uri() -> DocumentUri {
    DocumentUri::parse("file:///partial-edits/main.jai").unwrap()
}

fn at(text: &str, needle: &str) -> Position {
    position(text, text.find(needle).unwrap())
}

fn position(text: &str, byte: usize) -> Position {
    let prefix = &text[..byte];
    Position {
        line: prefix.matches('\n').count() as u32,
        character: prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    }
}

#[test]
fn custom_for_keyword_uses_selected_overload_without_changing_operand_reference() {
    let text = r#"For_Flags :: u8;
Bag :: struct { count: int; }
Other :: struct { count: int; }
for_expansion :: (b: Other, body: Code, flags: For_Flags) #expand {
    for 0..b.count { `it := 1; #insert body; }
}
for_expansion :: (b: *Bag, body: Code, flags: For_Flags) #expand {
    for 0..b.count { `it := 2; #insert body; }
}
main :: () {
    collection: Bag;
    for item: collection { result := item; }
    for n: 0..3 { result := n; }
    array: [3] int;
    for value: array { result := value; }
}
"#;
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    assert!(
        s.diagnostics(&uri())
            .unwrap()
            .iter()
            .all(|d| d.severity != jai_language_server::DiagnosticSeverity::Error),
        "{:?}",
        s.diagnostics(&uri())
    );
    let defs = s.definition(&uri(), at(text, "for item")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "for_expansion :: (b: *Bag"));
    let keyword_byte = text.find("for item").unwrap();
    for delta in 1..3 {
        assert_eq!(
            s.definition(&uri(), position(text, keyword_byte + delta))
                .unwrap(),
            defs,
            "every character of the keyword must select the same expansion"
        );
    }
    let defs = s.definition(&uri(), at(text, "collection {")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "collection: Bag"));
    for needle in ["for n:", "for value:"] {
        assert!(s.definition(&uri(), at(text, needle)).unwrap().is_empty());
    }
}

#[test]
fn partial_statement_preserves_syntax_and_semantics_after_it() {
    let text = "Record :: struct { field: int; }\nmain :: () {\n    before := 10;\n    broken := (\n    after: Record;\n    answer := before + after.field;\n}\nlast :: () {}\n";
    assert!(jaic::parser::parse_file(jaic::source::FileId(0), text).is_err());
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let symbols = s.document_symbols(&uri()).unwrap();
    assert!(symbols.iter().any(|sym| sym.name == "last"), "{symbols:?}");
    let hover = s
        .hover(&uri(), at(text, "after.field"))
        .unwrap()
        .expect("typed after");
    assert!(
        hover.contents.value.starts_with("after: Record"),
        "{hover:?}"
    );
    let defs = s.definition(&uri(), at(text, "before +")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "before :="));
    let items = s
        .completion(
            &uri(),
            position(text, text.find("after.field").unwrap() + 6),
        )
        .unwrap();
    assert!(items.items.iter().any(|i| i.label == "field"), "{items:?}");
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .all(|d| (3..=4).contains(&d.range.start.line)),
        "{diagnostics:?}"
    );
}

#[test]
fn named_generic_aliased_and_explicit_custom_iterations_resolve_actual_procedures() {
    for (declaration, extra, loop_header) in [
        (
            "walk :: (b: Bag, body: Code, flags: For_Flags)",
            "",
            "for :walk item: collection",
        ),
        (
            "walk :: (b: *Bag, body: Code, flags: For_Flags)",
            "",
            "for :walk item: collection",
        ),
        (
            "walk :: (b: $T, body: Code, flags: For_Flags)",
            "",
            "for :walk item: collection",
        ),
        (
            "walk :: (b: Bag, body: Code, flags: For_Flags)",
            "alias :: walk;",
            "for :alias item: collection",
        ),
        (
            "walk :: (b: Bag, body: Code, flags: For_Flags)",
            "for_expansion :: walk;",
            "for item: collection",
        ),
        (
            "walk :: (b: Bag, body: Code, flags: For_Flags)",
            "",
            "for item: walk(collection)",
        ),
        (
            "walk :: (b: *$T, body: Code, flags: For_Flags)",
            "",
            "for :walk item: collection",
        ),
    ] {
        let text = format!(
            "For_Flags :: u8;\nBag :: struct {{ count: int; }}\n{declaration} #expand {{\n    for 0..b.count {{ `it := 7; #insert body; }}\n}}\n{extra}\nmain :: () {{\n    collection: Bag;\n    {loop_header} {{ result := item; }}\n}}\n"
        );
        let mut s = session();
        s.open(uri(), 1, text.clone()).unwrap();
        let errors: Vec<_> = s
            .diagnostics(&uri())
            .unwrap()
            .into_iter()
            .filter(|d| d.severity == jai_language_server::DiagnosticSeverity::Error)
            .collect();
        assert!(errors.is_empty(), "{loop_header}: {errors:?}");
        let defs = s.definition(&uri(), at(&text, loop_header)).unwrap();
        assert_eq!(defs.len(), 1, "{loop_header}: {defs:?}");
        assert_eq!(defs[0].range.start, at(&text, declaration), "{loop_header}");
    }
}

#[test]
fn imported_custom_iteration_navigates_to_its_source() {
    let imported = DocumentUri::parse("file:///partial-edits/iterator.jai").unwrap();
    let library = "Bag :: struct { count: int; }\nFor_Flags :: u8;\nfor_expansion :: (b: Bag, body: Code, flags: For_Flags) #expand {\n    for 0..b.count { `it := 7; #insert body; }\n}\n";
    let text = "#import,file \"iterator.jai\";\nmain :: () {\n    collection: Bag;\n    for item: collection { result := item; }\n}\n";
    let mut s = session();
    s.open(imported.clone(), 1, library.into()).unwrap();
    s.open(uri(), 1, text.into()).unwrap();
    let defs = s.definition(&uri(), at(text, "for item")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}; {:?}", s.diagnostics(&uri()));
    assert_eq!(defs[0].uri, imported.as_str());
    assert_eq!(defs[0].range.start, at(library, "for_expansion"));

    // The macro is found through the collection type's home scope, without
    // importing its name into the caller's visible overload set.
    let qualified = text
        .replace(
            "#import,file \"iterator.jai\";",
            "P :: #import,file \"iterator.jai\";",
        )
        .replace("collection: Bag", "collection: P.Bag");
    s.change(
        &uri(),
        2,
        &[jai_language_server::TextChange {
            range: None,
            range_length: None,
            text: qualified.clone(),
        }],
    )
    .unwrap();
    let defs = s.definition(&uri(), at(&qualified, "for item")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].uri, imported.as_str());
    assert_eq!(defs[0].range.start, at(library, "for_expansion"));

    let incomplete = library.replace("`it := 7;", "`it := (");
    s.change(
        &imported,
        2,
        &[jai_language_server::TextChange {
            range: None,
            range_length: None,
            text: incomplete.clone(),
        }],
    )
    .unwrap();
    let defs = s.definition(&uri(), at(&qualified, "for item")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].uri, imported.as_str());
    assert_eq!(defs[0].range.start, at(&incomplete, "for_expansion"));
}

#[test]
fn semantic_error_keeps_later_independent_locals_and_reports_both_errors() {
    let text = "main :: () {\n    broken := missing_name;\n    healthy := 42;\n    result := healthy + 1;\n    another := other_missing;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = s
        .hover(&uri(), at(text, "healthy +"))
        .unwrap()
        .expect("healthy hover");
    assert_eq!(hover.contents.value, "healthy: s64");
    let defs = s.definition(&uri(), at(text, "healthy +")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "healthy :="));
    let errors: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == jai_language_server::DiagnosticSeverity::Error)
        .collect();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors.iter().any(|d| d.message.contains("missing_name")));
    assert!(errors.iter().any(|d| d.message.contains("other_missing")));
}

#[test]
fn missing_close_brace_keeps_procedure_locals_and_next_global_separate() {
    let text = "main :: () {\n    local := 42;\n    result := local + 1;\nnext :: () {\n    local := false;\n    result := local;\n}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let symbols = s.document_symbols(&uri()).unwrap();
    assert!(symbols.iter().any(|sym| sym.name == "next"), "{symbols:?}");
    let hover = s
        .hover(&uri(), at(text, "local +"))
        .unwrap()
        .expect("local hover");
    assert_eq!(hover.contents.value, "local: s64");
    let defs = s.definition(&uri(), at(text, "local;\n}")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "local := false"));
}

#[test]
fn recovered_or_semantically_invalid_bodies_cannot_run_at_compile_time() {
    use jaic::sema::{Compiler, Options, VirtualFs, ide::IdeFacts};
    use std::{path::Path, rc::Rc};
    for broken in [
        "bad := (",
        "bad := missing_name;",
        "same :: 1; same :: 2;",
        "push_context,defer_pop unknown;",
    ] {
        let source = format!(
            "Context :: struct {{ state: int; }}\ndamaged :: () -> int {{\n    {broken}\n    healthy := 42;\n    return healthy;\n}}\n#run damaged();\n"
        );
        for editor in [false, true] {
            let mut fs = VirtualFs::default();
            fs.insert("/partial-edits/main.jai", source.as_bytes());
            let mut options = Options::host();
            options.runtime_support = false;
            options.stack_trace = false;
            let mut compiler = Compiler::new(options, Rc::new(fs));
            if editor {
                compiler.ide = Some(Box::new(IdeFacts::new(vec!["/partial-edits/".into()])));
            }
            assert!(
                compiler
                    .compile_program(Path::new("/partial-edits/main.jai"))
                    .is_err(),
                "invalid body executed: {source}"
            );
        }
    }
    let source = "damaged :: () -> int #expand {\n    bad := (\n    return 42;\n}\ncaller :: () -> int { return damaged(); }\n#run caller();\n";
    let mut fs = VirtualFs::default();
    fs.insert("/partial-edits/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler.ide = Some(Box::new(IdeFacts::new(vec!["/partial-edits/".into()])));
    assert!(
        compiler
            .compile_program(Path::new("/partial-edits/main.jai"))
            .is_err(),
        "a recovered macro ran as valid code"
    );
}

#[test]
fn unicode_incremental_edit_and_restore_never_reuse_the_old_definition() {
    use jai_language_server::{Range, TextChange};
    let good = "main :: () {\n    label := \"😀é\"; value := 1;\n    result := value + 1;\n}\nlast :: () {}\n";
    let mut s = session();
    s.open(uri(), 1, good.into()).unwrap();
    assert_eq!(s.definition(&uri(), at(good, "value +")).unwrap().len(), 1);
    let line = good.lines().nth(1).unwrap();
    let start = line.find("value").unwrap();
    let cut = Position {
        line: 1,
        character: line[..start].encode_utf16().count() as u32,
    };
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: Some(Range {
                start: cut,
                end: Position {
                    line: 1,
                    character: line.encode_utf16().count() as u32,
                },
            }),
            range_length: Some(line[start..].encode_utf16().count() as u32),
            text: "bad := (".into(),
        }],
    )
    .unwrap();
    assert!(
        s.definition(&uri(), at(good, "value +"))
            .unwrap()
            .is_empty()
    );
    assert!(
        s.document_symbols(&uri())
            .unwrap()
            .iter()
            .any(|sym| sym.name == "last")
    );
    let fixed = good
        .replace("value := 1", "value := false")
        .replace("value + 1", "value");
    s.change(
        &uri(),
        3,
        &[TextChange {
            range: None,
            range_length: None,
            text: fixed.clone(),
        }],
    )
    .unwrap();
    let hover = s
        .hover(&uri(), at(&fixed, "value;\n}"))
        .unwrap()
        .expect("current type");
    assert_eq!(hover.contents.value, "value: bool");
    let defs = s.definition(&uri(), at(&fixed, "value;\n}")).unwrap();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].range.start, at(&fixed, "value := false"));
}

#[test]
fn incremental_damage_to_opaque_lexemes_keeps_their_contents_opaque() {
    use jai_language_server::TextChange;
    for (opaque, damage) in [
        ("/*\nfake :: () {}\n*/", "/*\nfake :: () {}\n*"),
        (
            "text :: #string END\nfake :: () {}\nEND",
            "text :: #string EN\nfake :: () {}\nEND",
        ),
        ("text :: \"hello\";", "text :: \"hel"),
    ] {
        let good = format!(
            "{opaque}\nmain :: () {{\n    healthy := 42;\n    result := healthy + 1;\n}}\n"
        );
        let bad = good.replacen(opaque, damage, 1);
        let mut s = session();
        s.open(uri(), 1, good.clone()).unwrap();
        assert!(
            s.document_symbols(&uri())
                .unwrap()
                .iter()
                .any(|sym| sym.name == "main")
        );
        s.change(
            &uri(),
            2,
            &[TextChange {
                range: None,
                range_length: None,
                text: bad.clone(),
            }],
        )
        .unwrap();
        let symbols = s.document_symbols(&uri()).unwrap();
        assert!(
            symbols.iter().any(|sym| sym.name == "main"),
            "{damage}: {symbols:?}"
        );
        assert!(
            !symbols.iter().any(|sym| sym.name == "fake"),
            "exposed opaque contents: {damage}"
        );
        let hover = s
            .hover(&uri(), at(&bad, "healthy +"))
            .unwrap()
            .expect("unaffected local");
        assert_eq!(hover.contents.value, "healthy: s64", "{damage}");
        assert!(
            !s.diagnostics(&uri()).unwrap().is_empty(),
            "damage lost diagnostics"
        );
        s.change(
            &uri(),
            3,
            &[TextChange {
                range: None,
                range_length: None,
                text: good.clone(),
            }],
        )
        .unwrap();
        assert!(
            s.document_symbols(&uri())
                .unwrap()
                .iter()
                .any(|sym| sym.name == "main")
        );
    }
}

#[test]
fn lexical_recovery_keeps_same_line_unicode_definition_positions() {
    use jai_language_server::TextChange;
    let good = "/*漢😀é*/ main :: () { healthy := 42; result := healthy + 1; }\n";
    let bad = good.replacen("*/", "*", 1);
    let mut s = session();
    s.open(uri(), 1, good.into()).unwrap();
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: bad.clone(),
        }],
    )
    .unwrap();
    let defs = s.definition(&uri(), at(&bad, "healthy +")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(&bad, "healthy :="));
}

#[test]
fn excessive_nesting_recovers_the_next_declaration_without_building_a_deep_tree() {
    // Match the existing core parser depth test and production compiler stack.
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            let text = format!("damaged :: () {{ {}\nlast :: () {{}}\n", "{".repeat(4000));
            let parsed = jaic::parser::parse_file_ide(jaic::source::FileId(0), &text);
            assert!(
                parsed
                    .diagnostics
                    .iter()
                    .any(|d| d.message.contains("nested too deeply"))
            );
            assert!(parsed.file.stmts.iter().any(
        |s| matches!(&s.kind, jaic::ast::StmtKind::Decl(d) if d.names[0].name.as_str() == "last")
    ));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn custom_keyword_selection_survives_a_partial_body_and_clears_after_operand_change() {
    use jai_language_server::TextChange;
    let good = "For_Flags :: u8;\nBag :: struct { count: int; }\nfor_expansion :: (b: Bag, body: Code, flags: For_Flags) #expand {\n    for 0..b.count { `it := 1; #insert body; }\n}\nmain :: () {\n    collection: Bag;\n    for item: collection {\n        result := item;\n    }\n}\n";
    let mut s = session();
    s.open(uri(), 1, good.into()).unwrap();
    let bad = good.replace("result := item;", "result := (");
    s.change(
        &uri(),
        2,
        &[TextChange {
            range: None,
            range_length: None,
            text: bad.clone(),
        }],
    )
    .unwrap();
    let defs = s.definition(&uri(), at(&bad, "for item")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(&bad, "for_expansion"));
    let array = good.replace("collection: Bag;", "collection: [3] int;");
    s.change(
        &uri(),
        3,
        &[TextChange {
            range: None,
            range_length: None,
            text: array.clone(),
        }],
    )
    .unwrap();
    assert!(
        s.definition(&uri(), at(&array, "for item"))
            .unwrap()
            .is_empty()
    );
    let operand = s.definition(&uri(), at(&array, "collection {")).unwrap();
    assert_eq!(operand.len(), 1);
    assert_eq!(operand[0].range.start, at(&array, "collection: [3]"));
}

#[test]
fn generic_loop_site_retains_all_actual_selected_expansions() {
    let text = "For_Flags :: u8;\nBag :: struct { count: int; }\nOther :: struct { count: int; }\nfor_expansion :: (b: Bag, body: Code, flags: For_Flags) #expand { for 0..b.count { `it := 1; #insert body; } }\nfor_expansion :: (b: Other, body: Code, flags: For_Flags) #expand { for 0..b.count { `it := 2; #insert body; } }\nvisit :: (bag: $T) { for item: bag { result := item; } }\nmain :: () { first: Bag; second: Other; visit(first); visit(second); }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let defs = s.definition(&uri(), at(text, "for item")).unwrap();
    let starts: std::collections::BTreeSet<_> = defs.iter().map(|d| d.range.start).collect();
    assert_eq!(
        starts,
        [
            at(text, "for_expansion :: (b: Bag"),
            at(text, "for_expansion :: (b: Other")
        ]
        .into_iter()
        .collect()
    );

    let generic = "For_Flags :: u8;\nBag :: struct { count: int; }\nOther :: struct { count: int; }\nfor_expansion :: (b: $T, body: Code, flags: For_Flags) #expand { for 0..b.count { `it := 1; #insert body; } }\nvisit :: (bag: $T) { for item: bag { result := item; } }\nmain :: () { first: Bag; second: Other; visit(first); visit(second); }\n";
    s.change(
        &uri(),
        2,
        &[jai_language_server::TextChange {
            range: None,
            range_length: None,
            text: generic.into(),
        }],
    )
    .unwrap();
    let defs = s.definition(&uri(), at(generic, "for item")).unwrap();
    assert_eq!(
        defs.len(),
        1,
        "instances of one generic source declaration: {defs:?}"
    );
    assert_eq!(defs[0].range.start, at(generic, "for_expansion"));
}

#[test]
fn fresh_unfinished_quote_does_not_hide_recovered_identifier_tokens() {
    let text = "broken :: \"unfinished\nmain :: () { healthy := 42; result := healthy + 1; }\nother :: \"closed\";\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let hover = s
        .hover(&uri(), at(text, "healthy +"))
        .unwrap()
        .expect("recovered identifier token");
    assert_eq!(hover.contents.value, "healthy: s64");
    let diagnostics = s.diagnostics(&uri()).unwrap();
    assert!(
        diagnostics
            .iter()
            .filter(|d| d.severity == jai_language_server::DiagnosticSeverity::Error)
            .all(|d| d.range.start.line == 0),
        "{diagnostics:?}"
    );
}

#[test]
fn missing_type_brace_recovers_global_variables_and_operators() {
    let text = "Record :: struct {\n    field: int;\nsettings: int;\noperator == :: (a: Record, b: Record) -> bool { return true; }\nmain :: () { value := settings; }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let symbols = s.document_symbols(&uri()).unwrap();
    assert!(
        symbols.iter().any(|sym| sym.name == "settings"),
        "{symbols:?}"
    );
    assert!(
        symbols.iter().any(|sym| sym.name == "operator=="),
        "{symbols:?}"
    );
}

#[test]
fn explicit_block_closer_keeps_unindented_locals_in_their_scopes() {
    let text = "global := false;\nmain :: () {\nglobal := 42;\nbroken := (\nresult := global + 1;\n}\nlast :: () {}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let defs = s.definition(&uri(), at(text, "global +")).unwrap();
    assert_eq!(defs.len(), 1, "{defs:?}");
    assert_eq!(defs[0].range.start, at(text, "global := 42"));
}

#[test]
fn partial_single_line_control_body_does_not_consume_the_next_procedure() {
    for header in ["if true", "for 0..1", "while true", "defer"] {
        let text = format!(
            "first :: () {{ {header}\nsecond :: () {{ healthy := 42; result := healthy + 1; }}\n"
        );
        let mut s = session();
        s.open(uri(), 1, text.clone()).unwrap();
        let symbols = s.document_symbols(&uri()).unwrap();
        assert!(
            symbols.iter().any(|sym| sym.name == "second"),
            "{header}: {symbols:?}"
        );
        let hover = s
            .hover(&uri(), at(&text, "healthy +"))
            .unwrap()
            .expect("second procedure typechecking");
        assert_eq!(hover.contents.value, "healthy: s64");
    }
}

#[test]
fn missing_brace_keeps_flagged_procedures_and_file_scope_markers_outside() {
    let text = "first :: () {\n    local := 42;\n#program_export \"api\"\nsecond :: () #c_call {}\n#scope_file\nstate: int;\nlast :: () {}\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let symbols = s.document_symbols(&uri()).unwrap();
    for name in ["second", "state", "last"] {
        assert!(symbols.iter().any(|sym| sym.name == name), "{symbols:?}");
    }
}

#[test]
fn different_real_instantiation_errors_at_one_source_site_are_not_discarded() {
    let text = "A :: struct {}\nB :: struct {}\nbad :: (v: $T) { result := v.missing; }\nmain :: () { a: A; b: B; bad(a); bad(b); }\n";
    let mut s = session();
    s.open(uri(), 1, text.into()).unwrap();
    let errors: Vec<_> = s
        .diagnostics(&uri())
        .unwrap()
        .into_iter()
        .filter(|d| d.severity == jai_language_server::DiagnosticSeverity::Error)
        .collect();
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("`A`") && d.message.contains("missing")),
        "{errors:?}"
    );
    assert!(
        errors
            .iter()
            .any(|d| d.message.contains("`B`") && d.message.contains("missing")),
        "{errors:?}"
    );
}

#[test]
fn declaration_hoisting_and_context_errors_keep_independent_later_locals() {
    for damaged in ["same :: 1; same :: 2;", "push_context,defer_pop unknown;"] {
        let text = format!(
            "Context :: struct {{ state: int; }}\nmain :: () {{\n    {damaged}\n    healthy := 42;\n    result := healthy + 1;\n}}\n"
        );
        let mut s = session();
        s.open(uri(), 1, text.clone()).unwrap();
        let hover = s
            .hover(&uri(), at(&text, "healthy +"))
            .unwrap()
            .expect("independent local after a hoisting/context error");
        assert_eq!(hover.contents.value, "healthy: s64", "{damaged}");
        let defs = s.definition(&uri(), at(&text, "healthy +")).unwrap();
        assert_eq!(defs.len(), 1, "{damaged}: {defs:?}");
        assert_eq!(defs[0].range.start, at(&text, "healthy :="));
        assert!(
            s.diagnostics(&uri())
                .unwrap()
                .iter()
                .any(|d| d.severity == jai_language_server::DiagnosticSeverity::Error),
            "{damaged}: real errors must remain"
        );
    }
}
