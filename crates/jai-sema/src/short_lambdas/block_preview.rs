//! Statement facts used before selecting a lambda's callback overload.
use super::*;
use crate::overloads::{ArgumentInfo, ArgumentType, ConstantArgument};

#[derive(Default)]
struct BlockResults {
    values: Vec<(ArgumentInfo, Span)>,
    void_return: bool,
}

#[derive(Clone, Copy)]
struct BlockContext<'a> {
    signature: &'a ProcedureType,
    source_results: Option<&'a [ResultSignature]>,
    inferred: bool,
    loops: usize,
    deferred: bool,
    depth: usize,
}

impl Resolver<'_> {
    pub(super) fn preview_short_lambda_block(
        &mut self,
        body: &[syntax::Statement],
        signature: &ProcedureType,
        inferred: bool,
        span: Span,
    ) -> Result<Option<ArgumentInfo>, Diagnostic> {
        self.preview_callable_block(body, signature, inferred, None, span)
    }
    pub(crate) fn preview_full_procedure_block(
        &mut self,
        body: &[syntax::Statement],
        signature: &ProcedureType,
        source_results: &[ResultSignature],
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.preview_callable_block(body, signature, false, Some(source_results), span)
            .map(|_| ())
    }
    fn preview_callable_block(
        &mut self,
        body: &[syntax::Statement],
        signature: &ProcedureType,
        inferred: bool,
        source_results: Option<&[ResultSignature]>,
        span: Span,
    ) -> Result<Option<ArgumentInfo>, Diagnostic> {
        let mut results = BlockResults::default();
        self.scopes.push(HashMap::new());
        let flow = self.preview_lambda_statements(
            body,
            BlockContext {
                signature,
                source_results,
                inferred,
                loops: 0,
                deferred: false,
                depth: 0,
            },
            &mut results,
        );
        self.scopes.pop();
        let returns = flow?;
        let expected = signature.results.first().copied();
        if !inferred {
            if expected.is_some() && !returns {
                return Err(Diagnostic::new(
                    span,
                    "value-returning short lambda block may reach its end",
                ));
            }
            return Ok(expected.map(ArgumentInfo::typed));
        }
        if results.values.is_empty() {
            return Ok(None);
        }
        if results.void_return || !returns {
            return Err(Diagnostic::new(
                span,
                "value-returning short lambda block may reach its end without a value",
            ));
        }
        // A strong return supplies context to weak leaves, just as a conditional does.
        let candidate = results
            .values
            .iter()
            .find(|(info, _)| {
                matches!(
                    info.ty,
                    ArgumentType::Known(_) | ArgumentType::StringLiteral(_)
                )
            })
            .or_else(|| {
                results.values.iter().find(|(info, _)| {
                    matches!(
                        info.ty,
                        ArgumentType::WeakFloat { .. } | ArgumentType::WeakFloatExpression { .. }
                    )
                })
            })
            .unwrap_or(&results.values[0]);
        let mut joined = candidate.0.clone();
        for (value, location) in &results.values {
            if matches!(
                &value.ty,
                ArgumentType::Known(_)
                    | ArgumentType::StringLiteral(_)
                    | ArgumentType::WeakInteger { .. }
                    | ArgumentType::WeakFloat { .. }
                    | ArgumentType::WeakFloatExpression { .. }
            ) {
                joined = if self.is_float_argument(&joined) || self.is_float_argument(value) {
                    self.common_float_argument(&joined, value, *location)?
                } else {
                    self.common_argument(joined, value.clone(), *location)?
                };
            }
        }
        let ty = self.argument_type(&joined, candidate.1)?;
        for (value, location) in &results.values {
            crate::overloads::contextual_conversion(self.types, self, ty, &value.ty, *location)?;
        }
        Ok(Some(ArgumentInfo::typed(ty)))
    }

    fn preview_lambda_statements(
        &mut self,
        statements: &[syntax::Statement],
        context: BlockContext<'_>,
        results: &mut BlockResults,
    ) -> Result<bool, Diagnostic> {
        use syntax::StatementKind as S;
        let BlockContext {
            signature,
            source_results,
            inferred,
            loops,
            deferred,
            depth,
        } = context;
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                self.span,
                "short lambda block exceeds source depth",
            ));
        }
        let mut returns = false;
        for statement in statements {
            let span = statement.span;
            if returns {
                return Err(Diagnostic::new(span, "unreachable statement"));
            }
            returns = match &statement.kind {
                S::Return(value) => {
                    if deferred {
                        return Err(Diagnostic::new(
                            span,
                            "a deferred body cannot return from its procedure",
                        ));
                    }
                    if let Some(source_results) = source_results {
                        self.preview_full_source_return(value.as_ref(), source_results, span)?;
                    } else {
                        match (inferred, signature.results.first(), value) {
                            (true, _, Some(value)) => {
                                self.validate_discarded_expression(value)?;
                                let info = self.describe_argument(value)?;
                                results.values.push((info, value.span));
                            }
                            (true, _, None) => results.void_return = true,
                            (false, Some(&ty), Some(value)) => {
                                self.check_discarded_argument(value, ty)?
                            }
                            (false, None, None) => {}
                            _ => {
                                return Err(Diagnostic::new(
                                    span,
                                    "short lambda return does not match its result type",
                                ));
                            }
                        }
                    }
                    true
                }
                S::ReturnValues(values) if source_results.is_some() => {
                    if deferred {
                        return Err(Diagnostic::new(
                            span,
                            "a deferred body cannot return from its procedure",
                        ));
                    }
                    self.preview_full_source_returns(
                        values,
                        source_results.expect("guarded source results"),
                        span,
                    )?;
                    true
                }
                S::Declare(declaration) => {
                    let (name, ty) = self.preview_lambda_declaration(declaration, span)?;
                    self.preview_lambda_bind(
                        name,
                        Binding::LambdaPreview(PreviewBinding::Parameter(ty)),
                        span,
                    )?;
                    false
                }
                S::Constant(constant) => {
                    self.validate_discarded_expression(&constant.initializer)?;
                    let info = self.describe_argument(&constant.initializer)?;
                    let ty = match &constant.ty {
                        Some(annotation) => {
                            let ty = self.preview_annotation(annotation, constant.span)?;
                            self.check_discarded_argument(&constant.initializer, ty)?;
                            ty
                        }
                        None => self.argument_type(&info, constant.initializer.span)?,
                    };
                    let binding = self.preview_lambda_constant(
                        &constant.initializer,
                        &info,
                        ty,
                        constant.ty.is_some(),
                        constant.span,
                    )?;
                    self.preview_lambda_bind(constant.name, binding, constant.span)?;
                    false
                }
                S::Assign(name, value) | S::Update(name, _, value) => {
                    let target = syntax::Expression {
                        kind: syntax::ExpressionKind::Name(*name),
                        span,
                    };
                    let ty = self.preview_lambda_assignment_type(&target)?;
                    self.check_discarded_argument(value, ty)?;
                    if let S::Update(_, operation, _) = &statement.kind {
                        self.preview_lambda_update(&target, *operation, value, span)?;
                    }
                    false
                }
                S::AssignPlace {
                    target,
                    value,
                }
                | S::UpdatePlace {
                    target,
                    value,
                    ..
                } => {
                    let target = Self::preview_lambda_place_expression(target)?;
                    let ty = self.preview_lambda_assignment_type(&target)?;
                    self.check_discarded_argument(value, ty)?;
                    if let S::UpdatePlace {
                        operation, ..
                    } = &statement.kind
                    {
                        self.preview_lambda_update(&target, *operation, value, span)?;
                    }
                    false
                }
                S::If(condition, yes, no) => {
                    self.preview_lambda_condition(condition)?;
                    let yes = self.preview_lambda_nested(yes, context, results)?;
                    let no = self.preview_lambda_nested(no, context, results)?;
                    yes && no
                }
                S::Block(body) => self.preview_lambda_nested(body, context, results)?,
                S::CheckScope {
                    checks,
                    body,
                } => {
                    let previous = self.checks;
                    self.checks = previous.overridden(*checks);
                    let checked = self.preview_lambda_nested(body, context, results);
                    self.checks = previous;
                    checked?
                }
                S::Defer(body) => {
                    self.preview_lambda_nested(
                        body,
                        BlockContext {
                            deferred: true,
                            loops: 0,
                            ..context
                        },
                        results,
                    )?;
                    false
                }
                S::While(condition, body) => {
                    self.scopes.push(HashMap::new());
                    let checked = (|| {
                        match condition {
                            syntax::WhileCondition::Expression(value) => {
                                self.preview_lambda_condition(value)?
                            }
                            syntax::WhileCondition::Binding {
                                name,
                                export_span,
                                initializer,
                            } => {
                                if let Some(span) = export_span {
                                    return Err(Diagnostic::new(
                                        *span,
                                        "caller exports require checked macro expansion",
                                    ));
                                }
                                self.preview_lambda_condition(initializer)?;
                                let info = self.describe_argument(initializer)?;
                                let ty = self.argument_type(&info, initializer.span)?;
                                self.preview_lambda_bind(
                                    *name,
                                    Binding::LambdaPreview(PreviewBinding::Parameter(ty)),
                                    span,
                                )?;
                            }
                        }
                        self.preview_lambda_statements(
                            body,
                            BlockContext {
                                loops: loops + 1,
                                depth: depth + 1,
                                ..context
                            },
                            results,
                        )
                    })();
                    self.scopes.pop();
                    checked?;
                    false
                }
                S::Jump {
                    kind: syntax::JumpKind::Break | syntax::JumpKind::Continue,
                    target: syntax::LoopTarget::Innermost,
                    ..
                } if loops > 0 => {
                    // The enclosing loop may complete; this only terminates its source body.
                    true
                }
                S::Expression(value) => {
                    self.validate_discarded_expression(value)?;
                    self.preview_short_lambda_void_body(value)?;
                    false
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "short lambda statement requires a pure source preview rule",
                    ));
                }
            };
        }
        Ok(returns)
    }

    fn preview_lambda_nested(
        &mut self,
        body: &[syntax::Statement],
        context: BlockContext<'_>,
        results: &mut BlockResults,
    ) -> Result<bool, Diagnostic> {
        self.scopes.push(HashMap::new());
        let checked = self.preview_lambda_statements(
            body,
            BlockContext {
                depth: context.depth + 1,
                ..context
            },
            results,
        );
        self.scopes.pop();
        checked
    }

    fn preview_lambda_bind(
        &mut self,
        name: Symbol,
        binding: Binding,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if self
            .scopes
            .last_mut()
            .expect("lambda preview frame")
            .insert(name, binding)
            .is_some()
        {
            return Err(Diagnostic::new(
                span,
                "duplicate declaration in short lambda block",
            ));
        }
        Ok(())
    }

    fn preview_lambda_declaration(
        &mut self,
        declaration: &syntax::Declaration,
        span: Span,
    ) -> Result<(Symbol, TypeId), Diagnostic> {
        let (name, annotation, initializer) = match declaration {
            syntax::Declaration::GroupMember {
                ..
            } => {
                return Err(Diagnostic::new(
                    self.span,
                    "file declaration group requires its source publication owner",
                ));
            }
            syntax::Declaration::Inferred {
                name,
                initializer,
                ..
            } => (*name, None, Some(initializer)),
            syntax::Declaration::Explicit {
                name,
                ty,
                initializer,
                ..
            } => (*name, Some(self.types.scalar(*ty)), initializer.as_ref()),
            syntax::Declaration::UnresolvedExplicit {
                name,
                ty,
                initializer,
                ..
            } => (
                *name,
                Some(self.preview_annotation(ty, span)?),
                initializer.as_ref(),
            ),
            syntax::Declaration::External {
                ..
            } => {
                return Err(Diagnostic::new(
                    span,
                    "external declaration requires its checked source binding before lambda preview",
                ));
            }
        };
        let ty = match (annotation, initializer) {
            (Some(ty), Some(value))
                if !matches!(value.kind, syntax::ExpressionKind::Uninitialized) =>
            {
                self.check_discarded_argument(value, ty)?;
                ty
            }
            (Some(ty), _) => ty,
            (None, Some(value)) => {
                self.validate_discarded_expression(value)?;
                let info = self.describe_argument(value)?;
                self.argument_type(&info, value.span)?
            }
            (None, None) => unreachable!("inferred source declaration has an initializer"),
        };
        Ok((name, ty))
    }

    fn preview_lambda_condition(&mut self, value: &syntax::Expression) -> Result<(), Diagnostic> {
        self.validate_discarded_expression(value)?;
        let info = self.describe_argument(value)?;
        self.check_discarded_condition(&info, value.span)
    }

    fn preview_lambda_assignment_type(
        &mut self,
        target: &syntax::Expression,
    ) -> Result<TypeId, Diagnostic> {
        self.preview_lambda_mutable_place(target)?;
        self.validate_discarded_expression(target)?;
        let info = self.describe_argument(target)?;
        self.argument_type(&info, target.span)
    }

    fn preview_lambda_mutable_place(
        &mut self,
        target: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        use syntax::ExpressionKind as E;
        let mutable = match &target.kind {
            E::Name(name) => matches!(
                self.lookup_path(
                    &syntax::NamePath {
                        root: *name,
                        members: vec![]
                    },
                    target.span,
                )?,
                Binding::Storage(_) | Binding::LambdaPreview(PreviewBinding::Parameter(_))
            ),
            E::QualifiedName(path) => {
                matches!(self.lookup_path(path, target.span)?, Binding::Storage(_))
            }
            E::Member {
                base, ..
            } => {
                self.preview_lambda_mutable_place(base)?;
                true
            }
            E::Index {
                base, ..
            } => {
                let description = self.describe_argument(base)?;
                let ty = self.argument_type(&description, base.span)?;
                if !matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))) {
                    self.preview_lambda_mutable_place(base)?;
                }
                true
            }
            E::Dereference(_) => true,
            E::Context => self.context_available,
            _ => false,
        };
        if mutable {
            Ok(())
        } else {
            Err(Diagnostic::new(
                target.span,
                "short lambda assignment target is not mutable storage",
            ))
        }
    }

    fn preview_lambda_update(
        &mut self,
        target: &syntax::Expression,
        operation: syntax::BinaryOp,
        value: &syntax::Expression,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let update = syntax::Expression {
            kind: syntax::ExpressionKind::Binary(
                operation,
                Box::new(target.clone()),
                Box::new(value.clone()),
            ),
            span,
        };
        self.validate_discarded_expression(&update)
    }

    fn preview_lambda_place_expression(
        place: &syntax::PlaceSyntax,
    ) -> Result<syntax::Expression, Diagnostic> {
        let kind = match &place.kind {
            syntax::PlaceKind::Name(name) => syntax::ExpressionKind::Name(*name),
            syntax::PlaceKind::Qualified(path) => {
                syntax::ExpressionKind::QualifiedName(path.clone())
            }
            syntax::PlaceKind::Member {
                base,
                member,
            } => syntax::ExpressionKind::Member {
                base: base.clone(),
                member: *member,
            },
            syntax::PlaceKind::Index {
                base,
                index,
            } => syntax::ExpressionKind::Index {
                base: base.clone(),
                index: index.clone(),
            },
            syntax::PlaceKind::Dereference(value) => {
                syntax::ExpressionKind::Dereference(value.clone())
            }
            syntax::PlaceKind::Insert(_) => {
                return Err(Diagnostic::new(
                    place.span,
                    "inserted lambda assignment requires its checked source recipe",
                ));
            }
        };
        Ok(syntax::Expression {
            kind,
            span: place.span,
        })
    }

    fn preview_lambda_constant(
        &self,
        source: &syntax::Expression,
        value: &ArgumentInfo,
        ty: TypeId,
        annotated: bool,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        use crate::polymorphism::BakedValue;
        let lookup = |path: &syntax::NamePath, span| match self.lookup_path(path, span)? {
            Binding::Constant(value) => Ok(value),
            _ => Err(Diagnostic::new(
                span,
                "lambda block constant requires immutable scalar facts",
            )),
        };
        if annotated && let Ok(TypeKind::Float(target)) = self.types.kind(ty) {
            let value = jai_eval::floats::evaluate_float_paths_with_overflow_check(
                source,
                *target,
                self.checks.arithmetic_overflow,
                lookup,
            )?;
            return Ok(Binding::Constant(ScalarConstant::Float(value)));
        }
        let scalar = match &value.constant {
            Some(ConstantArgument::IntegerLiteral(value)) if !annotated => {
                ScalarConstant::Literal(*value)
            }
            Some(ConstantArgument::IntegerLiteral(value))
                if matches!(self.types.kind(ty), Ok(TypeKind::Integer(_))) =>
            {
                let TypeKind::Integer(integer) = self.types.kind(ty).expect("checked integer type")
                else {
                    unreachable!()
                };
                ScalarConstant::Int(jai_types::Integer::checked(*integer, *value).ok_or_else(
                    || Diagnostic::new(span, "lambda block constant is outside its integer type"),
                )?)
            }
            Some(ConstantArgument::Value(BakedValue::Value(jai_ir::ConstantValue {
                kind: jai_ir::ConstantKind::Int(value),
                ..
            }))) => ScalarConstant::Int(*value),
            Some(ConstantArgument::Value(BakedValue::Value(jai_ir::ConstantValue {
                kind: jai_ir::ConstantKind::Bool(value),
                ..
            }))) => ScalarConstant::Bool(*value),
            Some(ConstantArgument::Value(BakedValue::Float(value))) => {
                ScalarConstant::Float(*value)
            }
            Some(
                ConstantArgument::FloatLiteral {
                    ..
                }
                | ConstantArgument::FloatExpression {
                    ..
                },
            ) => jai_eval::evaluate_paths_with_overflow_check(
                source,
                self.checks.arithmetic_overflow,
                lookup,
            )?,
            Some(ConstantArgument::Value(BakedValue::Type(ty))) => return Ok(Binding::Type(*ty)),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "lambda block constant requires checked immutable scalar or type facts",
                ));
            }
        };
        let scalar = if annotated {
            match self.types.kind(ty) {
                Ok(TypeKind::Integer(integer)) => scalar.coerce(ScalarType::Int(*integer), span)?,
                Ok(TypeKind::Bool) => scalar.coerce(ScalarType::Bool, span)?,
                _ => scalar,
            }
        } else {
            scalar
        };
        Ok(Binding::Constant(scalar))
    }
    fn preview_full_source_return(
        &mut self,
        value: Option<&syntax::Expression>,
        results: &[ResultSignature],
        span: Span,
    ) -> Result<(), Diagnostic> {
        match (value, results) {
            (None, results) if results.iter().all(|result| result.name.is_some()) => Ok(()),
            (Some(value), [result]) => self.check_discarded_argument(value, result.ty),
            (Some(value), results) if !results.is_empty() => {
                let path = match &value.kind {
                    syntax::ExpressionKind::Call(name, args) => (
                        syntax::NamePath {
                            root: *name,
                            members: Vec::new(),
                        },
                        args,
                    ),
                    syntax::ExpressionKind::QualifiedCall(path, args) => (path.clone(), args),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "multiple source results require a ready pure call preview",
                        ));
                    }
                };
                let types = self.describe_bound_call_results(
                    &path.0,
                    path.1,
                    &vec![true; results.len()],
                    value.span,
                )?;
                if types.len() != results.len()
                    || types
                        .iter()
                        .zip(results)
                        .any(|(ty, result)| *ty != result.ty)
                {
                    return Err(Diagnostic::new(
                        span,
                        "source return does not match checked result types",
                    ));
                }
                Ok(())
            }
            _ => Err(Diagnostic::new(
                span,
                "source return does not match checked result types",
            )),
        }
    }
    fn preview_full_source_returns(
        &mut self,
        values: &[syntax::ReturnValue],
        results: &[ResultSignature],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut bound = vec![false; results.len()];
        let mut positional = 0;
        let mut named = false;
        for value in values {
            let index = if let Some(name) = value.name {
                named = true;
                results
                    .iter()
                    .position(|result| result.name == Some(name))
                    .ok_or_else(|| Diagnostic::new(value.value.span, "unknown named return"))?
            } else {
                if named {
                    return Err(Diagnostic::new(
                        value.value.span,
                        "positional return cannot follow a named return",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let result = results
                .get(index)
                .ok_or_else(|| Diagnostic::new(value.value.span, "too many return values"))?;
            if bound[index] {
                return Err(Diagnostic::new(
                    value.value.span,
                    "duplicate return for result",
                ));
            }
            bound[index] = true;
            self.check_discarded_argument(&value.value, result.ty)?;
        }
        if bound
            .iter()
            .zip(results)
            .any(|(bound, result)| !bound && result.name.is_none())
        {
            return Err(Diagnostic::new(span, "missing unnamed return value"));
        }
        Ok(())
    }
}
