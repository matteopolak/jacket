//! Fold already bound scalar facts with the source evaluator's arithmetic rules.
use super::*;

/// Use resolved child values without reinterpreting their original source syntax.
pub fn binary_values(
    operation: jai_syntax::BinaryOp,
    lhs: Value,
    rhs: Value,
    overflow_check: CheckMode,
    span: Span,
) -> Result<Value, Diagnostic> {
    match bind_binary(operation, literal(lhs), literal(rhs), overflow_check, span)? {
        Expr::Number(value) => value.evaluate(),
        Expr::Bool(value) => value.evaluate().map(Value::Bool),
        Expr::Float(value) => value.into_value(span),
    }
}

pub(super) fn bind_binary(
    operation: jai_syntax::BinaryOp,
    lhs: Expr,
    rhs: Expr,
    overflow_check: CheckMode,
    span: Span,
) -> Result<Expr, Diagnostic> {
    if (matches!(lhs, Expr::Float(_)) || matches!(rhs, Expr::Float(_)))
        && !matches!(Operator::from(operation), Operator::And | Operator::Or)
    {
        return floats::binary(operation, lhs, rhs, span);
    }
    Ok(match Operator::from(operation) {
        Operator::Integer(operation) => {
            let (ty, lhs, rhs) = number_pair(lhs, rhs, span)?;
            Expr::Number(NumberExpr {
                overflow_check,
                ty,
                kind: NumberKind::Binary(operation, Box::new(lhs), Box::new(rhs), span),
            })
        }
        Operator::Relation(operation) => {
            let (_, lhs, rhs) = number_pair(lhs, rhs, span)?;
            Expr::Bool(BoolExpr::CompareNumbers(
                operation,
                Box::new(lhs),
                Box::new(rhs),
            ))
        }
        Operator::Equality(operation) => match (lhs, rhs) {
            (Expr::Bool(lhs), Expr::Bool(rhs)) => Expr::Bool(BoolExpr::CompareBools(
                operation,
                Box::new(lhs),
                Box::new(rhs),
            )),
            (lhs, rhs) => {
                let (_, lhs, rhs) = number_pair(lhs, rhs, span)?;
                Expr::Bool(BoolExpr::CompareNumbers(
                    match operation {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_syntax::BinaryOp;
    use jai_types::FloatValue;

    #[test]
    fn bound_facts_preserve_literal_and_integer_widths() {
        let span = Span::new(7, 12);
        assert_eq!(
            binary_values(
                BinaryOp::Add,
                Value::Literal(7),
                Value::Literal(14),
                CheckMode::Enabled,
                span
            )
            .unwrap(),
            Value::Literal(21)
        );
        let narrow = Value::Int(Integer::checked(IntegerType::U8, 250).unwrap());
        let error = binary_values(
            BinaryOp::Add,
            narrow.clone(),
            Value::Literal(6),
            CheckMode::Enabled,
            span,
        )
        .unwrap_err();
        assert_eq!(error.span, span);
        assert!(error.message.contains("overflow"));
        assert_eq!(
            binary_values(
                BinaryOp::Add,
                narrow,
                Value::Literal(6),
                CheckMode::Disabled,
                span
            )
            .unwrap(),
            Value::Int(Integer::wrapping(IntegerType::U8, 0))
        );
        assert!(
            binary_values(
                BinaryOp::Add,
                Value::Bool(true),
                Value::Bool(false),
                CheckMode::Enabled,
                span
            )
            .is_err()
        );
    }

    #[test]
    fn bound_floats_use_the_existing_common_width() {
        assert_eq!(
            binary_values(
                BinaryOp::Add,
                Value::Float(FloatValue::F32(1.0_f32.to_bits())),
                Value::Float(FloatValue::F64(2.0_f64.to_bits())),
                CheckMode::Enabled,
                Span::default(),
            )
            .unwrap(),
            Value::Float(FloatValue::F64(3.0_f64.to_bits()))
        );
    }
}
