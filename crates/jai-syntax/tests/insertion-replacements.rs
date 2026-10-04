use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CodeBody, FileDeclarationKind, FileItem, JumpKind, LoopControlReplacementBody, StatementKind,
};

fn parse(text: &str) -> Result<jai_syntax::ParsedFile, jai_source::LocatedDiagnostic> {
    let mut sources = SourceMap::default();
    let id = sources.insert("replacement.jai".into(), text.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default())
}

#[test]
fn replacement_bodies_preserve_their_original_spans_and_typed_jump_kinds() {
    let text = "main :: () { #insert (remove={entry.hash=1; map.count-=1;}, break=break row, continue=#assert(false)) body; }";
    let parsed = parse(text).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &declaration.kind else {
        panic!()
    };
    let StatementKind::Insert(directive) = &main.body[0].kind else {
        panic!()
    };
    assert_eq!(directive.replacements.len(), 3);
    assert_eq!(directive.replacements[0].kind, JumpKind::Remove);
    assert_eq!(
        directive.replacements[0].span.text(text),
        "remove={entry.hash=1; map.count-=1;}"
    );
    let LoopControlReplacementBody::Code(CodeBody::Block(statements)) =
        &directive.replacements[0].body
    else {
        panic!()
    };
    assert_eq!(statements[1].span.text(text), "map.count-=1;");
    let LoopControlReplacementBody::Code(CodeBody::Statement(statement)) =
        &directive.replacements[1].body
    else {
        panic!()
    };
    assert_eq!(statement.span.text(text), "break row");
    let LoopControlReplacementBody::Assert {
        span,
        condition,
        ..
    } = &directive.replacements[2].body
    else {
        panic!()
    };
    assert_eq!(span.text(text), "#assert(false)");
    assert_eq!(condition.span.text(text), "(false)");
}

#[test]
fn malformed_replacement_names_and_duplicates_reject_at_the_modifier() {
    for (text, expected, token) in [
        (
            "main::(){#insert(remove={},remove={}) body;}",
            "duplicate loop-control insertion replacement",
            "remove",
        ),
        (
            "main::(){#insert(return={}) body;}",
            "insertion replacement must name break, continue, or remove",
            "return",
        ),
    ] {
        let error = parse(text).unwrap_err();
        assert_eq!(error.message, expected);
        assert_eq!(error.location.span.text(text), token);
    }
}

#[test]
fn parenthesized_code_values_still_parse_without_replacement_modifiers() {
    let parsed = parse("main::(){#insert (body);}").unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(main) = &declaration.kind else {
        panic!()
    };
    let StatementKind::Insert(directive) = &main.body[0].kind else {
        panic!()
    };
    assert!(directive.replacements.is_empty());
}

#[test]
#[ignore = "requires the separately supplied reference and pinned upstream source corpus"]
fn unchanged_upstream_collection_files_parse_with_real_replacement_forms() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut failures = Vec::new();
    for relative in [
        "corpus/upstream/focus-editor--focus/src/utils/array.jai",
        "corpus/upstream/focus-editor--focus/src/utils/ring_buffer.jai",
        "corpus/upstream/ostef--Vk-Engine/Modules/Hash_Map.jai",
    ] {
        let path = root.join(relative);
        let text = std::fs::read_to_string(&path).unwrap();
        let mut sources = SourceMap::default();
        let id = sources.insert(path, text);
        let result = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default());
        if let Err(error) = result {
            failures.push(format!("{relative}: {error:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
