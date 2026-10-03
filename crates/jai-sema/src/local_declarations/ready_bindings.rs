//! Immutable lexical facts for previews that must not resolve initializers.
use super::*;

impl LocalDeclarationRegistry {
    pub(crate) fn record_namespace_binding(&self, owner: TypeId, name: Symbol) -> Option<&Binding> {
        self.record_namespaces.get(&owner)?.get(&name)
    }

    pub(crate) fn header_readiness(&self, procedure: ProcedureId) -> Option<HeaderReadiness> {
        self.header_readiness.get(&procedure).copied()
    }
}

impl Resolver<'_> {
    /// A reserved inner name blocks outer lookup even when its value is not
    /// ready. Imported bindings retain their graph identity for pure hydration.
    pub(crate) fn ready_local_default_binding(
        &self,
        name: Symbol,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&name) {
                return Ok(Some(binding.clone()));
            }
            let Some(frame) = self.local_scopes.frames.get(depth) else {
                continue;
            };
            if let Some(declaration) = frame.declarations.get(&name) {
                if let Some(binding) = self
                    .meta
                    .local_declarations
                    .entries
                    .get(&declaration.id)
                    .and_then(|entry| entry.binding.as_ref())
                {
                    return Ok(Some(binding.clone()));
                }
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "local declaration '{}' is not ready for parameter default preview",
                        self.symbols.name(name)
                    ),
                ));
            }
            if frame.runtime.contains_key(&name) || frame.runtime_symbols.contains(&name) {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "runtime local '{}' is unavailable before its declaration",
                        self.symbols.name(name)
                    ),
                ));
            }
            if let Some(binding) = frame.using_bindings.get(&name) {
                return Ok(Some(binding.clone()));
            }
            if let Some(&marker) = frame.using_placeholders.get(&name) {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "using placeholder requires its source graph")
                })?;
                return Ok(Some(Binding::Imported(
                    scope.imported_placeholder_binding(marker, span)?,
                )));
            }
            if frame.using_pending.contains(&name) {
                return Err(Diagnostic::new(
                    span,
                    format!(
                        "using place alias '{}' is unavailable before its source statement",
                        self.symbols.name(name)
                    ),
                ));
            }
            if let Some(&binding) = frame.imports.get(&name) {
                return Ok(Some(match binding {
                    jai_modules::Binding::Module(module) => Binding::Namespace(module),
                    binding => Binding::Imported(binding),
                }));
            }
            if let Some(marker) = frame.imports.placeholder(name) {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(span, "imported placeholder requires its source graph")
                })?;
                let binding = scope.imported_placeholder_binding(marker, span)?;
                return Ok(Some(match binding {
                    jai_modules::Binding::Module(module) => Binding::Namespace(module),
                    binding => Binding::Imported(binding),
                }));
            }
        }
        Ok(None)
    }
}
