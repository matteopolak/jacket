//! Pure constant evaluation with typed nodes and no host effects.
pub mod operators;
use jai_source::{Diagnostic, Span, Symbol};
use jai_syntax::{Expression, ExpressionKind, NamePath, UnaryOp};
pub use jai_types::{CastMode, Integer, IntegerType, ScalarType};
use operators::{Equality, IntOp, Operator, Relation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Literal(i128),
    Int(Integer),
    Bool(bool),
}
impl Value {
    pub fn ty(self) -> ScalarType {
        match self {
            Self::Literal(_) => ScalarType::Int(IntegerType::S64),
            Self::Int(n) => ScalarType::Int(n.ty()),
            Self::Bool(_) => ScalarType::Bool,
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
    match bind(expression, &mut lookup)? {
        Expr::Number(e) => e.evaluate(),
        Expr::Bool(e) => e.evaluate().map(Value::Bool),
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
        Operator::Integer(op) => Value::Literal(arithmetic(NumberType::Literal, op, a, b, span)?),
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
fn arithmetic(ty: NumberType, op: IntOp, a: i128, b: i128, span: Span) -> Result<i128, Diagnostic> {
    let invalid = || {
        Diagnostic::new(
            span,
            "invalid constant arithmetic: zero divisor, overflow or shift count",
        )
    };
    Ok(match op {
        IntOp::Add => match ty {
            NumberType::Literal => a.checked_add(b).ok_or_else(invalid)?,
            NumberType::Typed(_) => a.wrapping_add(b),
        },
        IntOp::Subtract => match ty {
            NumberType::Literal => a.checked_sub(b).ok_or_else(invalid)?,
            NumberType::Typed(_) => a.wrapping_sub(b),
        },
        IntOp::Multiply => match ty {
            NumberType::Literal => a.checked_mul(b).ok_or_else(invalid)?,
            NumberType::Typed(_) => a.wrapping_mul(b),
        },
        IntOp::BitAnd => a & b,
        IntOp::BitOr => a | b,
        IntOp::BitXor => a ^ b,
        IntOp::Divide | IntOp::Remainder => {
            if matches!(ty,NumberType::Typed(ty) if ty.signed() && a==ty.min() && b== -1) {
                return Err(invalid());
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
    })
}
#[derive(Clone, Copy, PartialEq, Eq)]
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
enum Expr {
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
        }
    }
}
struct NumberExpr {
    ty: NumberType,
    kind: NumberKind,
}
enum NumberKind {
    Literal(i128),
    Typed(Integer),
    FromBool(Box<BoolExpr>),
    Cast(CastMode, Box<NumberExpr>, Span),
    Negate(Box<NumberExpr>, Span),
    Complement(Box<NumberExpr>),
    Binary(IntOp, Box<NumberExpr>, Box<NumberExpr>, Span),
    Conditional(Box<Conditional<NumberExpr>>),
}
enum BoolExpr {
    Constant(bool),
    FromNumber(Box<NumberExpr>),
    Not(Box<BoolExpr>),
    CompareNumbers(Relation, Box<NumberExpr>, Box<NumberExpr>),
    CompareBools(Equality, Box<BoolExpr>, Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Conditional(Box<Conditional<BoolExpr>>),
}
struct Conditional<T> {
    condition: BoolExpr,
    then_value: T,
    else_value: T,
}
fn literal(value: Value) -> Expr {
    match value {
        Value::Literal(n) => Expr::Number(NumberExpr {
            ty: NumberType::Literal,
            kind: NumberKind::Literal(n),
        }),
        Value::Int(n) => Expr::Number(NumberExpr {
            ty: NumberType::Typed(n.ty()),
            kind: NumberKind::Typed(n),
        }),
        Value::Bool(b) => Expr::Bool(BoolExpr::Constant(b)),
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
            kind: NumberKind::Cast(CastMode::Checked, Box::new(self), span),
        }
    }
    fn evaluate(&self) -> Result<Value, Diagnostic> {
        let value = match &self.kind {
            NumberKind::Literal(n) => *n,
            NumberKind::Typed(n) => n.value(),
            NumberKind::FromBool(e) => i128::from(e.evaluate()?),
            NumberKind::Cast(mode, e, span) => {
                let n = e.evaluate()?.number(*span)?;
                if let NumberType::Typed(ty) = self.ty {
                    return match mode {
                        CastMode::Unchecked => Ok(Value::Int(Integer::wrapping(ty, n))),
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
                e.evaluate()?.number(*span)?.checked_neg().ok_or_else(|| {
                    Diagnostic::new(*span, "integer constant exceeds evaluator range")
                })?
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
    lookup: &mut impl FnMut(&NamePath, Span) -> Result<Value, Diagnostic>,
) -> Result<Expr, Diagnostic> {
    let span = expression.span;
    Ok(match &expression.kind {
        ExpressionKind::Integer(n) => literal(Value::Literal(*n)),
        ExpressionKind::Bool(b) => literal(Value::Bool(*b)),
        ExpressionKind::Name(name) => literal(lookup(
            &NamePath {
                root: *name,
                members: Vec::new(),
            },
            span,
        )?),
        ExpressionKind::QualifiedName(path) => literal(lookup(path, span)?),
        ExpressionKind::StructLiteral(_) | ExpressionKind::Member { .. } => {
            return Err(Diagnostic::new(
                span,
                "aggregate constant evaluation is not implemented",
            ));
        }
        ExpressionKind::Call(_, _) | ExpressionKind::QualifiedCall(_, _) => {
            return Err(Diagnostic::new(
                span,
                "procedure calls in constant expressions are not implemented yet",
            ));
        }
        ExpressionKind::Cast(mode, ty, e) => {
            let value = bind(e, lookup)?;
            match ty {
                ScalarType::Bool => Expr::Bool(value.condition()),
                ScalarType::Int(ty) => Expr::Number(NumberExpr {
                    ty: NumberType::Typed(*ty),
                    kind: match value {
                        Expr::Number(e) => NumberKind::Cast(*mode, Box::new(e), span),
                        Expr::Bool(e) => NumberKind::FromBool(Box::new(e)),
                    },
                }),
            }
        }
        ExpressionKind::Unary(op, e) => {
            let value = bind(e, lookup)?;
            match op {
                UnaryOp::LogicalNot => Expr::Bool(BoolExpr::Not(Box::new(value.condition()))),
                _ => {
                    let e = value.number(span)?;
                    let ty = e.ty;
                    Expr::Number(NumberExpr {
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
            let condition = bind(&e.condition, lookup)?.condition();
            let then_value = bind(&e.then_value, lookup)?;
            let else_value = e.else_value.as_ref().map(|e| bind(e, lookup)).transpose()?;
            match then_value {
                Expr::Number(yes) => {
                    let no = else_value.unwrap_or_else(|| literal(yes.ty.finish(0)));
                    let (ty, yes, no) = number_pair(Expr::Number(yes), no, span)?;
                    Expr::Number(NumberExpr {
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
            let lhs = bind(lhs, lookup)?;
            let rhs = bind(rhs, lookup)?;
            match Operator::from(*op) {
                Operator::Integer(op) => {
                    let (ty, lhs, rhs) = number_pair(lhs, rhs, span)?;
                    Expr::Number(NumberExpr {
                        ty,
                        kind: NumberKind::Binary(op, Box::new(lhs), Box::new(rhs), span),
                    })
                }
                Operator::Relation(op) => {
                    let (_, lhs, rhs) = number_pair(lhs, rhs, span)?;
                    Expr::Bool(BoolExpr::CompareNumbers(op, Box::new(lhs), Box::new(rhs)))
                }
                Operator::Equality(op) => match (lhs, rhs) {
                    (Expr::Bool(lhs), Expr::Bool(rhs)) => {
                        Expr::Bool(BoolExpr::CompareBools(op, Box::new(lhs), Box::new(rhs)))
                    }
                    (lhs, rhs) => {
                        let (_, lhs, rhs) = number_pair(lhs, rhs, span)?;
                        Expr::Bool(BoolExpr::CompareNumbers(
                            match op {
                                Equality::Equal => Relation::Equal,
                                Equality::NotEqual => Relation::NotEqual,
                            },
                            Box::new(lhs),
                            Box::new(rhs),
                        ))
                    }
                },
                Operator::And => Expr::Bool(BoolExpr::And(
                    Box::new(lhs.condition()),
                    Box::new(rhs.condition()),
                )),
                Operator::Or => Expr::Bool(BoolExpr::Or(
                    Box::new(lhs.condition()),
                    Box::new(rhs.condition()),
                )),
            }
        }
    })
}
impl BoolExpr {
    fn evaluate(&self) -> Result<bool, Diagnostic> {
        Ok(match self {
            Self::Constant(b) => *b,
            Self::FromNumber(e) => e.evaluate()?.number(Span::default())? != 0,
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
