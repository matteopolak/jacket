use jai_ir::SourceProcedureIdentity;
use jai_source::{SourceMap, SourceSpan, SourceTextSnapshot, Span};
use std::path::Path;

fn identity(
    map: &mut SourceMap,
    snapshot: SourceTextSnapshot,
    padding: bool,
) -> SourceProcedureIdentity {
    if padding {
        map.insert("unrelated.jai".into(), "padding".into());
    }
    let id = map.insert_snapshot("recipe.jai".into(), snapshot);
    SourceProcedureIdentity::new(
        map.get(id).unwrap(),
        SourceSpan {
            source: id,
            span: Span::new(0, 7),
        },
    )
    .unwrap()
}

#[test]
fn equal_reconstructed_text_and_dense_ids_do_not_match() {
    let first = identity(
        &mut SourceMap::default(),
        SourceTextSnapshot::new("#run {}".into()),
        false,
    );
    let second = identity(
        &mut SourceMap::default(),
        SourceTextSnapshot::new("#run {}".into()),
        false,
    );
    assert_eq!(first.location(), second.location());
    assert_ne!(first.key(), second.key());
    assert!(!first.matches_identity(&second));
    assert!(!first.same_source_identity(&second));
}

#[test]
fn retained_prefix_matches_new_dense_source_id_but_generated_suffix_does_not() {
    let snapshot = SourceTextSnapshot::new("#run {}".into());
    let first = identity(&mut SourceMap::default(), snapshot.clone(), false);
    let mut rebuilt = SourceMap::default();
    let second = identity(&mut rebuilt, snapshot.append("\nmore :: 1;"), true);
    assert_ne!(first.location().source, second.location().source);
    assert_eq!(first.key(), second.key());
    assert!(first.matches_identity(&second));
    assert!(!first.same_source_identity(&second));
    let record = rebuilt.get(second.location().source).unwrap();
    assert!(first.matches_source(record, second.location()));
    assert_eq!(first.body_text(), "#run {}");
    assert_eq!(first.path(), Path::new("recipe.jai"));
    let suffix = SourceProcedureIdentity::new(
        record,
        SourceSpan {
            source: record.id(),
            span: Span::new(8, record.text().len()),
        },
    )
    .unwrap();
    assert_ne!(suffix.key(), first.key());
}

#[test]
fn changed_path_or_invalid_utf8_span_never_rebases() {
    let snapshot = SourceTextSnapshot::new("#run {}é".into());
    let first = identity(&mut SourceMap::default(), snapshot.clone(), false);
    let mut other = SourceMap::default();
    let id = other.insert_snapshot("elsewhere.jai".into(), snapshot);
    let source = other.get(id).unwrap();
    assert!(!first.matches_source(
        source,
        SourceSpan {
            source: id,
            span: Span::new(0, 7)
        }
    ));
    assert!(
        SourceProcedureIdentity::new(
            source,
            SourceSpan {
                source: id,
                span: Span::new(7, 8)
            }
        )
        .is_err()
    );
}
