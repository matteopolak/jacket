use super::*;
use jai_ir::ExternalDataSource;
use jai_source::SourceMap;
use jai_types::{IntegerType, ScalarType};

fn declarations() -> [LocalDeclarationId; 3] {
    let mut sources = SourceMap::default();
    let source = sources.insert("external-ledger.jai".into(), "abc abc abc".into());
    let scope = LexicalScopeId {
        owner: LexicalScopeOwner::Procedure(ProcedureId::new(4)),
        file: None,
        source: Some(source),
        ordinal: 0,
    };
    [0, 4, 8].map(|start| LocalDeclarationId {
        scope,
        start,
        end: start + 3,
        ordinal: start / 4,
    })
}
fn data(
    registry: &mut ExternalGlobals,
    declaration: LocalDeclarationId,
    types: &TypeRegistry,
    symbol: &str,
) -> ExternalData {
    ExternalData::new(
        registry.local_identity(declaration).unwrap(),
        types.scalar(ScalarType::Int(IntegerType::S64)),
        ExternalDataSource::Program,
        symbol.into(),
        location(declaration).unwrap(),
        types,
    )
    .unwrap()
}

#[test]
fn retries_reuse_declaration_identity_and_storage_without_a_default_value() {
    let types = TypeRegistry::new();
    let base = vec![Global::new(0, GlobalInitializer::Bool(true), &types)];
    let [first, second, _] = declarations();
    let mut registry = ExternalGlobals::default();
    registry.reserve_file_prefix(base.len()).unwrap();
    let first_data = data(&mut registry, first, &types, "shared");
    let first_global = registry
        .publish(first, &base, first_data.clone(), &types)
        .unwrap();
    assert_eq!(first_global.id().index(), 1);
    assert_eq!(
        registry.publish(first, &base, first_data, &types).unwrap(),
        first_global
    );
    let second_data = data(&mut registry, second, &types, "shared");
    let second_global = registry
        .publish(second, &base, second_data, &types)
        .unwrap();
    assert_eq!(second_global.id().index(), 2);
    let globals = registry.snapshot(&base).unwrap();
    assert_eq!(globals.len(), 3);
    assert!(matches!(
        globals[1].initializer(),
        GlobalInitializer::External(_)
    ));
    let (GlobalInitializer::External(a), GlobalInitializer::External(b)) =
        (globals[1].initializer(), globals[2].initializer())
    else {
        panic!("external storage")
    };
    assert_ne!(a.id(), b.id());
    assert_eq!(a.symbol(), b.symbol());
}

#[test]
fn scope_and_concrete_procedure_owners_keep_separate_local_source_indices() {
    let [first, mut nested, mut specialized] = declarations();
    nested.scope.ordinal = 1;
    specialized.scope.owner = LexicalScopeOwner::Procedure(ProcedureId::new(5));
    let mut registry = ExternalGlobals::default();
    let a = registry.local_identity(first).unwrap();
    let b = registry.local_identity(nested).unwrap();
    let c = registry.local_identity(specialized).unwrap();
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_eq!(registry.local_identity(first).unwrap(), a);
    assert!(matches!(a, ExternalDataId::Local { index, .. } if index.index() == 0));
    assert!(matches!(b, ExternalDataId::Local { index, .. } if index.index() == 1));
    assert!(matches!(c, ExternalDataId::Local { index, .. } if index.index() == 0));
}

#[test]
fn changed_metadata_or_file_prefix_cannot_replace_a_published_external() {
    let types = TypeRegistry::new();
    let base = vec![Global::new(0, GlobalInitializer::Bool(true), &types)];
    let [first, second, _] = declarations();
    let mut registry = ExternalGlobals::default();
    registry.reserve_file_prefix(base.len()).unwrap();
    let initial = data(&mut registry, first, &types, "shared");
    registry.publish(first, &base, initial, &types).unwrap();
    let changed = data(&mut registry, first, &types, "replacement");
    assert!(registry.publish(first, &base, changed, &types).is_err());
    let wrong = data(&mut registry, second, &types, "shared");
    assert!(registry.publish(first, &base, wrong, &types).is_err());
    assert!(registry.snapshot(&[]).is_err());
    let alternate_base = vec![Global::new(0, GlobalInitializer::Bool(false), &types)];
    assert!(registry.snapshot(&alternate_base).is_err());
    assert_eq!(registry.snapshot(&base).unwrap().len(), 2);
}

#[test]
fn unowned_or_record_scoped_declarations_do_not_gain_external_identity() {
    let [mut missing, mut record, _] = declarations();
    missing.scope.source = None;
    let mut types = TypeRegistry::new();
    record.scope.owner = LexicalScopeOwner::Record(types.reserve_record(RecordKind::Struct));
    let mut registry = ExternalGlobals::default();
    assert!(registry.local_identity(missing).is_err());
    assert!(registry.local_identity(record).is_err());
}

#[test]
fn header_prerequisites_cannot_publish_storage_into_an_incomplete_file_prefix() {
    let types = TypeRegistry::new();
    let [first, _, _] = declarations();
    let mut registry = ExternalGlobals::default();
    registry.reserve_file_prefix(1).unwrap();
    let metadata = data(&mut registry, first, &types, "shared");
    let identity = metadata.id();
    assert!(
        registry
            .publish(first, &[], metadata.clone(), &types)
            .is_err()
    );
    assert!(registry.file_prefix_pending(&[]));
    assert!(registry.snapshot(&[]).unwrap().is_empty());
    let base = vec![Global::new(0, GlobalInitializer::Bool(false), &types)];
    let global = registry.publish(first, &base, metadata, &types).unwrap();
    assert_eq!(global.id().index(), 1);
    assert_eq!(registry.local_identity(first).unwrap(), identity);
    assert!(registry.reserve_file_prefix(2).is_err());
}
