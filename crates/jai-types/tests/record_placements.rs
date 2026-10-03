use jai_types::{
    IntegerType, LayoutEngine, LayoutPolicy, PlacementIssue, RecordKind, RecordLayout,
    ScalarLayout, ScalarType, TypeError, TypeRegistry,
};

#[test]
fn placement_anchors_retain_the_reserved_owner_after_freezing() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let original = types.fixed_array(byte, 79).unwrap();
    let overlay = types.fixed_array(word, 2).unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            record,
            [original, overlay, word],
            RecordLayout::default(),
            [None, Some(0), None],
        )
        .unwrap();
    let anchor = types
        .record_definition(record)
        .unwrap()
        .layout
        .field_placements[1]
        .unwrap();
    assert_eq!(anchor, types.field(record, 0).unwrap().id);
    assert_eq!(types.validate_field(record, anchor).unwrap(), original);
    let types = types.freeze().unwrap();
    let layout = LayoutEngine::new(&types, LayoutPolicy::lp64())
        .layout(record)
        .unwrap()
        .clone();
    assert_eq!(layout.field_offsets.as_ref(), &[0, 0, 16]);
    assert_eq!((layout.size, layout.alignment), (80, 8));
    assert_eq!(
        types
            .record_definition(record)
            .unwrap()
            .layout
            .field_placements[1],
        Some(anchor)
    );
}

#[test]
fn placements_use_target_alignment_and_nested_array_stride() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            record,
            [byte, byte, word, byte],
            RecordLayout::default(),
            [None, None, Some(1), None],
        )
        .unwrap();
    let array = types.fixed_array(record, 3).unwrap();
    let narrow = LayoutPolicy::new(
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
    .unwrap();
    for (policy, offsets, size) in [
        (LayoutPolicy::lp64(), [0, 1, 8, 16], 24),
        (narrow, [0, 1, 4, 12], 16),
    ] {
        let mut engine = LayoutEngine::new(&types, policy);
        let layout = engine.layout(record).unwrap();
        assert_eq!(layout.field_offsets.as_ref(), &offsets);
        assert_eq!(layout.size, size);
        let layout = engine.layout(array).unwrap();
        assert_eq!(layout.array_stride, Some(size));
        assert_eq!(layout.size, size * 3);
    }
}

#[test]
fn rejected_placement_definitions_can_be_retried() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    for (anchors, issue) in [
        (vec![None], PlacementIssue::FieldCount),
        (
            vec![None, Some(1)],
            PlacementIssue::AnchorNotEarlier {
                field: 1,
                anchor: 1,
            },
        ),
        (
            vec![None, Some(99)],
            PlacementIssue::AnchorNotEarlier {
                field: 1,
                anchor: 99,
            },
        ),
        (
            vec![Some(0), None],
            PlacementIssue::AnchorNotEarlier {
                field: 0,
                anchor: 0,
            },
        ),
    ] {
        let record = types.reserve_record(RecordKind::Struct);
        assert_eq!(
            types.define_record_with_placements(
                record,
                [word, word],
                RecordLayout::default(),
                anchors
            ),
            Err(TypeError::InvalidPlacement {
                record,
                issue
            }),
        );
        assert!(
            matches!(types.record_definition(record), Err(TypeError::Incomplete(id)) if id == record)
        );
        types
            .define_record_with_placements(
                record,
                [word, word],
                RecordLayout::default(),
                [None, Some(0)],
            )
            .unwrap();
    }
    let union = types.reserve_record(RecordKind::Union);
    assert_eq!(
        types.define_record_with_placements(
            union,
            [word, word],
            RecordLayout::default(),
            [None, Some(0)]
        ),
        Err(TypeError::InvalidPlacement {
            record: union,
            issue: PlacementIssue::Union
        }),
    );
    types.define_record(union, [word, word]).unwrap();
}

#[test]
fn supplied_field_id_metadata_cannot_cross_record_or_arena_boundaries() {
    let mut types = TypeRegistry::new();
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let owner = types.reserve_record(RecordKind::Struct);
    types.define_record(owner, [word]).unwrap();
    let local_anchor = types.field(owner, 0).unwrap().id;
    let mut other = TypeRegistry::new();
    let other_word = other.scalar(ScalarType::Int(IntegerType::U64));
    let other_owner = other.reserve_record(RecordKind::Struct);
    other.define_record(other_owner, [other_word]).unwrap();
    let foreign_anchor = other.field(other_owner, 0).unwrap().id;
    for anchor in [local_anchor, foreign_anchor] {
        let record = types.reserve_record(RecordKind::Struct);
        let layout = RecordLayout {
            field_placements: Box::new([None, Some(anchor)]),
            ..RecordLayout::default()
        };
        assert_eq!(
            types.define_record_with_layout(record, [word, word], layout),
            Err(TypeError::FieldOwner {
                record,
                field: anchor
            })
        );
        assert!(
            matches!(types.record_definition(record), Err(TypeError::Incomplete(id)) if id == record)
        );
        types.define_record(record, [word, word]).unwrap();
    }
}

#[test]
fn placement_alignment_overrides_are_applied_after_the_rewind() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let record = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            record,
            [byte, byte, word],
            RecordLayout {
                minimum_alignment: Some(16),
                field_alignments: Box::new([None, None, Some(4)]),
                ..RecordLayout::default()
            },
            [None, None, Some(1)],
        )
        .unwrap();
    let layout = LayoutEngine::new(&types, LayoutPolicy::lp64())
        .layout(record)
        .unwrap()
        .clone();
    assert_eq!(layout.field_offsets.as_ref(), &[0, 1, 4]);
    assert_eq!((layout.size, layout.alignment), (16, 16));
}

#[test]
fn rewound_growth_and_final_extent_rounding_check_overflow() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let word = types.scalar(ScalarType::Int(IntegerType::U64));
    let prefix = types.fixed_array(byte, u64::MAX - 1).unwrap();
    let growth = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            growth,
            [prefix, byte, word],
            RecordLayout {
                packed: true,
                ..RecordLayout::default()
            },
            [None, None, Some(1)],
        )
        .unwrap();
    let maximum = types.fixed_array(byte, u64::MAX).unwrap();
    let rounding = types.reserve_record(RecordKind::Struct);
    types
        .define_record_with_placements(
            rounding,
            [maximum, word],
            RecordLayout::default(),
            [None, Some(0)],
        )
        .unwrap();
    for record in [growth, rounding] {
        let mut engine = LayoutEngine::new(&types, LayoutPolicy::lp64());
        assert!(
            matches!(engine.layout(record), Err(jai_types::LayoutError::Overflow(id)) if id == record)
        );
        assert!(
            matches!(engine.layout(record), Err(jai_types::LayoutError::Overflow(id)) if id == record)
        );
        assert_eq!(engine.layout(byte).unwrap().size, 1);
    }
}
