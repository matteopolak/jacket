//! Pure constant evaluation with typed nodes and no host effects.
mod bound_values;
mod domains;
mod retained_metadata;
pub use retained_metadata::EvalRetainedMetadataError;
pub mod floats;
pub mod operators;
#[cfg(test)]
mod safety_checks_tests;
pub use bound_values::binary_values;
pub use domains::{DomainInference, ScalarDomain, ScalarInferenceError};
pub use floats::evaluate_float_paths;
use jai_source::{Diagnostic, Span, Symbol};
use jai_syntax::{Expression, ExpressionKind, NamePath, UnaryOp};
pub use jai_types::{CastMode, CheckMode, Integer, IntegerType, ScalarType};
use operators::{Equality, IntOp, Operator, Relation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Literal(i128),
    Int(Integer),
    Bool(bool),
    Float(jai_types::FloatValue),
    /// Exact bound decimal expression, rounded when a use supplies a float width.
    WeakFloat(std::sync::Arc<floats::WeakFloatValue>),
}
impl Value {
    /// Retain a bound weak expression's definition source for deferred errors.
    /// Contextual decimal conversion errors continue to use the materializing site.
    pub fn with_fallback_source(mut self, source: jai_source::SourceId) -> Self {
        if let Self::WeakFloat(value) = &mut self
            && !value.has_definition_source()
        {
            std::sync::Arc::make_mut(value).with_fallback_source(source);
        }
        self
    }
    pub fn type_id(&self, types: &dyn jai_types::TypeView) -> jai_types::TypeId {
        match self {
            Self::Float(value) => types.float(value.ty()),
            Self::WeakFloat(value) => types.float(value.default_type()),
            Self::Literal(_) => types.scalar(ScalarType::Int(IntegerType::S64)),
            Self::Int(value) => types.scalar(ScalarType::Int(value.ty())),
            Self::Bool(_) => types.scalar(ScalarType::Bool),
        }
    }
    pub fn scalar_type(&self) -> Option<ScalarType> {
        match self {
            Self::Literal(_) => Some(ScalarType::Int(IntegerType::S64)),
            Self::Int(n) => Some(ScalarType::Int(n.ty())),
            Self::Bool(_) => Some(ScalarType::Bool),
            Self::Float(_) | Self::WeakFloat(_) => None,
        }
    }
    pub fn zero(ty: ScalarType) -> Self {
        match ty {
            ScalarType::Int(ty) => Self::Int(Integer::wrapping(ty, 0)),
            ScalarType::Bool => Self::Bool(false),
        }
    }
    pub fn coerce(self, ty: ScalarType, span: Span) -> Result<Self, Diagnostic> {
        match (self, ty) {
            (Self::Bool(b), ScalarType::Bool) => Ok(Self::Bool(b)),
            (Self::Literal(n), ScalarType::Int(ty)) => {
                Integer::checked(ty, n).map(Self::Int).ok_or_else(|| {
                    Diagnostic::new(span, "integer constant is out of range for its target type")
                })
            }
            (Self::Int(n), ScalarType::Int(ty)) if ty.contains(n.ty()) => {
                Ok(Self::Int(Integer::wrapping(ty, n.value())))
            }
            _ => Err(Diagnostic::new(
                span,
                "implicit conversion does not preserve the source type's entire range",
            )),
        }
    }
    fn number(self, span: Span) -> Result<i128, Diagnostic> {
        match self {
            Self::Literal(n) => Ok(n),
            Self::Int(n) => Ok(n.value()),
            _ => Err(Diagnostic::new(span, "expected integer constant")),
        }
    }
}
/// Bind all operands, including unselected branches, before executing any arithmetic.
pub fn evaluate(
    expression: &Expression,
    mut lookup: impl FnMut(Symbol, Span) -> Result<Value, Diagnostic>,
) -> Result<Value, Diagnostic> {
    evaluate_paths(expression, |path, span| {
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "qualified constant requires a module scope",
            ));
        }
        lookup(path.root, span)
    })
}
/// Resolve complete namespace paths without combining spelling and declaration identity.
pub fn evaluate_paths(
    expression: &Expression,
    mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<Value, Diagnostic> {
    evaluate_paths_with_overflow_check(expression, CheckMode::Enabled, &mut lookup)
}
/// Pure source constants retain the same overflow decision as runtime operations.
pub fn evaluate_paths_with_overflow_check(
    expression: &Expression,
    overflow_check: CheckMode,
    mut lookup: impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<Value, Diagnostic> {
    match bind(expression, overflow_check, &mut lookup)? {
        Expr::Number(e) => e.evaluate(),
        Expr::Bool(e) => e.evaluate().map(Value::Bool),
        Expr::Float(e) => e.into_value(expression.span),
    }
}
/// Fold untyped literal operands without materializing them as s64 storage.
pub fn binary_literals(
    op: jai_syntax::BinaryOp,
    a: i128,
    b: i128,
    span: Span,
) -> Result<Value, Diagnostic> {
    Ok(match Operator::from(op) {
        Operator::Integer(op) => Value::Literal(arithmetic(
            NumberType::Literal,
            CheckMode::Enabled,
            op,
            a,
            b,
            span,
        )?),
        Operator::Relation(op) => Value::Bool(compare(op, a, b)),
        Operator::Equality(op) => Value::Bool(match op {
            Equality::Equal => a == b,
            Equality::NotEqual => a != b,
        }),
        Operator::And => Value::Bool(a != 0 && b != 0),
        Operator::Or => Value::Bool(a != 0 || b != 0),
    })
}
fn compare(op: Relation, a: i128, b: i128) -> bool {
    match op {
        Relation::Equal => a == b,
        Relation::NotEqual => a != b,
        Relation::Less => a < b,
        Relation::LessEqual => a <= b,
        Relation::Greater => a > b,
        Relation::GreaterEqual => a >= b,
    }
}
fn arithmetic(
    ty: NumberType,
    overflow_check: CheckMode,
    op: IntOp,
    a: i128,
    b: i128,
    span: Span,
) -> Result<i128, Diagnostic> {
    let invalid = || {
        Diagnostic::new(
            span,
            "invalid constant arithmetic: zero divisor, overflow or shift count",
        )
    };
    let value = match op {
        IntOp::Add => match ty {
            NumberType::Literal => a.checked_add(b).ok_or_else(invalid)?,
            NumberType::Typed(_) if overflow_check.enabled() => {
                a.checked_add(b).ok_or_else(invalid)?
            }
            NumberType::Typed(_) => a.wrapping_add(b),
        },
        IntOp::Subtract => match ty {
            NumberType::Literal => a.checked_sub(b).ok_or_else(invalid)?,
            NumberType::Typed(_) if overflow_check.enabled() => {
                a.checked_sub(b).ok_or_else(invalid)?
            }
            NumberType::Typed(_) => a.wrapping_sub(b),
        },
        IntOp::Multiply => match ty {
            NumberType::Literal => a.checked_mul(b).ok_or_else(invalid)?,
            NumberType::Typed(_) if overflow_check.enabled() => {
                a.checked_mul(b).ok_or_else(invalid)?
            }
            NumberType::Typed(_) => a.wrapping_mul(b),
        },
        IntOp::BitAnd => a & b,
        IntOp::BitOr => a | b,
        IntOp::BitXor => a ^ b,
        IntOp::Divide | IntOp::Remainder => {
            if matches!(ty,NumberType::Typed(ty) if ty.signed() && a==ty.min() && b== -1) {
                return if overflow_check.enabled() {
                    Err(invalid())
                } else {
                    Ok(if op == IntOp::Divide {
                        a
                    } else {
                        0
                    })
                };
            }
            if op == IntOp::Divide {
                a.checked_div(b)
            } else {
                a.checked_rem(b)
            }
            .ok_or_else(invalid)?
        }
        IntOp::ShiftLeft | IntOp::ShiftRight => {
            let width = match ty {
                NumberType::Literal => 64,
                NumberType::Typed(ty) => ty.bits(),
            };
            let count = u32::try_from(b)
                .ok()
                .filter(|n| *n < width)
                .ok_or_else(invalid)?;
            if op == IntOp::ShiftLeft {
                match ty {
                    NumberType::Literal => a.checked_mul(1i128 << count).ok_or_else(invalid)?,
                    NumberType::Typed(_) => a.wrapping_shl(count),
                }
            } else {
                a >> count
            }
        }
    };
    if overflow_check.enabled()
        && matches!(op, IntOp::Add | IntOp::Subtract | IntOp::Multiply)
        && let NumberType::Typed(ty) = ty
        && Integer::checked(ty, value).is_none()
    {
        return Err(Diagnostic::new(
            span,
            "integer constant arithmetic overflow",
        ));
    }
    Ok(value)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum NumberType {
    Literal,
    Typed(IntegerType),
}
impl NumberType {
    fn finish(self, value: i128) -> Value {
        match self {
            Self::Literal => Value::Literal(value),
            Self::Typed(ty) => Value::Int(Integer::wrapping(ty, value)),
        }
    }
    fn common(self, other: Self, span: Span) -> Result<Self, Diagnostic> {
        match (self, other) {
            (Self::Literal, t) | (t, Self::Literal) => Ok(t),
            (Self::Typed(a), Self::Typed(b)) => a
                .common(b)
                .map(Self::Typed)
                .ok_or_else(|| Diagnostic::new(span, "integer operands have incompatible ranges")),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr {
    Float(floats::FloatExpr),
    Number(NumberExpr),
    Bool(BoolExpr),
}
impl Expr {
    fn number(self, span: Span) -> Result<NumberExpr, Diagnostic> {
        match self {
            Self::Number(e) => Ok(e),
            _ => Err(Diagnostic::new(span, "expected integer constant")),
        }
    }
    fn condition(self) -> BoolExpr {
        match self {
            Self::Bool(e) => e,
            Self::Number(e) => BoolExpr::FromNumber(Box::new(e)),
            Self::Float(e) => BoolExpr::FromFloat(Box::new(e)),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct NumberExpr {
    ty: NumberType,
    overflow_check: CheckMode,
    kind: NumberKind,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum NumberKind {
    Literal(i128),
    Typed(Integer),
    FromBool(Box<BoolExpr>),
    FromFloat(CastMode, Box<floats::FloatExpr>, Span),
    Cast(CastMode, Box<NumberExpr>, Span),
    Negate(Box<NumberExpr>, Span),
    Complement(Box<NumberExpr>),
    Binary(IntOp, Box<NumberExpr>, Box<NumberExpr>, Span),
    Conditional(Box<Conditional<NumberExpr>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum BoolExpr {
    Constant(bool),
    FromNumber(Box<NumberExpr>),
    FromFloat(Box<floats::FloatExpr>),
    CompareFloats(
        Relation,
        Box<floats::FloatExpr>,
        Box<floats::FloatExpr>,
        Span,
    ),
    Not(Box<BoolExpr>),
    CompareNumbers(Relation, Box<NumberExpr>, Box<NumberExpr>),
    CompareBools(Equality, Box<BoolExpr>, Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Conditional(Box<Conditional<BoolExpr>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Conditional<T> {
    condition: BoolExpr,
    then_value: T,
    else_value: T,
}
fn literal(value: Value) -> Expr {
    match value {
        Value::Literal(n) => Expr::Number(NumberExpr {
            overflow_check: CheckMode::Disabled,
            ty: NumberType::Literal,
            kind: NumberKind::Literal(n),
        }),
        Value::Int(n) => Expr::Number(NumberExpr {
            overflow_check: CheckMode::Disabled,
            ty: NumberType::Typed(n.ty()),
            kind: NumberKind::Typed(n),
        }),
        Value::Bool(b) => Expr::Bool(BoolExpr::Constant(b)),
        Value::Float(value) => Expr::Float(floats::FloatExpr::constant(value)),
        Value::WeakFloat(value) => Expr::Float(floats::FloatExpr::bound(value)),
    }
}
impl NumberExpr {
    fn convert(self, ty: NumberType, span: Span) -> Self {
        if self.ty == ty {
            return self;
        }
        // Common-type selection has already proved range preservation for typed operands.
        Self {
            ty,
            overflow_check: self.overflow_check,
            kind: NumberKind::Cast(CastMode::Checked, Box::new(self), span),
        }
    }
    fn evaluate(&self) -> Result<Value, Diagnostic> {
        let value = match &self.kind {
            NumberKind::Literal(n) => *n,
            NumberKind::Typed(n) => n.value(),
            NumberKind::FromBool(e) => i128::from(e.evaluate()?),
            NumberKind::FromFloat(mode, value, span) => {
                let NumberType::Typed(ty) = self.ty else {
                    return Err(Diagnostic::new(
                        *span,
                        "float conversion requires an integer target",
                    ));
                };
                return floats::to_integer(value, ty, *mode, *span).map(Value::Int);
            }
            NumberKind::Cast(mode, e, span) => {
                let n = e.evaluate()?.number(*span)?;
                if let NumberType::Typed(ty) = self.ty {
                    return match mode {
                        CastMode::Force(_) => Err(Diagnostic::new(
                            *span,
                            "storage casts require target-layout VM evaluation",
                        )),
                        CastMode::Unchecked | CastMode::Truncate => {
                            Ok(Value::Int(Integer::wrapping(ty, n)))
                        }
                        CastMode::Checked => {
                            Integer::checked(ty, n).map(Value::Int).ok_or_else(|| {
                                Diagnostic::new(*span, "checked integer cast is out of range")
                            })
                        }
                    };
                }
                n
            }
            NumberKind::Negate(e, span) => {
                let value = e.evaluate()?.number(*span)?.checked_neg().ok_or_else(|| {
                    Diagnostic::new(*span, "integer constant exceeds evaluator range")
                })?;
                if self.overflow_check.enabled()
                    && let NumberType::Typed(ty) = self.ty
                    && Integer::checked(ty, value).is_none()
                {
                    return Err(Diagnostic::new(
                        *span,
                        "integer constant arithmetic overflow",
                    ));
                }
                value
            }
            NumberKind::Complement(e) => !e.evaluate()?.number(Span::default())?,
            NumberKind::Conditional(e) => {
                return if e.condition.evaluate()? {
                    e.then_value.evaluate()
                } else {
                    e.else_value.evaluate()
                };
            }
            NumberKind::Binary(op, lhs, rhs, span) => arithmetic(
                self.ty,
                self.overflow_check,
                *op,
                lhs.evaluate()?.number(*span)?,
                rhs.evaluate()?.number(*span)?,
                *span,
            )?,
        };
        Ok(self.ty.finish(value))
    }
}
fn number_pair(
    lhs: Expr,
    rhs: Expr,
    span: Span,
) -> Result<(NumberType, NumberExpr, NumberExpr), Diagnostic> {
    let lhs = lhs.number(span)?;
    let rhs = rhs.number(span)?;
    let ty = lhs.ty.common(rhs.ty, span)?;
    Ok((ty, lhs.convert(ty, span), rhs.convert(ty, span)))
}
fn bind(
    expression: &Expression,
    overflow_check: CheckMode,
    lookup: &mut impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<Expr, Diagnostic> {
    let span = expression.span;
    Ok(match &expression.kind {
        ExpressionKind::Integer(n) => literal(Value::Literal(*n)),
        ExpressionKind::Character(n) => literal(Value::Int(Integer::wrapping(
            IntegerType::U8,
            i128::from(*n),
        ))),
        ExpressionKind::Bool(b) => literal(Value::Bool(*b)),
        ExpressionKind::Name(name) => literal(lookup(
            &NamePath {
                root: *name,
                members: Vec::new(),
            },
            span,
        )?),
        ExpressionKind::QualifiedName(path) => literal(lookup(path, span)?),
        ExpressionKind::StructLiteral(_)
        | ExpressionKind::PositionalStructLiteral(_)
        | ExpressionKind::Member {
            ..
        } => {
            return Err(Diagnostic::new(
                span,
                "aggregate constant evaluation is not implemented",
            ));
        }
        ExpressionKind::Float(value) => Expr::Float(floats::FloatExpr::literal(value)),
        ExpressionKind::InferredCast {
            ..
        } => {
            return Err(Diagnostic::new(
                span,
                "xx cast requires a destination type from its context",
            ));
        }
        ExpressionKind::TypeCast {
            mode,
            ty,
            value,
        } => floats::cast(ty, bind(value, overflow_check, lookup)?, *mode, span)?,
        ExpressionKind::BakeArguments(_) => {
            return Err(Diagnostic::new(
                span,
                "#bake_arguments requires checked callable source preparation",
            ));
        }
        ExpressionKind::Code(_)
        | ExpressionKind::ShortLambda(_)
        | ExpressionKind::AnonymousProcedure(_)
        | ExpressionKind::Insert(_)
        | ExpressionKind::CompileTime(_)
        | ExpressionKind::CompileTimePredicate
        | ExpressionKind::CompileVariable(_)
        | ExpressionKind::Type(_)
        | ExpressionKind::String(_)
        | ExpressionKind::HereString(_)
        | ExpressionKind::Null
        | ExpressionKind::Uninitialized
        | ExpressionKind::CallHint {
            ..
        }
        | ExpressionKind::IndirectCall {
            ..
        }
        | ExpressionKind::ContextCall {
            ..
        }
        | ExpressionKind::InferredMember(_)
        | ExpressionKind::Context
        | ExpressionKind::CallerLocation
        | ExpressionKind::SourceLocation
        | ExpressionKind::SourceFile
        | ExpressionKind::SourceFilepath
        | ExpressionKind::SourceLine
        | ExpressionKind::AddressOf(_)
        | ExpressionKind::Dereference(_)
        | ExpressionKind::Index {
            ..
        }
        | ExpressionKind::ArrayLiteral(_)
        | ExpressionKind::TypeQuery {
            ..
        } => {
            return Err(Diagnostic::new(
                span,
                "this constant expression requires typed compile-time evaluation",
            ));
        }
        ExpressionKind::Call(_, _) | ExpressionKind::QualifiedCall(_, _) => {
            return Err(Diagnostic::new(
                span,
                "procedure calls in constant expressions are not implemented yet",
            ));
        }
        ExpressionKind::Cast(mode, ty, e) => {
            if matches!(mode, CastMode::Force(_)) {
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
            let value = bind(e, overflow_check, lookup)?;
            if *mode == CastMode::Truncate && matches!(value, Expr::Bool(_)) {
                return Err(Diagnostic::new(
                    span,
                    "trunc cast from bool has no established source policy",
                ));
            }
            match ty {
                ScalarType::Bool => Expr::Bool(value.condition()),
                ScalarType::Int(ty) => Expr::Number(NumberExpr {
                    overflow_check,
                    ty: NumberType::Typed(*ty),
                    kind: match value {
                        Expr::Number(e) => NumberKind::Cast(*mode, Box::new(e), span),
                        Expr::Bool(e) => NumberKind::FromBool(Box::new(e)),
                        Expr::Float(e) => NumberKind::FromFloat(*mode, Box::new(e), span),
                    },
                }),
            }
        }
        ExpressionKind::Unary(op, e) => {
            let value = bind(e, overflow_check, lookup)?;
            if matches!(value, Expr::Float(_)) && *op != UnaryOp::LogicalNot {
                return floats::unary(op, value, span);
            }
            match op {
                UnaryOp::LogicalNot => Expr::Bool(BoolExpr::Not(Box::new(value.condition()))),
                _ => {
                    let e = value.number(span)?;
                    let ty = e.ty;
                    Expr::Number(NumberExpr {
                        overflow_check,
                        ty,
                        kind: match op {
                            UnaryOp::Positive => return Ok(Expr::Number(e)),
                            UnaryOp::Negate => NumberKind::Negate(Box::new(e), span),
                            UnaryOp::Complement => NumberKind::Complement(Box::new(e)),
                            _ => unreachable!(),
                        },
                    })
                }
            }
        }
        ExpressionKind::Conditional(e) => {
            let condition = bind(&e.condition, overflow_check, lookup)?.condition();
            let then_value = bind(&e.then_value, overflow_check, lookup)?;
            let else_value = e
                .else_value
                .as_ref()
                .map(|e| bind(e, overflow_check, lookup))
                .transpose()?;
            if matches!(then_value, Expr::Float(_)) || matches!(else_value, Some(Expr::Float(_))) {
                return floats::conditional(condition, then_value, else_value, span);
            }
            match then_value {
                Expr::Float(_) => unreachable!("handled float conditional"),
                Expr::Number(yes) => {
                    let no = else_value.unwrap_or_else(|| literal(yes.ty.finish(0)));
                    let (ty, yes, no) = number_pair(Expr::Number(yes), no, span)?;
                    Expr::Number(NumberExpr {
                        overflow_check,
                        ty,
                        kind: NumberKind::Conditional(Box::new(Conditional {
                            condition,
                            then_value: yes,
                            else_value: no,
                        })),
                    })
                }
                Expr::Bool(yes) => {
                    let no = match else_value {
                        None => BoolExpr::Constant(false),
                        Some(Expr::Bool(e)) => e,
                        _ => {
                            return Err(Diagnostic::new(
                                span,
                                "ifx branches require compatible types",
                            ));
                        }
                    };
                    Expr::Bool(BoolExpr::Conditional(Box::new(Conditional {
                        condition,
                        then_value: yes,
                        else_value: no,
                    })))
                }
            }
        }
        ExpressionKind::Binary(op, lhs, rhs) => {
            let lhs = bind(lhs, overflow_check, lookup)?;
            let rhs = bind(rhs, overflow_check, lookup)?;
            bound_values::bind_binary(*op, lhs, rhs, overflow_check, span)?
        }
    })
}
impl BoolExpr {
    fn evaluate(&self) -> Result<bool, Diagnostic> {
        Ok(match self {
            Self::Constant(b) => *b,
            Self::FromNumber(e) => e.evaluate()?.number(Span::default())? != 0,
            Self::FromFloat(e) => e.evaluate(None, Span::default())?.to_f64() != 0.0,
            Self::CompareFloats(relation, a, b, span) => {
                let ty = match (a.ty, b.ty) {
                    (Some(jai_types::FloatType::F64), _) | (_, Some(jai_types::FloatType::F64)) => {
                        Some(jai_types::FloatType::F64)
                    }
                    (Some(ty), _) | (_, Some(ty)) => Some(ty),
                    _ => None,
                };
                let ty = ty.unwrap_or_else(|| floats::common_default(a, b));
                let a = a.evaluate(Some(ty), *span)?;
                let b = b.evaluate(Some(ty), *span)?;
                a.compare(*relation, b)
                    .map_err(|error| Diagnostic::new(*span, error.to_string()))?
            }
            Self::Not(e) => !e.evaluate()?,
            Self::And(a, b) => a.evaluate()? && b.evaluate()?,
            Self::Or(a, b) => a.evaluate()? || b.evaluate()?,
            Self::Conditional(e) => {
                if e.condition.evaluate()? {
                    e.then_value.evaluate()?
                } else {
                    e.else_value.evaluate()?
                }
            }
            Self::CompareNumbers(op, a, b) => compare(
                *op,
                a.evaluate()?.number(Span::default())?,
                b.evaluate()?.number(Span::default())?,
            ),
            Self::CompareBools(op, a, b) => {
                let a = a.evaluate()?;
                let b = b.evaluate()?;
                match op {
                    Equality::Equal => a == b,
                    Equality::NotEqual => a != b,
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run(text: &str) -> Result<Value, Diagnostic> {
        let module = jai_syntax::parse(&format!("value :: {text}; main :: () {{}}")).unwrap();
        evaluate(&module.constants()[0].initializer, |_, span| {
            Err(Diagnostic::new(span, "unknown constant"))
        })
    }
    #[test]
    fn integer_operators_and_casts() {
        for (expression, expected) in [
            ("1 + 2 * 3", 7),
            ("42 / 7 + 9 % 4", 7),
            ("(0xff & 0x3f) ^ 0x15", 42),
            ("(3 << 4) >> 1", 24),
            ("-8 >> 2", -2),
            ("~0", -1),
            ("+42", 42),
            ("-9223372036854775808", i128::from(i64::MIN)),
        ] {
            assert_eq!(
                run(expression).unwrap(),
                Value::Literal(expected),
                "{expression}"
            );
        }
    }
    #[test]
    fn truncation_preserves_integer_bits_without_enabling_float_or_bool_policy() {
        assert_eq!(
            run("cast,trunc(u8)256").unwrap(),
            Value::Int(Integer::wrapping(IntegerType::U8, 0))
        );
        assert_eq!(
            run("cast(u32,-32,trunc)").unwrap(),
            Value::Int(Integer::wrapping(IntegerType::U32, -32))
        );
        assert!(run("cast(u8)256").is_err());
        for expression in ["cast,trunc(bool)2", "cast,trunc(u8)true"] {
            assert!(run(expression).is_err(), "{expression}");
        }
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert(
            "trunc-float.jai".into(),
            "VALUE::cast,trunc(u8)42.5;".into(),
        );
        let file = jai_syntax::parse_file(
            sources.get(id).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let jai_syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let jai_syntax::FileDeclarationKind::Constant(value) = &declaration.kind else {
            panic!()
        };
        assert!(evaluate(&value.initializer, |_, _| panic!("no lookup")).is_err());
    }
    #[test]
    fn predicates_and_truthiness() {
        for expression in [
            "1 < 2",
            "2 <= 2",
            "3 > 1",
            "3 >= 3",
            "4 == 4",
            "4 != 5",
            "true != false",
            "!0",
            "!!-3",
            "cast(bool) 7",
        ] {
            assert_eq!(run(expression).unwrap(), Value::Bool(true), "{expression}");
        }
    }
    #[test]
    fn skipped_operands_are_bound_but_not_executed() {
        assert_eq!(run("false && (1 / 0)").unwrap(), Value::Bool(false));
        assert_eq!(run("true || (1 << 64)").unwrap(), Value::Bool(true));
        for expression in ["false && missing", "true || (false + 2)", "false && f()"] {
            assert!(run(expression).is_err(), "{expression}");
        }
    }
    #[test]
    fn invalid_arithmetic_and_scalar_mismatches_are_diagnostics() {
        for expression in [
            "1 / 0",
            "1 % 0",
            "1 << -1",
            "1 >> 64",
            "cast(int) -9223372036854775808 / cast(int) -1",
            "true + false",
            "1 == true",
            "false < true",
        ] {
            assert!(run(expression).is_err(), "{expression}");
        }
    }
    #[test]
    fn conditional_constants_bind_both_arms_and_execute_only_one() {
        for (source, expected) in [
            ("ifx true then 42 else 1 / 0", Value::Literal(42)),
            ("ifx false 1 / 0 else 42", Value::Literal(42)),
            ("ifx 0 then 42", Value::Literal(0)),
            ("ifx true then true else false", Value::Bool(true)),
            ("ifx false then true", Value::Bool(false)),
            (
                "ifx true then ifx false then 1 else 42 else 0",
                Value::Literal(42),
            ),
        ] {
            assert_eq!(run(source).unwrap(), expected, "{source}");
        }
        for source in [
            "ifx true then 42 else missing",
            "ifx false then true else 42",
            "ifx true then 42 else f()",
        ] {
            assert!(run(source).is_err(), "{source}");
        }
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;
    use jai_source::{SourceMap, Symbols};
    use jai_syntax::{FileDeclarationKind, FileItem};
    #[test]
    fn qualified_names_are_bound_even_when_execution_skips_the_branch() {
        let mut sources = SourceMap::default();
        let source = sources.insert(
            "fixture.jai".into(),
            "VALUE :: true || Math.missing;".into(),
        );
        let mut symbols = Symbols::default();
        let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!()
        };
        let mut seen = Vec::new();
        let result = evaluate_paths(&constant.initializer, |path, span| {
            seen.push((path.clone(), span));
            Err(Diagnostic::new(span, "unknown module member"))
        });
        assert!(result.is_err());
        assert_eq!(seen[0].0.root, symbols.find("Math").unwrap());
        assert_eq!(seen[0].0.members, vec![symbols.find("missing").unwrap()]);
        assert!(
            evaluate(&constant.initializer, |_, _| panic!(
                "qualified lookup cannot be truncated"
            ))
            .is_err()
        );
    }
}
