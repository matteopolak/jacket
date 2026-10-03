use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn scalar(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        substitution: Option<&Substitution>,
    ) -> TypeResult<ScalarConstant> {
        if substitution.is_none() && !self.lexical_active {
            return self.evaluate_scalar(file, expression);
        }
        jai_eval::evaluate_paths(expression, |path, span| {
            if self.lexical_active
                && let Some(binding) = self
                    .lexical
                    .and_then(|scope| scope.bindings.get(&(span.start, span.end)))
            {
                return lexical::lexical_scalar(binding).ok_or_else(|| {
                    Diagnostic::new(span, "lexical type argument requires a scalar constant")
                });
            }
            if self.lexical_active
                && let Some(lexical) = self.lexical
                && lexical.roots.contains_key(&path.root)
            {
                return lexical
                    .lookup(path)
                    .and_then(lexical::lexical_scalar)
                    .ok_or_else(|| {
                        Diagnostic::new(span, "lexical type argument requires a scalar constant")
                    });
            }
            if let Some(value) = member_value(
                self.graph,
                file,
                self.nominals,
                self.records,
                substitution,
                path,
                span,
            ) {
                return baked_scalar(&value).ok_or_else(|| {
                    Diagnostic::new(span, "record template value requires a scalar constant")
                });
            }
            (self.evaluate)(
                file,
                &syntax::Expression {
                    kind: syntax::ExpressionKind::QualifiedName(path.clone()),
                    span,
                },
            )
            .map_err(|e| Diagnostic::at_source(e.location, e.message))
        })
        .map_err(|e| failure(self.graph, file, e))
    }
    pub(super) fn parameter_type(
        &mut self,
        file: FileInstanceId,
        parameter: &syntax::RecordParameter,
        substitution: &Substitution,
    ) -> TypeResult<TypeId> {
        self.without_record_annotation(|resolver| {
            resolver.in_lexical_scope(false, |resolver| {
                resolver.parameter_type_inner(file, parameter, substitution)
            })
        })
    }
    pub(super) fn parameter_type_inner(
        &mut self,
        file: FileInstanceId,
        parameter: &syntax::RecordParameter,
        substitution: &Substitution,
    ) -> TypeResult<TypeId> {
        match &parameter.binding {
            syntax::RecordParameterBinding::Typed { ty, .. } => {
                self.resolve(file, ty, Some(substitution), parameter.span)
            }
            syntax::RecordParameterBinding::InferredDefault(default) => {
                if let Some(ty) = type_expression(default) {
                    match self.resolve(file, &ty, Some(substitution), default.span) {
                        Ok(_) => return Ok(self.types.meta_type()),
                        Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                        Err(TypeFailure::Diagnostic(_)) => {}
                    }
                }
                self.inferred_default_type(file, default, substitution)
            }
        }
    }
    pub(super) fn inferred_default_type(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        substitution: &Substitution,
    ) -> TypeResult<TypeId> {
        use syntax::ExpressionKind as E;
        if let Some(path) = expression_path(expression) {
            if let Some(value) = member_value(
                self.graph,
                file,
                self.nominals,
                self.records,
                Some(substitution),
                &path,
                expression.span,
            ) {
                return Ok(match value {
                    BakedValue::Value(value) => value.ty,
                    BakedValue::Float(value) => self.types.float(value.ty()),
                    BakedValue::String(_) => self.types.string(),
                    BakedValue::Type(_) => self.types.meta_type(),
                    BakedValue::Code(_) => self.types.code_type(),
                });
            }
            if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, &path) {
                match &self
                    .graph
                    .parameter(id)
                    .expect("parameter binding exists")
                    .value
                {
                    jai_modules::ParameterValue::Scalar(value) => {
                        return Ok(value.type_id(self.types));
                    }
                    jai_modules::ParameterValue::String(_) => return Ok(self.types.string()),
                    jai_modules::ParameterValue::Type(_) => return Ok(self.types.meta_type()),
                    jai_modules::ParameterValue::Enumeration(value) => {
                        return self
                            .nominals
                            .declarations
                            .get(&value.declaration)
                            .copied()
                            .ok_or_else(|| {
                                failure(
                                    self.graph,
                                    file,
                                    Diagnostic::new(
                                        expression.span,
                                        "module enum parameter has no resolved nominal declaration",
                                    ),
                                )
                            });
                    }
                    jai_modules::ParameterValue::ContextualMember(_) => {}
                }
            }
            if let Some(ty) = self.nominals.value_type(self.graph, file, &path) {
                return Ok(ty);
            }
        }
        match &expression.kind {
            E::String(_) | E::HereString(_) => Ok(self.types.string()),
            E::TypeCast { ty, .. } => self.resolve(file, ty, Some(substitution), expression.span),
            E::StructLiteral(literal) if literal.ty.is_some() => self.resolve(
                file,
                &syntax::TypeSyntax::Named(literal.ty.clone().unwrap()),
                Some(substitution),
                expression.span,
            ),
            E::PositionalStructLiteral(literal) if literal.ty.is_some() => self.resolve(
                file,
                &syntax::TypeSyntax::Named(literal.ty.clone().unwrap()),
                Some(substitution),
                expression.span,
            ),
            E::ArrayLiteral(literal) => {
                let element = match &literal.element_type {
                    Some(ty) => self.resolve(file, ty, Some(substitution), expression.span)?,
                    None => self.inferred_default_type(
                        file,
                        literal.elements.first().ok_or_else(|| {
                            failure(
                                self.graph,
                                file,
                                Diagnostic::new(
                                    expression.span,
                                    "empty baked array default requires an element type",
                                ),
                            )
                        })?,
                        substitution,
                    )?,
                };
                let count = u64::try_from(literal.elements.len()).map_err(|_| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "baked array default length is out of range",
                        ),
                    )
                })?;
                self.types.fixed_array(element, count).map_err(|error| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(expression.span, error.to_string()),
                    )
                })
            }
            _ => Ok(self
                .scalar(file, expression, Some(substitution))?
                .type_id(self.types)),
        }
    }
    pub(super) fn baked(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        expected: TypeId,
        substitution: Option<&Substitution>,
    ) -> TypeResult<BakedValue> {
        let lexical = self.lexical.filter(|_| self.lexical_active);
        let expression_key = (expression.span.start, expression.span.end);
        if let Some(binding) = lexical.and_then(|scope| scope.bindings.get(&expression_key)) {
            match binding {
                LexicalTypeArgument::Type(ty) if expected == self.types.meta_type() => {
                    return Ok(BakedValue::Type(*ty));
                }
                LexicalTypeArgument::Code(code) if expected == self.types.code_type() => {
                    return Ok(BakedValue::Code(*code));
                }
                LexicalTypeArgument::Typed(value) => {
                    return self.normalize_argument_constant(
                        file,
                        expression.span,
                        value.clone(),
                        expected,
                        lexical,
                    );
                }
                _ => {}
            }
        }
        if let Some(value) = lexical.and_then(|scope| scope.values.get(&expression_key)) {
            return self.normalize_argument_constant(
                file,
                expression.span,
                value.clone(),
                expected,
                lexical,
            );
        }
        if expected == self.types.meta_type() {
            if let Some(ty) = lexical.and_then(|scope| scope.types.get(&expression_key)) {
                return Ok(BakedValue::Type(*ty));
            }
            let syntax = type_expression(expression).ok_or_else(|| {
                failure(
                    self.graph,
                    file,
                    Diagnostic::new(expression.span, "record template argument requires a type"),
                )
            })?;
            return self
                .resolve(file, &syntax, substitution, expression.span)
                .map(BakedValue::Type);
        }
        let expression_path = match &expression.kind {
            syntax::ExpressionKind::Name(name) => Some(path(*name)),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if let Some(LexicalTypeArgument::Typed(value)) = expression_path
            .as_ref()
            .and_then(|path| lexical?.lookup(path))
        {
            return self.normalize_argument_constant(
                file,
                expression.span,
                value.clone(),
                expected,
                lexical,
            );
        }
        if expected == self.types.code_type() {
            let value = lexical
                .and_then(|scope| scope.codes.get(&expression_key))
                .copied()
                .or_else(|| {
                    expression_path
                        .as_ref()
                        .and_then(|path| lexical.and_then(|scope| scope.lookup(path)))
                        .and_then(|value| match value {
                            LexicalTypeArgument::Code(id) => Some(*id),
                            _ => None,
                        })
                        .or_else(|| {
                            expression_path
                                .as_ref()
                                .and_then(|path| {
                                    member_value(
                                        self.graph,
                                        file,
                                        self.nominals,
                                        self.records,
                                        substitution,
                                        path,
                                        expression.span,
                                    )
                                })
                                .and_then(|value| match value {
                                    BakedValue::Code(id) => Some(id),
                                    _ => None,
                                })
                        })
                });
            return value.map(BakedValue::Code).ok_or_else(|| {
                failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        expression.span,
                        "record Code argument requires captured code in its defining lexical scope",
                    ),
                )
            });
        }
        let graph = self.graph;
        let evaluate = &mut self.evaluate;
        let mut caller_evaluate = |file, expression: &syntax::Expression| {
            jai_eval::evaluate_paths(expression, |path, span| {
                if let Some(binding) =
                    lexical.and_then(|scope| scope.bindings.get(&(span.start, span.end)))
                {
                    return lexical::lexical_scalar(binding).ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "lexical type argument requires an immutable scalar constant",
                        )
                    });
                }
                if let Some(lexical) = lexical
                    && lexical.roots.contains_key(&path.root)
                {
                    return lexical
                        .lookup(path)
                        .and_then(lexical::lexical_scalar)
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "lexical type argument requires an immutable scalar constant",
                            )
                        });
                }
                evaluate(
                    file,
                    &syntax::Expression {
                        kind: syntax::ExpressionKind::QualifiedName(path.clone()),
                        span,
                    },
                )
                .map_err(|error| Diagnostic::at_source(error.location, error.message))
            })
            .map_err(|error| located(graph, file, error))
        };
        let mut evaluator = super::super::defaults::Defaults::with_evaluator(
            self.graph,
            self.types,
            self.nominals,
            &mut caller_evaluate,
        )
        .with_specializations(self.records)
        .with_substitution(substitution.cloned());
        let value = evaluator.expression(file, expression, expected)?;
        BakedValue::runtime(value, self.types).map_err(|error| {
            failure(
                self.graph,
                file,
                Diagnostic::new(expression.span, error.to_string()),
            )
        })
    }
    fn normalize_argument_constant(
        &mut self,
        file: FileInstanceId,
        span: Span,
        value: jai_ir::ConstantValue,
        expected: TypeId,
        lexical: Option<&LexicalTypeArguments>,
    ) -> TypeResult<BakedValue> {
        let mut unavailable = |file, expression: &syntax::Expression| {
            Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "typed argument projection does not evaluate source expressions",
                ),
            ))
        };
        let mut defaults = super::super::defaults::Defaults::with_evaluator(
            self.graph,
            self.types,
            self.nominals,
            &mut unavailable,
        )
        .with_specializations(self.records);
        if let Some(lexical) = lexical {
            defaults = defaults.with_conversion_fields(&lexical.conversions);
        }
        let value = defaults
            .coerce_field_constant(value, expected, span)
            .map_err(|error| failure(self.graph, file, error))?;
        BakedValue::runtime(value, self.types)
            .map_err(|error| failure(self.graph, file, Diagnostic::new(span, error.to_string())))
    }
    pub(super) fn application(
        &mut self,
        caller_file: FileInstanceId,
        application: &syntax::TypeApplicationSyntax,
        caller_substitution: Option<&Substitution>,
    ) -> TypeResult<TypeId> {
        let syntax::TypeSyntax::Named(base) = application.base.as_ref() else {
            return Err(failure(
                self.graph,
                caller_file,
                Diagnostic::new(
                    application.span,
                    "type application requires a named record template",
                ),
            ));
        };
        let id = if self.lexical_active
            && let Some(id) = self
                .lexical
                .and_then(|scope| {
                    scope
                        .templates
                        .get(&(application.span.start, application.span.end))
                })
                .copied()
        {
            id
        } else {
            self.template_declaration(caller_file, base, application.span)?
        };
        let declaration = self.graph.declaration(id).expect("record template exists");
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            unreachable!("template lookup establishes record kind");
        };
        if record.parameters.is_empty() {
            return Err(failure(
                self.graph,
                caller_file,
                Diagnostic::new(
                    application.span,
                    "record declaration has no template parameters",
                ),
            ));
        }
        let caller_lexical = self.lexical_active;
        let bound =
            binder::bind_arguments(&record.parameters, &application.arguments, application.span)
                .map_err(|e| failure(self.graph, caller_file, e))?;
        let mut substitution = Substitution::default();
        for bound in bound {
            let expected =
                self.parameter_type(declaration.file(), bound.parameter, &substitution)?;
            let (file, overlay) = if bound.defaulted {
                (declaration.file(), Some(&substitution))
            } else {
                (caller_file, caller_substitution)
            };
            let value = self.in_lexical_scope(caller_lexical && !bound.defaulted, |resolver| {
                resolver.baked(file, bound.expression, expected, overlay)
            })?;
            substitution.bind_constant(bound.parameter.name, value);
        }
        self.instantiate_at(
            id,
            substitution,
            jai_source::SourceSpan {
                source: self.graph.file(caller_file).unwrap().source(),
                span: application.span,
            },
        )
    }
}
