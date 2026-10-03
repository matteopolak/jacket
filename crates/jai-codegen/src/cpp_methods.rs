//! C++ vtable methods use explicit receiver-first signatures and target C ABI carriers.
use crate::abi::{Error, Platform};
use jai_types::{CallingConvention, ContextMode, ProcedureType, TypeId, TypeKind, Types, Variadic};

pub(crate) fn validate(
    types: &Types,
    signature: TypeId,
    procedure: &ProcedureType,
    platform: Platform,
) -> Result<(), Error> {
    if procedure.convention != CallingConvention::CppMethod {
        return Ok(());
    }
    if !platform.has_proven_cpp_method_abi() {
        return Err(Error::UnsupportedTarget(format!(
            "{platform:?} C++ method ABI"
        )));
    }
    if procedure.context != ContextMode::None
        || procedure.variadic != Variadic::None
        || procedure.results.len() > 1
        || !matches!(
            procedure.parameters.first().map(|ty| types.kind(*ty)),
            Some(Ok(TypeKind::Pointer(_)))
        )
    {
        return Err(Error::InvalidSignature(signature));
    }
    for &ty in procedure.parameters.iter().chain(procedure.results.iter()) {
        if !matches!(
            types.kind(ty)?,
            TypeKind::Bool
                | TypeKind::Integer(_)
                | TypeKind::Float(_)
                | TypeKind::Pointer(_)
                | TypeKind::Procedure(_)
                | TypeKind::Enum(_)
        ) {
            return Err(Error::UnsupportedType(ty));
        }
    }
    Ok(())
}
