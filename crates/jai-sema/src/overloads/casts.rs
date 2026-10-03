//! Contextual casts validate the selected target without lowering their operands.
use super::*;
use jai_ir::{ConstantKind, ConstantValue};

pub(super) fn concrete(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentInfo,
    mode: CastMode,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    if let CastMode::Force(strength) = mode {
        let source_type = force_source_type(types, nominals, source, span)?;
        let policy = nominals.layout_policy().ok_or_else(|| {
            Diagnostic::new(
                span,
                "storage cast is waiting for the compilation target layout",
            )
        })?;
        jai_types::StorageBitcast::prove(types, policy, source_type, target, strength)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        return Ok(ConversionRank::Literal);
    }
    if let ArgumentType::ContextualCast { mode, value } = &source.ty {
        concrete(types, nominals, target, value, *mode, span)?;
        return Ok(ConversionRank::Literal);
    }
    let kind = types
        .kind(target)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if matches!(source.ty, ArgumentType::ArrayLiteral { .. }) && matches!(kind, TypeKind::Slice(_))
    {
        literals::concrete(types, nominals, target, &source.ty, span)?;
        return Ok(ConversionRank::Literal);
    }
    if matches!(kind, TypeKind::Distinct(_)) {
        let representation = terminal_representation(types, target, span)?;
        let source = representation_argument(types, source, span)?;
        if matches!(
            types.kind(representation),
            Ok(TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Bool)
        ) {
            return concrete(types, nominals, representation, &source, mode, span);
        }
        if mode == CastMode::Truncate
            && !matches!(types.kind(representation), Ok(TypeKind::Pointer(_)))
        {
            return Err(Diagnostic::new(
                span,
                "truncate casts require an integer or pointer representation",
            ));
        }
        // A nonnumeric distinct cast wraps ordinary coercion of the unwrapped
        // source, rather than adding explicit casts to that representation.
        super::concrete(types, nominals, representation, &source.ty, true, span)?;
        return Ok(ConversionRank::Literal);
    }
    if mode == CastMode::Truncate
        && !matches!(
            kind,
            TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Pointer(_) | TypeKind::Distinct(_)
        )
    {
        return Err(Diagnostic::new(
            span,
            "truncate casts require an integer or pointer destination",
        ));
    }
    let valid = match &source.ty {
        ArgumentType::Null => matches!(
            kind,
            TypeKind::Pointer(_)
                | TypeKind::Procedure(_)
                | TypeKind::Integer(_)
                | TypeKind::Enum(_)
                | TypeKind::Bool
        ),
        ArgumentType::EnumMember(name) => {
            matches!(kind, TypeKind::Enum(_)) && nominals.enum_member(target, *name).is_some()
        }
        ArgumentType::WeakInteger { .. } => matches!(
            kind,
            TypeKind::Pointer(_)
                | TypeKind::Integer(_)
                | TypeKind::Enum(_)
                | TypeKind::Float(_)
                | TypeKind::Bool
        ),
        ArgumentType::WeakFloat { .. } | ArgumentType::WeakFloatExpression { .. } => {
            matches!(kind, TypeKind::Float(_) | TypeKind::Bool)
                || (mode == CastMode::Checked
                    && matches!(kind, TypeKind::Integer(_) | TypeKind::Enum(_)))
        }
        source => {
            let Some(source_type) = literals::default_type(source) else {
                return Err(Diagnostic::new(
                    span,
                    "contextual cast operand requires a source type",
                ));
            };
            if matches!(
                source,
                ArgumentType::RecordLiteral { .. } | ArgumentType::ArrayLiteral { .. }
            ) {
                literals::concrete(types, nominals, source_type, source, span)?;
            }
            crate::inferred_casts::cast_kind_convertible(types, source_type, target, mode)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                || (matches!(kind, TypeKind::Bool) && nominals.enum_flags(source_type))
                || (mode != CastMode::Truncate
                    && matches!(kind, TypeKind::Pointer(_))
                    && matches!(types.kind(source_type), Ok(TypeKind::Type))
                    && jai_types::RuntimeTypeSchema::from_view(types).is_ok())
        }
    };
    if valid {
        // Every alternative obtains its target from context. The source width
        // does not privilege one of several explicit cast destinations.
        Ok(ConversionRank::Literal)
    } else {
        Err(Diagnostic::new(
            span,
            "contextual cast is not defined for this parameter type",
        ))
    }
}

pub(super) fn structural(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    pattern: &TypePattern,
    source: &ArgumentInfo,
    substitution: &Substitution,
    mode: CastMode,
    span: Span,
) -> Result<ConversionRank, Diagnostic> {
    ensure_bound(pattern, substitution, span)?;
    if matches!(mode, CastMode::Force(_)) {
        let target = target_type(types, pattern, substitution, span)?;
        return concrete(types, nominals, target, source, mode, span);
    }
    match pattern {
        TypePattern::Pointer(_) => {
            if let ArgumentType::ContextualCast { mode, value } = &source.ty {
                return structural(types, nominals, pattern, value, substitution, *mode, span);
            }
            let valid = match &source.ty {
                ArgumentType::Null | ArgumentType::WeakInteger { .. } => true,
                ArgumentType::Known(source) => {
                    matches!(
                        types.kind(*source),
                        Ok(TypeKind::Pointer(_) | TypeKind::Integer(_))
                    ) || (mode != CastMode::Truncate
                        && matches!(types.kind(*source), Ok(TypeKind::Type))
                        && jai_types::RuntimeTypeSchema::from_view(types).is_ok())
                }
                _ => false,
            };
            if valid {
                Ok(ConversionRank::Literal)
            } else {
                Err(Diagnostic::new(
                    span,
                    "contextual pointer cast requires a pointer, integer, or null operand",
                ))
            }
        }
        TypePattern::Slice(_) => {
            compatible(
                types,
                nominals,
                pattern,
                &source.ty,
                substitution,
                true,
                span,
            )?;
            Ok(ConversionRank::Literal)
        }
        _ => {
            let target = target_type(types, pattern, substitution, span)?;
            concrete(types, nominals, target, source, mode, span)
        }
    }
}

pub(super) fn target_type(
    types: &dyn TypeView,
    pattern: &TypePattern,
    substitution: &Substitution,
    span: Span,
) -> Result<TypeId, Diagnostic> {
    let kind = match pattern {
        TypePattern::Restricted { ty, .. } => return target_type(types, ty, substitution, span),
        TypePattern::Procedure(pattern) => {
            let signature = procedures::signature(types, pattern, substitution, span)?;
            return types.lookup_procedure(&signature).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "baked callback requires an already canonical signature",
                )
            });
        }
        TypePattern::Concrete(ty) => return Ok(*ty),
        TypePattern::Infer(name) | TypePattern::Variable(name) => {
            return substitution.ty(*name).ok_or_else(|| {
                Diagnostic::new(span, "contextual cast cannot infer its target type")
            });
        }
        TypePattern::Pointer(element) => {
            TypeKind::Pointer(target_type(types, element, substitution, span)?)
        }
        TypePattern::Slice(element) => {
            TypeKind::Slice(target_type(types, element, substitution, span)?)
        }
        TypePattern::DynamicArray(element) => {
            TypeKind::DynamicArray(target_type(types, element, substitution, span)?)
        }
        TypePattern::FixedArray { element, count } => TypeKind::FixedArray {
            element: target_type(types, element, substitution, span)?,
            count: match count {
                CountPattern::Exact(count) => *count,
                CountPattern::Infer(name) | CountPattern::Variable(name) => substitution
                    .constant(*name)
                    .and_then(BakedValue::as_integer)
                    .and_then(|value| u64::try_from(value.value()).ok())
                    .ok_or_else(|| {
                        Diagnostic::new(span, "contextual cast array count is not bound")
                    })?,
            },
        },
        TypePattern::NominalApplication { .. } => {
            return Err(Diagnostic::new(
                span,
                "contextual casts to nominal applications are not supported",
            ));
        }
    };
    types.lookup(&kind).ok_or_else(|| {
        Diagnostic::new(
            span,
            "baked contextual cast requires an already canonical target type",
        )
    })
}

pub(super) fn bake(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentInfo,
    mode: CastMode,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    concrete(types, nominals, target, source, mode, span)?;
    if matches!(mode, CastMode::Force(_)) {
        return Err(Diagnostic::new(
            span,
            "baked force casts require target-layout VM constant materialization",
        ));
    }
    if let ArgumentType::ContextualCast { mode: inner, value } = &source.ty {
        let value = bake(types, nominals, target, value, *inner, span)?;
        return bake(
            types,
            nominals,
            target,
            &ArgumentInfo::constant(value, target),
            mode,
            span,
        );
    }
    let kind = types
        .kind(target)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    if let TypeKind::Distinct(_) = kind {
        let representation = types
            .distinct_definition(target)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .representation;
        let source = representation_argument(types, source, span)?;
        let value = if matches!(
            types.kind(terminal_representation(types, representation, span)?),
            Ok(TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Bool)
        ) {
            bake(types, nominals, representation, &source, mode, span)?
        } else {
            coerce_baked(types, nominals, representation, &source, span)?
        }
        .into_runtime(representation, types)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        return BakedValue::runtime(
            ConstantValue {
                ty: target,
                kind: ConstantKind::Distinct(Box::new(value)),
            },
            types,
        )
        .map_err(|error| Diagnostic::new(span, error.to_string()));
    }
    if matches!(
        kind,
        TypeKind::Record(_) | TypeKind::Any(_) | TypeKind::FixedArray { .. } | TypeKind::String
    ) {
        let source = representation_argument(types, source, span)?;
        return coerce_baked(types, nominals, target, &source, span);
    }
    let constant = source.constant.as_ref().ok_or_else(|| {
        Diagnostic::new(
            span,
            "baked contextual cast requires a compile-time constant",
        )
    })?;
    if matches!(kind, TypeKind::Slice(_))
        && let ConstantArgument::Value(BakedValue::Value(value)) = constant
        && value.ty == target
    {
        return BakedValue::runtime(value.clone(), types)
            .map_err(|error| Diagnostic::new(span, error.to_string()));
    }
    let integer = match constant {
        ConstantArgument::IntegerLiteral(value) => Some(*value),
        ConstantArgument::Null => Some(0),
        ConstantArgument::EnumMember(name) => {
            nominals.enum_member(target, *name).map(Integer::value)
        }
        ConstantArgument::Value(BakedValue::Value(value)) => integer_constant(value),
        _ => None,
    };
    let weak_float_target = if let TypeKind::Float(target) = kind {
        Some(*target)
    } else {
        None
    };
    let float = float_constant(source, weak_float_target, span)?;
    let result = match kind {
        TypeKind::Integer(_) | TypeKind::Enum(_) => {
            let integer_type = if let TypeKind::Integer(ty) = kind {
                *ty
            } else {
                types
                    .enum_definition(target)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                    .representation
            };
            let value = if let Some(value) = integer {
                match mode {
                    CastMode::Checked => {
                        Integer::checked(integer_type, value).ok_or_else(|| {
                            Diagnostic::new(span, "baked checked cast is out of range")
                        })?
                    }
                    CastMode::Unchecked | CastMode::Truncate => {
                        Integer::wrapping(integer_type, value)
                    }
                    CastMode::Force(_) => {
                        return Err(Diagnostic::new(
                            span,
                            "baked force casts require target-layout VM constant materialization",
                        ));
                    }
                }
            } else if let Some(value) = float {
                value
                    .to_integer(integer_type, jai_types::FloatToIntMode::Truncate)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
            } else {
                return Err(Diagnostic::new(
                    span,
                    "baked cast operand has no numeric value",
                ));
            };
            if matches!(kind, TypeKind::Enum(_)) {
                ConstantKind::Enum(value)
            } else {
                ConstantKind::Int(value)
            }
        }
        TypeKind::Float(ty) => ConstantKind::Float(if let Some(value) = integer {
            integer_float(value, *ty, span)?
        } else if let Some(value) = float {
            value.convert(*ty)
        } else {
            return Err(Diagnostic::new(
                span,
                "baked cast operand has no numeric value",
            ));
        }),
        TypeKind::Bool => ConstantKind::Bool(if let Some(value) = integer {
            value != 0
        } else if let Some(value) = float {
            value.to_f64() != 0.0
        } else {
            match constant {
                ConstantArgument::Value(BakedValue::Value(ConstantValue {
                    kind: ConstantKind::Procedure(_),
                    ..
                })) => true,
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "baked cast operand has no truth value",
                    ));
                }
            }
        }),
        TypeKind::Pointer(_) if integer == Some(0) => ConstantKind::Zero,
        TypeKind::Procedure(_) => match constant {
            ConstantArgument::Null => ConstantKind::Zero,
            ConstantArgument::Value(BakedValue::Value(value)) if value.ty == target => {
                value.kind.clone()
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "baked procedure cast requires an exact procedure identity or null",
                ));
            }
        },
        _ => {
            return Err(Diagnostic::new(
                span,
                "this contextual cast cannot supply an owned baked constant",
            ));
        }
    };
    BakedValue::runtime(
        ConstantValue {
            ty: target,
            kind: result,
        },
        types,
    )
    .map_err(|error| Diagnostic::new(span, error.to_string()))
}

fn terminal_representation(
    types: &dyn TypeView,
    mut ty: TypeId,
    span: Span,
) -> Result<TypeId, Diagnostic> {
    let mut seen = HashSet::new();
    while matches!(types.kind(ty), Ok(TypeKind::Distinct(_))) {
        if !seen.insert(ty) {
            return Err(Diagnostic::new(span, "cyclic distinct cast representation"));
        }
        ty = types
            .distinct_definition(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .representation;
    }
    types
        .kind(ty)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    Ok(ty)
}

fn representation_argument(
    types: &dyn TypeView,
    source: &ArgumentInfo,
    span: Span,
) -> Result<ArgumentInfo, Diagnostic> {
    let mut source = source.clone();
    if let Some(ConstantArgument::Value(BakedValue::Value(value))) = &source.constant
        && matches!(value.kind, ConstantKind::Distinct(_))
    {
        let mut value = value;
        while let ConstantKind::Distinct(inner) = &value.kind {
            let representation = types
                .distinct_definition(value.ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?
                .representation;
            if inner.ty != representation {
                return Err(Diagnostic::new(
                    span,
                    "distinct constant has a different representation type",
                ));
            }
            value = inner;
        }
        let value = BakedValue::runtime(value.clone(), types)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        if let ArgumentType::Known(ty) = &mut source.ty {
            *ty = terminal_representation(types, *ty, span)?;
        }
        source.constant = Some(ConstantArgument::Value(value));
    } else if let ArgumentType::Known(ty) = &mut source.ty {
        *ty = terminal_representation(types, *ty, span)?;
    }
    if let ArgumentType::StringLiteral(ty) = source.ty {
        // Ordinary coercion of a resolved string does not inherit the source
        // literal's special expected-*u8 lowering.
        source.ty = ArgumentType::Known(ty);
    }
    Ok(source)
}

fn coerce_baked(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    target: TypeId,
    source: &ArgumentInfo,
    span: Span,
) -> Result<BakedValue, Diagnostic> {
    if matches!(types.kind(target), Ok(TypeKind::Distinct(_))) {
        return bake(types, nominals, target, source, CastMode::Checked, span);
    }
    super::concrete(types, nominals, target, &source.ty, true, span)?;
    if matches!(types.kind(target), Ok(TypeKind::Pointer(_)))
        && let Some(ConstantArgument::Value(BakedValue::Value(value))) = &source.constant
        && matches!(value.kind, ConstantKind::Zero)
        && matches!(types.kind(value.ty), Ok(TypeKind::Pointer(_)))
    {
        return Ok(BakedValue::Value(ConstantValue {
            ty: target,
            kind: ConstantKind::Zero,
        }));
    }
    let value = super::bake_with_nominals(
        types,
        nominals,
        &TypePattern::Concrete(target),
        source,
        &Substitution::default(),
        span,
    )?;
    // A retained source value must already have the selected representation;
    // nonempty sequence views require a separate immutable backing recipe.
    value
        .clone()
        .into_runtime(target, types)
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
    Ok(value)
}

fn force_source_type(
    types: &dyn TypeView,
    nominals: &dyn NominalView,
    source: &ArgumentInfo,
    span: Span,
) -> Result<TypeId, Diagnostic> {
    let ty = match &source.ty {
        ArgumentType::WeakInteger { minimum, maximum }
            if *minimum >= IntegerType::S64.min() && *maximum <= IntegerType::S64.max() =>
        {
            types.scalar(ScalarType::Int(IntegerType::S64))
        }
        ArgumentType::WeakFloat {
            default, spelling, ..
        } => {
            FloatValue::parse_decimal(*default, spelling)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            types.float(*default)
        }
        ArgumentType::WeakFloatExpression {
            default,
            permits_f32,
            permits_f64,
        } => {
            if !match default {
                FloatType::F32 => *permits_f32,
                FloatType::F64 => *permits_f64,
            } {
                return Err(Diagnostic::new(
                    span,
                    "weak floating-point expression has no valid default type",
                ));
            }
            types.float(*default)
        }
        other => literals::default_type(other).ok_or_else(|| {
            Diagnostic::new(
                span,
                "force cast operand requires a context-free source type",
            )
        })?,
    };
    if matches!(
        source.ty,
        ArgumentType::RecordLiteral { .. } | ArgumentType::ArrayLiteral { .. }
    ) {
        literals::concrete(types, nominals, ty, &source.ty, span)?;
    }
    Ok(ty)
}

fn integer_constant(value: &ConstantValue) -> Option<i128> {
    match &value.kind {
        ConstantKind::Int(value) | ConstantKind::Enum(value) => Some(value.value()),
        ConstantKind::Bool(value) => Some(i128::from(*value)),
        ConstantKind::Distinct(value) => integer_constant(value),
        ConstantKind::Zero => Some(0),
        _ => None,
    }
}
fn float_constant(
    source: &ArgumentInfo,
    weak_target: Option<FloatType>,
    span: Span,
) -> Result<Option<FloatValue>, Diagnostic> {
    Ok(match source.constant.as_ref() {
        Some(ConstantArgument::Value(BakedValue::Float(value))) => Some(*value),
        Some(ConstantArgument::Value(BakedValue::Value(value))) => runtime_float_constant(value),
        Some(ConstantArgument::FloatLiteral { spelling, negative }) => {
            let ArgumentType::WeakFloat { default, .. } = source.ty else {
                return Ok(None);
            };
            let value = FloatValue::parse_decimal(weak_target.unwrap_or(default), spelling)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            Some(if *negative { value.negate() } else { value })
        }
        Some(ConstantArgument::FloatExpression { f32, f64 }) => {
            let ArgumentType::WeakFloatExpression { default, .. } = source.ty else {
                return Ok(None);
            };
            match weak_target.unwrap_or(default) {
                FloatType::F32 => *f32,
                FloatType::F64 => *f64,
            }
        }
        _ => None,
    })
}
fn runtime_float_constant(value: &ConstantValue) -> Option<FloatValue> {
    match &value.kind {
        ConstantKind::Float(value) => Some(*value),
        ConstantKind::Distinct(value) => runtime_float_constant(value),
        _ => None,
    }
}
