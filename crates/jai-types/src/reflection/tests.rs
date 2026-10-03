use super::*;
use crate::{ProcedureType, ScalarLayout, ScalarType, TypeRegistry};

#[test]
fn hiding_record_members_preserves_layout_and_source_record_metadata() {
    let mut types = TypeRegistry::new();
    let record = types.reserve_record(RecordKind::Struct);
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let wide = types.scalar(ScalarType::Int(IntegerType::U64));
    types.define_record(record, vec![byte, wide]).unwrap();
    types
        .add_record_reflection_flags(
            record,
            crate::RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
        )
        .unwrap();
    let mut metadata = ReflectionMetadata::default();
    metadata.name(record, b"PrivateRecord".as_slice());
    metadata.record(
        record,
        ReflectedRecordMetadata {
            notes: vec![b"@KeepRecord".to_vec().into_boxed_slice()].into_boxed_slice(),
            unsupported_members: true,
            ..ReflectedRecordMetadata::default()
        },
    );
    let graph = ready(ReflectionGraph::build(
        &types,
        record,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    assert_eq!(graph.descriptors().len(), 1);
    let descriptor = graph.get(graph.root()).unwrap();
    assert_eq!(
        descriptor.name.as_deref(),
        Some(b"PrivateRecord".as_slice())
    );
    assert_eq!(descriptor.runtime_size(), Some(16));
    assert_eq!(
        descriptor.layout.as_ref().unwrap().field_offsets.as_ref(),
        [0, 8]
    );
    let DescriptorKind::Record {
        fields, metadata, ..
    } = &descriptor.kind
    else {
        panic!()
    };
    assert!(fields.is_empty());
    assert_eq!(metadata.notes[0].as_ref(), b"@KeepRecord");
    assert_eq!(types.field(record, 1).unwrap().ty, wide);
}

#[test]
fn reducing_procedure_members_changes_only_the_immutable_reflection_edge() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let procedure = types
        .procedure(ProcedureType {
            parameters: vec![integer].into(),
            results: vec![integer].into(),
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, vec![procedure]).unwrap();
    let field = types.field(record, 0).unwrap();
    let metadata = ReflectionMetadata::default();
    let original = ready(ReflectionGraph::build(
        &types,
        record,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    types
        .add_record_reflection_flags(
            record,
            crate::RecordReflectionPolicy::from_flags([
                RecordReflectionFlag::ProceduresAreVoidPointers,
                RecordReflectionFlag::NoSizeComplaint,
            ]),
        )
        .unwrap();
    let frozen = types.freeze().unwrap();
    let reduced = ready(ReflectionGraph::build(
        &frozen,
        record,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    let DescriptorKind::Record { fields, .. } = &reduced.descriptor(record).unwrap().kind else {
        panic!()
    };
    assert_eq!(fields[0].id, field.id);
    assert_eq!(fields[0].offset_in_bytes, 0);
    assert!(
        fields[0]
            .procedure_as_void_pointer(&frozen, record)
            .unwrap()
    );
    let void_pointer = frozen
        .lookup(&TypeKind::Pointer(frozen.lookup(&TypeKind::Void).unwrap()))
        .unwrap();
    assert_eq!(fields[0].ty.represented_type(), void_pointer);
    assert!(
        matches!(reduced.descriptor(procedure), Err(ReflectionError::UnknownDescriptor(ty)) if ty == procedure)
    );
    assert!(
        matches!(reduced.descriptor(integer), Err(ReflectionError::UnknownDescriptor(ty)) if ty == integer)
    );
    assert_eq!(frozen.field(record, 0).unwrap().ty, procedure);
    assert_eq!(
        original.descriptor(record).unwrap().layout,
        reduced.descriptor(record).unwrap().layout
    );
    let DescriptorKind::Record { fields, .. } = &original.descriptor(record).unwrap().kind else {
        panic!()
    };
    assert_eq!(fields[0].ty.represented_type(), procedure);
    assert!(
        !fields[0]
            .procedure_as_void_pointer(&frozen, record)
            .unwrap()
    );
}

#[test]
fn unnamed_physical_field_retains_promotion_and_actual_union_layout() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let wide = types.scalar(ScalarType::Int(IntegerType::U64));
    let pair = types.reserve_record(RecordKind::Struct);
    types.define_record(pair, vec![wide, wide]).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, vec![wide, pair]).unwrap();
    let root = types.reserve_record(RecordKind::Struct);
    types.define_record(root, vec![byte, union, byte]).unwrap();
    let embedded = types.field(root, 1).unwrap();
    let mut metadata = ReflectionMetadata::default();
    metadata.field(
        embedded.id,
        ReflectedFieldMetadata {
            name: None,
            using: true,
        },
    );
    let graph = ready(ReflectionGraph::build(
        &types,
        root,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    let DescriptorKind::Record { fields, .. } = &graph.get(graph.root()).unwrap().kind else {
        panic!()
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.offset_in_bytes)
            .collect::<Vec<_>>(),
        [0, 8, 24]
    );
    assert_eq!(fields[1].id, embedded.id);
    assert_eq!(fields[1].name, None);
    assert!(fields[1].using);
    let union = graph.get(fields[1].ty).unwrap();
    assert_eq!(union.runtime_size(), Some(16));
    let DescriptorKind::Record { fields, .. } = &union.kind else {
        panic!()
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.offset_in_bytes)
            .collect::<Vec<_>>(),
        [0, 0]
    );
}

#[test]
fn universal_storage_has_a_leaf_any_descriptor_after_header_completion() {
    let mut types = TypeRegistry::new();
    let any = types.reserve_any();
    let metadata = ReflectionMetadata::default();
    assert!(matches!(
        ReflectionGraph::build(&types, any, Some(LayoutPolicy::lp64()), &metadata).unwrap(),
        ReflectionReadiness::Pending(_)
    ));
    let header = types.reserve_record(RecordKind::Struct);
    let tag = types.scalar(ScalarType::Int(IntegerType::U32));
    let size = types.scalar(ScalarType::Int(IntegerType::S64));
    types.define_record(header, vec![tag, size]).unwrap();
    types.define_any(any, header).unwrap();
    let graph = ready(ReflectionGraph::build(
        &types,
        any,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    assert_eq!(graph.descriptors().len(), 1);
    let descriptor = graph.get(graph.root()).unwrap();
    assert_eq!(descriptor.kind, DescriptorKind::Any);
    assert_eq!(descriptor.tag() as u32, 10);
    assert_eq!(descriptor.runtime_size(), Some(16));
}

fn ready(result: Result<ReflectionReadiness<ReflectionGraph>, ReflectionError>) -> ReflectionGraph {
    match result.unwrap() {
        ReflectionReadiness::Ready(graph) => graph,
        ReflectionReadiness::Pending(dependencies) => {
            panic!("unexpected dependencies: {dependencies:?}")
        }
    }
}

#[test]
fn certified_roots_share_one_ordered_closure_and_nominal_descriptor_identity() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    let first = types.reserve_record(RecordKind::Struct);
    let second = types.reserve_record(RecordKind::Struct);
    types.define_record(first, vec![integer]).unwrap();
    types.define_record(second, vec![integer]).unwrap();
    let pointer = types.pointer(first).unwrap();
    let graph = ready(ReflectionGraph::build_with_roots(
        &types,
        first,
        &[second, pointer, first, second],
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    ));
    assert_eq!(graph.root().represented_type(), first);
    assert_eq!(
        graph
            .descriptors()
            .iter()
            .map(|descriptor| descriptor.id.represented_type())
            .collect::<Vec<_>>(),
        [first, second, pointer, integer]
    );
    assert_eq!(
        graph.descriptor(pointer).unwrap().kind,
        DescriptorKind::Pointer {
            pointee: graph.root()
        }
    );
    let DescriptorKind::Record { fields, .. } = &graph.descriptor(second).unwrap().kind else {
        panic!("second nominal record");
    };
    assert_eq!(fields[0].ty, graph.descriptor(integer).unwrap().id);
}

#[test]
fn additional_roots_preserve_pending_and_arena_checks_before_publication() {
    let mut types = TypeRegistry::new();
    let root = types.scalar(ScalarType::Bool);
    let pending = types.reserve_record(RecordKind::Struct);
    assert!(matches!(
        ReflectionGraph::build_with_roots(
            &types,
            root,
            &[pending, pending],
            Some(LayoutPolicy::lp64()),
            &ReflectionMetadata::default(),
        )
        .unwrap(),
        ReflectionReadiness::Pending(dependencies)
            if dependencies.as_ref() == [ReflectionDependency::Definition(pending)]
    ));
    let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
    assert!(matches!(
        ReflectionGraph::build_with_roots(
            &types,
            root,
            &[foreign],
            None,
            &ReflectionMetadata::default(),
        ),
        Err(ReflectionError::Type(TypeError::ForeignType(ty))) if ty == foreign
    ));
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
fn runtime_type_descriptor_uses_the_selected_pointer_layout() {
    let types = TypeRegistry::new();
    let ty = types.meta_type();
    for (policy, size) in [(ilp32(), 4), (LayoutPolicy::lp64(), 8)] {
        let graph = ready(ReflectionGraph::build(
            &types,
            ty,
            Some(policy),
            &ReflectionMetadata::default(),
        ));
        let descriptor = graph.get(graph.root()).unwrap();
        assert_eq!(descriptor.kind, DescriptorKind::Type);
        assert_eq!(descriptor.tag() as u32, 13);
        assert_eq!(descriptor.runtime_size(), Some(size));
    }
    assert!(matches!(
        ReflectionGraph::build(&types, ty, None, &ReflectionMetadata::default()).unwrap(),
        ReflectionReadiness::Pending(dependencies)
            if dependencies.as_ref() == [ReflectionDependency::TargetLayout]
    ));
}

#[test]
fn record_names_offsets_and_recursion_follow_the_target() {
    let mut types = TypeRegistry::new();
    let record = types.reserve_record(RecordKind::Struct);
    let next = types.pointer(record).unwrap();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    types.define_record(record, vec![byte, next]).unwrap();
    let mut metadata = ReflectionMetadata::default();
    metadata.name(record, b"Node".as_slice());
    for (index, name) in [b"value".as_slice(), b"next".as_slice()]
        .into_iter()
        .enumerate()
    {
        metadata.field(
            types.field(record, index).unwrap().id,
            ReflectedFieldMetadata {
                name: Some(name.into()),
                using: false,
            },
        );
    }
    let graph32 = ready(ReflectionGraph::build(
        &types,
        record,
        Some(ilp32()),
        &metadata,
    ));
    let graph64 = ready(ReflectionGraph::build(
        &types,
        record,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    assert_eq!(graph32.descriptors().len(), 3);
    assert_eq!(graph32.get(graph32.root()).unwrap().runtime_size(), Some(8));
    assert_eq!(
        graph64.get(graph64.root()).unwrap().runtime_size(),
        Some(16)
    );
    let descriptor = graph32.get(graph32.root()).unwrap();
    assert_eq!(descriptor.name.as_deref(), Some(b"Node".as_slice()));
    let DescriptorKind::Record { fields, .. } = &descriptor.kind else {
        panic!("record")
    };
    assert_eq!(fields[1].offset_in_bytes, 4);
    assert_eq!(fields[1].name.as_deref(), Some(b"next".as_slice()));
    let DescriptorKind::Pointer { pointee } = &graph32.get(fields[1].ty).unwrap().kind else {
        panic!("pointer")
    };
    assert_eq!(*pointee, graph32.root());
}

#[test]
fn missing_target_and_incomplete_definitions_are_pending() {
    let mut types = TypeRegistry::new();
    let record = types.reserve_record(RecordKind::Struct);
    assert!(
        matches!(ReflectionGraph::build(&types, record, None, &ReflectionMetadata::default()).unwrap(),
        ReflectionReadiness::Pending(dependencies) if dependencies.as_ref() == [ReflectionDependency::TargetLayout])
    );
    assert!(
        matches!(ReflectionGraph::build(&types, record, Some(LayoutPolicy::lp64()), &ReflectionMetadata::default()).unwrap(),
        ReflectionReadiness::Pending(dependencies) if dependencies.as_ref() == [ReflectionDependency::Definition(record)])
    );
    types.define_record(record, vec![]).unwrap();
    assert_eq!(
        ready(ReflectionGraph::build(
            &types,
            record,
            Some(LayoutPolicy::lp64()),
            &ReflectionMetadata::default()
        ))
        .get(DescriptorId(record))
        .unwrap()
        .runtime_size(),
        Some(0)
    );
}

#[test]
fn nominal_identity_and_arena_ownership_survive_reflection() {
    let mut types = TypeRegistry::new();
    let first = types.reserve_record(RecordKind::Struct);
    let second = types.reserve_record(RecordKind::Struct);
    types.define_record(first, vec![]).unwrap();
    types.define_record(second, vec![]).unwrap();
    let graph = ready(ReflectionGraph::build(
        &types,
        first,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    ));
    assert!(
        matches!(graph.descriptor(second), Err(ReflectionError::UnknownDescriptor(ty)) if ty == second)
    );
    let foreign = TypeRegistry::new().scalar(ScalarType::Bool);
    assert!(
        matches!(ReflectionGraph::build(&types, foreign, Some(LayoutPolicy::lp64()), &ReflectionMetadata::default()), Err(ReflectionError::Type(TypeError::ForeignType(ty))) if ty == foreign)
    );
}

#[test]
fn enums_preserve_ordered_alias_names_and_integer_representation() {
    let mut types = TypeRegistry::new();
    let enumeration = types.reserve_enum(IntegerType::U8);
    let values = [
        Integer::wrapping(IntegerType::U8, 0),
        Integer::wrapping(IntegerType::U8, 1),
        Integer::wrapping(IntegerType::U8, 1),
    ];
    types.define_enum(enumeration, values.as_slice()).unwrap();
    let mut metadata = ReflectionMetadata::default();
    metadata.name(enumeration, b"Mode".as_slice());
    metadata.enumeration(
        enumeration,
        ReflectedEnumMetadata {
            members: ["NONE", "ACTIVE", "ENABLED"]
                .into_iter()
                .zip(values)
                .map(|(name, value)| ReflectedEnumMember {
                    name: Some(name.as_bytes().into()),
                    value,
                })
                .collect(),
            flags: true,
        },
    );
    let graph = ready(ReflectionGraph::build(
        &types,
        enumeration,
        Some(LayoutPolicy::lp64()),
        &metadata,
    ));
    let DescriptorKind::Enum {
        representation,
        members,
        flags,
    } = &graph.get(graph.root()).unwrap().kind
    else {
        panic!("enum")
    };
    assert_eq!(*representation, IntegerType::U8);
    assert!(*flags);
    assert_eq!(members[2].name.as_deref(), Some(b"ENABLED".as_slice()));
    assert_eq!(members[1].value, members[2].value);
    assert_eq!(graph.get(graph.root()).unwrap().runtime_size(), Some(1));
    metadata.enumeration(
        enumeration,
        ReflectedEnumMetadata {
            members: vec![ReflectedEnumMember {
                name: None,
                value: values[0],
            }]
            .into(),
            flags: false,
        },
    );
    assert!(
        matches!(ReflectionGraph::build(&types, enumeration, Some(LayoutPolicy::lp64()), &metadata), Err(ReflectionError::EnumMetadata(ty)) if ty == enumeration)
    );
}

#[test]
fn procedure_metatypes_keep_parameter_result_ids_and_context() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    let boolean = types.scalar(ScalarType::Bool);
    let procedure = types
        .procedure(ProcedureType {
            parameters: vec![integer].into(),
            results: vec![boolean, integer].into(),
            convention: CallingConvention::C,
            context: ContextMode::None,
            variadic: crate::Variadic::None,
        })
        .unwrap();
    let graph = ready(ReflectionGraph::build(
        &types,
        procedure,
        Some(ilp32()),
        &ReflectionMetadata::default(),
    ));
    let descriptor = graph.get(graph.root()).unwrap();
    assert_eq!(descriptor.runtime_size(), Some(4));
    let DescriptorKind::Procedure {
        parameters,
        results,
        convention,
        context,
        ..
    } = &descriptor.kind
    else {
        panic!("procedure")
    };
    assert_eq!(parameters[0].represented_type(), integer);
    assert_eq!(results[0].represented_type(), boolean);
    assert_eq!(*convention, CallingConvention::C);
    assert_eq!(*context, ContextMode::None);
}

#[test]
fn fixed_arrays_and_unions_use_layout_offsets_and_strides() {
    let mut types = TypeRegistry::new();
    let byte = types.scalar(ScalarType::Int(IntegerType::U8));
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    let array = types.fixed_array(integer, 3).unwrap();
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, vec![byte, array]).unwrap();
    let graph = ready(ReflectionGraph::build(
        &types,
        union,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    ));
    let DescriptorKind::Record { kind, fields, .. } = &graph.get(graph.root()).unwrap().kind else {
        panic!("record")
    };
    assert_eq!(*kind, RecordKind::Union);
    assert_eq!(
        fields
            .iter()
            .map(|field| field.offset_in_bytes)
            .collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(graph.get(graph.root()).unwrap().runtime_size(), Some(12));
    assert!(matches!(
        &graph.get(fields[1].ty).unwrap().kind,
        DescriptorKind::FixedArray {
            count: 3,
            stride: 4,
            ..
        }
    ));
}

#[test]
fn void_descriptions_exist_without_inventing_storage() {
    let types = TypeRegistry::new();
    let graph = ready(ReflectionGraph::build(
        &types,
        types.void(),
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    ));
    assert_eq!(graph.get(graph.root()).unwrap().tag(), TypeInfoTag::Void);
    assert_eq!(graph.get(graph.root()).unwrap().runtime_size(), None);
    assert!(matches!(
        reflected_size(&types, types.void(), Some(LayoutPolicy::lp64())),
        Err(ReflectionError::Layout(LayoutError::Unsized(_)))
    ));
}
#[test]
fn code_metatype_is_described_without_runtime_storage() {
    let mut types = TypeRegistry::new();
    let code = types.code_type();
    assert!(matches!(types.fixed_array(code, 1), Err(TypeError::NotAValue(id)) if id == code));
    assert!(
        matches!(LayoutEngine::new(&types, LayoutPolicy::lp64()).layout(code), Err(LayoutError::Unsized(id)) if id == code)
    );
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        code,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("code description must be ready");
    };
    assert_eq!(graph.get(graph.root()).unwrap().tag(), TypeInfoTag::Code);
    assert_eq!(graph.get(graph.root()).unwrap().runtime_size(), None);
    assert_eq!(TypeInfoTag::Code as u32, 14);
}
