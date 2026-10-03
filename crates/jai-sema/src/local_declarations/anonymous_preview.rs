//! Full callable candidate bodies use the shared pure statement facts.
use super::*;
impl Resolver<'_> {
    pub(crate) fn preview_anonymous_procedure_argument(
        &mut self,
        source: &syntax::SourceProcedureSyntax,
    ) -> Result<CheckedSourceHeader, Diagnostic> {
        use crate::short_lambdas::{PreviewBinding, PreviewIdentity};
        let header = self.preview_anonymous_procedure_header(source)?;
        if source.parameters.iter().any(|parameter| parameter.using) {
            return Err(Diagnostic::new(
                source.span,
                "using formals require a checked full-source body preview rule",
            ));
        }
        let descriptor = self
            .types
            .procedure_definition(header.ty)
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?
            .clone();
        let original_scopes = self.scopes.clone();
        let mut scopes = original_scopes.clone();
        for scope in &mut scopes {
            for binding in scope.values_mut() {
                match binding {
                    Binding::Storage(storage)
                        if self.short_lambda_captures_storage(*storage, source.span)? =>
                    {
                        *binding = Binding::LambdaPreview(PreviewBinding::RuntimeCapture)
                    }
                    Binding::LambdaPreview(PreviewBinding::Parameter(_)) => {
                        *binding = Binding::LambdaPreview(PreviewBinding::RuntimeCapture)
                    }
                    _ => {}
                }
            }
        }
        let mut formals = HashMap::new();
        for parameter in &header.parameters {
            let binding = match parameter.evaluation {
                syntax::ParameterEvaluation::Evaluate => {
                    Binding::LambdaPreview(PreviewBinding::Parameter(parameter.ty))
                }
                syntax::ParameterEvaluation::Discard => Binding::Discarded(parameter.ty),
            };
            formals.insert(parameter.name, binding);
        }
        for result in &header.results {
            if let Some(name) = result.name
                && formals
                    .insert(
                        name,
                        Binding::LambdaPreview(PreviewBinding::Parameter(result.ty)),
                    )
                    .is_some()
            {
                return Err(Diagnostic::new(
                    source.span,
                    "named result duplicates a parameter binding",
                ));
            }
        }
        scopes.push(formals);
        let original_context = self.context_available;
        let original_preview = self.meta.short_lambda_preview;
        let original_checks = self.checks;
        self.scopes = scopes;
        self.context_available = descriptor.convention == CallingConvention::Jai
            && descriptor.context == ContextMode::Implicit;
        self.meta.short_lambda_preview = Some(PreviewIdentity {
            ty: header.ty,
            span: source.span,
        });
        self.checks = self.checks.overridden(source.checks);
        let checked = self.preview_full_procedure_block(
            &source.body,
            &descriptor,
            &header.results,
            source.span,
        );
        self.scopes = original_scopes;
        self.context_available = original_context;
        self.meta.short_lambda_preview = original_preview;
        self.checks = original_checks;
        checked?;
        Ok(header)
    }
}
