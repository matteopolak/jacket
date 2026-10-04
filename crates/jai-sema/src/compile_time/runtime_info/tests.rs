use super::*;
use jai_types::{
    Integer, IntegerType, LayoutPolicy, RecordKind, ReflectionReadiness, RuntimeInfoSchema,
    ScalarType, TypeRegistry,
};

fn preparation() -> (
    TypeRegistry,
    RuntimeInfoSchema,
    crate::reflection::MetaContext,
    jai_source::SourceMap,
    jai_source::SourceSpan,
) {
    let mut types = TypeRegistry::new();
    let header = types.reserve_record(RecordKind::Struct);
    types
        .define_record(
            header,
            [
                types.scalar(ScalarType::Int(IntegerType::U32)),
                types.scalar(ScalarType::Int(IntegerType::S64)),
            ],
        )
        .unwrap();
    types.bind_runtime_type_header(header).unwrap();
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
    let header_pointer = types.pointer(header).unwrap();
    let table = types.slice(header_pointer).unwrap();
    let global_pointer = types.pointer(global).unwrap();
    let info = types.reserve_record(RecordKind::Struct);
    types.define_record(info, [table, global_pointer]).unwrap();
    let schema = RuntimeInfoSchema::validate(&types, info, global).unwrap();
    let meta = crate::reflection::MetaContext::default();
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("runtime-info-demand.jai".into(), "#run {}".into());
    let location = jai_source::SourceSpan {
        source,
        span: jai_source::Span::new(0, 7),
    };
    (types, schema, meta, sources, location)
}

fn checkpoint() -> Arc<RuntimeInfoCheckpoint> {
    let (types, schema, mut meta, _sources, location) = preparation();
    let ReflectionReadiness::Ready(checkpoint) = meta
        .runtime_info_checkpoint(
            &types,
            schema,
            Some(LayoutPolicy::lp64()),
            &jai_types::ReflectionMetadata::default(),
            location,
        )
        .unwrap()
    else {
        panic!("canonical ready schema")
    };
    Arc::new(checkpoint)
}
fn workspace(id: u64) -> WorkspaceId {
    WorkspaceId::from_raw(id).unwrap()
}

#[test]
fn concurrent_drives_sharing_a_checkpoint_have_independent_demand_lifetimes() {
    let checkpoint = checkpoint();
    let mut demands = RuntimeInfoDemands::default();
    let first = demands
        .prepare(workspace(31), Arc::clone(&checkpoint))
        .unwrap();
    let second = demands
        .prepare(workspace(31), Arc::clone(&checkpoint))
        .unwrap();
    assert_ne!(first, second);
    demands.release(first, workspace(31)).unwrap();
    assert!(
        matches!(demands.availability(second, workspace(31)).unwrap(), RuntimeInfoDemandAvailability::Requested(id) if id == second)
    );
    assert!(Arc::ptr_eq(
        &checkpoint,
        &demands.checkpoint(second, workspace(31)).unwrap()
    ));
}

#[test]
fn wrong_workspace_cannot_observe_or_retire_a_drive() {
    let mut demands = RuntimeInfoDemands::default();
    let id = demands.prepare(workspace(32), checkpoint()).unwrap();
    assert!(matches!(
        demands.checkpoint(id, workspace(33)),
        Err(RuntimeInfoDemandError::ForeignWorkspace)
    ));
    assert_eq!(
        demands.release(id, workspace(33)),
        Err(RuntimeInfoDemandError::ForeignWorkspace)
    );
    assert!(demands.checkpoint(id, workspace(32)).is_ok());
}

#[test]
fn released_and_foreign_registry_tokens_never_select_another_checkpoint() {
    let mut first = RuntimeInfoDemands::default();
    let mut second = RuntimeInfoDemands::default();
    let id = first.prepare(workspace(34), checkpoint()).unwrap();
    let other = second.prepare(workspace(34), checkpoint()).unwrap();
    assert_ne!(id, other);
    assert!(matches!(
        second.checkpoint(id, workspace(34)),
        Err(RuntimeInfoDemandError::Retired)
    ));
    first.release(id, workspace(34)).unwrap();
    let replacement = first.prepare(workspace(34), checkpoint()).unwrap();
    assert_ne!(id, replacement);
    assert!(matches!(
        first.checkpoint(id, workspace(34)),
        Err(RuntimeInfoDemandError::Retired)
    ));
}

#[test]
fn incomplete_source_frontier_retains_the_first_demand_through_definition_waiting() {
    let (mut types, schema, mut meta, sources, location) = preparation();
    let record = types.reserve_record(RecordKind::Struct);
    meta.register_reflection_source_type(
        &types,
        record,
        sources.get(location.source).unwrap(),
        location.span,
    )
    .unwrap();
    let metadata = jai_types::ReflectionMetadata::default();
    let mut demands = RuntimeInfoDemands::default();
    let id = demands
        .prepare_requested(
            workspace(35),
            schema,
            Some(LayoutPolicy::lp64()),
            RuntimeInfoService {
                meta: &mut meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap();
    let original = demands.frontier(id, workspace(35)).unwrap();
    let waiting = demands
        .service(
            id,
            workspace(35),
            RuntimeInfoService {
                meta: &mut meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap();
    assert!(
        matches!(waiting, ReflectionReadiness::Pending(ref dependencies)
        if dependencies.contains(&jai_types::ReflectionDependency::Definition(record)))
    );
    assert_eq!(demands.active.len(), 1);
    let unrelated = types.reserve_record(RecordKind::Struct);
    types.define_record(unrelated, []).unwrap();
    meta.register_reflection_source_type(
        &types,
        unrelated,
        sources.get(location.source).unwrap(),
        location.span,
    )
    .unwrap();
    types
        .define_record(record, [types.scalar(ScalarType::Bool)])
        .unwrap();
    let error = demands
        .service(
            id,
            workspace(35),
            RuntimeInfoService {
                meta: &mut meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap_err();
    assert!(error.message.contains("descriptor schema is not adopted"));
    let ready = demands.checkpoint(id, workspace(35)).unwrap();
    assert!(ready.represented_types().contains(&record));
    assert!(!ready.represented_types().contains(&unrelated));
    assert!(Arc::ptr_eq(
        &original,
        &demands.frontier(id, workspace(35)).unwrap()
    ));
    assert!(
        matches!(demands.availability_for(id, workspace(35), schema).unwrap(),
        RuntimeInfoDemandAvailability::Requested(request) if request == id)
    );
}

#[test]
fn another_canonical_schema_cannot_select_a_drive_checkpoint() {
    let first = checkpoint();
    let other = checkpoint();
    let mut demands = RuntimeInfoDemands::default();
    let id = demands.prepare(workspace(36), first).unwrap();
    assert!(matches!(
        demands.availability_for(id, workspace(36), other.schema()),
        Err(RuntimeInfoDemandError::ForeignSchema)
    ));
    assert!(demands.checkpoint(id, workspace(36)).is_ok());
}

#[test]
fn failed_materialization_does_not_publish_or_replace_the_retained_checkpoint() {
    let (mut types, schema, mut meta, _sources, location) = preparation();
    let metadata = jai_types::ReflectionMetadata::default();
    let mut demands = RuntimeInfoDemands::default();
    let id = demands
        .prepare_requested(
            workspace(37),
            schema,
            Some(LayoutPolicy::lp64()),
            RuntimeInfoService {
                meta: &mut meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap();
    let original = demands.frontier(id, workspace(37)).unwrap();
    let error = demands
        .service(
            id,
            workspace(37),
            RuntimeInfoService {
                meta: &mut meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap_err();
    assert!(error.message.contains("descriptor schema is not adopted"));
    assert!(Arc::ptr_eq(
        &original,
        &demands.frontier(id, workspace(37)).unwrap()
    ));
    assert!(
        matches!(demands.availability_for(id, workspace(37), schema).unwrap(), RuntimeInfoDemandAvailability::Requested(request) if request == id)
    );
    demands.release(id, workspace(37)).unwrap();
}

#[test]
fn another_source_catalog_cannot_service_a_same_registry_checkpoint() {
    let (mut types, schema, mut original_meta, _sources, location) = preparation();
    let metadata = jai_types::ReflectionMetadata::default();
    let mut demands = RuntimeInfoDemands::default();
    let id = demands
        .prepare_requested(
            workspace(38),
            schema,
            Some(LayoutPolicy::lp64()),
            RuntimeInfoService {
                meta: &mut original_meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap();
    let mut foreign_meta = crate::reflection::MetaContext::default();
    let error = demands
        .service(
            id,
            workspace(38),
            RuntimeInfoService {
                meta: &mut foreign_meta,
                types: &mut types,
                metadata: &metadata,
                location,
            },
        )
        .unwrap_err();
    assert!(error.message.contains("another source catalog"));
    assert!(
        matches!(demands.availability_for(id, workspace(38), schema).unwrap(), RuntimeInfoDemandAvailability::Requested(request) if request == id)
    );
}
