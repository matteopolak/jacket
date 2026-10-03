//! Authored caller-return proof and opt-in local original-source compatibility.
use jai_source::{SourceMap, Symbols};
use jai_syntax::{
    CodeBody, Declaration, ExpressionKind, FileDeclarationKind, FileItem, StatementKind,
};

#[test]
fn caller_returns_keep_inner_and_outer_ranges_inside_conditionals() {
    let text = "finish::()#expand{if failed {`return status, value;} `return;}";
    let mut sources = SourceMap::default();
    let source = sources.insert("caller-return.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!();
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!();
    };
    let StatementKind::If(_, body, _) = &procedure.body[0].kind else {
        panic!();
    };
    assert_eq!(body[0].span.text(text), "`return status, value;");
    let StatementKind::CallerExport(inner) = &body[0].kind else {
        panic!();
    };
    assert_eq!(inner.span.text(text), "return status, value;");
    assert!(matches!(&inner.kind, StatementKind::ReturnValues(values) if values.len() == 2));
    assert!(
        matches!(&procedure.body[1].kind, StatementKind::CallerExport(inner) if matches!(inner.kind, StatementKind::Return(None)))
    );
}

#[test]
fn quoted_caller_return_keeps_its_explicit_export_marker() {
    let text = "finish::()#expand{code:=#code `return 42;}";
    let mut sources = SourceMap::default();
    let source = sources.insert("quoted-caller-return.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!();
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!();
    };
    let StatementKind::Declare(Declaration::Inferred { initializer, .. }) = &procedure.body[0].kind
    else {
        panic!();
    };
    let ExpressionKind::Code(CodeBody::Statement(statement)) = &initializer.kind else {
        panic!();
    };
    assert_eq!(statement.span.text(text), "`return 42;");
    assert!(
        matches!(&statement.kind, StatementKind::CallerExport(inner) if matches!(inner.kind, StatementKind::Return(Some(_))))
    );
}

#[test]
fn authored_scalar_macro_retains_caller_return_after_runtime_branch() {
    let text = "choose_exit::(primary:int,alternative:int)#expand{chosen:=alternative;if primary>0 {chosen=primary+1;} `return chosen;}";
    let mut sources = SourceMap::default();
    let source = sources.insert("authored-scalar-caller-return.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!();
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!();
    };
    assert!(matches!(&procedure.body[1].kind, StatementKind::If(..)));
    let statement = procedure.body.last().unwrap();
    assert_eq!(statement.span.text(text), "`return chosen;");
    assert!(
        matches!(&statement.kind, StatementKind::CallerExport(inner) if matches!(inner.kind, StatementKind::Return(Some(_))))
    );
}

#[test]
#[ignore = "requires local licensed reference input; run explicitly with --ignored"]
fn local_original_apollo_conversion_macro_parses_without_rewriting_its_caller_return() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let source_text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "local original Apollo input unavailable at {}: {error}",
            path.display()
        )
    });
    let start = source_text.find("ConvertToApollo ::").unwrap();
    let end = source_text[start..].find("seconds_to_apollo ::").unwrap() + start;
    let text = &source_text[start..end];
    let mut sources = SourceMap::default();
    let source = sources.insert("Apollo_Time.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let procedure = file
        .items()
        .iter()
        .find_map(|item| {
            let FileItem::Declaration(declaration) = item else {
                return None;
            };
            let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
                return None;
            };
            (symbols.name(procedure.name) == "ConvertToApollo").then_some(procedure)
        })
        .expect("actual supplied macro");
    let statement = procedure.body.last().unwrap();
    assert_eq!(statement.span.text(text), "`return result;");
    assert!(
        matches!(&statement.kind, StatementKind::CallerExport(inner) if matches!(inner.kind, StatementKind::Return(Some(_))))
    );
}

#[test]
fn caller_exports_reject_backticked_loop_keywords_from_the_supplied_lexer_contract() {
    for text in [
        "macro::()#expand{`break;}",
        "macro::()#expand{`continue;}",
        "macro::()#expand{`remove row;}",
    ] {
        let mut sources = SourceMap::default();
        let source = sources.insert("unsupported-caller-keyword.jai".into(), text.into());
        let error = jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default())
            .unwrap_err();
        assert_eq!(
            error.message,
            "a caller export requires a declaration, defer, or return"
        );
        assert_eq!(
            error.location.span.text(text).chars().next(),
            Some(if text.contains("break") {
                'b'
            } else if text.contains("continue") {
                'c'
            } else {
                'r'
            })
        );
    }
}
