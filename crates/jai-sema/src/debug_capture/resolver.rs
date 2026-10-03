//! Resolve exact original source records independently of lexical name lookup.
use super::*;

impl Resolver<'_> {
    pub(crate) fn debug_location(
        &mut self,
        span: Span,
    ) -> Result<Option<DebugSourceLocation>, Diagnostic> {
        if !self.debug.policy().emits() {
            return Ok(None);
        }
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let source = self.debug.source().unwrap_or_else(|| scope.source());
        self.debug_location_for(source, span)
    }
    fn debug_location_for(
        &mut self,
        source: SourceId,
        span: Span,
    ) -> Result<Option<DebugSourceLocation>, Diagnostic> {
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        let record = scope
            .source_record(source)
            .ok_or_else(|| Diagnostic::new(span, "debug source origin is not retained"))?;
        self.meta
            .debug_sources
            .source_location(record, SourceSpan { source, span })
            .map(Some)
            .map_err(|error| Diagnostic::new(span, error.to_string()))
    }
    pub(crate) fn debug_local(
        &mut self,
        local: jai_ir::Local,
        name: Symbol,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(location) = self.debug_location(span)? {
            self.debug
                .local(local.id(), self.symbols.name(name).to_owned(), location);
        }
        Ok(())
    }
    pub(crate) fn debug_parameter(
        &mut self,
        local: jai_ir::Local,
        name: Symbol,
        span: Span,
        ordinal: usize,
    ) -> Result<(), Diagnostic> {
        if let Some(location) = self.debug_location(span)? {
            self.debug.parameter(
                local.id(),
                self.symbols.name(name).to_owned(),
                location,
                ordinal,
            );
        }
        Ok(())
    }
    pub(crate) fn debug_prefix_local(
        &mut self,
        local: jai_ir::Local,
        name: Symbol,
        declaration_span: Span,
        declaration_source: SourceId,
        initializer_span: Span,
        index: usize,
    ) -> Result<(), Diagnostic> {
        let declaration = self.debug_location_for(declaration_source, declaration_span)?;
        let initializer = self.debug_location(initializer_span)?;
        if let (Some(declaration), Some(initializer)) = (declaration, initializer) {
            self.debug.prefix_local(
                local.id(),
                self.symbols.name(name).to_owned(),
                declaration,
                initializer,
                index,
            );
        }
        Ok(())
    }
    pub(crate) fn publish_debug(&mut self, name: Symbol, span: Span) -> Result<(), Diagnostic> {
        self.publish_source_debug(Some(name), span)
    }
    pub(crate) fn publish_source_debug(
        &mut self,
        name: Option<Symbol>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.meta
            .debug_sources
            .set_procedure_policy(self.procedure, self.debug.policy());
        let source = self
            .debug
            .source()
            .or_else(|| self.graph_scope.map(|scope| scope.source()));
        if let Some(source) = source
            && let Some(location) = self.debug_location_for(source, span)?
        {
            self.meta.debug_sources.insert(
                self.procedure,
                ProcedureSource {
                    name: name
                        .map(|name| self.symbols.name(name).to_owned())
                        .unwrap_or_else(|| "anonymous procedure".into()),
                    location,
                },
            );
        }
        self.debug
            .publish(&mut self.meta.debug_sources, self.procedure)
            .map_err(|message| Diagnostic::new(span, message))?;
        Ok(())
    }
}
