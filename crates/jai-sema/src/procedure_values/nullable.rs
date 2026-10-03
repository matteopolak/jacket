//! Nullable procedure identity stays separate from data-pointer arithmetic.
use super::*;
use jai_types::{TypeKind, TypeView};

impl Expr {
    pub(crate) fn procedure_type(&self, types: &dyn TypeView) -> Option<TypeId> {
        let Self::Typed {
            ty, ..
        } = self
        else {
            return None;
        };
        matches!(types.kind(*ty), Ok(TypeKind::Procedure(_))).then_some(*ty)
    }
}

impl Resolver<'_> {
    pub(crate) fn procedure_binary(
        &self,
        operator: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        match Operator::from(operator) {
            Operator::And => Ok(Expr::Bool(BoolExpr::And(
                Box::new(left.condition(span, self.types)?),
                Box::new(right.condition(span, self.types)?),
            ))),
            Operator::Or => Ok(Expr::Bool(BoolExpr::Or(
                Box::new(left.condition(span, self.types)?),
                Box::new(right.condition(span, self.types)?),
            ))),
            Operator::Equality(operation) => {
                let left_type = left.procedure_type(self.types);
                let right_type = right.procedure_type(self.types);
                let ty = left_type
                    .or(right_type)
                    .expect("procedure equality is dispatched only for a procedure operand");
                if left_type.is_some_and(|value| value != ty)
                    || right_type.is_some_and(|value| value != ty)
                {
                    return Err(Diagnostic::new(
                        span,
                        "procedure equality requires the same canonical signature",
                    ));
                }
                let value = |expression: Expr| match expression {
                    Expr::Null => Ok(ValueExpr::Zero(ty)),
                    Expr::Typed {
                        ty: actual,
                        value,
                    } if actual == ty => Ok(value),
                    _ => Err(Diagnostic::new(
                        span,
                        "procedure equality requires a matching procedure value or null",
                    )),
                };
                Ok(Expr::Bool(BoolExpr::ComparePointers(
                    operation,
                    Box::new(value(left)?),
                    Box::new(value(right)?),
                )))
            }
            _ => Err(Diagnostic::new(
                span,
                "procedure values support equality and boolean operations, not address arithmetic",
            )),
        }
    }
}
