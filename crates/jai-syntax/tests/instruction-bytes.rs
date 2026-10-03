use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind, FileItem, MAX_INSTRUCTION_BYTES, StatementKind};
#[path = "support/optional_sources.rs"]
mod optional_sources;

#[test]
fn pinned_arm_breakpoint_bytes_are_decoded_and_keep_the_entire_source_span() {
    let Some(upstream) = optional_sources::read_optional(
        "corpus/upstream/focus-editor--focus/modules/Runtime_Support.jai",
    ) else {
        return;
    };
    let statement = upstream
        .lines()
        .find(|line| line.contains("#bytes") && line.contains("BRK 0x01"))
        .unwrap()
        .split("//")
        .next()
        .unwrap()
        .trim();
    let text = format!("// é before offsets\nmain :: () {{ {statement} }}");
    let module = jai_syntax::parse(&text).unwrap();
    let source_statement = &module.procedures()[0].body[0];
    let StatementKind::InstructionBytes(instruction) = &source_statement.kind else {
        panic!("expected instruction bytes")
    };
    assert_eq!(&*instruction.bytes, &[0x20, 0x00, 0x20, 0xd4]);
    assert_eq!(instruction.span, source_statement.span);
    assert_eq!(instruction.span.text(&text), statement);
}

#[test]
fn self_authored_breakpoint_byte_literals_and_source_spans_always_parse() {
    let text = "// é before offsets\nmain :: () { #bytes .[32,0,32,212,]; }";
    let module = jai_syntax::parse(text).unwrap();
    let statement = &module.procedures()[0].body[0];
    let StatementKind::InstructionBytes(instruction) = &statement.kind else {
        panic!("expected instruction bytes")
    };
    assert_eq!(&*instruction.bytes, &[0x20, 0, 0x20, 0xd4]);
    assert_eq!(instruction.span, statement.span);
    assert_eq!(instruction.span.text(text), "#bytes .[32,0,32,212,];");
}

#[test]
fn arbitrary_payloads_and_optional_trailing_commas_are_retained_before_selection() {
    for (statement, expected) in [
        ("#bytes .[];", Vec::new()),
        (
            "#bytes .[0, 255, 0xff, 0b0100_0001,];",
            vec![0, 255, 255, 65],
        ),
        (
            "#bytes .[0x3f,0x20,0x03,0xd5];",
            vec![0x3f, 0x20, 0x03, 0xd5],
        ),
    ] {
        let text = format!("main :: () {{ #if false {{ {statement} }} }}");
        let mut sources = SourceMap::default();
        let source = sources.insert("inactive-bytes.jai".into(), text.clone());
        let file =
            jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("expected declaration")
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!("expected procedure")
        };
        let StatementKind::CompileTimeIf { then_body, .. } = &procedure.body[0].kind else {
            panic!("expected inactive branch")
        };
        let StatementKind::InstructionBytes(instruction) = &then_body[0].kind else {
            panic!("expected retained instruction bytes")
        };
        assert_eq!(&*instruction.bytes, expected.as_slice());
        assert_eq!(instruction.span.text(&text), statement);
    }
}

#[test]
fn byte_literals_must_be_numeric_integers_in_range() {
    for literal in ["256", "0x100", "-1", "1.0", "VALUE", "0b2", "0x_"] {
        let text = format!("main :: () {{ #bytes .[{literal}]; }}");
        let diagnostic = jai_syntax::parse(&text).unwrap_err();
        assert!(
            diagnostic.message.contains("u8 integer literals"),
            "{literal}: {diagnostic}"
        );
        assert!(diagnostic.span.start >= text.find(literal).unwrap());
    }
}

#[test]
fn instruction_byte_arrays_require_complete_delimiters_and_a_terminator() {
    for statement in [
        "#bytes [0];",
        "#bytes .[0;",
        "#bytes .[0, }",
        "#bytes .[0 1];",
        "#bytes .[0]; } #bytes .[",
        "#bytes .[0]",
    ] {
        let text = format!("main :: () {{ {statement} }}");
        assert!(jai_syntax::parse(&text).is_err(), "{statement}");
    }
}

#[test]
fn instruction_byte_payloads_have_an_explicit_size_limit() {
    let payload = std::iter::repeat_n("0", MAX_INSTRUCTION_BYTES)
        .collect::<Vec<_>>()
        .join(",");
    let valid = format!("main :: () {{ #bytes .[{payload},]; }}");
    jai_syntax::parse(&valid).unwrap();
    let invalid = format!("main :: () {{ #bytes .[{payload},0]; }}");
    let diagnostic = jai_syntax::parse(&invalid).unwrap_err();
    assert!(diagnostic.message.contains("exceeds 4096 bytes"));
    assert_eq!(diagnostic.span.text(&invalid), "0");
}
