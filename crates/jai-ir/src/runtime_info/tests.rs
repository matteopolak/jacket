use super::*;
use crate::{ConstantValue, StaticDataBuilder};
use jai_types::{
    FloatType, Integer, IntegerType, RecordKind, ReflectionGraph, ReflectionMetadata,
    ReflectionReadiness, RuntimeTypeSchema, ScalarLayout, ScalarType, TypeRegistry,
};

struct Fixture {
    types: TypeRegistry,
    schema: RuntimeInfoSchema,
    data: Arc<StaticData>,
    object: StaticObjectId,
    roots: [TypeId; 2],
}
impl Fixture {
    fn new(
        policy: LayoutPolicy,
        row_order: &[usize],
        start: u64,
        count: u64,
        global_null: bool,
    ) -> Self {
        Self::with_float_policy(policy, policy, row_order, start, count, global_null)
    }
    fn with_float_policy(
        policy: LayoutPolicy,
        float_policy: LayoutPolicy,
        row_order: &[usize],
        start: u64,
        count: u64,
        global_null: bool,
    ) -> Self {
        let mut types = TypeRegistry::new();
        let tag = types.reserve_enum(IntegerType::U32);
        types
            .define_enum(
                tag,
                [1, 2].map(|value| Integer::checked(IntegerType::U32, value).unwrap()),
            )
            .unwrap();
        let header = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                header,
                [tag, types.scalar(ScalarType::Int(IntegerType::S64))],
            )
            .unwrap();
        types.bind_runtime_type_header(header).unwrap();
        let float_descriptor = types.reserve_record(RecordKind::Struct);
        types.define_record(float_descriptor, [header]).unwrap();
        let runtime = RuntimeTypeSchema::from_view(&types).unwrap();
        let segment_tag = types.reserve_enum(IntegerType::U16);
        types
            .define_enum(
                segment_tag,
                [0, 1, 2, 3, 5].map(|value| Integer::checked(IntegerType::U16, value).unwrap()),
            )
            .unwrap();
        let bytes = types
            .slice(types.scalar(ScalarType::Int(IntegerType::U8)))
            .unwrap();
        let segment = types.reserve_record(RecordKind::Struct);
        types.define_record(segment, [segment_tag, bytes]).unwrap();
        let segments = types.slice(segment).unwrap();
        let global = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                global,
                [types.scalar(ScalarType::Int(IntegerType::U64)), segments],
            )
            .unwrap();
        let table_type = types.slice(runtime.descriptor_type()).unwrap();
        let global_pointer = types.pointer(global).unwrap();
        let info = types.reserve_record(RecordKind::Struct);
        types
            .define_record(info, [table_type, global_pointer])
            .unwrap();
        let schema = RuntimeInfoSchema::validate(&types, info, global).unwrap();
        let roots = [types.scalar(ScalarType::Bool), types.float(FloatType::F32)];
        let ReflectionReadiness::Ready(graph) = ReflectionGraph::build_with_roots(
            &types,
            roots[0],
            &roots[1..],
            Some(policy),
            &ReflectionMetadata::default(),
        )
        .unwrap() else {
            panic!("ready builtin roots")
        };
        let mut builder = StaticDataBuilder::new();
        let descriptor_value = |tag_value, size| StaticValue {
            ty: header,
            kind: StaticValueKind::Record(vec![
                StaticValue::constant(ConstantValue {
                    ty: tag,
                    kind: ConstantKind::Enum(
                        Integer::checked(IntegerType::U32, tag_value).unwrap(),
                    ),
                }),
                StaticValue::constant(ConstantValue {
                    ty: types.scalar(ScalarType::Int(IntegerType::S64)),
                    kind: ConstantKind::Int(Integer::checked(IntegerType::S64, size).unwrap()),
                }),
            ]),
        };
        let boolean = builder.reserve(header, &types).unwrap();
        builder
            .define_type_descriptor(
                boolean,
                descriptor_value(2, 1),
                &graph,
                graph.descriptor(roots[0]).unwrap().id,
                &types,
            )
            .unwrap();
        let float = builder.reserve(float_descriptor, &types).unwrap();
        let ReflectionReadiness::Ready(float_graph) = ReflectionGraph::build(
            &types,
            roots[1],
            Some(float_policy),
            &ReflectionMetadata::default(),
        )
        .unwrap() else {
            panic!("ready float descriptor")
        };
        builder
            .define_type_descriptor(
                float,
                StaticValue {
                    ty: float_descriptor,
                    kind: StaticValueKind::Record(vec![descriptor_value(1, 4)]),
                },
                &float_graph,
                float_graph.root(),
                &types,
            )
            .unwrap();
        let unbound = builder.reserve(header, &types).unwrap();
        builder.define(unbound, descriptor_value(2, 1)).unwrap();
        let objects = [
            StaticAddress::new(boolean),
            StaticAddress::new(float).project(StaticProjection::Field(
                types.field(float_descriptor, 0).unwrap().id,
            )),
            StaticAddress::new(unbound),
        ];
        let backing = types
            .fixed_array(runtime.descriptor_type(), row_order.len() as u64)
            .unwrap();
        let backing_object = builder.reserve(backing, &types).unwrap();
        builder
            .define(
                backing_object,
                StaticValue {
                    ty: backing,
                    kind: StaticValueKind::Array(
                        row_order
                            .iter()
                            .map(|&index| StaticValue {
                                ty: runtime.descriptor_type(),
                                kind: StaticValueKind::Address(objects[index].clone()),
                            })
                            .collect(),
                    ),
                },
            )
            .unwrap();
        let global_value = if global_null {
            StaticValue::constant(ConstantValue {
                ty: global_pointer,
                kind: ConstantKind::Zero,
            })
        } else {
            let global_object = builder.reserve(global, &types).unwrap();
            builder
                .define(
                    global_object,
                    StaticValue::constant(ConstantValue {
                        ty: global,
                        kind: ConstantKind::Zero,
                    }),
                )
                .unwrap();
            StaticValue {
                ty: global_pointer,
                kind: StaticValueKind::Address(StaticAddress::new(global_object)),
            }
        };
        let object = builder.reserve(info, &types).unwrap();
        builder
            .define(
                object,
                StaticValue {
                    ty: info,
                    kind: StaticValueKind::Record(vec![
                        StaticValue {
                            ty: table_type,
                            kind: StaticValueKind::Slice {
                                data: if row_order.is_empty() {
                                    None
                                } else {
                                    Some(
                                        StaticAddress::new(backing_object)
                                            .project(StaticProjection::Index(start)),
                                    )
                                },
                                count,
                            },
                        },
                        global_value,
                    ]),
                },
            )
            .unwrap();
        let data = Arc::new(
            builder
                .publish(&types, StaticDataLimits::default())
                .unwrap(),
        );
        Self {
            types,
            schema,
            data,
            object,
            roots,
        }
    }
    fn snapshot(
        &self,
        expected: &[TypeId],
        policy: LayoutPolicy,
    ) -> Result<RuntimeInfoSnapshot, RuntimeInfoSnapshotError> {
        RuntimeInfoSnapshot::new_compile_time(
            Arc::clone(&self.data),
            self.object,
            self.schema,
            &self.types,
            policy,
            expected,
        )
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
fn sealed_rows_preserve_order_and_exact_headers_without_exposing_backing_types() {
    for policy in [LayoutPolicy::lp64(), ilp32()] {
        let fixture = Fixture::new(policy, &[1, 0, 1], 1, 2, true);
        let snapshot = fixture.snapshot(&fixture.roots, policy).unwrap();
        assert_eq!(
            snapshot.represented_types().collect::<Vec<_>>(),
            fixture.roots
        );
        assert_eq!(snapshot.address(), &StaticAddress::new(fixture.object));
        assert!(Arc::ptr_eq(snapshot.data(), &fixture.data));
        assert_eq!(snapshot.schema(), fixture.schema);
        snapshot.revalidate(&fixture.types, policy).unwrap();
        let frozen = fixture.types.freeze().unwrap();
        snapshot.revalidate(&frozen, policy).unwrap();
        assert!(matches!(
            snapshot.revalidate(&TypeRegistry::new(), policy),
            Err(RuntimeInfoSnapshotError::Schema(RuntimeInfoError::Type(_)))
        ));
        let other_policy = if policy == ilp32() {
            LayoutPolicy::lp64()
        } else {
            ilp32()
        };
        assert!(matches!(
            snapshot.revalidate(&frozen, other_policy),
            Err(RuntimeInfoSnapshotError::TargetMismatch)
        ));
    }
}

#[test]
fn ordered_checkpoint_rejects_missing_reordered_and_duplicate_rows() {
    let fixture = Fixture::new(LayoutPolicy::lp64(), &[0, 1], 0, 2, true);
    assert!(matches!(
        fixture.snapshot(&fixture.roots[..1], LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::InvalidTypeTable)
    ));
    assert!(matches!(
        fixture.snapshot(&[fixture.roots[1], fixture.roots[0]], LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::DescriptorMismatch)
    ));
    let duplicate = Fixture::new(LayoutPolicy::lp64(), &[0, 0], 0, 2, true);
    assert!(matches!(
        duplicate.snapshot(
            &[duplicate.roots[0], duplicate.roots[0]],
            LayoutPolicy::lp64()
        ),
        Err(RuntimeInfoSnapshotError::DuplicateType(_))
    ));
}

#[test]
fn ordinary_lookalike_headers_and_different_target_descriptors_cannot_certify_rows() {
    let fixture = Fixture::new(LayoutPolicy::lp64(), &[0, 2], 0, 2, true);
    assert!(matches!(
        fixture.snapshot(&fixture.roots, LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::DescriptorMismatch)
    ));
    let fixture = Fixture::new(LayoutPolicy::lp64(), &[0, 1], 0, 2, true);
    assert!(matches!(
        fixture.snapshot(&fixture.roots, ilp32()),
        Err(RuntimeInfoSnapshotError::TargetMismatch)
    ));
    let mixed = Fixture::with_float_policy(LayoutPolicy::lp64(), ilp32(), &[0], 0, 1, true);
    assert!(matches!(
        mixed.snapshot(&mixed.roots[..1], LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::TargetMismatch)
    ));
}

#[test]
fn compile_time_global_data_is_genuine_null_and_empty_checkpoints_are_explicit() {
    let fixture = Fixture::new(LayoutPolicy::lp64(), &[0, 1], 0, 2, false);
    assert!(matches!(
        fixture.snapshot(&fixture.roots, LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::GlobalDataNotNull)
    ));
    let empty = Fixture::new(LayoutPolicy::lp64(), &[], 0, 0, true);
    let snapshot = empty.snapshot(&[], LayoutPolicy::lp64()).unwrap();
    assert!(snapshot.rows().is_empty());
    assert!(matches!(
        empty.snapshot(&empty.roots, LayoutPolicy::lp64()),
        Err(RuntimeInfoSnapshotError::InvalidTypeTable)
    ));
}
