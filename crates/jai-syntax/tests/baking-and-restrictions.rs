use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    FileDeclarationKind, FileItem, ParameterBaking, ParameterBinding, TypeRestrictionSyntax,
    TypeSyntax, parse_file,
};

#[test]
fn source_formals_distinguish_required_optional_and_ordinary_baking() {
    let text = "inspect::(ordinary:int,$required:int,$$optional:int){}";
    let mut sources = SourceMap::default();
    let source = sources.insert("baking-formals.jai".into(), text.into());
    let parsed = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    assert_eq!(
        procedure
            .parameters
            .iter()
            .map(|parameter| parameter.baking)
            .collect::<Vec<_>>(),
        vec![
            ParameterBaking::None,
            ParameterBaking::Required,
            ParameterBaking::Optional
        ]
    );
    assert_eq!(procedure.parameters[2].span.text(text), "$$optional:int");
}

#[test]
fn nested_nominal_restrictions_and_interfaces_keep_original_leaf_spans() {
    let text = "nominal::(value:*[..]*[3]$T/Containers.Blentity){} interface_match::(value:$T/interface Matchable){}";
    let mut sources = SourceMap::default();
    let source = sources.insert("restricted-formals.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let ParameterBinding::RequiredType(TypeSyntax::Pointer(array)) =
        &procedure.parameters[0].binding
    else {
        panic!()
    };
    let TypeSyntax::DynamicArray(pointer) = array.as_ref() else {
        panic!()
    };
    let TypeSyntax::Pointer(array) = pointer.as_ref() else {
        panic!()
    };
    let TypeSyntax::FixedArray {
        element, ..
    } = array.as_ref()
    else {
        panic!()
    };
    let TypeSyntax::Restricted {
        variable,
        restriction: TypeRestrictionSyntax::Nominal(ty),
        span,
    } = element.as_ref()
    else {
        panic!()
    };
    assert_eq!(symbols.name(*variable), "T");
    assert_eq!(span.text(text), "$T/Containers.Blentity");
    let TypeSyntax::Named(path) = ty.as_ref() else {
        panic!()
    };
    assert_eq!(symbols.name(path.root), "Containers");
    assert_eq!(symbols.name(path.members[0]), "Blentity");
    let FileItem::Declaration(declaration) = &parsed.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    assert!(
        matches!(&procedure.parameters[0].binding, ParameterBinding::RequiredType(TypeSyntax::Restricted { restriction: TypeRestrictionSyntax::Interface(_), span, .. }) if span.text(text) == "$T/interface Matchable")
    );
}

#[test]
fn executing_restrictions_and_repeated_baking_markers_are_not_erased() {
    for (text, spelling, message) in [
        (
            "bad::(value:$T/#run build()){}",
            "#run",
            "compile-time execution is not permitted in a type restriction",
        ),
        (
            "bad::($$$value:int){}",
            "$",
            "a parameter baking marker must be '$' or '$$'",
        ),
    ] {
        let mut sources = SourceMap::default();
        let source = sources.insert("invalid-formal-policy.jai".into(), text.into());
        let error = parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.location.span.text(text), spelling);
        assert_eq!(error.message, message);
    }
}
