use jai_source::{SourceMap, SourceRecordKind, SourceTextSnapshot, Span};
use std::{path::Path, sync::Arc};

#[test]
fn equal_text_and_equal_dense_ids_do_not_grant_allocation_identity() {
    let mut first = SourceMap::default();
    let mut second = SourceMap::default();
    let a = first.insert("same.jai".into(), "answer :: 42;".into());
    let b = second.insert("same.jai".into(), "answer :: 42;".into());
    assert_eq!(a, b);
    let span = Span::new(0, 13);
    let first_owner = first.get(a).unwrap().span_owner(span).unwrap();
    let second_owner = second.get(b).unwrap().span_owner(span).unwrap();
    assert_ne!(first_owner.0, second_owner.0);
    assert!(!Arc::ptr_eq(&first_owner.1, &second_owner.1));
}

#[test]
fn append_preserves_only_the_original_prefix_owner_across_maps() {
    let original = SourceTextSnapshot::new("é :: 1;".into());
    let appended = original.append("\nnext :: 2;");
    let mut before = SourceMap::default();
    let mut after = SourceMap::default();
    let a = before.insert_snapshot("input.jai".into(), original.clone());
    after.insert("unrelated.jai".into(), "".into());
    let b = after.insert_snapshot("input.jai".into(), appended.clone());
    assert_ne!(a, b);
    assert_eq!(original, original.append(""));
    assert_ne!(original, appended);
    let prefix = Span::new(0, original.text().len());
    let owner = before.get(a).unwrap().span_owner(prefix).unwrap();
    let retained = after.get(b).unwrap().span_owner(prefix).unwrap();
    assert_eq!(owner.0, retained.0);
    assert!(Arc::ptr_eq(&owner.1, &retained.1));
    let suffix = after
        .get(b)
        .unwrap()
        .span_owner(Span::new(original.text().len(), appended.text().len()))
        .unwrap();
    assert_ne!(suffix.0, owner.0);
    let crossing = after
        .get(b)
        .unwrap()
        .span_owner(Span::new(original.text().len() - 1, appended.text().len()))
        .unwrap();
    assert_eq!(crossing.0, suffix.0);
    assert!(Arc::ptr_eq(&crossing.1, &suffix.1));
}

#[test]
fn invalid_utf8_reversed_and_out_of_bounds_spans_have_no_owner() {
    let mut sources = SourceMap::default();
    let id = sources.insert("input.jai".into(), "éx".into());
    let record = sources.get(id).unwrap();
    for span in [Span::new(1, 2), Span::new(2, 1), Span::new(0, 4)] {
        assert!(record.span_owner(span).is_none());
        assert!(
            record
                .span_owner_ref_with_work(span, &mut |_| Ok::<_, ()>(()))
                .unwrap()
                .is_none()
        );
    }
    assert!(record.span_owner(Span::new(0, 2)).is_some());
}

#[test]
fn embedded_labels_keep_the_actual_import_occurrence_and_resolution_anchor() {
    let mut sources = SourceMap::default();
    let parent = sources.insert("/modules/owner.jai".into(), "#import,string body".into());
    let importing = jai_source::SourceSpan {
        source: parent,
        span: Span::new(0, 19),
    };
    let embedded = sources.insert_embedded(
        "/display/embedded.jai".into(),
        "answer :: 42;".into(),
        importing,
        "/modules/owner.jai".into(),
    );
    let record = sources.get(embedded).unwrap();
    assert!(matches!(record.kind(), SourceRecordKind::Embedded { .. }));
    assert_eq!(record.importing_site(), Some(importing));
    assert_eq!(record.physical_path(), None);
    assert_eq!(record.path(), Path::new("/display/embedded.jai"));
    assert_eq!(record.resolution_path(), Path::new("/modules/owner.jai"));
    let owner = record
        .span_owner(Span::new(0, record.text().len()))
        .unwrap();
    let parent_owner = sources
        .get(parent)
        .unwrap()
        .span_owner(importing.span)
        .unwrap();
    assert_ne!(owner.0, parent_owner.0);
}
