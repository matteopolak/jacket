//! Ordinary source names may shadow the implicit context value.
use super::*;
impl FileScope<'_> {
    pub(crate) fn has_source_context(&self, name: Symbol, span: Span) -> Result<bool, Diagnostic> {
        match self.declarations.graph.lookup(
            self.file,
            &NamePath {
                root: name,
                members: vec![],
            },
        ) {
            Ok(_) => Ok(true),
            Err(jai_modules::LookupError::UnknownName(_)) => Ok(false),
            Err(_) => declaration_id(
                self.declarations.graph,
                self.file,
                &NamePath {
                    root: name,
                    members: vec![],
                },
                span,
            )
            .map(|_| true),
        }
    }
}
impl Resolver<'_> {
    pub(crate) fn source_context_binding(
        &mut self,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        let Some(name) = self.symbols.find("context") else {
            return Ok(None);
        };
        let path = NamePath {
            root: name,
            members: vec![],
        };
        let local = self.resolve_local_name(name, span)?;
        let imported = self.lexical_graph_binding(&path, span)?.is_some();
        let lexical = self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(&name));
        let source = self
            .graph_scope
            .map(|scope| scope.has_source_context(name, span))
            .transpose()?
            .unwrap_or(false);
        if local.is_some()
            || imported
            || lexical
            || source
            || (self.graph_scope.is_none() && self.globals.contains_key(&name))
        {
            return self.lookup_path(&path, span).map(Some);
        }
        Ok(None)
    }
}
