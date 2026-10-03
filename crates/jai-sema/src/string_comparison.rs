//! String equality compares the complete byte sequence, including embedded NUL.
use super::*;

impl Resolver<'_> {
    pub(crate) fn is_string_expression(&self, value: &Expr) -> bool {
        matches!(value, Expr::Typed { ty, .. } if *ty == self.types.string())
    }

    pub(crate) fn string_binary(
        &self,
        op: BinaryOp,
        left: Expr,
        right: Expr,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let relation = match op {
            BinaryOp::Equal => Equality::Equal,
            BinaryOp::NotEqual => Equality::NotEqual,
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "strings support equality and inequality comparisons",
                ));
            }
        };
        let ty = self.types.string();
        let left = self.coerce_value(left, ty, span)?;
        let right = self.coerce_value(right, ty, span)?;
        Ok(Expr::Bool(BoolExpr::CompareStrings(
            relation,
            Box::new(left),
            Box::new(right),
        )))
    }
}
