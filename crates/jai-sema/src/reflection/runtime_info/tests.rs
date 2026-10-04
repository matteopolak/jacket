use super::*;
use jai_types::{
    Integer, IntegerType, RecordKind, RecordReflectionFlag, RecordReflectionPolicy, ScalarType,
};

fn preparation() -> (
    TypeRegistry,
    MetaContext,
    RuntimeInfoSchema,
    jai_source::SourceMap,
    SourceSpan,
) {
    let mut types = TypeRegistry::new();
    let descriptors = Arc::new(schema::TypeInfoSchema::new(&mut types).unwrap());
    let tag = types.reserve_enum(IntegerType::U16);
    types
        .define_enum(
            tag,
            [0, 1, 2, 3, 5].map(|value| Integer::checked(IntegerType::U16, value).unwrap()),
        )
        .unwrap();
    let bytes = types
        .slice(types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let segment = types.reserve_record(RecordKind::Struct);
    types.define_record(segment, [tag, bytes]).unwrap();
    let segments = types.slice(segment).unwrap();
    let global = types.reserve_record(RecordKind::Struct);
    types
        .define_record(
            global,
            [types.scalar(ScalarType::Int(IntegerType::U64)), segments],
        )
        .unwrap();
    let table = types.slice(descriptors.header_pointer).unwrap();
    let globals = types.pointer(global).unwrap();
    let info = types.reserve_record(RecordKind::Struct);
    types.define_record(info, [table, globals]).unwrap();
    let schema = RuntimeInfoSchema::validate(&types, info, global).unwrap();
    let mut meta = MetaContext::default();
    meta.schema = Some(descriptors);
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert(
        "retained-runtime-info.jai".into(),
        "Record::struct{}".into(),
    );
    let location = SourceSpan {
        source,
        span: Span::new(0, 16),
    };
    (types, meta, schema, sources, location)
}

fn members(snapshot: &RuntimeInfoSnapshot, ty: TypeId) -> (jai_ir::StaticObjectId, u64) {
    let identity = snapshot.rows().iter().find(|row| row.ty() == ty).unwrap();
    let object = snapshot.data().object(identity.object()).unwrap();
    let StaticValueKind::Record(fields) = &object.value().kind else {
        panic!("record descriptor");
    };
    let StaticValueKind::Slice {
        count, ..
    } = fields[3].kind
    else {
        panic!("actual member slice");
    };
    (identity.object(), count)
}

#[test]
fn definition_waiting_does_not_replace_policies_observed_at_the_first_request() {
    let (mut types, mut meta, schema, sources, location) = preparation();
    let ready = types.reserve_record(RecordKind::Struct);
    types
        .define_record(ready, [types.scalar(ScalarType::Bool)])
        .unwrap();
    let pending = types.reserve_record(RecordKind::Struct);
    for record in [ready, pending] {
        meta.register_reflection_source_type(
            &types,
            record,
            sources.get(location.source).unwrap(),
            location.span,
        )
        .unwrap();
    }
    let metadata = jai_types::ReflectionMetadata::default();
    let frontier = meta
        .runtime_info_frontier(&types, schema, Some(LayoutPolicy::lp64()), location)
        .unwrap();
    assert!(matches!(
        meta.complete_runtime_info_frontier(&types, &frontier, &metadata, location)
            .unwrap(),
        ReflectionReadiness::Pending(_)
    ));
    let mut transaction = jai_types::RecordReflectionTransaction::default();
    transaction
        .stage(
            &types,
            ready,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
        )
        .unwrap();
    meta.commit_reflection_policy_transaction(&mut types, transaction, location)
        .unwrap();
    types.define_record(pending, []).unwrap();
    let ReflectionReadiness::Ready(old) = meta
        .complete_runtime_info_frontier(&types, &frontier, &metadata, location)
        .unwrap()
    else {
        panic!("completed retained request");
    };
    let jai_types::DescriptorKind::Record {
        fields, ..
    } = &old.graph.descriptor(ready).unwrap().kind
    else {
        panic!("nominal record");
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(old.policy_epoch, 0);
    let ReflectionReadiness::Ready(new) = meta
        .runtime_info_checkpoint(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            &metadata,
            location,
        )
        .unwrap()
    else {
        panic!("later ready request");
    };
    let jai_types::DescriptorKind::Record {
        fields, ..
    } = &new.graph.descriptor(ready).unwrap().kind
    else {
        panic!("nominal record");
    };
    assert!(fields.is_empty());
    assert_eq!(new.policy_epoch, 1);
}

#[test]
fn a_sealed_old_graph_publishes_after_a_new_policy_without_changing_current_storage() {
    let (mut types, mut meta, schema, sources, location) = preparation();
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record(record, [types.scalar(ScalarType::Bool)])
        .unwrap();
    meta.register_reflection_source_type(
        &types,
        record,
        sources.get(location.source).unwrap(),
        location.span,
    )
    .unwrap();
    let metadata = jai_types::ReflectionMetadata::default();
    let ReflectionReadiness::Ready(old) = meta
        .runtime_info_checkpoint(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            &metadata,
            location,
        )
        .unwrap()
    else {
        panic!("old ready frontier");
    };
    let mut transaction = jai_types::RecordReflectionTransaction::default();
    transaction
        .stage(
            &types,
            record,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
        )
        .unwrap();
    meta.commit_reflection_policy_transaction(&mut types, transaction, location)
        .unwrap();
    let ReflectionReadiness::Ready(new) = meta
        .runtime_info_checkpoint(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            &metadata,
            location,
        )
        .unwrap()
    else {
        panic!("new ready frontier");
    };
    let ReflectionReadiness::Ready(new_snapshot) = meta
        .runtime_info_snapshot(&mut types, &new, location)
        .unwrap()
    else {
        panic!("new descriptor storage");
    };
    let current_address = meta.storage.get(&record).unwrap().2.clone();
    let ReflectionReadiness::Ready(old_snapshot) = meta
        .runtime_info_snapshot(&mut types, &old, location)
        .unwrap()
    else {
        panic!("retained old descriptor storage");
    };
    let (old_object, old_members) = members(&old_snapshot, record);
    let (new_object, new_members) = members(&new_snapshot, record);
    assert_ne!(old_object, new_object);
    assert_eq!(old_members, 1);
    assert_eq!(new_members, 0);
    assert_eq!(meta.storage.get(&record).unwrap().2, current_address);
    assert_eq!(old.policy_epoch, 0);
    assert_eq!(new.policy_epoch, 1);
    assert_eq!(old.represented_types(), new.represented_types());
    old_snapshot
        .revalidate(&types, LayoutPolicy::lp64())
        .unwrap();
    new_snapshot
        .revalidate(&types, LayoutPolicy::lp64())
        .unwrap();
    let ReflectionReadiness::Ready(repeated) = meta
        .runtime_info_snapshot(&mut types, &old, location)
        .unwrap()
    else {
        panic!("retained cached snapshot");
    };
    assert!(Arc::ptr_eq(&old_snapshot, &repeated));
}

#[test]
fn a_run_overlay_frontier_retains_its_revision_and_never_reuses_committed_storage() {
    let (mut types, mut meta, schema, sources, location) = preparation();
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record(record, [types.scalar(ScalarType::Bool)])
        .unwrap();
    meta.register_reflection_source_type(
        &types,
        record,
        sources.get(location.source).unwrap(),
        location.span,
    )
    .unwrap();
    let metadata = jai_types::ReflectionMetadata::default();
    let ReflectionReadiness::Ready(canonical) = meta
        .runtime_info_checkpoint(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            &metadata,
            location,
        )
        .unwrap()
    else {
        panic!("canonical ready checkpoint")
    };
    let ReflectionReadiness::Ready(canonical_snapshot) = meta
        .runtime_info_snapshot(&mut types, &canonical, location)
        .unwrap()
    else {
        panic!("canonical descriptor storage")
    };
    let canonical_object = members(&canonical_snapshot, record).0;
    assert_eq!(members(&canonical_snapshot, record).1, 1);
    let source =
        jai_ir::SourceProcedureIdentity::new(sources.get(location.source).unwrap(), location)
            .unwrap();
    let mut journal =
        crate::compile_time::reflection_journal::ReflectionPolicyJournal::new(source.clone());
    journal
        .stage_at_in_run(
            &types,
            record,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
            source.clone(),
            256,
            &mut |_| Ok(()),
        )
        .unwrap();
    let overlay = journal.snapshot_policies(256, &mut |_| Ok(())).unwrap();
    let revision = overlay.revision().clone();
    let pending = types.reserve_record(RecordKind::Struct);
    meta.register_reflection_source_type(
        &types,
        pending,
        sources.get(location.source).unwrap(),
        location.span,
    )
    .unwrap();
    let frontier = meta
        .runtime_info_frontier_with_overlay(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            overlay,
            location,
        )
        .unwrap();
    assert!(matches!(
        meta.complete_runtime_info_frontier(&types, &frontier, &metadata, location)
            .unwrap(),
        ReflectionReadiness::Pending(_)
    ));
    journal
        .stage_at_in_run(
            &types,
            record,
            RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
            source,
            256,
            &mut |_| Ok(()),
        )
        .unwrap();
    types.define_record(pending, []).unwrap();
    let ReflectionReadiness::Ready(checkpoint) = meta
        .complete_runtime_info_frontier(&types, &frontier, &metadata, location)
        .unwrap()
    else {
        panic!("retained overlay completion")
    };
    assert!(checkpoint.policy_revision.as_ref().unwrap() == &revision);
    assert_eq!(
        checkpoint
            .record_policies
            .iter()
            .find(|(ty, _)| *ty == record)
            .unwrap()
            .1
            .bits(),
        1
    );
    let ReflectionReadiness::Ready(snapshot) = meta
        .runtime_info_snapshot(&mut types, &checkpoint, location)
        .unwrap()
    else {
        panic!("overlay descriptor storage")
    };
    assert_eq!(members(&snapshot, record).1, 0);
    assert_ne!(members(&snapshot, record).0, canonical_object);
    assert_eq!(
        meta.storage.get(&record).unwrap().2.object(),
        canonical_object
    );
    drop(journal);
    assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 0);
    snapshot
        .validate_owner(&types, LayoutPolicy::lp64())
        .unwrap();
}
