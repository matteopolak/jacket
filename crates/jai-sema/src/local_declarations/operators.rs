//! Lexical operator groups preserve their genuine local declaration origins.
use super::*;

pub(crate) type LocalOperatorCandidate = (
    crate::overloads::Candidate<LocalDeclarationId>,
    Option<Signature>,
    syntax::OperatorDeclaration,
);

impl DeclarationSyntax {
    pub(super) fn operator(&self) -> Option<syntax::OperatorDeclaration> {
        match self {
            Self::Procedure(procedure) => procedure.operator,
            _ => None,
        }
    }
}

impl Resolver<'_> {
    pub(super) fn extend_operator_imports(
        &self,
        modules: &mut Vec<jai_source::ModuleId>,
        import: &syntax::ScopedImportDeclaration,
    ) -> Result<(), Diagnostic> {
        if import.namespace.is_some() && !import.using {
            return Ok(());
        }
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(import.span, "operator imports require a source graph")
        })?;
        let module = scope.scoped_import_module(
            import.span,
            self.meta.source_specialization_keys.get(&self.procedure),
        )?;
        if !modules.contains(&module) {
            modules.push(module);
        }
        Ok(())
    }
    pub(crate) fn local_operator_scope_depth(&self, kind: syntax::OperatorKind) -> Option<usize> {
        self.local_scopes
            .frames
            .iter()
            .enumerate()
            .rev()
            .find_map(|(depth, frame)| {
                let declarations = frame.operators.iter().any(|declaration| {
                    declaration
                        .syntax
                        .operator()
                        .is_some_and(|operator| operator.kind.shares_token(kind))
                });
                let imports = self.graph_scope.is_some_and(|scope| {
                    frame.operator_imports.iter().any(|&module| {
                        scope.using_exported_operators(module).iter().any(
                            |(imported_kind, declarations)| {
                                imported_kind.shares_token(kind) && !declarations.is_empty()
                            },
                        )
                    })
                });
                let selected = self.graph_scope.is_some_and(|scope| {
                    frame.using_operator_declarations.iter().any(|&id| {
                        scope
                            .operator_metadata(id)
                            .is_some_and(|operator| operator.kind.shares_token(kind))
                    })
                });
                (declarations || imports || selected).then_some(depth)
            })
    }
    pub(crate) fn local_using_operator_declarations(
        &self,
        kind: syntax::OperatorKind,
    ) -> Vec<jai_source::DeclarationId> {
        let Some(depth) = self.local_operator_scope_depth(kind) else {
            return vec![];
        };
        let Some(scope) = self.graph_scope else {
            return vec![];
        };
        self.local_scopes.frames[depth]
            .using_operator_declarations
            .iter()
            .copied()
            .filter(|&id| {
                scope
                    .operator_metadata(id)
                    .is_some_and(|operator| operator.kind == kind)
            })
            .collect()
    }
    pub(crate) fn local_operator_imports(
        &self,
        kind: syntax::OperatorKind,
    ) -> Vec<jai_source::ModuleId> {
        self.local_operator_scope_depth(kind)
            .map(|depth| self.local_scopes.frames[depth].operator_imports.clone())
            .unwrap_or_default()
    }
    pub(crate) fn local_operator_candidates(
        &mut self,
        kind: syntax::OperatorKind,
        span: Span,
    ) -> Result<Vec<LocalOperatorCandidate>, Diagnostic> {
        let declarations = self
            .local_operator_scope_depth(kind)
            .into_iter()
            .flat_map(|depth| {
                let frame = &self.local_scopes.frames[depth];
                frame.operators.iter().filter_map(move |declaration| {
                    (declaration
                        .syntax
                        .operator()
                        .is_some_and(|operator| operator.kind == kind))
                    .then_some((depth, declaration.clone()))
                })
            })
            .collect::<Vec<_>>();
        let mut signatures = vec![];
        for (depth, declaration) in declarations {
            let operator = declaration
                .syntax
                .operator()
                .expect("operator group retains metadata");
            if let DeclarationSyntax::Procedure(procedure) = &declaration.syntax
                && crate::polymorphism::is_polymorphic(procedure)
            {
                signatures.push((
                    self.local_generic_operator_candidate(depth, &declaration)?,
                    None,
                    operator,
                ));
                continue;
            }
            let binding = self.resolve_local_declaration(depth, &declaration)?;
            let Binding::Procedure { procedure, .. } = binding else {
                return Err(Diagnostic::new(
                    span,
                    "local operator has no callable signature",
                ));
            };
            let signature = self
                .meta
                .local_declarations
                .signatures
                .get(&procedure)
                .cloned()
                .ok_or_else(|| Diagnostic::new(span, "local operator signature is unavailable"))?;
            self.validate_operator_signature(kind, &signature, declaration.syntax.span())?;
            let candidate = crate::polymorphism::integration::concrete_candidate(
                declaration.id,
                &signature,
                self.types,
            );
            signatures.push((candidate, Some(signature), operator));
        }
        Ok(signatures)
    }
}
