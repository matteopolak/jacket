//! Partial expressions retain their own checked callback bindings.
use super::*;
use crate::procedure_values::baked_arguments::{BakedProcedureArgument, BakedProcedureTarget};

impl Resolver<'_> {
    pub(crate) fn baked_wrapper_result_contracts(
        &mut self,
        target: &BakedProcedureTarget,
        arguments: &[(ParameterId, ValueExpr)],
        bound: &[BakedProcedureArgument],
        span: Span,
    ) -> Result<Vec<Option<ValueContract>>, Diagnostic> {
        let Some(scope) = self.graph_scope else {
            if bound.iter().any(|argument| argument.contract.is_some()) {
                return Err(Diagnostic::new(
                    span,
                    "partial lexical callback contract needs its original source adapter",
                ));
            }
            return self.procedure_result_contracts(target.signature.id, span);
        };
        let Some((_, parameters, _)) = scope.callback_contract_header(target.signature.id) else {
            if bound.iter().any(|argument| argument.contract.is_some()) {
                return Err(Diagnostic::new(
                    span,
                    "partial callback contract has no retained original source header",
                ));
            }
            return self.procedure_result_contracts(target.signature.id, span);
        };
        let mut overrides = HashMap::new();
        for argument in bound {
            let parameter = target
                .signature
                .parameters
                .get(argument.formal)
                .ok_or_else(|| {
                    Diagnostic::at_source(
                        argument.source,
                        "partial contract lost its checked formal",
                    )
                })?;
            let source = parameters.get(argument.source_formal).ok_or_else(|| {
                Diagnostic::at_source(argument.source, "partial contract lost its source ordinal")
            })?;
            if source.name != parameter.name
                || argument
                    .contract
                    .as_ref()
                    .is_some_and(|contract| contract.ty != parameter.ty)
            {
                return Err(Diagnostic::at_source(
                    argument.source,
                    "partial contract differs from its original checked formal",
                ));
            }
            if overrides
                .insert(source.name, argument.contract.clone())
                .is_some()
            {
                return Err(Diagnostic::at_source(
                    argument.source,
                    "partial callback formal supplied twice",
                ));
            }
        }
        self.call_result_contracts_for_source_overrides(
            target.signature.id,
            arguments,
            target.source_arguments.as_deref(),
            Some(&overrides),
            span,
            0,
        )
    }

    /// Publish a source proof under the actual constant producer, never a
    /// fabricated procedure identity or a global policy for all equivalent uses.
    pub(crate) fn bind_baked_wrapper_contract(
        &mut self,
        signature: &Signature,
        metadata: CallbackSignature,
        results: Vec<Option<ValueContract>>,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        if metadata.ty != signature.ty
            || metadata.results.len() != signature.results.len()
            || metadata
                .results
                .iter()
                .zip(&signature.results)
                .any(|(policy, checked)| policy.ty != checked.ty)
            || results.len() != signature.results.len()
            || results
                .iter()
                .zip(&signature.results)
                .any(|(contract, checked)| {
                    contract
                        .as_ref()
                        .is_some_and(|contract| contract.ty != checked.ty)
                })
        {
            return Err(Diagnostic::new(
                span,
                "partial result proof differs from its checked signature",
            ));
        }
        let producer = ValueExpr::ProcedureValue {
            procedure: signature.id,
            ty: signature.ty,
        };
        let contract = ValueContract {
            ty: signature.ty,
            kind: ContractKind::Callable {
                metadata,
                results,
                source: Some(signature.id),
            },
        };
        let binding = self.allocate_expression_binding(span)?;
        self.capture_baked_wrapper_contract(binding, &producer, contract, span)?;
        Ok(ValueExpr::Bind {
            bindings: vec![(binding, producer)],
            body: Box::new(ValueExpr::Bound {
                binding,
                ty: signature.ty,
            }),
            ty: signature.ty,
        })
    }
}
