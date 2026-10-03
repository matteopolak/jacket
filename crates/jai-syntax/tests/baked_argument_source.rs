//! Whole source partial-call ASTs preserve actual producer syntax and use sites.
use jai_source::{SourceMap, Symbols};
use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, StatementKind};

#[test]
fn complete_source_keeps_named_bound_formals_and_later_procedure_body() {
    let text = "PARTIAL::#bake_arguments Provider.worker(minimum_digits=2); main::()->int{local::#bake_arguments callbacks[0](value=40); return local(2);}";
    let mut sources = SourceMap::default();
    let id = sources.insert("baked-source.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    assert_eq!(parsed.items().len(), 2);
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!()
    };
    let ExpressionKind::BakeArguments(value) = &constant.initializer.kind else {
        panic!()
    };
    assert_eq!(
        value.span.text(text),
        "#bake_arguments Provider.worker(minimum_digits=2)"
    );
    assert_eq!(value.callee.span.text(text), "Provider.worker");
    assert_eq!(
        value.call_span.text(text),
        "Provider.worker(minimum_digits=2)"
    );
    assert_eq!(
        symbols.name(value.arguments[0].name.unwrap()),
        "minimum_digits"
    );
    assert_eq!(value.arguments[0].value.span.text(text), "2");
    let FileItem::Declaration(declaration) = &parsed.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    assert_eq!(procedure.body.len(), 2);
    let StatementKind::Constant(local) = &procedure.body[0].kind else {
        panic!()
    };
    let ExpressionKind::BakeArguments(value) = &local.initializer.kind else {
        panic!()
    };
    assert!(matches!(value.callee.kind, ExpressionKind::Index { .. }));
    assert_eq!(value.callee.span.text(text), "callbacks[0]");
    assert!(matches!(procedure.body[1].kind, StatementKind::Return(_)));
}

#[test]
fn later_invocation_belongs_to_outer_expression_and_malformed_bakes_stay_errors() {
    let text = "main::()->int{return (#bake_arguments worker(value=40))(2);}";
    let mut sources = SourceMap::default();
    let id = sources.insert("baked-use.jai".into(), text.into());
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let StatementKind::Return(values) = &procedure.body[0].kind else {
        panic!()
    };
    let ExpressionKind::IndirectCall {
        callee,
        args,
    } = &values.as_ref().unwrap().kind
    else {
        panic!()
    };
    let ExpressionKind::BakeArguments(value) = &callee.kind else {
        panic!()
    };
    assert_eq!(value.arguments[0].value.span.text(text), "40");
    assert_eq!(args[0].value.span.text(text), "2");
    for invalid in [
        "PARTIAL::#bake_arguments worker;",
        "PARTIAL::#bake_arguments worker(value=40,,context=0);",
        "PARTIAL::#bake_arguments 40;",
    ] {
        let id = sources.insert("bad-baked.jai".into(), invalid.into());
        assert!(jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
    }
}

#[test]
fn callback_input_defaults_do_not_consume_the_outer_variable_initializer() {
    let text = "Partial::#type(value:int=21)->int; sample:(value:int)->bool = null;";
    let mut sources = SourceMap::default();
    let id = sources.insert("callback-defaults.jai".into(), text.into());
    let parsed = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(alias) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::TypeAlias(alias) = &alias.kind else {
        panic!()
    };
    let jai_syntax::TypeSyntax::Procedure(signature) = &alias.ty else {
        panic!()
    };
    assert_eq!(
        signature.parameters[0]
            .default
            .as_ref()
            .unwrap()
            .span
            .text(text),
        "21"
    );
    assert!(signature.results[0].default.is_none());
    let FileItem::Declaration(global) = &parsed.items()[1] else {
        panic!()
    };
    let FileDeclarationKind::Global(global) = &global.kind else {
        panic!()
    };
    let jai_syntax::Declaration::UnresolvedExplicit {
        ty,
        initializer: Some(initializer),
        ..
    } = &global.declaration
    else {
        panic!()
    };
    let jai_syntax::TypeSyntax::Procedure(signature) = ty else {
        panic!()
    };
    assert!(signature.parameters[0].default.is_none());
    assert!(signature.results[0].default.is_none());
    assert!(matches!(initializer.kind, ExpressionKind::Null));
    assert_eq!(initializer.span.text(text), "null");
}
