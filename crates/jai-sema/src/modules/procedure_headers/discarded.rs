//! Header defaults prove discarded compatibility in their defining scope.
use super::*;

pub(super) fn check(
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    request: DefaultCheck<'_>,
) -> Result<(), LocatedDiagnostic> {
    let DefaultCheck {
        file,
        owner,
        name,
        expression,
        expected,
    } = request;
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut places = PlaceRegistry::new();
    let mut resolver = Resolver {
        expression_owner: None,
        debug: crate::debug_capture::Capture::default(),
        checks: crate::safety_checks::ActiveChecks::default(),
        local_scopes: crate::local_declarations::LocalScopes::default(),
        context: declarations.context.as_ref(),
        context_available: true,
        meta,
        procedure: owner,
        types,
        target_layout: declarations.nominals.annotation_target(),
        places: &mut places,
        signatures: &signatures,
        globals: &globals,
        graph_scope: Some(FileScope {
            declarations,
            file,
            substitution: None,
        }),
        compile_time: None,
        symbols: declarations.graph.symbols(),
        scopes: vec![HashMap::new()],
        locals: vec![],
        span: expression.span,
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
        .check_discarded_argument(expression, expected)
        .map_err(|error| located(declarations.graph, file, error))?;
    if let Some(contract) = resolver
        .preview_callback_expression_contract(expression)
        .map_err(|error| located(declarations.graph, file, error))?
        && contract.ty == expected
    {
        resolver
            .meta
            .callbacks
            .discarded_defaults
            .insert((owner, name), contract);
    }
    Ok(())
}

pub(super) struct DefaultCheck<'a> {
    pub(super) file: FileInstanceId,
    pub(super) owner: ProcedureId,
    pub(super) name: Symbol,
    pub(super) expression: &'a syntax::Expression,
    pub(super) expected: TypeId,
}
