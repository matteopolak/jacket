//! Bind available bodies before retrying source compile-time dependencies.
use super::*;
use crate::compile_time::{Cache, Context, EffectService};
use std::cell::RefCell;
mod alignments;
mod file_conditions;
mod header_prerequisites;
mod methods;
mod worklist;
use worklist::BodyJob;
pub(super) use worklist::Worklist;

pub(super) enum BindingProgress {
    SourceRunReady,
    SourceRunsReady,
    TypesReady,
    HeadersReady,
    InitializersReady,
    Complete(Vec<Procedure>),
    Pending(super::LibraryPending),
}

#[derive(Clone, Copy, Debug)]
pub(super) enum BindingMode<'a> {
    Headers(&'a std::collections::HashSet<jai_types::FieldId>),
    Initializers,
    SourceRuns,
    Types(super::SourcePreparationPending),
    Full,
}

pub(super) struct BindSession<'a, 'graph, 'requests> {
    pub mode: BindingMode<'a>,
    pub source_prefix: bool,
    pub graph: &'graph ModuleGraph,
    pub types: &'a mut TypeRegistry,
    pub declarations: &'a mut ScopedDeclarations<'graph>,
    pub globals: &'a mut Vec<Global>,
    pub initializers: Option<&'a mut global_initializers::Jobs<'graph>>,
    pub places: &'a mut PlaceRegistry,
    pub options: &'a crate::ResolveOptions,
    pub effects: &'a dyn EffectService,
    pub compiler: &'a HashMap<ProcedureId, jai_vm::CompilerProcedure>,
    pub runtime: &'a HashMap<ProcedureId, jai_vm::RuntimeProcedure>,
    pub deferred: &'a std::collections::HashSet<DeclarationId>,
    pub meta: &'a mut crate::reflection::MetaContext,
    pub alignment_jobs: &'a mut Vec<super::storage_alignment::Job>,
    pub discovery: Option<&'a mut super::discovery_conditions::Jobs<'requests>>,
    pub admission: Option<&'a InsertionAdmissionCallback<'graph>>,
}

pub(super) fn bind_procedures_resumable<'graph>(
    mut session: BindSession<'_, 'graph, '_>,
    retained: &mut Option<Worklist<'graph>>,
) -> Result<BindingProgress, LocatedDiagnostic> {
    let effects = session.effects;
    let root_source = session
        .graph
        .file(session.graph.module(session.graph.root()).unwrap().entry())
        .unwrap()
        .source();
    if let Some(worklist) = retained.as_mut() {
        worklist.refresh(&mut session)?;
    }
    let Worklist {
        file_abi,
        heap_abi,
        process_abi,
        code_constants,
        record_callable_aliases,
        mut signatures,
        foreign,
        cache,
        mut ready,
        mut pending,
        mut constants,
        mut constant_owners,
        mut runs,
        mut completed_runs,
        mut isolated_caches,
        mut completed_alignments,
        mut file_guards,
        mut completed_file_guards,
        alignment_owners,
        method_file,
        method_owner,
        mut methods_pending,
        mut procedure_defaults,
        mut header_prerequisites,
        insertion_owners,
    } = match retained.take() {
        Some(worklist) => worklist,
        None => Worklist::new(&mut session)?,
    };
    let outcome = (|| {
        let empty_signatures = HashMap::new();
        let empty_values = HashMap::new();
        let BindSession {
            mode,
            source_prefix,
            graph,
            types,
            declarations,
            globals,
            mut initializers,
            places,
            options,
            effects,
            compiler,
            runtime,
            deferred,
            meta,
            alignment_jobs,
            mut discovery,
            admission,
        } = session;
        if let BindingMode::Types(pending) = mode {
            if let Some(aggregates::parameterized::PendingType::Constant {
                declaration, ..
            }) = pending.cause()
            {
                header_prerequisites.constants.insert(declaration);
            }
        }
        if let BindingMode::Headers(fields) = mode {
            header_prerequisites.include_fields(fields);
        }
        let mut execution_deferred = deferred.clone();
        while !pending.is_empty()
            || !constants.is_empty()
            || !runs.is_empty()
            || declarations.generics.borrow().has_pending_bodies()
            || declarations.generics.borrow().has_pending_modifier_bodies()
            || methods_pending
            || !procedure_defaults.is_empty()
            || !alignment_jobs.is_empty()
            || !file_guards.is_empty()
            || matches!(
                mode,
                BindingMode::Types(_)
                    | BindingMode::Headers(_)
                    | BindingMode::Initializers
                    | BindingMode::SourceRuns
            )
        {
            worklist::admit_constant_prerequisites(
                graph,
                declarations,
                &header_prerequisites.constants,
                &mut constants,
                &mut constant_owners,
            )?;
            execution_deferred.extend(header_prerequisites.constants.iter().copied());
            let deferred = &execution_deferred;
            while let Some(body) = declarations.generics.borrow_mut().next_body() {
                if ready.remove(&body.signature.id).is_some() {
                    meta.debug_sources.clear_procedure(body.signature.id);
                    meta.storage_alignments.clear_procedure(body.signature.id);
                }
                pending.push(BodyJob {
                    declaration: body.declaration,
                    file: body.file,
                    signature: body.signature,
                    substitution: Some(body.substitution),
                    specialization: Some(body.specialization),
                    syntax: None,
                    modifier: None,
                });
            }
            while let Some(body) = declarations.generics.borrow_mut().next_modifier_body() {
                pending.push(BodyJob {
                    declaration: body.declaration,
                    file: body.file,
                    signature: body.signature,
                    substitution: None,
                    specialization: None,
                    syntax: Some(body.procedure),
                    modifier: Some(body.key),
                });
            }
            signatures.extend(declarations.generics.borrow().signature_snapshot());
            signatures.extend(
                declarations
                    .signatures
                    .values()
                    .map(|signature| (signature.id, signature.ty)),
            );
            signatures.extend(meta.local_declarations.signature_snapshot());
            let mut retry = Vec::new();
            let mut stalled = None;
            let mut body_error = None;
            let mut alignment_error = None;
            let mut file_guard_error = None;
            let pending_global_alignments: std::collections::HashSet<_> =
                alignment_jobs.iter().map(|job| job.global).collect();
            let before_modifier_queue = meta.record_specializations.queued_modifier_count();
            let before = ready.len()
                + meta.local_declarations.semantic_ready_count()
                + declarations.values.len()
                + declarations.generics.borrow().specialization_count()
                + completed_runs;
            let before = before + declarations.generics.borrow().modifier_job_count();
            let before = before + completed_alignments + completed_file_guards;
            let before = before
                + meta.field_default_jobs.ready().count()
                + meta.field_default_jobs.requested_count();
            let before = before
                + header_prerequisites.len()
                + meta.record_specializations.completed_modifier_count()
                + initializers.as_ref().map_or(0, |jobs| jobs.completed())
                + procedure_defaults.progress_count();
            let before_revision = (
                declarations.generics.borrow().callback_readiness_revision(),
                meta.local_declarations.callback_readiness_revision(),
            );
            let record_count = meta.record_specializations.records().count();
            let snapshot = places.snapshot();
            let method_context = Context {
                workspace: options
                    .compiler
                    .as_ref()
                    .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                        context.current_workspace
                    }),
                foreign: &foreign,
                owner: method_owner,
                generics: &declarations.generics,
                context: declarations
                    .context
                    .as_ref()
                    .map(|schema| &schema.definition),
                procedures: &ready,
                signatures: &signatures,
                globals,
                pending_global_alignments: &pending_global_alignments,
                places: &snapshot,
                source: graph.file(method_file).unwrap().source(),
                file: method_file,
                target: options
                    .target
                    .as_ref()
                    .map(jai_vm::ByteTarget::from)
                    .or_else(|| {
                        options.effective_layout().map(|policy| jai_vm::ByteTarget {
                            policy,
                            endian: jai_vm::Endian::Little,
                        })
                    }),
                limits: options.compile_time_limits,
                pending: RefCell::new(vec![]),
                pending_constants: RefCell::new(vec![]),
                pending_field_defaults: RefCell::new(vec![]),
                cache: &cache,
                effects,
                effect_mode: crate::compile_time::EffectsMode::Compiler,
                compiler,
                runtime,
                file_abi: &file_abi,
                heap_abi: &heap_abi,
                process_abi: &process_abi,
                deferred,
            };
            let default_checkpoint = procedure_defaults.bind(
                &method_context,
                declarations,
                types,
                places,
                meta,
                super::procedure_default_jobs::BindOptions {
                    resolve: options,
                    prefix: source_prefix
                        && matches!(mode, BindingMode::Types(_) | BindingMode::SourceRuns),
                },
            )?;
            if default_checkpoint {
                return Ok(BindingProgress::SourceRunReady);
            }
            let mut source_wait = procedure_defaults.source_wait();
            let mut initializer_lookup_wait = None;
            let initializer_lookup_allowed =
                source_prefix && matches!(mode, BindingMode::Initializers) && !runs.is_empty();
            let method_requests = super::procedure_default_jobs::request_snapshot(declarations);
            let method_result = match mode {
                BindingMode::Types(_) => {
                    methods::sweep_types(&method_context, declarations, types, places, meta)
                }
                BindingMode::Full => {
                    methods::sweep(&method_context, declarations, types, places, meta)
                }
                BindingMode::Headers(_) | BindingMode::Initializers | BindingMode::SourceRuns => {
                    header_prerequisites.observe_fields(&meta.field_default_jobs);
                    methods::sweep_headers(
                        &method_context,
                        declarations,
                        types,
                        places,
                        meta,
                        &header_prerequisites.procedures,
                    )
                }
            };
            let method_dependencies = method_context.pending.into_inner();
            let method_constants = method_context.pending_constants.into_inner();
            let method_fields = method_context.pending_field_defaults.into_inner();
            if matches!(
                mode,
                BindingMode::Types(_)
                    | BindingMode::Headers(_)
                    | BindingMode::Initializers
                    | BindingMode::SourceRuns
            ) {
                header_prerequisites.observe(
                    &method_dependencies,
                    &method_constants,
                    &method_fields,
                );
                header_prerequisites.observe_fields(&meta.field_default_jobs);
            }
            for &field in &method_fields {
                meta.field_default_jobs.request(field);
            }
            declarations.defaults.extend(
                meta.field_default_jobs
                    .ready()
                    .map(|(field, value)| (field, value.clone())),
            );
            super::record_method_headers::publish_callable_aliases(
                declarations,
                types,
                meta,
                &record_callable_aliases,
            )?;
            if let Some(wait) =
                super::procedure_default_jobs::requested_since(declarations, &method_requests)
            {
                source_wait = Some(wait);
            }
            methods_pending = method_result.is_err();
            let method_error = method_result.err();
            if !method_dependencies.is_empty()
                || !method_constants.is_empty()
                || !method_fields.is_empty()
            {
                stalled = Some((
                    method_file,
                    Span::default(),
                    format!(
                        "{method_dependencies:?}; constants {method_constants:?}; field defaults {method_fields:?}"
                    ),
                ));
            }
            signatures.extend(meta.local_declarations.signature_snapshot());
            let mut retry_constants = vec![];
            for declaration in std::mem::take(&mut constants) {
                if declarations.values.contains_key(&declaration.id()) {
                    continue;
                }
                if matches!(
                    mode,
                    BindingMode::Types(_)
                        | BindingMode::Headers(_)
                        | BindingMode::Initializers
                        | BindingMode::SourceRuns
                ) && !header_prerequisites.constants.contains(&declaration.id())
                {
                    retry_constants.push(declaration);
                    continue;
                }
                let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                    unreachable!()
                };
                let file = declaration.file();
                let snapshot = places.snapshot();
                let context = Context {
                    workspace: options
                        .compiler
                        .as_ref()
                        .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                            context.current_workspace
                        }),
                    foreign: &foreign,
                    owner: constant_owners[&declaration.id()],
                    generics: &declarations.generics,
                    context: declarations
                        .context
                        .as_ref()
                        .map(|schema| &schema.definition),
                    procedures: &ready,
                    signatures: &signatures,
                    globals,
                    pending_global_alignments: &pending_global_alignments,
                    places: &snapshot,
                    source: declaration.location().source,
                    file,
                    target: options
                        .target
                        .as_ref()
                        .map(jai_vm::ByteTarget::from)
                        .or_else(|| {
                            options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                policy,
                                endian: jai_vm::Endian::Little,
                            })
                        }),
                    limits: options.compile_time_limits,
                    pending: RefCell::new(vec![]),
                    pending_constants: RefCell::new(vec![]),
                    pending_field_defaults: RefCell::new(vec![]),
                    cache: &cache,
                    effects,
                    effect_mode: crate::compile_time::EffectsMode::Compiler,
                    compiler,
                    runtime,
                    file_abi: &file_abi,
                    heap_abi: &heap_abi,
                    process_abi: &process_abi,
                    deferred,
                };
                let default_requests =
                    super::procedure_default_jobs::request_snapshot(declarations);
                let lookup_attempt = if initializer_lookup_allowed {
                    Some(declarations.source_lookup.begin(
                        graph,
                        declaration.id(),
                        file,
                        declaration.location(),
                    )?)
                } else {
                    None
                };
                let canonical_initializer =
                    worklist::canonical_constant(declaration, declarations, types);
                let result = {
                    let mut resolver = Resolver {
                        expression_owner: Some(context.owner),
                        debug: crate::debug_capture::Capture::default(),
                        checks: crate::safety_checks::ActiveChecks::default(),
                        local_scopes: crate::local_declarations::LocalScopes::default(),
                        context: declarations.context.as_ref(),
                        context_available: true,
                        meta,
                        procedure: constant_owners[&declaration.id()],
                        types,
                        target_layout: options.effective_layout(),
                        places,
                        signatures: &empty_signatures,
                        globals: &empty_values,
                        graph_scope: Some(FileScope {
                            declarations,
                            file,
                            substitution: None,
                        }),
                        compile_time: Some(&context),
                        symbols: graph.symbols(),
                        scopes: vec![HashMap::new()],
                        locals: vec![],
                        span: constant.span,
                        results: &[],
                        loops: vec![],
                        next_loop: 0,
                        active_push: None,
                        next_push: 0,
                        cleanups: vec![],
                        deferred_scopes: vec![],
                        cleanup_context: None,
                    };
                    // Bind quotations in the existing compile-only CodeRegistry,
                    // preserving their defining scope without resolving the body.
                    // Runtime materialization is not a representation for Code.
                    if code_constants.contains(&declaration.id()) || canonical_initializer {
                        resolver.bind_semantic_constant(constant).and_then(|()| {
                            resolver
                                .scopes
                                .last()
                                .and_then(|scope| scope.get(&constant.name))
                                .cloned()
                                .ok_or_else(|| {
                                    Diagnostic::new(
                                        constant.span,
                                        "typed constant binding did not publish its declaration",
                                    )
                                })
                        })
                    } else {
                        let expected = constant
                            .ty
                            .as_ref()
                            .map(|annotation| {
                                resolver.lexical_annotation(annotation, constant.span)
                            })
                            .transpose();
                        expected.and_then(|expected| {
                            let run = match &constant.initializer.kind {
                                syntax::ExpressionKind::CompileTime(run) => run.clone(),
                                _ => syntax::CompileTimeRun {
                                    flags: Default::default(),
                                    body: syntax::CompileTimeBody::Expression(Box::new(
                                        constant.initializer.clone(),
                                    )),
                                },
                            };
                            resolver
                                .execute_compile_time(&run, constant.initializer.span, expected)
                                .and_then(|value| {
                                    let value = value.ok_or_else(|| {
                                        Diagnostic::new(
                                            constant.span,
                                            "void #run cannot initialize a constant",
                                        )
                                    })?;
                                    crate::compile_time::materialized_binding(
                                        value,
                                        None,
                                        constant.span,
                                        resolver.meta,
                                    )
                                })
                        })
                    }
                };
                let lookup_wait = lookup_attempt.and_then(|attempt| attempt.finish(graph, &result));
                if let Some(demand) = lookup_wait {
                    let wait = SourcePreparationPending::lookup(demand);
                    initializer_lookup_wait = Some(wait);
                    source_wait = Some(wait);
                }
                let dependencies = context.pending.into_inner();
                let constants_pending = context.pending_constants.into_inner();
                let field_defaults_pending = context.pending_field_defaults.into_inner();
                if matches!(
                    mode,
                    BindingMode::Types(_)
                        | BindingMode::Headers(_)
                        | BindingMode::Initializers
                        | BindingMode::SourceRuns
                ) {
                    header_prerequisites.observe(
                        &dependencies,
                        &constants_pending,
                        &field_defaults_pending,
                    );
                }
                for &field in &field_defaults_pending {
                    meta.field_default_jobs.request(field);
                }
                let default_wait =
                    super::procedure_default_jobs::requested_since(declarations, &default_requests);
                if let Some(wait) = default_wait {
                    source_wait = Some(wait);
                }
                match result {
                    Ok(binding) => {
                        declarations.values.insert(declaration.id(), binding);
                    }
                    Err(error)
                        if !dependencies.is_empty()
                            || !constants_pending.is_empty()
                            || !field_defaults_pending.is_empty()
                            || default_wait.is_some()
                            || lookup_wait.is_some() =>
                    {
                        stalled = Some((
                            file,
                            error.span,
                            format!(
                                "{dependencies:?}; constants {constants_pending:?}; field defaults {field_defaults_pending:?}"
                            ),
                        ));
                        retry_constants.push(declaration);
                    }
                    Err(error) => return Err(located(graph, file, error)),
                }
            }
            if let Some(jobs) = initializers.as_deref_mut() {
                while let Some(job) = jobs.next() {
                    let snapshot = places.snapshot();
                    let context = Context {
                        workspace: options
                            .compiler
                            .as_ref()
                            .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                                context.current_workspace
                            }),
                        foreign: &foreign,
                        owner: job.owner,
                        generics: &declarations.generics,
                        context: declarations
                            .context
                            .as_ref()
                            .map(|schema| &schema.definition),
                        procedures: &ready,
                        signatures: &signatures,
                        globals,
                        pending_global_alignments: &pending_global_alignments,
                        places: &snapshot,
                        source: job.location.source,
                        file: job.file,
                        target: options
                            .target
                            .as_ref()
                            .map(jai_vm::ByteTarget::from)
                            .or_else(|| {
                                options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                    policy,
                                    endian: jai_vm::Endian::Little,
                                })
                            }),
                        limits: options.compile_time_limits,
                        pending: RefCell::new(vec![]),
                        pending_constants: RefCell::new(vec![]),
                        pending_field_defaults: RefCell::new(vec![]),
                        cache: &cache,
                        effects,
                        effect_mode: crate::compile_time::EffectsMode::Compiler,
                        compiler,
                        runtime,
                        file_abi: &file_abi,
                        heap_abi: &heap_abi,
                        process_abi: &process_abi,
                        deferred,
                    };
                    let default_requests =
                        super::procedure_default_jobs::request_snapshot(declarations);
                    let lookup_attempt = if initializer_lookup_allowed {
                        Some(declarations.source_lookup.begin(
                            graph,
                            job.declaration,
                            job.file,
                            job.location,
                        )?)
                    } else {
                        None
                    };
                    let result = job.evaluate(&context, declarations, types, places, meta, options);
                    let lookup_wait =
                        lookup_attempt.and_then(|attempt| attempt.finish(graph, &result));
                    if let Some(demand) = lookup_wait {
                        let wait = SourcePreparationPending::lookup(demand);
                        initializer_lookup_wait = Some(wait);
                        source_wait = Some(wait);
                    }
                    let dependencies = context.pending.into_inner();
                    let constants_pending = context.pending_constants.into_inner();
                    let fields_pending = context.pending_field_defaults.into_inner();
                    header_prerequisites.observe(
                        &dependencies,
                        &constants_pending,
                        &fields_pending,
                    );
                    for &field in &fields_pending {
                        meta.field_default_jobs.request(field);
                    }
                    let default_wait = super::procedure_default_jobs::requested_since(
                        declarations,
                        &default_requests,
                    );
                    if let Some(wait) = default_wait {
                        source_wait = Some(wait);
                    }
                    match result {
                        Ok(global) => {
                            jobs.publish(global, declarations, globals, alignment_jobs)?
                        }
                        Err(error)
                            if !dependencies.is_empty()
                                || !constants_pending.is_empty()
                                || !fields_pending.is_empty()
                                || default_wait.is_some()
                                || lookup_wait.is_some() =>
                        {
                            stalled = Some((
                                job.file,
                                error.span,
                                format!(
                                    "{dependencies:?}; constants {constants_pending:?}; field defaults {fields_pending:?}"
                                ),
                            ));
                            break;
                        }
                        Err(error) => return Err(located(graph, job.file, error)),
                    }
                }
            }
            if matches!(mode, BindingMode::Types(_) | BindingMode::Full) {
                if let Some(jobs) = discovery.as_deref_mut() {
                    if let Some(insertions) = &mut jobs.insertions {
                        let snapshot = places.snapshot();
                        let context = Context {
                            workspace: options
                                .compiler
                                .as_ref()
                                .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                                    context.current_workspace
                                }),
                            foreign: &foreign,
                            owner: method_owner,
                            generics: &declarations.generics,
                            context: declarations
                                .context
                                .as_ref()
                                .map(|schema| &schema.definition),
                            procedures: &ready,
                            signatures: &signatures,
                            globals,
                            pending_global_alignments: &pending_global_alignments,
                            places: &snapshot,
                            source: graph.file(method_file).unwrap().source(),
                            file: method_file,
                            target: options
                                .target
                                .as_ref()
                                .map(jai_vm::ByteTarget::from)
                                .or_else(|| {
                                    options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                        policy,
                                        endian: jai_vm::Endian::Little,
                                    })
                                }),
                            limits: options.compile_time_limits,
                            pending: RefCell::new(vec![]),
                            pending_constants: RefCell::new(vec![]),
                            pending_field_defaults: RefCell::new(vec![]),
                            cache: &cache,
                            effects,
                            effect_mode: crate::compile_time::EffectsMode::Compiler,
                            compiler,
                            runtime,
                            file_abi: &file_abi,
                            heap_abi: &heap_abi,
                            process_abi: &process_abi,
                            deferred,
                        };
                        insertions.evaluate(
                            admission,
                            &context,
                            &insertion_owners,
                            declarations,
                            types,
                            places,
                            meta,
                        )?;
                        let dependencies = context.pending.into_inner();
                        let constants_pending = context.pending_constants.into_inner();
                        let fields_pending = context.pending_field_defaults.into_inner();
                        header_prerequisites.observe(
                            &dependencies,
                            &constants_pending,
                            &fields_pending,
                        );
                        for field in fields_pending {
                            meta.field_default_jobs.request(field);
                        }
                        if !insertions.decisions.is_empty() {
                            return Ok(BindingProgress::Complete(completed_procedures(
                                std::mem::take(&mut ready),
                                meta,
                                &declarations.generics,
                            )));
                        }
                        if !dependencies.is_empty() || !constants_pending.is_empty() {
                            stalled = Some((
                                method_file,
                                Span::default(),
                                format!("{dependencies:?}; constants {constants_pending:?}"),
                            ));
                        } else if matches!(mode, BindingMode::Types(_))
                            && !insertions.pending.is_empty()
                        {
                            return Ok(BindingProgress::Complete(completed_procedures(
                                std::mem::take(&mut ready),
                                meta,
                                &declarations.generics,
                            )));
                        }
                    }
                }
            }
            for mut job in std::mem::take(&mut pending) {
                if matches!(
                    mode,
                    BindingMode::Types(_)
                        | BindingMode::Headers(_)
                        | BindingMode::Initializers
                        | BindingMode::SourceRuns
                ) && !header_prerequisites.procedures.contains(&job.signature.id)
                {
                    retry.push(job);
                    continue;
                }
                if matches!(mode, BindingMode::Full)
                    && job.specialization.is_none()
                    && job.modifier.is_none()
                    && let Some(signature) = declarations.signatures.get(&job.declaration)
                {
                    job.signature = signature.clone();
                }
                let declaration = graph
                    .declaration(job.declaration)
                    .expect("body job retains declaration identity");
                let procedure = match job.syntax.as_ref() {
                    Some(procedure) => procedure,
                    None => match &declaration.syntax().kind {
                        FileDeclarationKind::Procedure(procedure) => procedure,
                        _ => unreachable!(),
                    },
                };
                let file = job.file;
                let signature = &job.signature;
                if let Some(substitution) = &job.substitution
                    && graph.dependency_templates().iter().any(|template| {
                        template.declaration() == job.declaration
                            && template.location().span == procedure.span
                    })
                {
                    let key = super::source_specializations::encode(
                        declarations,
                        types,
                        meta,
                        job.declaration,
                        procedure,
                        substitution,
                    )?;
                    meta.source_specialization_keys
                        .insert(signature.id, key.clone());
                    if !graph.source_specializations().contains(&key) {
                        if discovery.is_none() {
                            return Err(located(
                                graph,
                                file,
                                Diagnostic::new(
                                    procedure.span,
                                    "specialized source dependencies require retained semantic graph discovery",
                                ),
                            ));
                        }
                        if !meta.pending_source_specializations.contains(&key) {
                            meta.pending_source_specializations.push(key);
                        }
                        retry.push(job);
                        continue;
                    }
                }

                let mut no_effects = jai_vm::NoEffects;
                let isolated_effects = crate::compile_time::SharedEffects::new(&mut no_effects);
                let isolated = job.modifier.is_some()
                    || declarations
                        .generics
                        .borrow()
                        .is_isolated_procedure(signature.id);
                let body_effects: &dyn EffectService = if isolated {
                    &isolated_effects
                } else {
                    effects
                };
                let body_cache = if isolated {
                    isolated_caches.entry(signature.id).or_default()
                } else {
                    &cache
                };
                let snapshot = places.snapshot();
                let context = Context {
                    workspace: options
                        .compiler
                        .as_ref()
                        .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                            context.current_workspace
                        }),
                    foreign: &foreign,
                    owner: signature.id,
                    generics: &declarations.generics,
                    context: declarations
                        .context
                        .as_ref()
                        .map(|schema| &schema.definition),
                    procedures: &ready,
                    signatures: &signatures,
                    globals,
                    pending_global_alignments: &pending_global_alignments,
                    places: &snapshot,
                    source: declaration.location().source,
                    file,
                    target: options
                        .target
                        .as_ref()
                        .map(jai_vm::ByteTarget::from)
                        .or_else(|| {
                            options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                policy,
                                endian: jai_vm::Endian::Little,
                            })
                        }),
                    limits: options.compile_time_limits,
                    pending: RefCell::new(vec![]),
                    cache: body_cache,
                    effects: body_effects,
                    effect_mode: if isolated {
                        crate::compile_time::EffectsMode::Isolated
                    } else {
                        crate::compile_time::EffectsMode::Compiler
                    },
                    compiler,
                    runtime,
                    file_abi: &file_abi,
                    heap_abi: &heap_abi,
                    process_abi: &process_abi,
                    deferred,
                    pending_constants: RefCell::new(vec![]),
                    pending_field_defaults: RefCell::new(vec![]),
                };
                let default_requests =
                    super::procedure_default_jobs::request_snapshot(declarations);
                let lookup_attempt = if initializer_lookup_allowed {
                    Some(declarations.source_lookup.begin(
                        graph,
                        job.declaration,
                        file,
                        declaration.location(),
                    )?)
                } else {
                    None
                };
                let result = {
                    if job.modifier.is_none() {
                        meta.remember_inline_hint(signature.id, procedure.inline_hint);
                        meta.remember_execution(signature.id, procedure.execution);
                    }
                    meta.storage_alignments.clear_procedure(signature.id);
                    let mut resolver = Resolver {
                        expression_owner: Some(signature.id),
                        debug: crate::debug_capture::Capture::default(),
                        checks: crate::safety_checks::ActiveChecks::default()
                            .overridden(procedure.checks),
                        local_scopes: crate::local_declarations::LocalScopes::default(),
                        context: declarations.context.as_ref(),
                        context_available: procedure.context == jai_types::ContextMode::Implicit,
                        // Body check flags are not part of the canonical procedure type.
                        meta,
                        procedure: signature.id,
                        types,
                        target_layout: options.effective_layout(),
                        places,
                        signatures: &empty_signatures,
                        globals: &empty_values,
                        graph_scope: Some(FileScope {
                            declarations,
                            file,
                            substitution: job.substitution.as_ref(),
                        }),
                        compile_time: Some(&context),
                        symbols: graph.symbols(),
                        scopes: vec![HashMap::new()],
                        locals: vec![],
                        span: procedure.span,
                        results: &signature.results,
                        loops: vec![],
                        next_loop: 0,
                        active_push: None,
                        next_push: 0,
                        cleanups: vec![],
                        deferred_scopes: vec![],
                        cleanup_context: None,
                    };
                    resolver.debug.enter_policy(procedure.debug);
                    (|| {
                        resolver.register_result_contracts(
                            signature.id,
                            &procedure.results,
                            &signature.results,
                        )?;
                        if let Some(substitution) = &job.substitution {
                            for parameter in &procedure.parameters {
                                if parameter.evaluation != syntax::ParameterEvaluation::Discard {
                                    continue;
                                }
                                let Some(value) = substitution.constant(parameter.name) else {
                                    continue;
                                };
                                use crate::polymorphism::BakedValue;
                                let ty = match value {
                                    BakedValue::Value(value) => value.ty,
                                    BakedValue::Type(_) => resolver.types.meta_type(),
                                    BakedValue::Float(value) => resolver.types.float(value.ty()),
                                    BakedValue::String(_) => resolver.types.string(),
                                    BakedValue::Code(_) => resolver.types.code_type(),
                                };
                                resolver.bind_discarded_parameter(parameter.name, ty)?;
                            }
                        }
                        let mut parameters = Vec::new();
                        for parameter in &signature.parameters {
                            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                                resolver.bind_discarded_parameter(parameter.name, parameter.ty)?;
                                continue;
                            }
                            let ordinal = parameters.len();
                            let local = resolver.declare_typed(parameter.name, parameter.ty)?;
                            if let Some(source) = procedure
                                .parameters
                                .iter()
                                .find(|source| source.name == parameter.name)
                            {
                                resolver.bind_callback_parameter(
                                    local.place(),
                                    source,
                                    parameter.default.as_ref(),
                                )?;
                                if job.modifier.is_none() {
                                    resolver.debug_parameter(
                                        local,
                                        parameter.name,
                                        source.span,
                                        ordinal,
                                    )?;
                                }
                            }
                            parameters.push(local);
                        }
                        for parameter in &signature.parameters {
                            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                                continue;
                            }
                            if procedure
                                .parameters
                                .iter()
                                .any(|source| source.name == parameter.name && source.using)
                            {
                                let storage = resolver.storage(parameter.name)?;
                                resolver.using_record(storage)?;
                            }
                        }
                        let body = if job.modifier.is_some() {
                            // Auxiliary modifier source returns acceptance, explanation,
                            // and binding slots through its checked plan/signature. Its
                            // source header intentionally has no ordinary result rows.
                            resolver.block(&procedure.body, false)?
                        } else {
                            resolver.named_result_body(&procedure.results, &procedure.body)?
                        };
                        if !signature.results.is_empty() && body.flow != Flow::Terminates {
                            return Err(Diagnostic::new(
                                procedure.span,
                                "value-returning procedure may reach its end",
                            ));
                        }
                        if job.modifier.is_none() {
                            resolver.remember_procedure_notes(signature.id, &procedure.notes)?;
                            resolver.publish_debug(procedure.name, procedure.span)?;
                        }
                        Ok(Procedure {
                            id: signature.id,
                            signature: signature.ty,
                            parameters,
                            locals: resolver.locals,
                            cleanups: resolver.cleanups,
                            body,
                        })
                    })()
                };
                let lookup_wait = lookup_attempt.and_then(|attempt| attempt.finish(graph, &result));
                if let Some(demand) = lookup_wait {
                    let wait = SourcePreparationPending::lookup(demand);
                    initializer_lookup_wait = Some(wait);
                    source_wait = Some(wait);
                }
                let dependencies = context.pending.into_inner();
                let constants_pending = context.pending_constants.into_inner();
                let field_defaults_pending = context.pending_field_defaults.into_inner();
                if matches!(
                    mode,
                    BindingMode::Types(_)
                        | BindingMode::Headers(_)
                        | BindingMode::Initializers
                        | BindingMode::SourceRuns
                ) {
                    header_prerequisites.observe(
                        &dependencies,
                        &constants_pending,
                        &field_defaults_pending,
                    );
                }
                for &field in &field_defaults_pending {
                    meta.field_default_jobs.request(field);
                }
                let default_wait =
                    super::procedure_default_jobs::requested_since(declarations, &default_requests);
                if let Some(wait) = default_wait {
                    source_wait = Some(wait);
                }
                match result {
                    Ok(procedure) => {
                        ready.insert(signature.id, procedure);
                        if let Some(key) = &job.modifier {
                            declarations
                                .generics
                                .borrow_mut()
                                .complete_modifier_body(key)
                                .map_err(|error| located(graph, file, error))?;
                        }
                        if let Some(id) = job.specialization {
                            declarations
                                .generics
                                .borrow_mut()
                                .complete(id)
                                .map_err(|error| located(graph, file, error))?;
                        }
                    }
                    Err(error)
                        if !dependencies.is_empty()
                            || !constants_pending.is_empty()
                            || !field_defaults_pending.is_empty()
                            || default_wait.is_some()
                            || lookup_wait.is_some() =>
                    {
                        stalled = Some((
                            file,
                            error.span,
                            format!(
                                "{dependencies:?}; constants {constants_pending:?}; field defaults {field_defaults_pending:?}"
                            ),
                        ));
                        if let Some(key) = &job.modifier {
                            declarations
                                .generics
                                .borrow_mut()
                                .retry_modifier_body(key)
                                .map_err(|error| located(graph, file, error))?;
                        } else if let Some(id) = job.specialization {
                            declarations
                                .generics
                                .borrow_mut()
                                .retry_body(id)
                                .map_err(|error| located(graph, file, error))?;
                        } else {
                            retry.push(job);
                        }
                    }
                    Err(error) => {
                        body_error.get_or_insert((
                            file,
                            error,
                            job.specialization,
                            job.modifier.clone(),
                        ));
                        if let Some(key) = &job.modifier {
                            declarations
                                .generics
                                .borrow_mut()
                                .retry_modifier_body(key)
                                .map_err(|error| located(graph, file, error))?;
                        } else if let Some(id) = job.specialization {
                            declarations
                                .generics
                                .borrow_mut()
                                .retry_body(id)
                                .map_err(|error| located(graph, file, error))?;
                        } else {
                            retry.push(job);
                        }
                    }
                }
            }
            if (matches!(mode, BindingMode::Full)
                && (!alignment_jobs.is_empty() || !file_guards.is_empty() || discovery.is_some()))
                || (matches!(mode, BindingMode::SourceRuns) && !alignment_jobs.is_empty())
            {
                let snapshot = places.snapshot();
                let context = Context {
                    workspace: options
                        .compiler
                        .as_ref()
                        .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                            context.current_workspace
                        }),
                    foreign: &foreign,
                    owner: method_owner,
                    generics: &declarations.generics,
                    context: declarations
                        .context
                        .as_ref()
                        .map(|schema| &schema.definition),
                    procedures: &ready,
                    signatures: &signatures,
                    globals,
                    pending_global_alignments: &pending_global_alignments,
                    places: &snapshot,
                    source: graph.file(method_file).unwrap().source(),
                    file: method_file,
                    target: options
                        .target
                        .as_ref()
                        .map(jai_vm::ByteTarget::from)
                        .or_else(|| {
                            options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                policy,
                                endian: jai_vm::Endian::Little,
                            })
                        }),
                    limits: options.compile_time_limits,
                    pending: RefCell::new(vec![]),
                    pending_constants: RefCell::new(vec![]),
                    pending_field_defaults: RefCell::new(vec![]),
                    cache: &cache,
                    effects,
                    effect_mode: crate::compile_time::EffectsMode::Compiler,
                    compiler,
                    runtime,
                    file_abi: &file_abi,
                    heap_abi: &heap_abi,
                    process_abi: &process_abi,
                    deferred,
                };
                let progress = alignments::bind(
                    &context,
                    alignments::Requests {
                        jobs: alignment_jobs,
                        owners: &alignment_owners,
                    },
                    declarations,
                    types,
                    places,
                    meta,
                    options,
                );
                completed_alignments += progress.completed;
                alignment_error = progress.error;
                if progress.stalled.is_some() {
                    stalled = progress.stalled;
                }
                if matches!(mode, BindingMode::SourceRuns) {
                    header_prerequisites.observe(
                        &context.pending.borrow(),
                        &context.pending_constants.borrow(),
                        &context.pending_field_defaults.borrow(),
                    );
                    for &field in context.pending_field_defaults.borrow().iter() {
                        meta.field_default_jobs.request(field);
                    }
                }
                if matches!(mode, BindingMode::Full) {
                    let progress = file_conditions::bind(
                        &context,
                        &mut file_guards,
                        declarations,
                        types,
                        places,
                        meta,
                    );
                    completed_file_guards += progress.completed;
                    file_guard_error = progress.error;
                    if let Some(jobs) = discovery.as_deref_mut() {
                        super::discovery_conditions::evaluate(
                            &context,
                            declarations,
                            types,
                            places,
                            meta,
                            jobs,
                        )?;
                        if !jobs.decisions.is_empty()
                            || !jobs.case_decisions.is_empty()
                            || !jobs.using_decisions.is_empty()
                            || !meta.pending_source_specializations.is_empty()
                        {
                            return Ok(BindingProgress::Complete(completed_procedures(
                                std::mem::take(&mut ready),
                                meta,
                                &declarations.generics,
                            )));
                        }
                    }
                }
            }
            // An actual initializer lookup demand may select an independent
            // original producer. Its completed input journal is inspected before
            // the blocked job can bind against a newly published graph.
            let prefix_runs = source_prefix
                && declarations.context.is_some()
                && (matches!(mode, BindingMode::Types(_) | BindingMode::SourceRuns)
                    || initializer_lookup_wait.is_some());
            let mut source_run_checkpoint = false;
            if (matches!(mode, BindingMode::Full | BindingMode::SourceRuns) || prefix_runs)
                && (prefix_runs || retry_constants.is_empty())
            {
                signatures.extend(declarations.generics.borrow().signature_snapshot());
                let mut retry_runs = vec![];
                for (request, owner_ref) in runs.drain(..) {
                    if source_run_checkpoint || !retry_runs.is_empty() {
                        retry_runs.push((request, owner_ref));
                        continue;
                    }
                    let owner = owner_ref;
                    let snapshot = places.snapshot();
                    let context = Context {
                        workspace: options
                            .compiler
                            .as_ref()
                            .map_or(jai_vm::WorkspaceId::from_raw(1).unwrap(), |context| {
                                context.current_workspace
                            }),
                        foreign: &foreign,
                        owner,
                        generics: &declarations.generics,
                        context: declarations
                            .context
                            .as_ref()
                            .map(|schema| &schema.definition),
                        procedures: &ready,
                        signatures: &signatures,
                        globals,
                        pending_global_alignments: &pending_global_alignments,
                        places: &snapshot,
                        source: request.syntax.location.source,
                        file: request.file,
                        target: options
                            .target
                            .as_ref()
                            .map(jai_vm::ByteTarget::from)
                            .or_else(|| {
                                options.effective_layout().map(|policy| jai_vm::ByteTarget {
                                    policy,
                                    endian: jai_vm::Endian::Little,
                                })
                            }),
                        limits: options.compile_time_limits,
                        pending: RefCell::new(vec![]),
                        cache: &cache,
                        effects,
                        effect_mode: crate::compile_time::EffectsMode::Compiler,
                        compiler,
                        runtime,
                        file_abi: &file_abi,
                        heap_abi: &heap_abi,
                        process_abi: &process_abi,
                        deferred,
                        pending_constants: RefCell::new(vec![]),
                        pending_field_defaults: RefCell::new(vec![]),
                    };
                    let mut resolver = Resolver {
                        expression_owner: Some(context.owner),
                        debug: crate::debug_capture::Capture::default(),
                        checks: crate::safety_checks::ActiveChecks::default(),
                        local_scopes: crate::local_declarations::LocalScopes::default(),
                        context: declarations.context.as_ref(),
                        context_available: true,
                        meta,
                        procedure: owner,
                        types,
                        target_layout: options.effective_layout(),
                        places,
                        signatures: &empty_signatures,
                        globals: &empty_values,
                        graph_scope: Some(FileScope {
                            declarations,
                            file: request.file,
                            substitution: None,
                        }),
                        compile_time: Some(&context),
                        symbols: graph.symbols(),
                        scopes: vec![HashMap::new()],
                        locals: vec![],
                        span: request.syntax.location.span,
                        results: &[],
                        loops: vec![],
                        next_loop: 0,
                        active_push: None,
                        next_push: 0,
                        cleanups: vec![],
                        deferred_scopes: vec![],
                        cleanup_context: None,
                    };
                    let default_requests =
                        super::procedure_default_jobs::request_snapshot(declarations);
                    let result = resolver.resolve_compile_time_statement(
                        &syntax::CompileTimeRun {
                            flags: request.syntax.flags,
                            body: request.syntax.body.clone(),
                        },
                        request.syntax.location.span,
                    );
                    let dependencies = context.pending.into_inner();
                    let constants_pending = context.pending_constants.into_inner();
                    let field_defaults_pending = context.pending_field_defaults.into_inner();
                    if prefix_runs {
                        header_prerequisites.observe(
                            &dependencies,
                            &constants_pending,
                            &field_defaults_pending,
                        );
                    }
                    for &field in &field_defaults_pending {
                        meta.field_default_jobs.request(field);
                    }
                    let default_wait = super::procedure_default_jobs::requested_since(
                        declarations,
                        &default_requests,
                    );
                    if let Some(wait) = default_wait {
                        source_wait = Some(wait);
                    }
                    match result {
                        Ok(_) => {
                            completed_runs += 1;
                            source_run_checkpoint = prefix_runs;
                        }
                        Err(error)
                            if !dependencies.is_empty()
                                || !constants_pending.is_empty()
                                || !field_defaults_pending.is_empty()
                                || default_wait.is_some() =>
                        {
                            stalled = Some((
                                request.file,
                                error.span,
                                format!(
                                    "{dependencies:?}; constants {constants_pending:?}; field defaults {field_defaults_pending:?}"
                                ),
                            ));
                            retry_runs.push((request, owner_ref));
                        }
                        Err(error) => return Err(located(graph, request.file, error)),
                    }
                }
                runs = retry_runs;
            }
            procedure_defaults.admit_requested(declarations)?;
            if let Some(error) = cache.take_callback_failure() {
                return Err(LocatedDiagnostic::new(root_source, error));
            }
            if source_run_checkpoint {
                pending = retry;
                constants = retry_constants;
                return Ok(BindingProgress::SourceRunReady);
            }
            if prefix_runs
                && (!matches!(mode, BindingMode::Initializers)
                    || initializers.as_ref().is_some_and(|jobs| jobs.is_complete()))
                && runs.is_empty()
                && procedure_defaults.is_empty()
                && alignment_jobs.is_empty()
                && header_prerequisites.ready(&meta.field_default_jobs)
                && header_prerequisites
                    .constants
                    .iter()
                    .all(|id| declarations.values.contains_key(id))
                && meta.record_specializations.queued_modifier_count() == 0
                && !declarations.generics.borrow().has_pending_modifier_bodies()
                && cache.pending_execution().is_none()
                && isolated_caches
                    .values()
                    .all(|cache| cache.pending_execution().is_none())
            {
                pending = retry;
                constants = retry_constants;
                return Ok(BindingProgress::SourceRunsReady);
            }
            if matches!(mode, BindingMode::Headers(_))
                && procedure_defaults.is_empty()
                && header_prerequisites.ready(&meta.field_default_jobs)
                && cache.pending_execution().is_none()
                && isolated_caches
                    .values()
                    .all(|cache| cache.pending_execution().is_none())
            {
                pending = retry;
                constants = retry_constants;
                return Ok(BindingProgress::HeadersReady);
            }
            if matches!(mode, BindingMode::Types(_))
                && procedure_defaults.is_empty()
                && !prefix_runs
                && meta.record_specializations.queued_modifier_count() == 0
                && header_prerequisites
                    .constants
                    .iter()
                    .all(|id| declarations.values.contains_key(id))
                && !declarations.generics.borrow().has_pending_modifier_bodies()
                && cache.pending_execution().is_none()
            {
                pending = retry;
                constants = retry_constants;
                return Ok(BindingProgress::TypesReady);
            }
            if matches!(mode, BindingMode::Initializers)
                && procedure_defaults.is_empty()
                && initializers.as_ref().is_some_and(|jobs| jobs.is_complete())
                && cache.pending_execution().is_none()
            {
                pending = retry;
                constants = retry_constants;
                return Ok(BindingProgress::InitializersReady);
            }
            if ready.len()
                + meta.local_declarations.semantic_ready_count()
                + declarations.values.len()
                + declarations.generics.borrow().specialization_count()
                + completed_runs
                + completed_alignments
                + completed_file_guards
                + declarations.generics.borrow().modifier_job_count()
                + meta.field_default_jobs.ready().count()
                + meta.field_default_jobs.requested_count()
                + header_prerequisites.len()
                + meta.record_specializations.completed_modifier_count()
                + initializers.as_ref().map_or(0, |jobs| jobs.completed())
                + procedure_defaults.progress_count()
                == before
                && (
                    declarations.generics.borrow().callback_readiness_revision(),
                    meta.local_declarations.callback_readiness_revision(),
                ) == before_revision
                && record_count == meta.record_specializations.records().count()
                && before_modifier_queue == meta.record_specializations.queued_modifier_count()
            {
                if let Some(mut suspension) = cache.pending_execution() {
                    if !suspension.dependencies.iter().any(|dependency| {
                        matches!(
                            dependency,
                            jai_vm::Dependency::Effect(_) | jai_vm::Dependency::Host(_)
                        )
                    }) {
                        return Err(LocatedDiagnostic {
                            location: suspension.diagnostic.location,
                            message: format!(
                                "unresolved or cyclic suspended #run dependencies: {:?}",
                                suspension.dependencies
                            ),
                        });
                    }
                    pending = retry;
                    constants = retry_constants;
                    suspension.source = source_wait;
                    return Ok(BindingProgress::Pending(suspension));
                }
                if let Some(wait) = source_wait {
                    pending = retry;
                    constants = retry_constants;
                    return Ok(BindingProgress::Pending(LibraryPending {
                        dependencies: vec![],
                        diagnostic: wait.diagnostic(graph),
                        source: Some(wait),
                    }));
                }
                if matches!(mode, BindingMode::Full) && discovery.is_some() {
                    return Ok(BindingProgress::Complete(completed_procedures(
                        std::mem::take(&mut ready),
                        meta,
                        &declarations.generics,
                    )));
                }
                if let Some(error) = method_error {
                    return Err(error);
                }
                if let Some(error) = file_guard_error {
                    return Err(error);
                }
                if let Some(error) = alignment_error {
                    return Err(error);
                }
                if let Some((file, error, specialization, modifier)) = body_error {
                    if let Some(key) = modifier {
                        declarations
                            .generics
                            .borrow_mut()
                            .fail_modifier(&key, error.clone())
                            .map_err(|failure| located(graph, file, failure))?;
                    }
                    if let Some(id) = specialization {
                        declarations
                            .generics
                            .borrow_mut()
                            .fail(id, error.clone())
                            .map_err(|failure| located(graph, file, failure))?;
                    }
                    return Err(located(graph, file, error));
                }
                if stalled.is_none()
                    && retry.is_empty()
                    && retry_constants.is_empty()
                    && runs.is_empty()
                    && alignment_jobs.is_empty()
                    && file_guards.is_empty()
                    && !declarations.generics.borrow().has_pending_bodies()
                    && !declarations.generics.borrow().has_pending_modifier_bodies()
                {
                    break;
                }
                // A partial type phase retains the actual source demand even
                // when no VM dependency or body attempt has escaped this pass.
                let demanded_location = if stalled.is_none()
                    && let BindingMode::Types(pending) = mode
                {
                    Some(pending.location())
                } else {
                    None
                };
                let (file, span, dependencies) = match stalled {
                    Some(wait) => wait,
                    None => {
                        // Retained source work can be blocked before a VM read.
                        // Diagnose its actual source front, never fabricate a VM dependency.
                        let (file, span) = if let Some(job) = retry.first() {
                            let source = graph
                                .declaration(job.declaration)
                                .expect("retained body source");
                            (
                                job.file,
                                job.syntax
                                    .as_ref()
                                    .map_or(source.location().span, |source| source.span),
                            )
                        } else if let Some(source) = retry_constants.first() {
                            (source.file(), source.location().span)
                        } else if let Some((source, _)) = runs.first() {
                            (source.file, source.syntax.location.span)
                        } else if let Some(source) = alignment_jobs.first() {
                            (
                                source.file,
                                graph
                                    .declaration(source.declaration)
                                    .expect("alignment source")
                                    .location()
                                    .span,
                            )
                        } else {
                            // This is the real aggregate method sweep's defining file.
                            // No concrete source front remains to attribute more narrowly.
                            (method_file, Span::default())
                        };
                        let bodies = retry
                            .iter()
                            .map(|job| (job.declaration, job.signature.id))
                            .collect::<Vec<_>>();
                        let values = retry_constants
                            .iter()
                            .map(|source| source.id())
                            .collect::<Vec<_>>();
                        (
                            file,
                            span,
                            format!(
                                "retained source work made no progress; mode {mode:?}; selected constants ready {}, field prerequisites ready {}, record modifiers queued {}, selected layout {:?}, isolated VM waits {}; bodies {bodies:?}, constants {values:?}, original runs {}, alignment jobs {}, file guards {}, selected defaults {}, generic bodies {}, modifier bodies {}",
                                header_prerequisites
                                    .constants
                                    .iter()
                                    .all(|id| declarations.values.contains_key(id)),
                                header_prerequisites.ready(&meta.field_default_jobs),
                                meta.record_specializations.queued_modifier_count(),
                                options.effective_layout(),
                                isolated_caches
                                    .values()
                                    .any(|cache| cache.pending_execution().is_some()),
                                runs.len(),
                                alignment_jobs.len(),
                                file_guards.len(),
                                !procedure_defaults.is_empty(),
                                declarations.generics.borrow().has_pending_bodies(),
                                declarations.generics.borrow().has_pending_modifier_bodies()
                            ),
                        )
                    }
                };
                let mut failure = located(
                    graph,
                    file,
                    Diagnostic::new(
                        span,
                        format!("unresolved or cyclic #run dependencies: {dependencies:?}"),
                    ),
                );
                if let Some(location) = demanded_location {
                    failure.location = location;
                }
                return Err(failure);
            }
            pending = retry;
            constants = retry_constants;
            methods_pending |= record_count != meta.record_specializations.records().count();
        }

        Ok(BindingProgress::Complete(completed_procedures(
            std::mem::take(&mut ready),
            meta,
            &declarations.generics,
        )))
    })();
    let worklist = Worklist {
        file_abi,
        heap_abi,
        process_abi,
        code_constants,
        record_callable_aliases,
        signatures,
        foreign,
        cache,
        ready,
        pending,
        constants,
        constant_owners,
        runs,
        completed_runs,
        isolated_caches,
        completed_alignments,
        file_guards,
        completed_file_guards,
        alignment_owners,
        method_file,
        method_owner,
        methods_pending,
        procedure_defaults,
        header_prerequisites,
        insertion_owners,
    };
    match &outcome {
        Ok(
            BindingProgress::Pending(_)
            | BindingProgress::SourceRunReady
            | BindingProgress::SourceRunsReady
            | BindingProgress::TypesReady
            | BindingProgress::HeadersReady
            | BindingProgress::InitializersReady,
        ) => *retained = Some(worklist),
        _ => {
            if let Err(cancel_error) = worklist.cancel(effects) {
                let location = match &outcome {
                    Err(error) => error.location,
                    _ => jai_source::SourceSpan {
                        source: root_source,
                        span: Span::default(),
                    },
                };
                return Err(LocatedDiagnostic {
                    location,
                    message: cancel_error.to_string(),
                });
            }
        }
    }
    outcome
}

fn completed_procedures(
    mut ready: HashMap<ProcedureId, Procedure>,
    meta: &mut crate::reflection::MetaContext,
    generics: &RefCell<crate::polymorphism::integration::GenericContext>,
) -> Vec<Procedure> {
    ready.extend(meta.local_declarations.ready_snapshot());
    ready.retain(|id, _| {
        if generics.borrow().is_modifier_procedure(*id) {
            meta.storage_alignments.clear_procedure(*id);
            false
        } else {
            true
        }
    });
    let mut procedures: Vec<_> = ready.into_values().collect();
    procedures.sort_by_key(|procedure| procedure.id.index());
    procedures
}
