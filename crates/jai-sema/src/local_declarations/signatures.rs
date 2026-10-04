//! Header type reservation never executes the source's default expressions.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MethodPhase {
    TypesOnly,
    CompleteHeaders,
    Bodies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderReadiness {
    TypesOnly,
    Complete,
}

#[derive(Clone, Copy)]
pub(super) struct CallableSource<'a> {
    pub(super) parameters: &'a [syntax::Parameter],
    pub(super) results: &'a [syntax::ProcedureResult],
    pub(super) convention: CallingConvention,
    pub(super) return_abi: jai_types::ForeignReturnAbi,
    pub(super) context: ContextMode,
    pub(super) span: Span,
}

impl Resolver<'_> {
    pub(crate) fn complete_callable_signature(
        &mut self,
        signature: &Signature,
        span: Span,
    ) -> Result<Signature, Diagnostic> {
        if self
            .meta
            .local_declarations
            .header_readiness
            .get(&signature.id)
            == Some(&HeaderReadiness::TypesOnly)
        {
            let declaration = self
                .meta
                .local_declarations
                .callable_declarations
                .get(&signature.id)
                .copied()
                .ok_or_else(|| {
                    Diagnostic::new(span, "reserved callable has no retained source declaration")
                })?;
            if self.local_method_phase(declaration) == MethodPhase::TypesOnly {
                if !self.record_method_body_requested(declaration)
                    && let Some(context) = self.compile_time
                {
                    context.record_pending(vec![jai_vm::Dependency::Procedure(signature.id)]);
                }
                return Err(Diagnostic::new(
                    span,
                    "procedure calls require completed source default metadata",
                ));
            }
            let active = self
                .local_scopes
                .frames
                .iter()
                .enumerate()
                .find_map(|(depth, frame)| {
                    frame
                        .declarations
                        .values()
                        .find(|source| source.id == declaration)
                        .cloned()
                        .map(|source| (depth, source))
                });
            if let Some((depth, source)) = active {
                self.resolve_local_declaration(depth, &source)?;
            } else if let LexicalScopeOwner::Record(owner) = declaration.scope.owner {
                let environment = self
                    .meta
                    .local_declarations
                    .namespace_sources
                    .get(&owner)
                    .cloned()
                    .ok_or_else(|| {
                        Diagnostic::new(span, "callable definition environment is unavailable")
                    })?;
                self.with_local_source_environment(
                    &environment,
                    declaration.source_span(),
                    |definition| {
                        let source = definition
                            .local_scopes
                            .frames
                            .last()
                            .and_then(|frame| {
                                frame
                                    .declarations
                                    .values()
                                    .find(|source| source.id == declaration)
                            })
                            .cloned()
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    span,
                                    "callable source is absent from its definition namespace",
                                )
                            })?;
                        definition
                            .resolve_local_declaration(
                                definition.local_scopes.frames.len() - 1,
                                &source,
                            )
                            .map(|_| ())
                    },
                )?;
            }
            if self
                .meta
                .local_declarations
                .header_readiness
                .get(&signature.id)
                != Some(&HeaderReadiness::Complete)
            {
                return Err(Diagnostic::new(
                    span,
                    "procedure default metadata is not complete",
                ));
            }
        }
        Ok(self
            .meta
            .local_declarations
            .signature(signature.id)
            .cloned()
            .unwrap_or_else(|| signature.clone()))
    }

    pub(super) fn local_method_phase(&self, id: LocalDeclarationId) -> MethodPhase {
        if !self.record_method_body_requested(id) {
            return MethodPhase::TypesOnly;
        }
        match id.scope.owner {
            LexicalScopeOwner::Record(owner) => self
                .meta
                .local_declarations
                .method_phases
                .get(&owner)
                .copied()
                .unwrap_or(MethodPhase::Bodies),
            LexicalScopeOwner::Procedure(_) => MethodPhase::Bodies,
        }
    }

    pub(super) fn record_method_body_requested(&self, id: LocalDeclarationId) -> bool {
        match id.scope.owner {
            LexicalScopeOwner::Record(_) => {
                self.meta.local_declarations.method_body_demand.permits(
                    self.meta
                        .local_declarations
                        .entries
                        .get(&id)
                        .and_then(|entry| entry.procedure),
                )
            }
            LexicalScopeOwner::Procedure(_) => true,
        }
    }

    pub(super) fn local_signature(
        &mut self,
        id: LocalDeclarationId,
        source: CallableSource<'_>,
    ) -> Result<Signature, Diagnostic> {
        let procedure = self.reserve_local_procedure(id, source.span)?;
        let isolated = self.local_method_phase(id) != MethodPhase::TypesOnly
            && self.compile_time.is_some_and(|context| {
                context.effect_mode == crate::compile_time::EffectsMode::Compiler
            })
            && self
                .graph_scope
                .is_some_and(|scope| scope.is_isolated_procedure(procedure));
        if isolated {
            return self.with_isolated_callable_source(procedure, source.span, |definition| {
                definition.local_signature_inner(id, source)
            });
        }
        self.local_signature_inner(id, source)
    }

    fn local_signature_inner(
        &mut self,
        id: LocalDeclarationId,
        source: CallableSource<'_>,
    ) -> Result<Signature, Diagnostic> {
        let procedure = self.reserve_local_procedure(id, source.span)?;
        self.meta
            .local_declarations
            .callable_declarations
            .insert(procedure, id);
        let phase = self.local_method_phase(id);
        let readiness = if phase == MethodPhase::TypesOnly {
            HeaderReadiness::TypesOnly
        } else {
            HeaderReadiness::Complete
        };
        let previous = self
            .meta
            .local_declarations
            .signatures
            .get(&procedure)
            .cloned();
        if let Some(signature) = &previous
            && (readiness == HeaderReadiness::TypesOnly
                || self
                    .meta
                    .local_declarations
                    .header_readiness
                    .get(&procedure)
                    == Some(&HeaderReadiness::Complete))
        {
            return Ok(signature.clone());
        }
        let header = self.source_header_components(
            source,
            SourceHeaderPhase::Definition {
                procedure,
                readiness,
            },
        )?;
        let ty = header.ty;
        if previous.is_some_and(|previous| previous.ty != ty) {
            return Err(Diagnostic::new(
                source.span,
                "completed method header differs from its reserved canonical procedure type",
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
            .insert(procedure, readiness);
        Ok(signature)
    }

    pub(super) fn local_default_type(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        match &expression.kind {
            syntax::ExpressionKind::CallerLocation => self.caller_location_type(expression.span),
            syntax::ExpressionKind::Code(_) => Ok(self.types.code_type()),
            syntax::ExpressionKind::StructLiteral(literal) if literal.ty.is_some() => {
                self.lexical_annotation(literal.ty.as_ref().unwrap(), expression.span)
            }
            syntax::ExpressionKind::PositionalStructLiteral(literal) if literal.ty.is_some() => {
                self.lexical_annotation(literal.ty.as_ref().unwrap(), expression.span)
            }
            syntax::ExpressionKind::CompileTime(syntax::CompileTimeRun {
                body: syntax::CompileTimeBody::Expression(value),
                ..
            }) => self.local_default_type(value),
            syntax::ExpressionKind::Call(name, _) => self.local_default_call_type(
                &syntax::NamePath {
                    root: *name,
                    members: vec![],
                },
                expression,
            ),
            syntax::ExpressionKind::QualifiedCall(path, _) => {
                self.local_default_call_type(path, expression)
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                if let Some(ty) = self.local_default_namespace_type(path, expression.span)? {
                    return Ok(ty);
                }
                let description = self.describe_argument(expression)?;
                self.argument_type(&description, expression.span)
            }
            _ => {
                let description = self.describe_argument(expression)?;
                self.argument_type(&description, expression.span)
            }
        }
    }

    fn local_default_call_type(
        &mut self,
        path: &syntax::NamePath,
        expression: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        if let Some(signature) = self.ready_local_callable_signature(path, expression.span)? {
            return match signature.results.as_slice() {
                [result] => Ok(result.ty),
                _ => Err(Diagnostic::new(
                    expression.span,
                    "inferred default requires one procedure result",
                )),
            };
        }
        let description = self.describe_argument(expression)?;
        self.argument_type(&description, expression.span)
    }

    fn local_default_namespace_type(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        let Some((&member, owner_members)) = path.members.split_last() else {
            return Ok(None);
        };
        let Some(owner) = self.local_ready_type_path(&syntax::NamePath {
            root: path.root,
            members: owner_members.to_vec(),
        }) else {
            return Ok(None);
        };
        if self
            .meta
            .local_declarations
            .enum_member_value(owner, member)
            .is_some()
            || self
                .meta
                .record_specializations
                .member_enum(owner)
                .is_some_and(|enumeration| {
                    enumeration.values.iter().any(|(name, _)| *name == member)
                })
            || self
                .graph_scope
                .is_some_and(|scope| scope.enum_member_value(owner, member).is_some())
        {
            return Ok(Some(owner));
        }
        let mut binding = self.ready_namespace_member(owner, member);
        if binding.is_none() {
            let (ty, value) = self
                .meta
                .record_specializations
                .member_bindings(owner)
                .map(|namespace| (namespace.ty(member), namespace.constant(member).cloned()))
                .unwrap_or_default();
            binding = ty.map(Binding::Type);
            if binding.is_none()
                && let Some(value) = value
            {
                binding = Some(self.baked_record_binding(value));
            }
        }
        let Some(binding) = binding else {
            return Ok(None);
        };
        let value = self.binding_expression(binding, span)?;
        self.expression_type(&value, span).map(Some)
    }
}
