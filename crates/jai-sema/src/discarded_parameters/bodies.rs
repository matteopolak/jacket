//! Anonymous discarded runs check source statements using typed facts, without IR.
use super::*;

#[derive(Clone, Copy)]
enum PureReturn {
    Procedure(Option<TypeId>),
    Deferred,
}

impl Resolver<'_> {
    pub(super) fn check_discarded_run_body(
        &mut self,
        result: &syntax::TypeSyntax,
        body: &[syntax::Statement],
        span: Span,
    ) -> Result<(), Diagnostic> {
        let ty = self.preview_annotation(result, span)?;
        let result = if ty == self.types.void() {
            None
        } else {
            Some(ty)
        };
        let saved = self.scopes.clone();
        let mut preview = saved.clone();
        for scope in &mut preview {
            for binding in scope.values_mut() {
                match binding {
                    Binding::Storage(storage)
                        if self.short_lambda_captures_storage(*storage, span)? =>
                    {
                        *binding = Binding::LambdaPreview(
                            crate::short_lambdas::PreviewBinding::RuntimeCapture,
                        );
                    }
                    Binding::LambdaPreview(crate::short_lambdas::PreviewBinding::Parameter(_)) => {
                        *binding = Binding::LambdaPreview(
                            crate::short_lambdas::PreviewBinding::RuntimeCapture,
                        );
                    }
                    _ => {}
                }
            }
        }
        preview.push(HashMap::new());
        self.scopes = preview;
        let saved_context = self.context_available;
        self.context_available = true;
        let checked = self.check_discarded_statements(body, PureReturn::Procedure(result), 0);
        self.context_available = saved_context;
        self.scopes = saved;
        let returns = checked?;
        if result.is_some() && !returns {
            return Err(Diagnostic::new(
                span,
                "anonymous discarded #run can complete without a result",
            ));
        }
        Ok(())
    }
    fn check_discarded_statements(
        &mut self,
        body: &[syntax::Statement],
        result: PureReturn,
        depth: usize,
    ) -> Result<bool, Diagnostic> {
        use syntax::StatementKind as S;
        if depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(Diagnostic::new(
                self.span,
                "discarded #run body exceeds source depth",
            ));
        }
        let mut returns = false;
        for statement in body {
            let span = statement.span;
            match &statement.kind {
                S::Return(value) => {
                    match (result, value) {
                        (PureReturn::Procedure(Some(ty)), Some(value)) => {
                            self.check_discarded_argument(value, ty)?
                        }
                        (PureReturn::Procedure(None), None) => {}
                        (PureReturn::Deferred, _) => {
                            return Err(Diagnostic::new(
                                span,
                                "a deferred body cannot return from its procedure",
                            ));
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                span,
                                "anonymous discarded #run return does not match its result type",
                            ));
                        }
                    }
                    returns = true;
                }
                S::Declare(declaration) => {
                    let (name, ty) = match declaration {
                        syntax::Declaration::Inferred {
                            name, initializer, ..
                        } => {
                            self.validate_discarded_expression(initializer)?;
                            let info = self.describe_argument(initializer)?;
                            (*name, self.argument_type(&info, initializer.span)?)
                        }
                        syntax::Declaration::Explicit {
                            name,
                            ty,
                            initializer,
                            ..
                        } => {
                            let ty = self.types.scalar(*ty);
                            if let Some(value) = initializer {
                                self.check_discarded_argument(value, ty)?;
                            }
                            (*name, ty)
                        }
                        syntax::Declaration::UnresolvedExplicit {
                            name,
                            ty,
                            initializer,
                            ..
                        } => {
                            let ty = self.preview_annotation(ty, span)?;
                            if let Some(value) = initializer {
                                self.check_discarded_argument(value, ty)?;
                            }
                            (*name, ty)
                        }
                        syntax::Declaration::External { name, ty, .. } => {
                            if self.symbols.name(*name) == "_" {
                                return Err(Diagnostic::new(
                                    span,
                                    "external data requires a named source declaration",
                                ));
                            }
                            self.preview_annotation(ty, span)?;
                            return Err(Diagnostic::new(
                                span,
                                "external data declaration is pending checked provider metadata in discarded #run body",
                            ));
                        }
                    };
                    let scope = self.scopes.last_mut().expect("pure body scope");
                    if scope
                        .insert(
                            name,
                            Binding::LambdaPreview(
                                crate::short_lambdas::PreviewBinding::Parameter(ty),
                            ),
                        )
                        .is_some()
                    {
                        return Err(Diagnostic::new(
                            span,
                            "duplicate declaration in discarded #run body",
                        ));
                    }
                }
                S::Assign(name, value) | S::Update(name, _, value) => {
                    let path = syntax::NamePath {
                        root: *name,
                        members: vec![],
                    };
                    let binding = self.lookup_path(&path, span)?;
                    let ty = match binding {
                        Binding::Storage(storage) => storage.place().ty(),
                        Binding::LambdaPreview(
                            crate::short_lambdas::PreviewBinding::Parameter(ty),
                        ) => ty,
                        Binding::Discarded(_) => {
                            return Err(Diagnostic::new(
                                span,
                                "#discard parameter cannot be assigned",
                            ));
                        }
                        Binding::LambdaPreview(
                            crate::short_lambdas::PreviewBinding::RuntimeCapture,
                        ) => {
                            return Err(Diagnostic::new(
                                span,
                                "anonymous #run cannot capture runtime local storage",
                            ));
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                span,
                                "discarded #run assignment target is not storage",
                            ));
                        }
                    };
                    self.check_discarded_argument(value, ty)?;
                    if let S::Update(_, operation, _) = &statement.kind {
                        let target = syntax::Expression {
                            kind: syntax::ExpressionKind::Name(*name),
                            span,
                        };
                        let update = syntax::Expression {
                            kind: syntax::ExpressionKind::Binary(
                                *operation,
                                Box::new(target),
                                Box::new(value.clone()),
                            ),
                            span,
                        };
                        self.validate_discarded_expression(&update)?;
                    }
                }
                S::If(condition, yes, no) => {
                    self.validate_discarded_expression(condition)?;
                    let condition = self.describe_argument(condition)?;
                    self.check_discarded_condition(&condition, span)?;
                    self.scopes.push(HashMap::new());
                    let yes = self.check_discarded_statements(yes, result, depth + 1);
                    self.scopes.pop();
                    self.scopes.push(HashMap::new());
                    let no = self.check_discarded_statements(no, result, depth + 1);
                    self.scopes.pop();
                    returns |= yes? && no?;
                }
                S::Block(body) | S::CheckScope { body, .. } => {
                    self.scopes.push(HashMap::new());
                    let checked = self.check_discarded_statements(body, result, depth + 1);
                    self.scopes.pop();
                    returns |= checked?;
                }
                S::Defer(body) => {
                    self.scopes.push(HashMap::new());
                    let checked =
                        self.check_discarded_statements(body, PureReturn::Deferred, depth + 1);
                    self.scopes.pop();
                    checked?;
                }
                S::Expression(value) => {
                    self.validate_discarded_expression(value)?;
                    match &value.kind {
                        syntax::ExpressionKind::Call(name, args) => {
                            self.describe_discarded_call_results(
                                &syntax::NamePath {
                                    root: *name,
                                    members: vec![],
                                },
                                args,
                                span,
                            )?;
                        }
                        syntax::ExpressionKind::QualifiedCall(path, args) => {
                            self.describe_discarded_call_results(path, args, span)?;
                        }
                        _ => {
                            self.describe_argument(value)?;
                        }
                    }
                }
                _ => {
                    return Err(Diagnostic::new(
                        span,
                        "discarded anonymous #run statement is pending a pure source-checking rule",
                    ));
                }
            }
        }
        Ok(returns)
    }
}
