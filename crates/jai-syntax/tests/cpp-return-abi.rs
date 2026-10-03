use jai_source::{LocatedDiagnostic, SourceMap, Symbols};
use jai_syntax::{
    Declaration, ExpressionKind, FileDeclarationKind, FileItem, ParsedFile, StatementKind,
    TypeSyntax,
};
use jai_types::{CallingConvention, ContextMode, ForeignReturnAbi};

fn parse(text: &str) -> Result<ParsedFile, LocatedDiagnostic> {
    let mut sources = SourceMap::default();
    let id = sources.insert("cpp-return.jai".into(), text.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default())
}

fn declaration(file: &ParsedFile, index: usize) -> &FileDeclarationKind {
    let FileItem::Declaration(declaration) = &file.items()[index] else {
        panic!("expected declaration")
    };
    &declaration.kind
}

#[test]
fn foreign_results_and_indirect_callable_types_retain_the_same_policy() {
    let file = parse(
        "Pair::struct{x:s32;y:s32;} ordinary::()->Pair #foreign native; make::()->Pair #cpp_return_type_is_non_pod #foreign native; Method::#type(this:*void)->Pair #cpp_method #cpp_return_type_is_non_pod; Callback::#type()->Pair #cpp_return_type_is_non_pod #c_call;",
    )
    .unwrap();

    let FileDeclarationKind::ProcedurePrototype(ordinary) = declaration(&file, 1) else {
        panic!("expected ordinary foreign prototype")
    };
    let FileDeclarationKind::ProcedurePrototype(make) = declaration(&file, 2) else {
        panic!("expected C++ foreign prototype")
    };
    assert_eq!(ordinary.return_abi, ForeignReturnAbi::Natural);
    assert_eq!(make.return_abi, ForeignReturnAbi::CppNonPod);
    assert_eq!(make.convention, CallingConvention::C);
    assert_eq!(make.context, ContextMode::None);

    for (index, convention) in [(3, CallingConvention::CppMethod), (4, CallingConvention::C)] {
        let FileDeclarationKind::TypeAlias(alias) = declaration(&file, index) else {
            panic!("expected callable type alias")
        };
        let TypeSyntax::Procedure(signature) = &alias.ty else {
            panic!("expected callable type")
        };
        assert_eq!(signature.return_abi, make.return_abi);
        assert_eq!(signature.convention, convention);
        assert_eq!(signature.context, ContextMode::None);
    }
}

#[test]
fn named_and_anonymous_source_headers_preserve_the_return_policy() {
    let file = parse(
        "Pair::struct{x:s32;y:s32;} named::()->Pair #c_call #cpp_return_type_is_non_pod{return Pair.{1,2};} holder::(){callback:=()->Pair #cpp_return_type_is_non_pod #c_call{return Pair.{1,2};};}",
    )
    .unwrap();
    let FileDeclarationKind::Procedure(named) = declaration(&file, 1) else {
        panic!("expected named source procedure")
    };
    let FileDeclarationKind::Procedure(holder) = declaration(&file, 2) else {
        panic!("expected procedure containing callback")
    };
    let StatementKind::Declare(Declaration::Inferred {
        initializer, ..
    }) = &holder.body[0].kind
    else {
        panic!("expected callback declaration")
    };
    let ExpressionKind::AnonymousProcedure(callback) = &initializer.kind else {
        panic!("expected anonymous source procedure")
    };
    assert_eq!(named.return_abi, ForeignReturnAbi::CppNonPod);
    assert_eq!(callback.return_abi, named.return_abi);
    assert_eq!(callback.convention, named.convention);
    assert_eq!(callback.body.len(), 1);
}

#[test]
fn duplicate_return_markers_are_reported_at_the_second_marker() {
    let source = "make::()->Pair #cpp_return_type_is_non_pod #c_call #cpp_return_type_is_non_pod #foreign native;";
    let error = parse(source).unwrap_err();
    assert_eq!(error.message, "duplicate #cpp_return_type_is_non_pod");
    assert_eq!(
        error.location.span.start as usize,
        source.rfind("#cpp_return_type_is_non_pod").unwrap()
    );
}
