use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, TypeSyntax};

#[test]
fn builtin_pointer_aliases_are_definite_types_and_named_pointers_keep_ambiguity() {
    let text = "marg_list :: *void; PointerPointer :: **void; BytePointer :: *u8; NamedPointer :: *Node; Address :: *value; IMP :: #type () -> void #c_call;";
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("aliases.jai".into(), text.into());
    let parsed = jai_syntax::parse_file(
        sources.get(source).unwrap(),
        &mut jai_source::Symbols::default(),
    )
    .unwrap();
    let declarations: Vec<_> = parsed
        .items()
        .iter()
        .map(|item| {
            let FileItem::Declaration(declaration) = item else {
                panic!()
            };
            &declaration.kind
        })
        .collect();
    for declaration in &declarations[..3] {
        assert!(
            matches!(declaration, FileDeclarationKind::TypeAlias(alias) if matches!(alias.ty, TypeSyntax::Pointer(_)))
        );
    }
    for declaration in &declarations[3..5] {
        assert!(
            matches!(declaration, FileDeclarationKind::Constant(value) if matches!(value.initializer.kind, ExpressionKind::AddressOf(_)))
        );
    }
    assert!(
        matches!(declarations[5], FileDeclarationKind::TypeAlias(alias) if matches!(&alias.ty, TypeSyntax::Procedure(signature) if signature.results.len()==1 && matches!(signature.results[0].ty, TypeSyntax::Builtin(jai_syntax::BuiltinType::Void))))
    );
}
