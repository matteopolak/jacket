use super::*;

pub(super) fn bind_one(
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    file: FileInstanceId,
    constant: &syntax::ConstantDeclaration,
    options: &crate::ResolveOptions,
) -> Result<Binding, Diagnostic> {
    let expected = constant
        .ty
        .as_ref()
        .map(|annotation| {
            FileScope {
                declarations,
                file,
                substitution: None,
            }
            .annotation_with_specializations(
                annotation,
                types,
                &mut meta.record_specializations,
                constant.span,
            )
        })
        .transpose()?;
    let value = evaluate_expression(
        declarations,
        types,
        meta,
        file,
        &constant.initializer,
        expected,
        options,
    )?;
    crate::compile_time::materialized_binding(value, None, constant.span, meta)
}

pub(crate) fn is_pure_scalar(expression: &syntax::Expression) -> bool {
    unsupported_scalar(expression).is_none()
}

fn unsupported_scalar(expression: &syntax::Expression) -> Option<Span> {
    let mut unsupported = None;
    deferred_constants::visit(expression, |expression| {
        use syntax::ExpressionKind as E;
        if !matches!(
            expression.kind,
            E::Integer(_)
                | E::Float(_)
                | E::Bool(_)
                | E::Name(_)
                | E::QualifiedName(_)
                | E::InferredMember(_)
                | E::Unary(_, _)
                | E::Binary(_, _, _)
                | E::Cast(_, _, _)
                | E::TypeCast { .. }
                | E::Conditional(_)
        ) {
            unsupported = Some(expression.span);
        }
    });
    unsupported
}

pub(crate) fn uses_enum(
    declarations: &ScopedDeclarations<'_>,
    file: FileInstanceId,
    expression: &syntax::Expression,
) -> bool {
    let graph = declarations.graph;
    let mut found = false;
    deferred_constants::visit(expression, |expression| {
        let path = match &expression.kind {
            syntax::ExpressionKind::Name(name) => path(*name),
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return,
        };
        if captured_enum(graph, file, &path)
            || super::super::target_values::is_target_path(graph, file, &path)
            || matches!(graph.lookup(file, &path), Ok(GraphBinding::Parameter(id)) if matches!(graph.parameter(id).map(|parameter| &parameter.value), Some(jai_modules::ParameterValue::Enumeration(_))))
            || declarations
                .nominals
                .enum_member(graph, file, &path, expression.span)
                .ok()
                .flatten()
                .is_some()
            || referenced(graph, file, &path)
                .is_some_and(|id| matches!(declarations.values.get(&id), Some(Binding::Enum(_))))
        {
            found = true;
        }
    });
    found
}

pub(crate) fn evaluate_expression(
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    file: FileInstanceId,
    expression: &syntax::Expression,
    expected: Option<TypeId>,
    options: &crate::ResolveOptions,
) -> Result<jai_ir::ConstantValue, Diagnostic> {
    if let Some(span) = unsupported_scalar(expression) {
        return Err(Diagnostic::new(
            span,
            "nominal constant requires a pure scalar expression; procedure execution requires #run",
        ));
    }
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut places = jai_ir::PlaceRegistry::default();
    let (value, ty) = {
        let mut resolver = Resolver {
            expression_owner: None,
            debug: crate::debug_capture::Capture::default(),
            checks: crate::safety_checks::ActiveChecks::default(),
            local_scopes: crate::local_declarations::LocalScopes::default(),
            context: None,
            context_available: false,
            meta,
            procedure: ProcedureId::new(0),
            types,
            target_layout: options.effective_layout(),
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
        let value = match expected {
            Some(ty) => resolver.expr_expected(expression, ty)?,
            None => resolver.expr(expression)?,
        };
        let ty = match expected {
            Some(ty) => ty,
            None => resolver.expression_type(&value, expression.span)?,
        };
        (resolver.coerce_value(value, ty, expression.span)?, ty)
    };
    let places = places.freeze();
    let procedures = HashMap::new();
    let provider_signatures = HashMap::new();
    let provider = crate::compile_time::ReadyProcedures::new(
        types,
        &procedures,
        &provider_signatures,
        &[],
        &places,
    )
    .map_err(|error| Diagnostic::new(expression.span, error.to_string()))?;
    let location = SourceSpan {
        source: declarations.graph.file(file).unwrap().source(),
        span: expression.span,
    };
    let target = options
        .target
        .as_ref()
        .map(jai_vm::ByteTarget::from)
        .or_else(|| {
            options.effective_layout().map(|policy| jai_vm::ByteTarget {
                policy,
                endian: jai_vm::Endian::Little,
            })
        });
    match crate::compile_time::evaluate_for_target(
        &provider,
        jai_vm::NoEffects,
        &value,
        ty,
        location,
        options.compile_time_limits,
        crate::compile_time::EvaluationScope {
            owner: None,
            target,
        },
    ) {
        crate::compile_time::RunOutcome::Complete(Some(value)) => Ok(value),
        crate::compile_time::RunOutcome::Failed(error) => {
            Err(Diagnostic::at_source(error.location, error.message))
        }
        _ => Err(Diagnostic::new(
            expression.span,
            "pure nominal constant did not produce a complete value",
        )),
    }
}
