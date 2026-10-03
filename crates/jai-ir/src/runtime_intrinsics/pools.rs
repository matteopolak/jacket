//! Exact source-visible storage contracts for the supplied pool declarations.
use super::*;
use jai_types::RecordKind;

pub(super) fn identity(operation: RuntimeIntrinsic) -> Option<(TypeId, bool)> {
    Some(match operation {
        RuntimeIntrinsic::PoolGet {
            pool,
        }
        | RuntimeIntrinsic::PoolReset {
            pool,
        }
        | RuntimeIntrinsic::PoolRelease {
            pool,
        } => (pool, false),
        RuntimeIntrinsic::FlatPoolGet {
            pool,
        }
        | RuntimeIntrinsic::FlatPoolReset {
            pool,
        }
        | RuntimeIntrinsic::FlatPoolFinish {
            pool,
        } => (pool, true),
        _ => return None,
    })
}

pub(super) fn bind(
    name: IntrinsicName,
    parameters: &[TypeId],
    types: &dyn TypeView,
) -> Result<RuntimeIntrinsic, RuntimeIntrinsicError> {
    let Some(pointer) = parameters.first() else {
        return Err(RuntimeIntrinsicError::Signature(
            "pool operation requires its nominal self pointer",
        ));
    };
    let TypeKind::Pointer(pool) = types.kind(*pointer)? else {
        return Err(RuntimeIntrinsicError::Signature(
            "pool operation requires its nominal self pointer",
        ));
    };
    let flat = types.record_storage_definition(*pool)?.fields.len() == 5;
    shape(types, *pool, flat)?;
    Ok(match (name, flat) {
        (IntrinsicName::PoolGet, false) => RuntimeIntrinsic::PoolGet {
            pool: *pool,
        },
        (IntrinsicName::PoolGet, true) => RuntimeIntrinsic::FlatPoolGet {
            pool: *pool,
        },
        (IntrinsicName::PoolReset, false) => RuntimeIntrinsic::PoolReset {
            pool: *pool,
        },
        (IntrinsicName::PoolReset, true) => RuntimeIntrinsic::FlatPoolReset {
            pool: *pool,
        },
        (IntrinsicName::PoolRelease, false) => RuntimeIntrinsic::PoolRelease {
            pool: *pool,
        },
        (IntrinsicName::FlatPoolFinish, true) => RuntimeIntrinsic::FlatPoolFinish {
            pool: *pool,
        },
        _ => {
            return Err(RuntimeIntrinsicError::Signature(
                "release belongs to Pool; fini belongs to Flat_Pool",
            ));
        }
    })
}

pub(super) fn signature(
    operation: RuntimeIntrinsic,
    pool: TypeId,
    parameters: &[TypeId],
    results: &[TypeId],
    types: &dyn TypeView,
) -> Result<bool, RuntimeIntrinsicError> {
    let pointer = |ty| matches!(types.kind(ty), Ok(TypeKind::Pointer(pointee)) if *pointee == pool);
    Ok(match operation {
        RuntimeIntrinsic::PoolGet {
            ..
        }
        | RuntimeIntrinsic::FlatPoolGet {
            ..
        } => {
            matches!(parameters, [this, size] if pointer(*this)
                && *size == types.scalar(ScalarType::Int(IntegerType::S64)))
                && matches!(results, [result] if matches!(types.kind(*result)?, TypeKind::Pointer(pointee) if matches!(types.kind(*pointee)?, TypeKind::Void)))
        }
        RuntimeIntrinsic::FlatPoolReset {
            ..
        } => {
            matches!(parameters, [this, overwrite]
            if pointer(*this) && *overwrite == types.scalar(ScalarType::Bool))
                && results.is_empty()
        }
        RuntimeIntrinsic::PoolReset {
            ..
        }
        | RuntimeIntrinsic::PoolRelease {
            ..
        }
        | RuntimeIntrinsic::FlatPoolFinish {
            ..
        } => matches!(parameters, [this] if pointer(*this)) && results.is_empty(),
        _ => false,
    })
}

pub(super) fn storage(
    types: &dyn TypeView,
    pool: TypeId,
    flat: bool,
    policy: LayoutPolicy,
) -> Result<(), RuntimeIntrinsicError> {
    shape(types, pool, flat)?;
    LayoutEngine::new(types, policy)
        .layout(pool)
        .map_err(layout_error)?;
    Ok(())
}

pub(super) fn shape(
    types: &dyn TypeView,
    pool: TypeId,
    flat: bool,
) -> Result<(), RuntimeIntrinsicError> {
    if !matches!(types.kind(pool)?, TypeKind::Record(_)) {
        return Err(RuntimeIntrinsicError::Signature(
            "pool self type must be a nominal struct",
        ));
    }
    let definition = types.record_storage_definition(pool)?;
    let fields = definition.fields.as_ref();
    let integer = |ty| matches!(types.kind(ty), Ok(TypeKind::Integer(IntegerType::S64)));
    let void_pointer = |ty| matches!(types.kind(ty), Ok(TypeKind::Pointer(pointee)) if matches!(types.kind(*pointee), Ok(TypeKind::Void)));
    let valid = definition.kind == RecordKind::Struct
        && fields.len()
            == if flat {
                5
            } else {
                4
            }
        && integer(fields[0])
        && integer(fields[1])
        && void_pointer(fields[2])
        && integer(fields[3])
        && (!flat || integer(fields[4]));
    if valid {
        Ok(())
    } else {
        Err(RuntimeIntrinsicError::Signature(if flat {
            "Flat_Pool storage requires (s64, s64, *void, s64, s64) fields"
        } else {
            "Pool storage requires (s64, s64, *void, s64) fields"
        }))
    }
}
