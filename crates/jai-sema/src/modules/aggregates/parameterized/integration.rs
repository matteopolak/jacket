//! Expression calls that name type templates produce type values.
use crate::{Diagnostic, Expr, Resolver, Span, syntax};

impl Resolver<'_> {
    pub(crate) fn try_record_application(
        &mut self,
        path: &syntax::NamePath,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        let Some(scope) = self.graph_scope else {
            return Ok(None);
        };
        if let Some(binding) = self.lexical_graph_binding_ready(path, span)? {
            if scope.imported_record_template(binding, span).is_err() {
                return Ok(None);
            }
        } else if self.local_name_present(path.root)
            || scope.record_template_origin(path, span).is_err()
        {
            return Ok(None);
        }
        let syntax = syntax::TypeSyntax::Application(syntax::TypeApplicationSyntax {
            base: Box::new(syntax::TypeSyntax::Named(path.clone())),
            arguments: arguments.to_vec(),
            span,
        });
        self.lexical_annotation(&syntax, span)
            .map(|ty| Some(Expr::Type(ty)))
    }
}
