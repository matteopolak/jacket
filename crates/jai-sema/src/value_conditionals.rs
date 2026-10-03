//! Preserve nominal values and evaluate only the selected runtime branch.
use super::*;

impl Resolver<'_> {
    pub(crate) fn value_conditional(
        &mut self,
        source: &syntax::ConditionalExpression,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let condition = self.condition_expression(&source.condition)?;
        let yes = self.expr_expected(&source.then_value, ty)?;
        let no = source
            .else_value
            .as_ref()
            .map(|value| self.expr_expected(value, ty))
            .transpose()?;
        self.value_conditional_pair(condition, yes, no, ty, span)
    }

    pub(crate) fn value_conditional_pair(
        &self,
        condition: BoolExpr,
        yes: Expr,
        no: Option<Expr>,
        ty: TypeId,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let then_value = self.coerce_value(yes, ty, span)?;
        let else_value = match no {
            Some(no) => self.coerce_value(no, ty, span)?,
            None => self
                .graph_scope
                .ok_or_else(|| {
                    Diagnostic::new(span, "typed conditional defaults require a type scope")
                })?
                .default_value(ty, self.types, span)?
                .into_expression(),
        };
        self.typed_value(
            ValueExpr::Conditional {
                ty,
                expression: Box::new(Conditional {
                    condition,
                    then_value,
                    else_value,
                }),
            },
            ty,
            span,
        )
    }
}
