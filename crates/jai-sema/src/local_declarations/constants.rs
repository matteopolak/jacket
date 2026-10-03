//! Resolve declarative values through their lexical, rather than graph, environment.
use super::*;

impl Resolver<'_> {
    pub(super) fn local_scalar_path(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<ScalarConstant, Diagnostic> {
        if let Some(binding) = self.namespace_binding(path, span)? {
            return self.local_scalar_binding(binding, span);
        }
        if let Some(binding) = self.resolve_local_name(path.root, span)? {
            if !path.members.is_empty() {
                return self
                    .local_enum_member(path, span)?
                    .map(|member| ScalarConstant::Int(member.value))
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "local declaration is not a scalar constant namespace",
                        )
                    });
            }
            return self.local_scalar_binding(binding, span);
        }
        self.reject_lexical_capture(path.root, span)?;
        let binding = self.lookup_path(path, span)?;
        self.local_scalar_binding(binding, span)
    }

    fn local_scalar_binding(
        &self,
        binding: Binding,
        span: Span,
    ) -> Result<ScalarConstant, Diagnostic> {
        match binding {
            Binding::Imported(binding) => {
                self.local_scalar_binding(self.imported_binding_value_ready(binding, span)?, span)
            }
            Binding::Constant(value) => Ok(value),
            Binding::Enum(value) => Ok(ScalarConstant::Int(value.value)),
            Binding::TypedConstant(id) => match self.meta.constant(id).map(|value| &value.kind) {
                Some(ConstantKind::Int(value)) => Ok(ScalarConstant::Int(*value)),
                Some(ConstantKind::Bool(value)) => Ok(ScalarConstant::Bool(*value)),
                Some(ConstantKind::Float(value)) => Ok(ScalarConstant::Float(*value)),
                _ => Err(Diagnostic::new(
                    span,
                    "typed value is not a scalar compile-time constant",
                )),
            },
            Binding::Storage(_) => Err(Diagnostic::new(
                span,
                "runtime storage cannot supply a local constant",
            )),
            _ => Err(Diagnostic::new(
                span,
                "declaration is not a scalar compile-time constant",
            )),
        }
    }

    pub(crate) fn local_scalar_expression(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<ScalarConstant, Diagnostic> {
        jai_eval::evaluate_paths_with_overflow_check(
            expression,
            self.checks.arithmetic_overflow,
            |path, span| self.local_scalar_path(path, span),
        )
    }

    pub(crate) fn local_integer_count(
        &mut self,
        expression: &syntax::Expression,
    ) -> Result<u64, Diagnostic> {
        let value = self.local_scalar_expression(expression).or_else(|error| {
            if !reflection::is_semantic_constant(expression) {
                return Err(error);
            }
            match self.expr(expression)? {
                Expr::Literal(value) => Ok(ScalarConstant::Literal(value)),
                Expr::Int(value) => match value.kind() {
                    IntExprKind::Constant(value) => Ok(ScalarConstant::Int(*value)),
                    _ => Err(Diagnostic::new(
                        expression.span,
                        "array count requires a compile-time integer constant",
                    )),
                },
                _ => Err(Diagnostic::new(
                    expression.span,
                    "array count requires a compile-time integer constant",
                )),
            }
        })?;
        let integer = match value {
            ScalarConstant::Literal(value) => Some(value),
            ScalarConstant::Int(value) => Some(value.value()),
            _ => None,
        };
        integer
            .and_then(|value| u64::try_from(value).ok())
            .ok_or_else(|| {
                Diagnostic::new(
                    expression.span,
                    "array count requires a nonnegative integer constant",
                )
            })
    }

    pub(super) fn local_constant_binding(
        &mut self,
        constant: &syntax::ConstantDeclaration,
    ) -> Result<Binding, Diagnostic> {
        if self.bind_baked_source_constant(constant)?.is_some() {
            return self
                .scopes
                .last()
                .and_then(|scope| scope.get(&constant.name))
                .cloned()
                .ok_or_else(|| {
                    Diagnostic::new(
                        constant.span,
                        "partial constant did not publish its actual declaration",
                    )
                });
        }
        if let Some(annotation) = &constant.ty {
            let ty = self.lexical_annotation(annotation, constant.span)?;
            if ty == self.types.code_type() {
                self.bind_semantic_constant(constant)?;
                return self
                    .scopes
                    .last()
                    .and_then(|scope| scope.get(&constant.name))
                    .cloned()
                    .ok_or_else(|| {
                        Diagnostic::new(
                            constant.span,
                            "typed Code constant did not publish its declaration",
                        )
                    });
            }
            let value = self.local_typed_constant(&constant.initializer, ty)?;
            self.annotation_value_contract(ty, annotation, constant.span)?;
            return crate::compile_time::materialized_binding(
                value,
                None,
                constant.span,
                self.meta,
            );
        }
        if constant.ty.is_none()
            && matches!(
                constant.initializer.kind,
                syntax::ExpressionKind::AddressOf(_)
            )
        {
            return match self.expr(&constant.initializer)? {
                Expr::Type(ty) => Ok(Binding::Type(ty)),
                _ => Err(Diagnostic::new(
                    constant.span,
                    "local address constants require compile-time evaluation",
                )),
            };
        }
        let path = match &constant.initializer.kind {
            syntax::ExpressionKind::Name(name) => Some(syntax::NamePath {
                root: *name,
                members: vec![],
            }),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            _ => None,
        };
        if constant.ty.is_none() {
            let application = match &constant.initializer.kind {
                syntax::ExpressionKind::Call(name, arguments) => Some((
                    syntax::NamePath {
                        root: *name,
                        members: vec![],
                    },
                    arguments,
                )),
                syntax::ExpressionKind::QualifiedCall(path, arguments) => {
                    Some((path.clone(), arguments))
                }
                _ => None,
            };
            if let Some((path, arguments)) = application
                && let Some(Expr::Type(ty)) =
                    self.try_record_application(&path, arguments, constant.initializer.span)?
            {
                return Ok(Binding::Type(ty));
            }
            if let Some(path) = path.as_ref() {
                if let Some(binding) = self.namespace_binding(path, constant.initializer.span)? {
                    return Ok(binding);
                }
                if let Some(binding) =
                    self.resolve_local_name(path.root, constant.initializer.span)?
                {
                    if path.members.is_empty() {
                        return match binding {
                            Binding::Storage(_) => Err(Diagnostic::new(
                                constant.span,
                                "runtime storage cannot supply a local constant",
                            )),
                            binding => Ok(binding),
                        };
                    }
                    if let Some(member) = self.local_enum_member(path, constant.initializer.span)? {
                        return Ok(Binding::Enum(member));
                    }
                }
                if let Some(scope) = self.graph_scope {
                    if let Ok(ty) = scope.type_name(path, constant.initializer.span) {
                        return Ok(Binding::Type(ty));
                    }
                    if let Ok(signature) = scope.signature(path, constant.initializer.span) {
                        self.meta
                            .local_declarations
                            .signatures
                            .insert(signature.id, signature.clone());
                        return Ok(Binding::Procedure {
                            procedure: signature.id,
                            ty: signature.ty,
                        });
                    }
                    if let Ok(binding @ (Binding::Enum(_) | Binding::TypedConstant(_))) =
                        scope.value(path, constant.initializer.span)
                    {
                        return Ok(binding);
                    }
                }
            }
        }
        if !reflection::is_semantic_constant(&constant.initializer) {
            if crate::compile_time_conditionals::has_contextual_member(&constant.initializer) {
                let expression = self.expr(&constant.initializer)?;
                let ty = self.expression_type(&expression, constant.initializer.span)?;
                let expression = self.coerce_value(expression, ty, constant.initializer.span)?;
                let value = self.evaluate_pure_constant(expression, constant.initializer.span)?;
                return crate::compile_time::materialized_binding(
                    value,
                    None,
                    constant.span,
                    self.meta,
                );
            }
            let value = self.local_scalar_expression(&constant.initializer)?;
            return Ok(Binding::Constant(value));
        }
        // The reflection/code subsystem owns semantic value materialization.
        // While this declaration is active, bind_name permits its one publish.
        self.bind_semantic_constant(constant)?;
        self.scopes
            .last()
            .and_then(|scope| scope.get(&constant.name))
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(
                    constant.span,
                    "semantic constant did not publish a lexical binding",
                )
            })
    }

    pub(crate) fn local_typed_constant(
        &mut self,
        expression: &syntax::Expression,
        ty: TypeId,
    ) -> Result<jai_ir::ConstantValue, Diagnostic> {
        if let TypeKind::Float(float) = *self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(expression.span, error.to_string()))?
            && !reflection::is_semantic_constant(expression)
        {
            let mut path_refused = false;
            let value = jai_eval::floats::evaluate_float_paths_with_overflow_check(
                expression,
                float,
                self.checks.arithmetic_overflow,
                |path, span| {
                    let value = self.local_scalar_path(path, span);
                    path_refused |= value.is_err();
                    value
                },
            );
            let value = match value {
                Ok(value) => value,
                Err(error) => {
                    // A typed record path may project a closed #as float field.
                    // Arithmetic and source-width errors keep the float evaluator's diagnostic.
                    if path_refused
                        && matches!(
                            expression.kind,
                            syntax::ExpressionKind::Name(_)
                                | syntax::ExpressionKind::QualifiedName(_)
                        )
                    {
                        let projected = self
                            .expr_expected(expression, ty)
                            .and_then(|value| self.coerce_value(value, ty, expression.span))
                            .and_then(|value| self.literal_constant(value, expression.span));
                        if let Ok(constant) = projected
                            && constant.ty == ty
                        {
                            return Ok(constant);
                        }
                    }
                    return Err(error);
                }
            };
            return Ok(jai_ir::ConstantValue {
                ty,
                kind: ConstantKind::Float(value),
            });
        }
        let scalar = match self
            .types
            .kind(ty)
            .map_err(|error| Diagnostic::new(expression.span, error.to_string()))?
        {
            TypeKind::Integer(integer) => Some(ScalarType::Int(*integer)),
            TypeKind::Bool => Some(ScalarType::Bool),
            _ => None,
        };
        if let Some(scalar) = scalar
            && !reflection::is_semantic_constant(expression)
        {
            let value = self
                .local_scalar_expression(expression)?
                .coerce(scalar, expression.span)?;
            return modules::aggregates::scalar_constant(ty, value, self.types, expression.span);
        }
        let value = self.expr_expected(expression, ty)?;
        let value = self.coerce_value(value, ty, expression.span)?;
        self.literal_constant(value.clone(), expression.span)
            .or_else(|error| {
                if reflection::is_semantic_constant(expression) {
                    self.evaluate_pure_constant(value, expression.span)
                } else {
                    Err(error)
                }
            })
    }
}
