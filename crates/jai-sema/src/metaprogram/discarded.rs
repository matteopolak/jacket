//! Discarded expanded formals retain checked types without runtime operands.
use super::*;

impl Resolver<'_> {
    pub(super) fn expanded_discarded_type(
        &mut self,
        target: &ExpandedTarget,
        parameter: &syntax::Parameter,
    ) -> Result<jai_types::TypeId, Diagnostic> {
        let (annotation, default) = match &parameter.binding {
            syntax::ParameterBinding::Required(ty) => (
                Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(
                    *ty,
                ))),
                None,
            ),
            syntax::ParameterBinding::RequiredType(ty) => (Some(ty.clone()), None),
            syntax::ParameterBinding::Defaulted {
                ty,
                expression,
            } => (
                ty.map(|ty| syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty))),
                Some(expression),
            ),
            syntax::ParameterBinding::DefaultedType {
                ty,
                expression,
            } => (ty.clone(), Some(expression)),
        };
        let expected = annotation
            .as_ref()
            .map(|annotation| self.expanded_annotation(target, annotation, parameter.span))
            .transpose()?;
        let Some(default) = default else {
            return expected.ok_or_else(|| {
                Diagnostic::new(
                    parameter.span,
                    "discarded expanded parameter requires an annotation or a typed default",
                )
            });
        };
        let substitution = target
            .capture
            .as_ref()
            .and_then(|capture| capture.substitution.as_ref());
        self.with_definition_scope(target.file, substitution, |resolver| {
            let frames = target
                .capture
                .as_ref()
                .map_or_else(Vec::new, |capture| capture.frames.clone());
            let original_frames = std::mem::replace(&mut resolver.scopes, frames);
            let locals = target.capture.as_ref().map_or_else(
                || resolver.local_scopes.isolated_expansion(),
                |capture| capture.local_scopes.clone(),
            );
            let mut original_locals = std::mem::replace(&mut resolver.local_scopes, locals);
            let source = target
                .capture
                .as_ref()
                .map(|capture| capture.location.source)
                .or_else(|| resolver.graph_scope.map(|scope| scope.source()));
            let original_source = resolver.debug.replace_source(source);
            let result = (|| {
                let ty = match expected {
                    Some(ty) => ty,
                    None if matches!(default.kind, syntax::ExpressionKind::CallerLocation) => {
                        resolver.caller_location_type(default.span)?
                    }
                    None => {
                        let description = resolver.describe_argument(default)?;
                        resolver.argument_type(&description, default.span)?
                    }
                };
                resolver.check_discarded_argument(default, ty)?;
                Ok(ty)
            })()
            .map_err(|error: Diagnostic| match source {
                Some(source) => error.with_fallback_source(source),
                None => error,
            });
            resolver.debug.replace_source(original_source);
            resolver.scopes = original_frames;
            original_locals.resume_after_expansion(&resolver.local_scopes);
            resolver.local_scopes = original_locals;
            result
        })
    }

    pub(super) fn missing_discarded_binding(
        &mut self,
        target: &ExpandedTarget,
        parameter: &syntax::Parameter,
    ) -> Result<Binding, Diagnostic> {
        if !parameter.variadic
            && matches!(
                &parameter.binding,
                syntax::ParameterBinding::Required(_) | syntax::ParameterBinding::RequiredType(_)
            )
        {
            return Err(Diagnostic::new(
                parameter.span,
                "missing required discarded expanded argument",
            ));
        }
        let ty = self.expanded_discarded_type(target, parameter)?;
        Ok(Binding::Discarded(ty))
    }
}
