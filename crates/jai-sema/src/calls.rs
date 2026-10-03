//! Bind source-ordered arguments to declaration-ordered scalar parameters.
use super::*;

pub(super) fn parameters(
    procedure: &syntax::Procedure,
    globals: &HashMap<Symbol, Binding>,
    types: &TypeRegistry,
) -> Result<Vec<ParameterSignature>, Diagnostic> {
    parameters_paths(procedure, types, |path, span| {
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "qualified default requires a module scope",
            ));
        }
        match globals.get(&path.root) {
            Some(Binding::Constant(value)) => Ok(value.clone()),
            _ => Err(Diagnostic::new(
                span,
                "parameter default requires a compile-time constant",
            )),
        }
    })
}
pub(super) fn parameters_paths(
    procedure: &syntax::Procedure,
    types: &TypeRegistry,
    mut lookup: impl FnMut(&syntax::NamePath, Span) -> Result<ScalarConstant, Diagnostic>,
) -> Result<Vec<ParameterSignature>, Diagnostic> {
    let mut names = std::collections::HashSet::new();
    procedure
        .parameters
        .iter()
        .map(|parameter| {
            if !names.insert(parameter.name) {
                return Err(Diagnostic::new(procedure.span, "duplicate parameter name"));
            }
            let (ty, default) = match &parameter.binding {
                syntax::ParameterBinding::Required(ty) => (*ty, None),
                syntax::ParameterBinding::Defaulted { ty, expression } => {
                    let value = jai_eval::evaluate_paths(expression, &mut lookup)?;
                    let ty = match ty {
                        Some(ty) => *ty,
                        None => value.scalar_type().ok_or_else(|| {
                            Diagnostic::new(
                                parameter.span,
                                "float parameter defaults require graph type resolution",
                            )
                        })?,
                    };
                    (ty, Some(value.coerce(ty, expression.span)?))
                }
                _ => {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "single-file scalar resolver requires scalar parameters",
                    ));
                }
            };
            let ty = types.scalar(ty);
            Ok(ParameterSignature {
                name: parameter.name,
                ty,
                evaluation: parameter.evaluation,
                default: if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                    default.map(|_| ParameterDefault::Discarded)
                } else {
                    default
                        .map(|value| {
                            modules::aggregates::scalar_constant(ty, value, types, parameter.span)
                                .map(ParameterDefault::Constant)
                        })
                        .transpose()?
                },
            })
        })
        .collect()
}

impl Resolver<'_> {
    pub(super) fn resolve_call(
        &mut self,
        name: Symbol,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        self.resolve_call_path(
            &syntax::NamePath {
                root: name,
                members: Vec::new(),
            },
            args,
            span,
        )
    }
    pub(super) fn resolve_call_path(
        &mut self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        if let Some(query) = self.constant_query_call(path, args, span)? {
            return Ok(query);
        }
        if let Some(call) = self.named_short_lambda_call(path, args, span)? {
            return Ok(call);
        }
        self.reject_value_expansion(path, span)?;
        if self.call_is_indirect(path, span) {
            let baked = self.baked_callable_type(path).is_some();
            if baked {
                self.checked_baked_callback_binding_contract(path, span)?
                    .ok_or_else(|| {
                        Diagnostic::new(span, "baked callback has no checked source contract")
                    })?;
            }
            let callee = self.path_expression(path, span)?;
            if baked {
                let source = syntax::Expression {
                    kind: if path.members.is_empty() {
                        syntax::ExpressionKind::Name(path.root)
                    } else {
                        syntax::ExpressionKind::QualifiedName(path.clone())
                    },
                    span,
                };
                return self.indirect_call_from_source(callee, args, span, Some(&source));
            }
            return self.indirect_call(callee, args, span);
        }
        let (signature, call) = self.resolve_call_binding(path, args, span)?;
        match signature.results.as_slice() {
            [] => Ok(Expr::Void(call)),
            [result] => self.typed_value(
                ValueExpr::Call {
                    ty: result.ty,
                    call,
                },
                result.ty,
                span,
            ),
            _ => Err(Diagnostic::new(
                span,
                "multiple procedure results require result binding",
            )),
        }
    }

    pub(crate) fn baked_callable_type(&self, path: &syntax::NamePath) -> Option<TypeId> {
        if !path.members.is_empty() || self.local_name_present(path.root) {
            return None;
        }
        let crate::polymorphism::BakedValue::Value(value) = self.graph_scope?.baked_value(path)?
        else {
            return None;
        };
        self.types.procedure_definition(value.ty).ok()?;
        Some(value.ty)
    }

    pub(crate) fn call_is_indirect(&mut self, path: &syntax::NamePath, span: Span) -> bool {
        if self
            .local_callable_signature(path, span)
            .is_ok_and(|signature| signature.is_some())
        {
            return false;
        }
        if self.baked_callable_type(path).is_some() {
            return true;
        }
        if self
            .lexical_imported_value_root(path, span)
            .is_ok_and(|value| {
                value.is_some_and(|(binding, _)| {
                    matches!(
                        binding,
                        Binding::Storage(_)
                            | Binding::TypedConstant(_)
                            | Binding::Constant(_)
                            | Binding::Enum(_)
                    )
                })
            })
        {
            return true;
        }
        if self.lexical_graph_binding(path, span).is_ok_and(|binding| {
            binding.is_some_and(|binding| {
                self.graph_scope
                    .is_some_and(|scope| scope.imported_callable(binding, span).is_ok())
            })
        }) {
            return false;
        }
        self.scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(&path.root))
            || self
                .graph_scope
                .is_some_and(|scope| scope.value_root(path, span).is_ok())
            || (path.members.is_empty() && self.globals.contains_key(&path.root))
    }

    pub(super) fn resolve_call_binding(
        &mut self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(Signature, Call), Diagnostic> {
        self.resolve_call_binding_with_defaults(path, args, span)
            .map(|(signature, call, _)| (signature, call))
    }

    pub(crate) fn resolve_call_binding_with_defaults(
        &mut self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<crate::runtime_defaults::DirectCallWithDefaults, Diagnostic> {
        self.resolve_local_name(path.root, span)?;
        if let Some(signature) = self.local_callable_signature(path, span)? {
            let (call, reads) =
                self.bind_signature_arguments_with_defaults(&signature, args, span)?;
            return Ok((signature, call, reads));
        }
        if let Some(binding) = self.lexical_graph_binding(path, span)?
            && self
                .graph_scope
                .is_some_and(|scope| scope.imported_callable(binding, span).is_ok())
            && let Some(result) =
                self.resolve_overloaded_call_binding_with_defaults(path, args, span)
        {
            return result;
        }
        if self
            .scopes
            .iter()
            .rev()
            .any(|scope| scope.contains_key(&path.root))
        {
            return Err(Diagnostic::new(
                span,
                if path.members.is_empty() {
                    "scalar value is not a procedure"
                } else {
                    "scalar value is not a namespace"
                },
            ));
        }
        if let Some(result) = self.resolve_overloaded_call_binding_with_defaults(path, args, span) {
            return result;
        }
        let signature = if let Some(scope) = self.graph_scope {
            scope.signature(path, span)?
        } else {
            if !path.members.is_empty() {
                return Err(Diagnostic::new(
                    span,
                    "qualified call requires a module scope",
                ));
            }
            if self.globals.contains_key(&path.root) {
                return Err(Diagnostic::new(span, "scalar value is not a procedure"));
            }
            self.signatures.get(&path.root).ok_or_else(|| {
                Diagnostic::new(
                    span,
                    format!("unknown procedure '{}'", self.symbols.name(path.root)),
                )
            })?
        };
        let signature = signature.clone();
        let (call, reads) = self.bind_signature_arguments_with_defaults(&signature, args, span)?;
        Ok((signature, call, reads))
    }

    pub(crate) fn bind_signature_arguments(
        &mut self,
        signature: &Signature,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Call, Diagnostic> {
        self.bind_signature_arguments_with_defaults(signature, args, span)
            .map(|(call, _)| call)
    }

    pub(crate) fn bind_signature_arguments_with_defaults(
        &mut self,
        signature: &Signature,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<(Call, crate::runtime_defaults::RuntimeDefaultBindings), Diagnostic> {
        let completed = self.complete_callable_signature(signature, span)?;
        let signature = &completed;
        if let Some(bound) = self.bind_compiler_code_arguments(signature, args, span)? {
            self.warn_deprecated_procedure(signature.id, span)?;
            return Ok(bound);
        }
        let descriptor = self
            .types
            .procedure_definition(signature.ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?
            .clone();
        self.check_call_context(&descriptor, span)?;
        let metadata = crate::procedure_values::bindings::CallbackSignature::source(signature);
        let (arguments, reads) =
            self.bind_callable_arguments_with_defaults(&descriptor, Some(&metadata), args, span)?;
        self.warn_deprecated_procedure(signature.id, span)?;
        Ok((Call::new(signature.id, arguments), reads))
    }
}
