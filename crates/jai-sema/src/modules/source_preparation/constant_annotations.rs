//! Publish canonical annotation facts before a selected typed initializer is consumed.
use super::*;
impl<'graph> PendingAliases<'graph> {
    pub(super) fn prepare_constant_annotation(
        &mut self,
        declaration: DeclarationId,
        declarations: &mut ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        meta: &mut crate::reflection::MetaContext,
    ) -> Result<Option<PendingType>, LocatedDiagnostic> {
        let source = self
            .graph
            .declaration(declaration)
            .expect("actual constant demand");
        let FileDeclarationKind::Constant(constant) = &source.syntax().kind else {
            return Ok(None);
        };
        let Some(annotation) = &constant.ty else {
            return Ok(None);
        };
        if self.constants.annotation(declaration).is_some()
            || declarations.nominals.is_type_alias(self.graph, declaration)
        {
            return Ok(None);
        }
        let outcome = aggregates::parameterized::prepare_type_paired(
            self.graph,
            aggregates::parameterized::TypeRequest::new(source.file(), annotation, constant.span),
            types,
            &declarations.nominals,
            &mut meta.record_specializations,
            &mut |file, expression| self.constants.prepare_evaluate_lazy(file, expression),
        )?;
        match outcome {
            TypePreparation::Pending(cause) => Ok(Some(cause)),
            TypePreparation::Ready(ty) => {
                self.constants
                    .register_annotation(declaration, ty, types)
                    .map_err(|error| {
                        located(
                            self.graph,
                            source.file(),
                            Diagnostic::new(constant.span, error.to_string()),
                        )
                    })?;
                declarations.nominals.value_types.insert(declaration, ty);
                Ok(None)
            }
        }
    }
}
