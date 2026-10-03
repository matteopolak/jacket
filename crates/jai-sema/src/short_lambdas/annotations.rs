//! Pure structural annotation facts: no source evaluation or body scheduling.
use super::*;

impl Resolver<'_> {
    pub(crate) fn preview_annotation(
        &mut self,
        annotation: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        self.preview_annotation_inner(annotation, span, true)
    }

    fn preview_annotation_inner(
        &mut self,
        annotation: &syntax::TypeSyntax,
        span: Span,
        allow_this: bool,
    ) -> Result<TypeId, Diagnostic> {
        use syntax::{BuiltinType as B, TypeSyntax as T};
        let result =
            match annotation {
                T::Builtin(builtin) => return Ok(match builtin {
                    B::Scalar(ty) => self.types.scalar(*ty),
                    B::Float(ty) => self.types.float(*ty),
                    B::String => self.types.string(),
                    B::Void => self.types.void(),
                    B::Type => self.types.meta_type(),
                    B::Any => self.types.any_type().ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "Any annotation requires ready schema metadata before preview",
                        )
                    })?,
                    B::Context => self
                        .context
                        .ok_or_else(|| {
                            Diagnostic::new(
                                span,
                                "context annotation requires ready source schema before preview",
                            )
                        })?
                        .definition
                        .record_type,
                }),
                T::This if allow_this => return self.lexical_annotation(annotation, span),
                T::This => {
                    return Err(Diagnostic::new(
                        span,
                        "#this is not allowed in procedure annotations",
                    ));
                }
                T::Named(path) => {
                    return match self.preview_annotation_binding(path, span)? {
                        Binding::Type(ty) => Ok(ty),
                        _ => Err(Diagnostic::new(
                            span,
                            "annotation name does not denote a ready type",
                        )),
                    };
                }
                T::Variable(name)
                | T::Restricted {
                    variable: name, ..
                } => {
                    if let Some(Binding::Type(ty)) =
                        self.scopes.iter().rev().find_map(|scope| scope.get(name))
                    {
                        return Ok(*ty);
                    }
                    return self
                        .graph_scope
                        .and_then(|scope| scope.substitution)
                        .and_then(|values| values.ty(*name))
                        .ok_or_else(|| {
                            Diagnostic::new(span, "annotation requires a ready bound type variable")
                        });
                }
                T::TypeOf(value) => {
                    return self.queried_expression_type(value);
                }
                T::Pointer(inner) => {
                    let inner = self.preview_annotation_inner(inner, span, allow_this)?;
                    self.types.pointer(inner)
                }
                T::Slice(inner) => {
                    let inner = self.preview_annotation_inner(inner, span, allow_this)?;
                    self.types.slice(inner)
                }
                T::DynamicArray(inner) => {
                    let inner = self.preview_annotation_inner(inner, span, allow_this)?;
                    self.types.dynamic_array(inner)
                }
                T::FixedArray {
                    count,
                    element,
                } => {
                    let element = self.preview_annotation_inner(element, span, allow_this)?;
                    let count_value = jai_eval::evaluate_paths_with_overflow_check(
                        count,
                        self.checks.arithmetic_overflow,
                        |path, span| match self.preview_annotation_binding(path, span)? {
                            Binding::Constant(value) => Ok(value),
                            Binding::Enum(value) => Ok(ScalarConstant::Int(value.value)),
                            _ => Err(Diagnostic::new(
                                span,
                                "array preview count requires ready immutable scalar facts",
                            )),
                        },
                    )?;
                    let count = match count_value {
                        ScalarConstant::Literal(value) => u64::try_from(value).ok(),
                        ScalarConstant::Int(value) => u64::try_from(value.value()).ok(),
                        _ => None,
                    }
                    .ok_or_else(|| {
                        Diagnostic::new(
                            count.span,
                            "array preview count is not a nonnegative integer",
                        )
                    })?;
                    self.types.fixed_array(element, count)
                }
                T::Procedure(source) => {
                    let mut parameters = Vec::new();
                    let mut source_parameters = Vec::new();
                    let mut variadic = Variadic::None;
                    let mut seen_variadic = false;
                    for (index, parameter) in source.parameters.iter().enumerate() {
                        if parameter.variadic {
                            if seen_variadic {
                                return Err(Diagnostic::new(
                                    parameter.span,
                                    "procedure annotation has multiple variadic parameters",
                                ));
                            }
                            seen_variadic = true;
                            if source.convention == CallingConvention::C {
                                if index + 1 != source.parameters.len() {
                                    return Err(Diagnostic::new(
                                        parameter.span,
                                        "C variadic annotation must be last",
                                    ));
                                }
                                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                                    variadic = Variadic::C {
                                        fixed_parameters: parameters.len(),
                                    };
                                }
                                continue;
                            }
                        }
                        let element =
                            self.preview_annotation_inner(&parameter.ty, parameter.span, false)?;
                        let ty = if parameter.variadic {
                            let pack = self.types.slice(element).map_err(|error| {
                                Diagnostic::new(parameter.span, error.to_string())
                            })?;
                            if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                                variadic = Variadic::Jai {
                                    parameter: parameters.len(),
                                    element,
                                };
                            }
                            pack
                        } else {
                            element
                        };
                        source_parameters.push(ty);
                        if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                            parameters.push(ty);
                        }
                    }
                    let mut results = Vec::new();
                    for result in &source.results {
                        let ty = self.preview_annotation_inner(&result.ty, result.span, false)?;
                        results.push(ty);
                    }
                    crate::procedure_values::signatures::normalize_results(
                        &mut results,
                        self.types,
                        |ty| *ty,
                    );
                    let ty = self
                        .types
                        .procedure(ProcedureType {
                            parameters: parameters.into(),
                            results: results.into(),
                            return_abi: source.return_abi,
                            convention: source.convention,
                            context: source.context,
                            variadic,
                        })
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    self.remember_local_procedure_annotation(source, source_parameters, ty, span)?;
                    return Ok(ty);
                }
                T::Application(_)
                | T::InlineRecord(_)
                | T::InlineEnum(_)
                | T::Variant {
                    ..
                } => {
                    return Err(Diagnostic::new(
                        span,
                        "annotation requires ready nominal source metadata before pure preview",
                    ));
                }
            };
        result.map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    fn preview_annotation_binding(
        &self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Binding, Diagnostic> {
        if let Some(binding) = self.lexical_graph_binding_ready(path, span)? {
            return self.imported_binding_value_ready(binding, span);
        }
        if path.members.is_empty()
            && let Some(binding) = self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(&path.root))
        {
            return Ok(binding.clone());
        }
        if self.local_name_present(path.root) {
            return Err(Diagnostic::new(
                span,
                "local annotation binding is not ready for pure preview",
            ));
        }
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(
                span,
                "annotation name requires a ready defining source scope",
            )
        })?;
        match scope.value(path, span) {
            Ok(binding) => Ok(binding),
            Err(error) => scope
                .type_name(path, span)
                .map(Binding::Type)
                .map_err(|_| error),
        }
    }
}
