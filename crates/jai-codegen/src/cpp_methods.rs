//! Explicit C++ receivers and Microsoft class-return carriers.
use crate::abi::{Error, Platform};
use jai_types::{
    CallingConvention, ContextMode, ForeignReturnAbi, ProcedureType, TypeId, TypeKind, Types,
    Variadic,
};

fn record_result(types: &Types, mut ty: TypeId) -> Result<bool, Error> {
    let root = ty;
    let mut seen = std::collections::HashSet::new();
    loop {
        if seen.len() == 65_536 {
            return Err(Error::ClassificationLimit {
                ty: root,
                limit: 65_536,
            });
        }
        if !seen.insert(ty) {
            return Err(Error::UnsupportedType(root));
        }
        match types.kind(ty)? {
            TypeKind::Distinct(id) => ty = types.distinct(*id)?.representation,
            TypeKind::Record(_) => return Ok(true),
            _ => return Ok(false),
        }
    }
}

pub(crate) fn indirect_result(
    types: &Types,
    procedure: &ProcedureType,
    platform: Platform,
) -> Result<bool, Error> {
    let microsoft = matches!(platform, Platform::WindowsX86_64 | Platform::WindowsArm64);
    if procedure.return_abi == ForeignReturnAbi::CppNonPod {
        if !microsoft {
            return Err(Error::UnsupportedTarget(format!(
                "{platform:?} #cpp_return_type_is_non_pod (Microsoft C++ result ABI required)"
            )));
        }
        return Ok(true);
    }
    Ok(microsoft
        && procedure.convention == CallingConvention::CppMethod
        && procedure
            .results
            .first()
            .map(|ty| record_result(types, *ty))
            .transpose()?
            .unwrap_or(false))
}

/// A Windows target is not itself a Microsoft C++ ABI proof (e.g. MinGW).
/// The classified Platform is an explicit ABI contract; actual modules must
/// additionally prove their selected LLVM environment before using it.
pub(crate) fn validate_triple(
    types: &Types,
    procedure: &ProcedureType,
    platform: Platform,
    triple: &str,
) -> Result<(), Error> {
    let microsoft_result = procedure.return_abi == ForeignReturnAbi::CppNonPod
        || (procedure.convention == CallingConvention::CppMethod
            && matches!(platform, Platform::WindowsX86_64 | Platform::WindowsArm64)
            && procedure
                .results
                .first()
                .map(|ty| record_result(types, *ty))
                .transpose()?
                .unwrap_or(false));
    if microsoft_result
        && (Platform::from_triple(triple)? != platform
            || !triple.contains("windows")
            || !triple.split('-').any(|part| {
                part == "msvc"
                    || part.strip_prefix("msvc").is_some_and(|version| {
                        !version.is_empty()
                            && version
                                .bytes()
                                .all(|byte| byte.is_ascii_digit() || byte == b'.')
                    })
            }))
    {
        return Err(Error::UnsupportedTarget(format!(
            "{triple} Microsoft C++ class result ABI requires an explicit msvc environment"
        )));
    }
    Ok(())
}

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
    for &ty in &procedure.parameters {
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
    for &ty in &procedure.results {
        if !record_result(types, ty)?
            && !matches!(
                types.kind(ty)?,
                TypeKind::Bool
                    | TypeKind::Integer(_)
                    | TypeKind::Float(_)
                    | TypeKind::Pointer(_)
                    | TypeKind::Procedure(_)
                    | TypeKind::Enum(_)
            )
        {
            return Err(Error::UnsupportedType(ty));
        }
    }
    Ok(())
}
