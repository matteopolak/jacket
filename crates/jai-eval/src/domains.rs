//! Infer scalar domains from binding facts before requesting selected values.
use super::*;
use jai_syntax::{BuiltinType, FloatLiteral, TypeSyntax};
use jai_types::{FloatType, FloatValue};

/// A scalar binding's arithmetic domain, independent of its readiness or value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarDomain {
    IntegerLiteral,
    Integer(IntegerType),
    Bool,
    Float(FloatType),
    /// Exact decimal expressions retain their default width until contextual use.
    WeakFloat(FloatType),
}
/// Preparation distinguishes an actual typed source producer from a scalar error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScalarInferenceError {
    Diagnostic(Diagnostic),
    RequiresTypedExecution(Diagnostic),
}
impl ScalarInferenceError {
    pub fn into_diagnostic(self) -> Diagnostic {
        match self {
            Self::Diagnostic(error) | Self::RequiresTypedExecution(error) => error,
        }
    }
}
impl Value {
    pub fn domain(&self) -> ScalarDomain {
        match self {
            Self::Literal(_) => ScalarDomain::IntegerLiteral,
            Self::Int(value) => ScalarDomain::Integer(value.ty()),
            Self::Bool(_) => ScalarDomain::Bool,
            Self::Float(value) => ScalarDomain::Float(value.ty()),
            Self::WeakFloat(value) => ScalarDomain::WeakFloat(value.default_type()),
        }
    }
}
impl ScalarDomain {
    fn number(self, span: Span) -> Result<NumberType, Diagnostic> {
        match self {
            Self::IntegerLiteral => Ok(NumberType::Literal),
            Self::Integer(ty) => Ok(NumberType::Typed(ty)),
            _ => Err(Diagnostic::new(span, "expected integer constant")),
        }
    }
    fn float(self, span: Span) -> Result<(Option<FloatType>, FloatType), Diagnostic> {
        match self {
            Self::IntegerLiteral => Ok((None, FloatType::F32)),
            Self::Float(ty) => Ok((Some(ty), ty)),
            Self::WeakFloat(ty) => Ok((None, ty)),
            _ => Err(Diagnostic::new(
                span,
                "expected numeric floating-point constant",
            )),
        }
    }
    fn is_float(self) -> bool {
        matches!(self, Self::Float(_) | Self::WeakFloat(_))
    }
    fn numeric_common(self, other: Self, span: Span) -> Result<Self, Diagnostic> {
        if self.is_float() || other.is_float() {
            let (a, da) = self.float(span)?;
            let (b, db) = other.float(span)?;
            return Ok(match (a, b) {
                (Some(FloatType::F64), _) | (_, Some(FloatType::F64)) => {
                    Self::Float(FloatType::F64)
                }
                (Some(ty), _) | (_, Some(ty)) => Self::Float(ty),
                _ => Self::WeakFloat(if da == FloatType::F64 || db == FloatType::F64 {
                    FloatType::F64
                } else {
                    FloatType::F32
                }),
            });
        }
        Ok(
            match self.number(span)?.common(other.number(span)?, span)? {
                NumberType::Literal => Self::IntegerLiteral,
                NumberType::Typed(ty) => Self::Integer(ty),
            },
        )
    }
}

/// A fully checked scalar expression retaining actual binding domains, not values.
/// Inactive names still require valid binding facts; their values are never read.
#[derive(Debug)]
pub struct DomainInference<'a> {
    expression: &'a Expression,
    domain: ScalarDomain,
    children: Vec<DomainInference<'a>>,
}
impl<'a> DomainInference<'a> {
    pub fn infer(
        expression: &'a Expression,
        mut lookup: impl FnMut(&NamePath, Span) -> Result<ScalarDomain, Diagnostic>,
    ) -> Result<Self, Diagnostic> {
        Self::infer_for_preparation(expression, &mut lookup)
            .map_err(ScalarInferenceError::into_diagnostic)
    }
    /// Bind readiness structurally; callers never classify rendered error messages.
    pub fn infer_for_preparation(
        expression: &'a Expression,
        mut lookup: impl FnMut(&NamePath, Span) -> Result<ScalarDomain, Diagnostic>,
    ) -> Result<Self, ScalarInferenceError> {
        let typed = std::cell::Cell::new(false);
        Self::infer_inner(expression, &mut lookup, &typed).map_err(|error| {
            if typed.get() {
                ScalarInferenceError::RequiresTypedExecution(error)
            } else {
                ScalarInferenceError::Diagnostic(error)
            }
        })
    }
    pub fn domain(&self) -> ScalarDomain {
        self.domain
    }
    fn infer_inner(
        expression: &'a Expression,
        lookup: &mut impl FnMut(&NamePath, Span) -> Result<ScalarDomain, Diagnostic>,
        typed: &std::cell::Cell<bool>,
    ) -> Result<Self, Diagnostic> {
        use ScalarDomain as D;
        let span = expression.span;
        let mut children = Vec::new();
        let mut child = |e: &'a Expression| -> Result<D, Diagnostic> {
            let inferred = Self::infer_inner(e, lookup, typed)?;
            let domain = inferred.domain;
            children.push(inferred);
            Ok(domain)
        };
        let domain = match &expression.kind {
            ExpressionKind::Integer(_) => D::IntegerLiteral,
            ExpressionKind::Character(_) => D::Integer(IntegerType::U8),
            ExpressionKind::Bool(_) => D::Bool,
            ExpressionKind::Float(FloatLiteral::Decimal(value)) => {
                D::WeakFloat(floats::default_decimal_type(value))
            }
            ExpressionKind::Float(FloatLiteral::Bits32(_)) => D::Float(FloatType::F32),
            ExpressionKind::Float(FloatLiteral::Bits64(_)) => D::Float(FloatType::F64),
            ExpressionKind::Name(root) => lookup(
                &NamePath {
                    root: *root,
                    members: Vec::new(),
                },
                span,
            )?,
            ExpressionKind::QualifiedName(path) => lookup(path, span)?,
            ExpressionKind::Unary(op, value) => {
                let value = child(value)?;
                match op {
                    UnaryOp::LogicalNot => D::Bool,
                    UnaryOp::Positive | UnaryOp::Negate if value.is_float() => value,
                    UnaryOp::Complement if value.is_float() => {
                        return Err(Diagnostic::new(
                            span,
                            "operator is not defined for floating-point constants",
                        ));
                    }
                    _ => {
                        value.number(span)?;
                        value
                    }
                }
            }
            ExpressionKind::Cast(mode, ty, value) => {
                if matches!(mode, CastMode::Force(_)) {
                    typed.set(true);
                    return Err(Diagnostic::new(
                        span,
                        "storage casts require target-layout VM evaluation",
                    ));
                }
                if *mode == CastMode::Truncate && *ty == ScalarType::Bool {
                    return Err(Diagnostic::new(
                        span,
                        "trunc cast to bool has no established source policy",
                    ));
                }
                let value = child(value)?;
                if *mode == CastMode::Truncate && value == D::Bool {
                    return Err(Diagnostic::new(
                        span,
                        "trunc cast from bool has no established source policy",
                    ));
                }
                match ty {
                    ScalarType::Bool => D::Bool,
                    ScalarType::Int(ty) => D::Integer(*ty),
                }
            }
            ExpressionKind::TypeCast {
                mode,
                ty,
                value,
            } => {
                let value = child(value)?;
                let TypeSyntax::Builtin(BuiltinType::Float(ty)) = ty else {
                    typed.set(true);
                    return Err(Diagnostic::new(
                        span,
                        "constant cast requires a builtin numeric type",
                    ));
                };
                if *mode == CastMode::Truncate {
                    return Err(Diagnostic::new(
                        span,
                        "cast,trunc targeting floating-point types has no established source policy",
                    ));
                }
                if matches!(mode, CastMode::Force(_)) {
                    typed.set(true);
                    return Err(Diagnostic::new(
                        span,
                        "force storage casts require target-bound typed constant evaluation",
                    ));
                }
                if !matches!(value, D::Integer(_)) {
                    value.float(span)?;
                }
                D::Float(*ty)
            }
            ExpressionKind::Binary(op, lhs, rhs) => {
                let lhs = child(lhs)?;
                let rhs = child(rhs)?;
                match Operator::from(*op) {
                    Operator::And | Operator::Or => D::Bool,
                    Operator::Equality(_) if lhs == D::Bool && rhs == D::Bool => D::Bool,
                    Operator::Relation(_) | Operator::Equality(_) => {
                        lhs.numeric_common(rhs, span)?;
                        D::Bool
                    }
                    Operator::Integer(op) => {
                        let common = lhs.numeric_common(rhs, span)?;
                        if common.is_float()
                            && !matches!(
                                op,
                                IntOp::Add
                                    | IntOp::Subtract
                                    | IntOp::Multiply
                                    | IntOp::Divide
                                    | IntOp::Remainder
                            )
                        {
                            return Err(Diagnostic::new(
                                span,
                                "operator is not defined for floating-point constants",
                            ));
                        }
                        common
                    }
                }
            }
            ExpressionKind::Conditional(value) => {
                child(&value.condition)?;
                let yes = child(&value.then_value)?;
                let no = value.else_value.as_ref().map(|e| child(e)).transpose()?;
                if yes.is_float() || no.is_some_and(D::is_float) {
                    yes.numeric_common(no.unwrap_or(D::IntegerLiteral), span)?
                } else if yes == D::Bool {
                    if no.is_some_and(|no| no != D::Bool) {
                        return Err(Diagnostic::new(
                            span,
                            "ifx branches require compatible types",
                        ));
                    }
                    D::Bool
                } else {
                    yes.numeric_common(no.unwrap_or(yes), span)?
                }
            }
            _ => {
                typed.set(true);
                // Unsupported forms share the existing binder's precise diagnostic.
                bind(expression, CheckMode::Enabled, &mut |_, span| {
                    Err(Diagnostic::new(span, "unreachable unsupported binding"))
                })?;
                return Err(Diagnostic::new(
                    span,
                    "this constant expression requires typed compile-time evaluation",
                ));
            }
        };
        Ok(Self {
            expression,
            domain,
            children,
        })
    }
    pub fn evaluate_paths(
        &self,
        overflow_check: CheckMode,
        mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
    ) -> Result<Value, Diagnostic> {
        finish(
            self.bind_selected(overflow_check, &mut lookup)?,
            self.expression.span,
        )
    }
    pub fn evaluate_float_paths(
        &self,
        target: FloatType,
        overflow_check: CheckMode,
        mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
    ) -> Result<FloatValue, Diagnostic> {
        let value = self
            .bind_selected(overflow_check, &mut lookup)?
            .float(self.expression.span)?;
        if value.ty == Some(FloatType::F64) && target == FloatType::F32 {
            return Err(Diagnostic::new(
                self.expression.span,
                "implicit floating-point conversion does not preserve the source width",
            ));
        }
        value.evaluate(Some(target), self.expression.span)
    }
    fn bind_selected(
        &self,
        overflow_check: CheckMode,
        lookup: &mut impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
    ) -> Result<Expr, Diagnostic> {
        let span = self.expression.span;
        let child = |index: usize, lookup: &mut _| {
            self.children[index].bind_selected(overflow_check, lookup)
        };
        let value = match &self.expression.kind {
            ExpressionKind::Name(root) => literal(lookup(
                &NamePath {
                    root: *root,
                    members: Vec::new(),
                },
                span,
            )?),
            ExpressionKind::QualifiedName(path) => literal(lookup(path, span)?),
            ExpressionKind::Conditional(value) => {
                if child(0, lookup)?.condition().evaluate()? {
                    child(1, lookup)?
                } else if value.else_value.is_some() {
                    child(2, lookup)?
                } else {
                    literal(match self.domain {
                        ScalarDomain::Bool => Value::Bool(false),
                        ScalarDomain::Integer(ty) => Value::Int(Integer::wrapping(ty, 0)),
                        _ => Value::Literal(0),
                    })
                }
            }
            ExpressionKind::Binary(op, _, _) => {
                let lhs = child(0, lookup)?;
                match Operator::from(*op) {
                    Operator::And if !lhs.clone().condition().evaluate()? => {
                        literal(Value::Bool(false))
                    }
                    Operator::Or if lhs.clone().condition().evaluate()? => {
                        literal(Value::Bool(true))
                    }
                    _ => bound_values::bind_binary(
                        *op,
                        lhs,
                        child(1, lookup)?,
                        overflow_check,
                        span,
                    )?,
                }
            }
            ExpressionKind::Unary(op, _) => {
                let value = child(0, lookup)?;
                if matches!(value, Expr::Float(_)) && *op != UnaryOp::LogicalNot {
                    floats::unary(op, value, span)?
                } else if *op == UnaryOp::LogicalNot {
                    Expr::Bool(BoolExpr::Not(Box::new(value.condition())))
                } else {
                    let value = value.number(span)?;
                    if *op == UnaryOp::Positive {
                        Expr::Number(value)
                    } else {
                        Expr::Number(NumberExpr {
                            ty: value.ty,
                            overflow_check,
                            kind: if *op == UnaryOp::Negate {
                                NumberKind::Negate(Box::new(value), span)
                            } else {
                                NumberKind::Complement(Box::new(value))
                            },
                        })
                    }
                }
            }
            ExpressionKind::TypeCast {
                mode,
                ty,
                ..
            } => floats::cast(ty, child(0, lookup)?, *mode, span)?,
            ExpressionKind::Cast(mode, ty, _) => {
                let value = child(0, lookup)?;
                match ty {
                    ScalarType::Bool => Expr::Bool(value.condition()),
                    ScalarType::Int(ty) => Expr::Number(NumberExpr {
                        ty: NumberType::Typed(*ty),
                        overflow_check,
                        kind: match value {
                            Expr::Number(value) => NumberKind::Cast(*mode, Box::new(value), span),
                            Expr::Bool(value) => NumberKind::FromBool(Box::new(value)),
                            Expr::Float(value) => {
                                NumberKind::FromFloat(*mode, Box::new(value), span)
                            }
                        },
                    }),
                }
            }
            _ => bind(self.expression, overflow_check, lookup)?,
        };
        if matches!(
            self.expression.kind,
            ExpressionKind::Name(_) | ExpressionKind::QualifiedName(_)
        ) && expression_domain(&value) != self.domain
        {
            return Err(Diagnostic::new(
                span,
                "scalar binding value does not match its inferred domain",
            ));
        }
        align(value, self.domain, span)
    }
}
fn expression_domain(value: &Expr) -> ScalarDomain {
    match value {
        Expr::Bool(_) => ScalarDomain::Bool,
        Expr::Number(value) => match value.ty {
            NumberType::Literal => ScalarDomain::IntegerLiteral,
            NumberType::Typed(ty) => ScalarDomain::Integer(ty),
        },
        Expr::Float(value) => match value.ty {
            Some(ty) => ScalarDomain::Float(ty),
            None => ScalarDomain::WeakFloat(value.default_type()),
        },
    }
}
fn align(value: Expr, domain: ScalarDomain, span: Span) -> Result<Expr, Diagnostic> {
    Ok(match domain {
        ScalarDomain::IntegerLiteral => Expr::Number(value.number(span)?),
        ScalarDomain::Integer(ty) => {
            Expr::Number(value.number(span)?.convert(NumberType::Typed(ty), span))
        }
        ScalarDomain::Bool => match value {
            Expr::Bool(_) => value,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "ifx branches require compatible types",
                ));
            }
        },
        ScalarDomain::Float(ty) => {
            let value = value.float(span)?;
            if value.ty == Some(ty) {
                Expr::Float(value)
            } else {
                Expr::Float(floats::widen(value, ty))
            }
        }
        ScalarDomain::WeakFloat(default) => {
            Expr::Float(floats::retain_default(value.float(span)?, default))
        }
    })
}
fn finish(value: Expr, span: Span) -> Result<Value, Diagnostic> {
    match value {
        Expr::Number(value) => value.evaluate(),
        Expr::Bool(value) => value.evaluate().map(Value::Bool),
        Expr::Float(value) => value.into_value(span),
    }
}
