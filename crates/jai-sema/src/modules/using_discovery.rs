//! Evaluate retained using requests inside the shared typed preparation.
use super::*;
use crate::compile_time::Context;

pub(super) fn prepare_iteration_target(
    resolver: &mut Resolver<'_>,
    outer: &[jai_modules::DiscoveryLexicalScope],
    names: &[Symbol],
    body: Option<&[syntax::Statement]>,
    target: Span,
) -> Result<(), Diagnostic> {
    let Some(body) = body else {
        return Ok(());
    };
    let Some(loop_) = outer
        .iter()
        .rev()
        .find_map(|scope| enclosing_sequence(&scope.statements, names, body, target))
    else {
        return Ok(());
    };
    let source = resolver.expr(&loop_.sequence)?;
    let ty = resolver.expression_type(&source, loop_.sequence.span)?;
    let element = match resolver
        .types
        .kind(ty)
        .map_err(|error| Diagnostic::new(target, error.to_string()))?
    {
        jai_types::TypeKind::FixedArray {
            element, ..
        }
        | jai_types::TypeKind::Slice(element)
        | jai_types::TypeKind::DynamicArray(element)
            if loop_.expansion.is_none() =>
        {
            *element
        }
        jai_types::TypeKind::String if loop_.expansion.is_none() => resolver
            .types
            .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
        _ => {
            return Err(Diagnostic::new(
                target,
                "using iterator target requires its checked custom iteration protocol",
            ));
        }
    };
    let by_pointer = match &loop_.pointer_control {
        Some(control) => resolver.iteration_modifier(control)?,
        None => loop_.by_pointer,
    };
    let iterator_ty = if by_pointer {
        resolver
            .types
            .pointer(element)
            .map_err(|error| Diagnostic::new(target, error.to_string()))?
    } else {
        element
    };
    resolver.declare_typed(loop_.iterator, iterator_ty)?;
    if let Some(index) = loop_.index {
        let ty = resolver
            .types
            .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64));
        resolver.declare_typed(index, ty)?;
    }
    Ok(())
}

fn enclosing_sequence<'a>(
    statements: &'a [syntax::Statement],
    names: &[Symbol],
    retained_body: &[syntax::Statement],
    target: Span,
) -> Option<&'a syntax::ArrayLoop> {
    for statement in statements {
        if statement.span.start > target.start || statement.span.end < target.end {
            continue;
        }
        let found = match &statement.kind {
            syntax::StatementKind::ArrayLoop(loop_) => {
                enclosing_sequence(&loop_.body, names, retained_body, target).or_else(|| {
                    (names.contains(&loop_.iterator)
                        && same_source_body(&loop_.body, retained_body))
                    .then_some(loop_)
                })
            }
            syntax::StatementKind::Block(body)
            | syntax::StatementKind::Defer(body)
            | syntax::StatementKind::While(_, body)
            | syntax::StatementKind::CheckScope {
                body, ..
            }
            | syntax::StatementKind::PushContext {
                body, ..
            } => enclosing_sequence(body, names, retained_body, target),
            syntax::StatementKind::If(_, yes, no)
            | syntax::StatementKind::CompileTimeIf {
                then_body: yes,
                else_body: no,
                ..
            } => enclosing_sequence(yes, names, retained_body, target)
                .or_else(|| enclosing_sequence(no, names, retained_body, target)),
            syntax::StatementKind::Range(range) => {
                enclosing_sequence(&range.body, names, retained_body, target)
            }
            syntax::StatementKind::Procedure(procedure) => {
                enclosing_sequence(&procedure.body, names, retained_body, target)
            }
            syntax::StatementKind::CallerExport(inner) => {
                enclosing_sequence(std::slice::from_ref(inner), names, retained_body, target)
            }
            syntax::StatementKind::Cases(cases) => cases
                .arms
                .iter()
                .find_map(|(_, body, _)| enclosing_sequence(body, names, retained_body, target))
                .or_else(|| {
                    cases
                        .default
                        .as_ref()
                        .and_then(|body| enclosing_sequence(body, names, retained_body, target))
                }),
            syntax::StatementKind::CompileTimeCases(cases) => cases
                .arms
                .iter()
                .find_map(|arm| enclosing_sequence(&arm.body, names, retained_body, target))
                .or_else(|| {
                    cases
                        .default
                        .as_ref()
                        .and_then(|arm| enclosing_sequence(&arm.body, names, retained_body, target))
                }),
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}
fn same_source_body(original: &[syntax::Statement], retained: &[syntax::Statement]) -> bool {
    original.len() == retained.len()
        && original.iter().zip(retained).all(|(a, b)| {
            a.span == b.span && std::mem::discriminant(&a.kind) == std::mem::discriminant(&b.kind)
        })
}

pub(super) fn evaluate(
    request: &jai_modules::FileUsingRequest,
    context: &Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
) -> Result<jai_modules::FileUsingDecision, LocatedDiagnostic> {
    let substitution = request
        .specialization
        .as_ref()
        .map(|key| source_specializations::decode(key, declarations, types, meta))
        .transpose()?;
    if let Some(key) = &request.specialization {
        meta.source_specialization_keys
            .insert(context.owner, key.clone());
    }
    discovery_conditions::with_source_resolver(
        discovery_conditions::SourceRequest {
            file: request.file,
            purpose: discovery_conditions::SourceRequestPurpose::UsingTarget,
            expression: &request.directive.target,
            lexical: Some(&request.context),
            assertion: None,
            substitution: substitution.as_ref(),
        },
        context,
        declarations,
        types,
        places,
        meta,
        |resolver| {
            if let Some(jai_modules::UsingDeclarationSource::File(declaration)) =
                &request.declaration
                && resolver.symbols.name(declaration.declared_name()) == "_"
            {
                let scope = resolver.graph_scope.ok_or_else(|| {
                    Diagnostic::new(
                        request.location.span,
                        "discarded using requires its source graph",
                    )
                })?;
                let owner = scope
                    .declarations
                    .graph
                    .declaration_at(request.file, declaration.location)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            request.location.span,
                            "discarded using original child declaration is unavailable",
                        )
                    })?;
                return resolver.using_source_declaration_decision(&request.directive, owner.id());
            }
            if let Some(jai_modules::UsingDeclarationSource::Statement(declaration)) =
                &request.declaration
            {
                if matches!(declaration.kind, jai_syntax::StatementKind::Declare(_)) {
                    // Discovery checks the genuine target plan; execution remains in the body.
                    resolver.statement(declaration)?;
                } else if let Some(name) = declaration.declared_name() {
                    resolver.resolve_local_name(name, request.directive.target.span)?;
                }
            }
            resolver.using_source_decision(&request.directive, request.owner.is_some())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{DiscoveryConditionContext, Filesystem, GraphDiscovery, GraphOptions};
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "jai-using-staged-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("modules")).unwrap();
            fs::write(root.join("main.jai"), "Lib :: #import \"Lib\"; names :: () -> []string { return .[\"value\"]; } mapper :: (names: []string) { names[0] = \"\"; names[1] = \"renamed\"; } main :: () {}").unwrap();
            fs::write(root.join("modules/Lib.jai"), "value :: 42; omitted :: 5;").unwrap();
            Self(root)
        }
        fn discovery(&self) -> GraphDiscovery<'static> {
            let mut discovery = GraphDiscovery::new(
                &self.0.join("main.jai"),
                GraphOptions {
                    import_dirs: vec![self.0.join("modules")],
                },
                &Filesystem,
            )
            .unwrap();
            assert!(discovery.advance().unwrap().is_complete());
            discovery
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn request(
        discovery: &mut GraphDiscovery<'_>,
        selection: syntax::UsingSelection,
    ) -> jai_modules::UsingRequestId {
        let graph = discovery.graph();
        let file = graph.module(graph.root()).unwrap().entry();
        let name = graph.symbols().find("Lib").unwrap();
        let span = Span::new(0, 3);
        discovery
            .retain_using(
                file,
                syntax::UsingDirective {
                    target: syntax::Expression {
                        kind: syntax::ExpressionKind::Name(name),
                        span,
                    },
                    selection,
                    span,
                },
                syntax::Visibility::Export,
                DiscoveryConditionContext::File,
            )
            .unwrap()
    }
    fn decision(discovery: &GraphDiscovery<'_>) -> jai_modules::FileUsingDecision {
        let requests = discovery.pending_using_requests();
        let outcome = resolve_discovery_using(
            discovery.graph(),
            &requests,
            &crate::ResolveOptions::default(),
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        assert!(outcome.pending.is_empty(), "{:?}", outcome.pending);
        assert_eq!(outcome.decisions.len(), 1);
        outcome.decisions.into_iter().next().unwrap().1
    }
    #[test]
    fn staged_non_string_selector_is_rejected_without_publication() {
        let fixture = Fixture::new();
        let mut discovery = fixture.discovery();
        request(
            &mut discovery,
            syntax::UsingSelection::Only(syntax::UsingNames::Expression(Box::new(
                syntax::Expression {
                    kind: syntax::ExpressionKind::Integer(42),
                    span: Span::new(0, 3),
                },
            ))),
        );
        let requests = discovery.pending_using_requests();
        let outcome = resolve_discovery_using(
            discovery.graph(),
            &requests,
            &crate::ResolveOptions::default(),
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        assert!(outcome.decisions.is_empty());
        assert_eq!(outcome.pending.len(), 1);
        assert!(
            outcome.pending[0]
                .diagnostic
                .message
                .contains("using selector requires an array of strings")
        );
        assert!(discovery.graph().using_publications().is_empty());
    }
    #[test]
    fn staged_qualified_namespace_is_resolved_once() {
        let fixture = Fixture::new();
        fs::write(
            fixture.0.join("modules/Lib.jai"),
            "Nested :: #import,file \"nested.jai\";",
        )
        .unwrap();
        fs::write(fixture.0.join("modules/nested.jai"), "value :: 42;").unwrap();
        let mut discovery = fixture.discovery();
        let graph = discovery.graph();
        let file = graph.module(graph.root()).unwrap().entry();
        let root = graph.symbols().find("Lib").unwrap();
        let member = graph.symbols().find("Nested").unwrap();
        let span = Span::new(0, 3);
        let id = discovery
            .retain_using(
                file,
                syntax::UsingDirective {
                    target: syntax::Expression {
                        kind: syntax::ExpressionKind::QualifiedName(NamePath {
                            root,
                            members: vec![member],
                        }),
                        span,
                    },
                    selection: syntax::UsingSelection::All,
                    span,
                },
                syntax::Visibility::Export,
                DiscoveryConditionContext::File,
            )
            .unwrap();
        let decision = decision(&discovery);
        assert_eq!(decision.bindings.len(), 1);
        assert_eq!(decision.bindings[0].name, "value");
        discovery.resolve_using(id, decision).unwrap();
    }
    #[test]
    fn staged_computed_only_and_except_preserve_original_bindings() {
        for only in [true, false] {
            let fixture = Fixture::new();
            let mut discovery = fixture.discovery();
            let name = discovery.graph().symbols().find("names").unwrap();
            let expression = syntax::Expression {
                kind: syntax::ExpressionKind::Call(name, vec![]),
                span: Span::new(22, 27),
            };
            let names = syntax::UsingNames::Expression(Box::new(expression));
            let id = request(
                &mut discovery,
                if only {
                    syntax::UsingSelection::Only(names)
                } else {
                    syntax::UsingSelection::Except(names)
                },
            );
            let decision = decision(&discovery);
            assert_eq!(decision.bindings.len(), 1);
            assert_eq!(
                decision.bindings[0].name,
                if only {
                    "value"
                } else {
                    "omitted"
                }
            );
            let original = decision.bindings[0].binding;
            discovery.resolve_using(id, decision).unwrap();
            let graph = discovery.graph();
            let file = graph.module(graph.root()).unwrap().entry();
            let name = graph
                .symbols()
                .find(if only {
                    "value"
                } else {
                    "omitted"
                })
                .unwrap();
            assert_eq!(
                graph
                    .lookup(
                        file,
                        &NamePath {
                            root: name,
                            members: vec![]
                        }
                    )
                    .unwrap(),
                original
            );
        }
    }
    #[test]
    fn staged_mapper_publishes_new_canonical_name_for_real_declaration() {
        let fixture = Fixture::new();
        let mut discovery = fixture.discovery();
        assert!(discovery.graph().symbols().find("renamed").is_none());
        let mapper = discovery.graph().symbols().find("mapper").unwrap();
        let id = request(
            &mut discovery,
            syntax::UsingSelection::Map(Box::new(syntax::Expression {
                kind: syntax::ExpressionKind::Name(mapper),
                span: Span::new(0, 3),
            })),
        );
        let decision = decision(&discovery);
        assert_eq!(decision.bindings.len(), 1);
        assert_eq!(decision.bindings[0].name, "renamed");
        let original = decision.bindings[0].binding;
        discovery.resolve_using(id, decision).unwrap();
        let graph = discovery.graph();
        let file = graph.module(graph.root()).unwrap().entry();
        let name = graph.symbols().find("renamed").unwrap();
        assert_eq!(
            graph
                .lookup(
                    file,
                    &NamePath {
                        root: name,
                        members: vec![]
                    }
                )
                .unwrap(),
            original
        );
    }
}
