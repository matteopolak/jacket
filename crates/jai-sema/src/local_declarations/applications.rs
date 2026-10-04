//! Explicit record arguments snapshot typed lexical values before leaving the caller.
use super::*;
use crate::modules::aggregates::parameterized::{LexicalTypeArgument, LexicalTypeArguments};

impl Resolver<'_> {
    pub(super) fn local_record_application(
        &mut self,
        syntax: &syntax::TypeSyntax,
        application: &syntax::TypeApplicationSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                span,
                "record template application requires a defining file scope",
            )
        })?;
        let mut template = None;
        if let syntax::TypeSyntax::Named(path) = application.base.as_ref()
            && let Some(binding) = self.resolve_local_name(path.root, span)?
        {
            template = self
                .lexical_graph_binding(path, application.span)?
                .map(|binding| scope.imported_record_template(binding, application.span))
                .transpose()?;
            if template.is_none()
                && let Binding::Type(ty) = binding
                && self
                    .local_scopes
                    .frames
                    .iter()
                    .any(|frame| frame.id.owner == LexicalScopeOwner::Record(ty))
                && let Some(record) = self.meta.record_specializations.record(ty)
                && record.shape.name == Some(path.root)
                && let Some(origin) = record.origin
                && scope.record_template_origin(path, application.span).ok() == Some(origin.0)
            {
                template = Some(origin.0);
            }
            if template.is_none() {
                return Err(Diagnostic::new(
                    span,
                    "lexical declaration does not denote a parameterized record template",
                ));
            }
        }
        let mut lexical = self.record_argument_snapshot(application)?;
        if let Some(template) = template {
            lexical
                .templates
                .insert((application.span.start, application.span.end), template);
        }
        scope.annotation_with_lexical_bindings(
            syntax,
            self.types,
            &mut self.meta.record_specializations,
            &lexical,
            span,
        )
    }

    fn record_argument_snapshot(
        &mut self,
        application: &syntax::TypeApplicationSyntax,
    ) -> Result<LexicalTypeArguments, Diagnostic> {
        let mut snapshot = LexicalTypeArguments::default();
        let mut expressions: Vec<_> = application
            .arguments
            .iter()
            .map(|argument| &argument.value)
            .collect();
        while let Some(expression) = expressions.pop() {
            use syntax::ExpressionKind as E;
            let key = (expression.span.start, expression.span.end);
            match &expression.kind {
                E::Name(root) | E::CompileVariable(root) => {
                    if let Some(binding) = self.resolve_record_argument_path(
                        &syntax::NamePath {
                            root: *root,
                            members: vec![],
                        },
                        expression.span,
                    )? {
                        snapshot.bindings.insert(key, binding);
                    }
                }
                E::QualifiedName(path) => {
                    if let Some(binding) =
                        self.resolve_record_argument_path(path, expression.span)?
                    {
                        snapshot.bindings.insert(key, binding);
                    }
                }
                E::Code(body) => {
                    let Expr::Code(code) = self.capture_code(body, expression.span)? else {
                        unreachable!();
                    };
                    snapshot.codes.insert(key, code);
                }
                E::Type(ty) => {
                    snapshot
                        .types
                        .insert(key, self.lexical_annotation(ty, expression.span)?);
                }
                E::TypeQuery {
                    ..
                }
                | E::CompileTime(_)
                | E::Cast(_, _, _)
                | E::TypeCast {
                    ..
                } => {
                    let value = self.expr(expression)?;
                    match value {
                        Expr::Type(ty) => {
                            snapshot.types.insert(key, ty);
                        }
                        Expr::Code(code) => {
                            snapshot.codes.insert(key, code);
                        }
                        value => {
                            let value = self
                                .literal_constant(value.value(expression.span)?, expression.span)?;
                            snapshot.values.insert(key, value);
                        }
                    }
                }
                E::CallHint {
                    call, ..
                } => expressions.push(call),
                E::AddressOf(inner) => {
                    if let Expr::Type(ty) = self.expr(expression)? {
                        snapshot.types.insert(key, ty);
                    } else {
                        expressions.push(inner);
                    }
                }
                E::Dereference(inner)
                | E::Unary(_, inner)
                | E::Member {
                    base: inner, ..
                } => expressions.push(inner),
                E::Binary(_, left, right)
                | E::Index {
                    base: left,
                    index: right,
                } => {
                    expressions.push(right);
                    expressions.push(left);
                }
                E::Conditional(conditional) => {
                    expressions.push(&conditional.condition);
                    if let Some(value) = conditional.explicit_then() {
                        expressions.push(value);
                    }
                    if let Some(value) = &conditional.else_value {
                        expressions.push(value);
                    }
                }
                E::Call(root, arguments) => {
                    self.resolve_record_argument_callee(
                        &syntax::NamePath {
                            root: *root,
                            members: vec![],
                        },
                        expression.span,
                        &mut snapshot,
                    )?;
                    expressions.extend(arguments.iter().map(|argument| &argument.value));
                }
                E::QualifiedCall(path, arguments) => {
                    self.resolve_record_argument_callee(path, expression.span, &mut snapshot)?;
                    expressions.extend(arguments.iter().map(|argument| &argument.value));
                }
                E::IndirectCall {
                    callee,
                    args,
                } => {
                    expressions.push(callee);
                    expressions.extend(args.iter().map(|argument| &argument.value));
                }
                E::ContextCall {
                    callee,
                    args,
                    overrides,
                } => {
                    expressions.push(callee);
                    expressions
                        .extend(args.iter().chain(overrides).map(|argument| &argument.value));
                }
                E::ArrayLiteral(array) => expressions.extend(&array.elements),
                E::StructLiteral(record) => {
                    for field in &record.fields {
                        expressions.extend(crate::modules::aggregates::promoted_literals::target_expressions::index_expressions(&field.target)?);
                        expressions.push(&field.value);
                    }
                }
                E::PositionalStructLiteral(record) => expressions.extend(&record.values),
                _ => {}
            }
        }
        for frame in &self.scopes {
            for (&name, binding) in frame {
                if let Some(binding) = self.record_argument_binding(binding) {
                    snapshot.roots.insert(name, binding);
                } else {
                    // An inner runtime declaration shadows any outer type or
                    // constant even when it cannot supply a baked argument.
                    snapshot.roots.remove(&name);
                }
            }
        }
        for (&ty, members) in &self.meta.local_declarations.record_namespaces {
            snapshot.namespaces.insert(
                ty,
                members
                    .iter()
                    .filter_map(|(&name, binding)| {
                        self.record_argument_binding(binding)
                            .map(|binding| (name, binding))
                    })
                    .collect(),
            );
        }
        for (&ty, scope) in &self.meta.local_declarations.namespace_scopes {
            if let Some(depth) = self
                .local_scopes
                .frames
                .iter()
                .position(|frame| frame.id == *scope)
            {
                snapshot.namespaces.insert(
                    ty,
                    self.scopes[depth]
                        .iter()
                        .filter_map(|(&name, binding)| {
                            self.record_argument_binding(binding)
                                .map(|binding| (name, binding))
                        })
                        .collect(),
                );
            }
        }
        for (&ty, enumeration) in &self.meta.local_declarations.enumerations {
            snapshot.namespaces.insert(
                ty,
                enumeration
                    .members
                    .iter()
                    .map(|(&name, &value)| {
                        (
                            name,
                            LexicalTypeArgument::Typed(jai_ir::ConstantValue {
                                ty,
                                kind: ConstantKind::Enum(value),
                            }),
                        )
                    })
                    .collect(),
            );
        }
        for (&ty, record) in &self.meta.local_declarations.records {
            let routes: Vec<_> = record
                .fields
                .iter()
                .filter(|field| field.syntax.conversion() == syntax::FieldConversion::Implicit)
                .map(|field| (field.id, field.ty))
                .collect();
            if !routes.is_empty() {
                snapshot.conversions.insert(ty, routes);
            }
        }
        Ok(snapshot)
    }

    fn resolve_record_argument_path(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<LexicalTypeArgument>, Diagnostic> {
        let binding = match self.namespace_binding(path, span)? {
            Some(binding) => Some(binding),
            None => self.resolve_local_name(path.root, span)?,
        };
        let Some(binding) = binding else {
            return Ok(None);
        };
        let binding = match binding {
            Binding::Imported(binding) => self.imported_binding_value(binding, span)?,
            binding => binding,
        };
        self.record_argument_binding(&binding)
            .map(Some)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "record template argument requires a lexical compile-time value",
                )
            })
    }

    fn resolve_record_argument_callee(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
        snapshot: &mut LexicalTypeArguments,
    ) -> Result<(), Diagnostic> {
        if let Some(binding) = self.lexical_graph_binding(path, span)?
            && let Some(scope) = self.graph_scope
        {
            if let Ok(template) = scope.imported_record_template(binding, span) {
                snapshot.templates.insert((span.start, span.end), template);
                return Ok(());
            }
            if scope.imported_callable(binding, span).is_ok() {
                return Ok(());
            }
        }
        self.resolve_record_argument_path(path, span).map(|_| ())
    }

    pub(super) fn record_argument_binding(&self, binding: &Binding) -> Option<LexicalTypeArgument> {
        Some(match binding {
            Binding::Type(ty) => LexicalTypeArgument::Type(*ty),
            Binding::Constant(value) => LexicalTypeArgument::Scalar(value.clone()),
            Binding::TypedConstant(id) => {
                LexicalTypeArgument::Typed(self.meta.constant(*id)?.clone())
            }
            Binding::Enum(value) => LexicalTypeArgument::Typed(jai_ir::ConstantValue {
                ty: value.ty,
                kind: ConstantKind::Enum(value.value),
            }),
            Binding::Code(code) => LexicalTypeArgument::Code(*code),
            Binding::Procedure {
                procedure,
                ty,
            } => LexicalTypeArgument::Typed(jai_ir::ConstantValue {
                ty: *ty,
                kind: ConstantKind::Procedure(*procedure),
            }),
            Binding::Imported(binding) => {
                let binding = self
                    .graph_scope?
                    .imported_value(*binding, Span::default())
                    .ok()?;
                if matches!(binding, Binding::Imported(_)) {
                    return None;
                }
                return self.record_argument_binding(&binding);
            }
            _ => return None,
        })
    }
}
