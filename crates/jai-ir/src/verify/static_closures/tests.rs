use super::*;
use jai_types::{
    Integer, LayoutPolicy, RecordKind, ReflectionGraph, ReflectionMetadata, ReflectionReadiness,
    TypeInfoTag, TypeRegistry,
};
use std::sync::Arc;

fn graph(types: &mut TypeRegistry, count: usize) -> StaticData {
    let boolean = types.scalar(ScalarType::Bool);
    let array = types.fixed_array(boolean, count as u64).unwrap();
    let mut builder = StaticDataBuilder::new();
    let root = builder.reserve(array, types).unwrap();
    builder
        .define(
            root,
            StaticValue {
                ty: array,
                kind: StaticValueKind::Array(vec![
                    StaticValue::constant(ConstantValue {
                        ty: boolean,
                        kind: ConstantKind::Bool(true),
                    });
                    count
                ]),
            },
        )
        .unwrap();
    builder.finish(types, StaticDataLimits::default()).unwrap()
}

#[test]
fn identical_immutable_closures_are_charged_and_verified_once() {
    let mut types = TypeRegistry::new();
    let data = Arc::new(graph(&mut types, 1024));
    let signatures = HashMap::new();
    let mut proof = StaticClosures::default();
    for _ in 0..1024 {
        let alias = Arc::clone(&data);
        proof.procedures(&alias, &types, &signatures).unwrap();
    }
    assert_eq!(proof.nodes, 2049);
    assert_eq!(proof.validated.len(), 1);
    assert_eq!(proof.procedures.len(), 1);
}

#[test]
fn distinct_closures_share_one_work_budget() {
    let mut types = TypeRegistry::new();
    let first = graph(&mut types, 3);
    let second = graph(&mut types, 3);
    let mut proof = StaticClosures {
        nodes: MAX_STATIC_WORK - 10,
        ..Default::default()
    };
    proof.validate(&first, &types).unwrap();
    assert_eq!(proof.nodes, MAX_STATIC_WORK - 3);
    assert!(matches!(
        proof.validate(&second, &types),
        Err(IrError::VerificationDepth)
    ));
    assert_eq!(proof.validated.len(), 1);
}

#[test]
fn expression_proof_reuses_static_closures_across_real_tree_leaves() {
    let mut types = TypeRegistry::new();
    let data = Arc::new(graph(&mut types, 1024));
    let root = data.objects()[0].id();
    let root_type = data.objects()[0].ty();
    let pointer = types.pointer(root_type).unwrap();
    let array = types.fixed_array(pointer, 1024).unwrap();
    let expression = ValueExpr::Array {
        ty: array,
        elements: vec![
            ValueExpr::StaticAddress {
                data,
                address: StaticAddress::new(root),
                ty: pointer,
            };
            1024
        ],
    };
    let signatures = HashMap::new();
    let places = PlaceRegistry::new().freeze();
    let proof = Context::root(&types, &signatures, &[], &places, None);
    proof.value(&expression).unwrap();
    let closures = proof.static_closures.borrow();
    assert_eq!(closures.nodes, 2049);
    assert_eq!(closures.validated.len(), 1);
    assert_eq!(closures.procedures.len(), 1);
}

#[test]
fn repeated_global_references_validate_the_immutable_initializer_once() {
    let types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    let global = Global::new_typed(
        0,
        ConstantValue {
            ty: boolean,
            kind: ConstantKind::Bool(true),
        },
        &types,
    )
    .unwrap();
    let place = global.place();
    let globals = [global];
    let signatures = HashMap::new();
    let places = PlaceRegistry::new().freeze();
    let proof = Context::root(&types, &signatures, &globals, &places, None);
    for _ in 0..1024 {
        proof.place(place).unwrap();
    }
    assert_eq!(proof.checked_globals.borrow().len(), 1);
}

#[test]
fn runtime_type_constant_arrays_share_shape_and_signature_closure_checks() {
    let mut types = TypeRegistry::new();
    let boolean = types.scalar(ScalarType::Bool);
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let tag = types.reserve_enum(IntegerType::U32);
    types
        .define_enum(
            tag,
            [Integer::checked(IntegerType::U32, TypeInfoTag::Bool as i128).unwrap()],
        )
        .unwrap();
    let header = types.reserve_record(RecordKind::Struct);
    types.define_record(header, [tag, s64]).unwrap();
    types.bind_runtime_type_header(header).unwrap();
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        boolean,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("bool descriptor must be ready")
    };
    let count = 1024;
    let array = types.fixed_array(boolean, count).unwrap();
    let mut builder = StaticDataBuilder::new();
    let descriptor = builder.reserve(header, &types).unwrap();
    builder
        .define_type_descriptor(
            descriptor,
            StaticValue {
                ty: header,
                kind: StaticValueKind::Record(vec![
                    StaticValue::constant(ConstantValue {
                        ty: tag,
                        kind: ConstantKind::Enum(
                            Integer::checked(IntegerType::U32, TypeInfoTag::Bool as i128).unwrap(),
                        ),
                    }),
                    StaticValue::constant(ConstantValue {
                        ty: s64,
                        kind: ConstantKind::Int(Integer::checked(IntegerType::S64, 1).unwrap()),
                    }),
                ]),
            },
            &graph,
            graph.root(),
            &types,
        )
        .unwrap();
    let extra = builder.reserve(array, &types).unwrap();
    builder
        .define(
            extra,
            StaticValue {
                ty: array,
                kind: StaticValueKind::Array(vec![
                    StaticValue::constant(ConstantValue {
                        ty: boolean,
                        kind: ConstantKind::Bool(true),
                    });
                    count as usize
                ]),
            },
        )
        .unwrap();
    let data = Arc::new(builder.finish(&types, StaticDataLimits::default()).unwrap());
    let value = RuntimeTypeConstant::new(data, descriptor, &types).unwrap();
    let root = ConstantValue {
        ty: types.fixed_array(types.meta_type(), count).unwrap(),
        kind: ConstantKind::Array(vec![
            ConstantValue {
                ty: types.meta_type(),
                kind: ConstantKind::RuntimeType(value),
            };
            count as usize
        ]),
    };
    let mut proof = StaticClosures::default();
    expressions::constant_with_closures(&types, &root, &mut proof).unwrap();
    expressions::constant_procedures_with_closures(&types, &root, &HashMap::new(), &mut proof)
        .unwrap();
    assert_eq!(proof.nodes, 2 * count as usize + 6);
    assert_eq!(proof.validated.len(), 1);
    assert_eq!(proof.procedures.len(), 1);
}
