//! A partial callable policy belongs to its actual checked producer binding.
use super::*;

impl Resolver<'_> {
    pub(crate) fn capture_baked_wrapper_contract(
        &mut self,
        binding: ExpressionBindingId,
        checked: &ValueExpr,
        contract: ValueContract,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.validate_expression_contract_producer(binding, span)?;
        let ValueExpr::ProcedureValue {
            procedure,
            ty,
        } = checked
        else {
            return Err(Diagnostic::new(
                span,
                "partial contract requires its actual procedure producer",
            ));
        };
        let signature = self
            .contract_procedure_signature(*procedure)
            .ok_or_else(|| {
                Diagnostic::new(span, "partial producer has no checked source signature")
            })?;
        if signature.ty != *ty || contract.ty != *ty {
            return Err(Diagnostic::new(
                span,
                "partial contract differs from its checked producer",
            ));
        }
        self.publish_expression_contract(binding, checked, Some(contract), span)
    }
}
