//! Typed decisions for retained dependency discovery, using normal readiness.
use super::*;
use crate::compile_time::Context;
use jai_modules::{DeferredCondition, DiscoveryConditionContext};

pub(super) struct Jobs<'a> {
    requests: &'a [DeferredCondition],
    pub(super) insertions: Option<insertion_jobs::Jobs<'a>>,
    cases: &'a [jai_modules::DeferredCase],
    using: &'a [jai_modules::FileUsingRequest],
    pub(super) using_decisions: Vec<(jai_modules::UsingRequestId, jai_modules::FileUsingDecision)>,
    pub(super) using_pending: Vec<DiscoveryUsingPending>,
    pub(super) case_decisions: Vec<(jai_modules::CaseRequestId, syntax::CompileTimeCaseChoice)>,
    pub(super) case_pending: Vec<DiscoveryCasePending>,
    pub(super) decisions: Vec<(jai_modules::ConditionRequestId, bool)>,
    pub(super) pending: Vec<DiscoveryConditionPending>,
}

impl<'a> Jobs<'a> {
    pub(super) fn new(requests: &'a [DeferredCondition]) -> Self {
        Self {
            requests,
            insertions: None,
            cases: &[],
            using: &[],
            using_decisions: vec![],
            using_pending: vec![],
            case_decisions: Vec::new(),
            case_pending: Vec::new(),
            decisions: Vec::new(),
            pending: Vec::new(),
        }
    }
    pub(super) fn new_insertions(requests: &'a [jai_modules::DeclarationInsertionRequest]) -> Self {
        let mut jobs = Self::new(&[]);
        jobs.insertions = Some(insertion_jobs::Jobs::new(requests));
        jobs
    }
    pub(super) fn has_insertions(&self) -> bool {
        self.insertions.is_some()
    }

    pub(super) fn new_using(requests: &'a [jai_modules::FileUsingRequest]) -> Self {
        let mut jobs = Self::new(&[]);
        jobs.using = requests;
        jobs
    }
    pub(super) fn new_cases(cases: &'a [jai_modules::DeferredCase]) -> Self {
        let mut jobs = Self::new(&[]);
        jobs.cases = cases;
        jobs
    }
}

pub(super) fn evaluate(
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    jobs: &mut Jobs<'_>,
) -> Result<usize, LocatedDiagnostic> {
    jobs.pending.clear();
    jobs.case_pending.clear();
    jobs.using_pending.clear();
    let before = jobs.decisions.len() + jobs.case_decisions.len() + jobs.using_decisions.len();
    for request in jobs.requests {
        if request.selected.is_some() || jobs.decisions.iter().any(|(id, _)| *id == request.id) {
            continue;
        }
        let source_owner = request.specialization.as_ref().and_then(|key| {
            meta.source_specialization_keys
                .iter()
                .find(|(_, candidate)| *candidate == key)
                .map(|(owner, _)| *owner)
        });
        let owner = match &request.context {
            DiscoveryConditionContext::Lexical {
                declaration, ..
            } => declarations
                .signatures
                .get(declaration)
                .map_or(context.owner, |signature| signature.id),
            DiscoveryConditionContext::File => context.owner,
        };
        let child = context.for_source(
            source_owner.unwrap_or(owner),
            request.file,
            request.location.source,
        );
        let result = evaluate_one(request, &child, declarations, types, places, meta);
        let procedures = child.pending.into_inner();
        let constants = child.pending_constants.into_inner();
        match result {
            Ok(selected) => jobs.decisions.push((request.id, selected)),
            Err(diagnostic) => jobs.pending.push(DiscoveryConditionPending {
                request: request.id,
                diagnostic,
                procedures,
                constants,
            }),
        }
    }
    for request in jobs.cases {
        if request.selected.is_some() || jobs.case_decisions.iter().any(|(id, _)| *id == request.id)
        {
            continue;
        }
        let source_owner = request.specialization.as_ref().and_then(|key| {
            meta.source_specialization_keys
                .iter()
                .find(|(_, candidate)| *candidate == key)
                .map(|(owner, _)| *owner)
        });
        let owner = match &request.context {
            DiscoveryConditionContext::Lexical {
                declaration, ..
            } => declarations
                .signatures
                .get(declaration)
                .map_or(context.owner, |signature| signature.id),
            DiscoveryConditionContext::File => context.owner,
        };
        let child = context.for_source(
            source_owner.unwrap_or(owner),
            request.file,
            request.location.source,
        );
        let result = evaluate_case_one(request, &child, declarations, types, places, meta);
        let procedures = child.pending.into_inner();
        let constants = child.pending_constants.into_inner();
        match result {
            Ok(choice) => jobs.case_decisions.push((request.id, choice)),
            Err(diagnostic) => jobs.case_pending.push(DiscoveryCasePending {
                request: request.id,
                diagnostic,
                procedures,
                constants,
            }),
        }
    }
    for request in jobs.using {
        if request.decision.is_some()
            || jobs.using_decisions.iter().any(|(id, _)| *id == request.id)
        {
            continue;
        }
        let source_owner = request.specialization.as_ref().and_then(|key| {
            meta.source_specialization_keys
                .iter()
                .find(|(_, candidate)| *candidate == key)
                .map(|(owner, _)| *owner)
        });
        let owner = request
            .owner
            .and_then(|id| declarations.signatures.get(&id))
            .map_or(context.owner, |signature| signature.id);
        let child = context.for_source(
            source_owner.unwrap_or(owner),
            request.file,
            request.location.source,
        );
        let result =
            super::using_discovery::evaluate(request, &child, declarations, types, places, meta);
        let procedures = child.pending.into_inner();
        let constants = child.pending_constants.into_inner();
        match result {
            Ok(decision) => jobs.using_decisions.push((request.id, decision)),
            Err(diagnostic) => jobs.using_pending.push(DiscoveryUsingPending {
                request: request.id,
                diagnostic,
                procedures,
                constants,
            }),
        }
    }
    Ok(jobs.decisions.len() + jobs.case_decisions.len() + jobs.using_decisions.len() - before)
}

fn evaluate_one(
    request: &DeferredCondition,
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<bool, LocatedDiagnostic> {
    let substitution = request
        .specialization
        .as_ref()
        .map(|key| super::source_specializations::decode(key, declarations, types, meta))
        .transpose()?;
    if let Some(key) = &request.specialization {
        meta.source_specialization_keys
            .insert(context.owner, key.clone());
    }
    evaluate_source(
        SourceRequest {
            file: request.file,
            purpose: SourceRequestPurpose::Condition,
            expression: &request.expression,
            lexical: Some(&request.context),
            assertion: None,
            substitution: substitution.as_ref(),
        },
        context,
        declarations,
        types,
        places,
        meta,
    )
}

fn evaluate_case_one(
    request: &jai_modules::DeferredCase,
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<syntax::CompileTimeCaseChoice, LocatedDiagnostic> {
    let substitution = request
        .specialization
        .as_ref()
        .map(|key| super::source_specializations::decode(key, declarations, types, meta))
        .transpose()?;
    if let Some(key) = &request.specialization {
        meta.source_specialization_keys
            .insert(context.owner, key.clone());
    }
    with_source_resolver(
        SourceRequest {
            file: request.file,
            purpose: SourceRequestPurpose::Condition,
            expression: &request.header.value,
            lexical: Some(&request.context),
            assertion: None,
            substitution: substitution.as_ref(),
        },
        context,
        declarations,
        types,
        places,
        meta,
        |resolver| resolver.compile_time_case_selection(&request.header),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceRequestPurpose {
    Condition,
    UsingTarget,
}
pub(super) struct SourceRequest<'a> {
    pub file: FileInstanceId,
    pub purpose: SourceRequestPurpose,
    pub expression: &'a syntax::Expression,
    pub lexical: Option<&'a DiscoveryConditionContext>,
    pub assertion: Option<Assertion<'a>>,
    pub substitution: Option<&'a crate::polymorphism::Substitution>,
}

pub(super) struct Assertion<'a> {
    pub message: Option<&'a syntax::Expression>,
    pub span: Span,
}

pub(super) fn evaluate_source(
    mut source: SourceRequest<'_>,
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<bool, LocatedDiagnostic> {
    let expression = source.expression;
    let assertion = source.assertion.take();
    with_source_resolver(
        source,
        context,
        declarations,
        types,
        places,
        meta,
        |resolver| match assertion {
            None => resolver.compile_time_condition(expression),
            Some(assertion) => resolver
                .compile_time_assertion(expression, assertion.message, assertion.span)
                .map(|()| true),
        },
    )
}

pub(super) fn with_source_resolver<T>(
    source: SourceRequest<'_>,
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    operation: impl FnOnce(&mut Resolver<'_>) -> Result<T, Diagnostic>,
) -> Result<T, LocatedDiagnostic> {
    let SourceRequest {
        file,
        purpose,
        expression,
        lexical,
        assertion: _,
        substitution,
    } = source;
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
            file,
            substitution,
        }),
        compile_time: Some(context),
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
    (|| {
        if let Some(DiscoveryConditionContext::Lexical { scopes, .. }) = lexical {
            for (depth, scope) in scopes.iter().enumerate() {
                resolver.scopes.push(HashMap::new());
                resolver.push_local_scope();
                if depth == 0 && let Some(substitution) = substitution {
                    for binding in &substitution.types { resolver.bind_name(binding.name, Binding::Type(binding.ty))?; }
                }
                for parameter in &scope.parameters {
                    if parameter.baking != syntax::ParameterBaking::None && substitution.and_then(|substitution| substitution.constant(parameter.name)).is_some() { continue; }
                    if parameter.baking != syntax::ParameterBaking::None && substitution.and_then(|substitution| substitution.ty(parameter.name)).is_some() { continue; }
                    let ty = match &parameter.binding {
                        syntax::ParameterBinding::Required(ty)
                        | syntax::ParameterBinding::Defaulted { ty: Some(ty), .. } => resolver.types.scalar(*ty),
                        syntax::ParameterBinding::RequiredType(ty)
                        | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => resolver.lexical_annotation(ty, parameter.span)?,
                        _ => return Err(Diagnostic::new(parameter.span,
                            "discovery condition requires the defining parameter's resolved type")),
                    };
                    resolver.declare_typed(parameter.name, ty)?;
                }
                resolver.register_local_record_declarations(&scope.record_members)?;
                resolver.register_local_declarations(&scope.statements)?;
                for &name in &scope.runtime_names {
                    resolver.declare_local_runtime_symbol(name);
                }
                if purpose == SourceRequestPurpose::UsingTarget
                    && scope.statements.is_empty() && scope.parameters.is_empty() && scope.record_members.is_empty()
                {
                    super::using_discovery::prepare_iteration_target(&mut resolver, &scopes[..depth], &scope.runtime_names, scopes.get(depth + 1).map(|scope| scope.statements.as_slice()), expression.span)?;
                }
                // Resolve earlier static source branches before looking up a
                // later guard. Their imports shadow file bindings only after
                // the chosen dependency has actually been discovered.
                let prefix: Vec<_> = scope
                    .statements
                    .iter()
                    .filter(|statement| statement.span.end <= expression.span.start)
                    .cloned()
                    .collect();
                let prefix = resolver.select_compile_time_statements(&prefix)?;
                for statement in &prefix {
                    if let syntax::StatementKind::Import(import) = &statement.kind {
                        resolver.bind_scoped_import(import)?;
                    } else if let syntax::StatementKind::Using(directive) = &statement.kind {
                        resolver.bind_checked_using_prefix(directive)?;
                    } else if let Some(directive) = statement.using_declaration_directive() {
                        if purpose == SourceRequestPurpose::UsingTarget {
                            resolver.using_declaration(statement)?;
                        } else {
                            resolver.bind_checked_using_prefix(&directive)?;
                        }
                    } else if purpose == SourceRequestPurpose::UsingTarget
                        && matches!(statement.kind, syntax::StatementKind::Declare(_) | syntax::StatementKind::DeclareResults { .. })
                    {
                        resolver.statement(statement)?;
                    }
                }
            }
        }
        operation(&mut resolver)
    })().map_err(|error| located(declarations.graph, file, error))
}
