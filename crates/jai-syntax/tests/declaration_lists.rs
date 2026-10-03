//! Declaration lists retain written names and one common initializer owner.
use jai_source::{SourceMap, Symbols};
use jai_syntax::{Declaration, ExpressionKind, FileDeclarationKind, FileItem, StatementKind};
use std::sync::Arc;

fn parse(text: &str) -> jai_syntax::ParsedFile {
    let mut sources = SourceMap::default();
    let id = sources.insert("lists.jai".into(), text.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
}

#[test]
fn file_groups_have_distinct_names_and_shared_original_source_syntax() {
    let text = "first, second, third: int = produce() + 1;";
    let file = parse(text);
    let members: Vec<_> = file
        .items()
        .iter()
        .map(|item| {
            let FileItem::Declaration(declaration) = item else {
                panic!("declaration")
            };
            let FileDeclarationKind::Global(global) = &declaration.kind else {
                panic!("global")
            };
            let Declaration::GroupMember {
                name,
                ordinal,
                group,
            } = &global.declaration
            else {
                panic!("shared source member")
            };
            assert_eq!(group.names()[*ordinal].0, *name);
            assert_eq!(
                declaration.location.span.start,
                group.names()[*ordinal].1.start
            );
            assert_eq!(global.span.end, text.len());
            (*ordinal, group)
        })
        .collect();
    assert_eq!(
        members
            .iter()
            .map(|(ordinal, _)| *ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(Arc::ptr_eq(members[0].1, members[1].1));
    assert!(Arc::ptr_eq(members[1].1, members[2].1));
    assert_eq!(
        members[0].1.initializer_for(0).unwrap().span.text(text),
        "produce() + 1"
    );
    assert!(std::ptr::eq(
        members[0].1.initializer_for(0).unwrap(),
        members[2].1.initializer_for(2).unwrap()
    ));
    let clone = file.clone();
    let FileItem::Declaration(declaration) = &clone.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Global(global) = &declaration.kind else {
        panic!()
    };
    let Declaration::GroupMember {
        group, ..
    } = &global.declaration
    else {
        panic!()
    };
    assert!(Arc::ptr_eq(group, members[0].1));
}

#[test]
fn file_explicit_initializer_lists_retain_each_written_expression_once() {
    let text = "a, b, c := 1, false, \"retained\";";
    let file = parse(text);
    let FileItem::Declaration(declaration) = &file.items()[2] else {
        panic!()
    };
    let FileDeclarationKind::Global(global) = &declaration.kind else {
        panic!()
    };
    let Declaration::GroupMember {
        group,
        ordinal,
        ..
    } = &global.declaration
    else {
        panic!()
    };
    assert_eq!(*ordinal, 2);
    assert_eq!(group.extra_initializers().len(), 2);
    assert!(
        group.retained_owner_byte_bound()
            >= std::mem::size_of::<jai_syntax::DeclarationGroup>()
                + 2 * std::mem::size_of::<usize>()
    );
    assert_eq!(group.initializer_for(1).unwrap().span.text(text), "false");
    assert_eq!(
        group.initializer_for(2).unwrap().span.text(text),
        "\"retained\""
    );
    let mut bytes = 0;
    file.visit_retained_metadata(&mut |_, amount| {
        bytes += amount;
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(
        bytes
            >= group.extra_initializers().capacity()
                * std::mem::size_of::<jai_syntax::Expression>()
                + 8
    );
}

#[test]
fn local_uninitialized_lists_and_call_results_use_existing_typed_nodes() {
    let file = parse("main::(){first,second:int=---; left,right:=pair();}");
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    assert!(
        matches!(&procedure.body[0].kind, StatementKind::DeclareResults { names, ty:Some(_), values }
        if names.len()==2 && matches!(values.as_slice(), [value] if matches!(value.kind, ExpressionKind::Uninitialized)))
    );
    assert!(
        matches!(&procedure.body[1].kind, StatementKind::DeclareResults { names, ty:None, values }
        if names.len()==2 && values.len()==1 && matches!(values[0].kind, ExpressionKind::Call(..)))
    );
}

#[test]
fn grouped_record_names_retain_their_actual_source_start() {
    let text = "Point::struct {x, y, z: int;}";
    let file = parse(text);
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Record(record) = &declaration.kind else {
        panic!()
    };
    assert_eq!(
        record
            .fields()
            .map(|field| field.span.text(text))
            .collect::<Vec<_>>(),
        ["x, y, z: int;", "y, z: int;", "z: int;"]
    );
}

#[test]
fn mixed_uninitialized_values_and_initializer_arity_are_rejected() {
    for text in ["a,b:int=---,1;", "a,b:int=1,2,3;", "a,b:int,1;"] {
        let mut sources = SourceMap::default();
        let id = sources.insert("bad.jai".into(), text.into());
        assert!(
            jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err(),
            "{text}"
        );
    }
}
