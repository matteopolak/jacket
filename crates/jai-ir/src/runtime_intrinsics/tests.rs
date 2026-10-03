use super::*;
use jai_types::{DistinctKind, FloatType, ProcedureType, RecordKind, TypeRegistry};

fn signature(types: &mut TypeRegistry, parameters: &[TypeId], results: &[TypeId]) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap()
}

#[test]
fn pools_bind_exact_nominal_self_types_and_target_storage() {
    let mut types = TypeRegistry::new();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let void = types.void();
    let data = types.pointer(void).unwrap();
    let pool = types.reserve_record(RecordKind::Struct);
    types.define_record(pool, [s64, s64, data, s64]).unwrap();
    let flat = types.reserve_record(RecordKind::Struct);
    types
        .define_record(flat, [s64, s64, data, s64, s64])
        .unwrap();
    let pool_pointer = types.pointer(pool).unwrap();
    let flat_pointer = types.pointer(flat).unwrap();
    let lp64 = LayoutPolicy::lp64();
    let ilp32 = LayoutPolicy::new(
        jai_types::ScalarLayout::new(4, 4),
        [
            jai_types::ScalarLayout::new(1, 1),
            jai_types::ScalarLayout::new(2, 2),
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        [
            jai_types::ScalarLayout::new(4, 4),
            jai_types::ScalarLayout::new(8, 4),
        ],
        jai_types::ScalarLayout::new(1, 1),
    )
    .unwrap();
    for (name, parameters, results, expected) in [
        (
            "get",
            vec![pool_pointer, s64],
            vec![data],
            RuntimeIntrinsic::PoolGet {
                pool,
            },
        ),
        (
            "reset",
            vec![pool_pointer],
            vec![],
            RuntimeIntrinsic::PoolReset {
                pool,
            },
        ),
        (
            "release",
            vec![pool_pointer],
            vec![],
            RuntimeIntrinsic::PoolRelease {
                pool,
            },
        ),
        (
            "get",
            vec![flat_pointer, s64],
            vec![data],
            RuntimeIntrinsic::FlatPoolGet {
                pool: flat,
            },
        ),
        (
            "reset",
            vec![flat_pointer, boolean],
            vec![],
            RuntimeIntrinsic::FlatPoolReset {
                pool: flat,
            },
        ),
        (
            "fini",
            vec![flat_pointer],
            vec![],
            RuntimeIntrinsic::FlatPoolFinish {
                pool: flat,
            },
        ),
    ] {
        let signature = signature(&mut types, &parameters, &results);
        for policy in [lp64, ilp32] {
            assert_eq!(
                RuntimeIntrinsic::bind(name, signature, &types, policy).unwrap(),
                expected
            );
        }
    }
    let wrong = signature(&mut types, &[flat_pointer], &[]);
    assert!(RuntimeIntrinsic::bind("release", wrong, &types, lp64).is_err());
    let wrong = signature(&mut types, &[pool_pointer, boolean], &[]);
    assert!(RuntimeIntrinsic::bind("reset", wrong, &types, lp64).is_err());
    let wrong = signature(&mut types, &[data, s64], &[data]);
    assert!(RuntimeIntrinsic::bind("get", wrong, &types, lp64).is_err());
    let other = types.reserve_record(RecordKind::Struct);
    types.define_record(other, [s64, s64, data, s64]).unwrap();
    let other_pointer = types.pointer(other).unwrap();
    let other_signature = signature(&mut types, &[other_pointer, s64], &[data]);
    assert!(
        RuntimeIntrinsic::PoolGet {
            pool
        }
        .validate_signature_shape(other_signature, &types)
        .is_err()
    );
    let union = types.reserve_record(RecordKind::Union);
    types.define_record(union, [s64, s64, data, s64]).unwrap();
    assert!(RuntimeIntrinsic::validate_pool_storage(&types, union, false, lp64).is_err());
}

#[test]
fn memory_signatures_are_exact_and_tag_catalog_is_closed() {
    let mut types = TypeRegistry::new();
    let void = types.void();
    let pointer = types.pointer(void).unwrap();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let s16 = types.scalar(ScalarType::Int(IntegerType::S16));
    let u8 = types.scalar(ScalarType::Int(IntegerType::U8));
    for (name, parameters, results, expected) in [
        (
            "memcpy",
            vec![pointer, pointer, s64],
            vec![],
            RuntimeIntrinsic::MemoryCopy,
        ),
        (
            "memcpy",
            vec![pointer, pointer, s64],
            vec![pointer],
            RuntimeIntrinsic::MemoryCopyReturningDestination,
        ),
        (
            "memcmp",
            vec![pointer, pointer, s64],
            vec![s16],
            RuntimeIntrinsic::MemoryCompare,
        ),
        (
            "memset",
            vec![pointer, u8, s64],
            vec![],
            RuntimeIntrinsic::MemorySet,
        ),
        (
            "memset",
            vec![pointer, s64, s64],
            vec![pointer],
            RuntimeIntrinsic::MemorySetReturningDestination,
        ),
        (
            "llvm.debugtrap",
            vec![],
            vec![],
            RuntimeIntrinsic::DebugTrap,
        ),
    ] {
        let ty = signature(&mut types, &parameters, &results);
        assert_eq!(
            RuntimeIntrinsic::bind(name, ty, &types, LayoutPolicy::lp64()).unwrap(),
            expected
        );
    }
    let invalid = signature(&mut types, &[pointer, pointer, u8], &[]);
    assert!(RuntimeIntrinsic::bind("memcpy", invalid, &types, LayoutPolicy::lp64()).is_err());
    for (name, parameters, results) in [
        ("memcpy", vec![pointer, pointer, s64], vec![s64]),
        ("memset", vec![pointer, u8, s64], vec![pointer]),
        ("memset", vec![pointer, s64, s64], vec![]),
    ] {
        let invalid = signature(&mut types, &parameters, &results);
        assert!(RuntimeIntrinsic::bind(name, invalid, &types, LayoutPolicy::lp64()).is_err());
    }
    let empty = signature(&mut types, &[], &[]);
    assert!(matches!(
        RuntimeIntrinsic::bind("llvm.arbitrary", empty, &types, LayoutPolicy::lp64()),
        Err(RuntimeIntrinsicError::Unknown(_))
    ));
}

#[test]
fn swap_preserves_exact_aggregate_identity_and_rejects_compiler_only_storage() {
    let mut types = TypeRegistry::new();
    let integer = types.scalar(ScalarType::Int(IntegerType::S64));
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [integer, integer]).unwrap();
    let pointer = types.pointer(record).unwrap();
    let swap = signature(&mut types, &[pointer, pointer], &[]);
    assert_eq!(
        RuntimeIntrinsic::bind("swap", swap, &types, LayoutPolicy::lp64()).unwrap(),
        RuntimeIntrinsic::Swap {
            value: record
        }
    );
    let other = types.reserve_record(RecordKind::Struct);
    types.define_record(other, [integer, integer]).unwrap();
    let other_pointer = types.pointer(other).unwrap();
    let mismatched = signature(&mut types, &[pointer, other_pointer], &[]);
    assert!(RuntimeIntrinsic::bind("swap", mismatched, &types, LayoutPolicy::lp64()).is_err());
    for value in [types.void(), types.meta_type()] {
        let pointer = types.pointer(value).unwrap();
        let signature = signature(&mut types, &[pointer, pointer], &[]);
        assert!(RuntimeIntrinsic::bind("swap", signature, &types, LayoutPolicy::lp64()).is_err());
    }
}

#[test]
fn atomics_preserve_exact_nominal_types_and_reject_aggregate_and_float_types() {
    let mut types = TypeRegistry::new();
    let s64 = types.scalar(ScalarType::Int(IntegerType::S64));
    let boolean = types.scalar(ScalarType::Bool);
    let distinct = types.reserve_distinct(DistinctKind::Distinct);
    types.define_distinct(distinct, s64).unwrap();
    for value in [s64, boolean, distinct] {
        let pointer = types.pointer(value).unwrap();
        let ty = signature(&mut types, &[pointer, value, value], &[boolean, value]);
        assert_eq!(
            RuntimeIntrinsic::bind("compare_and_swap", ty, &types, LayoutPolicy::lp64()).unwrap(),
            RuntimeIntrinsic::CompareAndSwap {
                value
            }
        );
    }
    let pointer = types.pointer(distinct).unwrap();
    let wrong_nominal = signature(&mut types, &[pointer, s64, s64], &[boolean, s64]);
    assert!(
        RuntimeIntrinsic::bind(
            "compare_and_swap",
            wrong_nominal,
            &types,
            LayoutPolicy::lp64()
        )
        .is_err()
    );
    let record = types.reserve_record(RecordKind::Struct);
    types.define_record(record, [s64]).unwrap();
    let float = types.float(FloatType::F64);
    for value in [record, float] {
        let pointer = types.pointer(value).unwrap();
        let ty = signature(&mut types, &[pointer, value, value], &[boolean, value]);
        assert!(
            RuntimeIntrinsic::bind("compare_and_swap", ty, &types, LayoutPolicy::lp64()).is_err()
        );
    }
}

#[test]
fn foreign_contextful_and_variadic_signatures_cannot_masquerade_as_intrinsics() {
    let mut types = TypeRegistry::new();
    for (convention, context, variadic) in [
        (CallingConvention::C, ContextMode::None, Variadic::None),
        (
            CallingConvention::Jai,
            ContextMode::Implicit,
            Variadic::None,
        ),
        (
            CallingConvention::C,
            ContextMode::None,
            Variadic::C {
                fixed_parameters: 0,
            },
        ),
    ] {
        let ty = types
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention,
                context,
                variadic,
            })
            .unwrap();
        assert!(
            RuntimeIntrinsic::bind("llvm.debugtrap", ty, &types, LayoutPolicy::lp64()).is_err()
        );
    }
}
