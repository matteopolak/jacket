use jai_types::*;

fn signature(types: &mut TypeRegistry) -> ProcedureType {
    let record = types.reserve_record(RecordKind::Struct);
    let float = types.float(FloatType::F32);
    types.define_record(record, [float, float]).unwrap();
    ProcedureType {
        parameters: Box::new([]),
        results: Box::new([record]),
        convention: CallingConvention::C,
        return_abi: ForeignReturnAbi::CppNonPod,
        context: ContextMode::None,
        variadic: Variadic::None,
    }
}

#[test]
fn result_policy_is_canonical_identity_and_reflected_metadata() {
    let mut types = TypeRegistry::new();
    let forced = signature(&mut types);
    let ty = types.procedure(forced.clone()).unwrap();
    assert_eq!(types.procedure(forced.clone()).unwrap(), ty);
    let natural = types
        .procedure(ProcedureType {
            return_abi: ForeignReturnAbi::Natural,
            ..forced
        })
        .unwrap();
    assert_ne!(natural, ty);
    let ReflectionReadiness::Ready(graph) = ReflectionGraph::build(
        &types,
        ty,
        Some(LayoutPolicy::lp64()),
        &ReflectionMetadata::default(),
    )
    .unwrap() else {
        panic!("complete signature");
    };
    assert!(matches!(
        graph.get(graph.root()).unwrap().kind,
        DescriptorKind::Procedure {
            return_abi: ForeignReturnAbi::CppNonPod,
            ..
        }
    ));
}

#[test]
fn non_pod_policy_rejects_scalar_multiple_context_and_jai_signatures() {
    let mut types = TypeRegistry::new();
    let valid = signature(&mut types);
    let integer = types.scalar(ScalarType::Int(IntegerType::S32));
    for (changed, expected) in [
        (
            ProcedureType {
                results: Box::new([integer]),
                ..valid.clone()
            },
            ForeignReturnIssue::RecordResult,
        ),
        (
            ProcedureType {
                results: Box::new([]),
                ..valid.clone()
            },
            ForeignReturnIssue::ResultCount,
        ),
        (
            ProcedureType {
                results: Box::new([valid.results[0], valid.results[0]]),
                ..valid.clone()
            },
            ForeignReturnIssue::ResultCount,
        ),
        (
            ProcedureType {
                context: ContextMode::Implicit,
                ..valid.clone()
            },
            ForeignReturnIssue::Context,
        ),
        (
            ProcedureType {
                convention: CallingConvention::Jai,
                ..valid.clone()
            },
            ForeignReturnIssue::CallingConvention,
        ),
    ] {
        assert!(
            matches!(types.procedure(changed), Err(TypeError::InvalidForeignReturn(issue)) if issue == expected)
        );
    }
    let distinct = types.reserve_distinct(DistinctKind::Distinct);
    types.define_distinct(distinct, valid.results[0]).unwrap();
    types
        .procedure(ProcedureType {
            results: Box::new([distinct]),
            ..valid
        })
        .unwrap();
}
