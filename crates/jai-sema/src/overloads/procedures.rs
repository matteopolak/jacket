//! Callback patterns preserve all ABI fields and never widen through a signature.
use super::*;
use jai_types::{CallingConvention, ProcedureType, Variadic};

pub(crate) fn procedure_pattern(
    source: &jai_syntax::ProcedureTypeSyntax,
    span: Span,
    mut resolve: impl FnMut(&jai_syntax::TypeSyntax, Span) -> Result<TypePattern, Diagnostic>,
) -> Result<TypePattern, Diagnostic> {
    let mut parameters = Vec::new();
    let mut variadic = CandidateVariadic::None;
    for parameter in &source.parameters {
        let ty = resolve(&parameter.ty, parameter.span)?;
        if parameter.evaluation == jai_syntax::ParameterEvaluation::Discard {
            continue;
        }
        if parameter.variadic {
            if variadic != CandidateVariadic::None {
                return Err(Diagnostic::new(
                    parameter.span,
                    "callback type has more than one variadic parameter",
                ));
            }
            if source.convention == CallingConvention::C {
                if !matches!(
                    parameter.ty,
                    jai_syntax::TypeSyntax::Builtin(jai_syntax::BuiltinType::Any)
                ) {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "C callback variadic pattern requires Any",
                    ));
                }
                variadic = CandidateVariadic::C {
                    fixed_parameters: parameters.len(),
                };
                continue;
            }
            variadic = CandidateVariadic::Jai {
                parameter: parameters.len(),
            };
            parameters.push(TypePattern::Slice(Box::new(ty)));
        } else {
            if matches!(variadic, CandidateVariadic::C { .. }) {
                return Err(Diagnostic::new(
                    parameter.span,
                    "C callback variadic parameter must be last",
                ));
            }
            parameters.push(ty);
        }
    }
    let mut results = source
        .results
        .iter()
        .map(|result| resolve(&result.ty, result.span))
        .collect::<Result<Vec<_>, _>>()?;
    if source.results.len() == 1
        && matches!(
            source.results[0].ty,
            jai_syntax::TypeSyntax::Builtin(jai_syntax::BuiltinType::Void)
        )
    {
        results.clear();
    }
    if parameters.is_empty()
        && source.parameters.iter().any(|parameter| parameter.variadic)
        && !matches!(variadic, CandidateVariadic::C { .. })
    {
        return Err(Diagnostic::new(span, "invalid callback variadic pattern"));
    }
    Ok(TypePattern::Procedure(Box::new(ProcedurePattern {
        parameters,
        results,
        convention: source.convention,
        context: source.context,
        variadic,
    })))
}

fn shape(pattern: &ProcedurePattern, source: &ProcedureType, span: Span) -> Result<(), Diagnostic> {
    let variadic = match source.variadic {
        Variadic::None => CandidateVariadic::None,
        Variadic::C { fixed_parameters } => CandidateVariadic::C { fixed_parameters },
        Variadic::Jai { parameter, .. } => CandidateVariadic::Jai { parameter },
    };
    if source.convention != pattern.convention
        || source.context != pattern.context
        || variadic != pattern.variadic
        || source.parameters.len() != pattern.parameters.len()
        || (source.results.len() != pattern.results.len()
            && !(source.results.is_empty() && void_result_pattern(pattern)))
    {
        return Err(Diagnostic::new(
            span,
            "callback argument has a different parameter, result, ABI, context, or variadic signature",
        ));
    }
    Ok(())
}
fn void_result_pattern(pattern: &ProcedurePattern) -> bool {
    matches!(
        pattern.results.as_slice(),
        [TypePattern::Infer(_) | TypePattern::Variable(_) | TypePattern::Concrete(_)]
    )
}

fn result_arguments(types: &dyn TypeView, source: &ProcedureType) -> Vec<TypeId> {
    if source.results.is_empty() {
        vec![types.lookup(&TypeKind::Void).expect("canonical void type")]
    } else {
        source.results.to_vec()
    }
}
pub(super) fn infer(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &ProcedurePattern,
    source: TypeId,
    substitution: &mut Substitution,
    span: Span,
) -> Result<(), Diagnostic> {
    let source = types
        .procedure_definition(source)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    shape(pattern, source, span)?;
    for (pattern, &ty) in pattern
        .parameters
        .iter()
        .zip(source.parameters.iter())
        .chain(
            pattern
                .results
                .iter()
                .zip(result_arguments(types, source).iter()),
        )
    {
        super::infer(
            types,
            nominals,
            pattern,
            &ArgumentType::Known(ty),
            substitution,
            false,
            span,
        )?;
    }
    Ok(())
}

/// A contextual body can infer results only after its parameter context is known.
pub(super) fn infer_contextual(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &ProcedurePattern,
    signatures: &[TypeId],
    substitution: &mut Substitution,
    span: Span,
) -> Result<(), Diagnostic> {
    if signatures.is_empty()
        || pattern
            .parameters
            .iter()
            .any(|item| ensure_bound(item, substitution, span).is_err())
    {
        return Ok(());
    }
    let mut inferred = None;
    for &signature in signatures {
        let mut draft = substitution.clone();
        if infer(types, nominals, pattern, signature, &mut draft, span).is_err()
            || compatible(
                types,
                nominals,
                pattern,
                &ArgumentType::Known(signature),
                &draft,
                span,
            )
            .is_err()
        {
            continue;
        }
        if inferred.as_ref().is_some_and(|previous| previous != &draft) {
            return Err(Diagnostic::new(
                span,
                "source lambda has ambiguous callback result inference",
            ));
        }
        inferred = Some(draft);
    }
    if let Some(inferred) = inferred {
        *substitution = inferred;
    }
    Ok(())
}
pub(super) fn compatible(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &ProcedurePattern,
    source: &ArgumentType,
    substitution: &Substitution,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    if let ArgumentType::ContextualProcedure {
        compatible_signatures,
    } = source
    {
        for &signature in compatible_signatures {
            if compatible(
                types,
                nominals,
                pattern,
                &ArgumentType::Known(signature),
                substitution,
                span,
            )
            .is_ok()
            {
                return Ok(ConversionRank::Literal);
            }
        }
        return Err(Diagnostic::new(
            span,
            "source lambda does not match this callback pattern",
        ));
    }
    if matches!(source, ArgumentType::Null) {
        for item in pattern.parameters.iter().chain(&pattern.results) {
            ensure_bound(item, substitution, span)?;
        }
        return Ok(ConversionRank::Literal);
    }
    let ArgumentType::Known(source) = source else {
        return Err(Diagnostic::new(
            span,
            "callback pattern requires a typed procedure argument",
        ));
    };
    let source = types
        .procedure_definition(*source)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    shape(pattern, source, span)?;
    for (pattern, &ty) in pattern
        .parameters
        .iter()
        .zip(source.parameters.iter())
        .chain(
            pattern
                .results
                .iter()
                .zip(result_arguments(types, source).iter()),
        )
    {
        super::compatible(
            types,
            nominals,
            pattern,
            &ArgumentType::Known(ty),
            substitution,
            false,
            span,
        )?;
    }
    Ok(ConversionRank::Exact)
}

pub(super) fn signature(
    types: &dyn TypeView,
    pattern: &ProcedurePattern,
    substitution: &Substitution,
    span: Span,
) -> Result<ProcedureType, Diagnostic> {
    let parameters = pattern
        .parameters
        .iter()
        .map(|item| casts::target_type(types, item, substitution, span))
        .collect::<Result<Vec<_>, _>>()?;
    let mut results = pattern
        .results
        .iter()
        .map(|item| casts::target_type(types, item, substitution, span))
        .collect::<Result<Vec<_>, _>>()?;
    if results.len() == 1 && matches!(types.kind(results[0]), Ok(TypeKind::Void)) {
        results.clear();
    }
    let variadic = match pattern.variadic {
        CandidateVariadic::None => Variadic::None,
        CandidateVariadic::C { fixed_parameters } => Variadic::C { fixed_parameters },
        CandidateVariadic::Jai { parameter } => {
            let element = parameters
                .get(parameter)
                .and_then(|ty| match types.kind(*ty) {
                    Ok(TypeKind::Slice(element)) => Some(*element),
                    _ => None,
                })
                .ok_or_else(|| {
                    Diagnostic::new(span, "Jai callback pack has no slice element type")
                })?;
            Variadic::Jai { parameter, element }
        }
    };
    Ok(ProcedureType {
        parameters: parameters.into_boxed_slice(),
        results: results.into_boxed_slice(),
        convention: pattern.convention,
        context: pattern.context,
        variadic,
    })
}

pub(crate) fn infer_lambda_result(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &ProcedurePattern,
    result: Option<&ArgumentInfo>,
    substitution: &mut Substitution,
    span: Span,
) -> Result<(), Diagnostic> {
    let [result_pattern] = pattern.results.as_slice() else {
        return if pattern.results.is_empty() {
            Ok(())
        } else {
            Err(Diagnostic::new(
                span,
                "short lambda cannot infer a multi-result callback",
            ))
        };
    };
    let void;
    let result = if let Some(result) = result {
        result
    } else {
        void = ArgumentInfo::typed(types.lookup(&TypeKind::Void).expect("canonical void type"));
        &void
    };
    super::infer(
        types,
        nominals,
        result_pattern,
        &result.ty,
        substitution,
        true,
        span,
    )?;
    super::compatible(
        types,
        nominals,
        result_pattern,
        &result.ty,
        substitution,
        true,
        span,
    )?;
    Ok(())
}
