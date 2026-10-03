use super::*;
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};

fn integer(ty: TypeId, value: i128) -> StaticValue {
    StaticValue::constant(ConstantValue {
        ty,
        kind: ConstantKind::Int(Integer::wrapping(IntegerType::S64, value)),
    })
}

#[test]
fn admission_bounds_the_total_of_unpublished_values_and_rollback_releases_it() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let count = StaticDataLimits::default().value_nodes / 4;
    let array = types.fixed_array(int, count as u64).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = builder.reserve(array, &types).unwrap();
    let second = builder.reserve(array, &types).unwrap();
    let value = || StaticValue {
        ty: array,
        kind: StaticValueKind::Array((0..count).map(|_| integer(int, 1)).collect()),
    };
    builder.define(first, value()).unwrap();
    assert!(matches!(
        builder.define(second, value()),
        Err(StaticDataError::Limit("value node count"))
    ));
    builder.discard_unpublished();
    let scalar = builder.reserve(int, &types).unwrap();
    builder.define(scalar, integer(int, 42)).unwrap();
    assert_eq!(builder.retained_nodes, 2);
    builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
}

#[test]
fn successive_publications_preserve_objects_and_rollback_only_private_work() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(int).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = builder.reserve(int, &types).unwrap();
    builder.define(first, integer(int, 7)).unwrap();
    let original = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    let failed = builder.reserve(pointer, &types).unwrap();
    assert!(
        matches!(builder.publish(&types, StaticDataLimits::default()), Err(StaticDataError::IncompleteObject(id)) if id == failed)
    );
    builder.discard_unpublished();
    let second = builder.reserve(pointer, &types).unwrap();
    builder
        .define(
            second,
            StaticValue {
                ty: pointer,
                kind: StaticValueKind::Address(StaticAddress::new(first)),
            },
        )
        .unwrap();
    let extended = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    assert_eq!(original.identity(), extended.identity());
    assert_eq!(original.objects().len(), 1);
    assert_eq!(extended.objects().len(), 2);
    assert!(Arc::ptr_eq(&original.objects()[0], &extended.objects()[0]));
    for data in [&original, &extended] {
        assert!(
            matches!(&data.object(first).unwrap().value().kind, StaticValueKind::Constant(ConstantValue { kind: ConstantKind::Int(value), .. }) if value.value() == 7)
        );
    }
    assert!(
        matches!(builder.define(first, integer(int, 99)), Err(StaticDataError::AlreadyDefined(id)) if id == first)
    );
}

#[test]
fn cyclic_static_pointers_validate_without_recursive_expansion() {
    let mut types = TypeRegistry::new();
    let node = types.reserve_record(RecordKind::Struct);
    let next = types.pointer(node).unwrap();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    types.define_record(node, vec![int, next]).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = builder.reserve(node, &types).unwrap();
    let second = builder.reserve(node, &types).unwrap();
    for (object, referent, value) in [(first, second, 3), (second, first, 5)] {
        builder
            .define(
                object,
                StaticValue {
                    ty: node,
                    kind: StaticValueKind::Record(vec![
                        integer(int, value),
                        StaticValue {
                            ty: next,
                            kind: StaticValueKind::Address(StaticAddress::new(referent)),
                        },
                    ]),
                },
            )
            .unwrap();
    }
    let data = builder.finish(&types, StaticDataLimits::default()).unwrap();
    assert_eq!(data.objects().len(), 2);
    assert_eq!(
        data.address_type(&StaticAddress::new(first), &types)
            .unwrap(),
        node
    );
    data.validate(&types).unwrap();
}

#[test]
fn member_and_array_addresses_preserve_nominal_owners() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(int, 3).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    let other = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![array]).unwrap();
    types.define_record(other, vec![array]).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(record, &types).unwrap();
    builder
        .define(
            object,
            StaticValue {
                ty: record,
                kind: StaticValueKind::Record(vec![StaticValue {
                    ty: array,
                    kind: StaticValueKind::Array(vec![
                        integer(int, 1),
                        integer(int, 2),
                        integer(int, 3),
                    ]),
                }]),
            },
        )
        .unwrap();
    let data = builder.finish(&types, StaticDataLimits::default()).unwrap();
    let address = StaticAddress::new(object)
        .project(StaticProjection::Field(types.field(record, 0).unwrap().id))
        .project(StaticProjection::Index(2));
    assert_eq!(data.address_type(&address, &types).unwrap(), int);
    assert!(matches!(
        data.address_type(
            &StaticAddress::new(object)
                .project(StaticProjection::Field(types.field(other, 0).unwrap().id)),
            &types
        ),
        Err(StaticDataError::Type(TypeError::FieldOwner { .. }))
    ));
}

#[test]
fn immutable_views_cannot_exceed_the_underlying_static_array() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let array = types.fixed_array(int, 3).unwrap();
    let slice = types.slice(int).unwrap();
    let build = |count| {
        let mut builder = StaticDataBuilder::new();
        let items = builder.reserve(array, &types).unwrap();
        let view = builder.reserve(slice, &types).unwrap();
        builder
            .define(
                items,
                StaticValue {
                    ty: array,
                    kind: StaticValueKind::Array(vec![
                        integer(int, 1),
                        integer(int, 2),
                        integer(int, 3),
                    ]),
                },
            )
            .unwrap();
        builder
            .define(
                view,
                StaticValue {
                    ty: slice,
                    kind: StaticValueKind::Slice {
                        data: Some(StaticAddress::new(items).project(StaticProjection::Index(1))),
                        count,
                    },
                },
            )
            .unwrap();
        builder.finish(&types, StaticDataLimits::default())
    };
    assert!(build(2).is_ok());
    assert!(matches!(
        build(3),
        Err(StaticDataError::OutOfBounds { index: 3, count: 2 })
    ));
}

#[test]
fn foreign_and_pending_objects_never_publish_a_graph() {
    let types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut first = StaticDataBuilder::new();
    let object = first.reserve(int, &types).unwrap();
    assert!(matches!(
        StaticDataBuilder::new().define(object, integer(int, 1)),
        Err(StaticDataError::ForeignObject(_))
    ));
    assert!(
        matches!(first.finish(&types, StaticDataLimits::default()), Err(StaticDataError::IncompleteObject(id)) if id == object)
    );
}

#[test]
fn wrong_nominal_pointer_referents_are_rejected() {
    let mut types = TypeRegistry::new();
    let first = types.reserve_record(RecordKind::Struct);
    let second = types.reserve_record(RecordKind::Struct);
    types.define_record(first, vec![]).unwrap();
    types.define_record(second, vec![]).unwrap();
    let pointer = types.pointer(first).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(second, &types).unwrap();
    let root = builder.reserve(pointer, &types).unwrap();
    builder
        .define(
            object,
            StaticValue {
                ty: second,
                kind: StaticValueKind::Record(vec![]),
            },
        )
        .unwrap();
    builder
        .define(
            root,
            StaticValue {
                ty: pointer,
                kind: StaticValueKind::Address(StaticAddress::new(object)),
            },
        )
        .unwrap();
    assert!(
        matches!(builder.finish(&types, StaticDataLimits::default()), Err(StaticDataError::TypeMismatch { expected, actual }) if expected == first && actual == second)
    );
}

#[test]
fn bounded_iterative_validation_rejects_excessive_value_depth() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut ty = int;
    let mut value = integer(int, 1);
    for _ in 0..32 {
        ty = types.fixed_array(ty, 1).unwrap();
        value = StaticValue {
            ty,
            kind: StaticValueKind::Array(vec![value]),
        };
    }
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(ty, &types).unwrap();
    builder.define(object, value).unwrap();
    assert!(matches!(
        builder.finish(
            &types,
            StaticDataLimits {
                value_depth: 8,
                ..StaticDataLimits::default()
            }
        ),
        Err(StaticDataError::Limit("value depth"))
    ));
}

#[test]
fn deep_public_trees_are_rejected_and_disposed_on_a_small_stack() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let types = TypeRegistry::new();
            let int = types.scalar(ScalarType::Int(IntegerType::S64));
            let mut builder = StaticDataBuilder::new();
            let object = builder.reserve(int, &types).unwrap();
            let mut value = integer(int, 1);
            for _ in 0..10_000 {
                value = StaticValue {
                    ty: int,
                    kind: StaticValueKind::Array(vec![value]),
                };
            }
            assert!(matches!(
                builder.define(object, value),
                Err(StaticDataError::Limit("value depth"))
            ));
            let mut value = ConstantValue {
                ty: int,
                kind: ConstantKind::Zero,
            };
            for _ in 0..10_000 {
                value = ConstantValue {
                    ty: int,
                    kind: ConstantKind::Distinct(Box::new(value)),
                };
            }
            assert!(matches!(
                builder.define(object, StaticValue::constant(value)),
                Err(StaticDataError::Limit("value depth"))
            ));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn publication_budget_bounds_retained_snapshot_reference_overhead() {
    let types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(int, &types).unwrap();
    builder.define(object, integer(int, 7)).unwrap();
    let limits = StaticDataLimits {
        value_nodes: 8,
        ..StaticDataLimits::default()
    };
    let mut snapshots = Vec::new();
    for _ in 0..8 {
        snapshots.push(builder.publish(&types, limits).unwrap());
    }
    assert!(matches!(
        builder.publish(&types, limits),
        Err(StaticDataError::Limit("publication reference count"))
    ));
    for snapshot in &snapshots {
        assert!(Arc::ptr_eq(
            &snapshot.objects()[0],
            &snapshots[0].objects()[0]
        ));
        assert_eq!(snapshot.object(object).unwrap().ty(), int);
    }
}

#[test]
fn rolled_back_reservations_cannot_define_same_typed_replacements() {
    let types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut builder = StaticDataBuilder::new();
    let retired = builder.reserve(int, &types).unwrap();
    builder.discard_unpublished();
    let replacement = builder.reserve(int, &types).unwrap();
    assert_eq!(retired.index(), replacement.index());
    assert!(
        matches!(builder.define(retired, integer(int, 9)), Err(StaticDataError::ForeignObject(id)) if id == retired)
    );
    assert_ne!(retired, replacement);
    builder.define(replacement, integer(int, 42)).unwrap();
    let data = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    assert!(
        matches!(data.object(retired), Err(StaticDataError::ForeignObject(id)) if id == retired)
    );
    assert!(
        matches!(data.address_type(&StaticAddress::new(retired), &types), Err(StaticDataError::ForeignObject(id)) if id == retired)
    );
    assert!(
        matches!(&data.object(replacement).unwrap().value().kind, StaticValueKind::Constant(ConstantValue {kind: ConstantKind::Int(v), ..}) if v.value() == 42)
    );
}

#[test]
fn stale_relocations_fail_without_invalidating_published_prefixes() {
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let pointer = types.pointer(int).unwrap();
    let mut builder = StaticDataBuilder::new();
    let first = builder.reserve(int, &types).unwrap();
    builder.define(first, integer(int, 7)).unwrap();
    let original = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    let retired = builder.reserve(int, &types).unwrap();
    builder.discard_unpublished();
    let replacement = builder.reserve(int, &types).unwrap();
    builder.define(replacement, integer(int, 8)).unwrap();
    let relocation = builder.reserve(pointer, &types).unwrap();
    builder
        .define(
            relocation,
            StaticValue {
                ty: pointer,
                kind: StaticValueKind::Address(StaticAddress::new(retired)),
            },
        )
        .unwrap();
    assert!(
        matches!(builder.publish(&types, StaticDataLimits::default()), Err(StaticDataError::ForeignObject(id)) if id == retired)
    );
    builder.discard_unpublished();
    let ready = builder.reserve(int, &types).unwrap();
    builder.define(ready, integer(int, 42)).unwrap();
    let data = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    assert!(Arc::ptr_eq(&original.objects()[0], &data.objects()[0]));
    assert_eq!(data.object(first).unwrap().id(), first);
    assert!(
        matches!(data.object(retired), Err(StaticDataError::ForeignObject(id)) if id == retired)
    );
}

#[test]
fn exhausted_reservation_identity_does_not_allocate_or_wrap() {
    let types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let mut builder = StaticDataBuilder::new();
    builder.next_reservation = u64::MAX - 1;
    let last = builder.reserve(int, &types).unwrap();
    assert_eq!(last.reservation_identity(), u64::MAX - 1);
    builder.define(last, integer(int, 42)).unwrap();
    let published = builder
        .publish(&types, StaticDataLimits::default())
        .unwrap();
    let before = builder.objects.len();
    assert!(matches!(
        builder.reserve(int, &types),
        Err(StaticDataError::ReservationExhausted)
    ));
    assert_eq!(builder.objects.len(), before);
    assert_eq!(builder.next_reservation, u64::MAX);
    assert_eq!(published.object(last).unwrap().id(), last);
    builder.discard_unpublished();
    assert!(matches!(
        builder.reserve(int, &types),
        Err(StaticDataError::ReservationExhausted)
    ));
}
