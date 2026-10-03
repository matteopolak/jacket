//! Nominal type variants retain identity across their representation operations.
use super::super::*;
use jai_types::{DistinctKind, TypeKind};
use std::collections::HashSet;

impl Resolver<'_> {
    pub(crate) fn is_variant_expression(&self, expression: &Expr) -> bool {
        matches!(expression, Expr::Typed { ty, .. } if matches!(self.types.kind(*ty), Ok(TypeKind::Distinct(_))))
    }

    /// `isa` permits conversion along its base chain, never to a sibling or child.
    pub(crate) fn implicit_variant_base(
        &self,
        expression: Expr,
        target: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Typed {
            ty, ..
        } = &expression
        else {
            return Ok(expression);
        };
        if *ty == target {
            return Ok(expression);
        }
        let mut current = *ty;
        let mut path = Vec::new();
        let mut visited = HashSet::new();
        while visited.insert(current) {
            let Ok(definition) = self.types.distinct_definition(current) else {
                break;
            };
            if definition.kind != DistinctKind::IsA {
                break;
            }
            current = definition.representation;
            path.push(current);
            if current == target {
                let mut value = expression.value(span)?;
                for ty in path {
                    value = ValueExpr::UnwrapDistinct {
                        value: Box::new(value),
                        ty,
                    };
                }
                return self.typed_value(value, target, span);
            }
        }
        if !path.is_empty() && !matches!(self.types.kind(current), Ok(TypeKind::Distinct(_))) {
            let mut value = expression.value(span)?;
            for ty in path {
                value = ValueExpr::UnwrapDistinct {
                    value: Box::new(value),
                    ty,
                };
            }
            return self.typed_value(value, current, span);
        }
        Ok(expression)
    }

    pub(crate) fn variant_coercion(
        &self,
        expression: Expr,
        target: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        if let Expr::Typed {
            ty, ..
        } = &expression
            && *ty == target
        {
            return expression.value(span);
        }
        if !matches!(
            expression,
            Expr::Literal(_) | Expr::WeakFloat(_) | Expr::Null | Expr::Bool(BoolExpr::Constant(_))
        ) && !matches!(
            &expression,
            Expr::Typed {
                value: ValueExpr::StringBytes { .. },
                ..
            }
        ) {
            return Err(Diagnostic::new(
                span,
                "nonliteral value requires an explicit cast to a distinct type",
            ));
        }
        let (representation, wrappers) = self.variant_storage(target, span)?;
        let mut value = self.coerce_value(expression, representation, span)?;
        for ty in wrappers.into_iter().rev() {
            value = ValueExpr::Distinct {
                ty,
                value: Box::new(value),
            };
        }
        Ok(value)
    }

    pub(crate) fn unwrap_variant(
        &self,
        mut expression: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let mut visited = HashSet::new();
        while let Expr::Typed {
            ty, ..
        } = &expression
        {
            let Ok(definition) = self.types.distinct_definition(*ty) else {
                break;
            };
            if !visited.insert(*ty) {
                return Err(Diagnostic::new(span, "cyclic distinct type representation"));
            }
            let representation = definition.representation;
            expression = self.typed_value(
                ValueExpr::UnwrapDistinct {
                    value: Box::new(expression.value(span)?),
                    ty: representation,
                },
                representation,
                span,
            )?;
        }
        Ok(expression)
    }

    pub(crate) fn variant_cast(
        &self,
        expression: Expr,
        target: TypeId,
        mode: CastMode,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let (representation, wrappers) = self.variant_storage(target, span)?;
        let expression = self.unwrap_variant(expression, span)?;
        if mode == CastMode::Truncate
            && (!matches!(
                self.types.kind(representation),
                Ok(TypeKind::Integer(_) | TypeKind::Pointer(_))
            ) || matches!(
                expression,
                Expr::Float(_) | Expr::WeakFloat(_) | Expr::Bool(_)
            ))
        {
            return Err(Diagnostic::new(
                span,
                "trunc cast is supported only for integer and pointer representations",
            ));
        }
        let mut value = match self
            .types
            .kind(representation)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
        {
            TypeKind::Integer(integer) => {
                ValueExpr::Int(expression.cast_integer(*integer, mode, span)?)
            }
            TypeKind::Float(float) => ValueExpr::Float(expression.cast_float(*float, span)?),
            TypeKind::Bool => ValueExpr::Bool(expression.condition(span, self.types)?),
            _ => self.coerce_value(expression, representation, span)?,
        };
        for ty in wrappers.into_iter().rev() {
            value = ValueExpr::Distinct {
                ty,
                value: Box::new(value),
            };
        }
        self.typed_value(value, target, span)
    }
    fn variant_storage(
        &self,
        target: TypeId,
        span: Span,
    ) -> Result<(TypeId, Vec<TypeId>), Diagnostic> {
        let mut representation = target;
        let mut wrappers = Vec::new();
        let mut visited = HashSet::new();
        while let Ok(definition) = self.types.distinct_definition(representation) {
            if !visited.insert(representation) {
                return Err(Diagnostic::new(span, "cyclic distinct type representation"));
            }
            wrappers.push(representation);
            representation = definition.representation;
        }
        Ok((representation, wrappers))
    }

    pub(crate) fn variant_binary(
        &mut self,
        operation: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let target = match (&left, &right) {
            (
                Expr::Typed {
                    ty, ..
                },
                _,
            ) if self.is_variant_expression(&left) => *ty,
            (
                _,
                Expr::Typed {
                    ty, ..
                },
            ) if self.is_variant_expression(&right) => *ty,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "distinct operation requires a nominal operand",
                ));
            }
        };
        let left = self.implicit_variant_base(left, target, span)?;
        let right = self.implicit_variant_base(right, target, span)?;
        let left = self.variant_coercion(left, target, span)?;
        let right = self.variant_coercion(right, target, span)?;
        let left = self.unwrap_variant(
            Expr::Typed {
                ty: target,
                value: left,
            },
            span,
        )?;
        let right = self.unwrap_variant(
            Expr::Typed {
                ty: target,
                value: right,
            },
            span,
        )?;
        let result = self.binary(operation, left, right, span)?;
        if matches!(result, Expr::Bool(_)) {
            return Ok(result);
        }
        self.variant_cast(result, target, CastMode::Unchecked, span)
    }
    pub(crate) fn variant_unary(
        &self,
        operation: UnaryOp,
        expression: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let Expr::Typed {
            ty: target, ..
        } = &expression
        else {
            return Err(Diagnostic::new(
                span,
                "distinct operation requires a nominal operand",
            ));
        };
        let target = *target;
        if operation == UnaryOp::Positive {
            return Ok(expression);
        }
        let expression = self.unwrap_variant(expression, span)?;
        let result = match operation {
            UnaryOp::LogicalNot => {
                return Ok(Expr::Bool(BoolExpr::Not(Box::new(
                    expression.condition(span, self.types)?,
                ))));
            }
            UnaryOp::Negate if expression.has_float() => expression.negate_float(span)?,
            UnaryOp::Negate | UnaryOp::Complement => {
                let expression = expression.int(span)?;
                let ty = expression.ty();
                let kind = if operation == UnaryOp::Negate {
                    IntExprKind::Negate(Box::new(expression))
                } else {
                    IntExprKind::Complement(Box::new(expression))
                };
                Expr::Int(
                    IntExpr::new(ty, kind).with_overflow_check(self.checks.arithmetic_overflow),
                )
            }
            UnaryOp::Positive => unreachable!("positive handled before unwrapping"),
        };
        self.variant_cast(result, target, CastMode::Unchecked, span)
    }
}

#[cfg(test)]
#[path = "variants/tests.rs"]
mod tests;
