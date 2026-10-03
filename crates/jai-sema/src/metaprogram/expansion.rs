//! Source macros evaluate runtime inputs once and bind quoted inputs structurally.
use super::*;
use crate::Block;

impl Resolver<'_> {
    pub(crate) fn expand_statement(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<Option<Block>, Diagnostic> {
        match &expression.kind {
            syntax::ExpressionKind::Call(name, arguments) => self.expand_call(
                &syntax::NamePath {
                    root: *name,
                    members: Vec::new(),
                },
                arguments,
                expression.span,
            ),
            syntax::ExpressionKind::QualifiedCall(path, arguments) => {
                self.expand_call(path, arguments, expression.span)
            }
            _ => Ok(None),
        }
    }
    pub(crate) fn reject_value_expansion(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self.expanded_target(path, span)?.is_some() {
            return Err(Diagnostic::new(
                span,
                "#expand procedure calls currently require statement position; value results need expansion result binding",
            ));
        }
        Ok(())
    }
    pub(crate) fn expand_call(
        &mut self,
        path: &syntax::NamePath,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Option<Block>, Diagnostic> {
        let Some(target) = self.expanded_target(path, span)? else {
            return Ok(None);
        };
        let procedure = &target.procedure;
        if !procedure.results.is_empty() {
            return Err(Diagnostic::new(
                span,
                "#expand procedures with return values or return statements require expansion result binding",
            ));
        }
        if procedure.parameters.iter().any(|parameter| {
            parameter.using
                || (parameter.variadic
                    && parameter.evaluation != syntax::ParameterEvaluation::Discard)
        }) {
            return Err(Diagnostic::new(
                span,
                "variadic and using #expand parameters are not implemented",
            ));
        }
        self.meta.codes.enter_macro(target.id, span)?;
        self.meta.codes.push_export_remap(Vec::new());
        let result = self.expand_body(&target, arguments, span).and_then(|body| {
            self.publish_caller_exports(span)?;
            Ok(body)
        });
        self.meta.codes.leave_macro(target.id);
        self.meta.codes.pop_export_remap();
        result.map(Some)
    }

    pub(super) fn expanded_target(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<ExpandedTarget>, Diagnostic> {
        // A pending inferred lambda acquires its procedure type from the call.
        // Inspect its source without eagerly binding it as an ordinary value.
        if self.with_local_constant_source(path, span, |_, _, constant| {
            Ok(Some(matches!(
                &constant.initializer.kind,
                syntax::ExpressionKind::ShortLambda(_)
            )))
        })? == Some(true)
        {
            return Ok(None);
        }
        if let Some(binding) = self.lexical_graph_binding(path, span)? {
            if matches!(
                binding,
                jai_modules::Binding::SourceMember { .. } | jai_modules::Binding::StorageMember(_)
            ) {
                return self
                    .expanded_source_member_candidates(binding, span)
                    .map(|mut targets| targets.pop());
            }
            return self
                .graph_scope
                .expect("lexical import has a graph")
                .imported_expanded_procedure(binding, span)
                .map(|target| target.map(ExpandedTarget::module));
        }
        if let Some(binding) = self.namespace_binding(path, span)? {
            return match binding {
                Binding::Macro(id) => self.local_expanded_target(id, span).map(Some),
                _ => Ok(None),
            };
        }
        // Ordinary lexical values shadow same-spelled module procedures.
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            return match binding {
                Binding::Macro(id) if path.members.is_empty() => {
                    self.local_expanded_target(id, span).map(Some)
                }
                _ => Ok(None),
            };
        }
        self.graph_scope
            .map_or(Ok(None), |scope| scope.expanded_procedure(path, span))
            .map(|target| target.map(ExpandedTarget::module))
    }

    fn expand_body(
        &mut self,
        target: &ExpandedTarget,
        arguments: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Block, Diagnostic> {
        let procedure = &target.procedure;
        let mut assigned = vec![false; procedure.parameters.len()];
        let mut bindings = Vec::with_capacity(procedure.parameters.len());
        let mut initializers = Vec::new();
        let mut positional = 0;
        // Argument expressions are resolved in source order in the caller scope.
        for argument in arguments {
            let index = if let Some(name) = argument.name {
                procedure
                    .parameters
                    .iter()
                    .position(|parameter| parameter.name == name)
                    .ok_or_else(|| {
                        Diagnostic::new(argument.value.span, "unknown expanded parameter name")
                    })?
            } else {
                while positional < assigned.len()
                    && assigned[positional]
                    && !(procedure.parameters[positional].variadic
                        && procedure.parameters[positional].evaluation
                            == syntax::ParameterEvaluation::Discard)
                {
                    positional += 1;
                }
                let index = positional;
                if index < procedure.parameters.len() && !procedure.parameters[index].variadic {
                    positional += 1;
                }
                index
            };
            if index >= assigned.len() {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "too many expanded arguments",
                ));
            }
            let parameter = &procedure.parameters[index];
            let discarded_pack =
                parameter.variadic && parameter.evaluation == syntax::ParameterEvaluation::Discard;
            if argument.spread && !discarded_pack {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "spread #expand arguments require macro pack binding",
                ));
            }
            let already_assigned = std::mem::replace(&mut assigned[index], true);
            if already_assigned && (!discarded_pack || argument.name.is_some()) {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate expanded argument",
                ));
            }
            if discarded_pack && argument.name.is_some() {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "named discarded macro packs require explicit pack binding",
                ));
            }
            if argument.spread {
                let element = self.expanded_discarded_type(target, parameter)?;
                let pack = self
                    .types
                    .slice(element)
                    .map_err(|error| Diagnostic::new(argument.value.span, error.to_string()))?;
                self.check_discarded_argument(&argument.value, pack)?;
                if !already_assigned {
                    bindings.push((parameter.name, Binding::Discarded(pack)));
                }
                continue;
            }
            let binding =
                self.expanded_argument(target, parameter, &argument.value, &mut initializers)?;
            if !already_assigned {
                bindings.push((parameter.name, binding));
            }
        }
        for (index, assigned) in assigned.iter().enumerate() {
            if *assigned {
                continue;
            }
            let parameter = &procedure.parameters[index];
            if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                bindings.push((
                    parameter.name,
                    self.missing_discarded_binding(target, parameter)?,
                ));
            } else if let Some(binding) =
                self.expanded_caller_location_default(target, parameter, &mut initializers, span)?
            {
                bindings.push((parameter.name, binding));
            } else {
                return Err(Diagnostic::new(
                    span,
                    "missing expanded argument; macro parameter defaults are not implemented",
                ));
            }
        }
        self.expand_bound_target_body(target, bindings, initializers, span)
    }

    pub(super) fn expand_bound_definition_body(
        &mut self,
        target: &ExpandedTarget,
        bindings: Vec<(Symbol, Binding)>,
        mut initializers: Vec<Statement>,
        span: Span,
    ) -> Result<Block, Diagnostic> {
        let procedure = &target.procedure;
        self.remember_local_deprecation(
            crate::deprecation_warnings::DeprecationKey::Macro(target.id),
            procedure.name,
            procedure.deprecation.as_ref(),
            crate::deprecation_warnings::procedure_extent(procedure),
            target
                .capture
                .as_ref()
                .map(|capture| capture.location.source)
                .or_else(|| self.graph_scope.map(|scope| scope.source())),
        )?;
        self.warn_deprecated(
            crate::deprecation_warnings::DeprecationKey::Macro(target.id),
            span,
        )?;
        if let Some(capture) = &target.capture
            && capture.procedure != self.procedure
            && capture.frames.iter().flat_map(|frame| frame.values()).any(
                |binding| matches!(binding, Binding::Storage(storage) if matches!(storage.place().kind(), jai_ir::PlaceKind::Local(_))),
            )
        {
            return Err(Diagnostic::new(
                span,
                "local macro storage cannot escape its defining procedure",
            ));
        }
        let original_file = self.graph_scope;
        let mut frames = target
            .capture
            .as_ref()
            .map_or_else(Vec::new, |capture| capture.frames.clone());
        if let Some(substitution) = self.graph_scope.and_then(|scope| scope.substitution) {
            let mut inferred = HashMap::new();
            for binding in &substitution.types {
                inferred.insert(binding.name, Binding::Type(binding.ty));
            }
            for binding in &substitution.constants {
                inferred.insert(
                    binding.name,
                    self.baked_record_binding(binding.value.clone()),
                );
            }
            frames.push(inferred);
        }
        frames.push(HashMap::new());
        let original_scopes = std::mem::replace(&mut self.scopes, frames);
        let expansion_locals = target.capture.as_ref().map_or_else(
            || self.local_scopes.isolated_expansion(),
            |capture| {
                let mut locals = capture.local_scopes.clone();
                locals.resume_after_expansion(&self.local_scopes);
                locals
            },
        );
        let mut original_locals = std::mem::replace(&mut self.local_scopes, expansion_locals);
        // The reborrow wrapper installed only this definition's substitution.
        self.graph_scope = original_file;
        let source = target
            .capture
            .as_ref()
            .map(|capture| capture.location.source)
            .or_else(|| self.graph_scope.map(|scope| scope.source()));
        let original_source = self.debug.replace_source(source);
        self.meta.codes.source_files.push(
            target
                .capture
                .as_ref()
                .map_or(target.file, |capture| capture.source_file),
        );
        let declaration_policy = target.capture.as_ref().map_or(procedure.debug, |capture| {
            capture.debug.nested(procedure.debug)
        });
        let original_debug_policy = self.debug.enter_policy(declaration_policy);
        let original_span = std::mem::replace(&mut self.span, procedure.span);
        let original_checks = self.checks;
        if let Some(capture) = &target.capture {
            self.checks = capture.checks;
        }
        self.meta.codes.return_regions.push((self.procedure, false));
        let result = (|| {
            for (name, binding) in bindings {
                self.bind_name(name, binding)?;
            }
            let body = self.checked_block(procedure.checks, &procedure.body, false)?;
            self.debug.prepend_block(initializers.len());
            self.debug
                .attach_block(&[jai_ir::DebugPathStep::Child(jai_ir::DebugBranch::Block)]);
            initializers.extend(body.statements);
            Ok(Block {
                statements: initializers,
                flow: body.flow,
            })
        })();
        let result = result.map_err(|error: Diagnostic| match source {
            Some(source) => error.with_fallback_source(source),
            None => error,
        });
        self.scopes = original_scopes;
        original_locals.resume_after_expansion(&self.local_scopes);
        self.local_scopes = original_locals;
        self.graph_scope = original_file;
        self.debug.replace_source(original_source);
        self.meta.codes.source_files.pop();
        self.debug.restore_policy(original_debug_policy);
        self.span = original_span;
        self.checks = original_checks;
        self.meta.codes.return_regions.pop();
        result
    }

    fn expanded_argument(
        &mut self,
        target: &ExpandedTarget,
        parameter: &syntax::Parameter,
        expression: &syntax::Expression,
        initializers: &mut Vec<Statement>,
    ) -> Result<Binding, Diagnostic> {
        if parameter.baking == syntax::ParameterBaking::Optional {
            return Err(Diagnostic::new(
                expression.span,
                "optional baking in #expand parameters requires selected constant or runtime binding",
            ));
        }
        if parameter.evaluation == syntax::ParameterEvaluation::Discard {
            let expected = self.expanded_discarded_type(target, parameter)?;
            self.check_discarded_argument(expression, expected)?;
            return Ok(Binding::Discarded(expected));
        }
        let explicit = match &parameter.binding {
            syntax::ParameterBinding::Required(ty) => Some(syntax::TypeSyntax::Builtin(
                syntax::BuiltinType::Scalar(*ty),
            )),
            syntax::ParameterBinding::RequiredType(ty) => Some(ty.clone()),
            syntax::ParameterBinding::Defaulted { ty, .. } => {
                ty.map(|ty| syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty)))
            }
            syntax::ParameterBinding::DefaultedType { ty, .. } => ty.clone(),
        };
        let explicit = explicit
            .map(|ty| self.expanded_annotation(target, &ty, parameter.span))
            .transpose()?;
        let caller_location_type =
            self.expanded_caller_location_parameter_type(target, parameter)?;
        let explicit = explicit.or(caller_location_type);
        let is_code = explicit == Some(self.types.code_type());
        if is_code {
            // Explicit Code values keep their original capture. Ordinary source
            // arguments are quoted without evaluating their runtime expression.
            let existing = match &expression.kind {
                syntax::ExpressionKind::Code(_) => Some(self.expr(expression)?),
                syntax::ExpressionKind::Name(name) => {
                    let lexical = self.resolve_local_name(*name, expression.span)?;
                    let binding = lexical.or_else(|| {
                        self.graph_scope.and_then(|scope| {
                            scope
                                .value(
                                    &syntax::NamePath {
                                        root: *name,
                                        members: Vec::new(),
                                    },
                                    expression.span,
                                )
                                .ok()
                        })
                    });
                    match binding {
                        Some(Binding::Code(id)) => Some(Expr::Code(id)),
                        _ => None,
                    }
                }
                syntax::ExpressionKind::QualifiedName(path) => self
                    .graph_scope
                    .and_then(|scope| scope.value(path, expression.span).ok())
                    .and_then(|binding| match binding {
                        Binding::Code(id) => Some(Expr::Code(id)),
                        _ => None,
                    }),
                _ => None,
            };
            let id = match existing {
                Some(Expr::Code(id)) => id,
                _ => match self.capture_code(
                    &syntax::CodeBody::Expression(Box::new(expression.clone())),
                    expression.span,
                )? {
                    Expr::Code(id) => id,
                    _ => unreachable!(),
                },
            };
            return Ok(Binding::Code(id));
        }
        let value = self.expr(expression)?;
        if explicit == Some(self.types.meta_type()) {
            return match value {
                Expr::Type(ty) => Ok(Binding::Type(ty)),
                _ => Err(Diagnostic::new(
                    expression.span,
                    "expanded Type parameter requires a type value",
                )),
            };
        }
        if parameter.baking == syntax::ParameterBaking::Required {
            return match value {
                Expr::Type(_) | Expr::Code(_) => Err(Diagnostic::new(
                    expression.span,
                    "expanded metatype argument does not match the formal parameter type",
                )),
                value => {
                    let ty = match explicit {
                        Some(ty) => ty,
                        None => self.expression_type(&value, expression.span)?,
                    };
                    let value = self.coerce_value(value, ty, expression.span)?;
                    let value = self.literal_constant(value, expression.span)?;
                    Ok(Binding::TypedConstant(self.meta.intern_constant(value)))
                }
            };
        }
        let ty = match explicit {
            Some(ty) => ty,
            None => self.expression_type(&value, expression.span)?,
        };
        let value = self.coerce_value(value, ty, expression.span)?;
        let local = self.allocate_typed(ty)?;
        let storage = crate::Storage::local(local, self.types);
        self.debug_prefix_local(
            local,
            parameter.name,
            parameter.span,
            target.capture.as_ref().map_or_else(
                || {
                    self.graph_scope
                        .expect("macro source graph")
                        .code_file(target.file)
                        .source()
                },
                |capture| capture.location.source,
            ),
            expression.span,
            initializers.len(),
        )?;
        initializers.push(Statement::Store(local.place(), value));
        Ok(Binding::Storage(storage))
    }
}
