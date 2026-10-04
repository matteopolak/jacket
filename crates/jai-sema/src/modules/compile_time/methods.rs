//! Check instantiated record methods through the same provider and dependency channels.
use super::*;

pub(super) fn sweep(
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<(), LocatedDiagnostic> {
    sweep_with_demand(context, declarations, types, places, meta, None)
}

pub(super) fn sweep_headers(
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    required: &std::collections::HashSet<jai_ir::ProcedureId>,
) -> Result<(), LocatedDiagnostic> {
    sweep_with_demand(context, declarations, types, places, meta, Some(required))
}

pub(super) fn sweep_types(
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<(), LocatedDiagnostic> {
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut resolver = Resolver {
        conditional_subjects: Vec::new(),
        expression_owner: Some(context.owner),
        debug: crate::debug_capture::Capture::default(),
        checks: crate::safety_checks::ActiveChecks::default(),
        local_scopes: crate::local_declarations::LocalScopes::default(),
        context: declarations.context.as_ref(),
        context_available: false,
        meta,
        procedure: context.owner,
        types,
        target_layout: context.target.map(|target| target.policy),
        places,
        signatures: &signatures,
        globals: &globals,
        graph_scope: Some(FileScope {
            declarations,
            file: context.file,
            substitution: None,
        }),
        compile_time: Some(context),
        symbols: declarations.graph.symbols(),
        scopes: vec![HashMap::new()],
        locals: vec![],
        span: Span::default(),
        results: &[],
        loops: vec![],
        next_loop: 0,
        active_push: None,
        next_push: 0,
        cleanups: vec![],
        deferred_scopes: vec![],
        cleanup_context: None,
    };
    resolver
        .service_record_modifiers(aggregates::parameterized::RecordModifierPolicy {
            context: jai_types::ContextMode::None,
            checks: syntax::SafetyChecks::default(),
        })
        .map(|_| ())
}

fn sweep_with_demand(
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    required: Option<&std::collections::HashSet<jai_ir::ProcedureId>>,
) -> Result<(), LocatedDiagnostic> {
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut resolver = Resolver {
        conditional_subjects: Vec::new(),
        expression_owner: Some(context.owner),
        debug: crate::debug_capture::Capture::default(),
        checks: crate::safety_checks::ActiveChecks::default(),
        local_scopes: crate::local_declarations::LocalScopes::default(),
        context: declarations.context.as_ref(),
        context_available: true,
        meta,
        procedure: context.owner,
        types,
        target_layout: context.target.map(|target| target.policy),
        places,
        signatures: &signatures,
        globals: &globals,
        graph_scope: Some(FileScope {
            declarations,
            file: context.file,
            substitution: None,
        }),
        compile_time: Some(context),
        symbols: declarations.graph.symbols(),
        scopes: vec![HashMap::new()],
        locals: vec![],
        span: Span::default(),
        results: &[],
        loops: vec![],
        next_loop: 0,
        active_push: None,
        next_push: 0,
        cleanups: vec![],
        deferred_scopes: vec![],
        cleanup_context: None,
    };
    if let Some(required) = required {
        let mut required = required.clone();
        required.extend(resolver.meta.field_default_jobs.prerequisite_procedures());
        return resolver.with_record_method_prerequisites(&required, |resolver| {
            let fields = super::super::field_default_jobs::sweep(resolver);
            let bodies = resolver.bind_record_method_prerequisites_located(&required);
            bodies.and(fields)
        });
    }
    // Body annotations can queue an actual recipe after the initial type cursor.
    // Service that same retained intent before retrying its source body; count
    // modifiers keep their genuine no-context execution policy here as well.
    resolver.service_record_modifiers(aggregates::parameterized::RecordModifierPolicy {
        context: jai_types::ContextMode::None,
        checks: syntax::SafetyChecks::default(),
    })?;
    // Complete defaults through this worklist's genuine provider. A pending
    // default must still allow default-free sibling bodies to become ready.
    let fields = super::super::field_default_jobs::sweep(&mut resolver);
    let headers = resolver.complete_all_record_method_signatures_located();
    let bodies = resolver.bind_all_record_methods_located();
    match bodies {
        Ok(()) => fields,
        Err(error) => Err(fields.err().or_else(|| headers.err()).unwrap_or(error)),
    }
}
