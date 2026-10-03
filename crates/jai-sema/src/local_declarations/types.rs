//! Resolve structural annotations through lexical bindings before module lookup.
use super::*;

impl Resolver<'_> {
    pub(super) fn record_field_annotation<R>(
        &mut self,
        owner: TypeId,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let context = self.local_scopes.annotation_owner.record(owner);
        self.nominal_annotation_context(context, operation)
    }

    fn nominal_annotation_context<R>(
        &mut self,
        context: NominalAnnotationContext,
        operation: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = std::mem::replace(&mut self.local_scopes.annotation_owner, context);
        let result = operation(self);
        self.local_scopes.annotation_owner = previous;
        result
    }

    pub(crate) fn lexical_annotation(
        &mut self,
        syntax: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        use syntax::{BuiltinType, TypeSyntax};
        let ty = match syntax {
            TypeSyntax::This => return self.local_scopes.annotation_owner.this_type(span),
            TypeSyntax::TypeOf(value) => {
                return self.annotation_expression_type(value);
            }
            TypeSyntax::Builtin(builtin) => {
                return Ok(match builtin {
                    BuiltinType::Scalar(ty) => self.types.scalar(*ty),
                    BuiltinType::Float(ty) => self.types.float(*ty),
                    BuiltinType::String => self.types.string(),
                    BuiltinType::Void => self.types.void(),
                    BuiltinType::Type => {
                        self.schema_header_type(span)?;
                        self.types.meta_type()
                    }
                    BuiltinType::Any => {
                        let ty = self.types.reserve_any();
                        self.any_schema(ty, span)?;
                        ty
                    }
                    BuiltinType::Context => {
                        if let Some(context) = self.context {
                            context.definition.record_type
                        } else {
                            return self
                                .graph_scope
                                .ok_or_else(|| {
                                    Diagnostic::new(
                                        span,
                                        "context type requires a source schema identity",
                                    )
                                })?
                                .annotation(syntax, self.types, span);
                        }
                    }
                });
            }
            TypeSyntax::Named(path) => return self.local_type_name(path, span),
            TypeSyntax::Application(application) => {
                return self.local_record_application(syntax, application, span);
            }
            TypeSyntax::Variable(name) => return self.lexical_type_variable(*name, span),
            TypeSyntax::Restricted { variable, span, .. } => {
                return self.lexical_type_variable(*variable, *span);
            }
            TypeSyntax::InlineRecord(record) => return self.local_inline_record(record),
            TypeSyntax::InlineEnum(enumeration) => return self.local_inline_enum(enumeration),
            TypeSyntax::Variant { .. } => {
                return Err(Diagnostic::new(
                    span,
                    "distinct and isa types require a named type alias declaration",
                ));
            }
            TypeSyntax::Pointer(inner) => {
                let inner = self.lexical_annotation(inner, span)?;
                self.types.pointer(inner)
            }
            TypeSyntax::FixedArray { count, element } => {
                let element = self.lexical_annotation(element, span)?;
                let count = self.local_integer_count(count)?;
                self.types.fixed_array(element, count)
            }
            TypeSyntax::Slice(element) => {
                let element = self.lexical_annotation(element, span)?;
                self.types.slice(element)
            }
            TypeSyntax::DynamicArray(element) => {
                let element = self.lexical_annotation(element, span)?;
                self.types.dynamic_array(element)
            }
            TypeSyntax::Procedure(procedure) => {
                return self
                    .nominal_annotation_context(NominalAnnotationContext::Forbidden, |resolver| {
                        resolver.lexical_procedure_annotation(procedure, span)
                    });
            }
        };
        ty.map_err(|error| Diagnostic::new(span, error.to_string()))
    }

    fn lexical_type_variable(&mut self, name: Symbol, span: Span) -> Result<TypeId, Diagnostic> {
        if let Some(binding) = self.resolve_local_name(name, span)? {
            return match binding {
                Binding::Type(ty) => Ok(ty),
                Binding::Discarded(_) => Err(Diagnostic::new(
                    span,
                    "#discard parameter cannot be used as a type",
                )),
                _ => Err(Diagnostic::new(
                    span,
                    "lexical type variable is shadowed by a value declaration",
                )),
            };
        }
        if let Some(ty) = self
            .graph_scope
            .and_then(|scope| scope.substitution)
            .and_then(|substitution| substitution.ty(name))
        {
            return Ok(ty);
        }
        if let Some(scope) = self.graph_scope
            && let Some(ty) = scope.module_type_parameter(name, span)?
        {
            return Ok(ty);
        }
        Err(Diagnostic::new(
            span,
            "type variable requires a bound specialization in this lexical scope",
        ))
    }

    fn lexical_procedure_annotation(
        &mut self,
        procedure: &syntax::ProcedureTypeSyntax,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        let mut parameters = Vec::with_capacity(procedure.parameters.len());
        let mut source_parameters = Vec::with_capacity(procedure.parameters.len());
        let mut results = Vec::with_capacity(procedure.results.len());
        let mut variadic = jai_types::Variadic::None;
        let mut seen_variadic = false;
        for (index, parameter) in procedure.parameters.iter().enumerate() {
            if parameter.variadic {
                if seen_variadic {
                    return Err(Diagnostic::new(
                        parameter.span,
                        "a procedure type can have only one variadic parameter",
                    ));
                }
                seen_variadic = true;
                if procedure.convention == jai_types::CallingConvention::C {
                    if index + 1 != procedure.parameters.len() {
                        return Err(Diagnostic::new(
                            parameter.span,
                            "C variadic parameter must be last",
                        ));
                    }
                    if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                        variadic = jai_types::Variadic::C {
                            fixed_parameters: parameters.len(),
                        };
                    }
                    continue;
                }
                let element = self.lexical_annotation(&parameter.ty, parameter.span)?;
                let pack = self
                    .types
                    .slice(element)
                    .map_err(|error| Diagnostic::new(parameter.span, error.to_string()))?;
                source_parameters.push(pack);
                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                    variadic = jai_types::Variadic::Jai {
                        parameter: parameters.len(),
                        element,
                    };
                    parameters.push(pack);
                }
            } else {
                let ty = self.lexical_annotation(&parameter.ty, parameter.span)?;
                source_parameters.push(ty);
                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                    parameters.push(ty);
                }
            }
        }
        for result in &procedure.results {
            results.push(self.lexical_annotation(&result.ty, result.span)?);
        }
        crate::procedure_values::signatures::normalize_results(&mut results, self.types, |ty| *ty);
        let ty = self
            .types
            .procedure(ProcedureType {
                parameters: parameters.into_boxed_slice(),
                results: results.into_boxed_slice(),
                convention: procedure.convention,
                context: procedure.context,
                variadic,
            })
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        self.remember_local_procedure_annotation(procedure, source_parameters, ty, span)?;
        Ok(ty)
    }
}
