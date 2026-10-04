//! Retain actual file initializer recipes until checked values can be published.
use super::*;

pub(super) struct Job<'graph> {
    pub(super) declaration: DeclarationId,
    pub(super) file: FileInstanceId,
    pub(super) global: jai_ir::GlobalId,
    pub(super) expected: TypeId,
    pub(super) owner: ProcedureId,
    pub(super) location: SourceSpan,
    source: &'graph jai_modules::Declaration,
    recipe: Recipe<'graph>,
}

enum Recipe<'graph> {
    Ready(Global),
    Expression(&'graph syntax::Expression),
    GroupCopy(jai_ir::GlobalId),
}

pub(super) struct Jobs<'graph> {
    pending: std::collections::VecDeque<Job<'graph>>,
    completed: usize,
}

impl<'graph> Jobs<'graph> {
    pub(super) fn prepare(
        graph: &'graph ModuleGraph,
        declarations: &ScopedDeclarations<'graph>,
        types: &mut TypeRegistry,
        constants: &mut Constants<'graph>,
        meta: &mut crate::reflection::MetaContext,
        options: &crate::ResolveOptions,
    ) -> Result<Self, LocatedDiagnostic> {
        let mut pending = std::collections::VecDeque::new();
        let mut first_members = HashMap::new();
        for source in graph.declarations() {
            let FileDeclarationKind::Global(global) = &source.syntax().kind else {
                continue;
            };
            let file = source.file();
            let index = pending.len();
            let selected_initializer = match &global.declaration {
                syntax::Declaration::GroupMember {
                    group,
                    ordinal,
                    ..
                } => group.initializer_for(*ordinal),
                _ => None,
            };
            if let syntax::Declaration::GroupMember {
                group, ..
            } = &global.declaration
                && group.extra_initializers().is_empty()
                && let Some(expression) = selected_initializer
                && is_result_call(expression)
            {
                return Err(located(
                    graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "file declaration call-result lists require joint global result publication",
                    ),
                ));
            }
            let (expected, expression, external) = match global.declaration.source() {
                syntax::Declaration::GroupMember {
                    ..
                } => unreachable!("source() returns the original non-group declaration"),
                syntax::Declaration::External {
                    ty,
                    binding,
                    ..
                } => {
                    let expected = declarations.nominals.resolve_type_with_specializations(
                        graph,
                        aggregates::parameterized::TypeRequest::new(file, ty, global.span),
                        types,
                        &mut meta.record_specializations,
                        &mut |file, expression| constants.evaluate_lazy(file, expression),
                    )?;
                    let external =
                        external_data::declaration(graph, source, binding, expected, index, types)?;
                    (expected, None, Some(external))
                }
                syntax::Declaration::Inferred {
                    initializer, ..
                } => {
                    let initializer = selected_initializer.unwrap_or(initializer);
                    let expected = if enum_constants::is_pure_scalar(initializer)
                        && enum_constants::uses_enum(declarations, file, initializer)
                    {
                        enum_constants::evaluate_expression(
                            declarations,
                            types,
                            meta,
                            file,
                            initializer,
                            None,
                            options,
                        )
                        .map_err(|error| located(graph, file, error))?
                        .ty
                    } else {
                        match sequence_constants::infer_with_records(
                            graph,
                            file,
                            initializer,
                            types,
                            &declarations.nominals,
                            constants,
                            &mut meta.record_specializations,
                        )? {
                            Some(ty) => ty,
                            None => infer_constant_type(
                                graph,
                                file,
                                initializer,
                                types,
                                &declarations.nominals,
                                constants,
                            )?,
                        }
                    };
                    (expected, Some(initializer), None)
                }
                syntax::Declaration::Explicit {
                    ty,
                    initializer,
                    ..
                } => (
                    types.scalar(*ty),
                    selected_initializer.or(initializer.as_ref()),
                    None,
                ),
                syntax::Declaration::UnresolvedExplicit {
                    ty,
                    initializer,
                    ..
                } => {
                    let expected = declarations.nominals.resolve_type_with_specializations(
                        graph,
                        aggregates::parameterized::TypeRequest::new(file, ty, global.span),
                        types,
                        &mut meta.record_specializations,
                        &mut |file, expression| constants.evaluate_lazy(file, expression),
                    )?;
                    (
                        expected,
                        selected_initializer.or(initializer.as_ref()),
                        None,
                    )
                }
            };
            let copy_from = if let syntax::Declaration::GroupMember {
                group,
                ordinal,
                ..
            } = &global.declaration
            {
                if expression.is_some() {
                    let key = std::sync::Arc::as_ptr(group);
                    if !group.extra_initializers().is_empty() {
                        None
                    } else {
                        if *ordinal == 0 {
                            first_members.insert(key, jai_ir::GlobalId::new(index));
                            None
                        } else {
                            Some(
                                *first_members
                                    .get(&key)
                                    .expect("file group retains its source order"),
                            )
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            };
            let recipe = if let Some(first) = copy_from {
                Recipe::GroupCopy(first)
            } else {
                match (external, expression) {
                    (Some(external), _) => Recipe::Ready(external),
                    (_, Some(expression))
                        if requires_worklist(graph, file, expression, declarations) =>
                    {
                        Recipe::Expression(expression)
                    }
                    (_, Some(expression)) => {
                        let value = if enum_constants::is_pure_scalar(expression)
                            && (expected == types.meta_type()
                                || matches!(types.kind(expected), Ok(TypeKind::Enum(_)))
                                || enum_constants::uses_enum(declarations, file, expression))
                        {
                            enum_constants::evaluate_expression(
                                declarations,
                                types,
                                meta,
                                file,
                                expression,
                                Some(expected),
                                options,
                            )
                            .map_err(|error| located(graph, file, error))?
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
                            evaluator.expression(file, expression, expected)?
                        };
                        Recipe::Ready(Global::new_typed(index, value, types).map_err(|error| {
                            located(graph, file, Diagnostic::new(global.span, error.to_string()))
                        })?)
                    }
                    (_, None) => {
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
                        let value = evaluator.default_value(file, expected, global.span)?;
                        Recipe::Ready(Global::new_typed(index, value, types).map_err(|error| {
                            located(graph, file, Diagnostic::new(global.span, error.to_string()))
                        })?)
                    }
                }
            };
            let owner = declarations
                .generics
                .borrow_mut()
                .reserve_local_procedure()
                .map_err(|error| located(graph, file, error))?;
            pending.push_back(Job {
                declaration: source.id(),
                file,
                global: jai_ir::GlobalId::new(index),
                expected,
                owner,
                location: source.location(),
                source,
                recipe,
            });
        }
        Ok(Self {
            pending,
            completed: 0,
        })
    }

    pub(super) fn next(&self) -> Option<&Job<'graph>> {
        self.pending.front()
    }

    pub(super) fn is_complete(&self) -> bool {
        self.pending.is_empty()
    }

    pub(super) fn completed(&self) -> usize {
        self.completed
    }

    /// Publication follows original source order and happens only after an
    /// actual checked value exists. Later jobs cannot fabricate earlier storage.
    pub(super) fn publish(
        &mut self,
        value: Global,
        declarations: &mut ScopedDeclarations<'graph>,
        globals: &mut Vec<Global>,
        alignments: &mut Vec<storage_alignment::Job>,
    ) -> Result<(), LocatedDiagnostic> {
        let job = self
            .pending
            .front()
            .expect("publication retains its recipe");
        if value.id() != job.global
            || value.id().index() != globals.len()
            || value.ty() != job.expected
        {
            return Err(LocatedDiagnostic {
                location: job.location,
                message: "global initializer publication changed its reserved source identity"
                    .into(),
            });
        }
        let job = self.pending.pop_front().unwrap();
        declarations
            .values
            .insert(job.declaration, Binding::Storage(value.storage()));
        let FileDeclarationKind::Global(source) = &job.source.syntax().kind else {
            unreachable!("initializer retains a global source declaration")
        };
        if !source.declaration.attributes().is_empty() {
            alignments.push(storage_alignment::Job {
                declaration: job.declaration,
                file: job.file,
                global: job.global,
                syntax: source.declaration.clone(),
            });
        }
        globals.push(value);
        self.completed += 1;
        Ok(())
    }
}

fn requires_worklist(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    declarations: &ScopedDeclarations<'_>,
) -> bool {
    let deferred = deferred_constants::classify(graph);
    let mut required = false;
    let scope = FileScope {
        declarations,
        file,
        substitution: None,
    };
    deferred_constants::visit(expression, |expression| {
        use syntax::ExpressionKind as E;
        if matches!(
            expression.kind,
            E::CompileTime(_)
                | E::AnonymousProcedure(_)
                | E::BakeArguments(_)
                | E::ShortLambda(_)
                | E::Call(..)
                | E::QualifiedCall(..)
                | E::IndirectCall { .. }
        ) {
            required = true;
        }
        let name = match &expression.kind {
            E::Name(name) => Some(path(*name)),
            E::QualifiedName(name) => Some(name.clone()),
            _ => None,
        };
        if let Some(name) = name {
            // A genuine builtin type value needs canonical expression binding,
            // even when its destination later rejects the meta Type. Scalar
            // preparation cannot decide that binding alternative.
            if scope.type_value_name_absent(&name)
                && syntax::BuiltinType::from_spelling(graph.symbols().name(name.root)).is_some()
            {
                required = true;
            }
            if let Ok(jai_modules::Binding::Declaration(id)) = graph.lookup(file, &name)
                && deferred.contains(&id)
                && !declarations.values.contains_key(&id)
            {
                required = true;
            }
        }
    });
    required
}

impl Job<'_> {
    pub(super) fn evaluate(
        &self,
        context: &crate::compile_time::Context<'_>,
        declarations: &ScopedDeclarations<'_>,
        types: &mut TypeRegistry,
        places: &mut PlaceRegistry,
        meta: &mut crate::reflection::MetaContext,
        options: &crate::ResolveOptions,
    ) -> Result<Global, Diagnostic> {
        if let Recipe::GroupCopy(first) = self.recipe {
            let first = context.globals.get(first.index()).ok_or_else(|| {
                Diagnostic::new(
                    self.location.span,
                    "shared global initializer has not been published",
                )
            })?;
            if first.ty() != self.expected {
                return Err(Diagnostic::new(
                    self.location.span,
                    "shared global initializer type changed",
                ));
            }
            return Ok(Global::new(
                self.global.index(),
                first.initializer().clone(),
                types,
            ));
        }
        let Recipe::Expression(expression) = &self.recipe else {
            let Recipe::Ready(global) = &self.recipe else {
                unreachable!()
            };
            return Ok(global.clone());
        };
        let empty_signatures = HashMap::new();
        let empty_values = HashMap::new();
        let mut resolver = Resolver {
            conditional_subjects: Vec::new(),
            expression_owner: Some(self.owner),
            debug: crate::debug_capture::Capture::default(),
            checks: crate::safety_checks::ActiveChecks::default(),
            local_scopes: crate::local_declarations::LocalScopes::default(),
            context: declarations.context.as_ref(),
            context_available: true,
            meta,
            procedure: self.owner,
            types,
            target_layout: options.effective_layout(),
            places,
            signatures: &empty_signatures,
            globals: &empty_values,
            graph_scope: Some(FileScope {
                declarations,
                file: self.file,
                substitution: None,
            }),
            compile_time: Some(context),
            symbols: declarations.graph.symbols(),
            scopes: vec![HashMap::new()],
            locals: vec![],
            span: self.location.span,
            results: &[],
            loops: vec![],
            next_loop: 0,
            active_push: None,
            next_push: 0,
            cleanups: vec![],
            deferred_scopes: vec![],
            cleanup_context: None,
        };
        let run = match &expression.kind {
            syntax::ExpressionKind::CompileTime(run) => run.clone(),
            _ => syntax::CompileTimeRun {
                flags: Default::default(),
                body: syntax::CompileTimeBody::Expression(Box::new((*expression).clone())),
            },
        };
        let value = resolver
            .execute_compile_time(&run, expression.span, Some(self.expected))?
            .ok_or_else(|| {
                Diagnostic::new(
                    expression.span,
                    "void #run cannot initialize global storage",
                )
            })?;
        Global::new_typed(self.global.index(), value, resolver.types)
            .map_err(|error| Diagnostic::new(expression.span, error.to_string()))
    }
}

fn is_result_call(expression: &syntax::Expression) -> bool {
    matches!(
        expression.kind,
        syntax::ExpressionKind::Call(..)
            | syntax::ExpressionKind::QualifiedCall(..)
            | syntax::ExpressionKind::IndirectCall { .. }
            | syntax::ExpressionKind::ContextCall { .. }
            | syntax::ExpressionKind::CallHint { .. }
    )
}
