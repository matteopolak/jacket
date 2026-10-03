//! Materialize graph source-type identities in the existing semantic registry.
use super::*;
use jai_modules::{ModuleBuiltin, ModuleType, ModuleVariadic, ParameterValue};
pub(crate) struct ModuleTypeRequest<'a> {
    pub file: FileInstanceId,
    pub value: &'a ModuleType,
    pub span: Span,
}
impl Nominals<'_> {
    pub(crate) fn inserted_capture_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<TypeId>, Diagnostic> {
        if !matches!(
            graph.insertion_capture_value(file, name),
            Some(jai_modules::SourceCaptureValue::Type(_))
        ) {
            return Ok(None);
        }
        self.inserted_capture_types
            .get(&(file, name))
            .copied()
            .map(Some)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "captured insertion type requires canonical semantic materialization",
                )
            })
    }
    pub(crate) fn resolve_module_type(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        value: &ModuleType,
        types: &mut TypeRegistry,
        span: Span,
    ) -> Result<TypeId, LocatedDiagnostic> {
        self.resolve_module_type_inner(graph, file, value, types, span, &mut |_, _, _| {
            Err(located(
                graph,
                file,
                Diagnostic::new(
                    span,
                    "module specialization requires the canonical record materializer",
                ),
            ))
        })
    }
    pub(crate) fn resolve_module_type_with_specializations(
        &self,
        graph: &ModuleGraph,
        request: ModuleTypeRequest<'_>,
        types: &mut TypeRegistry,
        records: &mut super::super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        let ModuleTypeRequest {
            file,
            value,
            span,
        } = request;
        self.resolve_module_type_inner(
            graph,
            file,
            value,
            types,
            span,
            &mut |types, template, arguments| {
                super::super::parameterized::instantiate_module_application(
                    graph,
                    super::super::parameterized::ModuleApplicationRequest {
                        file,
                        template,
                        arguments,
                        span,
                    },
                    types,
                    self,
                    records,
                    evaluate,
                )
            },
        )
    }
    fn resolve_module_type_inner(
        &self,
        graph: &ModuleGraph,
        file: FileInstanceId,
        value: &ModuleType,
        types: &mut TypeRegistry,
        span: Span,
        application: &mut impl FnMut(
            &mut TypeRegistry,
            DeclarationId,
            &[jai_modules::ModuleBoundArgument],
        ) -> Result<TypeId, LocatedDiagnostic>,
    ) -> Result<TypeId, LocatedDiagnostic> {
        let failure = |message: &str| located(graph, file, Diagnostic::new(span, message));
        let result = match value {
            ModuleType::Builtin(builtin) => {
                return Ok(match builtin {
                    ModuleBuiltin::Scalar(ty) => types.scalar(*ty),
                    ModuleBuiltin::Float(ty) => types.float(*ty),
                    ModuleBuiltin::String => types.string(),
                    ModuleBuiltin::Void => types.void(),
                    ModuleBuiltin::Type => types.meta_type(),
                    ModuleBuiltin::Context => self.context_type(types),
                    ModuleBuiltin::Any => self
                        .reserve_any_for_graph(graph, types)
                        .map_err(|error| failure(&error.to_string()))?,
                });
            }
            ModuleType::Declaration(id) => {
                return self.declarations.get(id).copied().ok_or_else(|| {
                    failure("module type declaration requires semantic nominal specialization")
                });
            }
            ModuleType::Application {
                template,
                arguments,
            } => {
                let ty = application(types, *template, arguments)?;
                return Ok(ty);
            }
            ModuleType::Pointer(inner) => {
                let inner =
                    self.resolve_module_type_inner(graph, file, inner, types, span, application)?;
                types.pointer(inner)
            }
            ModuleType::Slice(inner) => {
                let inner =
                    self.resolve_module_type_inner(graph, file, inner, types, span, application)?;
                types.slice(inner)
            }
            ModuleType::DynamicArray(inner) => {
                let inner =
                    self.resolve_module_type_inner(graph, file, inner, types, span, application)?;
                types.dynamic_array(inner)
            }
            ModuleType::FixedArray {
                element,
                count,
            } => {
                let element =
                    self.resolve_module_type_inner(graph, file, element, types, span, application)?;
                types.fixed_array(element, *count)
            }
            ModuleType::Procedure(procedure) => {
                let mut parameters = Vec::new();
                let mut results = Vec::new();
                for parameter in &procedure.parameters {
                    parameters.push(self.resolve_module_type_inner(
                        graph,
                        file,
                        parameter,
                        types,
                        span,
                        application,
                    )?);
                }
                for result in &procedure.results {
                    results.push(self.resolve_module_type_inner(
                        graph,
                        file,
                        result,
                        types,
                        span,
                        application,
                    )?);
                }
                crate::procedure_values::signatures::normalize_results(&mut results, types, |ty| {
                    *ty
                });
                let variadic = match procedure.variadic {
                    ModuleVariadic::None => jai_types::Variadic::None,
                    ModuleVariadic::C {
                        fixed_parameters,
                    } => jai_types::Variadic::C {
                        fixed_parameters,
                    },
                    ModuleVariadic::Jai {
                        parameter,
                    } => {
                        let Some(&element) = parameters.get(parameter) else {
                            return Err(failure(
                                "module procedure variadic parameter is out of range",
                            ));
                        };
                        parameters[parameter] = types
                            .slice(element)
                            .map_err(|error| failure(&error.to_string()))?;
                        jai_types::Variadic::Jai {
                            parameter,
                            element,
                        }
                    }
                };
                types.procedure(jai_types::ProcedureType {
                    parameters: parameters.into_boxed_slice(),
                    results: results.into_boxed_slice(),
                    return_abi: procedure.return_abi,
                    convention: procedure.convention,
                    context: procedure.context,
                    variadic,
                })
            }
        };
        result.map_err(|error| failure(&error.to_string()))
    }
    pub(crate) fn materialize_module_parameters(
        &mut self,
        graph: &ModuleGraph,
        types: &mut TypeRegistry,
        records: &mut super::super::parameterized::RecordSpecializations,
        evaluate: &mut impl FnMut(
            FileInstanceId,
            &syntax::Expression,
        ) -> Result<ConstantValue, LocatedDiagnostic>,
    ) -> Result<(), LocatedDiagnostic> {
        for parameter in graph.parameters() {
            if let ParameterValue::Type(value) = &parameter.value {
                // Discovery may expose a suspended module before its entry is finalized.
                // Header parameters always belong to that module's first published file.
                let file = graph
                    .module(parameter.module)
                    .and_then(|module| module.files().first().copied())
                    .ok_or_else(|| LocatedDiagnostic {
                        location: parameter.location,
                        message: "module type parameter has no defining file".into(),
                    })?;
                let ty = self.resolve_module_type_with_specializations(
                    graph,
                    ModuleTypeRequest {
                        file,
                        value,
                        span: parameter.location.span,
                    },
                    types,
                    records,
                    evaluate,
                )?;
                let id = match graph
                    .module(parameter.module)
                    .and_then(|module| module.bindings().get(&parameter.name))
                {
                    Some(jai_modules::Binding::Parameter(id)) => *id,
                    _ => {
                        return Err(located(
                            graph,
                            file,
                            Diagnostic::new(
                                parameter.location.span,
                                "module type parameter binding is missing",
                            ),
                        ));
                    }
                };
                self.module_parameter_types.insert(id, ty);
            }
        }
        Ok(())
    }
}
