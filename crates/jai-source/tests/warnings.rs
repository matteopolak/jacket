use jai_source::{
    SourceMap, SourceSpan, SourceWarning, SourceWarningKind, Span, WarningLocation,
    WarningLocationError, WarningNote,
};

#[test]
fn warnings_survive_graph_drop_and_keep_both_original_source_sites() {
    let warning = {
        let mut sources = SourceMap::default();
        let call = sources.insert("caller.jai".into(), "é\r\nold();".into());
        let declaration = sources.insert("api.jai".into(), "old::() #deprecated {}".into());
        let location = WarningLocation::new(
            sources.get(call).unwrap(),
            SourceSpan {
                source: call,
                span: Span::new(4, 7),
            },
        )
        .unwrap();
        let target = WarningLocation::new(
            sources.get(declaration).unwrap(),
            SourceSpan {
                source: declaration,
                span: Span::new(8, 19),
            },
        )
        .unwrap();
        assert!(location.matches_source(sources.get(call).unwrap()));
        assert_eq!(location.text(), "old");
        assert_eq!(location.line(), 2);
        assert_eq!(location.column(), 1);
        assert_eq!(location.path(), std::path::Path::new("caller.jai"));
        assert!(location.same_source(&location.clone()));
        let _retained = location.shared_text();
        SourceWarning::new(
            SourceWarningKind::DeprecatedProcedureReference,
            location,
            "procedure 'old' is deprecated",
            vec![WarningNote {
                location: target,
                message: "declared deprecated here".into(),
            }],
        )
    };
    assert_eq!(
        warning.kind(),
        SourceWarningKind::DeprecatedProcedureReference
    );
    assert_eq!(warning.location().span().span, Span::new(4, 7));
    assert_eq!(warning.message(), "procedure 'old' is deprecated");
    assert_eq!(warning.notes().len(), 1);
    assert_eq!(
        warning.render(),
        "caller.jai:2:1: warning: procedure 'old' is deprecated\napi.jai:1:9: note: declared deprecated here"
    );
}

#[test]
fn equal_source_ids_paths_and_bytes_do_not_replace_original_allocation_identity() {
    let mut first = SourceMap::default();
    let mut second = SourceMap::default();
    let a = first.insert("same.jai".into(), "same".into());
    let b = second.insert("same.jai".into(), "same".into());
    assert_eq!(a, b);
    let span = SourceSpan {
        source: a,
        span: Span::new(0, 4),
    };
    let location = WarningLocation::new(first.get(a).unwrap(), span).unwrap();
    let other = WarningLocation::new(second.get(b).unwrap(), span).unwrap();
    assert!(!location.matches_source(second.get(b).unwrap()));
    assert!(!location.same_source(&other));
}

#[test]
fn construction_rejects_wrong_source_and_invalid_unicode_boundaries() {
    let mut sources = SourceMap::default();
    let id = sources.insert("a.jai".into(), "é".into());
    let other = sources.insert("b.jai".into(), "b".into());
    assert_eq!(
        WarningLocation::new(
            sources.get(id).unwrap(),
            SourceSpan {
                source: other,
                span: Span::new(0, 1)
            }
        )
        .unwrap_err(),
        WarningLocationError::WrongSource
    );
    assert_eq!(
        WarningLocation::new(
            sources.get(id).unwrap(),
            SourceSpan {
                source: id,
                span: Span::new(1, 2)
            }
        )
        .unwrap_err(),
        WarningLocationError::InvalidSpan
    );
}
