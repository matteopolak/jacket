//! Collection protocols retain real module and lexical macro origins.
use super::*;

impl Resolver<'_> {
    pub(super) fn expanded_candidates(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Vec<ExpandedTarget>, Diagnostic> {
        if self.with_local_constant_source(path, span, |_, _, constant| {
            Ok(Some(matches!(
                constant.initializer.kind,
                syntax::ExpressionKind::ShortLambda(_)
            )))
        })? == Some(true)
        {
            return Ok(Vec::new());
        }
        if let Some(binding) = self.lexical_graph_binding(path, span)? {
            if matches!(
                binding,
                jai_modules::Binding::SourceMember { .. } | jai_modules::Binding::StorageMember(_)
            ) {
                return self.expanded_source_member_candidates(binding, span);
            }
            return self
                .graph_scope
                .expect("lexical import has a graph")
                .expanded_binding_procedures(binding, span)
                .map(|targets| targets.into_iter().map(ExpandedTarget::module).collect());
        }
        if let Some(binding) = self.namespace_binding(path, span)? {
            return match binding {
                Binding::Macro(id) => self
                    .local_expanded_target(id, span)
                    .map(|target| vec![target]),
                _ => Ok(Vec::new()),
            };
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            return match binding {
                Binding::Macro(id) if path.members.is_empty() => self
                    .local_expanded_target(id, span)
                    .map(|target| vec![target]),
                _ => Ok(Vec::new()),
            };
        }
        self.graph_scope
            .map_or_else(
                || Ok(Vec::new()),
                |scope| scope.expanded_procedures(path, span),
            )
            .map(|targets| targets.into_iter().map(ExpandedTarget::module).collect())
    }

    pub(super) fn expanded_source_member_candidates(
        &mut self,
        binding: jai_modules::Binding,
        span: Span,
    ) -> Result<Vec<ExpandedTarget>, Diagnostic> {
        // Source members acquire canonical method/macro identities only after
        // their concrete owner namespace is hydrated by ordinary semantics.
        match self.imported_binding_value(binding, span)? {
            Binding::Macro(id) => self
                .local_expanded_target(id, span)
                .map(|target| vec![target]),
            _ => Ok(Vec::new()),
        }
    }
}
