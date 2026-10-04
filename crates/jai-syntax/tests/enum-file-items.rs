use jai_source::{SourceMap, Symbols};
use jai_syntax::{EnumBodyItem, FileDeclarationKind, FileItem, PokedName, RecordMember};
fn parse(text: &str) -> jai_syntax::ParsedFile {
    let mut sources = SourceMap::default();
    let id = sources.insert("enum-items.jai".into(), text.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
}
#[test]
fn enum_retains_conditional_branches_generator_and_exact_spans() {
    let text = "E::enum{A::7;#if true{B;}else #if false{C;}else{D;}#insert -> string{return \"GENERATED;\";}}";
    let parsed = parse(text);
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Enum(enumeration) = &declaration.kind else {
        panic!()
    };
    let [
        EnumBodyItem::Member(first),
        EnumBodyItem::Conditional {
            then_items,
            else_items,
            span,
            ..
        },
        EnumBodyItem::Insert(generator),
    ] = enumeration.members.as_slice()
    else {
        panic!()
    };
    assert_eq!(first.span.text(text), "A::7;");
    assert_eq!(then_items[0].span().text(text), "B;");
    assert!(matches!(&else_items[0], EnumBodyItem::Conditional { .. }));
    assert_eq!(span.text(text), "#if true{B;}else #if false{C;}else{D;}");
    assert_eq!(
        generator.span.text(text),
        "#insert -> string{return \"GENERATED;\";}"
    );
    assert_eq!(
        jai_syntax::EnumMemberSyntax::new(&enumeration.members).count(),
        4
    );
}
#[test]
fn file_operations_and_record_imports_remain_distinct_source_items() {
    let text = "#system_library,link_always \"user32\"; #poke_name Basic operator==; #if false { compiler_report(\"unsupported\"); } Owner::struct{#if false{#import \"Windows\";}value:int;}";
    let parsed = parse(text);
    assert!(
        matches!(&parsed.items()[0], FileItem::Library { declaration, .. } if declaration.options.link_always && declaration.target == "user32")
    );
    assert!(
        matches!(&parsed.items()[1], FileItem::PokeName { directive, .. } if matches!(directive.name, PokedName::Operator(_)))
    );
    let FileItem::Conditional {
        then_items, ..
    } = &parsed.items()[2]
    else {
        panic!()
    };
    assert!(
        matches!(&then_items[0], FileItem::Execute { expression, .. } if expression.span.text(text) == "compiler_report(\"unsupported\")")
    );
    let FileItem::Declaration(declaration) = &parsed.items()[3] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    let RecordMember::Conditional {
        then_members, ..
    } = &record.members[0]
    else {
        panic!()
    };
    assert!(
        matches!(&then_members[0], RecordMember::Import(import) if import.span.text(text) == "#import \"Windows\";")
    );
}
#[test]
fn here_string_import_preserves_body_bytes_and_rejects_arbitrary_computed_paths() {
    let text = "#import,string #string END\nanswer::42;\nEND\n;";
    let parsed = parse(text);
    let FileItem::Import(import) = &parsed.items()[0] else {
        panic!()
    };
    assert_eq!(import.target, "answer::42;\n");
    assert_eq!(import.location.span.text(text), text);
    for source in [
        "E::enum{A}",
        "#poke_name Basic;",
        "#library,unknown \"x\";",
        "#if true{value = 42;}",
        "#import,file #string END\na.jai\nEND\n;",
        "#import,string compute();",
    ] {
        let mut sources = SourceMap::default();
        let id = sources.insert("negative.jai".into(), source.into());
        assert!(
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err(),
            "{source}"
        );
    }
}

#[test]
fn conditional_enum_member_notes_retain_spans_and_charge_inactive_source() {
    let plain = parse("E::enum{#if true{A;}else{B;}}");
    let text = r#"E::enum @Nominal {;#if true{A; @Selected("short")}else{B; @Inactive("a retained payload in an inactive branch")};} @Suffix"#;
    let decorated = parse(text);
    let FileItem::Declaration(declaration) = &decorated.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Enum(enumeration) = &declaration.kind else {
        panic!()
    };
    assert_eq!(enumeration.notes.len(), 2);
    let mut members = jai_syntax::EnumMemberSyntax::new(&enumeration.members);
    let first = members.next().unwrap();
    let second = members.next().unwrap();
    assert_eq!(first.notes[0].span.text(text), "@Selected(\"short\")");
    assert_eq!(
        second.notes[0].span.text(text),
        "@Inactive(\"a retained payload in an inactive branch\")"
    );
    assert!(members.next().is_none());
    let cost = |file: &jai_syntax::ParsedFile| {
        let mut bytes = 0;
        file.visit_retained_metadata(&mut |_, retained| {
            bytes += retained;
            Ok::<_, ()>(())
        })
        .unwrap();
        bytes
    };
    let plain_bytes = cost(&plain);
    assert!(cost(&decorated) > plain_bytes);
    let mut remaining = plain_bytes;
    assert!(
        decorated
            .visit_retained_metadata(&mut |_, bytes| -> Result<(), ()> {
                remaining = remaining.checked_sub(bytes).ok_or(())?;
                Ok(())
            })
            .is_err()
    );
}
