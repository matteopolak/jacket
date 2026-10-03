use super::*;
use crate::{StaticCatalogAdmissionError, StaticCatalogBudget};
use jai_types::TypeRegistry;

fn catalog(types: &TypeRegistry, capacity: usize) -> Arc<StaticData> {
    let mut builder = StaticDataBuilder::new();
    let ty = types.string();
    let object = builder.reserve(ty, types).unwrap();
    builder
        .define(
            object,
            StaticValue::constant(ConstantValue {
                ty,
                kind: ConstantKind::StringBytes(Vec::with_capacity(capacity)),
            }),
        )
        .unwrap();
    Arc::new(builder.finish(types, StaticDataLimits::default()).unwrap())
}

#[test]
fn empty_string_capacity_is_admitted_as_owned_payload_before_publication() {
    let types = TypeRegistry::new();
    let data = catalog(&types, 4096);
    assert!(data.retained_bytes() >= 4096);
    assert!(data.objects()[0].retained_bytes() >= 4096);
    assert!(data.validation_work() > 0);
    let mut builder = StaticDataBuilder::new();
    let ty = types.string();
    let object = builder.reserve(ty, &types).unwrap();
    builder
        .define(
            object,
            StaticValue::constant(ConstantValue {
                ty,
                kind: ConstantKind::StringBytes(Vec::with_capacity(4096)),
            }),
        )
        .unwrap();
    assert!(matches!(
        builder.publish(
            &types,
            StaticDataLimits {
                retained_bytes: 4095,
                ..StaticDataLimits::default()
            }
        ),
        Err(StaticDataError::Limit("retained bytes"))
    ));
}

#[test]
fn genuine_shared_objects_deduplicate_but_each_cloned_catalog_table_is_retained() {
    let types = TypeRegistry::new();
    let data = catalog(&types, 4096);
    let mut tables = vec![Arc::clone(&data)];
    for _ in 1..64 {
        tables.push(Arc::new(data.as_ref().clone()));
    }
    let mut budget = StaticCatalogBudget::new();
    let mut work = 0;
    for table in &tables {
        budget
            .admit(table, 1_000_000, &mut |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .unwrap();
    }
    assert_eq!(
        budget.bytes(),
        data.retained_bytes() + 63 * data.table_retained_bytes()
    );
    assert_eq!(
        budget
            .admit(&data, 1_000_000, &mut |_| Ok::<_, ()>(()))
            .unwrap(),
        0
    );
    assert!(work > tables.len());
    let bound = budget.accounted_bytes().unwrap();
    assert!(matches!(
        budget.admit(&data, bound - 1, &mut |_| Ok::<_, ()>(())),
        Err(StaticCatalogAdmissionError::Storage(
            StaticDataError::Limit("catalog retained bytes")
        ))
    ));
}

#[test]
fn rejected_union_preserves_facts_without_retaining_strong_catalog_history() {
    let types = TypeRegistry::new();
    let first = catalog(&types, 0);
    let extra = catalog(&types, 8192);
    let mut budget = StaticCatalogBudget::new();
    budget
        .admit(&first, 1_000_000, &mut |_| Ok::<_, ()>(()))
        .unwrap();
    let before = budget.bytes();
    let count = Arc::strong_count(&extra);
    let mut work = 0;
    assert!(
        budget
            .admit(&extra, before + 512, &mut |n| {
                work += n;
                Ok::<_, ()>(())
            })
            .is_err()
    );
    assert_eq!(budget.bytes(), before);
    assert_eq!(Arc::strong_count(&extra), count);
    assert!(work > 0);
    budget
        .admit(&extra, 1_000_000, &mut |_| Ok::<_, ()>(()))
        .unwrap();
    let owner = budget.into_ownership();
    drop(extra);
    let mut next = StaticCatalogBudget::new();
    assert!(matches!(
        next.admit_ownership(&owner, 1_000_000, &mut |_| Ok::<_, ()>(())),
        Err(StaticCatalogAdmissionError::Storage(
            StaticDataError::Limit("catalog owner retired")
        ))
    ));
    assert_eq!(next.bytes(), 0);
}
