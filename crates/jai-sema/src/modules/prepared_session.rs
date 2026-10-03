//! Own semantic arenas and immutable source identity across readiness boundaries.
use super::*;

/// A live source continuation's dependencies, distinct from a failed graph.
#[derive(Clone, Debug)]
pub struct LibraryPending {
    pub dependencies: Vec<jai_vm::Dependency>,
    pub diagnostic: LocatedDiagnostic,
    pub source: Option<source_preparation::SourcePreparationPending>,
}

pub enum SourcePrefixReadiness {
    /// One original run completed; inspect source/configuration deltas now.
    Ready,
    /// All original runs completed under the retained semantic snapshot.
    Complete,
    Pending(LibraryPending),
    Failed(LocatedDiagnostic),
}

pub enum LibraryReadiness {
    Complete(Box<Library>),
    Pending(LibraryPending),
    Failed(LocatedDiagnostic),
}

/// Retains one graph's semantic arenas and source worklist between drive calls.
/// The caller must cancel a pending session before releasing its effects owner.
pub struct PreparedLibrarySession<'graph> {
    phase: Option<Box<PreparedPhase<'graph>>>,
    worklist: Option<compile_time::Worklist<'graph>>,
    location: jai_source::SourceSpan,
    terminal: Option<SessionTerminal>,
    source_prefix_complete: bool,
    full_started: bool,
}

enum SessionTerminal {
    Complete,
    Cancelled,
    Failed(LocatedDiagnostic),
}

impl<'graph> PreparedLibrarySession<'graph> {
    pub fn new(
        graph: &'graph ModuleGraph,
        options: &crate::ResolveOptions,
    ) -> Result<Self, LocatedDiagnostic> {
        let PreparedStart::Phase(phase) = prepare(graph, options, None)? else {
            unreachable!("library preparation has no parameter discovery requests")
        };
        let location = jai_source::SourceSpan {
            source: graph.file(phase.root_file).unwrap().source(),
            span: Span::default(),
        };
        Ok(Self {
            phase: Some(phase),
            worklist: None,
            location,
            terminal: None,
            source_prefix_complete: false,
            full_started: false,
        })
    }

    /// Complete the original source-run prefix before checking unrelated bodies.
    /// The owner may inspect its actual input/configuration journals at Ready.
    pub fn drive_source_prefix(
        &mut self,
        effects: &mut dyn jai_vm::CompilerEffects,
    ) -> SourcePrefixReadiness {
        if let Some(terminal) = &self.terminal {
            return SourcePrefixReadiness::Failed(match terminal {
                SessionTerminal::Failed(error) => error.clone(),
                SessionTerminal::Complete => self.lifecycle_error("already completed"),
                SessionTerminal::Cancelled => self.lifecycle_error("was cancelled"),
            });
        }
        if self.full_started {
            return SourcePrefixReadiness::Failed(
                self.lifecycle_error(
                    "cannot request the source prefix after ordinary driving began",
                ),
            );
        }
        if self.source_prefix_complete {
            return SourcePrefixReadiness::Complete;
        }
        let effects = crate::compile_time::SharedEffects::new(effects);
        let phase = self
            .phase
            .as_mut()
            .expect("active session retains its phase");
        phase.source_prefix_requested = true;
        let progress = phase.drive_bindings(&effects, None, None, &mut self.worklist);
        match progress {
            Ok(compile_time::BindingProgress::SourceRunReady) => SourcePrefixReadiness::Ready,
            Ok(compile_time::BindingProgress::SourceRunsReady) => {
                self.source_prefix_complete = true;
                SourcePrefixReadiness::Complete
            }
            Ok(compile_time::BindingProgress::Pending(pending)) => {
                SourcePrefixReadiness::Pending(pending)
            }
            Ok(_) => unreachable!("source prefix stops at its original run boundary"),
            Err(mut error) => {
                if let Some(worklist) = self.worklist.take()
                    && let Err(cause) = worklist.cancel(&effects)
                {
                    error
                        .message
                        .push_str(&format!("; cancellation failed: {cause}"));
                }
                self.phase = None;
                self.terminal = Some(SessionTerminal::Failed(error.clone()));
                SourcePrefixReadiness::Failed(error)
            }
        }
    }

    /// Advances the existing worklist; pending retains every semantic identity.
    pub fn drive(&mut self, effects: &mut dyn jai_vm::CompilerEffects) -> LibraryReadiness {
        if let Some(terminal) = &self.terminal {
            return LibraryReadiness::Failed(match terminal {
                SessionTerminal::Failed(error) => error.clone(),
                SessionTerminal::Complete => self.lifecycle_error("already completed"),
                SessionTerminal::Cancelled => self.lifecycle_error("was cancelled"),
            });
        }
        self.full_started = true;
        self.phase
            .as_mut()
            .expect("active session retains its phase")
            .source_prefix_requested = false;
        let effects = crate::compile_time::SharedEffects::new(effects);
        let progress = self
            .phase
            .as_mut()
            .expect("active session retains its phase")
            .drive_bindings(&effects, None, None, &mut self.worklist);
        match progress {
            Ok(compile_time::BindingProgress::Pending(pending)) => {
                LibraryReadiness::Pending(pending)
            }
            Ok(
                compile_time::BindingProgress::SourceRunReady
                | compile_time::BindingProgress::SourceRunsReady
                | compile_time::BindingProgress::TypesReady
                | compile_time::BindingProgress::HeadersReady
                | compile_time::BindingProgress::InitializersReady,
            ) => {
                unreachable!("phase drives header readiness internally")
            }
            Ok(compile_time::BindingProgress::Complete(procedures)) => {
                let phase = self.phase.take().expect("active session retains its phase");
                self.worklist = None;
                match phase.into_library(procedures) {
                    Ok(library) => {
                        self.terminal = Some(SessionTerminal::Complete);
                        LibraryReadiness::Complete(Box::new(library))
                    }
                    Err(error) => self.fail(error),
                }
            }
            Err(mut error) => {
                if let Some(worklist) = self.worklist.take()
                    && let Err(cause) = worklist.cancel(&effects)
                {
                    error
                        .message
                        .push_str(&format!("; cancellation failed: {cause}"));
                }
                self.phase = None;
                self.fail(error)
            }
        }
    }

    /// Cancels the exact retained VM tokens without beginning another run.
    pub fn cancel(
        &mut self,
        effects: &mut dyn jai_vm::CompilerEffects,
    ) -> Result<(), jai_vm::Error> {
        if self.terminal.is_some() {
            return Ok(());
        }
        let effects = crate::compile_time::SharedEffects::new(effects);
        let result = self
            .worklist
            .take()
            .map_or(Ok(()), |worklist| worklist.cancel(&effects));
        self.phase = None;
        self.terminal = Some(SessionTerminal::Cancelled);
        result
    }

    fn lifecycle_error(&self, state: &str) -> LocatedDiagnostic {
        LocatedDiagnostic {
            location: self.location,
            message: format!("semantic preparation session {state}"),
        }
    }

    fn fail(&mut self, error: LocatedDiagnostic) -> LibraryReadiness {
        self.terminal = Some(SessionTerminal::Failed(error.clone()));
        LibraryReadiness::Failed(error)
    }
}

pub(super) enum PreparedStart<'graph> {
    Phase(Box<PreparedPhase<'graph>>),
    Parameters(DiscoveryParameterOutcome),
}

/// The graph owner must retain its immutable source snapshot while this phase
/// exists. All mutable semantic arenas and identity allocators live here.
pub(super) struct PreparedPhase<'graph> {
    graph: &'graph ModuleGraph,
    options: crate::ResolveOptions,
    types: TypeRegistry,
    declarations: ScopedDeclarations<'graph>,
    globals: Vec<Global>,
    places: PlaceRegistry,
    meta: crate::reflection::MetaContext,
    alignment_jobs: Vec<storage_alignment::Job>,
    compiler: HashMap<ProcedureId, jai_vm::CompilerProcedure>,
    runtime: HashMap<ProcedureId, jai_vm::RuntimeProcedure>,
    deferred: std::collections::HashSet<DeclarationId>,
    prototypes: Vec<ProcedurePrototype>,
    program_exports: Vec<jai_ir::ProgramExport>,
    root_file: FileInstanceId,
    headers: Option<PendingHeaders<'graph>>,
    initial_types: Option<PendingTypes<'graph>>,
    initializers: Option<PendingInitializers<'graph>>,
    source_prefix_requested: bool,
}

struct PendingTypes<'graph> {
    aliases: source_preparation::PendingAliases<'graph>,
    enums: bool,
    pending: Option<source_preparation::SourcePreparationPending>,
}

struct PendingInitializers<'graph> {
    constants: Constants<'graph>,
    jobs: global_initializers::Jobs<'graph>,
}

struct PendingHeaders<'graph> {
    constants: Constants<'graph>,
    fields: std::collections::HashSet<jai_types::FieldId>,
}

pub(super) fn prepare<'graph>(
    graph: &'graph ModuleGraph,
    options: &crate::ResolveOptions,
    discovery: Option<PreparedDiscovery<'_>>,
) -> Result<PreparedStart<'graph>, LocatedDiagnostic> {
    if discovery.is_none()
        && let Some(request) = graph.pending_insertions().next()
    {
        return Err(LocatedDiagnostic {
            location: request.location,
            message: "unpublished top-level #insert requires retained graph discovery".into(),
        });
    }
    let mut meta = crate::reflection::MetaContext::default();
    meta.external_globals
        .reserve_file_prefix(
            graph
                .declarations()
                .iter()
                .filter(|declaration| {
                    matches!(declaration.syntax().kind, FileDeclarationKind::Global(_))
                })
                .count(),
        )
        .map_err(|error| located(graph, graph.module(graph.root()).unwrap().entry(), error))?;
    let mut types = TypeRegistry::new();
    let mut constants = Constants::new(graph);
    let callable_aliases = crate::polymorphism::integration::callable_aliases(graph);
    let source_procedures = procedure_headers::identities::reserve(graph, &callable_aliases)?;
    let deferred = deferred_constants::classify(graph);
    let mut nominals = Nominals::reserve(graph, &mut types)?;
    nominals.set_annotation_target(options.effective_layout());
    constants.register_startup_annotations(&mut types)?;
    nominals.materialize_module_parameters(
        graph,
        &mut types,
        &mut meta.record_specializations,
        &mut |file, expression| constants.evaluate_lazy(file, expression),
    )?;
    constants.register_module_parameter_annotations(&nominals, &types)?;
    aggregates::parameterized::reserve_static_members(
        graph,
        &nominals,
        &mut types,
        &mut meta.record_specializations,
        &mut |file, expression| constants.evaluate_lazy(file, expression),
    )?;
    let declarations = ScopedDeclarations {
        context: None,
        graph,
        values: HashMap::new(),
        signatures: HashMap::new(),
        callable_aliases,
        generics: std::cell::RefCell::new(crate::polymorphism::integration::GenericContext::new(
            source_procedures.len(),
        )),
        source_procedures,
        source_lookup: Default::default(),
        nominals,
        defaults: HashMap::new(),
    };
    let root_file = graph.module(graph.root()).unwrap().entry();
    let mut phase = PreparedPhase {
        graph,
        options: options.clone(),
        types,
        declarations,
        globals: Vec::new(),
        places: PlaceRegistry::new(),
        meta,
        alignment_jobs: Vec::new(),
        compiler: HashMap::new(),
        runtime: HashMap::new(),
        deferred,
        prototypes: Vec::new(),
        program_exports: Vec::new(),
        root_file,
        initial_types: Some(PendingTypes {
            aliases: source_preparation::PendingAliases::new(graph, constants),
            enums: false,
            pending: None,
        }),
        initializers: None,
        headers: None,
        source_prefix_requested: false,
    };
    if let Some(pending) = phase.advance_initial()? {
        if matches!(discovery, Some(PreparedDiscovery::Parameters(_))) {
            return Err(pending.diagnostic(graph));
        }
        return Ok(PreparedStart::Phase(Box::new(phase)));
    }
    if let Some(outcome) = phase.finish_initial(discovery)? {
        return Ok(PreparedStart::Parameters(outcome));
    }
    Ok(PreparedStart::Phase(Box::new(phase)))
}

impl<'graph> PreparedPhase<'graph> {
    fn advance_initial(
        &mut self,
    ) -> Result<Option<source_preparation::SourcePreparationPending>, LocatedDiagnostic> {
        let Some(mut initial) = self.initial_types.take() else {
            return Ok(None);
        };
        for (&id, binding) in &self.declarations.values {
            if let Binding::TypedConstant(constant) = binding
                && let Some(value) = self.meta.constant(*constant)
            {
                if initial.aliases.constants_mut().annotation(id).is_some() {
                    initial
                        .aliases
                        .constants_mut()
                        .register_checked_typed_value(id, value.ty)?;
                }
                self.declarations.nominals.value_types.insert(id, value.ty);
                self.declarations
                    .nominals
                    .value_constants
                    .insert(id, value.clone());
            }
            if let Binding::Enum(value) = binding {
                if initial.aliases.constants_mut().annotation(id).is_some() {
                    initial
                        .aliases
                        .constants_mut()
                        .register_checked_typed_value(id, value.ty)?;
                }
                self.declarations.nominals.value_types.insert(id, value.ty);
                self.declarations.nominals.value_constants.insert(
                    id,
                    jai_ir::ConstantValue {
                        ty: value.ty,
                        kind: jai_ir::ConstantKind::Enum(value.value),
                    },
                );
            }
            let value = match binding {
                Binding::Constant(value) => Some(value.clone()),
                Binding::TypedConstant(constant) => {
                    self.meta
                        .constant(*constant)
                        .and_then(|value| match value.kind {
                            jai_ir::ConstantKind::Int(integer) => Some(ConstantValue::Int(integer)),
                            jai_ir::ConstantKind::Bool(boolean) => {
                                Some(ConstantValue::Bool(boolean))
                            }
                            jai_ir::ConstantKind::Float(float) => Some(ConstantValue::Float(float)),
                            _ => None,
                        })
                }
                _ => None,
            };
            if let Some(value) = value {
                initial
                    .aliases
                    .constants_mut()
                    .register_ready_value(id, value)?;
            }
        }
        self.deferred.extend(record_method_headers::aliases(
            self.graph,
            &self.declarations.nominals,
            &self.types,
        ));
        let progress = (|| {
            let _headers = initial.aliases.prepare_independent_headers(
                &mut self.declarations,
                &mut self.types,
                &mut self.meta,
            )?;
            if let source_preparation::AliasProgress::Pending(pending) =
                initial
                    .aliases
                    .drive(&mut self.declarations, &mut self.types, &mut self.meta)?
            {
                return Ok(Some(pending));
            }
            self.deferred.extend(record_method_headers::aliases(
                self.graph,
                &self.declarations.nominals,
                &self.types,
            ));
            if let source_preparation::AliasProgress::Pending(pending) = initial
                .aliases
                .prepare_annotations(&mut self.declarations, &mut self.types, &mut self.meta)?
            {
                return Ok(Some(pending));
            }
            if !initial.enums {
                self.declarations.nominals.define_enums(
                    self.graph,
                    &mut self.types,
                    &mut |file, path, span| {
                        initial
                            .aliases
                            .constants_mut()
                            .lookup_value(file, path, span)
                    },
                )?;
                initial
                    .aliases
                    .constants_mut()
                    .register_enums(&self.declarations.nominals);
                initial.enums = true;
            }
            let registration =
                context_registration::collect(self.graph, initial.aliases.constants_mut())?;
            let nominal = enum_constants::classify(self.graph, &self.declarations.nominals);
            if let source_preparation::AliasProgress::Pending(pending) =
                initial.aliases.prepare_scalars(
                    &mut self.declarations,
                    &self.deferred,
                    &registration.consumed,
                    &nominal,
                )?
            {
                return Ok(Some(pending));
            }
            if let Some(pending) = initial.aliases.prepare_independent_headers(
                &mut self.declarations,
                &mut self.types,
                &mut self.meta,
            )? {
                return Ok(Some(pending));
            }
            if let source_preparation::AliasProgress::Pending(pending) =
                initial.aliases.prepare_global_annotations(
                    &mut self.declarations,
                    &mut self.types,
                    &mut self.meta,
                )?
            {
                return Ok(Some(pending));
            }
            Ok(None)
        })();
        if let Ok(pending) = &progress {
            initial.pending = *pending;
        }
        self.initial_types = Some(initial);
        progress
    }

    /// Publish an independently complete source Context before unrelated type waits.
    /// No Context additions are omitted: this path applies only when the original
    /// graph and designated bootstrap have no context declarations at all.
    fn prepare_source_prefix_context(&mut self) -> Result<bool, LocatedDiagnostic> {
        if self.declarations.context.is_some() {
            return Ok(true);
        }
        if !self.graph.context_fields().is_empty()
            || (self.graph.prelude().is_some() && self.graph.runtime_support().is_some())
        {
            return Ok(false);
        }
        let initial = self.initial_types.as_mut().expect("retained type prefix");
        let registration =
            context_registration::collect(self.graph, initial.aliases.constants_mut())?;
        debug_assert!(registration.fields.is_empty());
        self.declarations.context = Some(context::build(
            self.graph,
            &mut self.types,
            &self.declarations,
            initial.aliases.constants_mut(),
            &registration.fields,
            &mut self.meta,
        )?);
        Ok(true)
    }

    fn finish_initial(
        &mut self,
        discovery: Option<PreparedDiscovery<'_>>,
    ) -> Result<Option<DiscoveryParameterOutcome>, LocatedDiagnostic> {
        let initial = self
            .initial_types
            .take()
            .expect("completed type prefix retains original constants");
        let (mut constants, mut context_headers) = match initial.aliases.into_constants() {
            Ok(prepared) => prepared,
            Err(_) => unreachable!("finish only after original aliases complete"),
        };
        let graph = self.graph;
        let options = &self.options;
        let mut types = &mut self.types;
        let mut declarations = &mut self.declarations;
        let mut meta = &mut self.meta;
        let deferred = &mut self.deferred;
        let mut places = &mut self.places;
        let record_callable_aliases =
            record_method_headers::aliases(graph, &declarations.nominals, &types);
        deferred.extend(record_callable_aliases.iter().copied());
        let context_registration = context_registration::collect(graph, &mut constants)?;
        let mut nominal_constants = enum_constants::classify(graph, &declarations.nominals);
        for declaration in graph.declarations() {
            if let Some(ty) = constants.annotation(declaration.id())
                && matches!(
                    types.kind(ty),
                    Ok(jai_types::TypeKind::Enum(_) | jai_types::TypeKind::Distinct(_))
                )
            {
                nominal_constants.insert(declaration.id());
            }
        }
        let mut values = HashMap::new();
        for declaration in graph.declarations() {
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                continue;
            };
            if declarations.nominals.is_type_alias(graph, declaration.id()) {
                continue;
            }
            if context_registration.consumed.contains(&declaration.id()) {
                continue;
            }
            if declarations
                .callable_aliases
                .contains_key(&declaration.id())
            {
                continue;
            }
            if deferred.contains(&declaration.id())
                || nominal_constants.contains(&declaration.id())
                || constants.has_typed_annotation(declaration.id())
                || sequence_constants::is_sequence_constant(graph, declaration)
            {
                continue;
            }
            let name = match &constant.initializer.kind {
                syntax::ExpressionKind::Name(name) => Some(path(*name)),
                syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                _ => None,
            };
            if let Some(path) = name {
                if let Ok(jai_modules::Binding::Parameter(id)) =
                    graph.lookup(declaration.file(), &path)
                    && let jai_modules::ParameterValue::Enumeration(value) =
                        &graph.parameter(id).unwrap().value
                {
                    if constant.ty.is_some() {
                        return Err(located(
                            graph,
                            declaration.file(),
                            Diagnostic::new(
                                constant.span,
                                "module enum parameter cannot implicitly convert to a scalar constant",
                            ),
                        ));
                    }
                    let ty = declarations
                        .nominals
                        .declarations
                        .get(&value.declaration)
                        .copied()
                        .ok_or_else(|| {
                            located(
                                graph,
                                declaration.file(),
                                Diagnostic::new(
                                    constant.span,
                                    "module enum parameter has no resolved nominal declaration",
                                ),
                            )
                        })?;
                    values.insert(
                        declaration.id(),
                        Binding::Enum(aggregates::EnumConstant {
                            ty,
                            value: value.value,
                        }),
                    );
                    continue;
                }
                if let Some(member) = declarations
                    .nominals
                    .enum_member(graph, declaration.file(), &path, constant.span)
                    .map_err(|error| located(graph, declaration.file(), error))?
                {
                    if constant.ty.is_some() {
                        return Err(located(
                            graph,
                            declaration.file(),
                            Diagnostic::new(
                                constant.span,
                                "enum constant cannot implicitly convert to scalar type",
                            ),
                        ));
                    }
                    values.insert(declaration.id(), Binding::Enum(member));
                    constants.value(declaration.id(), declaration.location())?;
                    continue;
                }
            }
            let value = constants.value(declaration.id(), declaration.location())?;
            values.insert(declaration.id(), Binding::Constant(value));
        }
        declarations.values.extend(values);
        enum_constants::bind(
            &mut declarations,
            &mut types,
            &mut meta,
            &nominal_constants,
            &deferred,
            options,
        )?;
        for (id, binding) in &declarations.values {
            if let Binding::Constant(value) = binding {
                let ty = value.type_id(types);
                constants.register_nominal_type(*id, ty);
                declarations.nominals.value_types.insert(*id, ty);
            }
            if let Binding::Enum(value) = binding {
                constants.register_nominal_type(*id, value.ty);
                declarations.nominals.value_types.insert(*id, value.ty);
            }
        }
        // Pending module headers have not published their type variables yet.
        // Resolve their retained requests before requiring procedure annotations.
        if !matches!(discovery, Some(PreparedDiscovery::Parameters(_))) {
            context_headers.extend(procedure_headers::register_before_context(
                graph,
                types,
                declarations,
                &mut constants,
                meta,
            )?);
            record_method_headers::bind(
                &declarations,
                &mut types,
                &mut places,
                &mut meta,
                options,
            )?;
            record_method_headers::publish_callable_aliases(
                &mut declarations,
                &mut types,
                &mut meta,
                &record_callable_aliases,
            )?;
        }
        declarations.nominals.define_records_with_specializations(
            graph,
            &mut types,
            &mut meta.record_specializations,
            &mut |file, expression| constants.evaluate_lazy(file, expression),
        )?;
        allocator_schema::bind(graph, &declarations.nominals, &mut types)?;
        meta.install_preload_schema(
            graph,
            &declarations.nominals,
            &mut types,
            options.effective_layout(),
        )?;
        if let Some(PreparedDiscovery::Parameters(requests)) = discovery {
            let outcome = parameter_discovery::resolve(
                graph,
                requests,
                &declarations,
                &mut types,
                &mut meta,
                &mut constants,
            );
            return Ok(Some(outcome));
        }
        if declarations.context.is_none() {
            declarations.context = Some(context::build(
                graph,
                &mut types,
                &declarations,
                &mut constants,
                &context_registration.fields,
                &mut meta,
            )?);
        } else {
            // The early Context producer admits only the genuinely empty original
            // schema. Keep its canonical TypeId and checked definition intact.
            debug_assert!(context_registration.fields.is_empty());
            debug_assert!(graph.context_fields().is_empty());
        }
        if !matches!(discovery, Some(PreparedDiscovery::Parameters(_))) {
            procedure_headers::complete_context_headers(
                graph,
                &context_headers,
                types,
                declarations,
                &mut constants,
                meta,
            )?;
        }
        // Header prerequisites use the same checked constant worklist as full
        // binding. Keep unresolved typed values available to its dependency graph.
        deferred.extend(graph.declarations().iter().filter_map(|declaration| {
            let id = declaration.id();
            if !matches!(declaration.syntax().kind, FileDeclarationKind::Constant(_))
                || declarations.values.contains_key(&id)
                || declarations.callable_aliases.contains_key(&id)
                || context_registration.consumed.contains(&id)
                || declarations.nominals.is_type_alias(graph, id)
            {
                return None;
            }
            (sequence_constants::is_sequence_constant(graph, declaration)
                || constants.has_typed_annotation(id))
            .then_some(id)
        }));
        meta.field_default_jobs
            .collect(field_default_jobs::FieldDefaultSources {
                graph,
                nominals: &declarations.nominals,
                records: &meta.record_specializations,
                types: &types,
                deferred_constants: &deferred,
            })?;
        let mut evaluator =
            aggregates::Defaults::new(graph, &types, &declarations.nominals, &constants)
                .with_specializations(&meta.record_specializations)
                .with_context(declarations.context.as_ref());
        enum_constants::seed_defaults(&mut evaluator, &declarations.values, &meta);
        hydrate_constants(&mut evaluator, &declarations, &meta);
        let prepared_defaults = evaluator.prepare(&meta.field_default_jobs)?;
        declarations.defaults = prepared_defaults.ready;

        self.headers = Some(PendingHeaders {
            constants,
            fields: prepared_defaults.pending,
        });
        if self.headers.as_ref().unwrap().fields.is_empty() {
            self.complete_headers()?;
        } else {
            self.install_intrinsics()?;
        }
        debug_sources::retain_graph(
            &mut self.meta.debug_sources,
            graph,
            &self.declarations.signatures,
        )?;
        Ok(None)
    }

    fn install_ready_intrinsics(&mut self) -> Result<(), LocatedDiagnostic> {
        self.runtime = runtime_intrinsics::bind(
            self.graph,
            &self.types,
            &self.declarations,
            self.options.effective_layout(),
        )?;
        self.compiler = compiler_intrinsics::bind_ready(
            self.graph,
            &self.types,
            &self.declarations,
            self.options.compiler.as_ref(),
            &self.meta,
        )?;
        Ok(())
    }

    fn install_intrinsics(&mut self) -> Result<(), LocatedDiagnostic> {
        self.runtime = runtime_intrinsics::bind(
            self.graph,
            &self.types,
            &self.declarations,
            self.options.effective_layout(),
        )?;
        self.compiler = compiler_intrinsics::bind(
            self.graph,
            &self.types,
            &self.declarations,
            self.options.compiler.as_ref(),
            &self.meta,
        )?;
        Ok(())
    }

    fn complete_headers(&mut self) -> Result<(), LocatedDiagnostic> {
        let Some(mut headers) = self.headers.take() else {
            return Ok(());
        };
        let graph = self.graph;
        let options = &self.options;
        let types = &mut self.types;
        let declarations = &mut self.declarations;
        let meta = &mut self.meta;
        let constants = &mut headers.constants;
        sequence_constants::bind(declarations, types, constants, meta)?;
        let jobs = global_initializers::Jobs::prepare(
            graph,
            declarations,
            types,
            constants,
            meta,
            options,
        )?;
        self.initializers = Some(PendingInitializers {
            constants: headers.constants,
            jobs,
        });
        self.install_intrinsics()?;
        Ok(())
    }

    fn complete_initializers(&mut self) -> Result<(), LocatedDiagnostic> {
        let Some(mut initializers) = self.initializers.take() else {
            return Ok(());
        };
        let graph = self.graph;
        let options = &self.options;
        let types = &mut self.types;
        let declarations = &mut self.declarations;
        let meta = &mut self.meta;
        let constants = &mut initializers.constants;
        let globals = &mut self.globals;
        meta.external_globals
            .complete_file_prefix(globals)
            .map_err(|error| located(graph, graph.module(graph.root()).unwrap().entry(), error))?;
        procedure_headers::register(
            graph,
            types,
            declarations,
            constants,
            meta,
            procedure_headers::HeaderPhase::Complete,
        )?;
        program_exports::bind_entry_aliases(graph, declarations)?;
        let runtime =
            runtime_intrinsics::bind(graph, types, declarations, options.effective_layout())?;
        let program_exports = program_exports::bind(graph, declarations, types)?;
        let prototypes: Vec<ProcedurePrototype> = graph
            .declarations()
            .iter()
            .filter_map(|declaration| {
                let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind
                else {
                    return None;
                };
                if matches!(prototype.binding, syntax::PrototypeBinding::EntryPoint) {
                    return None;
                }
                let signature = declarations.signatures.get(&declaration.id())?;
                let origin = if let Some(runtime) = runtime.get(&signature.id) {
                    Ok(PrototypeOrigin::Intrinsic(runtime.intrinsic))
                } else {
                    foreign_libraries::origin(graph, declaration.file(), prototype)
                };
                Some(origin.map(|origin| ProcedurePrototype {
                    id: signature.id,
                    signature: signature.ty,
                    origin,
                }))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.runtime = runtime;
        self.program_exports = program_exports;
        self.prototypes = prototypes;
        self.compiler =
            compiler_intrinsics::bind(graph, types, declarations, options.compiler.as_ref(), meta)?;
        Ok(())
    }

    pub(super) fn drive_bindings<'requests>(
        &mut self,
        effects: &dyn crate::compile_time::EffectService,
        mut discovery: Option<&mut discovery_conditions::Jobs<'requests>>,
        admission: Option<&InsertionAdmissionCallback<'graph>>,
        worklist: &mut Option<compile_time::Worklist<'graph>>,
    ) -> Result<compile_time::BindingProgress, LocatedDiagnostic> {
        loop {
            if self.initial_types.is_some() {
                match self.advance_initial()? {
                    None => {
                        self.finish_initial(None)?;
                    }
                    Some(pending) => {
                        let prefix_ready =
                            self.source_prefix_requested && self.prepare_source_prefix_context()?;
                        if matches!(
                            pending.cause(),
                            Some(aggregates::parameterized::PendingType::Placeholder(_))
                        ) && !prefix_ready
                            && discovery.as_ref().is_none_or(|jobs| !jobs.has_insertions())
                        {
                            return Ok(compile_time::BindingProgress::Pending(LibraryPending {
                                dependencies: vec![],
                                diagnostic: pending.diagnostic(self.graph),
                                source: Some(pending),
                            }));
                        }
                        self.install_ready_intrinsics()?;
                        let progress = compile_time::bind_procedures_resumable(
                            self.bind_session(effects, discovery.as_deref_mut(), admission),
                            worklist,
                        )?;
                        match progress {
                            compile_time::BindingProgress::TypesReady => continue,
                            compile_time::BindingProgress::Pending(mut wait) => {
                                wait.source.get_or_insert(pending);
                                return Ok(compile_time::BindingProgress::Pending(wait));
                            }
                            progress => return Ok(progress),
                        }
                    }
                }
            }
            let progress = compile_time::bind_procedures_resumable(
                self.bind_session(effects, discovery.as_deref_mut(), admission),
                worklist,
            )?;
            match progress {
                compile_time::BindingProgress::HeadersReady => self.complete_headers()?,
                compile_time::BindingProgress::InitializersReady => self.complete_initializers()?,
                progress => return Ok(progress),
            }
        }
    }

    pub(super) fn bind_session<'a, 'requests>(
        &'a mut self,
        effects: &'a dyn crate::compile_time::EffectService,
        discovery: Option<&'a mut discovery_conditions::Jobs<'requests>>,
        admission: Option<&'a InsertionAdmissionCallback<'graph>>,
    ) -> compile_time::BindSession<'a, 'graph, 'requests> {
        compile_time::BindSession {
            mode: match &self.headers {
                _ if self.initial_types.is_some() => compile_time::BindingMode::Types(
                    self.initial_types
                        .as_ref()
                        .unwrap()
                        .pending
                        .expect("type worklist retains demand"),
                ),
                Some(headers) => compile_time::BindingMode::Headers(&headers.fields),
                None if self.initializers.is_some() => compile_time::BindingMode::Initializers,
                None if self.source_prefix_requested => compile_time::BindingMode::SourceRuns,
                None => compile_time::BindingMode::Full,
            },
            source_prefix: self.source_prefix_requested,
            graph: self.graph,
            types: &mut self.types,
            declarations: &mut self.declarations,
            globals: &mut self.globals,
            initializers: self
                .initializers
                .as_mut()
                .map(|initializers| &mut initializers.jobs),
            places: &mut self.places,
            options: &self.options,
            effects,
            compiler: &self.compiler,
            runtime: &self.runtime,
            deferred: &self.deferred,
            meta: &mut self.meta,
            alignment_jobs: &mut self.alignment_jobs,
            discovery,
            admission,
        }
    }
    pub(super) fn take_specializations(&mut self) -> Vec<jai_modules::SourceSpecializationKey> {
        std::mem::take(&mut self.meta.pending_source_specializations)
    }
    pub(super) fn into_library(
        self,
        procedures: Vec<Procedure>,
    ) -> Result<Library, LocatedDiagnostic> {
        let Self {
            graph,
            types,
            declarations,
            globals,
            places,
            mut meta,
            mut prototypes,
            program_exports,
            root_file,
            ..
        } = self;
        let globals = meta
            .external_globals
            .snapshot(&globals)
            .map_err(|error| located(graph, root_file, error))?;
        prototypes.extend(meta.local_declarations.prototypes());
        prototypes.extend(declarations.generics.borrow().prototype_snapshot());
        // Body capture uses the header's name span. Restore each actual file
        // declaration's complete parser-owned extent after all bodies have bound.
        debug_sources::retain_graph(&mut meta.debug_sources, graph, &declarations.signatures)?;
        debug_sources::retain_types(
            &mut meta.debug_sources,
            graph,
            &declarations.nominals,
            &meta.record_specializations,
            &meta.local_declarations,
        )?;
        let types = types.freeze().map_err(|error| {
            located(
                graph,
                root_file,
                Diagnostic::new(Span::default(), error.to_string()),
            )
        })?;
        let procedure_ids = declarations
            .signatures
            .iter()
            .map(|(declaration, signature)| (*declaration, signature.id))
            .collect();
        ProgramBuilder::new(types)
            .source_procedure_owners(meta.source_procedure_owners)
            .source_warnings(
                jai_ir::SourceWarnings::checked(meta.deprecations.take(), graph.sources())
                    .map_err(|error| {
                        located(
                            graph,
                            root_file,
                            Diagnostic::new(Span::default(), error.to_string()),
                        )
                    })?,
            )
            .program_exports(program_exports)
            .storage_alignments(meta.storage_alignments)
            .procedure_hints(meta.procedure_hints)
            .procedure_phases(meta.procedure_phases)
            .debug_sources(std::mem::take(&mut meta.debug_sources))
            .context(
                declarations
                    .context
                    .as_ref()
                    .expect("context schema was constructed")
                    .definition
                    .clone(),
            )
            .procedures(procedures)
            .prototypes(prototypes)
            .foreign_libraries({
                let mut libraries = foreign_libraries::all(graph)?;
                libraries.extend(meta.local_declarations.foreign_libraries());
                libraries
            })
            .foreign_library_sources(foreign_libraries::source_provenance(graph)?)
            .globals(globals)
            .places(places.freeze())
            .declarations(procedure_ids)
            .finish_library()
            .map_err(|error| {
                located(
                    graph,
                    root_file,
                    Diagnostic::new(Span::default(), error.to_string()),
                )
            })
    }
}
