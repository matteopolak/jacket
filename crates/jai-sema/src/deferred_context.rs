//! Lower one real lexical suffix without rediscovering its declarations.
use super::*;

impl Resolver<'_> {
    pub(crate) fn deferred_context_statement(
        &mut self,
        statement: &syntax::Statement,
        suffix: &[syntax::Statement],
    ) -> Result<Statement, Diagnostic> {
        let syntax::StatementKind::PushContextDeferred { value } = &statement.kind else {
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
