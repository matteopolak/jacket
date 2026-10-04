//! Contextual literal validation uses immutable typed declaration metadata.
use super::*;
use jai_ir::{ConstantKind, ConstantValue};

pub(super) fn default_type(source: &ArgumentType) -> Option<TypeId> {
    match source {
        ArgumentType::Known(ty)
        | ArgumentType::StringLiteral(ty)
        | ArgumentType::RecordLiteral {
            ty: Some(ty), ..
        }
        | ArgumentType::ArrayLiteral {
            default: Some(ty), ..
        } => Some(*ty),
        _ => None,
    }
}
pub(super) fn explicit_type(source: &ArgumentType) -> Option<TypeId> {
    match source {
        ArgumentType::RecordLiteral {
            ty, ..
        } => *ty,
        ArgumentType::ArrayLiteral {
            explicit: Some(_),
            default,
            ..
        } => *default,
        _ => None,
    }
}

pub(super) fn concrete(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentType,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    match source {
        ArgumentType::RecordLiteral {
            ty,
            fields,
        } => {
            if let Some(source_type) = ty.filter(|ty| *ty != target) {
                concrete(types, nominals, source_type, source, span)?;
                return if nominals.implicit_conversion(source_type, target, span)? {
                    Ok(ConversionRank::Widening)
                } else {
                    Err(Diagnostic::new(
                        span,
                        "record literal has a different nominal type",
                    ))
                };
            }
            super::record_literals::compatible(types, nominals, target, fields, ty.is_some(), span)
        }
        ArgumentType::ArrayLiteral {
            explicit,
            elements,
            ..
        } => {
            let (element, count, rank) = match types
                .kind(target)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            {
                TypeKind::FixedArray {
                    element,
                    count,
                } => (*element, Some(*count), ConversionRank::Exact),
                TypeKind::Slice(element) => (*element, None, ConversionRank::ArrayView),
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "array literal requires a fixed array or slice parameter",
                    ));
                }
            };
            if count.is_some_and(|count| usize::try_from(count).ok() != Some(elements.len())) {
                return Err(Diagnostic::new(
                    span,
                    "array literal length differs from its context",
                ));
            }
            if explicit.is_some_and(|explicit| explicit != element) {
                return Err(Diagnostic::new(
                    span,
                    "array literal element type differs from its context",
                ));
            }
            let mut rank = rank;
            for value in elements {
                rank = rank.max(compatible(
                    types,
                    nominals,
                    &TypePattern::Concrete(element),
                    &value.ty,
                    &Substitution::default(),
                    true,
                    span,
                )?);
            }
            Ok(rank)
        }
        _ => Err(Diagnostic::new(
            span,
            "expected a contextual aggregate literal",
        )),
    }
}

pub(super) fn array_pattern(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &TypePattern,
    source: &ArgumentType,
    substitution: &Substitution,
    allow_conversion: bool,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    let ArgumentType::ArrayLiteral {
        explicit,
        elements,
        ..
    } = source
    else {
        unreachable!();
    };
    let (element, count, mut rank) = match pattern {
        TypePattern::FixedArray {
            element,
            count,
        } => (element.as_ref(), Some(count), ConversionRank::Exact),
        TypePattern::Slice(element) if allow_conversion => {
            (element.as_ref(), None, ConversionRank::ArrayView)
        }
        _ => {
            return Err(Diagnostic::new(
                span,
                "array literal does not match the structural parameter type",
            ));
        }
    };
    if let Some(count) = count {
        let expected = match count {
            CountPattern::Exact(count) => *count,
            CountPattern::Infer(name) | CountPattern::Variable(name) => substitution
                .constant(*name)
                .and_then(BakedValue::as_integer)
                .and_then(|value| u64::try_from(value.value()).ok())
                .ok_or_else(|| Diagnostic::new(span, "array count could not be inferred"))?,
        };
        if usize::try_from(expected).ok() != Some(elements.len()) {
            return Err(Diagnostic::new(
                span,
                "array literal length differs from its context",
            ));
        }
    }
    if let Some(explicit) = explicit {
        compatible(
            types,
            nominals,
            element,
            &ArgumentType::Known(*explicit),
            substitution,
            false,
            span,
        )?;
    }
    ensure_bound(element, substitution, span)?;
    for value in elements {
        rank = rank.max(compatible(
            types,
            nominals,
            element,
            &value.ty,
            substitution,
            true,
            span,
        )?);
    }
    Ok(rank)
}

pub(super) fn bake(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentType,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    concrete(types, nominals, target, source, span)?;
    let kind = match source {
        ArgumentType::RecordLiteral {
            fields, ..
        } => super::record_literals::bake(types, nominals, target, fields, span)?.kind,
        ArgumentType::ArrayLiteral {
            elements, ..
        } => {
            let TypeKind::FixedArray {
                element, ..
            } = *types
                .kind(target)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
            else {
                return Err(Diagnostic::new(
                    span,
                    "baked array literal requires fixed-array storage",
                ));
            };
            ConstantKind::Array(
                elements
                    .iter()
                    .map(|value| {
                        super::bake_with_nominals(
                            types,
                            nominals,
                            &TypePattern::Concrete(element),
                            value,
                            &Substitution::default(),
                            span,
                        )?
                        .into_runtime(element, types)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))
                    })
                    .collect::<Result<_, _>>()?,
            )
        }
        _ => unreachable!("literal bake receives a contextual aggregate"),
    };
    BakedValue::runtime(
        ConstantValue {
            ty: target,
            kind,
        },
        types,
    )
    .map_err(|error| Diagnostic::new(span, error.to_string()))
}
