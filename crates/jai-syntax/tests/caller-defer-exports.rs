use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CodeBody, Declaration, ExpressionKind, FileDeclarationKind, FileItem, StatementKind,
};

#[test]
fn allocator_macro_preserves_outer_and_deferred_statement_ranges() {
    let text = "push_allocator::(allocator:Allocator)#expand#no_debug{old_allocator:=context.allocator;context.allocator=allocator;`defer context.allocator=old_allocator;}";
    let mut sources = SourceMap::default();
    let id = sources.insert("caller-cleanup.jai".into(), text.into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let export = &procedure.body[2];
    assert_eq!(
        export.span.text(text),
        "`defer context.allocator=old_allocator;"
    );
    let StatementKind::CallerExport(deferred) = &export.kind else {
        panic!()
    };
    assert_eq!(
        deferred.span.text(text),
        "defer context.allocator=old_allocator;"
    );
    let StatementKind::Defer(body) = &deferred.kind else {
        panic!()
    };
    assert_eq!(body[0].span.text(text), "context.allocator=old_allocator;");
    assert!(matches!(body[0].kind, StatementKind::AssignPlace { .. }));
}

#[test]
fn quoted_exported_defers_keep_a_structural_caller_export() {
    let text = "macro::()#expand{cleanup:=#code `defer restore(saved);}";
    let mut sources = SourceMap::default();
    let id = sources.insert("quoted-caller-cleanup.jai".into(), text.into());
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let StatementKind::Declare(Declaration::Inferred {
        initializer, ..
    }) = &procedure.body[0].kind
    else {
        panic!()
    };
    let ExpressionKind::Code(CodeBody::Statement(export)) = &initializer.kind else {
        panic!()
    };
    assert_eq!(export.span.text(text), "`defer restore(saved);");
    assert!(matches!(
        &export.kind,
        StatementKind::CallerExport(statement)
            if matches!(statement.kind, StatementKind::Defer(_))
    ));
}
