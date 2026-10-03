//! Result Type inputs retain original result patterns and source argument indices.
use super::*;

fn actual(types: &dyn TypeView, argument: &Argument) -> Result<TypeId, Diagnostic> {
    let Some(ConstantArgument::Value(BakedValue::Type(ty))) = &argument.info.constant else {
        return Err(Diagnostic::new(
            argument.span,
            "result input requires a compile-time Type value",
        ));
    };
    if argument.info.ty != ArgumentType::Known(types.meta_type()) {
        return Err(Diagnostic::new(
            argument.span,
            "result input requires the actual canonical Type value",
        ));
    }
    types
        .kind(*ty)
        .map_err(|error| Diagnostic::new(argument.span, error.to_string()))?;
    Ok(*ty)
}
pub(super) fn bind<Origin>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    bindings: &[ResultTypeArgument],
    substitution: &mut Substitution,
) -> Result<(), Diagnostic> {
    // Signature order makes canonical substitutions independent of named call order.
    for (ordinal, parameter) in candidate.result_type_parameters.iter().enumerate() {
        let binding = bindings
            .iter()
            .find(|binding| binding.parameter == ordinal)
            .ok_or_else(|| {
                Diagnostic::new(
                    parameter.span,
                    "missing required named result Type argument",
                )
            })?;
        let argument = &arguments[binding.argument];
        let ty = actual(types, argument)?;
        infer(
            types,
            nominals,
            &parameter.pattern,
            &ArgumentType::Known(ty),
            substitution,
            false,
            argument.span,
        )?;
    }
    Ok(())
}
pub(super) fn check<Origin>(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    candidate: &Candidate<Origin>,
    arguments: &[Argument],
    bindings: &[ResultTypeArgument],
    substitution: &Substitution,
) -> Result<(), Diagnostic> {
    for binding in bindings {
        let parameter = candidate
            .result_type_parameters
            .get(binding.parameter)
            .ok_or_else(|| {
                Diagnostic::new(
                    Span::default(),
                    "result Type binding has no original source parameter",
                )
            })?;
        let argument = &arguments[binding.argument];
        let ty = actual(types, argument)?;
        if substitution.ty(parameter.name) != Some(ty) {
            return Err(Diagnostic::new(
                argument.span,
                "accepted result Type binding differs from its original argument",
            ));
        }
        compatible(
            types,
            nominals,
            &parameter.pattern,
            &ArgumentType::Known(ty),
            substitution,
            false,
            argument.span,
        )?;
    }
    Ok(())
}

pub(crate) fn collect(
    pattern: &TypePattern,
    result: usize,
    span: Span,
    parameters: &mut Vec<ResultTypeParameter>,
) -> Result<(), Diagnostic> {
    let mut pending = vec![pattern];
    while let Some(pattern) = pending.pop() {
        match pattern {
            TypePattern::Infer(name) => parameters.push(ResultTypeParameter {
                name: *name,
                result,
                span,
                pattern: pattern.clone(),
            }),
            TypePattern::Restricted {
                ty, ..
            } if matches!(ty.as_ref(), TypePattern::Infer(_)) => {
                let TypePattern::Infer(name) = ty.as_ref() else {
                    unreachable!()
                };
                parameters.push(ResultTypeParameter {
                    name: *name,
                    result,
                    span,
                    pattern: pattern.clone(),
                });
            }
            TypePattern::Restricted {
                ty, ..
            }
            | TypePattern::Pointer(ty)
            | TypePattern::Slice(ty)
            | TypePattern::DynamicArray(ty) => pending.push(ty),
            TypePattern::Procedure(procedure) => {
                pending.extend(procedure.results.iter().rev());
                pending.extend(procedure.parameters.iter().rev());
            }
            TypePattern::FixedArray {
                element,
                count,
            } => {
                if matches!(count, CountPattern::Infer(_)) {
                    return Err(Diagnostic::new(
                        span,
                        "result-only array count introduction requires an explicit baked source parameter",
                    ));
                }
                pending.push(element);
            }
            TypePattern::NominalApplication {
                arguments, ..
            } => {
                for argument in arguments.iter().rev() {
                    match &argument.kind {
                        NominalArgumentKind::Type(ty) => pending.push(ty),
                        NominalArgumentKind::InferValue(_) => {
                            return Err(Diagnostic::new(
                                span,
                                "result-only record value introduction requires an explicit baked source parameter",
                            ));
                        }
                        _ => {}
                    }
                }
            }
            TypePattern::Concrete(_) | TypePattern::Variable(_) => {}
        }
    }
    Ok(())
}
