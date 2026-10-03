//! Full anonymous procedures use their declared source signature and ordinary child binder.
use super::*;

impl Resolver<'_> {
    pub(crate) fn preview_anonymous_procedure_header(
        &mut self,
        source: &syntax::SourceProcedureSyntax,
    ) -> Result<CheckedSourceHeader, Diagnostic> {
        self.check_anonymous_source_domain(source)?;
        let preview = self.source_header_components(
            CallableSource {
                parameters: &source.parameters,
                results: &source.results,
                return_abi: source.return_abi,
                convention: source.convention,
                context: source.context,
                span: source.span,
            },
            SourceHeaderPhase::Preview,
        )?;
        if preview.parameters.iter().any(|parameter| {
            parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                && matches!(self.types.kind(parameter.ty), Ok(TypeKind::Code))
        }) || preview
            .results
            .iter()
            .any(|result| matches!(self.types.kind(result.ty), Ok(TypeKind::Code)))
        {
            return Err(Diagnostic::new(
                source.span,
                "anonymous Code parameters and results require a compiler-only callable domain",
            ));
        }
        Ok(preview)
    }

    fn check_anonymous_source_domain(
        &self,
        source: &syntax::SourceProcedureSyntax,
    ) -> Result<(), Diagnostic> {
        if source.expands {
            return Err(Diagnostic::new(
                source.span,
                "anonymous expansion procedures require a lexical macro origin",
            ));
        }
        if let Some(modifier) = &source.modify {
            return Err(Diagnostic::new(
                modifier.span,
                "anonymous polymorphic modifiers require a specialization origin",
            ));
        }
        if source.compiler.is_some() {
            return Err(Diagnostic::new(
                source.span,
                "anonymous compiler procedures require a registered provider origin",
            ));
        }
        Ok(())
    }

    pub(crate) fn anonymous_procedure_value(
        &mut self,
        source: &syntax::SourceProcedureSyntax,
        span: Span,
        expected: Option<TypeId>,
    ) -> Result<Expr, Diagnostic> {
        let preview = self.preview_anonymous_procedure_header(source)?;
        if expected.is_some_and(|expected| expected != preview.ty) {
            return Err(Diagnostic::new(
                span,
                "anonymous procedure header does not match its contextual type",
            ));
        }
        let procedure = self.reserve_local_anonymous_procedure(
            source.span,
            LocalCallableSpecialization::Expected(preview.ty),
        )?;
        self.remember_anonymous_procedure_source(procedure, preview.ty, source.span)?;
        if self
            .meta
            .local_declarations
            .ready_procedure(procedure)
            .is_some()
        {
            return self.typed_value(
                ValueExpr::ProcedureValue {
                    procedure,
                    ty: preview.ty,
                },
                preview.ty,
                span,
            );
        }
        let header = self.source_header_components(
            CallableSource {
                parameters: &source.parameters,
                results: &source.results,
                return_abi: source.return_abi,
                convention: source.convention,
                context: source.context,
                span: source.span,
            },
            SourceHeaderPhase::Definition {
                procedure,
                readiness: HeaderReadiness::Complete,
            },
        )?;
        if header.ty != preview.ty {
            return Err(Diagnostic::new(
                span,
                "anonymous procedure header changed after its checked type reservation",
            ));
        }
        let signature = header.with_identity(procedure);
        self.meta
            .local_declarations
            .signatures
            .insert(procedure, signature.clone());
        self.meta
            .local_declarations
            .header_readiness
            .insert(procedure, HeaderReadiness::Complete);
        self.meta
            .remember_inline_hint(procedure, source.inline_hint);
        self.meta.remember_execution(procedure, source.execution);
        if source.deprecation.is_some() {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "anonymous deprecation requires retained source records",
                )
            })?;
            let source_id = self.debug.source().unwrap_or_else(|| scope.source());
            let record = scope.source_record(source_id).ok_or_else(|| {
                Diagnostic::new(span, "anonymous declaration source origin is not retained")
            })?;
            self.meta.remember_deprecation(
                crate::deprecation_warnings::DeprecationKey::Procedure(procedure),
                record,
                "anonymous procedure",
                source.deprecation.as_ref(),
                source.span,
            )?;
        }
        self.meta
            .local_declarations
            .begin_anonymous_procedure(procedure, span)?;
        let body = self.lower_source_procedure_body(SourceProcedureDefinition {
            signature: &signature,
            source,
            name: None,
        });
        self.meta
            .local_declarations
            .end_anonymous_procedure(procedure);
        self.meta
            .local_declarations
            .publish_generated(signature, body?)?;
        self.typed_value(
            ValueExpr::ProcedureValue {
                procedure,
                ty: preview.ty,
            },
            preview.ty,
            span,
        )
    }
}
