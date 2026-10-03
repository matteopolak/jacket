//! Source-result checks use an already bound operator's actual signature.
use super::*;

impl Resolver<'_> {
    pub(crate) fn check_bound_call_result_use(
        &self,
        call: &Call,
        used: &[bool],
        offset: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let signature = self
            .contract_procedure_signature(call.procedure)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "bound operator has no checked source result signature",
                )
            })?;
        crate::result_obligations::check_result_use(&signature.results, used, offset, span)
    }
}
