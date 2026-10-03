//! A selected call retains its checked source proof under its real producer ID.
use super::*;

impl Resolver<'_> {
    pub(crate) fn capture_selected_call_result_contract(
        &mut self,
        binding: ExpressionBindingId,
        checked: &ValueExpr,
        contract: Option<ValueContract>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.validate_expression_contract_producer(binding, span)?;
        let ValueExpr::Call { call, ty } = checked else {
            return Err(Diagnostic::new(
                span,
                "selected call contract requires its actual call producer",
            ));
        };
        let signature = self
            .contract_procedure_signature(call.procedure)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "selected call contract lacks its checked source signature",
                )
            })?;
        if !matches!(signature.results.as_slice(), [result] if result.ty == *ty) {
            return Err(Diagnostic::new(
                span,
                "selected call capture requires one checked result of its producer type",
            ));
        }
        self.publish_expression_contract(binding, checked, contract, span)
    }
}
