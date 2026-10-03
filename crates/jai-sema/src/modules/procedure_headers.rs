//! Separate callable type reservation from source default materialization.
use super::*;
mod dependencies;
pub(super) use dependencies::{dependencies as source_dependencies, ordered as ordered_sources};
mod compiler_code;
mod context_readiness;
pub(super) use context_readiness::{ContextHeaderPrerequisite, HeaderPreparation};
mod discarded;
pub(super) mod identities;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum HeaderPhase {
    TypesOnly,
    Complete,
}

pub(super) fn register<'a>(
    graph: &'a ModuleGraph,
    types: &mut TypeRegistry,
    declarations: &mut ScopedDeclarations<'a>,
    constants: &mut Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
    phase: HeaderPhase,
) -> Result<(), LocatedDiagnostic> {
    let sources = dependencies::ordered(graph, &declarations.callable_aliases)?;
    register_sources(graph, sources, types, declarations, constants, meta, phase)
}

pub(super) fn register_selected<'a>(
    graph: &'a ModuleGraph,
    source: &'a jai_modules::Declaration,
    types: &mut TypeRegistry,
    declarations: &mut ScopedDeclarations<'a>,
    constants: &mut Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<HeaderPreparation, LocatedDiagnostic> {
    if let Some(request) = context_readiness::prerequisite(graph, source, declarations)? {
        return Ok(HeaderPreparation::Pending(request));
    }
    register_sources(
        graph,
        [source],
        types,
        declarations,
        constants,
        meta,
        HeaderPhase::TypesOnly,
    )?;
    Ok(HeaderPreparation::Ready)
}

/// Retain only genuine Context prerequisites while other source headers advance.
pub(super) fn register_before_context<'graph>(
    graph: &'graph ModuleGraph,
    types: &mut TypeRegistry,
    declarations: &mut ScopedDeclarations<'graph>,
    constants: &mut Constants<'graph>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<Vec<ContextHeaderPrerequisite>, LocatedDiagnostic> {
    let mut pending = Vec::new();
    for source in dependencies::ordered(graph, &declarations.callable_aliases)? {
        if let Some(request) = context_readiness::prerequisite(graph, source, declarations)? {
            pending.push(request);
            continue;
        }
        if identities::is_concrete(source)
            && source_dependencies(graph, &declarations.callable_aliases, source)
                .iter()
                .any(|dependency| !declarations.signatures.contains_key(dependency))
        {
            continue;
        }
        register_sources(
            graph,
            [source],
            types,
            declarations,
            constants,
            meta,
            HeaderPhase::TypesOnly,
        )?;
    }
    Ok(pending)
}

pub(super) use context_readiness::complete as complete_context_headers;

fn register_sources<'a>(
    graph: &'a ModuleGraph,
    sources: impl IntoIterator<Item = &'a jai_modules::Declaration>,
    types: &mut TypeRegistry,
    declarations: &mut ScopedDeclarations<'a>,
    constants: &mut Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
    phase: HeaderPhase,
) -> Result<(), LocatedDiagnostic> {
    for declaration in sources {
        if meta.compiler_code.template(declaration.id()).is_some() {
            continue;
        }
        if let Some(template) =
            compiler_code::template(graph, declaration, declarations, types, constants, meta)?
        {
            meta.compiler_code
                .register(template)
                .map_err(|error| located(graph, declaration.file(), error))?;
            continue;
        }
        if phase == HeaderPhase::TypesOnly
            && declarations.signatures.contains_key(&declaration.id())
        {
            continue;
        }

        if matches!(&declaration.syntax().kind, FileDeclarationKind::Procedure(procedure) if procedure.expands)
        {
            continue;
        }
        if phase == HeaderPhase::TypesOnly {
            match &declaration.syntax().kind {
                FileDeclarationKind::Procedure(procedure)
                    if crate::polymorphism::is_polymorphic(procedure) =>
                {
                    continue;
                }
                FileDeclarationKind::ProcedurePrototype(prototype)
                    if crate::polymorphism::is_polymorphic_prototype(prototype) =>
                {
                    continue;
                }
                _ => {}
            }
        }
        if let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.syntax().kind
            && polymorphic_headers::register_prototype(
                graph,
                declaration.id(),
                declaration.file(),
                prototype,
                types,
                declarations,
                constants,
                meta,
            )?
        {
            continue;
        }
        if let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind
            && crate::polymorphism::is_polymorphic(procedure)
        {
            let file = declaration.file();
            let mut defaults = HashMap::new();
            for (expression, expected) in
                polymorphic_defaults::sources(&procedure.parameters, &procedure.results)
            {
                let info = polymorphic_defaults::prepare(
                    graph,
                    file,
                    expression,
                    expected.as_ref(),
                    types,
                    declarations,
                    constants,
                    meta,
                )?;
                defaults.insert((expression.span.start, expression.span.end), info);
            }
            let nominal_patterns = aggregates::parameterized::procedure_patterns(
                graph,
                file,
                procedure,
                types,
                &declarations.nominals,
                &mut meta.record_specializations,
                &mut |file, expression| constants.evaluate(file, expression),
            )?;
            let scope = FileScope {
                declarations,
                file,
                substitution: None,
            };
            let template = crate::polymorphism::from_procedure_with_patterns(
                declaration.id(),
                procedure,
                |syntax, span| {
                    scope.annotation_with_specializations(
                        syntax,
                        types,
                        &mut meta.record_specializations,
                        span,
                    )
                },
                |expression| Ok(defaults[&(expression.span.start, expression.span.end)].clone()),
                |expression| {
                    let value = constants
                        .evaluate(file, expression)
                        .map_err(|error| Diagnostic::at_source(error.location, error.message))?;
                    let integer = match value {
                        ConstantValue::Literal(value) => value,
                        ConstantValue::Int(value) => value.value(),
                        _ => {
                            return Err(Diagnostic::new(
                                expression.span,
                                "array count requires an integer constant",
                            ));
                        }
                    };
                    u64::try_from(integer).map_err(|_| {
                        Diagnostic::new(expression.span, "array count is out of range")
                    })
                },
                &nominal_patterns,
            )
            .map_err(|error| located(graph, file, error))?;
            declarations.generics.borrow_mut().register(
                crate::polymorphism::integration::TemplateDefinition {
                    template,
                    file,
                    span: procedure.span,
                    convention: procedure.convention,
                    context: procedure.context,
                },
            );
            continue;
        }
        let (source_parameters, source_results, convention, context, span) =
            match &declaration.syntax().kind {
                FileDeclarationKind::Procedure(procedure) => (
                    &procedure.parameters,
                    &procedure.results,
                    procedure.convention,
                    procedure.context,
                    procedure.span,
                ),
                FileDeclarationKind::ProcedurePrototype(prototype) => (
                    &prototype.parameters,
                    &prototype.results,
                    prototype.convention,
                    prototype.context,
                    prototype.span,
                ),
                _ => continue,
            };
        let file = declaration.file();
        let mut parameters = Vec::new();
        let mut names = std::collections::HashSet::new();
        for (parameter_index, parameter) in source_parameters.iter().enumerate() {
            if parameter.variadic && convention == CallingConvention::C {
                continue;
            }
            if !names.insert(parameter.name) {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(parameter.span, "duplicate parameter name"),
                ));
            }
            if parameter.baking != syntax::ParameterBaking::None {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        parameter.span,
                        "baked parameters require specialization support",
                    ),
                ));
            }
            let (ty, default) = match &parameter.binding {
                syntax::ParameterBinding::Required(ty) => (types.scalar(*ty), None),
                syntax::ParameterBinding::RequiredType(ty) => (
                    declarations.nominals.resolve_type_with_specializations(
                        graph,
                        aggregates::parameterized::TypeRequest::new(file, ty, parameter.span),
                        types,
                        &mut meta.record_specializations,
                        &mut |file, expression| constants.evaluate_lazy(file, expression),
                    )?,
                    None,
                ),
                syntax::ParameterBinding::Defaulted {
                    ty,
                    expression,
                } => (
                    match ty {
                        Some(ty) => types.scalar(*ty),
                        None if parameter.evaluation == syntax::ParameterEvaluation::Discard => {
                            procedure_signatures::infer_discarded_default(
                                graph,
                                file,
                                expression,
                                types,
                                &declarations.nominals,
                                constants,
                            )?
                        }
                        None => runtime_defaults::infer(
                            graph,
                            file,
                            expression,
                            types,
                            declarations,
                            constants,
                            meta,
                        )?
                        .map(Ok)
                        .unwrap_or_else(|| {
                            procedure_signatures::infer_default(
                                graph,
                                file,
                                expression,
                                types,
                                &declarations.nominals,
                                constants,
                            )
                        })?,
                    },
                    Some(expression),
                ),
                syntax::ParameterBinding::DefaultedType {
                    ty,
                    expression,
                } => {
                    let ty = match ty {
                        Some(ty) => declarations.nominals.resolve_type_with_specializations(
                            graph,
                            aggregates::parameterized::TypeRequest::new(file, ty, parameter.span),
                            types,
                            &mut meta.record_specializations,
                            &mut |file, expression| constants.evaluate_lazy(file, expression),
                        )?,
                        None if parameter.evaluation == syntax::ParameterEvaluation::Discard => {
                            procedure_signatures::infer_discarded_default(
                                graph,
                                file,
                                expression,
                                types,
                                &declarations.nominals,
                                constants,
                            )?
                        }
                        None => runtime_defaults::infer(
                            graph,
                            file,
                            expression,
                            types,
                            declarations,
                            constants,
                            meta,
                        )?
                        .map(Ok)
                        .unwrap_or_else(|| {
                            procedure_signatures::infer_default(
                                graph,
                                file,
                                expression,
                                types,
                                &declarations.nominals,
                                constants,
                            )
                        })?,
                    };
                    (ty, Some(expression))
                }
            };
            let retained_default = declarations
                .signatures
                .get(&declaration.id())
                .and_then(|signature| {
                    signature
                        .parameters
                        .iter()
                        .find(|checked| checked.name == parameter.name)
                })
                .and_then(|checked| match &checked.default {
                    Some(ParameterDefault::Source(source)) => Some(std::rc::Rc::clone(source)),
                    _ => None,
                });
            if retained_default
                .as_ref()
                .is_some_and(|source| source.key.expected != ty)
            {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        parameter.span,
                        "source default changed its checked formal type",
                    ),
                ));
            }
            let default = match default {
                Some(_)
                    if retained_default
                        .as_ref()
                        .is_some_and(|source| source.ready().is_some()) =>
                {
                    Some(ParameterDefault::Source(retained_default.unwrap()))
                }
                Some(expression)
                    if parameter.evaluation == syntax::ParameterEvaluation::Discard =>
                {
                    if phase == HeaderPhase::Complete {
                        let owner = declarations
                            .signatures
                            .get(&declaration.id())
                            .ok_or_else(|| {
                                located(
                                    graph,
                                    file,
                                    Diagnostic::new(
                                        parameter.span,
                                        "discarded default is pending its checked procedure header",
                                    ),
                                )
                            })?
                            .id;
                        discarded::check(
                            declarations,
                            types,
                            meta,
                            discarded::DefaultCheck {
                                file,
                                owner,
                                name: parameter.name,
                                expression,
                                expected: ty,
                            },
                        )?;
                    }
                    Some(ParameterDefault::Discarded)
                }
                Some(expression)
                    if matches!(
                        expression.kind,
                        syntax::ExpressionKind::Code(syntax::CodeBody::Null)
                    ) =>
                {
                    if ty != types.code_type() {
                        return Err(located(
                            graph,
                            file,
                            Diagnostic::new(
                                expression.span,
                                "#code,null requires a compile-time Code parameter",
                            ),
                        ));
                    }
                    Some(ParameterDefault::CodeNull {
                        ty,
                    })
                }
                Some(expression)
                    if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) =>
                {
                    if phase == HeaderPhase::Complete {
                        crate::caller_locations::validate_target(
                            graph,
                            &declarations.nominals,
                            types,
                            ty,
                            expression.span,
                        )
                        .map_err(|error| located(graph, file, error))?;
                    }
                    Some(ParameterDefault::CallerLocation)
                }
                Some(expression) if phase == HeaderPhase::TypesOnly => {
                    let procedure = declarations
                        .source_procedures
                        .get(declaration.id())
                        .ok_or_else(|| {
                            located(
                                graph,
                                file,
                                Diagnostic::new(
                                    parameter.span,
                                    "source default requires its reserved concrete procedure",
                                ),
                            )
                        })?;
                    let source = crate::source_parameter_defaults::SourceParameterDefault::new(
                        crate::source_parameter_defaults::SourceParameterDefaultKey {
                            declaration: declaration.id(),
                            file,
                            parameter: parameter_index,
                            expected: ty,
                            procedure,
                            location: SourceSpan {
                                source: declaration.location().source,
                                span: expression.span,
                            },
                        },
                        expression.clone(),
                        graph
                            .sources()
                            .get(declaration.location().source)
                            .expect("original source default")
                            .shared_text(),
                    );
                    Some(ParameterDefault::Source(source))
                }
                Some(expression) => {
                    let value = if let Some(read) = runtime_defaults::prepare(
                        graph,
                        file,
                        expression,
                        Some(ty),
                        types,
                        declarations,
                        constants,
                        meta,
                    )? {
                        ParameterDefault::RuntimeRead(read)
                    } else {
                        let mut evaluator = aggregates::Defaults::new(
                            graph,
                            types,
                            &declarations.nominals,
                            constants,
                        )
                        .with_specializations(&meta.record_specializations)
                        .with_context(declarations.context.as_ref());
                        evaluator.fields = declarations.defaults.clone();
                        hydrate_constants(&mut evaluator, declarations, meta);
                        ParameterDefault::Constant(evaluator.expression(file, expression, ty)?)
                    };
                    if let Some(source) = retained_default {
                        source
                            .publish(value)
                            .map_err(|error| located(graph, file, error))?;
                        Some(ParameterDefault::Source(source))
                    } else {
                        Some(value)
                    }
                }
                None => None,
            };
            parameters.push(ParameterSignature {
                name: parameter.name,
                ty,
                default,
                evaluation: parameter.evaluation,
            });
        }
        let mut results = Vec::new();
        let mut names = std::collections::HashSet::new();
        for result in source_results {
            if result.name.is_some_and(|name| !names.insert(name)) {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(result.span, "duplicate result name"),
                ));
            }
            let (ty, default) = match &result.binding {
                syntax::ResultBinding::Typed {
                    ty,
                    default,
                } => (
                    declarations.nominals.resolve_type_with_specializations(
                        graph,
                        aggregates::parameterized::TypeRequest::new(file, ty, result.span),
                        types,
                        &mut meta.record_specializations,
                        &mut |file, expression| constants.evaluate_lazy(file, expression),
                    )?,
                    default.as_ref(),
                ),
                syntax::ResultBinding::InferredDefault(expression) => (
                    procedure_signatures::infer_default(
                        graph,
                        file,
                        expression,
                        types,
                        &declarations.nominals,
                        constants,
                    )?,
                    Some(expression),
                ),
            };
            let default = match default {
                Some(_) if phase == HeaderPhase::TypesOnly => None,
                Some(expression) => {
                    let mut evaluator =
                        aggregates::Defaults::new(graph, types, &declarations.nominals, constants)
                            .with_specializations(&meta.record_specializations)
                            .with_context(declarations.context.as_ref());
                    evaluator.fields = declarations.defaults.clone();
                    hydrate_constants(&mut evaluator, declarations, meta);
                    Some(evaluator.expression(file, expression, ty)?)
                }
                None => None,
            };
            results.push(ResultSignature {
                usage: result.usage,
                name: result.name,
                ty,
                default,
            });
        }
        crate::procedure_values::signatures::normalize_results(&mut results, types, |result| {
            result.ty
        });
        let compiler_source_signature = compiler_intrinsics::lower_code_signature(
            &declaration.syntax().kind,
            &mut parameters,
            graph.symbols(),
            types,
            span,
        )
        .map_err(|error| located(graph, file, error))?;
        let variadic = crate::procedure_values::signatures::normalize_variadic(
            source_parameters,
            &mut parameters,
            convention,
            types,
            span,
        )
        .map_err(|error| located(graph, file, error))?;
        let ty = types
            .procedure(ProcedureType {
                parameters: parameters
                    .iter()
                    .filter(|parameter| {
                        parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                    })
                    .map(|parameter| parameter.ty)
                    .collect(),
                results: results.iter().map(|result| result.ty).collect(),
                convention,
                context,
                variadic,
            })
            .map_err(|error| located(graph, file, Diagnostic::new(span, error.to_string())))?;
        let id = declarations
            .source_procedures
            .get(declaration.id())
            .ok_or_else(|| {
                located(
                    graph,
                    file,
                    Diagnostic::new(span, "concrete source procedure has no reserved identity"),
                )
            })?;
        let deprecation = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => source.deprecation.as_ref(),
            FileDeclarationKind::ProcedurePrototype(source) => source.deprecation.as_ref(),
            _ => None,
        };
        meta.remember_deprecation(
            crate::deprecation_warnings::DeprecationKey::Procedure(id),
            graph
                .sources()
                .get(graph.file(file).expect("retained file").source())
                .expect("retained source"),
            graph.symbols().name(declaration.name()),
            deprecation,
            declaration.location().span,
        )
        .map_err(|error| located(graph, file, error))?;
        if let Some(metadata) = compiler_source_signature {
            meta.compiler_source_signatures.insert(id, metadata);
        }
        declarations
            .nominals
            .value_types
            .insert(declaration.id(), ty);
        declarations.signatures.insert(
            declaration.id(),
            Signature {
                id,
                source_variadic: crate::procedure_values::signatures::source_variadic(
                    source_parameters,
                    &parameters,
                    convention,
                ),
                ty,
                parameters,
                results,
            },
        );
        if phase == HeaderPhase::Complete
            && let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind
            && let Some(operator) = procedure.operator
        {
            crate::operator_overloads::validate_signature(
                types,
                operator.kind,
                &declarations.signatures[&declaration.id()],
                span,
            )
            .map_err(|error| located(graph, file, error))?;
        }
        publish_callable_values(declarations);
    }
    Ok(())
}

fn publish_callable_values(declarations: &mut ScopedDeclarations<'_>) {
    for (&declaration, signature) in &declarations.signatures {
        declarations.nominals.value_constants.insert(
            declaration,
            jai_ir::ConstantValue {
                ty: signature.ty,
                kind: jai_ir::ConstantKind::Procedure(signature.id),
            },
        );
    }
    for (&alias, targets) in &declarations.callable_aliases {
        let [target] = targets.as_slice() else {
            continue;
        };
        let Some(signature) = declarations.signatures.get(target) else {
            continue;
        };
        declarations
            .nominals
            .value_types
            .insert(alias, signature.ty);
        declarations.nominals.value_constants.insert(
            alias,
            jai_ir::ConstantValue {
                ty: signature.ty,
                kind: jai_ir::ConstantKind::Procedure(signature.id),
            },
        );
    }
}
