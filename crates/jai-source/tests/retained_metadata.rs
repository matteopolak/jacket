use jai_source::{SourceMap, SourceTextSnapshot, Symbols};
use std::sync::Arc;

#[test]
fn prefix_visitor_lends_each_real_text_allocation_without_clone() {
    let original = SourceTextSnapshot::new("FIRST :: 1;".into());
    let snapshot = original.append("SECOND :: 2;").append("THIRD :: 3;");
    let mut sources = SourceMap::default();
    let id = sources.insert_snapshot("retained.jai".into(), snapshot);
    let record = sources.get(id).unwrap();
    let mut allocations = Vec::new();
    record
        .visit_retained_metadata(&mut |_, _| Ok::<_, ()>(()), &mut |allocation, text| {
            allocations.push(allocation);
            assert!(Arc::strong_count(text) >= 1);
            Ok(())
        })
        .unwrap();
    assert_eq!(allocations.len(), 3);
    assert!(allocations.windows(2).all(|pair| pair[0] != pair[1]));
}

#[test]
fn denied_record_walk_does_not_project_a_source_owner() {
    let mut sources = SourceMap::default();
    let id = sources.insert("retained.jai".into(), "X :: 1;".into());
    let mut visits = 0;
    let result =
        sources
            .get(id)
            .unwrap()
            .visit_retained_metadata(&mut |_, _| Err("stop"), &mut |_, _| {
                visits += 1;
                Ok(())
            });
    assert_eq!(result, Err("stop"));
    assert_eq!(visits, 0);
}

#[test]
fn symbols_count_separate_vector_and_hash_key_strings() {
    let mut symbols = Symbols::default();
    let spelling = "long_retained_source_spelling".repeat(20);
    symbols.intern(&spelling);
    let mut bytes = 0;
    symbols
        .visit_retained_metadata(&mut |_, n| {
            bytes += n;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(bytes >= 2 * spelling.len());
}
