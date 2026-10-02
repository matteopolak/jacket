//! Bind source-ordered arguments to declaration-ordered scalar parameters.
use super::*;

pub(super) fn parameters(
    procedure: &syntax::Procedure,
    globals: &HashMap<Symbol, Binding>,
) -> Result<Vec<ParameterSignature>, Diagnostic> {
    parameters_paths(procedure, |path, span| {
        if !path.members.is_empty() {
            return Err(Diagnostic::new(
                span,
                "qualified default requires a module scope",
            ));
        }
        match globals.get(&path.root) {
            Some(Binding::Constant(value)) => Ok(*value),
            _ => Err(Diagnostic::new(
                span,
                "parameter default requires a compile-time constant",
            )),
        }
    })
}
pub(super) fn parameters_paths(
    procedure: &syntax::Procedure,
    mut lookup: impl FnMut(&syntax::NamePath, Span) -> Result<ConstantValue, Diagnostic>,
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
                    let ty = ty.unwrap_or_else(|| value.ty());
                    (ty, Some(value.coerce(ty, expression.span)?))
                }
            };
            Ok(ParameterSignature {
                name: parameter.name,
                ty,
                default,
            })
        })
        .collect()
}

impl Resolver<'_> {
    pub(super) fn resolve_call(
        &self,
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
        &self,
        path: &syntax::NamePath,
        args: &[syntax::CallArgument],
        span: Span,
    ) -> Result<Expr, Diagnostic> {
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
        let mut bound = vec![false; signature.parameters.len()];
        let mut arguments = Vec::with_capacity(bound.len());
        let mut positional = 0;
        let mut named = false;
        for argument in args {
            let index = if let Some(name) = argument.name {
                named = true;
                signature
                    .parameters
                    .iter()
                    .position(|p| p.name == name)
                    .ok_or_else(|| Diagnostic::new(argument.value.span, "unknown named argument"))?
            } else {
                if named {
                    return Err(Diagnostic::new(
                        argument.value.span,
                        "positional argument cannot follow a named argument",
                    ));
                }
                let index = positional;
                positional += 1;
                index
            };
            let parameter = signature
                .parameters
                .get(index)
                .ok_or_else(|| Diagnostic::new(argument.value.span, "too many arguments"))?;
            if bound[index] {
                return Err(Diagnostic::new(
                    argument.value.span,
                    "duplicate argument for parameter",
                ));
            }
            bound[index] = true;
            let value = self.expr(&argument.value)?;
            arguments.push((
                ParameterId(index),
                coerce(value, parameter.ty, argument.value.span)?,
            ));
        }
        for (index, parameter) in signature.parameters.iter().enumerate() {
            if !bound[index] {
                let default = parameter.default.ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        format!(
                            "missing required argument '{}'",
                            self.symbols.name(parameter.name)
                        ),
                    )
                })?;
                arguments.push((
                    ParameterId(index),
                    coerce(Self::constant(default), parameter.ty, span)?,
                ));
            }
        }
        let call = Call {
            procedure: signature.id,
            arguments,
        };
        Ok(match signature.result {
            ReturnType::Void => Expr::Void(call),
            ReturnType::Value(ScalarType::Int(ty)) => {
                Expr::Int(IntExpr::new(ty, IntExprKind::Call(call)))
            }
            ReturnType::Value(ScalarType::Bool) => Expr::Bool(BoolExpr::Call(call)),
        })
    }
}
fn coerce(value: Expr, ty: ScalarType, span: Span) -> Result<ValueExpr, Diagnostic> {
    Ok(match ty {
        ScalarType::Int(ty) => ValueExpr::Int(value.int_as(ty, span)?),
        ScalarType::Bool => ValueExpr::Bool(value.bool(span)?),
    })
}
