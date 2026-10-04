//! Retain declaration environments for contextual source values and type aliases.
use super::*;

#[derive(Clone)]
pub(super) struct SourceEnvironment {
    bindings: Vec<HashMap<Symbol, Binding>>,
    scopes: LocalScopes,
    checks: crate::safety_checks::ActiveChecks,
    debug: jai_types::DebugPolicy,
    procedure: ProcedureId,
    expression_owner: Option<ProcedureId>,
    file: Option<FileInstanceId>,
    source: Option<SourceId>,
    substitution: Option<crate::polymorphism::Substitution>,
}

enum SourceEffects {
    Inherited,
    Isolated(ProcedureId),
}

impl Resolver<'_> {
    pub(crate) fn with_record_source_environment<T>(
        &mut self,
        owner: TypeId,
        span: Span,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        if !self
            .meta
            .local_declarations
            .namespace_sources
            .contains_key(&owner)
        {
            // Even an empty source method list has a genuine record owner,
            // defining file, and substitution in the reserved environment.
            self.prepare_record_source_namespace(owner)?;
        }
        let environment = self
            .meta
            .local_declarations
            .namespace_sources
            .get(&owner)
            .cloned()
            .ok_or_else(|| Diagnostic::new(span, "record definition environment is unavailable"))?;
        self.with_local_source_environment(&environment, span, evaluate)
    }

    pub(crate) fn record_callback_alias(
        &self,
        owner: TypeId,
        name: Symbol,
    ) -> Option<crate::procedure_values::contracts::ContractSyntax> {
        let scope = self.meta.local_declarations.namespace_scopes.get(&owner)?;
        let frame = self
            .local_scopes
            .frames
            .iter()
            .find(|frame| frame.id == *scope)
            .or_else(|| {
                self.meta
                    .local_declarations
                    .namespace_sources
                    .get(&owner)?
                    .scopes
                    .frames
                    .last()
            })?;
        let declaration = frame.declarations.get(&name)?;
        if let Some(source) = self
            .meta
            .local_declarations
            .alias_sources
            .get(&declaration.id)
        {
            return Some(source.clone());
        }
        match &declaration.syntax {
            DeclarationSyntax::Alias(alias) => {
                self.retained_callback_syntax(&alias.ty, alias.span).ok()
            }
            DeclarationSyntax::Constant(constant) if constant.ty.is_none() => {
                crate::procedure_values::contracts::callback_source_type(&constant.initializer)
                    .and_then(|source| {
                        self.retained_callback_syntax(&source, constant.initializer.span)
                            .ok()
                    })
            }
            _ => None,
        }
    }
    pub(crate) fn local_callback_alias(
        &self,
        path: &syntax::NamePath,
    ) -> Option<crate::procedure_values::contracts::ContractSyntax> {
        let declaration = if path.members.is_empty() {
            let mut declaration = None;
            for depth in (0..self.scopes.len()).rev() {
                if let Some(source) = self
                    .local_scopes
                    .frames
                    .get(depth)
                    .and_then(|frame| frame.declarations.get(&path.root))
                {
                    declaration = Some(source);
                    break;
                }
                if self.scopes[depth].contains_key(&path.root) {
                    break;
                }
            }
            declaration
        } else {
            let (&member, owner_members) = path.members.split_last()?;
            let owner = self.local_ready_type_path(&syntax::NamePath {
                root: path.root,
                members: owner_members.to_vec(),
            })?;
            let active_scope = self.meta.local_declarations.namespace_scopes.get(&owner)?;
            self.local_scopes
                .frames
                .iter()
                .find(|frame| frame.id == *active_scope)
                .or_else(|| {
                    self.meta
                        .local_declarations
                        .namespace_sources
                        .get(&owner)?
                        .scopes
                        .frames
                        .last()
                })?
                .declarations
                .get(&member)
        }?;
        if let Some(source) = self
            .meta
            .local_declarations
            .alias_sources
            .get(&declaration.id)
        {
            return Some(source.clone());
        }
        match &declaration.syntax {
            DeclarationSyntax::Alias(alias) => {
                self.retained_callback_syntax(&alias.ty, alias.span).ok()
            }
            DeclarationSyntax::Constant(constant) if constant.ty.is_none() => {
                crate::procedure_values::contracts::callback_source_type(&constant.initializer)
                    .and_then(|source| {
                        self.retained_callback_syntax(&source, constant.initializer.span)
                            .ok()
                    })
            }
            _ => None,
        }
    }

    pub(crate) fn remember_local_constant_source(&mut self, name: Symbol) {
        let Some(declaration) = self
            .local_scopes
            .frames
            .last()
            .and_then(|frame| frame.declarations.get(&name))
            .cloned()
        else {
            return;
        };
        let mut environment = self.capture_local_source_environment();
        environment.checks = declaration.checks;
        environment.file = declaration.id.defining_file().or(environment.file);
        environment.source = environment.source.or(declaration.id.defining_source());
        if let Some(frame) = environment.scopes.frames.last_mut() {
            frame.imports = declaration.imports.clone();
            frame.operator_imports = declaration.operator_imports.clone();
            declaration.using.clone().restore(frame);
        }
        self.meta
            .local_declarations
            .constant_sources
            .entry(declaration.id)
            .or_insert(environment);
    }

    pub(super) fn capture_local_source_environment(&self) -> SourceEnvironment {
        SourceEnvironment {
            bindings: self.scopes.clone(),
            scopes: self.local_scopes.clone(),
            checks: self.checks,
            debug: self.debug.policy(),
            procedure: self.procedure,
            expression_owner: self.expression_owner,
            file: self.graph_scope.map(|scope| scope.code_origin().0),
            source: self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source())),
            substitution: self
                .graph_scope
                .and_then(|scope| scope.substitution)
                .cloned(),
        }
    }

    pub(super) fn local_procedure_source_environment(
        &self,
        depth: usize,
        declaration: &Declaration,
    ) -> SourceEnvironment {
        let mut environment = self.capture_local_source_environment();
        environment.bindings.truncate(depth + 1);
        environment.scopes.frames.truncate(depth + 1);
        let frame = &mut environment.scopes.frames[depth];
        frame.imports = declaration.imports.clone();
        frame.operator_imports = declaration.operator_imports.clone();
        declaration.using.clone().restore(frame);
        environment.checks = declaration.checks;
        environment.file = declaration.id.defining_file().or(environment.file);
        environment.source = declaration.id.defining_source().or(environment.source);
        environment
    }

    pub(super) fn with_local_procedure_substitution<T>(
        &mut self,
        environment: &SourceEnvironment,
        substitution: &crate::polymorphism::Substitution,
        span: Span,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        let mut environment = environment.clone();
        let mut combined = environment.substitution.clone().unwrap_or_default();
        for binding in &substitution.types {
            combined
                .types
                .retain(|previous| previous.name != binding.name);
            combined.types.push(binding.clone());
        }
        for binding in &substitution.constants {
            combined
                .constants
                .retain(|previous| previous.name != binding.name);
            combined.constants.push(binding.clone());
        }
        for binding in &substitution.callables {
            combined.callables.retain(|previous| {
                previous.name != binding.name || previous.occurrence != binding.occurrence
            });
            combined.callables.push(binding.clone());
        }
        environment.substitution = Some(combined);
        self.with_local_source_environment(&environment, span, |definition| {
            for binding in &substitution.types {
                definition
                    .scopes
                    .last_mut()
                    .unwrap()
                    .insert(binding.name, Binding::Type(binding.ty));
            }
            for binding in &substitution.constants {
                let value = definition.baked_record_binding(binding.value.clone());
                definition
                    .scopes
                    .last_mut()
                    .unwrap()
                    .insert(binding.name, value);
            }
            evaluate(definition)
        })
    }

    pub(crate) fn with_local_constant_source<T>(
        &mut self,
        path: &syntax::NamePath,
        _span: Span,
        evaluate: impl FnOnce(
            &mut Resolver<'_>,
            LocalDeclarationId,
            &syntax::ConstantDeclaration,
        ) -> Result<Option<T>, Diagnostic>,
    ) -> Result<Option<T>, Diagnostic> {
        let source = if path.members.is_empty() {
            let mut source = None;
            for depth in (0..self.scopes.len()).rev() {
                if let Some(declaration) = self
                    .local_scopes
                    .frames
                    .get(depth)
                    .and_then(|frame| frame.declarations.get(&path.root))
                    .cloned()
                {
                    if matches!(declaration.syntax, DeclarationSyntax::Constant(_)) {
                        let mut environment = self
                            .meta
                            .local_declarations
                            .constant_sources
                            .get(&declaration.id)
                            .cloned()
                            .unwrap_or_else(|| self.capture_local_source_environment());
                        environment.bindings.truncate(depth + 1);
                        environment.scopes.frames.truncate(depth + 1);
                        environment.scopes.frames[depth].imports = declaration.imports.clone();
                        environment.scopes.frames[depth].operator_imports =
                            declaration.operator_imports.clone();
                        declaration
                            .using
                            .clone()
                            .restore(&mut environment.scopes.frames[depth]);
                        environment.checks = declaration.checks;
                        environment.file = declaration.id.defining_file().or(environment.file);
                        environment.source =
                            environment.source.or(declaration.id.defining_source());
                        source = Some((environment, declaration));
                    }
                    break;
                }
                if self.scopes[depth].contains_key(&path.root)
                    || self.local_scopes.frames.get(depth).is_some_and(|frame| {
                        frame.runtime.contains_key(&path.root)
                            || frame.runtime_symbols.contains(&path.root)
                            || frame.imports.contains_key(&path.root)
                            || frame.using_bindings.contains_key(&path.root)
                            || frame.using_pending.contains(&path.root)
                            || frame.using_placeholders.contains_key(&path.root)
                    })
                {
                    break;
                }
            }
            source
        } else {
            let (&member, owner_members) = path.members.split_last().unwrap();
            let owner_path = syntax::NamePath {
                root: path.root,
                members: owner_members.to_vec(),
            };
            let Some(owner) = self.local_ready_type_path(&owner_path) else {
                return Ok(None);
            };
            if !self
                .meta
                .local_declarations
                .namespace_sources
                .contains_key(&owner)
                && self.meta.record_specializations.methods(owner).is_some()
            {
                self.prepare_record_source_namespace(owner)?;
            }
            let Some(environment) = self
                .meta
                .local_declarations
                .namespace_sources
                .get(&owner)
                .cloned()
            else {
                return Ok(None);
            };
            environment
                .scopes
                .frames
                .last()
                .and_then(|frame| frame.declarations.get(&member))
                .filter(|declaration| matches!(declaration.syntax, DeclarationSyntax::Constant(_)))
                .cloned()
                .map(|declaration| (environment.as_ref().clone(), declaration))
        };
        let Some((environment, declaration)) = source else {
            return Ok(None);
        };
        let DeclarationSyntax::Constant(constant) = declaration.syntax else {
            unreachable!("source lookup selected a constant declaration");
        };
        self.with_local_source_environment(&environment, declaration.id.source_span(), |resolver| {
            evaluate(resolver, declaration.id, &constant)
        })
    }

    pub(super) fn with_local_source_environment<T>(
        &mut self,
        environment: &SourceEnvironment,
        span: Span,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        self.with_local_source_effects(environment, span, SourceEffects::Inherited, evaluate)
    }

    pub(super) fn with_isolated_callable_source<T>(
        &mut self,
        owner: ProcedureId,
        span: Span,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        let environment = self.capture_local_source_environment();
        self.with_local_source_effects(&environment, span, SourceEffects::Isolated(owner), evaluate)
    }

    fn with_local_source_effects<T>(
        &mut self,
        environment: &SourceEnvironment,
        span: Span,
        effects: SourceEffects,
        evaluate: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        let mut scope = self
            .graph_scope
            .map(|scope| environment.file.map_or(scope, |file| scope.code_file(file)));
        if let Some(scope) = scope.as_mut() {
            scope.substitution = environment.substitution.as_ref();
        }
        let isolated_cache = crate::compile_time::Cache::default();
        let mut no_effects = jai_vm::NoEffects;
        let isolated_effects = crate::compile_time::SharedEffects::new(&mut no_effects);
        let child_context = self.compile_time.and_then(|context| {
            let file = environment.file?;
            let source = environment.source?;
            Some(match effects {
                SourceEffects::Inherited => context.for_source(
                    environment.scopes.body_owner().unwrap_or(context.owner),
                    file,
                    source,
                ),
                SourceEffects::Isolated(owner) => context.isolated_for_source(
                    owner,
                    file,
                    source,
                    &isolated_cache,
                    &isolated_effects,
                ),
            })
        });
        let result = {
            let debug_policy = self.debug.policy().nested(environment.debug);
            let mut child = Resolver {
                conditional_subjects: Vec::new(),
                debug: crate::debug_capture::Capture::new(environment.source),
                checks: environment.checks,
                context: self.context,
                context_available: self.context_available,
                meta: &mut *self.meta,
                graph_scope: scope,
                compile_time: child_context.as_ref(),
                target_layout: self.target_layout,
                procedure: environment.procedure,
                expression_owner: environment.expression_owner,
                types: &mut *self.types,
                places: &mut *self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes: environment.bindings.clone(),
                local_scopes: environment.scopes.clone(),
                globals: self.globals,
                locals: vec![],
                span,
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
            evaluate(&mut child)
        };
        if let (Some(parent), Some(child)) = (self.compile_time, child_context.as_ref()) {
            parent.merge_pending_from(child);
        }
        result.map_err(|error| match environment.source {
            Some(source) => error.with_fallback_source(source),
            None => error,
        })
    }

    pub(crate) fn local_ready_type_path(&self, path: &syntax::NamePath) -> Option<TypeId> {
        if let Ok(Some(binding)) = self.lexical_graph_binding_ready(path, Span::default()) {
            return self
                .graph_scope?
                .imported_type(binding, Span::default())
                .ok();
        }
        let mut ty = None;
        for depth in (0..self.scopes.len()).rev() {
            if let Some(binding) = self.scopes[depth].get(&path.root) {
                ty = self.ready_type_binding(binding);
                break;
            }
            if let Some(frame) = self.local_scopes.frames.get(depth) {
                if let Some(declaration) = frame.declarations.get(&path.root) {
                    let entry = self.meta.local_declarations.entries.get(&declaration.id)?;
                    ty = entry
                        .binding
                        .as_ref()
                        .and_then(|binding| self.ready_type_binding(binding))
                        .or(entry.nominal);
                    break;
                }
                if frame.runtime.contains_key(&path.root)
                    || frame.runtime_symbols.contains(&path.root)
                {
                    return None;
                }
                if let Some(binding) = frame.using_bindings.get(&path.root) {
                    ty = self.ready_type_binding(binding);
                    break;
                }
                if let Some(&marker) = frame.using_placeholders.get(&path.root) {
                    let scope = self.graph_scope?;
                    let binding = scope
                        .imported_placeholder_binding(marker, Span::default())
                        .ok()?;
                    ty = scope.imported_type(binding, Span::default()).ok();
                    break;
                }
                if frame.using_pending.contains(&path.root) {
                    return None;
                }
                if let Some(binding) = frame.imports.get(&path.root) {
                    ty = self
                        .graph_scope?
                        .imported_type(*binding, Span::default())
                        .ok();
                    break;
                }
                if let Some(marker) = frame.imports.placeholder(path.root) {
                    let scope = self.graph_scope?;
                    let binding = scope
                        .imported_placeholder_binding(marker, Span::default())
                        .ok()?;
                    ty = scope.imported_type(binding, Span::default()).ok();
                    break;
                }
            }
        }
        let mut ty = ty.or_else(|| {
            if self.local_name_present(path.root) {
                return None;
            }
            self.graph_scope?
                .type_name(
                    &syntax::NamePath {
                        root: path.root,
                        members: vec![],
                    },
                    Span::default(),
                )
                .ok()
        })?;
        for &member in &path.members {
            ty = self.ready_type_binding(&self.ready_namespace_member(ty, member)?)?;
        }
        Some(ty)
    }

    fn ready_type_binding(&self, binding: &Binding) -> Option<TypeId> {
        match binding {
            Binding::Type(ty) => Some(*ty),
            Binding::Imported(binding) => self
                .graph_scope?
                .imported_type(*binding, Span::default())
                .ok(),
            _ => None,
        }
    }
}
