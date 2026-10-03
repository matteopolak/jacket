//! Casts use a known destination identity without inventing an operand type.
use super::*;
use jai_types::{TypeError, TypeKind, TypeView};
#[cfg(test)]
mod tests;

/// Candidate selection shares the cast domain without evaluating the operand.
pub(crate) fn cast_kind_convertible(
    types: &dyn TypeView,
    source: TypeId,
    target: TypeId,
    mode: CastMode,
) -> Result<bool, TypeError> {
    let mut source = source;
    let mut target = target;
    let mut seen = std::collections::HashSet::new();
    while let TypeKind::Distinct(_) = types.kind(source)? {
        if !seen.insert(source) {
            return Ok(false);
        }
        source = types.distinct_definition(source)?.representation;
    }
    seen.clear();
    let wrapped_target = matches!(types.kind(target)?, TypeKind::Distinct(_));
    while let TypeKind::Distinct(_) = types.kind(target)? {
        if !seen.insert(target) {
            return Ok(false);
        }
        target = types.distinct_definition(target)?.representation;
    }
    let source_kind = types.kind(source)?;
    if mode == CastMode::Truncate
        && (!matches!(
            types.kind(target)?,
            TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Pointer(_)
        ) || matches!(source_kind, TypeKind::Float(_) | TypeKind::Bool))
    {
        return Ok(false);
    }
    if wrapped_target
        && !matches!(
            types.kind(target)?,
            TypeKind::Integer(_) | TypeKind::Float(_) | TypeKind::Bool
        )
    {
        // Distinct scalar casts convert; other representations use ordinary coercion.
        let pointer_erasure = matches!(
            (source_kind, types.kind(target)?),
            (TypeKind::Pointer(_), TypeKind::Pointer(pointee))
                if matches!(types.kind(*pointee)?, TypeKind::Void)
        );
        return Ok(source == target
            || pointer_erasure
            || matches!((source_kind, types.kind(target)?),
            (TypeKind::FixedArray { element: a, .. } | TypeKind::DynamicArray(a), TypeKind::Slice(b)) if a == b));
    }
    Ok(match types.kind(target)? {
        TypeKind::Pointer(_) => matches!(source_kind, TypeKind::Pointer(_) | TypeKind::Integer(_)),
        TypeKind::Integer(_) | TypeKind::Enum(_) => {
            matches!(
                source_kind,
                TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Bool | TypeKind::Pointer(_)
            ) || (mode == CastMode::Checked && matches!(source_kind, TypeKind::Float(_)))
        }
        TypeKind::Float(_) => matches!(source_kind, TypeKind::Float(_) | TypeKind::Integer(_)),
        TypeKind::Bool => matches!(
            source_kind,
            TypeKind::Integer(_)
                | TypeKind::Float(_)
                | TypeKind::Bool
                | TypeKind::Pointer(_)
                | TypeKind::Procedure(_)
        ),
        TypeKind::Procedure(_) => source == target,
        TypeKind::Record(_) | TypeKind::Any(_) | TypeKind::FixedArray { .. } | TypeKind::String => {
            source == target
        }
        TypeKind::Slice(element) => match source_kind {
            TypeKind::FixedArray {
                element: source, ..
            }
            | TypeKind::Slice(source)
            | TypeKind::DynamicArray(source) => source == element,
            TypeKind::String => *element == types.scalar(ScalarType::Int(IntegerType::U8)),
            _ => false,
        },
        _ => false,
    })
}

impl Resolver<'_> {
    pub(crate) fn condition_expression(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<BoolExpr, Diagnostic> {
        let value = if needs_cast_context(source) {
            let target = self.types.scalar(ScalarType::Bool);
            self.expr_expected(source, target)?
        } else {
            self.expr(source)?
        };
        value.condition(source.span, self.types)
    }

    pub(crate) fn compound_operand(
        &mut self,
        source: &syntax::Expression,
        target: TypeId,
        operation: BinaryOp,
    ) -> Result<Expr, Diagnostic> {
        let target = if matches!(operation, BinaryOp::Add | BinaryOp::Subtract)
            && matches!(self.types.kind(target), Ok(TypeKind::Pointer(_)))
        {
            self.types.scalar(ScalarType::Int(IntegerType::S64))
        } else {
            target
        };
        self.expr_expected(source, target)
    }
    pub(crate) fn inferred_binary(
        &mut self,
        operation: BinaryOp,
        left: &syntax::Expression,
        right: &syntax::Expression,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let left_inferred = matches!(left.kind, syntax::ExpressionKind::InferredCast { .. });
        let right_inferred = matches!(right.kind, syntax::ExpressionKind::InferredCast { .. });
        let peer = if left_inferred { right } else { left };
        let target = if matches!(Operator::from(operation), Operator::And | Operator::Or) {
            self.types.scalar(ScalarType::Bool)
        } else {
            self.describe_argument_type(peer)?.ok_or_else(|| {
                Diagnostic::new(span, "xx cast requires a destination type from its context")
            })?
        };
        let integer_offset = (operation == BinaryOp::Add
            || (!left_inferred && operation == BinaryOp::Subtract))
            && matches!(self.types.kind(target), Ok(TypeKind::Pointer(_)));
        let target = if integer_offset {
            self.types.scalar(ScalarType::Int(IntegerType::S64))
        } else {
            target
        };
        // Describing the peer is pure; resolving operands retains written source order.
        let left = if left_inferred {
            self.expr_expected(left, target)?
        } else {
            self.expr(left)?
        };
        let right = if right_inferred {
            self.expr_expected(right, target)?
        } else {
            self.expr(right)?
        };
        self.binary(operation, left, right, span)
    }

    pub(crate) fn contextual_scalar_conditional(
        &mut self,
        source: &syntax::ConditionalExpression,
        target: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let condition = self.condition_expression(&source.condition)?;
        let yes = self.expr_expected(&source.then_value, target)?;
        let no = match &source.else_value {
            Some(source) => self.expr_expected(source, target)?,
            None => self.typed_value(ValueExpr::Zero(target), target, span)?,
        };
        self.value_conditional_pair(condition, yes, Some(no), target, span)
    }

    pub(crate) fn cast_expression(
        &mut self,
        source: &syntax::Expression,
        target: TypeId,
        mode: CastMode,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if mode == CastMode::Truncate {
            let mut representation = target;
            let mut seen = std::collections::HashSet::new();
            loop {
                if !seen.insert(representation) {
                    return Err(Diagnostic::new(span, "cyclic distinct cast representation"));
                }
                match self
                    .types
                    .kind(representation)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                {
                    TypeKind::Distinct(_) => {
                        representation = self
                            .types
                            .distinct_definition(representation)
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?
                            .representation;
                    }
                    TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Pointer(_) => break,
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "trunc cast is supported only for integer and pointer representations",
                        ));
                    }
                }
            }
        }
        if let syntax::ExpressionKind::CompileTime(body) = &source.kind {
            return self.resolve_compile_time_cast(body, source.span, target, mode);
        }
        let operand = if matches!(
            source.kind,
            syntax::ExpressionKind::InferredCast { .. } | syntax::ExpressionKind::InferredMember(_)
        ) || (matches!(source.kind, syntax::ExpressionKind::ArrayLiteral(_))
            && matches!(self.types.kind(target), Ok(TypeKind::Slice(_))))
            || (matches!(source.kind, syntax::ExpressionKind::Conditional(_))
                && matches!(
                    self.types.kind(target),
                    Ok(TypeKind::Pointer(_) | TypeKind::Procedure(_))
                ))
            || needs_cast_context(source)
        {
            self.expr_expected(source, target)?
        } else {
            self.expr(source)?
        };
        self.cast_operand(operand, target, mode, span)
    }

    pub(crate) fn cast_operand(
        &mut self,
        operand: Expr,
        target: TypeId,
        mode: CastMode,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let CastMode::Force(strength) = mode {
            return self.storage_cast_operand(operand, target, strength, span);
        }
        let kind = self
            .types
            .kind(target)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        if matches!(kind, TypeKind::Distinct(_)) {
            return self.variant_cast(operand, target, mode, span);
        }
        let operand = self.unwrap_variant(operand, span)?;
        if mode == CastMode::Truncate
            && (!matches!(
                kind,
                TypeKind::Integer(_) | TypeKind::Enum(_) | TypeKind::Pointer(_)
            ) || matches!(
                operand,
                Expr::Float(_) | Expr::WeakFloat(_) | Expr::Bool(_) | Expr::Type(_) | Expr::Code(_)
            ) || self.is_runtime_type_expression(&operand))
        {
            return Err(Diagnostic::new(
                span,
                "trunc cast is supported only for integer and pointer representations",
            ));
        }
        Ok(match kind {
            TypeKind::Pointer(_) => self.pointer_from_operand(operand, target, mode, span)?,
            TypeKind::Procedure(_) => {
                let value = self.coerce_value(operand, target, span)?;
                self.typed_value(value, target, span)?
            }
            TypeKind::Record(_)
            | TypeKind::Any(_)
            | TypeKind::FixedArray { .. }
            | TypeKind::String => {
                if self.expression_type(&operand, span)? != target {
                    return Err(Diagnostic::new(
                        span,
                        "cast requires the exact canonical storage representation",
                    ));
                }
                let value = self.coerce_value(operand, target, span)?;
                self.typed_value(value, target, span)?
            }
            TypeKind::Slice(_) => {
                let value = self.sequence_coercion(operand, target, span)?;
                self.typed_value(value, target, span)?
            }
            TypeKind::Enum(_) => {
                let representation = self.types.enum_definition(target).unwrap().representation;
                let value = operand.cast_integer(representation, mode, span)?;
                Expr::Enum {
                    ty: target,
                    flags: self.enum_is_flags(target),
                    representation,
                    value: ValueExpr::EnumFromInt { ty: target, value },
                }
            }
            TypeKind::Integer(integer) => Expr::Int(operand.cast_integer(integer, mode, span)?),
            TypeKind::Float(float) => Expr::Float(operand.cast_float(float, span)?),
            TypeKind::Bool => Expr::Bool(operand.condition(span, self.types)?),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "cast is not implemented for this type",
                ));
            }
        })
    }
}

pub(crate) fn needs_cast_context(expression: &syntax::Expression) -> bool {
    use syntax::ExpressionKind as E;
    match &expression.kind {
        E::InferredCast { .. } => true,
        E::Conditional(value) => {
            needs_cast_context(&value.then_value)
                || value
                    .else_value
                    .as_ref()
                    .is_some_and(|value| needs_cast_context(value))
        }
        E::Binary(_, left, right) => needs_cast_context(left) || needs_cast_context(right),
        _ => false,
    }
}
