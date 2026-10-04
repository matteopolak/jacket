//! Independent fixtures preserve source metadata and assertion operands without evaluation.
use jai_source::{SourceMap, Symbols};
use jai_syntax::{FileDeclarationKind as D, FileItem, NoteValue, StatementKind as S};

fn parse(text: &str) -> (jai_syntax::ParsedFile, Symbols) {
    let mut sources = SourceMap::default();
    let id = sources.insert("notes-assertions.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    (file, symbols)
}

#[test]
fn notes_keep_quoted_names_selectors_and_both_nominal_positions() {
    let text = r#"Record::struct @Before #align 8 @"Display label" { item:int; @selector(setItem:forIndex:) } @After
        Choice::enum @Before u8 { first; @deprecated second::9; } @After"#;
    let (file, symbols) = parse(text);
    let FileItem::Declaration(record) = &file.items()[0] else {
        panic!()
    };
    let D::Record(record) = &record.kind else {
        panic!()
    };
    assert_eq!(
        record
            .notes
            .iter()
            .map(|n| symbols.name(n.name))
            .collect::<Vec<_>>(),
        ["Before", "Display label", "After"]
    );
    assert_eq!(record.notes[1].span.text(text), "@\"Display label\"");
    let NoteValue::Selector(selector) =
        &record.fields().next().unwrap().notes[0].arguments[0].value
    else {
        panic!()
    };
    assert_eq!(
        selector
            .components
            .iter()
            .map(|s| symbols.name(*s))
            .collect::<Vec<_>>(),
        ["setItem", "forIndex"]
    );
    assert_eq!(selector.span.text(text), "setItem:forIndex:");
    let FileItem::Declaration(choice) = &file.items()[1] else {
        panic!()
    };
    let D::Enum(choice) = &choice.kind else {
        panic!()
    };
    assert_eq!(choice.notes.len(), 2);
    let mut members = jai_syntax::EnumMemberSyntax::new(&choice.members);
    assert_eq!(
        members.next().unwrap().notes[0].span.text(text),
        "@deprecated"
    );
    assert!(members.next().unwrap().notes.is_empty());
}

#[test]
fn alignment_after_an_initializer_keeps_the_real_operand_and_duplicate_guard() {
    let text = "R::struct{value:u64=--- #align 4; other:int #align 8=1;}";
    let (file, _) = parse(text);
    let FileItem::Declaration(record) = &file.items()[0] else {
        panic!()
    };
    let D::Record(record) = &record.kind else {
        panic!()
    };
    let jai_syntax::FieldAttribute::Alignment(value) =
        &record.fields().next().unwrap().attributes[0]
    else {
        panic!()
    };
    assert_eq!(value.span.text(text), "4");
    let mut sources = SourceMap::default();
    let text = "R::struct{value:int #align 8=1 #align 16;}";
    let id = sources.insert("duplicate-align.jai".into(), text.into());
    let error =
        jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
    assert!(error.message.contains("duplicate field alignment"));
}

#[test]
fn comma_messages_and_lazy_replacement_messages_remain_distinct_from_list_delimiters() {
    let text = "#assert (1==1), Message; main::(){ #assert true, Message; #insert(remove=#assert false \"stop\", break={}) body; }";
    let (file, _) = parse(text);
    let FileItem::Assert {
        condition,
        message,
        ..
    } = &file.items()[0]
    else {
        panic!()
    };
    assert_eq!(condition.span.text(text), "(1==1)");
    assert_eq!(message.as_ref().unwrap().span.text(text), "Message");
    let FileItem::Declaration(main) = &file.items()[1] else {
        panic!()
    };
    let D::Procedure(main) = &main.kind else {
        panic!()
    };
    let S::Insert(insert) = &main.body[1].kind else {
        panic!()
    };
    assert_eq!(insert.replacements.len(), 2);
    let jai_syntax::LoopControlReplacementBody::Assert {
        condition,
        message,
        span,
    } = &insert.replacements[0].body
    else {
        panic!()
    };
    assert_eq!(condition.span.text(text), "false");
    assert_eq!(message.as_ref().unwrap().span.text(text), "\"stop\"");
    assert_eq!(span.text(text), "#assert false \"stop\"");
}

#[test]
fn selector_storage_and_enum_notes_are_charged_before_metadata_cloning() {
    let (short, _) = parse("Choice::enum{item;} @selector(one:)");
    let (long, _) = parse("Choice::enum{item;} @selector(one:two:three:four:five:)");
    let cost = |file: &jai_syntax::ParsedFile| {
        let mut total = 0usize;
        file.visit_retained_metadata(&mut |work, bytes| {
            total += work + bytes;
            Ok::<_, ()>(())
        })
        .unwrap();
        total
    };
    assert!(cost(&long) > cost(&short));
    let mut remaining = cost(&short);
    assert!(
        long.visit_retained_metadata(&mut |work, bytes| {
            let demand = work + bytes;
            if demand > remaining {
                Err(())
            } else {
                remaining -= demand;
                Ok(())
            }
        })
        .is_err()
    );
}

#[test]
fn commas_inside_nested_aggregate_conditions_are_not_message_separators() {
    let text = r#"#assert (int.[1,2][0]==1), "array condition";
        #assert (R.{first=1,second=2}.first==1), "record condition";
        R::struct{first:int;second:int;}"#;
    let (file, _) = parse(text);
    for (item, expected) in file.items()[..2].iter().zip([
        ("(int.[1,2][0]==1)", "\"array condition\""),
        ("(R.{first=1,second=2}.first==1)", "\"record condition\""),
    ]) {
        let FileItem::Assert {
            condition,
            message,
            ..
        } = item
        else {
            panic!()
        };
        assert_eq!(condition.span.text(text), expected.0);
        assert_eq!(message.as_ref().unwrap().span.text(text), expected.1);
    }
}
