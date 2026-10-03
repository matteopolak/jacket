//! Preserve the source pack position while distinguishing C ellipsis from Jai slices.
use super::*;
use jai_types::{CallingConvention, TypeRegistry, Variadic};

pub(crate) fn source_variadic(
    source: &[syntax::Parameter],
    parameters: &[ParameterSignature],
    convention: CallingConvention,
) -> crate::overloads::CandidateVariadic {
    let Some(pack) = source.iter().find(|parameter| parameter.variadic) else {
        return crate::overloads::CandidateVariadic::None;
    };
    if convention == CallingConvention::C {
        return crate::overloads::CandidateVariadic::C {
            fixed_parameters: parameters.len(),
        };
    }
    parameters
        .iter()
        .position(|parameter| parameter.name == pack.name)
        .map_or(crate::overloads::CandidateVariadic::None, |parameter| {
            crate::overloads::CandidateVariadic::Jai { parameter }
        })
}

/// A sole source `void` result denotes an empty runtime result list.
pub(crate) fn normalize_results<T>(
    results: &mut Vec<T>,
    types: &TypeRegistry,
    result_type: impl FnOnce(&T) -> TypeId,
) {
    if results.len() == 1 && result_type(&results[0]) == types.void() {
        results.clear();
    }
}

pub(crate) fn normalize_variadic(
    source: &[syntax::Parameter],
    parameters: &mut Vec<ParameterSignature>,
    convention: CallingConvention,
    types: &mut TypeRegistry,
    span: Span,
) -> Result<Variadic, Diagnostic> {
    let Some(index) = source.iter().position(|parameter| parameter.variadic) else {
        return Ok(Variadic::None);
    };
    if source.iter().filter(|parameter| parameter.variadic).count() != 1 {
        return Err(Diagnostic::new(
            span,
            "a procedure can have only one variadic parameter",
        ));
    }
    match convention {
        CallingConvention::C => {
            if index + 1 != source.len() {
                return Err(Diagnostic::new(
                    source[index].span,
                    "C variadic parameter must be last",
                ));
            }
            // The C source pack is a binding description rather than a runtime slot.
            // Callers may omit it during type resolution (e.g. the special Any pack).
            if let Some(index) = parameters
                .iter()
                .position(|parameter| parameter.name == source[index].name)
            {
                parameters.remove(index);
            }
            if source[index].evaluation == syntax::ParameterEvaluation::Discard {
                return Ok(Variadic::None);
            }
            Ok(Variadic::C {
                fixed_parameters: parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .count(),
            })
        }
        CallingConvention::Jai => {
            let retained = parameters
                .iter()
                .position(|parameter| parameter.name == source[index].name)
                .ok_or_else(|| {
                    Diagnostic::new(span, "variadic signature lost its source parameter")
                })?;
            let runtime = parameters[..retained]
                .iter()
                .filter(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Evaluate)
                .count();
            let parameter = &mut parameters[retained];
            let element = parameter.ty;
            parameter.ty = types
                .slice(element)
                .map_err(|error| Diagnostic::new(source[index].span, error.to_string()))?;
            if source.iter().skip(index + 1).any(|parameter| {
                matches!(
                    parameter.binding,
                    syntax::ParameterBinding::Required(_)
                        | syntax::ParameterBinding::RequiredType(_)
                )
            }) {
                return Err(Diagnostic::new(
                    span,
                    "parameters after a Jai variadic pack require defaults and named arguments",
                ));
            }
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                return Ok(Variadic::None);
            }
            Ok(Variadic::Jai {
                parameter: runtime,
                element,
            })
        }
        CallingConvention::CppMethod => Err(Diagnostic::new(
            span,
            "C++ method variadic signatures are not supported",
        )),
        CallingConvention::Stdcall => Err(Diagnostic::new(
            span,
            "stdcall procedures cannot be variadic",
        )),
    }
}
