//! Lower one real lexical suffix without rediscovering its declarations.
use super::*;

impl Resolver<'_> {
    pub(crate) fn push_context_suffix(
        &mut self,
        value: Option<&syntax::Expression>,
        suffix: &[syntax::Statement],
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        let definition = &self
            .context
            .ok_or_else(|| Diagnostic::new(span, "context schema is unavailable"))?
            .definition;
        let ty = definition.record_type;
        let value = match value {
            Some(value) => {
                let expression = self.expr_expected(value, ty)?;
                self.coerce_value(expression, ty, value.span)?
            }
            None => definition.default.clone().into_expression(),
        };
        let id = PushContextId::new(self.procedure, self.next_push);
        self.next_push = self
            .next_push
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new(span, "context push identity space exhausted"))?;
        let previous_push = self.active_push.replace(id);
        let previous = std::mem::replace(&mut self.context_available, true);
        // The original containing block already prepared every declaration and
        // selected source conditional. Keep those exact lexical identities.
        let result = self.block_with_preparation(suffix, false, false);
        self.context_available = previous;
        self.active_push = previous_push;
        let body = result?;
        self.debug
            .attach_block(&[DebugPathStep::Child(DebugBranch::PushContext)]);
        Ok(Statement::PushContext {
            id,
            value,
            body,
        })
    }

    pub(crate) fn deferred_context_statement(
        &mut self,
        statement: &syntax::Statement,
        suffix: &[syntax::Statement],
    ) -> Result<Statement, Diagnostic> {
        let syntax::StatementKind::PushContextDeferred {
            value,
        } = &statement.kind
        else {
            return Err(Diagnostic::new(
                statement.span,
                "expected a deferred context push",
            ));
        };
        let debug_location = self.debug_location(statement.span)?;
        let outer_span = std::mem::replace(&mut self.span, statement.span);
        self.debug.begin_statement(debug_location);
        let result = self
            .push_context_suffix(value.as_ref(), suffix, statement.span)
            .map_err(|diagnostic| {
                match self
                    .debug
                    .source()
                    .or_else(|| self.graph_scope.map(|scope| scope.source()))
                {
                    Some(source) => diagnostic.with_fallback_source(source),
                    None => diagnostic,
                }
            });
        self.debug.finish_statement(result.as_ref().ok());
        self.span = outer_span;
        result
    }
}
