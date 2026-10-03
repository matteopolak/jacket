//! Record headers complete before checked bodies publish into their namespace.
use super::*;
use crate::modules::aggregates::parameterized::{RecordMethod, RecordMethodSource};

struct MethodSources<'a> {
    owner: TypeId,
    methods: &'a [RecordMethod],
    bindings: Vec<(Symbol, Binding)>,
    file: FileInstanceId,
    substitution: &'a crate::polymorphism::Substitution,
}

/// A header pass requests only the source bodies needed by actual VM jobs.
pub(crate) struct RecordMethodPrerequisites<'a> {
    pub owner: TypeId,
    pub methods: &'a [RecordMethod],
    pub bindings: Vec<(Symbol, Binding)>,
    pub file: FileInstanceId,
    pub substitution: &'a crate::polymorphism::Substitution,
    pub required: &'a HashSet<ProcedureId>,
}

#[derive(Default)]
pub(super) enum MethodBodyDemand {
    #[default]
    All,
    Prerequisites(HashSet<ProcedureId>),
}

impl MethodBodyDemand {
    pub(super) fn permits(&self, procedure: Option<ProcedureId>) -> bool {
        match self {
            Self::All => true,
            Self::Prerequisites(required) => procedure.is_some_and(|id| required.contains(&id)),
        }
    }

    pub(super) fn covers_all(&self) -> bool {
        matches!(self, Self::All)
    }
}

impl Resolver<'_> {
    pub(crate) fn bind_pending_local_record_methods(&mut self) -> Result<(), Diagnostic> {
        let mut visited = HashSet::new();
        let mut pending_error = None;
        loop {
            let mut owners: Vec<_> = self
                .meta
                .local_declarations
                .pending_local_method_records
                .iter()
                .filter(|owner| !visited.contains(*owner))
                .copied()
                .collect();
            if owners.is_empty() {
                break;
            }
            owners.sort_by_key(|owner| owner.index());
            for owner in owners {
                if visited.len() == 65_536 {
                    return Err(Diagnostic::new(
                        self.span,
                        "local record method binding exceeds declaration budget",
                    ));
                }
                visited.insert(owner);
                let environment = self
                    .meta
                    .local_declarations
                    .namespace_sources
                    .get(&owner)
                    .cloned()
                    .ok_or_else(|| {
                        Diagnostic::new(
                            self.span,
                            "pending record methods have no definition environment",
                        )
                    })?;
                let result =
                    self.with_local_source_environment(&environment, self.span, |definition| {
                        let depth = definition
                            .local_scopes
                            .frames
                            .len()
                            .checked_sub(1)
                            .ok_or_else(|| {
                                Diagnostic::new(
                                    definition.span,
                                    "pending record methods have no retained lexical namespace",
                                )
                            })?;
                        let mut declarations: Vec<_> = definition.local_scopes.frames[depth]
                            .declarations
                            .values()
                            .chain(definition.local_scopes.frames[depth].operators.iter())
                            .filter(|declaration| {
                                matches!(
                                    declaration.syntax,
                                    DeclarationSyntax::Procedure(_)
                                        | DeclarationSyntax::Prototype(_)
                                ) && definition.record_method_body_requested(declaration.id)
                            })
                            .cloned()
                            .collect();
                        declarations.sort_by_key(|declaration| declaration.id.ordinal);
                        definition
                            .meta
                            .local_declarations
                            .method_phases
                            .insert(owner, MethodPhase::CompleteHeaders);
                        definition.resolve_local_declaration_batch(depth, &declarations)?;
                        definition
                            .meta
                            .local_declarations
                            .method_phases
                            .insert(owner, MethodPhase::Bodies);
                        definition.resolve_local_declaration_batch(depth, &declarations)?;
                        definition.publish_record_source_namespace(owner);
                        Ok(())
                    });
                self.meta.local_declarations.method_phases.remove(&owner);
                match result {
                    Ok(()) if self.meta.local_declarations.method_body_demand.covers_all() => {
                        self.meta
                            .local_declarations
                            .pending_local_method_records
                            .remove(&owner);
                    }
                    Ok(()) => {}
                    Err(error) if self.local_declaration_pending() => {
                        pending_error.get_or_insert(error);
                    }
                    Err(error) => return Err(error),
                }
            }
        }
        pending_error.map_or(Ok(()), Err)
    }

    pub(crate) fn with_record_method_prerequisites<R>(
        &mut self,
        required: &HashSet<ProcedureId>,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = std::mem::replace(
            &mut self.meta.local_declarations.method_body_demand,
            MethodBodyDemand::Prerequisites(required.clone()),
        );
        let result = operation(self);
        self.meta.local_declarations.method_body_demand = previous;
        result
    }

    pub(crate) fn bind_record_method_prerequisites(
        &mut self,
        request: RecordMethodPrerequisites<'_>,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        self.with_record_method_prerequisites(request.required, |definition| {
            definition.bind_record_method_phase(
                MethodSources {
                    owner: request.owner,
                    methods: request.methods,
                    bindings: request.bindings,
                    file: request.file,
                    substitution: request.substitution,
                },
                MethodPhase::Bodies,
            )
        })
    }

    pub(crate) fn bind_record_methods(
        &mut self,
        owner: TypeId,
        methods: &[RecordMethod],
        bindings: Vec<(Symbol, Binding)>,
        file: FileInstanceId,
        substitution: &crate::polymorphism::Substitution,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        self.bind_record_method_phase(
            MethodSources {
                owner,
                methods,
                bindings,
                file,
                substitution,
            },
            MethodPhase::Bodies,
        )
    }

    pub(crate) fn bind_record_method_signatures(
        &mut self,
        owner: TypeId,
        methods: &[RecordMethod],
        bindings: Vec<(Symbol, Binding)>,
        file: FileInstanceId,
        substitution: &crate::polymorphism::Substitution,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        self.bind_record_method_phase(
            MethodSources {
                owner,
                methods,
                bindings,
                file,
                substitution,
            },
            MethodPhase::TypesOnly,
        )
    }

    pub(crate) fn complete_record_method_signatures(
        &mut self,
        owner: TypeId,
        methods: &[RecordMethod],
        bindings: Vec<(Symbol, Binding)>,
        file: FileInstanceId,
        substitution: &crate::polymorphism::Substitution,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        self.bind_record_method_phase(
            MethodSources {
                owner,
                methods,
                bindings,
                file,
                substitution,
            },
            MethodPhase::CompleteHeaders,
        )
    }

    fn bind_record_method_phase(
        &mut self,
        sources: MethodSources<'_>,
        phase: MethodPhase,
    ) -> Result<HashMap<Symbol, Binding>, Diagnostic> {
        let MethodSources {
            owner,
            methods,
            bindings,
            file,
            substitution,
        } = sources;
        if self
            .meta
            .local_declarations
            .checked_method_records
            .contains(&owner)
            || self
                .meta
                .local_declarations
                .active_method_records
                .contains(&owner)
        {
            return Ok(self
                .meta
                .local_declarations
                .record_namespaces
                .get(&owner)
                .cloned()
                .unwrap_or_default());
        }
        let mut scope = self
            .graph_scope
            .ok_or_else(|| {
                Diagnostic::new(
                    self.span,
                    "record methods require their defining source file",
                )
            })?
            .code_file(file);
        scope.substitution = Some(substitution);
        let scope_id = LexicalScopeId {
            owner: LexicalScopeOwner::Record(owner),
            file: Some(file),
            source: Some(scope.source()),
            ordinal: 0,
        };
        let mut frame = ScopeFrame {
            id: scope_id,
            declarations: HashMap::new(),
            operators: vec![],
            operator_imports: vec![],
            runtime: HashMap::new(),
            external: HashMap::new(),
            runtime_symbols: HashSet::new(),
            imports: imports::ScopedImportEnvironment::default(),
            import_aliases: HashSet::new(),
            using_bindings: HashMap::new(),
            using_pending: HashSet::new(),
            using_placeholders: HashMap::new(),
            using_origins: HashMap::new(),
            using_operator_declarations: vec![],
        };
        let bindings: HashMap<_, _> = bindings.into_iter().collect();
        for method in methods {
            if method.id.owner != owner || method.file != file {
                return Err(Diagnostic::new(
                    method.source.span(),
                    "record method source ownership does not match its namespace",
                ));
            }
            let syntax = match &method.source {
                RecordMethodSource::Procedure(source) => {
                    DeclarationSyntax::Procedure(source.clone())
                }
                RecordMethodSource::Prototype(source) => {
                    DeclarationSyntax::Prototype(source.clone())
                }
                RecordMethodSource::Constant(source) => DeclarationSyntax::Constant(source.clone()),
            };
            let name = syntax.name();
            let span = syntax.span();
            if bindings.contains_key(&name) || frame.declarations.contains_key(&name) {
                return Err(Diagnostic::new(
                    span,
                    "duplicate record method namespace member",
                ));
            }
            let id = LocalDeclarationId {
                scope: scope_id,
                start: span.start,
                end: span.end,
                ordinal: method.id.member,
            };
            self.meta.local_declarations.entries.entry(id).or_default();
            frame.declarations.insert(
                name,
                Declaration {
                    id,
                    syntax,
                    checks: crate::safety_checks::ActiveChecks::default(),
                    imports: imports::ScopedImportEnvironment::default(),
                    operator_imports: vec![],
                    using: imports::UsingPrefixEnvironment::default(),
                },
            );
        }
        self.meta
            .local_declarations
            .namespace_scopes
            .insert(owner, scope_id);
        self.meta
            .local_declarations
            .active_method_records
            .insert(owner);
        self.meta
            .local_declarations
            .method_phases
            .insert(owner, MethodPhase::TypesOnly);
        let body_owner = self.lexical_owner();
        let child_context = self
            .compile_time
            .map(|context| context.for_source(context.owner, file, scope.source()));
        let debug_policy = self.debug.policy();
        let result = (|| {
            let mut child = Resolver {
                debug: crate::debug_capture::Capture::new(Some(scope.source())),
                checks: crate::safety_checks::ActiveChecks::default(),
                context: self.context,
                context_available: self.context_available,
                meta: &mut *self.meta,
                graph_scope: Some(scope),
                compile_time: child_context.as_ref(),
                target_layout: self.target_layout,
                procedure: self.procedure,
                expression_owner: self.expression_owner,
                types: &mut *self.types,
                places: &mut *self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes: vec![bindings],
                local_scopes: LocalScopes {
                    frames: vec![frame],
                    next_scope: 1,
                    body_owner: Some(body_owner),
                    annotation_owner: NominalAnnotationContext::None,
                    active: HashSet::new(),
                },
                globals: self.globals,
                locals: vec![],
                span: methods
                    .first()
                    .map_or(self.span, |method| method.source.span()),
                results: &[],
                loops: vec![],
                next_loop: 0,
                cleanups: vec![],
                active_push: None,
                next_push: 0,
                deferred_scopes: vec![],
                cleanup_context: None,
            };
            child.debug.enter_policy(debug_policy);
            child.resolve_registered_local_declarations()?;
            let declarations: Vec<_> = methods
                .iter()
                .map(|method| {
                    child.local_scopes.frames[0].declarations[&method.source.name()].clone()
                })
                .collect();
            for declaration in &declarations {
                if matches!(&declaration.syntax, DeclarationSyntax::Procedure(source) if source.expands)
                {
                    child.resolve_local_declaration(0, declaration)?;
                }
            }
            child.publish_record_source_namespace(owner);
            if phase == MethodPhase::TypesOnly {
                return Ok(child.scopes[0].clone());
            }
            child
                .meta
                .local_declarations
                .method_phases
                .insert(owner, MethodPhase::CompleteHeaders);
            let mut completed = Vec::new();
            let mut pending_error = None;
            for declaration in &declarations {
                if !matches!(
                    declaration.syntax,
                    DeclarationSyntax::Procedure(_) | DeclarationSyntax::Prototype(_)
                ) || !child.record_method_body_requested(declaration.id)
                {
                    continue;
                }
                match child.resolve_local_declaration(0, declaration) {
                    Ok(_) => completed.push(declaration.clone()),
                    Err(error) if child.local_declaration_pending() => {
                        pending_error.get_or_insert(error);
                    }
                    Err(error) => return Err(error),
                }
            }
            child.publish_record_source_namespace(owner);
            if phase == MethodPhase::Bodies {
                child
                    .meta
                    .local_declarations
                    .method_phases
                    .insert(owner, MethodPhase::Bodies);
                child.resolve_local_declaration_batch(0, &completed)?;
                child.publish_record_source_namespace(owner);
            }
            if let Some(error) = pending_error {
                return Err(error);
            }
            if phase == MethodPhase::Bodies
                && child
                    .meta
                    .local_declarations
                    .method_body_demand
                    .covers_all()
            {
                child
                    .meta
                    .local_declarations
                    .checked_method_records
                    .insert(owner);
            }
            Ok(child.scopes[0].clone())
        })();
        if let (Some(parent), Some(child)) = (self.compile_time, child_context.as_ref()) {
            parent.merge_pending_from(child);
        }
        self.meta.local_declarations.method_phases.remove(&owner);
        self.meta
            .local_declarations
            .active_method_records
            .remove(&owner);
        result
    }

    fn publish_record_source_namespace(&mut self, owner: TypeId) {
        self.meta
            .local_declarations
            .record_namespaces
            .insert(owner, self.scopes.last().unwrap().clone());
        let environment = self.capture_local_source_environment();
        self.meta
            .local_declarations
            .namespace_sources
            .insert(owner, environment.into());
    }

    fn local_declaration_pending(&self) -> bool {
        self.compile_time.is_some_and(|context| {
            !context.pending.borrow().is_empty()
                || !context.pending_constants.borrow().is_empty()
                || !context.pending_field_defaults.borrow().is_empty()
        })
    }

    pub(super) fn resolve_local_declaration_batch(
        &mut self,
        depth: usize,
        declarations: &[Declaration],
    ) -> Result<(), Diagnostic> {
        let mut pending_error = None;
        for declaration in declarations {
            if matches!(&declaration.syntax, DeclarationSyntax::Procedure(source)
                if source.operator.is_some() && crate::polymorphism::is_polymorphic(source))
            {
                continue;
            }
            if let Err(error) = self.resolve_local_declaration(depth, declaration) {
                if !self.local_declaration_pending() {
                    return Err(error);
                }
                pending_error.get_or_insert(error);
            }
        }
        match pending_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(super) fn finish_local_record_dependencies(
        &mut self,
        owner: TypeId,
        depth: usize,
        methods: &[Declaration],
        members: &[syntax::RecordMember],
    ) -> Result<(), Diagnostic> {
        loop {
            let before = self.local_record_dependency_progress();
            let mut pending = self.finish_local_record_defaults(owner).err();
            if pending.is_none() {
                pending = self
                    .apply_local_record_default_overrides(owner, members)
                    .err();
            }
            let mut body_error = None;
            for method in methods {
                if matches!(&method.syntax, DeclarationSyntax::Procedure(source)
                    if source.operator.is_some() && crate::polymorphism::is_polymorphic(source))
                {
                    continue;
                }
                if let Err(error) = self.resolve_local_declaration(depth, method) {
                    if self.meta.local_declarations.entries[&method.id]
                        .procedure
                        .is_some_and(|procedure| {
                            self.meta
                                .local_declarations
                                .header_readiness
                                .get(&procedure)
                                == Some(&HeaderReadiness::Complete)
                        })
                    {
                        body_error.get_or_insert_with(|| error.clone());
                    }
                    pending.get_or_insert(error);
                }
            }
            match pending {
                None => return Ok(()),
                Some(error) if self.local_record_dependency_progress() == before => {
                    return Err(body_error.unwrap_or(error));
                }
                Some(_) => {}
            }
        }
    }

    fn local_record_dependency_progress(&self) -> (usize, usize, usize, usize) {
        let declarations = &self.meta.local_declarations;
        (
            declarations.defaults.len(),
            declarations.record_overrides_ready.len(),
            declarations
                .header_readiness
                .values()
                .filter(|&&readiness| readiness == HeaderReadiness::Complete)
                .count(),
            declarations.semantic_ready_count(),
        )
    }
}
