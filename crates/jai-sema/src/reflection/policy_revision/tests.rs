use super::*;
use jai_source::{SourceMap, SourceSpan};
use jai_types::{RecordKind, RecordReflectionFlag, RecordReflectionPolicy};

fn location() -> SourceSpan {
    let mut sources = SourceMap::default();
    let source = sources.insert("policy-journal.jai".into(), "#run {}".into());
    SourceSpan {
        source,
        span: Span::new(0, 7),
    }
}

#[test]
fn cancelled_and_noop_transactions_do_not_consume_policy_revisions() {
    let mut types = TypeRegistry::new();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, []).unwrap();
    let mut meta = MetaContext::default();
    let mut canceled = RecordReflectionTransaction::default();
    canceled
        .stage(
            &types,
            record,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
        )
        .unwrap();
    meta.validate_reflection_policy_transaction(&types, &canceled, location())
        .unwrap();
    drop(canceled);
    assert_eq!(meta.descriptor_policy_epoch, 0);
    assert_eq!(
        types.record_reflection_policy(record).unwrap(),
        RecordReflectionPolicy::default()
    );
    let commit = meta
        .commit_reflection_policy_transaction(
            &mut types,
            RecordReflectionTransaction::default(),
            location(),
        )
        .unwrap();
    assert!(commit.is_empty());
    assert_eq!(meta.descriptor_policy_epoch, 0);
}

#[test]
fn committed_batches_advance_once_and_rejected_batches_do_not_publish() {
    let mut types = TypeRegistry::new();
    let first = types.reserve_record(RecordKind::Struct);
    let second = types.reserve_record(RecordKind::Struct);
    types.define_record(first, []).unwrap();
    types.define_record(second, []).unwrap();
    let mut meta = MetaContext::default();
    let flags = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
    let mut transaction = RecordReflectionTransaction::default();
    transaction.stage(&types, first, flags).unwrap();
    transaction.stage(&types, second, flags).unwrap();
    let commit = meta
        .commit_reflection_policy_transaction(&mut types, transaction, location())
        .unwrap();
    assert_eq!(commit.len(), 2);
    assert_eq!(meta.descriptor_policy_epoch, 1);
    let mut transaction = RecordReflectionTransaction::default();
    transaction
        .stage(
            &types,
            first,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::ProceduresAreVoidPointers]),
        )
        .unwrap();
    types
        .add_record_reflection_flags(
            first,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
        )
        .unwrap();
    assert!(
        meta.commit_reflection_policy_transaction(&mut types, transaction, location())
            .is_err()
    );
    assert_eq!(meta.descriptor_policy_epoch, 1);
    assert!(
        !types
            .record_reflection_policy(first)
            .unwrap()
            .contains(RecordReflectionFlag::ProceduresAreVoidPointers)
    );
}
