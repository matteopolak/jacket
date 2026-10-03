use super::*;
use crate::{ConstantValue, StaticDataBuilder, StaticDataLimits};
use jai_types::{
    Integer, IntegerType, LayoutPolicy, RecordKind, ReflectionMetadata, ReflectionReadiness,
    ScalarType, TypeRegistry,
};

struct Catalog {
    types: TypeRegistry,
    tag: TypeId,
    header: TypeId,
    integer: TypeId,
    graph: ReflectionGraph,
}
impl Catalog {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let tag = types.reserve_enum(IntegerType::U32);
        types
            .define_enum(
                tag,
                [0, 1, 2, 4, 5, 13].map(|tag| Integer::checked(IntegerType::U32, tag).unwrap()),
            )
            .unwrap();
        let header = types.reserve_record(RecordKind::Struct);
        let integer = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                header,
                [tag, types.scalar(ScalarType::Int(IntegerType::S64))],
            )
            .unwrap();
        types
            .define_record(integer, [header, types.scalar(ScalarType::Bool)])
            .unwrap();
        types.bind_runtime_type_header(header).unwrap();
        let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
            &types,
            types.scalar(ScalarType::Int(IntegerType::S32)),
            Some(LayoutPolicy::lp64()),
            &ReflectionMetadata::default(),
        )
        .unwrap() else {
            panic!("scalar descriptor must be ready")
        };
        Self {
            types,
            tag,
            header,
            integer,
            graph,
        }
    }
    fn value(&self, tag: i128, size: i128, signed: bool) -> StaticValue {
        let scalar = |ty, kind| StaticValue::constant(ConstantValue { ty, kind });
        StaticValue {
            ty: self.integer,
            kind: StaticValueKind::Record(vec![
                StaticValue {
                    ty: self.header,
                    kind: StaticValueKind::Record(vec![
                        scalar(
                            self.tag,
                            ConstantKind::Enum(Integer::checked(IntegerType::U32, tag).unwrap()),
                        ),
                        scalar(
                            self.types.scalar(ScalarType::Int(IntegerType::S64)),
                            ConstantKind::Int(Integer::checked(IntegerType::S64, size).unwrap()),
                        ),
                    ]),
                },
                scalar(
                    self.types.scalar(ScalarType::Bool),
                    ConstantKind::Bool(signed),
                ),
            ]),
        }
    }
    fn header(&self, tag: i128, size: i128) -> StaticValue {
        let StaticValueKind::Record(mut fields) = self.value(tag, size, true).kind else {
            unreachable!()
        };
        fields.remove(0)
    }
    fn graph(&self, ty: TypeId) -> ReflectionGraph {
        let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
            &self.types,
            ty,
            Some(LayoutPolicy::lp64()),
            &ReflectionMetadata::default(),
        )
        .unwrap() else {
            panic!("ready descriptor")
        };
        graph
    }
}

#[test]
fn descriptor_binding_certifies_header_projection_and_survives_later_snapshots() {
    let catalog = Catalog::new();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(catalog.integer, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            object,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    let first = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    let value = RuntimeTypeConstant::new(Arc::clone(&first), object, &catalog.types).unwrap();
    assert_eq!(value.ty(), catalog.types.meta_type());
    assert_eq!(value.identity().policy(), LayoutPolicy::lp64());
    assert_eq!(
        value.identity().ty(),
        catalog.types.scalar(ScalarType::Int(IntegerType::S32))
    );
    assert_eq!(
        value.address().path(),
        &[StaticProjection::Field(
            catalog.types.field(catalog.integer, 0).unwrap().id
        )]
    );
    assert_eq!(
        first.address_type(value.address(), &catalog.types).unwrap(),
        catalog.header
    );
    let boolean = catalog.types.scalar(ScalarType::Bool);
    let extra = builder.reserve(boolean, &catalog.types).unwrap();
    builder
        .define(
            extra,
            StaticValue::constant(ConstantValue {
                ty: boolean,
                kind: ConstantKind::Bool(true),
            }),
        )
        .unwrap();
    let later = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    let later_value = RuntimeTypeConstant::new(Arc::clone(&later), object, &catalog.types).unwrap();
    assert_eq!(value, later_value);
    assert!(Arc::ptr_eq(&first.objects()[0], &later.objects()[0]));
    assert_eq!(first.objects().len(), 1);
    value.validate(&catalog.types).unwrap();
    assert!(matches!(
        value.validate(&TypeRegistry::new()),
        Err(StaticDataError::Type(jai_types::TypeError::ForeignType(_)))
    ));
}

#[test]
fn pointer_identity_requires_the_complete_payload_and_exact_pointee_binding() {
    let mut catalog = Catalog::new();
    let header_pointer = catalog.types.pointer(catalog.header).unwrap();
    let pointer_record = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record(pointer_record, [catalog.header, header_pointer])
        .unwrap();
    let pointer = catalog
        .types
        .pointer(catalog.types.scalar(ScalarType::Int(IntegerType::S32)))
        .unwrap();
    let graph = catalog.graph(pointer);
    let mut builder = StaticDataBuilder::new();
    let truncated = builder.reserve(catalog.header, &catalog.types).unwrap();
    assert!(matches!(
        builder.define_type_descriptor(
            truncated,
            catalog.header(4, 8),
            &graph,
            graph.root(),
            &catalog.types,
        ),
        Err(StaticDataError::InvalidValue(_))
    ));
    builder.discard_unpublished();

    let boolean = catalog.types.scalar(ScalarType::Bool);
    let boolean_graph = catalog.graph(boolean);
    let boolean_object = builder.reserve(catalog.header, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            boolean_object,
            catalog.header(2, 1),
            &boolean_graph,
            boolean_graph.root(),
            &catalog.types,
        )
        .unwrap();
    let forged = builder.reserve(pointer_record, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            forged,
            StaticValue {
                ty: pointer_record,
                kind: StaticValueKind::Record(vec![
                    catalog.header(4, 8),
                    StaticValue {
                        ty: header_pointer,
                        kind: StaticValueKind::Address(StaticAddress::new(boolean_object)),
                    },
                ]),
            },
            &graph,
            graph.root(),
            &catalog.types,
        )
        .unwrap();
    // All nominal storage types are correct, but the serialized pointer names
    // bool while its purported represented identity is *s32.
    assert!(matches!(
        builder.publish(&catalog.types, StaticDataLimits::default()),
        Err(StaticDataError::InvalidValue(_))
    ));
    builder.discard_unpublished();

    let integer = builder.reserve(catalog.integer, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            integer,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    let pointer_object = builder.reserve(pointer_record, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            pointer_object,
            StaticValue {
                ty: pointer_record,
                kind: StaticValueKind::Record(vec![
                    catalog.header(4, 8),
                    StaticValue {
                        ty: header_pointer,
                        kind: StaticValueKind::Address(StaticAddress::new(integer).project(
                            StaticProjection::Field(
                                catalog.types.field(catalog.integer, 0).unwrap().id,
                            ),
                        )),
                    },
                ]),
            },
            &graph,
            graph.root(),
            &catalog.types,
        )
        .unwrap();
    let data = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    assert_eq!(
        RuntimeTypeConstant::new(data, pointer_object, &catalog.types)
            .unwrap()
            .identity()
            .ty(),
        pointer
    );
}

#[test]
fn static_type_cells_use_canonical_same_graph_addresses_and_reject_nested_arcs() {
    let catalog = Catalog::new();
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(catalog.integer, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            object,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    let data = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    let literal = RuntimeTypeConstant::new(data, object, &catalog.types).unwrap();
    let cell = builder
        .reserve(catalog.types.meta_type(), &catalog.types)
        .unwrap();
    assert!(matches!(
        builder.define(
            cell,
            StaticValue::constant(ConstantValue {
                ty: catalog.types.meta_type(),
                kind: ConstantKind::RuntimeType(literal.clone()),
            })
        ),
        Err(StaticDataError::InvalidValue(_))
    ));
    // Failed admission leaves the reservation available for a valid relocation.
    builder
        .define(
            cell,
            StaticValue {
                ty: catalog.types.meta_type(),
                kind: StaticValueKind::Address(literal.address().clone()),
            },
        )
        .unwrap();
    builder
        .publish(&catalog.types, StaticDataLimits::default())
        .unwrap();
    let imitation = builder.reserve(catalog.integer, &catalog.types).unwrap();
    builder
        .define(imitation, catalog.value(0, 4, true))
        .unwrap();
    let forged_cell = builder
        .reserve(catalog.types.meta_type(), &catalog.types)
        .unwrap();
    builder
        .define(
            forged_cell,
            StaticValue {
                ty: catalog.types.meta_type(),
                kind: StaticValueKind::Address(StaticAddress::new(imitation).project(
                    StaticProjection::Field(catalog.types.field(catalog.integer, 0).unwrap().id),
                )),
            },
        )
        .unwrap();
    assert!(matches!(
        builder.publish(&catalog.types, StaticDataLimits::default()),
        Err(StaticDataError::InvalidValue(_))
    ));
}

#[test]
fn aliased_descriptor_views_share_storage_but_consume_bounded_validation_work() {
    use jai_types::{CallingConvention, ContextMode, ProcedureType, Variadic};
    let mut catalog = Catalog::new();
    let boolean = catalog.types.scalar(ScalarType::Bool);
    let header_pointer = catalog.types.pointer(catalog.header).unwrap();
    let descriptor_view = catalog.types.slice(header_pointer).unwrap();
    let flags = catalog.types.reserve_enum(IntegerType::U32);
    catalog
        .types
        .define_enum(
            flags,
            [8, 40].map(|n| Integer::checked(IntegerType::U32, n).unwrap()),
        )
        .unwrap();
    let procedure_record = catalog.types.reserve_record(RecordKind::Struct);
    catalog
        .types
        .define_record(
            procedure_record,
            [catalog.header, descriptor_view, descriptor_view, flags],
        )
        .unwrap();
    let mut builder = StaticDataBuilder::new();
    let boolean_object = builder.reserve(catalog.header, &catalog.types).unwrap();
    let graph = catalog.graph(boolean);
    builder
        .define_type_descriptor(
            boolean_object,
            catalog.header(2, 1),
            &graph,
            graph.root(),
            &catalog.types,
        )
        .unwrap();
    let backing_type = catalog.types.fixed_array(header_pointer, 100).unwrap();
    let backing = builder.reserve(backing_type, &catalog.types).unwrap();
    builder
        .define(
            backing,
            StaticValue {
                ty: backing_type,
                kind: StaticValueKind::Array(
                    (0..100)
                        .map(|_| StaticValue {
                            ty: header_pointer,
                            kind: StaticValueKind::Address(StaticAddress::new(boolean_object)),
                        })
                        .collect(),
                ),
            },
        )
        .unwrap();
    for (convention, flag) in [(CallingConvention::Jai, 8), (CallingConvention::C, 40)] {
        let procedure = catalog
            .types
            .procedure(ProcedureType {
                parameters: vec![boolean; 100].into(),
                results: Box::new([]),
                convention,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let graph = catalog.graph(procedure);
        let object = builder.reserve(procedure_record, &catalog.types).unwrap();
        builder
            .define_type_descriptor(
                object,
                StaticValue {
                    ty: procedure_record,
                    kind: StaticValueKind::Record(vec![
                        catalog.header(5, 8),
                        StaticValue {
                            ty: descriptor_view,
                            kind: StaticValueKind::Slice {
                                data: Some(
                                    StaticAddress::new(backing).project(StaticProjection::Index(0)),
                                ),
                                count: 100,
                            },
                        },
                        StaticValue {
                            ty: descriptor_view,
                            kind: StaticValueKind::Slice {
                                data: None,
                                count: 0,
                            },
                        },
                        StaticValue::constant(ConstantValue {
                            ty: flags,
                            kind: ConstantKind::Enum(
                                Integer::checked(IntegerType::U32, flag).unwrap(),
                            ),
                        }),
                    ]),
                },
                &graph,
                graph.root(),
                &catalog.types,
            )
            .unwrap();
    }
    // The one backing array has fewer than 250 stored nodes. Repeated view
    // scans and retained signature metadata are charged across all bindings.
    assert!(matches!(
        builder.publish(
            &catalog.types,
            StaticDataLimits {
                value_nodes: 250,
                ..Default::default()
            }
        ),
        Err(StaticDataError::Limit("descriptor work count"))
    ));
    builder
        .publish(&catalog.types, StaticDataLimits::default())
        .unwrap();
}

#[test]
fn ordinary_storage_cannot_gain_descriptor_identity_and_failed_bindings_are_retryable() {
    let catalog = Catalog::new();
    let mut ordinary = StaticDataBuilder::new();
    let object = ordinary.reserve(catalog.integer, &catalog.types).unwrap();
    ordinary.define(object, catalog.value(0, 4, true)).unwrap();
    let data = Arc::new(
        ordinary
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    assert!(matches!(
        RuntimeTypeConstant::new(data, object, &catalog.types),
        Err(StaticDataError::InvalidValue(_))
    ));
    let mut builder = StaticDataBuilder::new();
    let object = builder.reserve(catalog.integer, &catalog.types).unwrap();
    for value in [
        catalog.value(1, 4, true),
        catalog.value(0, 8, true),
        catalog.value(0, 4, false),
    ] {
        assert!(matches!(
            builder.define_type_descriptor(
                object,
                value,
                &catalog.graph,
                catalog.graph.root(),
                &catalog.types
            ),
            Err(StaticDataError::InvalidValue(_))
        ));
    }
    assert!(
        matches!(builder.publish(&catalog.types, StaticDataLimits::default()), Err(StaticDataError::IncompleteObject(id)) if id == object)
    );
    builder
        .define_type_descriptor(
            object,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    let data = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    RuntimeTypeConstant::new(data, object, &catalog.types).unwrap();
}

#[test]
fn duplicate_or_foreign_descriptor_bindings_cannot_be_published() {
    let catalog = Catalog::new();
    let foreign = Catalog::new();
    let mut builder = StaticDataBuilder::new();
    let first = builder.reserve(catalog.integer, &catalog.types).unwrap();
    assert!(matches!(
        builder.define_type_descriptor(
            first,
            catalog.value(0, 4, true),
            &foreign.graph,
            foreign.graph.root(),
            &catalog.types
        ),
        Err(StaticDataError::Type(jai_types::TypeError::ForeignType(_)))
    ));
    builder
        .define_type_descriptor(
            first,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    let published = Arc::new(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap(),
    );
    let duplicate = builder.reserve(catalog.integer, &catalog.types).unwrap();
    builder
        .define_type_descriptor(
            duplicate,
            catalog.value(0, 4, true),
            &catalog.graph,
            catalog.graph.root(),
            &catalog.types,
        )
        .unwrap();
    assert!(matches!(
        builder.publish(&catalog.types, StaticDataLimits::default()),
        Err(StaticDataError::InvalidValue(_))
    ));
    builder.discard_unpublished();
    assert_eq!(
        builder
            .publish(&catalog.types, StaticDataLimits::default())
            .unwrap()
            .objects()
            .len(),
        1
    );
    RuntimeTypeConstant::new(published, first, &catalog.types).unwrap();
}
