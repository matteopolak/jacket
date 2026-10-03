//! Closed runtime operations supplied by the independent implementation.
//!
//! Source spelling selects a candidate only for a declaration explicitly marked
//! `#intrinsic`; a checked signature binds its identity before either backend runs.
use jai_types::{
    CallingConvention, ContextMode, IntegerType, LayoutEngine, LayoutPolicy, ScalarType, TypeError,
    TypeId, TypeKind, TypeView, Variadic,
};
use std::fmt;

mod pools;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeIntrinsic {
    MemoryCopy,
    MemoryCopyReturningDestination,
    MemoryCompare,
    MemorySet,
    MemorySetReturningDestination,
    Swap { value: TypeId },
    CompareAndSwap { value: TypeId },
    PoolGet { pool: TypeId },
    PoolReset { pool: TypeId },
    PoolRelease { pool: TypeId },
    FlatPoolGet { pool: TypeId },
    FlatPoolReset { pool: TypeId },
    FlatPoolFinish { pool: TypeId },
    DebugTrap,
}

enum IntrinsicName {
    MemoryCopy,
    MemoryCompare,
    MemorySet,
    Swap,
    CompareAndSwap,
    PoolGet,
    PoolReset,
    PoolRelease,
    FlatPoolFinish,
    DebugTrap,
}
impl IntrinsicName {
    fn parse(name: &str) -> Result<Self, RuntimeIntrinsicError> {
        Ok(match name {
            "memcpy" => Self::MemoryCopy,
            "memcmp" => Self::MemoryCompare,
            "memset" => Self::MemorySet,
            "swap" => Self::Swap,
            "compare_and_swap" => Self::CompareAndSwap,
            "get" => Self::PoolGet,
            "reset" => Self::PoolReset,
            "release" => Self::PoolRelease,
            "fini" => Self::FlatPoolFinish,
            "llvm.debugtrap" => Self::DebugTrap,
            _ => return Err(RuntimeIntrinsicError::Unknown(name.to_owned())),
        })
    }
}

#[derive(Debug)]
pub enum RuntimeIntrinsicError {
    Unknown(String),
    Type(TypeError),
    Signature(&'static str),
    Layout(String),
}
impl From<TypeError> for RuntimeIntrinsicError {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}
impl fmt::Display for RuntimeIntrinsicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(name) => write!(f, "unsupported #intrinsic `{name}`"),
            Self::Type(error) => error.fmt(f),
            Self::Signature(reason) => write!(f, "invalid runtime intrinsic signature: {reason}"),
            Self::Layout(reason) => write!(f, "invalid runtime intrinsic target layout: {reason}"),
        }
    }
}
impl std::error::Error for RuntimeIntrinsicError {}

impl RuntimeIntrinsic {
    /// Validate pool storage without replacing its nominal self type.
    pub fn validate_pool_storage(
        types: &dyn TypeView,
        pool: TypeId,
        flat: bool,
        policy: LayoutPolicy,
    ) -> Result<(), RuntimeIntrinsicError> {
        pools::storage(types, pool, flat, policy)
    }
    /// Validate a marked generic declaration before its concrete signature exists.
    pub fn validate_name(name: &str) -> Result<(), RuntimeIntrinsicError> {
        IntrinsicName::parse(name).map(|_| ())
    }
    /// Bind only explicitly marked declarations. Ordinary same-named calls remain ordinary calls.
    pub fn bind(
        name: &str,
        signature: TypeId,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<Self, RuntimeIntrinsicError> {
        let name = IntrinsicName::parse(name)?;
        let declaration = types.procedure_definition(signature)?;
        let returns_destination = matches!(
            (declaration.parameters.first(), declaration.results.as_ref()),
            (Some(destination), [result]) if destination == result
                && matches!(types.kind(*result)?, TypeKind::Pointer(pointee)
                    if matches!(types.kind(*pointee)?, TypeKind::Void))
        );
        let operation = match name {
            IntrinsicName::MemoryCopy if returns_destination => {
                Self::MemoryCopyReturningDestination
            }
            IntrinsicName::MemoryCopy => Self::MemoryCopy,
            IntrinsicName::MemoryCompare => Self::MemoryCompare,
            IntrinsicName::MemorySet if returns_destination => Self::MemorySetReturningDestination,
            IntrinsicName::MemorySet => Self::MemorySet,
            IntrinsicName::Swap => {
                let Some(pointer) = declaration.parameters.first() else {
                    return Err(RuntimeIntrinsicError::Signature(
                        "swap requires (*T, *T) -> void",
                    ));
                };
                let TypeKind::Pointer(value) = types.kind(*pointer)? else {
                    return Err(RuntimeIntrinsicError::Signature(
                        "swap requires (*T, *T) -> void",
                    ));
                };
                Self::Swap { value: *value }
            }
            IntrinsicName::CompareAndSwap => {
                let TypeKind::Procedure(id) = types.kind(signature)? else {
                    return Err(RuntimeIntrinsicError::Signature(
                        "expected a procedure type",
                    ));
                };
                let signature = types.procedure_type(*id)?;
                let Some(value) = signature.parameters.get(1) else {
                    return Err(RuntimeIntrinsicError::Signature(
                        "compare_and_swap requires (*T, T, T) -> (bool, T)",
                    ));
                };
                Self::CompareAndSwap { value: *value }
            }
            IntrinsicName::DebugTrap => Self::DebugTrap,
            IntrinsicName::PoolGet
            | IntrinsicName::PoolReset
            | IntrinsicName::PoolRelease
            | IntrinsicName::FlatPoolFinish => pools::bind(name, &declaration.parameters, types)?,
        };
        operation.validate_signature(signature, types, policy)?;
        Ok(operation)
    }

    /// Recheck a staged enum against the frozen signature and the chosen target.
    pub fn validate_signature(
        self,
        signature: TypeId,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<(), RuntimeIntrinsicError> {
        self.validate_signature_shape(signature, types)?;
        if let Self::CompareAndSwap { value } = self {
            atomic_scalar(types, policy, value)?;
        } else if let Self::Swap { value } = self {
            LayoutEngine::new(types, policy)
                .layout(value)
                .map_err(layout_error)?;
        } else if let Some((pool, flat)) = pools::identity(self) {
            pools::storage(types, pool, flat, policy)?;
        }
        Ok(())
    }

    /// The checked IR validates the declaration without assuming a host target.
    /// Execution and native emission separately validate target-specific storage.
    pub fn validate_signature_shape(
        self,
        signature: TypeId,
        types: &dyn TypeView,
    ) -> Result<(), RuntimeIntrinsicError> {
        let TypeKind::Procedure(id) = types.kind(signature)? else {
            return Err(RuntimeIntrinsicError::Signature(
                "expected a procedure type",
            ));
        };
        let signature = types.procedure_type(*id)?;
        if signature.convention != CallingConvention::Jai
            || signature.context != ContextMode::None
            || signature.variadic != Variadic::None
        {
            return Err(RuntimeIntrinsicError::Signature(
                "requires a fixed Jai procedure without implicit context",
            ));
        }
        let integer = |ty, expected| matches!(types.kind(ty), Ok(TypeKind::Integer(actual)) if *actual == expected);
        let void_pointer = |ty| matches!(types.kind(ty), Ok(TypeKind::Pointer(pointee)) if matches!(types.kind(*pointee), Ok(TypeKind::Void)));
        let parameters = signature.parameters.as_ref();
        let results = signature.results.as_ref();
        let valid = match self {
            Self::MemoryCopy => {
                matches!(parameters, [destination, source, count]
                    if void_pointer(*destination) && void_pointer(*source)
                        && integer(*count, IntegerType::S64))
                    && results.is_empty()
            }
            Self::MemoryCopyReturningDestination => {
                matches!(parameters, [destination, source, count]
                    if void_pointer(*destination) && void_pointer(*source)
                        && integer(*count, IntegerType::S64))
                    && matches!(results, [result] if void_pointer(*result))
            }
            Self::MemoryCompare => {
                matches!(parameters, [left, right, count]
                    if void_pointer(*left) && void_pointer(*right)
                        && integer(*count, IntegerType::S64))
                    && matches!(results, [result] if integer(*result, IntegerType::S16))
            }
            Self::MemorySet => {
                matches!(parameters, [destination, byte, count]
                    if void_pointer(*destination) && integer(*byte, IntegerType::U8)
                        && integer(*count, IntegerType::S64))
                    && results.is_empty()
            }
            Self::MemorySetReturningDestination => {
                matches!(parameters, [destination, byte, count]
                    if void_pointer(*destination) && integer(*byte, IntegerType::S64)
                        && integer(*count, IntegerType::S64))
                    && matches!(results, [result] if void_pointer(*result))
            }
            Self::Swap { value } => {
                if matches!(
                    types.kind(value)?,
                    TypeKind::Void | TypeKind::Code | TypeKind::Type
                ) {
                    return Err(RuntimeIntrinsicError::Signature(
                        "swap requires runtime data storage",
                    ));
                }
                matches!(parameters, [left, right]
                    if matches!(types.kind(*left)?, TypeKind::Pointer(pointee) if *pointee == value)
                        && *left == *right)
                    && results.is_empty()
            }
            Self::CompareAndSwap { value } => {
                atomic_scalar_kind(types, value)?;
                matches!(parameters, [pointer, old, new]
                    if matches!(types.kind(*pointer), Ok(TypeKind::Pointer(pointee)) if *pointee == value)
                        && *old == value && *new == value)
                    && matches!(results, [success, old]
                        if *success == types.scalar(ScalarType::Bool) && *old == value)
            }
            Self::DebugTrap => parameters.is_empty() && results.is_empty(),
            Self::PoolGet { pool }
            | Self::FlatPoolGet { pool }
            | Self::PoolReset { pool }
            | Self::FlatPoolReset { pool }
            | Self::PoolRelease { pool }
            | Self::FlatPoolFinish { pool } => {
                let flat = pools::identity(self)
                    .ok_or(RuntimeIntrinsicError::Signature("missing pool identity"))?
                    .1;
                pools::shape(types, pool, flat)?;
                pools::signature(self, pool, parameters, results, types)?
            }
        };
        if valid {
            Ok(())
        } else {
            Err(RuntimeIntrinsicError::Signature(match self {
                Self::MemoryCopy => "memcpy requires (*void, *void, s64) -> void",
                Self::MemoryCopyReturningDestination => {
                    "destination-returning memcpy requires (*void, *void, s64) -> *void"
                }
                Self::MemoryCompare => "memcmp requires (*void, *void, s64) -> s16",
                Self::MemorySet => "memset requires (*void, u8, s64) -> void",
                Self::MemorySetReturningDestination => {
                    "destination-returning memset requires (*void, s64, s64) -> *void"
                }
                Self::Swap { .. } => {
                    "swap requires (*T, *T) -> void with identical nominal pointee types"
                }
                Self::CompareAndSwap { .. } => "compare_and_swap requires (*T, T, T) -> (bool, T)",
                Self::DebugTrap => "llvm.debugtrap requires () -> void",
                Self::PoolGet { .. } | Self::FlatPoolGet { .. } => {
                    "pool get requires (*Pool, s64) -> *void"
                }
                Self::PoolReset { .. } | Self::PoolRelease { .. } => {
                    "Pool reset/release requires (*Pool) -> void"
                }
                Self::FlatPoolReset { .. } => "Flat_Pool reset requires (*Flat_Pool, bool) -> void",
                Self::FlatPoolFinish { .. } => "Flat_Pool fini requires (*Flat_Pool) -> void",
            }))
        }
    }
}

/// Atomics.jai's documented scalar domain; nominal wrappers retain their identity.
pub fn atomic_scalar(
    types: &dyn TypeView,
    policy: LayoutPolicy,
    ty: TypeId,
) -> Result<(), RuntimeIntrinsicError> {
    atomic_scalar_kind(types, ty)?;
    let size = LayoutEngine::new(types, policy)
        .layout(ty)
        .map_err(layout_error)?
        .size;
    if !matches!(size, 1 | 2 | 4 | 8) {
        return Err(RuntimeIntrinsicError::Signature(
            "compare_and_swap requires a 1, 2, 4 or 8 byte scalar",
        ));
    }
    Ok(())
}

fn layout_error(error: jai_types::LayoutError) -> RuntimeIntrinsicError {
    match error {
        jai_types::LayoutError::Type(error) => RuntimeIntrinsicError::Type(error),
        error => RuntimeIntrinsicError::Layout(error.to_string()),
    }
}

fn atomic_scalar_kind(types: &dyn TypeView, mut ty: TypeId) -> Result<(), RuntimeIntrinsicError> {
    let mut depth = 0usize;
    loop {
        match types.kind(ty)? {
            TypeKind::Bool | TypeKind::Integer(_) | TypeKind::Pointer(_) => break,
            TypeKind::Enum(id) => {
                types.enumeration(*id)?;
                break;
            }
            TypeKind::Distinct(id) if depth < 128 => {
                ty = types.distinct(*id)?.representation;
                depth += 1;
            }
            _ => {
                return Err(RuntimeIntrinsicError::Signature(
                    "compare_and_swap supports bool, integer, enum, pointer and their nominal variants",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "runtime_intrinsics/tests.rs"]
mod tests;
