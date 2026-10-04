//! Pure parameter binding. Values live in the defining instance, never synthetic declarations.
use super::*;
use jai_eval::Value;
use jai_source::Diagnostic;
use jai_syntax::{
    BuiltinType, Expression, ExpressionKind, ImportArgument, ModuleArgumentValue, ModuleParameter,
    ModuleParameterType, ModuleParameters, TypeSyntax,
};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterId(pub(super) usize);
impl ParameterId {
    pub fn index(self) -> usize {
        self.0
    }
}
/// An enum value retains source nominal identity until sema registers its TypeId.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumParameter {
    pub declaration: DeclarationId,
    pub value: jai_types::Integer,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParameterValue {
    Scalar(Value),
    String(String),
    Enumeration(EnumParameter),
    Type(ModuleType),
    /// Import request only; the binder resolves this before publishing a parameter.
    ContextualMember(Symbol),
}
impl Hash for ParameterValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::String(s) => s.hash(state),
            Self::Enumeration(value) => {
                value.declaration.hash(state);
                value.value.ty().hash(state);
                value.value.bits().hash(state);
            }
            Self::ContextualMember(name) => name.hash(state),
            Self::Type(value) => value.hash(state),
            Self::Scalar(Value::Literal(n)) => {
                0u8.hash(state);
                n.hash(state);
            }
            Self::Scalar(Value::Int(n)) => {
                1u8.hash(state);
                n.ty().hash(state);
                n.bits().hash(state);
            }
            Self::Scalar(Value::Bool(b)) => {
                2u8.hash(state);
                b.hash(state);
            }
            Self::Scalar(Value::Float(value)) => {
                3u8.hash(state);
                value.ty().hash(state);
                value.bits().hash(state);
            }
            Self::Scalar(Value::WeakFloat(value)) => {
                4u8.hash(state);
                value.request_key().hash(state);
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct BoundParameter {
    pub name: Symbol,
    pub module: ModuleId,
    pub value: ParameterValue,
    pub location: SourceSpan,
    pub program_wide: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Argument {
    pub name: Option<Symbol>,
    pub value: ParameterValue,
}
/// Embedded entry identities never alias a provider's physical file at the diagnostic label.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ModuleSourceKey {
    pub path: PathBuf,
    pub embedded: Option<SourceId>,
}
impl From<PathBuf> for ModuleSourceKey {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            embedded: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ModuleKey {
    pub path: PathBuf,
    pub arguments: Option<Vec<Argument>>,
    pub embedded: Option<SourceId>,
}
impl ModuleKey {
    pub fn source_key(&self) -> ModuleSourceKey {
        ModuleSourceKey {
            path: self.path.clone(),
            embedded: self.embedded,
        }
    }
}

impl Builder<'_> {
    pub(super) fn parameter_diagnostic(
        &self,
        location: SourceSpan,
        diagnostic: Diagnostic,
    ) -> GraphError {
        self.located(
            SourceSpan {
                source: diagnostic.source.unwrap_or(location.source),
                span: diagnostic.span,
            },
            diagnostic.message,
        )
    }
    pub(super) fn evaluate_arguments(
        &self,
        file: FileInstanceId,
        arguments: &Option<Vec<ImportArgument>>,
        location: SourceSpan,
    ) -> Result<Option<Vec<Argument>>, GraphError> {
        arguments
            .as_ref()
            .map(|arguments| {
                arguments
                    .iter()
                    .map(|argument| {
                        Ok(Argument {
                            name: argument.name,
                            value: self.argument_value(file, &argument.value, location)?,
                        })
                    })
                    .collect()
            })
            .transpose()
    }
    pub(super) fn argument_value(
        &self,
        file: FileInstanceId,
        value: &ModuleArgumentValue,
        _location: SourceSpan,
    ) -> Result<ParameterValue, GraphError> {
        match value {
            ModuleArgumentValue::String(s) => Ok(ParameterValue::String(s.clone())),
            ModuleArgumentValue::Expression(expression) => {
                if let Some(value) = self.specialization_argument_value(file, expression)? {
                    return Ok(value);
                }
                if let Some(value) =
                    self.module_type_expression(file, expression, &mut Vec::new())?
                {
                    return Ok(ParameterValue::Type(value));
                }
                if let Some(value) = self.nominal_argument(file, expression, &mut Vec::new())? {
                    return Ok(value);
                }
                self.constant_expression(file, expression, &mut Vec::new())
                    .map(ParameterValue::Scalar)
            }
        }
    }
    pub(super) fn constant_expression(
        &self,
        file: FileInstanceId,
        expression: &Expression,
        active: &mut Vec<DeclarationId>,
    ) -> Result<Value, GraphError> {
        let expanded = self.enum_tests(file, expression)?;
        let expression = &expanded;
        if let Some(span) = unsupported_expression(expression) {
            let diagnostic = self.graph.diagnostic(
                SourceSpan {
                    source: self.graph.files[file.0].source,
                    span,
                },
                "expression requires unsupported compile-time evaluation",
            );
            let rendered = diagnostic.render(&self.graph.sources);
            return Err(GraphError::Unsupported {
                diagnostic,
                rendered,
            });
        }
        let mut dependency = None;
        let result = jai_eval::evaluate_paths(expression, |path, span| {
            let result = self.constant_path(file, path, active, span);
            result.map_err(|error| {
                dependency = Some(error);
                Diagnostic::new(span, "compile-time dependency unavailable")
            })
        });
        if let Some(error) = dependency {
            return Err(error);
        }
        result
            .map(|value| value.with_fallback_source(self.graph.files[file.0].source))
            .map_err(|diagnostic| {
                self.parameter_diagnostic(
                    SourceSpan {
                        source: self.graph.files[file.0].source,
                        span: expression.span,
                    },
                    diagnostic,
                )
            })
    }
    pub(super) fn constant_path(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        active: &mut Vec<DeclarationId>,
        span: jai_source::Span,
    ) -> Result<Value, GraphError> {
        let location = SourceSpan {
            source: self.graph.files[file.0].source,
            span,
        };
        let diagnostic = |message| {
            let diagnostic = self.graph.diagnostic(location, message);
            let rendered = diagnostic.render(&self.graph.sources);
            GraphError::Pending {
                diagnostic,
                rendered,
            }
        };

        if path.members.is_empty()
            && let Some(value) = self.graph.insertion_capture_value(file, path.root)
        {
            return match value {
                SourceCaptureValue::Scalar(value) => Ok(value.clone()),
                _ => Err(self.located(
                    location,
                    "expected scalar constant, found a non-scalar insertion capture",
                )),
            };
        }
        let binding = self.graph.lookup(file, path).map_err(|_| {
            diagnostic("constant name is not available for compile-time evaluation")
        })?;
        match binding {
            Binding::StorageMember(_) => Err(diagnostic(
                "runtime storage member requires semantic evaluation",
            )),
            Binding::SourceMember {
                declaration, ..
            } => {
                if matches!(
                    self.graph.declarations[declaration.index()].syntax.kind,
                    FileDeclarationKind::Enum(_)
                ) {
                    Err(self.located(
                        location,
                        "expected scalar constant, found a nominal enum member",
                    ))
                } else {
                    Err(diagnostic(
                        "static record member requires semantic compile-time evaluation",
                    ))
                }
            }
            Binding::Parameter(id) => match &self.graph.parameters[id.0].value {
                ParameterValue::Scalar(value) => Ok(value.clone()),
                ParameterValue::String(_)
                | ParameterValue::Enumeration(_)
                | ParameterValue::ContextualMember(_)
                | ParameterValue::Type(_) => Err(self.located(
                    location,
                    "expected scalar constant, found non-scalar module parameter",
                )),
            },
            Binding::Declaration(id) => {
                if active.contains(&id) {
                    return Err(self.located(location, "cyclic compile-time constant dependency"));
                }
                let declaration = &self.graph.declarations[id.index()];
                let FileDeclarationKind::Constant(constant) = &declaration.syntax.kind else {
                    return Err(diagnostic(
                        "declaration requires semantic compile-time evaluation",
                    ));
                };
                active.push(id);
                let result =
                    self.constant_expression(declaration.file, &constant.initializer, active);
                active.pop();
                result.and_then(|value| {
                    if let Some(annotation) = &constant.ty {
                        let ty = self.bind_module_type(
                            declaration.file,
                            annotation,
                            constant.span,
                            &mut vec![],
                        )?;
                        let value = self.coerce_module_value(
                            declaration.file,
                            &ty,
                            ParameterValue::Scalar(value),
                            declaration.location(),
                            false,
                        )?;
                        match value {
                            ParameterValue::Scalar(value) => Ok(value),
                            _ => Err(self.located(
                                declaration.location(),
                                "typed constant does not denote a scalar value",
                            )),
                        }
                    } else {
                        Ok(value)
                    }
                })
            }
            Binding::OverloadSet(_) => Err(diagnostic(
                "procedure overload requires semantic evaluation",
            )),
            Binding::Module(_) => {
                Err(self.located(location, "module namespace is not a constant value"))
            }
        }
    }
    pub(super) fn bind_parameters(
        &mut self,
        file: FileInstanceId,
        parameters: &ModuleParameters,
        instance: Option<&[Argument]>,
        program: Option<&[Argument]>,
    ) -> Result<(), GraphError> {
        self.parameter_list(
            file,
            &parameters.instance,
            instance.unwrap_or_default(),
            false,
            parameters.location,
        )?;
        self.parameter_list(
            file,
            parameters.program.as_deref().unwrap_or_default(),
            program.unwrap_or_default(),
            true,
            parameters.location,
        )
    }
    fn parameter_list(
        &mut self,
        file: FileInstanceId,
        parameters: &[ModuleParameter],
        supplied: &[Argument],
        program_wide: bool,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        let mut values = HashMap::new();
        let mut positional = 0;
        let mut saw_named = false;
        for argument in supplied {
            let name = if let Some(name) = argument.name {
                saw_named = true;
                name
            } else {
                if saw_named {
                    return Err(self.located(
                        location,
                        "positional module argument follows named argument",
                    ));
                }
                let Some(parameter) = parameters.get(positional) else {
                    return Err(self.located(location, "too many positional module arguments"));
                };
                positional += 1;
                parameter.name
            };
            if !parameters.iter().any(|p| p.name == name) {
                return Err(self.located(
                    location,
                    format!(
                        "unknown module parameter '{}'",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
            if values.insert(name, argument.value.clone()).is_some() {
                return Err(self.located(
                    location,
                    format!(
                        "duplicate module argument '{}'",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
        }
        let mut waiting: Vec<_> = parameters.iter().collect();
        while !waiting.is_empty() {
            let mut deferred = Vec::new();
            let mut first_pending = None;
            let mut progressed = false;
            for parameter in waiting {
                // A suspended file may resume after some parameters were published.
                // Preserve those IDs; equal names at another source span still collide.
                let module = self.graph.files[file.index()].module;
                if let Some(Binding::Parameter(id)) = self.graph.modules[module.index()]
                    .bindings
                    .get(&parameter.name)
                    && let Some(bound) = self.graph.parameters.get(id.index())
                    && bound.module == module
                    && bound.name == parameter.name
                    && bound.program_wide == program_wide
                    && bound.location == parameter.location
                {
                    progressed = true;
                    continue;
                }
                let result = self.parameter_value(
                    file,
                    parameter,
                    values.get(&parameter.name).cloned(),
                    program_wide,
                );
                let value = match result {
                    Ok(value) => value,
                    Err(
                        error @ GraphError::Pending {
                            ..
                        },
                    ) => {
                        first_pending.get_or_insert(error);
                        deferred.push(parameter);
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                let id = ParameterId(self.graph.parameters.len());
                self.bind(
                    file,
                    Visibility::Module,
                    parameter.name,
                    Binding::Parameter(id),
                    parameter.location,
                    false,
                )?;
                let variable = match &parameter.ty {
                    ModuleParameterType::Interface {
                        variable, ..
                    }
                    | ModuleParameterType::Unresolved(jai_syntax::TypeSyntax::Restricted {
                        variable,
                        ..
                    }) => Some(*variable),
                    _ => None,
                };
                if let Some(variable) = variable
                    && variable != parameter.name
                {
                    self.bind(
                        file,
                        Visibility::Module,
                        variable,
                        Binding::Parameter(id),
                        parameter.location,
                        false,
                    )?;
                }
                self.graph.parameters.push(BoundParameter {
                    name: parameter.name,
                    module,
                    value,
                    location: parameter.location,
                    program_wide,
                });
                progressed = true;
            }
            if !progressed {
                return Err(
                    first_pending.expect("deferred parameters require pending dependencies")
                );
            }
            waiting = deferred;
        }
        Ok(())
    }
    fn parameter_value(
        &self,
        file: FileInstanceId,
        parameter: &ModuleParameter,
        supplied: Option<ParameterValue>,
        program_wide: bool,
    ) -> Result<ParameterValue, GraphError> {
        let value = match supplied {
            Some(value) => value,
            None => match &parameter.default {
                Some(default) => self.argument_value(file, default, parameter.location)?,
                None => {
                    return Err(self.located(
                        parameter.location,
                        format!(
                            "required module parameter '{}' was not supplied",
                            self.graph.symbols.name(parameter.name)
                        ),
                    ));
                }
            },
        };
        let value = match value {
            ParameterValue::Type(ty) if program_wide => {
                ParameterValue::Type(self.rebind_program_type(file, ty))
            }
            value => value,
        };
        let value = match (&parameter.ty, value) {
            (ModuleParameterType::Inferred, value) => {
                let default = self.argument_value(
                    file,
                    parameter
                        .default
                        .as_ref()
                        .expect("inferred parameter has a default"),
                    parameter.location,
                )?;
                match (default, value) {
                    (ParameterValue::String(_), value @ ParameterValue::String(_)) => value,
                    (ParameterValue::Type(_), value @ ParameterValue::Type(_)) => value,
                    (ParameterValue::Enumeration(default), value) => self.coerce_enum(
                        file,
                        default.declaration,
                        value,
                        parameter.location,
                        program_wide,
                    )?,
                    (ParameterValue::Scalar(default), ParameterValue::Scalar(value)) => {
                        ParameterValue::Scalar(
                            match default {
                                Value::Float(default) => {
                                    coerce_float(value, default.ty(), parameter.location.span)
                                }
                                Value::WeakFloat(default) => coerce_float(
                                    value,
                                    default.default_type(),
                                    parameter.location.span,
                                ),
                                default => value.coerce(
                                    default
                                        .scalar_type()
                                        .expect("non-float default has a scalar type"),
                                    parameter.location.span,
                                ),
                            }
                            .map_err(|d| self.parameter_diagnostic(parameter.location, d))?,
                        )
                    }
                    _ => {
                        return Err(self.located(
                            parameter.location,
                            "module argument type differs from inferred parameter type",
                        ));
                    }
                }
            }
            (
                ModuleParameterType::Unresolved(jai_syntax::TypeSyntax::Restricted {
                    restriction: jai_syntax::TypeRestrictionSyntax::Nominal(constraint),
                    ..
                }),
                value @ ParameterValue::Type(_),
            ) => {
                let required =
                    self.bind_module_type(file, constraint, parameter.location.span, &mut vec![])?;
                let ParameterValue::Type(actual) = &value else {
                    unreachable!()
                };
                let response = self.semantic_parameter(
                    file,
                    parameter.location,
                    ParameterTask::CheckNominal {
                        actual: actual.clone(),
                        required,
                    },
                    "module nominal restriction requires semantic ancestry proof",
                )?;
                debug_assert!(matches!(response, ParameterResponse::NominalSatisfied));
                value
            }
            (
                ModuleParameterType::Unresolved(jai_syntax::TypeSyntax::Restricted {
                    ..
                }),
                _,
            ) => {
                return Err(self.located(
                    parameter.location,
                    "restricted module parameter requires a type argument",
                ));
            }
            (ModuleParameterType::Unresolved(ty), value) => {
                let ty = self.bind_module_type(file, ty, parameter.location.span, &mut vec![])?;
                self.coerce_module_value(file, &ty, value, parameter.location, program_wide)?
            }
            (
                ModuleParameterType::Interface {
                    constraint, ..
                },
                value @ ParameterValue::Type(_),
            ) => {
                let constraint =
                    self.bind_module_type(file, constraint, parameter.location.span, &mut vec![])?;
                let ParameterValue::Type(actual) = &value else {
                    unreachable!()
                };
                self.validate_module_interface(file, actual, &constraint, parameter.location)?;
                value
            }
            (ModuleParameterType::String, value @ ParameterValue::String(_)) => value,
            (ModuleParameterType::Scalar(ty), ParameterValue::Scalar(value)) => {
                ParameterValue::Scalar(
                    value
                        .coerce(*ty, parameter.location.span)
                        .map_err(|d| self.parameter_diagnostic(parameter.location, d))?,
                )
            }
            _ => {
                return Err(self.located(
                    parameter.location,
                    "module argument type differs from parameter type",
                ));
            }
        };
        Ok(value)
    }
}
fn unsupported_expression(expression: &Expression) -> Option<jai_source::Span> {
    match &expression.kind {
        ExpressionKind::Integer(_)
        | ExpressionKind::Character(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Name(_)
        | ExpressionKind::QualifiedName(_) => None,
        ExpressionKind::Unary(_, expression) | ExpressionKind::Cast(_, _, expression) => {
            unsupported_expression(expression)
        }
        ExpressionKind::TypeCast {
            ty: TypeSyntax::Builtin(BuiltinType::Float(_)),
            value,
            ..
        } => unsupported_expression(value),
        ExpressionKind::Binary(_, lhs, rhs) => {
            unsupported_expression(lhs).or_else(|| unsupported_expression(rhs))
        }
        ExpressionKind::Conditional(e) => e.expressions().find_map(unsupported_expression),
        _ => Some(expression.span),
    }
}

pub(super) fn coerce_float(
    value: Value,
    target: jai_types::FloatType,
    span: jai_source::Span,
) -> Result<Value, Diagnostic> {
    match value {
        Value::WeakFloat(value) => value.round(target, span).map(Value::Float),
        Value::Float(value)
            if value.ty() == target
                || (value.ty() == jai_types::FloatType::F32
                    && target == jai_types::FloatType::F64) =>
        {
            Ok(Value::Float(value.convert(target)))
        }
        Value::Float(_) => Err(Diagnostic::new(
            span,
            "implicit module argument conversion cannot narrow float precision",
        )),
        _ => Err(Diagnostic::new(
            span,
            "floating-point module parameter requires a floating-point argument",
        )),
    }
}
