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


#[test]
fn changed_policy_publishes_an_independent_descriptor_and_retains_the_old_graph() {
    use jai_types::{
        DescriptorKind, LayoutPolicy, ReflectionGraph, ReflectionMetadata, ReflectionReadiness,
    };
    use std::sync::Arc;
    let mut types = TypeRegistry::new();
    let schema = Arc::new(schema::TypeInfoSchema::new(&mut types).unwrap());
    let record = types.reserve_record(RecordKind::Struct);
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    types.define_record(record, [int]).unwrap();
    let metadata = ReflectionMetadata::default();
    let mut meta = MetaContext::default();
    meta.schema = Some(Arc::clone(&schema));
    let ReflectionReadiness::Ready(old_graph) =
        ReflectionGraph::build(&types, record, Some(LayoutPolicy::lp64()), &metadata).unwrap()
    else {
        panic!("actual record descriptor is ready")
    };
    let (old_data, additions) = storage::materialize(
        &mut types,
        &old_graph,
        &schema,
        &meta.storage,
        &mut meta.storage_builder,
    )
    .unwrap();
    for (ty, pointer, address) in additions {
        meta.storage
            .insert(ty, (pointer, Arc::clone(&old_data), address));
    }
    let old_object = meta.storage[&record].2.object();
    let old = jai_ir::RuntimeTypeConstant::new(Arc::clone(&old_data), old_object, &types).unwrap();
    meta.storage_policies
        .insert(record, types.record_reflection_policy(record).unwrap());
    types
        .add_record_reflection_flags(
            record,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
        )
        .unwrap();
    meta.synchronize_reflection_policy(&types, Span::default())
        .unwrap();
    assert!(meta.storage.is_empty());
    assert_eq!(meta.descriptor_policy_epoch, 1);
    let ReflectionReadiness::Ready(current_graph) =
        ReflectionGraph::build(&types, record, Some(LayoutPolicy::lp64()), &metadata).unwrap()
    else {
        panic!("hidden record descriptor is ready")
    };
    let (current_data, additions) = storage::materialize(
        &mut types,
        &current_graph,
        &schema,
        &meta.storage,
        &mut meta.storage_builder,
    )
    .unwrap();
    let current_object = additions
        .iter()
        .find(|(ty, _, _)| *ty == record)
        .unwrap()
        .2
        .object();
    let current =
        jai_ir::RuntimeTypeConstant::new(Arc::clone(&current_data), current_object, &types)
            .unwrap();
    assert_eq!(old.identity().ty(), current.identity().ty());
    assert_ne!(old.identity().object(), current.identity().object());
    assert!(Arc::ptr_eq(
        &old_data.objects()[old_object.index()],
        &current_data.objects()[old_object.index()]
    ));
    let DescriptorKind::Record {
        fields: old_fields,
        ..
    } = &old_graph.get(old_graph.root()).unwrap().kind
    else {
        panic!()
    };
    let DescriptorKind::Record {
        fields: current_fields,
        ..
    } = &current_graph.get(current_graph.root()).unwrap().kind
    else {
        panic!()
    };
    assert_eq!(old_fields.len(), 1);
    assert!(current_fields.is_empty());
    old.validate(&types).unwrap();
    current.validate(&types).unwrap();
    old_data.validate(&types).unwrap();
    current_data.validate(&types).unwrap();
}

#[test]
fn rejected_host_publication_drops_prepared_policy_and_epoch_together() {
    let mut types = TypeRegistry::new();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, []).unwrap();
    let mut meta = MetaContext::default();
    let flags = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
    let mut transaction = RecordReflectionTransaction::default();
    transaction.stage(&types, record, flags).unwrap();
    let prepared = meta
        .prepare_reflection_policy_transaction(&mut types, transaction, location())
        .unwrap();
    // The real host service is independent of these exclusive semantic owners.
    let host_publication: Result<(), ()> = Err(());
    assert!(host_publication.is_err());
    drop(prepared);
    assert_eq!(
        types.record_reflection_policy(record).unwrap(),
        RecordReflectionPolicy::default()
    );
    assert_eq!(meta.descriptor_policy_epoch, 0);
    let mut transaction = RecordReflectionTransaction::default();
    transaction.stage(&types, record, flags).unwrap();
    let prepared = meta
        .prepare_reflection_policy_transaction(&mut types, transaction, location())
        .unwrap();
    let receipt = prepared.apply();
    assert_eq!(receipt.len(), 1);
    assert_eq!(types.record_reflection_policy(record).unwrap(), flags);
    assert_eq!(meta.descriptor_policy_epoch, 1);
}
