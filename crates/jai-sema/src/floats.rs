//! Contextual decimal binding and the floating-point expression domain.
use super::*;
use jai_syntax::{DecimalLiteral, FloatLiteral};
use jai_types::{FloatOp, FloatType, FloatValue};

pub(super) enum WeakFloat {
    Bound(std::sync::Arc<jai_eval::floats::WeakFloatValue>),
    Decimal(DecimalLiteral),
    Negate(Box<WeakFloat>),
    Binary(FloatOp, Box<Expr>, Box<Expr>),
}
impl WeakFloat {
    fn default_type(&self) -> FloatType {
        match self {
            Self::Bound(value) => value.default_type(),
            Self::Decimal(decimal) => jai_eval::floats::default_decimal_type(decimal),
            Self::Negate(value) => value.default_type(),
            Self::Binary(_, left, right) => {
                if left.default_float_type() == FloatType::F64
                    || right.default_float_type() == FloatType::F64
                {
                    FloatType::F64
                } else {
                    FloatType::F32
                }
            }
        }
    }
    fn bind(self, ty: FloatType, span: Span) -> Result<FloatExpr, Diagnostic> {
        Ok(match self {
            Self::Bound(value) => FloatExpr::constant(value.round(ty, span)?),
            Self::Decimal(decimal) => {
                let value = match ty {
                    FloatType::F32 => FloatValue::from_f32(
                        decimal
                            .round_f32()
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                    ),
                    FloatType::F64 => FloatValue::from_f64(
                        decimal
                            .round_f64()
                            .map_err(|error| Diagnostic::new(span, error.to_string()))?,
                    ),
                };
                FloatExpr::constant(value)
            }
            Self::Negate(value) => {
                FloatExpr::new(ty, FloatExprKind::Negate(Box::new(value.bind(ty, span)?)))
            }
            Self::Binary(operation, left, right) => FloatExpr::new(
                ty,
                FloatExprKind::Binary(
                    operation,
                    Box::new(left.float_as(ty, span)?),
                    Box::new(right.float_as(ty, span)?),
                ),
            ),
        })
    }
}
impl Expr {
    pub(super) fn float_literal(literal: &FloatLiteral) -> Self {
        match literal {
            FloatLiteral::Decimal(value) => Self::WeakFloat(WeakFloat::Decimal(value.clone())),
            FloatLiteral::Bits32(value) => {
                Self::Float(FloatExpr::constant(FloatValue::F32(*value)))
            }
            FloatLiteral::Bits64(value) => {
                Self::Float(FloatExpr::constant(FloatValue::F64(*value)))
            }
        }
    }
    pub(super) fn has_float(&self) -> bool {
        match self {
            Self::Float(_) | Self::WeakFloat(_) => true,
            Self::WeakConditional(value) => {
                value.then_value.has_float() || value.else_value.has_float()
            }
            _ => false,
        }
    }
    pub(super) fn default_float_type(&self) -> FloatType {
        match self {
            Self::Float(value) => value.ty(),
            Self::WeakFloat(value) => value.default_type(),
            Self::WeakConditional(value) => {
                if let (Self::Float(a), Self::Float(b)) = (&value.then_value, &value.else_value) {
                    return if a.ty() == FloatType::F64 || b.ty() == FloatType::F64 {
                        FloatType::F64
                    } else {
                        FloatType::F32
                    };
                }
                if let Self::Float(a) = &value.then_value {
                    return a.ty();
                }
                if let Self::Float(b) = &value.else_value {
                    return b.ty();
                }
                if value.then_value.default_float_type() == FloatType::F64
                    || value.else_value.default_float_type() == FloatType::F64
                {
                    FloatType::F64
                } else {
                    FloatType::F32
                }
            }
            _ => FloatType::F32,
        }
    }
    pub(super) fn float(self, span: Span) -> Result<FloatExpr, Diagnostic> {
        let ty = self.default_float_type();
        self.float_as(ty, span)
    }
    pub(super) fn float_as(self, ty: FloatType, span: Span) -> Result<FloatExpr, Diagnostic> {
        match self {
            Self::Float(value) if value.ty() == ty => Ok(value),
            Self::Float(value) if ty == FloatType::F64 => {
                Ok(FloatExpr::new(ty, FloatExprKind::Cast(Box::new(value))))
            }
            Self::WeakFloat(value) => value.bind(ty, span),
            Self::Literal(value) => {
                let value = IntegerValue::checked(IntegerType::S64, value)
                    .or_else(|| IntegerValue::checked(IntegerType::U64, value))
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "integer literal exceeds the supported float conversion range",
                        )
                    })?;
                Ok(FloatExpr::constant(FloatValue::from_integer(ty, value)))
            }
            Self::WeakConditional(value) => Ok(FloatExpr::new(
                ty,
                FloatExprKind::Conditional(Box::new(Conditional {
                    condition: value.condition,
                    then_value: value.then_value.float_as(ty, span)?,
                    else_value: value.else_value.float_as(ty, span)?,
                })),
            )),
            _ => Err(Diagnostic::new(
                span,
                "implicit floating-point conversion requires a weak number or a preserving float width",
            )),
        }
    }
    pub(super) fn cast_float(self, ty: FloatType, span: Span) -> Result<FloatExpr, Diagnostic> {
        match self {
            Self::Float(value) => Ok(FloatExpr::new(ty, FloatExprKind::Cast(Box::new(value)))),
            Self::Int(value) => Ok(FloatExpr::new(ty, FloatExprKind::FromInt(Box::new(value)))),
            value => value.float_as(ty, span),
        }
    }
    pub(super) fn negate_float(self, span: Span) -> Result<Self, Diagnostic> {
        Ok(match self {
            Self::WeakFloat(value) => Self::WeakFloat(WeakFloat::Negate(Box::new(value))),
            value => {
                let value = value.float(span)?;
                Self::Float(FloatExpr::new(
                    value.ty(),
                    FloatExprKind::Negate(Box::new(value)),
                ))
            }
        })
    }
}
impl Resolver<'_> {
    pub(super) fn float_binary(
        &self,
        operation: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let relation = match Operator::from(operation) {
            Operator::Relation(value) => Some(value),
            Operator::Equality(Equality::Equal) => Some(Relation::Equal),
            Operator::Equality(Equality::NotEqual) => Some(Relation::NotEqual),
            _ => None,
        };
        let arithmetic = match Operator::from(operation) {
            Operator::Integer(IntOp::Add) => Some(FloatOp::Add),
            Operator::Integer(IntOp::Subtract) => Some(FloatOp::Subtract),
            Operator::Integer(IntOp::Multiply) => Some(FloatOp::Multiply),
            Operator::Integer(IntOp::Divide) => Some(FloatOp::Divide),
            Operator::Integer(IntOp::Remainder) => Some(FloatOp::Remainder),
            _ => None,
        };
        if let Some(operation) = arithmetic
            && !matches!(left, Expr::Float(_))
            && !matches!(right, Expr::Float(_))
        {
            return Ok(Expr::WeakFloat(WeakFloat::Binary(
                operation,
                Box::new(left),
                Box::new(right),
            )));
        }
        let ty = match (&left, &right) {
            (Expr::Float(a), Expr::Float(b)) => {
                if a.ty() == FloatType::F64 || b.ty() == FloatType::F64 {
                    FloatType::F64
                } else {
                    FloatType::F32
                }
            }
            (Expr::Float(value), _) | (_, Expr::Float(value)) => value.ty(),
            _ => {
                if left.default_float_type() == FloatType::F64
                    || right.default_float_type() == FloatType::F64
                {
                    FloatType::F64
                } else {
                    FloatType::F32
                }
            }
        };
        let (left, right) = (left.float_as(ty, span)?, right.float_as(ty, span)?);
        if let Some(operation) = relation {
            return Ok(Expr::Bool(BoolExpr::CompareFloats(
                operation,
                Box::new(left),
                Box::new(right),
            )));
        }
        let operation = arithmetic.ok_or_else(|| {
            Diagnostic::new(span, "operator is not defined for floating-point values")
        })?;
        Ok(Expr::Float(FloatExpr::new(
            ty,
            FloatExprKind::Binary(operation, Box::new(left), Box::new(right)),
        )))
    }
}
