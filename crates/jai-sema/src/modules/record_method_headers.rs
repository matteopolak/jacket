//! Reserve real member callable headers before shapes consume their constants.
use super::*;

pub(super) fn bind(
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    options: &crate::ResolveOptions,
) -> Result<(), LocatedDiagnostic> {
    let Some(file) = meta
        .record_specializations
        .method_owners()
        .find_map(|owner| {
            meta.record_specializations
                .method_environment(owner)
                .map(|environment| environment.file)
        })
    else {
        return Ok(());
    };
    let owner = declarations
        .generics
        .borrow_mut()
        .reserve_local_procedure()
        .map_err(|error| located(declarations.graph, file, error))?;
    let signatures = HashMap::new();
    let globals = HashMap::new();
    let mut resolver = Resolver {
        conditional_subjects: Vec::new(),
        expression_owner: None,
        debug: crate::debug_capture::Capture::default(),
        checks: crate::safety_checks::ActiveChecks::default(),
        local_scopes: crate::local_declarations::LocalScopes::default(),
        context: declarations.context.as_ref(),
        context_available: declarations.context.is_some(),
        meta,
        procedure: owner,
        types,
        target_layout: options.effective_layout(),
        places,
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
    resolver.bind_all_record_method_signatures_located()
}

/// Only a declaration-owned nominal prefix introduces a record namespace.
pub(super) fn aliases(
    graph: &ModuleGraph,
    nominals: &Nominals<'_>,
    types: &TypeRegistry,
) -> std::collections::HashSet<DeclarationId> {
    let mut aliases = std::collections::HashSet::new();
    loop {
        let before = aliases.len();
        for declaration in graph.declarations() {
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                continue;
            };
            let path = match &constant.initializer.kind {
                syntax::ExpressionKind::Name(name) => path(*name),
                syntax::ExpressionKind::QualifiedName(path) => path.clone(),
                _ => continue,
            };
            let is_alias = match graph.lookup(declaration.file(), &path) {
                Ok(jai_modules::Binding::Declaration(id)) => aliases.contains(&id),
                _ => false,
            };
            let is_member = (0..path.members.len()).any(|count| {
                let prefix = NamePath {
                    root: path.root,
                    members: path.members[..count].to_vec(),
                };
                let Ok(jai_modules::Binding::Declaration(id)) =
                    graph.lookup(declaration.file(), &prefix)
                else {
                    return false;
                };
                nominals
                    .declarations
                    .get(&id)
                    .is_some_and(|&ty| matches!(types.kind(ty), Ok(TypeKind::Record(_))))
            });
            if is_alias || is_member {
                aliases.insert(declaration.id());
            }
        }
        if aliases.len() == before {
            return aliases;
        }
    }
}

pub(super) fn publish_callable_aliases(
    declarations: &mut ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    meta: &mut crate::reflection::MetaContext,
    aliases: &std::collections::HashSet<DeclarationId>,
) -> Result<(), LocatedDiagnostic> {
    let graph = declarations.graph;
    loop {
        let mut changed = false;
        for &id in aliases {
            let first_publication = !declarations.values.contains_key(&id);
            let declaration = graph
                .declaration(id)
                .expect("record aliases retain source identity");
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                continue;
            };
            let path = match &constant.initializer.kind {
                syntax::ExpressionKind::Name(name) => path(*name),
                syntax::ExpressionKind::QualifiedName(path) => path.clone(),
                _ => continue,
            };
            let Some(crate::polymorphism::BakedValue::Value(value)) =
                aggregates::parameterized::member_value(
                    graph,
                    declaration.file(),
                    &declarations.nominals,
                    &meta.record_specializations,
                    None,
                    &path,
                    constant.span,
                )
            else {
                continue;
            };
            let jai_ir::ConstantKind::Procedure(procedure) = value.kind else {
                continue;
            };
            if let Some(annotation) = &constant.ty {
                let annotated = FileScope {
                    declarations,
                    file: declaration.file(),
                    substitution: None,
                }
                .annotation_with_specializations(
                    annotation,
                    types,
                    &mut meta.record_specializations,
                    constant.span,
                )
                .map_err(|error| located(graph, declaration.file(), error))?;
                if annotated != value.ty {
                    return Err(located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(
                            constant.span,
                            "record callable alias annotation differs from its canonical procedure type",
                        ),
                    ));
                }
            }
            let signature = meta
                .local_declarations
                .signature(procedure)
                .cloned()
                .or_else(|| {
                    declarations
                        .signatures
                        .values()
                        .find(|signature| signature.id == procedure)
                        .cloned()
                })
                .ok_or_else(|| {
                    located(
                        graph,
                        declaration.file(),
                        Diagnostic::new(
                            constant.span,
                            "record callable alias has no checked signature",
                        ),
                    )
                })?;
            if signature.ty != value.ty {
                return Err(located(
                    graph,
                    declaration.file(),
                    Diagnostic::new(
                        constant.span,
                        "record callable alias differs from its canonical signature",
                    ),
                ));
            }
            declarations.nominals.value_types.insert(id, value.ty);
            declarations
                .nominals
                .value_constants
                .insert(id, value.clone());
            declarations.values.insert(
                id,
                Binding::Procedure {
                    procedure,
                    ty: value.ty,
                },
            );
            declarations.signatures.insert(id, signature);
            // A later complete-header sweep refreshes defaults and result names
            // while preserving the published callable identity.
            changed |= first_publication;
        }
        if !changed {
            return Ok(());
        }
    }
}
