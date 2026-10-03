//! Early qualified annotations use the original transparent namespace source.
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn source_namespace_root(
        &mut self,
        file: FileInstanceId,
        prefix: &syntax::NamePath,
        substitution: Option<&Substitution>,
        span: Span,
    ) -> TypeResult<Option<TypeId>> {
        let Ok(jai_modules::Binding::Declaration(id)) = self.graph.lookup(file, prefix) else {
            return Ok(None);
        };
        if let Some(&ty) = self.nominals.declarations.get(&id) {
            return Ok(Some(ty));
        }
        let declaration = self
            .graph
            .declaration(id)
            .expect("checked namespace source");
        let named = match &declaration.syntax().kind {
            syntax::FileDeclarationKind::TypeAlias(alias) => {
                matches!(alias.ty, syntax::TypeSyntax::Named(_))
            }
            syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                matches!(
                    type_expression(&constant.initializer),
                    Some(syntax::TypeSyntax::Named(_))
                )
            }
            _ => false,
        };
        if !named {
            return Ok(None);
        }
        // This is the shorter original prefix, not a fabricated application or
        // a guessed owner. Normal alias resolution retains its defining file,
        // cycle guard and paired PendingType before any member is selected.
        self.resolve(
            file,
            &syntax::TypeSyntax::Named(prefix.clone()),
            substitution,
            span,
        )
        .map(Some)
    }
}
