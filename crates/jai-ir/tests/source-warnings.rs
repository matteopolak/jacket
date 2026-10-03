use jai_ir::{ProgramBuilder, SourceWarnings};
use jai_source::{
    SourceMap, SourceSpan, SourceWarning, SourceWarningKind, Span, WarningLocation, WarningNote,
};

fn warning(sources: &SourceMap) -> SourceWarning {
    let source = &sources.records()[0];
    let at = |start, end| {
        WarningLocation::new(
            source,
            SourceSpan {
                source: source.id(),
                span: Span::new(start, end),
            },
        )
        .unwrap()
    };
    SourceWarning::new(
        SourceWarningKind::DeprecatedProcedureReference,
        at(0, 3),
        "procedure 'old' is deprecated",
        vec![WarningNote {
            location: at(4, 15),
            message: "declared deprecated here".into(),
        }],
    )
}
fn sources() -> SourceMap {
    let mut sources = SourceMap::default();
    sources.insert("warnings.jai".into(), "old #deprecated".into());
    sources
}

#[test]
fn warnings_publish_without_debug_information_or_runtime_procedure_ids() {
    let library = {
        let sources = sources();
        let checked = SourceWarnings::checked(vec![warning(&sources)], &sources).unwrap();
        ProgramBuilder::new(jai_types::TypeRegistry::new().freeze().unwrap())
            .source_warnings(checked)
            .finish_library()
            .unwrap()
    };
    assert!(library.debug_sources().is_none());
    assert!(library.procedures().is_empty());
    assert!(
        library.source_warnings()[0]
            .render()
            .contains("warnings.jai:1:1: warning:")
    );
    assert_eq!(
        library.source_warnings()[0].notes()[0].location.text(),
        "#deprecated"
    );
}

#[test]
fn publication_rejects_foreign_source_allocations_with_equal_ids_and_text() {
    let original = sources();
    let foreign = sources();
    assert!(SourceWarnings::checked(vec![warning(&foreign)], &original).is_err());
}

#[test]
fn retained_warning_limit_is_explicit_instead_of_silently_truncating() {
    let sources = sources();
    assert!(
        SourceWarnings::checked(
            vec![warning(&sources); jai_source::MAX_SOURCE_WARNINGS + 1],
            &sources
        )
        .is_err()
    );
}
