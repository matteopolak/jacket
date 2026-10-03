//! Source definition facts survive header completion and debug suppression.
use super::*;
use jai_source::SourceSpan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcedureSourceIdentity {
    pub name: Option<Symbol>,
    pub file: Option<FileInstanceId>,
    pub location: SourceSpan,
    pub ty: TypeId,
}

impl LocalDeclarationRegistry {
    pub(crate) fn procedure_source_identity(
        &self,
        procedure: ProcedureId,
    ) -> Option<ProcedureSourceIdentity> {
        self.procedure_sources.get(&procedure).copied()
    }

    fn retain_procedure_source(
        &mut self,
        procedure: ProcedureId,
        identity: ProcedureSourceIdentity,
    ) -> Result<(), Diagnostic> {
        if let Some(previous) = self.procedure_sources.get(&procedure) {
            if *previous != identity {
                return Err(Diagnostic::at_source(
                    identity.location,
                    "procedure source identity differs from its reserved definition",
                ));
            }
        } else {
            self.procedure_sources.insert(procedure, identity);
        }
        Ok(())
    }
}

impl Resolver<'_> {
    pub(super) fn remember_local_callable_source(
        &mut self,
        declaration: LocalDeclarationId,
        signature: &Signature,
        name: Symbol,
    ) -> Result<(), Diagnostic> {
        let Some(source) = self.debug.source().or(declaration.defining_source()) else {
            // The scalar-only adapter does not own a SourceMap.
            return Ok(());
        };
        self.retain_checked_procedure_source(
            signature.id,
            ProcedureSourceIdentity {
                name: Some(name),
                file: declaration.defining_file(),
                location: SourceSpan {
                    source,
                    span: declaration.source_span(),
                },
                ty: signature.ty,
            },
        )
    }

    pub(crate) fn remember_anonymous_procedure_source(
        &mut self,
        procedure: ProcedureId,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let source = self
            .debug
            .source()
            .or_else(|| self.graph_scope.map(|scope| scope.source()))
            .or_else(|| self.compile_time.map(|context| context.source));
        let Some(source) = source else {
            return Ok(());
        };
        self.retain_checked_procedure_source(
            procedure,
            ProcedureSourceIdentity {
                name: None,
                file: self
                    .graph_scope
                    .map(|scope| scope.code_origin().0)
                    .or_else(|| self.compile_time.map(|context| context.file)),
                location: SourceSpan {
                    source,
                    span,
                },
                ty,
            },
        )
    }

    fn retain_checked_procedure_source(
        &mut self,
        procedure: ProcedureId,
        identity: ProcedureSourceIdentity,
    ) -> Result<(), Diagnostic> {
        self.types
            .procedure_definition(identity.ty)
            .map_err(|error| Diagnostic::at_source(identity.location, error.to_string()))?;
        self.meta
            .local_declarations
            .retain_procedure_source(procedure, identity)
    }

    pub(crate) fn local_definition_owner(&self) -> Option<LexicalScopeOwner> {
        // Expression lambdas have an actual procedure and parameter frame
        // before their source body enters a block scope.
        if self.local_scopes.frames.len() < self.scopes.len() {
            return self
                .expression_owner
                .filter(|owner| *owner == self.procedure)
                .map(LexicalScopeOwner::Procedure);
        }
        self.local_scopes.frames.last().map(|frame| frame.id.owner)
    }

    pub(crate) fn local_definition_queries_forbidden(&self) -> bool {
        matches!(
            self.local_scopes.annotation_owner,
            NominalAnnotationContext::Forbidden
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn named_identity() -> (ProcedureId, ProcedureSourceIdentity, Symbols) {
        let text = "worker :: (value:int)->int #no_debug { return value; } main :: () {}";
        let module = syntax::parse(text).unwrap();
        let source = SourceMap::default().insert("identity.jai".into(), text.into());
        let definition = &module.procedures()[0];
        assert_eq!(definition.debug, jai_types::DebugPolicy::Suppress);
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let ty = types
            .procedure(ProcedureType {
                parameters: vec![int].into(),
                results: vec![int].into(),
                return_abi: definition.return_abi,
                convention: definition.convention,
                context: definition.context,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        (
            ProcedureId::new(0),
            ProcedureSourceIdentity {
                name: Some(definition.name),
                file: None,
                location: SourceSpan {
                    source,
                    span: definition.span,
                },
                ty,
            },
            module.symbols().clone(),
        )
    }

    #[test]
    fn reserved_source_identity_survives_no_debug_and_header_retries() {
        let (procedure, identity, symbols) = named_identity();
        let mut registry = LocalDeclarationRegistry::default();
        registry
            .retain_procedure_source(procedure, identity)
            .unwrap();
        registry
            .retain_procedure_source(procedure, identity)
            .unwrap();
        let retained = registry.procedure_source_identity(procedure).unwrap();
        assert_eq!(symbols.name(retained.name.unwrap()), "worker");
        assert_eq!(retained, identity);
        assert!(registry.ready_procedure(procedure).is_none());
    }

    #[test]
    fn procedure_alias_cannot_replace_its_targets_source_name() {
        let (procedure, identity, mut symbols) = named_identity();
        let mut registry = LocalDeclarationRegistry::default();
        registry
            .retain_procedure_source(procedure, identity)
            .unwrap();
        let alias = ProcedureSourceIdentity {
            name: Some(symbols.intern("alias")),
            ..identity
        };
        let error = registry
            .retain_procedure_source(procedure, alias)
            .unwrap_err();
        assert_eq!(error.source, Some(identity.location.source));
        assert_eq!(
            registry.procedure_source_identity(procedure),
            Some(identity)
        );
    }

    #[test]
    fn anonymous_identity_retains_no_invented_name() {
        let text = "main :: () { callback := (value:int) => value; }";
        let module = syntax::parse(text).unwrap();
        let syntax::StatementKind::Declare(syntax::Declaration::Inferred {
            initializer, ..
        }) = &module.procedures()[0].body[0].kind
        else {
            panic!("the fixture contains a real anonymous lambda");
        };
        assert!(matches!(
            initializer.kind,
            syntax::ExpressionKind::ShortLambda(_)
        ));
        let source = SourceMap::default().insert("anonymous.jai".into(), text.into());
        let mut types = TypeRegistry::new();
        let int = types.scalar(ScalarType::Int(IntegerType::S64));
        let ty = types
            .procedure(ProcedureType {
                parameters: vec![int].into(),
                results: vec![int].into(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let procedure = ProcedureId::new(1);
        let anonymous = ProcedureSourceIdentity {
            name: None,
            file: None,
            location: SourceSpan {
                source,
                span: initializer.span,
            },
            ty,
        };
        let mut registry = LocalDeclarationRegistry::default();
        registry
            .retain_procedure_source(procedure, anonymous)
            .unwrap();
        assert_eq!(
            registry.procedure_source_identity(procedure).unwrap().name,
            None
        );
    }
}
