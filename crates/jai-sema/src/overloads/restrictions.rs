//! Restrictions validate a bound type without converting it to the restriction.
use super::*;

pub(super) fn validate(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    restriction: &TypeRestrictionPattern,
    actual: TypeId,
    substitution: &Substitution,
    span: Span,
) -> Result<(), Diagnostic> {
    match restriction {
        TypeRestrictionPattern::Nominal(pattern) => match pattern.as_ref() {
            TypePattern::Concrete(required) => {
                if !nominals.nominal_ancestor(actual, *required, span)? {
                    return Err(Diagnostic::new(
                        span,
                        "type does not satisfy the nominal restriction",
                    ));
                }
            }
            TypePattern::Infer(name) | TypePattern::Variable(name) => {
                let required = substitution.ty(*name).ok_or_else(|| {
                    Diagnostic::new(span, "nominal restriction type is not bound")
                })?;
                if !nominals.nominal_ancestor(actual, required, span)? {
                    return Err(Diagnostic::new(
                        span,
                        "type does not satisfy the nominal restriction",
                    ));
                }
            }
            _ => {
                compatible(
                    types,
                    nominals,
                    pattern,
                    &ArgumentType::Known(actual),
                    substitution,
                    false,
                    span,
                )?;
            }
        },
        TypeRestrictionPattern::Interface(required) => {
            if !matches!(types.kind(*required), Ok(TypeKind::Record(_))) {
                return Err(Diagnostic::new(
                    span,
                    "interface restriction requires a record type",
                ));
            }
            for field in nominals.record_fields(*required, span)? {
                let name = field.name.ok_or_else(|| {
                    Diagnostic::new(span, "interface restriction requires named members")
                })?;
                let member = nominals
                    .interface_member(actual, name, span)?
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            format!(
                                "interface requires member '{}'",
                                nominals.symbol_name(name).unwrap_or("<unnamed>")
                            ),
                        )
                    })?;
                if member != field.ty {
                    return Err(Diagnostic::new(
                        span,
                        format!(
                            "interface member '{}' has a different type",
                            nominals.symbol_name(name).unwrap_or("<unnamed>")
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}
