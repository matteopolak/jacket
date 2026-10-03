use jai_source::{SourceMap, Symbols};
use jai_syntax::{ExpressionKind, FileDeclarationKind, FileItem, StatementKind};
use jai_types::ProcedureExecution;

fn parse(source: &str) -> Result<jai_syntax::ParsedFile, jai_source::LocatedDiagnostic> {
    let mut sources = SourceMap::default();
    let id = sources.insert("/phase.jai".into(), source.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default())
}

#[test]
fn body_phase_predicate_is_distinct_from_the_procedure_execution_attribute() {
    for (attribute, execution) in [
        ("", ProcedureExecution::RuntimeAndCompileTime),
        ("#compile_time", ProcedureExecution::CompileTimeOnly),
        (
            "#no_context #compile_time",
            ProcedureExecution::CompileTimeOnly,
        ),
        (
            "#compile_time #no_context",
            ProcedureExecution::CompileTimeOnly,
        ),
    ] {
        let parsed = parse(&format!(
            "phase :: ()->bool {attribute} {{return #compile_time;}}"
        ))
        .unwrap();
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        assert_eq!(procedure.execution, execution);
        let StatementKind::Return(Some(value)) = &procedure.body[0].kind else {
            panic!()
        };
        assert!(matches!(value.kind, ExpressionKind::CompileTimePredicate));
    }
}

#[test]
fn bodyless_and_duplicate_execution_attributes_have_precise_diagnostics() {
    for (source, expected) in [
        (
            "f :: () #compile_time #compile_time {}",
            "duplicate #compile_time",
        ),
        ("f :: () #compile_time #compiler;", "source procedure body"),
        ("f :: () #compile_time #foreign;", "source procedure body"),
        ("f: ()->bool #compile_time;", "source body contract"),
    ] {
        let error = parse(source).unwrap_err();
        assert!(error.message.contains(expected), "{error:?}");
    }
}

#[test]
fn legacy_scalar_parser_requires_the_typed_execution_phase_pipeline() {
    let error = jai_syntax::parse("phase :: ()->bool {return #compile_time;}").unwrap_err();
    assert!(
        error.message.contains("typed execution phase resolution"),
        "{error:?}"
    );
}
