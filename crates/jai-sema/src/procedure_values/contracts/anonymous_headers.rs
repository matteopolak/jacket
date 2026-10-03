//! ID-free callable contracts retain the actual source result annotations.
use super::*;
impl Resolver<'_> {
    pub(crate) fn preview_source_header_contract(
        &mut self,
        header: &crate::local_declarations::CheckedSourceHeader,
        source: &syntax::CallableHeaderSyntax,
        span: Span,
    ) -> Result<ValueContract, Diagnostic> {
        let results = header
            .result_sources(&source.results, span)?
            .iter()
            .zip(&header.results)
            .map(|(source, result)| match &source.binding {
                syntax::ResultBinding::Typed {
                    ty, ..
                } => self.annotation_value_contract(result.ty, ty, source.span),
                syntax::ResultBinding::InferredDefault(expression) => {
                    self.preview_expected_callback_contract(expression, result.ty, source.span)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ValueContract {
            ty: header.ty,
            kind: ContractKind::Callable {
                metadata: header.callback_metadata(),
                results,
                source: None,
            },
        })
    }
}
