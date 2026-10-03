//! Nested source procedures share the program's allocator and checked body arena.
use super::*;

impl Resolver<'_> {
    pub(super) fn reserve_local_procedure(
        &mut self,
        id: LocalDeclarationId,
        span: Span,
    ) -> Result<ProcedureId, Diagnostic> {
        if let Some(procedure) = self.meta.local_declarations.entries[&id].procedure {
            return Ok(procedure);
        }
        let procedure = if let Some(scope) = self.graph_scope {
            scope.reserve_local_procedure()?
        } else {
            let next = self
                .meta
                .local_declarations
                .next_legacy_procedure
                .get_or_insert_with(|| {
                    self.signatures
                        .values()
                        .map(|signature| signature.id.index())
                        .max()
                        .map_or(0, |index| index + 1)
                });
            let procedure = ProcedureId::new(*next);
            *next = next
                .checked_add(1)
                .ok_or_else(|| Diagnostic::new(span, "procedure identity space exhausted"))?;
            procedure
        };
        self.meta
            .local_declarations
            .entries
            .get_mut(&id)
            .unwrap()
            .procedure = Some(procedure);
        self.remember_local_procedure_origin(id, procedure, span);
        Ok(procedure)
    }

    pub(super) fn define_local_procedure(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::Procedure,
    ) -> Result<Binding, Diagnostic> {
        if let Some(modifier) = &source.modify {
            return Err(Diagnostic::new(
                modifier.span,
                "local procedure #modify requires checked modifier execution",
            ));
        }
        if source.compiler.is_some() {
            return Err(Diagnostic::new(
                source.span,
                "local compiler procedures require provider registration in their defining file",
            ));
        }
        if source.expands {
            return self.define_local_macro(id, source);
        }
        let signature = self.local_signature(
            id,
            CallableSource {
                parameters: &source.parameters,
                results: &source.results,
                convention: source.convention,
                context: source.context,
                span: source.span,
            },
        )?;
        self.define_prepared_local_procedure(id, source, signature)
    }

    pub(super) fn define_prepared_local_procedure(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::Procedure,
        signature: Signature,
    ) -> Result<Binding, Diagnostic> {
        let phase = self.local_method_phase(id);
        self.define_prepared_local_procedure_in_phase(id, source, signature, phase)
    }

    pub(super) fn selected_local_procedure_phase(
        &self,
        id: LocalDeclarationId,
        procedure: ProcedureId,
    ) -> MethodPhase {
        let LexicalScopeOwner::Record(owner) = id.scope.owner else {
            return MethodPhase::Bodies;
        };
        let phase = self
            .meta
            .local_declarations
            .method_phases
            .get(&owner)
            .copied()
            .unwrap_or(MethodPhase::Bodies);
        if phase == MethodPhase::TypesOnly {
            return phase;
        }
        if self
            .meta
            .local_declarations
            .method_body_demand
            .permits(Some(procedure))
        {
            phase
        } else {
            MethodPhase::CompleteHeaders
        }
    }

    pub(super) fn define_selected_local_procedure(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::Procedure,
        signature: Signature,
    ) -> Result<Binding, Diagnostic> {
        let phase = self.selected_local_procedure_phase(id, signature.id);
        if matches!(id.scope.owner, LexicalScopeOwner::Record(_))
            && !self
                .meta
                .local_declarations
                .method_body_demand
                .permits(Some(signature.id))
            && let Some(context) = self.compile_time
        {
            context.record_pending(vec![jai_vm::Dependency::Procedure(signature.id)]);
        }
        self.define_prepared_local_procedure_in_phase(id, source, signature, phase)
    }

    fn define_prepared_local_procedure_in_phase(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::Procedure,
        signature: Signature,
        phase: MethodPhase,
    ) -> Result<Binding, Diagnostic> {
        self.remember_local_callable_source(id, &signature, source.name)?;
        self.remember_local_deprecation(
            crate::deprecation_warnings::DeprecationKey::Procedure(signature.id),
            source.name,
            source.deprecation.as_ref(),
            crate::deprecation_warnings::procedure_extent(source),
            self.debug.source().or(id.defining_source()),
        )?;
        if let Some(operator) = source.operator {
            self.validate_source_operator_signature(
                operator.kind,
                &signature,
                &source.parameters,
                source.span,
            )?;
        }
        self.meta
            .remember_inline_hint(signature.id, source.inline_hint);
        self.meta.remember_execution(signature.id, source.execution);
        let binding = Binding::Procedure {
            procedure: signature.id,
            ty: signature.ty,
        };
        if source.operator.is_none() {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(source.name, binding.clone());
        }
        if phase != MethodPhase::Bodies {
            return Ok(binding);
        }
        if self
            .meta
            .local_declarations
            .procedures
            .contains_key(&signature.id)
        {
            return Ok(binding);
        }
        let body_check = self
            .meta
            .local_declarations
            .begin_callback_body_check(signature.id, source.span)?;
        let procedure = self.lower_source_procedure_body(SourceProcedureDefinition {
            signature: &signature,
            source: &source.source,
            name: Some(source.name),
        })?;
        if !self.meta.local_declarations.publish_callback_body(
            body_check,
            procedure,
            source.span,
        )? {
            self.meta.debug_sources.clear_procedure(signature.id);
            self.meta.storage_alignments.clear_procedure(signature.id);
        }
        Ok(binding)
    }

    pub(super) fn define_local_prototype(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::ProcedurePrototype,
    ) -> Result<Binding, Diagnostic> {
        if matches!(source.binding, syntax::PrototypeBinding::EntryPoint) {
            let entry = self
                .graph_scope
                .ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "#entry_point requires the application's source graph",
                    )
                })?
                .program_entry(source.span)?;
            let descriptor = self
                .types
                .procedure_definition(entry.ty)
                .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
            if !source.parameters.is_empty()
                || !source.results.is_empty()
                || !descriptor.parameters.is_empty()
                || !descriptor.results.is_empty()
                || descriptor.convention != source.convention
                || descriptor.context != source.context
            {
                return Err(Diagnostic::new(
                    source.span,
                    "local #entry_point alias requires the application's matching parameterless void entry signature",
                ));
            }
            self.meta
                .local_declarations
                .signatures
                .insert(entry.id, entry.clone());
            return Ok(Binding::Procedure {
                procedure: entry.id,
                ty: entry.ty,
            });
        }
        let signature = self.local_signature(
            id,
            CallableSource {
                parameters: &source.parameters,
                results: &source.results,
                convention: source.convention,
                context: source.context,
                span: source.span,
            },
        )?;
        self.remember_local_callable_source(id, &signature, source.name)?;
        self.remember_local_deprecation(
            crate::deprecation_warnings::DeprecationKey::Procedure(signature.id),
            source.name,
            source.deprecation.as_ref(),
            source.span,
            self.debug.source().or(id.defining_source()),
        )?;
        let origin = match &source.binding {
            syntax::PrototypeBinding::EntryPoint => {
                unreachable!("entry alias was bound without a prototype")
            }
            syntax::PrototypeBinding::Foreign(binding)
                if source.convention == jai_types::CallingConvention::Jai =>
            {
                if binding.library.is_some() || binding.symbol.is_some() {
                    return Err(Diagnostic::new(
                        source.span,
                        "Jai source contracts cannot carry native library or symbol bindings",
                    ));
                }
                PrototypeOrigin::SourceContract {
                    symbol: self.symbols.name(source.name).to_owned(),
                }
            }
            syntax::PrototypeBinding::Foreign(binding) => PrototypeOrigin::Foreign {
                symbol: binding
                    .symbol
                    .clone()
                    .unwrap_or_else(|| self.symbols.name(source.name).to_owned()),
                library: binding
                    .library
                    .as_ref()
                    .map(|path| self.foreign_library_path(path, source.span))
                    .transpose()?,
            },
            syntax::PrototypeBinding::Compiler(_) => {
                return Err(Diagnostic::new(
                    source.span,
                    "local compiler prototypes require provider registration in their defining file",
                ));
            }
            syntax::PrototypeBinding::Intrinsic { .. } => {
                crate::modules::runtime_intrinsics::bind_prototype(
                    source,
                    self.symbols.name(source.name),
                    &signature,
                    self.types,
                    self.target_layout,
                )?
                .origin
            }
        };
        self.remember_procedure_notes(signature.id, &source.notes)?;
        self.meta.local_declarations.prototypes.insert(
            signature.id,
            ProcedurePrototype {
                id: signature.id,
                signature: signature.ty,
                origin,
            },
        );
        Ok(Binding::Procedure {
            procedure: signature.id,
            ty: signature.ty,
        })
    }

    pub(crate) fn ready_local_callable_signature(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<Signature>, Diagnostic> {
        let binding = if let Some(binding) = self.lexical_graph_binding_ready(path, span)? {
            let scope = self.graph_scope.ok_or_else(|| {
                Diagnostic::new(span, "imported callables require a source graph")
            })?;
            if scope.imported_callable(binding, span).is_ok() {
                let value = scope.imported_value(binding, span)?;
                if matches!(value, Binding::Imported(_)) {
                    // Genuine graph templates and overload groups need the
                    // common source candidate selector, not a scalar value.
                    return Ok(None);
                }
                Some(value)
            } else {
                Some(self.imported_binding_value_ready(binding, span)?)
            }
        } else if let Some((&member, owner_members)) = path.members.split_last() {
            self.local_ready_type_path(&syntax::NamePath {
                root: path.root,
                members: owner_members.to_vec(),
            })
            .and_then(|owner| self.ready_namespace_member(owner, member))
        } else {
            let mut binding = None;
            for depth in (0..self.scopes.len()).rev() {
                if let Some(value) = self.scopes[depth].get(&path.root) {
                    binding = Some(value.clone());
                    break;
                }
                if let Some(frame) = self.local_scopes.frames.get(depth) {
                    if frame.declarations.contains_key(&path.root)
                        || frame.runtime.contains_key(&path.root)
                        || frame.runtime_symbols.contains(&path.root)
                    {
                        break;
                    }
                    if let Some(value) = frame.using_bindings.get(&path.root) {
                        binding = Some(value.clone());
                        break;
                    }
                    if frame.using_pending.contains(&path.root) {
                        return Err(Diagnostic::new(
                            span,
                            "using place alias is unavailable before its source statement",
                        ));
                    }
                }
            }
            binding
        };
        let Some(Binding::Procedure { procedure, .. }) = binding else {
            return Ok(None);
        };
        Ok(self
            .meta
            .local_declarations
            .signature(procedure)
            .or_else(|| {
                self.signatures
                    .values()
                    .find(|signature| signature.id == procedure)
            })
            .cloned()
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.callback_procedure_signature(procedure))
            }))
    }

    pub(crate) fn local_callable_signature(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<Signature>, Diagnostic> {
        let binding = if let Some(binding) = self.lexical_graph_binding(path, span)? {
            Some(
                self.graph_scope
                    .ok_or_else(|| {
                        Diagnostic::new(span, "imported callables require a source graph")
                    })?
                    .imported_value(binding, span)?,
            )
        } else if path.members.is_empty() {
            self.resolve_local_name(path.root, span)?
        } else {
            self.namespace_binding(path, span)?
        };
        let Some(binding) = binding else {
            return Ok(None);
        };
        let Binding::Procedure { procedure, .. } = binding else {
            return Ok(None);
        };
        self.meta
            .local_declarations
            .signature(procedure)
            .or_else(|| {
                self.signatures
                    .values()
                    .find(|signature| signature.id == procedure)
            })
            .cloned()
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.callback_procedure_signature(procedure))
            })
            .map(Some)
            .ok_or_else(|| Diagnostic::new(span, "local procedure signature is not registered"))
    }
}
