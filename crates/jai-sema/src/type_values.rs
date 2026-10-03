//! Builtin type values use the syntax catalog after ordinary names are resolved.
use super::*;

impl Resolver<'_> {
    pub(crate) fn builtin_type_expression(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<Expr>, Diagnostic> {
        if !path.members.is_empty() {
            return Ok(None);
        }
        if self.local_name_present(path.root) {
            return Ok(None);
        }
        if let Some(scope) = self.graph_scope
            && !scope.type_value_name_absent(path)
        {
            return Ok(None);
        }
        let Some(builtin) = syntax::BuiltinType::from_spelling(self.symbols.name(path.root)) else {
            return Ok(None);
        };
        let ty = self.reflected_type_syntax(&syntax::TypeSyntax::Builtin(builtin), span)?;
        Ok(Some(Expr::Type(ty)))
    }
}
