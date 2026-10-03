use jai_ir::*;
use jai_types::*;

fn policy32() -> LayoutPolicy {
    LayoutPolicy::new(
        ScalarLayout::new(4, 4),
        [
            ScalarLayout::new(1, 1),
            ScalarLayout::new(2, 2),
            ScalarLayout::new(4, 4),
            ScalarLayout::new(8, 4),
        ],
        [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
        ScalarLayout::new(1, 1),
    )
    .unwrap()
}
fn value(ty: TypeId, n: i128) -> StaticValue {
    StaticValue::constant(ConstantValue {
        ty,
        kind: ConstantKind::Int(Integer::checked(IntegerType::U32, n).unwrap()),
    })
}
#[test]
fn selected_byte_views_retain_real_heterogeneous_backing_and_relocation() {
    for policy in [policy32(), LayoutPolicy::lp64()] {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let word = types.scalar(ScalarType::Int(IntegerType::U32));
        let pointer = types.pointer(word).unwrap();
        let backing = types.reserve_record(RecordKind::Struct);
        types.define_record(backing, [word, pointer]).unwrap();
        let bytes = types.slice(byte).unwrap();
        let mut builder = StaticDataBuilder::new();
        let referent = builder.reserve(word, &types).unwrap();
        let root = builder.reserve(backing, &types).unwrap();
        let slice = builder.reserve(bytes, &types).unwrap();
        let size = LayoutEngine::new(&types, policy)
            .layout(backing)
            .unwrap()
            .size;
        let view = builder.byte_view(root, policy, 0, size, &types).unwrap();
        builder.define(referent, value(word, 20)).unwrap();
        builder
            .define(
                root,
                StaticValue {
                    ty: backing,
                    kind: StaticValueKind::Record(vec![
                        value(word, 22),
                        StaticValue {
                            ty: pointer,
                            kind: StaticValueKind::Address(StaticAddress::new(referent)),
                        },
                    ]),
                },
            )
            .unwrap();
        builder
            .define(
                slice,
                StaticValue {
                    ty: bytes,
                    kind: StaticValueKind::Slice {
                        data: Some(view.address()),
                        count: size,
                    },
                },
            )
            .unwrap();
        let data = builder.finish(&types, StaticDataLimits::default()).unwrap();
        assert_eq!(data.address_type(&view.address(), &types).unwrap(), byte);
        assert_eq!(view.backing_type(), backing);
        assert_eq!(view.length(), size);
        assert!(matches!(
            data.object(root).unwrap().value().kind,
            StaticValueKind::Record(_)
        ));
        data.validate(&types).unwrap();
    }
}
#[test]
fn byte_views_reject_foreign_objects_overflow_and_wrong_selected_target() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let mut builder = StaticDataBuilder::new();
    let root = builder.reserve(word, &types).unwrap();
    let mut other = StaticDataBuilder::new();
    let foreign = other.reserve(word, &types).unwrap();
    assert!(matches!(
        builder.byte_view(foreign, LayoutPolicy::lp64(), 0, 4, &types),
        Err(StaticDataError::ForeignObject(_))
    ));
    assert!(
        builder
            .byte_view(root, LayoutPolicy::lp64(), 1, 4, &types)
            .is_err()
    );
    assert!(
        builder
            .byte_view(root, LayoutPolicy::lp64(), u64::MAX, 2, &types)
            .is_err()
    );
    let view = builder
        .byte_view(root, LayoutPolicy::lp64(), 0, 4, &types)
        .unwrap();
    assert!(view.validate_target(policy32()).is_err());
    view.validate_target(LayoutPolicy::lp64()).unwrap();
    builder.define(root, value(word, 42)).unwrap();
    let data = builder.finish(&types, StaticDataLimits::default()).unwrap();
    let malformed = StaticAddress::new(root)
        .project(StaticProjection::Index(0))
        .project(StaticProjection::ByteView(std::sync::Arc::new(view)));
    assert!(data.address_type(&malformed, &types).is_err());
}
#[test]
fn byte_views_require_complete_storage_and_do_not_authorize_a_longer_slice() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let bytes = types
        .slice(types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let mut builder = StaticDataBuilder::new();
    let root = builder.reserve(word, &types).unwrap();
    let slice = builder.reserve(bytes, &types).unwrap();
    let view = builder
        .byte_view(root, LayoutPolicy::lp64(), 1, 2, &types)
        .unwrap();
    builder
        .define(
            slice,
            StaticValue {
                ty: bytes,
                kind: StaticValueKind::Slice {
                    data: Some(view.address()),
                    count: 3,
                },
            },
        )
        .unwrap();
    assert!(matches!(
        builder.publish(&types, StaticDataLimits::default()),
        Err(StaticDataError::IncompleteObject(_))
    ));
    builder.define(root, value(word, 42)).unwrap();
    assert!(matches!(
        builder.publish(&types, StaticDataLimits::default()),
        Err(StaticDataError::OutOfBounds { .. })
    ));
}
#[test]
fn empty_views_allow_only_an_exact_one_past_address_and_u8_elements() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let words = types.slice(word).unwrap();
    let mut builder = StaticDataBuilder::new();
    let root = builder.reserve(word, &types).unwrap();
    let slice = builder.reserve(words, &types).unwrap();
    let view = builder
        .byte_view(root, LayoutPolicy::lp64(), 4, 0, &types)
        .unwrap();
    assert!(
        builder
            .byte_view(root, LayoutPolicy::lp64(), 5, 0, &types)
            .is_err()
    );
    builder.define(root, value(word, 42)).unwrap();
    builder
        .define(
            slice,
            StaticValue {
                ty: words,
                kind: StaticValueKind::Slice {
                    data: Some(view.address()),
                    count: 0,
                },
            },
        )
        .unwrap();
    assert!(matches!(
        builder.publish(&types, StaticDataLimits::default()),
        Err(StaticDataError::TypeMismatch { .. })
    ));
}

#[test]
fn discarded_object_reservations_never_revalidate_as_new_same_typed_storage() {
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let mut builder = StaticDataBuilder::new();
    let old = builder.reserve(word, &types).unwrap();
    let view = builder
        .byte_view(old, LayoutPolicy::lp64(), 0, 4, &types)
        .unwrap();
    builder.discard_unpublished();
    let fresh = builder.reserve(word, &types).unwrap();
    assert_eq!(old.index(), fresh.index());
    assert_ne!(old, fresh);
    assert!(matches!(
        builder.byte_view(old, LayoutPolicy::lp64(), 0, 4, &types),
        Err(StaticDataError::ForeignObject(_))
    ));
    assert!(matches!(
        builder.define(old, value(word, 13)),
        Err(StaticDataError::ForeignObject(_))
    ));
    builder.define(fresh, value(word, 42)).unwrap();
    let data = builder.finish(&types, StaticDataLimits::default()).unwrap();
    assert!(data.object(old).is_err());
    assert!(data.address_type(&view.address(), &types).is_err());
    assert!(data.object(fresh).is_ok());
}

#[test]
fn byte_ranges_cannot_exceed_target_addresses_or_the_signed_descriptor_count() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let huge = types.fixed_array(byte, 1u64 << 32).unwrap();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(huge, &types).unwrap();
    assert!(matches!(
        builder.byte_view(object, policy32(), 0, 1, &types),
        Err(StaticDataError::ByteView(
            StaticByteViewError::UnaddressableBacking { .. }
        ))
    ));
    let huge = types.fixed_array(byte, i64::MAX as u64 + 1).unwrap();
    let object = builder.reserve(huge, &types).unwrap();
    assert!(matches!(
        builder.byte_view(object, LayoutPolicy::lp64(), 0, i64::MAX as u64 + 1, &types),
        Err(StaticDataError::ByteView(StaticByteViewError::SliceCount(
            _
        )))
    ));
}

#[test]
fn byte_view_errors_keep_typed_target_context_without_large_error_results() {
    assert!(std::mem::size_of::<StaticByteViewError>() <= 128);
    assert!(std::mem::size_of::<StaticDataError>() <= 128);
    let types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U32));
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(word, &types).unwrap();
    let view = builder
        .byte_view(object, LayoutPolicy::lp64(), 0, 4, &types)
        .unwrap();
    let Err(StaticByteViewError::Target { expected, actual }) = view.validate_target(policy32())
    else {
        panic!("target mismatch retains its typed context")
    };
    assert_eq!(*expected, LayoutPolicy::lp64());
    assert_eq!(*actual, policy32());
}
