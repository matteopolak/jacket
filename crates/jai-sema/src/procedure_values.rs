//! Procedure addresses and indirect calls use the same canonical signature as declarations.
use super::*;
mod arguments;
pub(crate) mod baked_arguments;
mod baked_preview;
mod bind_arguments;
pub(crate) mod bindings;
mod context_calls;
pub(crate) mod contracts;
mod nullable;
mod preview;
mod sequence_packs;
pub(crate) mod signatures;
pub(crate) mod source_annotations;
#[cfg(test)]
mod tests;

pub(crate) type IndirectCallBinding = (
    ValueExpr,
    Vec<(ParameterId, ValueExpr)>,
    Vec<ResultSignature>,
);

impl Resolver<'_> {
    pub(crate) fn procedure_value(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(signature) = self.local_callable_signature(path, span)? {
            self.warn_deprecated_procedure(signature.id, span)?;
            return self.typed_value(
                ValueExpr::ProcedureValue {
                    procedure: signature.id,
                    ty: signature.ty,
                },
                signature.ty,
                span,
            );
        }
        if self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(&path.root))
        {
            return Err(Diagnostic::new(
                span,
                "local value shadows the procedure name",
            ));
        }
        let signature = if let Some(scope) = self.graph_scope {
            scope.signature(path, span)?
        } else {
            if !path.members.is_empty() {
                return Err(Diagnostic::new(
                    span,
                    "qualified procedure value requires a module scope",
                ));
            }
            self.signatures
                .get(&path.root)
                .ok_or_else(|| Diagnostic::new(span, "name does not denote a procedure"))?
        };
        let id = signature.id;
        let ty = signature.ty;
        self.warn_deprecated_procedure(id, span)?;
        Ok(Expr::Typed {
            ty,
            value: ValueExpr::ProcedureValue {
                procedure: id,
                ty,
            },
        })
    }

    pub(crate) fn indirect_call(
        &mut self,
        callee: Expr,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        self.indirect_call_from_source(callee, arguments, span, None)
    }

    pub(crate) fn indirect_call_from_source(
        &mut self,
        callee: Expr,
        arguments: &[syntax::CallArgument],
        span: Span,
        source: Option<&syntax::Expression>,
    ) -> Result<Expr, Diagnostic> {
        let (callee, values, results) =
            self.resolve_indirect_call_binding_from_source(callee, arguments, span, source)?;
        self.indirect_call_expression(callee, values, &results, span)
    }

    pub(crate) fn indirect_call_expression(
        &mut self,
        callee: ValueExpr,
        values: Vec<(ParameterId, ValueExpr)>,
        results: &[ResultSignature],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let result = match results {
            [result] => result.ty,
            [] => {
                return Ok(Expr::IndirectVoid {
                    inline_hint: jai_types::InlineHint::Automatic,
                    callee: Box::new(callee),
                    arguments: values,
                });
            }
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "multiple indirect results require result binding",
                ));
            }
        };
        let value = ValueExpr::IndirectCall {
            inline_hint: jai_types::InlineHint::Automatic,
            callee: Box::new(callee),
            arguments: values,
            ty: result,
        };
        self.typed_value(value, result, span)
    }

    pub(crate) fn resolve_indirect_call_binding_from_source(
        &mut self,
        callee: Expr,
        arguments: &[syntax::CallArgument],
        span: Span,
        source: Option<&syntax::Expression>,
    ) -> Result<IndirectCallBinding, Diagnostic> {
        self.resolve_indirect_call_binding_from_source_with_defaults(
            callee, arguments, span, source,
        )
        .map(|(callee, values, results, _)| (callee, values, results))
    }

    pub(crate) fn resolve_indirect_call_binding_from_source_with_defaults(
        &mut self,
        callee: Expr,
        arguments: &[syntax::CallArgument],
        span: Span,
        source: Option<&syntax::Expression>,
    ) -> Result<crate::runtime_defaults::IndirectCallWithDefaults, Diagnostic> {
        let ty = self.expression_type(&callee, span)?;
        let signature = self
            .types
            .procedure_definition(ty)
            .map_err(|_| Diagnostic::new(span, "call target is not a procedure value"))?
            .clone();
        self.check_call_context(&signature, span)?;
        let callee = callee.value(span)?;
        let metadata = match source {
            Some(source) => self
                .callback_expression_contract(source, &callee, span)?
                .and_then(|contract| contract.callback().cloned()),
            None => self.callback_value_metadata(&callee, span)?,
        };
        self.bind_indirect_call_metadata_with_defaults(
            callee,
            ty,
            &signature,
            metadata.as_ref(),
            arguments,
            span,
        )
    }

    pub(crate) fn resolve_indirect_call_binding_with_metadata(
        &mut self,
        callee: Expr,
        arguments: &[syntax::CallArgument],
        span: Span,
        metadata: Option<bindings::CallbackSignature>,
    ) -> Result<IndirectCallBinding, Diagnostic> {
        let ty = self.expression_type(&callee, span)?;
        let signature = self
            .types
            .procedure_definition(ty)
            .map_err(|_| Diagnostic::new(span, "call target is not a procedure value"))?
            .clone();
        self.check_call_context(&signature, span)?;
        let callee = callee.value(span)?;
        let metadata = match metadata {
            Some(metadata) => Some(metadata),
            None => self.callback_value_metadata(&callee, span)?,
        };
        self.bind_indirect_call_metadata(callee, ty, &signature, metadata.as_ref(), arguments, span)
    }

    fn bind_indirect_call_metadata(
        &mut self,
        callee: ValueExpr,
        ty: TypeId,
        signature: &ProcedureType,
        metadata: Option<&bindings::CallbackSignature>,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<IndirectCallBinding, Diagnostic> {
        self.bind_indirect_call_metadata_with_defaults(
            callee, ty, signature, metadata, arguments, span,
        )
        .map(|(callee, values, results, _)| (callee, values, results))
    }

    fn bind_indirect_call_metadata_with_defaults(
        &mut self,
        callee: ValueExpr,
        ty: TypeId,
        signature: &ProcedureType,
        metadata: Option<&bindings::CallbackSignature>,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<crate::runtime_defaults::IndirectCallWithDefaults, Diagnostic> {
        if metadata.is_some_and(|metadata| metadata.ty != ty) {
            return Err(Diagnostic::new(
                span,
                "callback binding metadata has a different canonical signature",
            ));
        }
        let (values, reads) =
            self.bind_callable_arguments_with_defaults(signature, metadata, arguments, span)?;
        let results = signature
            .results
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                metadata
                    .and_then(|metadata| metadata.results.get(index))
                    .cloned()
                    .unwrap_or(ResultSignature {
                        name: None,
                        ty: *ty,
                        default: None,
                        usage: syntax::ResultUsage::Optional,
                    })
            })
            .collect();
        Ok((callee, values, results, reads))
    }
}
