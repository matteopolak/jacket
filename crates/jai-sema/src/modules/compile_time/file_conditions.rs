//! Validate graph scalar selections against the canonical typed source scope.
use super::*;

pub(super) struct Guard<'a> {
    file: FileInstanceId,
    expression: &'a syntax::Expression,
    selected: Option<bool>,
    message: Option<&'a syntax::Expression>,
    span: Span,
    case: Option<(syntax::CompileTimeCaseHeader, syntax::CompileTimeCaseChoice)>,
}

pub(super) fn collect(graph: &ModuleGraph) -> Vec<Guard<'_>> {
    let mut guards = vec![];
    for file in graph.files() {
        let mut pending: Vec<_> = file.syntax().items().iter().collect();
        while let Some(item) = pending.pop() {
            if let syntax::FileItem::Assert {
                condition,
                message,
                location,
            } = item
            {
                guards.push(Guard {
                    file: file.id(),
                    expression: condition,
                    selected: None,
                    message: message.as_ref(),
                    span: location.span,
                    case: None,
                });
            }
            if let syntax::FileItem::Conditional {
                condition,
                then_items,
                else_items,
                ..
            } = item
                && let Some(selected) = graph.selected_condition(file.id(), condition.span)
            {
                if graph
                    .selected_semantic_condition(file.id(), condition.span)
                    .is_none()
                {
                    guards.push(Guard {
                        file: file.id(),
                        expression: condition,
                        selected: Some(selected),
                        message: None,
                        span: condition.span,
                        case: None,
                    });
                }
                pending.extend(if selected { then_items } else { else_items });
            }
            if let syntax::FileItem::CompileTimeCases { cases, .. } = item
                && let Some(selection) = graph.source_cases().iter().find(|selection| {
                    selection.file == file.id()
                        && selection.location.span == cases.span
                        && selection.specialization.is_none()
                })
            {
                if selection.origin == jai_modules::SourceConditionOrigin::Scalar {
                    guards.push(Guard {
                        file: file.id(),
                        expression: &cases.value,
                        selected: None,
                        message: None,
                        span: cases.span,
                        case: Some((cases.header(), selection.choice)),
                    });
                }
                if let Some(body) = cases.selected_body_refs(selection.choice) {
                    pending.extend(body);
                }
            }
        }
    }
    guards.sort_by_key(|guard| (guard.file.index(), guard.span.start));
    guards
}

pub(super) struct Progress {
    pub completed: usize,
    pub error: Option<LocatedDiagnostic>,
}

pub(super) fn bind(
    context: &Context<'_>,
    guards: &mut Vec<Guard<'_>>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Progress {
    let mut retry = vec![];
    let mut completed = 0;
    let mut error = None;
    for guard in guards.drain(..) {
        let source = declarations.graph.file(guard.file).unwrap().source();
        let child = context.for_source(context.owner, guard.file, source);
        let result = if let Some((header, expected)) = &guard.case {
            super::super::discovery_conditions::with_source_resolver(
                super::super::discovery_conditions::SourceRequest {
                    file: guard.file,
                    purpose: super::super::discovery_conditions::SourceRequestPurpose::Condition,
                    expression: guard.expression,
                    lexical: None,
                    substitution: None,
                    assertion: None,
                },
                &child,
                declarations,
                types,
                places,
                meta,
                |resolver| resolver.compile_time_case_selection(header),
            )
            .and_then(|choice| {
                if choice == *expected {
                    Ok(())
                } else {
                    Err(located(
                        declarations.graph,
                        guard.file,
                        Diagnostic::new(
                            guard.span,
                            "typed source case disagrees with the dependency graph selection",
                        ),
                    ))
                }
            })
        } else {
            super::super::discovery_conditions::evaluate_source(
                super::super::discovery_conditions::SourceRequest {
                    file: guard.file,
                    purpose: super::super::discovery_conditions::SourceRequestPurpose::Condition,
                    expression: guard.expression,
                    lexical: None,
                    substitution: None,
                    assertion: guard.selected.is_none().then_some(
                        super::super::discovery_conditions::Assertion {
                            message: guard.message,
                            span: guard.span,
                        },
                    ),
                },
                &child,
                declarations,
                types,
                places,
                meta,
            )
            .and_then(|selected| {
                if guard.selected.is_none_or(|expected| selected == expected) {
                    Ok(())
                } else {
                    Err(located(
                        declarations.graph,
                        guard.file,
                        Diagnostic::new(
                            guard.expression.span,
                            "typed #if condition disagrees with the dependency graph selection",
                        ),
                    ))
                }
            })
        };
        match result {
            Ok(()) => completed += 1,
            Err(diagnostic) => {
                error.get_or_insert(diagnostic);
                retry.push(guard);
            }
        }
    }
    *guards = retry;
    Progress { completed, error }
}
