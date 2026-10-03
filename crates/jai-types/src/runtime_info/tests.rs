use super::*;
use crate::{Integer, RecordLayout, ScalarLayout, ScalarType, TypeRegistry};

struct Catalog {
    types: TypeRegistry,
    info: TypeId,
    global: TypeId,
    segment: TypeId,
    tag: TypeId,
}
impl Catalog {
    fn new() -> Self {
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
        let table = types
            .slice(
                RuntimeTypeSchema::from_view(&types)
                    .unwrap()
                    .descriptor_type(),
            )
            .unwrap();
        let global_pointer = types.pointer(global).unwrap();
        let info = types.reserve_record(RecordKind::Struct);
        types.define_record(info, [table, global_pointer]).unwrap();
        Self {
            types,
            info,
            global,
            segment,
            tag,
        }
    }
    fn schema(&self) -> RuntimeInfoSchema {
        RuntimeInfoSchema::validate(&self.types, self.info, self.global).unwrap()
    }
}
fn ilp32() -> LayoutPolicy {
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

#[test]
fn source_schema_retains_field_owners_and_selected_target_layout_after_freeze() {
    let catalog = Catalog::new();
    let schema = catalog.schema();
    assert_eq!(schema.global_data_segment_type(), catalog.segment);
    assert_eq!(
        schema.field(RuntimeInfoField::TypeTable).id,
        catalog.types.field(catalog.info, 0).unwrap().id
    );
    assert_eq!(
        schema.field(RuntimeInfoField::GlobalDataInfo).id,
        catalog.types.field(catalog.info, 1).unwrap().id
    );
    let types = catalog.types.freeze().unwrap();
    schema.revalidate(&types).unwrap();
    for (policy, size, offsets, global_size, segment_size) in [
        (LayoutPolicy::lp64(), 24, [0, 16], 24, 24),
        (ilp32(), 16, [0, 12], 20, 16),
    ] {
        let layout = schema.layout(&types, policy).unwrap();
        assert_eq!(layout.size, size);
        assert_eq!(layout.field_offsets.as_ref(), offsets);
        let mut layouts = LayoutEngine::new(&types, policy);
        assert_eq!(layouts.layout(catalog.global).unwrap().size, global_size);
        assert_eq!(layouts.layout(catalog.segment).unwrap().size, segment_size);
    }
    assert!(matches!(
        schema.revalidate(&TypeRegistry::new()),
        Err(RuntimeInfoError::Type(TypeError::ForeignType(_)))
    ));
}

#[test]
fn imitated_header_and_unrelated_global_identity_do_not_satisfy_the_relation() {
    let mut catalog = Catalog::new();
    let header = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record(
            header,
            catalog
                .types
                .record_definition(catalog.schema().runtime_type_schema().header_type())
                .unwrap()
                .fields
                .to_vec(),
        )
        .unwrap();
    let pointer = catalog.types.pointer(header).unwrap();
    let imitation_table = catalog.types.slice(pointer).unwrap();
    let global_pointer = catalog.types.pointer(catalog.global).unwrap();
    let imitation_info = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record(imitation_info, [imitation_table, global_pointer])
        .unwrap();
    assert!(matches!(
        RuntimeInfoSchema::validate(&catalog.types, imitation_info, catalog.global),
        Err(RuntimeInfoError::InvalidTypeTable(_))
    ));
    let other_global = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record(
            other_global,
            catalog
                .types
                .record_definition(catalog.global)
                .unwrap()
                .fields
                .to_vec(),
        )
        .unwrap();
    assert!(matches!(
        RuntimeInfoSchema::validate(&catalog.types, catalog.info, other_global),
        Err(RuntimeInfoError::InvalidGlobalData(_))
    ));
}

#[test]
fn segment_tag_gap_and_byte_view_are_checked_in_the_actual_source_fields() {
    let mut catalog = Catalog::new();
    let wrong_tags = catalog.types.reserve_enum(IntegerType::U16);
    catalog
        .types
        .define_enum(
            wrong_tags,
            [0, 1, 2, 3, 4].map(|value| Integer::checked(IntegerType::U16, value).unwrap()),
        )
        .unwrap();
    for (tag, element) in [
        (wrong_tags, IntegerType::U8),
        (catalog.tag, IntegerType::S8),
    ] {
        let bytes = catalog
            .types
            .slice(catalog.types.scalar(ScalarType::Int(element)))
            .unwrap();
        let segment = catalog.types.reserve_record(RecordKind::Struct);
        catalog.types.define_record(segment, [tag, bytes]).unwrap();
        let segments = catalog.types.slice(segment).unwrap();
        let global = catalog.types.reserve_record(RecordKind::Struct);
        catalog
            .types
            .define_record(
                global,
                [
                    catalog.types.scalar(ScalarType::Int(IntegerType::U64)),
                    segments,
                ],
            )
            .unwrap();
        let pointer = catalog.types.pointer(global).unwrap();
        let table = catalog.schema().field(RuntimeInfoField::TypeTable).ty;
        let info = catalog.types.reserve_record(RecordKind::Struct);
        catalog.types.define_record(info, [table, pointer]).unwrap();
        assert!(matches!(
            RuntimeInfoSchema::validate(&catalog.types, info, global),
            Err(RuntimeInfoError::InvalidGlobalData(_))
        ));
    }
}

#[test]
fn pending_catalog_and_invalid_source_layout_remain_structured_errors() {
    let mut catalog = Catalog::new();
    let pending = catalog.types.reserve_record(RecordKind::Struct);
    let pointer = catalog.types.pointer(pending).unwrap();
    let table = catalog.schema().field(RuntimeInfoField::TypeTable).ty;
    let info = catalog.types.reserve_record(RecordKind::Struct);
    catalog.types.define_record(info, [table, pointer]).unwrap();
    assert!(
        matches!(RuntimeInfoSchema::validate(&catalog.types, info, pending), Err(RuntimeInfoError::Type(TypeError::Incomplete(id))) if id == pending)
    );
    let malformed = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record_with_layout(
            malformed,
            catalog
                .types
                .record_definition(catalog.info)
                .unwrap()
                .fields
                .to_vec(),
            RecordLayout {
                minimum_alignment: Some(3),
                ..RecordLayout::default()
            },
        )
        .unwrap();
    let schema = RuntimeInfoSchema::validate(&catalog.types, malformed, catalog.global).unwrap();
    assert!(
        matches!(schema.layout(&catalog.types, LayoutPolicy::lp64()), Err(RuntimeInfoError::Layout(LayoutError::InvalidRecordAlignment(id))) if id == malformed)
    );
}
