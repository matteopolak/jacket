use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn resolve(
        &mut self,
        file: FileInstanceId,
        syntax: &syntax::TypeSyntax,
        substitution: Option<&Substitution>,
        span: Span,
    ) -> TypeResult<TypeId> {
        use syntax::{BuiltinType as B, TypeSyntax as T};
        let result = match syntax {
            T::Restricted {
                variable,
                span,
                ..
            } => {
                return self.resolve(file, &T::Variable(*variable), substitution, *span);
            }
            T::This => {
                return self
                    .nominal_context
                    .this_type(span)
                    .map_err(|error| failure(self.graph, file, error));
            }
            T::TypeOf(value) => return self.annotation_type_of(file, value, substitution),
            T::Builtin(builtin) => {
                return Ok(match builtin {
                    B::Scalar(ty) => self.types.scalar(*ty),
                    B::Float(ty) => self.types.float(*ty),
                    B::String => self.types.string(),
                    B::Void => self.types.void(),
                    B::Type => self
                        .nominals
                        .runtime_type_for_graph(self.graph, self.types)
                        .map_err(|error| {
                            failure(self.graph, file, Diagnostic::new(span, error.to_string()))
                        })?,
                    B::Context => self.nominals.context_type(self.types),
                    B::Any => self
                        .nominals
                        .reserve_any_for_graph(self.graph, self.types)
                        .map_err(|error| {
                            failure(self.graph, file, Diagnostic::new(span, error.to_string()))
                        })?,
                });
            }
            T::Variable(name) => {
                if let Some(ty) = substitution.and_then(|s| s.ty(*name)) {
                    return Ok(ty);
                }
                if let Some(ty) = self
                    .nominals
                    .inserted_capture_type(self.graph, file, *name, span)
                    .map_err(|error| failure(self.graph, file, error))?
                {
                    return Ok(ty);
                }
                if let Ok(jai_modules::Binding::Parameter(id)) =
                    self.graph.lookup(file, &path(*name))
                    && let jai_modules::ParameterValue::Type(value) =
                        &self.graph.parameter(id).unwrap().value
                {
                    return self
                        .nominals
                        .resolve_module_type_with_specializations(
                            self.graph,
                            crate::modules::aggregates::types::ModuleTypeRequest {
                                file,
                                value,
                                span,
                            },
                            self.types,
                            self.records,
                            self.evaluate,
                        )
                        .map_err(TypeFailure::from);
                }
                self.pending_lookup(file, &path(*name), span)?;
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(span, "unbound record template type parameter"),
                ));
            }
            T::Named(path) => {
                if self.lexical_active
                    && let Some(binding) = self
                        .lexical
                        .and_then(|scope| scope.bindings.get(&(span.start, span.end)))
                {
                    return match binding {
                        LexicalTypeArgument::Type(ty) => Ok(*ty),
                        _ => Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(span, "lexical binding does not denote a type"),
                        )),
                    };
                }
                if self.lexical_active
                    && let Some(lexical) = self.lexical
                    && lexical.roots.contains_key(&path.root)
                {
                    return match lexical.lookup(path) {
                        Some(LexicalTypeArgument::Type(ty)) => Ok(*ty),
                        _ => Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(
                                span,
                                "lexical binding does not denote a type or has no such namespace member",
                            ),
                        )),
                    };
                }
                if path.members.is_empty()
                    && let Some(ty) = substitution.and_then(|s| s.ty(path.root))
                {
                    return Ok(ty);
                }
                if path.members.is_empty()
                    && let Some(ty) = self
                        .nominals
                        .inserted_capture_type(self.graph, file, path.root, span)
                        .map_err(|error| failure(self.graph, file, error))?
                {
                    return Ok(ty);
                }
                if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, path)
                    && let jai_modules::ParameterValue::Type(value) =
                        &self.graph.parameter(id).unwrap().value
                {
                    return self
                        .nominals
                        .resolve_module_type_with_specializations(
                            self.graph,
                            crate::modules::aggregates::types::ModuleTypeRequest {
                                file,
                                value,
                                span,
                            },
                            self.types,
                            self.records,
                            self.evaluate,
                        )
                        .map_err(TypeFailure::from);
                }
                if path.members.is_empty()
                    && matches!(
                        self.graph.lookup(file, path),
                        Err(jai_modules::LookupError::UnknownName(_))
                    )
                    && let Some(builtin) = B::from_spelling(self.graph.symbols().name(path.root))
                {
                    return self.resolve(file, &T::Builtin(builtin), substitution, span);
                }
                if !path.members.is_empty() {
                    for count in (0..path.members.len()).rev() {
                        let prefix = syntax::NamePath {
                            root: path.root,
                            members: path.members[..count].to_vec(),
                        };
                        let bound = if count == 0 {
                            substitution.and_then(|s| s.ty(path.root))
                        } else {
                            None
                        };
                        let root = match bound {
                            Some(ty) => Some(ty),
                            None => {
                                self.source_namespace_root(file, &prefix, substitution, span)?
                            }
                        };
                        if let Some(mut ty) = root {
                            let mut found = true;
                            for member in &path.members[count..] {
                                if let Some(next) = self
                                    .records
                                    .member_bindings(ty)
                                    .and_then(|scope| scope.ty(*member))
                                {
                                    ty = next;
                                } else {
                                    found = false;
                                    break;
                                }
                            }
                            if found {
                                return Ok(ty);
                            }
                        }
                    }
                }
                self.pending_lookup(file, path, span)?;
                let id = match declaration_id(self.graph, file, path, span) {
                    Ok(id) => id,
                    Err(error) => {
                        if let Some(ty) = self
                            .nominals
                            .generated_reflection_type(self.graph, file, path, self.types, span)?
                        {
                            return Ok(ty);
                        }
                        return Err(failure(self.graph, file, error));
                    }
                };
                if let Some(&ty) = self.nominals.declarations.get(&id) {
                    return Ok(ty);
                }
                let declaration = self
                    .graph
                    .declaration(id)
                    .expect("resolved declaration exists");
                if !self.aliases.insert(id) {
                    return Err(failure(
                        self.graph,
                        file,
                        Diagnostic::new(span, "cyclic type alias dependencies"),
                    ));
                }
                let alias = match &declaration.syntax().kind {
                    syntax::FileDeclarationKind::TypeAlias(alias) => Some(alias.ty.clone()),
                    syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                        type_expression(&constant.initializer)
                    }
                    syntax::FileDeclarationKind::Record(record)
                        if !record.parameters.is_empty() =>
                    {
                        self.aliases.remove(&id);
                        return Err(failure(
                            self.graph,
                            file,
                            Diagnostic::new(span, "record template requires a type application"),
                        ));
                    }
                    _ => None,
                }
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(span, "declaration does not denote a type"),
                    )
                })?;
                let result = self.resolve(
                    declaration.file(),
                    &alias,
                    substitution,
                    declaration.location().span,
                );
                self.aliases.remove(&id);
                return result;
            }
            T::Application(application) => {
                return self.application(file, application, substitution);
            }
            T::Pointer(inner) => {
                let inner = self.resolve(file, inner, substitution, span)?;
                self.types.pointer(inner)
            }
            T::Slice(inner) => {
                let inner = self.resolve(file, inner, substitution, span)?;
                self.types.slice(inner)
            }
            T::DynamicArray(inner) => {
                let inner = self.resolve(file, inner, substitution, span)?;
                self.types.dynamic_array(inner)
            }
            T::FixedArray {
                count,
                element,
            } => {
                let element = self.resolve(file, element, substitution, span)?;
                let value = self.scalar(file, count, substitution)?;
                let count_value = match value {
                    ScalarConstant::Literal(value) => Some(value),
                    ScalarConstant::Int(value) => Some(value.value()),
                    _ => None,
                }
                .and_then(|value| u64::try_from(value).ok())
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            count.span,
                            "array count requires a nonnegative integer constant",
                        ),
                    )
                })?;
                self.types.fixed_array(element, count_value)
            }
            T::Procedure(procedure) => {
                return self.without_record_annotation(|resolver| {
                    resolver.procedure_annotation(file, procedure, substitution, span)
                });
            }
            T::InlineRecord(record) => return self.inline_record(file, record, substitution),
            T::InlineEnum(enumeration) => return self.inline_enum(file, enumeration, substitution),
            T::Variant {
                ..
            } => {
                return self
                    .nominals
                    .resolve_type(self.graph, file, syntax, self.types, span, self.evaluate)
                    .map_err(TypeFailure::from);
            }
        };
        result.map_err(|e| failure(self.graph, file, Diagnostic::new(span, e.to_string())))
    }

    fn procedure_annotation(
        &mut self,
        file: FileInstanceId,
        procedure: &syntax::ProcedureTypeSyntax,
        substitution: Option<&Substitution>,
        span: Span,
    ) -> TypeResult<TypeId> {
        let mut parameters = Vec::new();
        let mut source_parameters = Vec::new();
        let mut results = Vec::new();
        let mut variadic = Variadic::None;
        for parameter in &procedure.parameters {
            let ty = self.resolve(file, &parameter.ty, substitution, parameter.span)?;
            if parameter.variadic && procedure.convention == jai_types::CallingConvention::C {
                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                    variadic = Variadic::C {
                        fixed_parameters: parameters.len(),
                    };
                }
                continue;
            }
            if parameter.variadic {
                let pack = self.types.slice(ty).map_err(|e| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(parameter.span, e.to_string()),
                    )
                })?;
                source_parameters.push(pack);
                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                    variadic = Variadic::Jai {
                        parameter: parameters.len(),
                        element: ty,
                    };
                    parameters.push(pack);
                }
            } else {
                source_parameters.push(ty);
                if parameter.evaluation == syntax::ParameterEvaluation::Evaluate {
                    parameters.push(ty);
                }
            }
        }
        for result in &procedure.results {
            results.push(self.resolve(file, &result.ty, substitution, result.span)?);
        }
        crate::procedure_values::signatures::normalize_results(&mut results, self.types, |ty| *ty);
        let ty = self
            .types
            .procedure(ProcedureType {
                parameters: parameters.into_boxed_slice(),
                results: results.into_boxed_slice(),
                return_abi: procedure.return_abi,
                convention: procedure.convention,
                context: procedure.context,
                variadic,
            })
            .map_err(|error| failure(self.graph, file, Diagnostic::new(span, error.to_string())))?;
        let defaults = procedure
            .parameters
            .iter()
            .filter(|parameter| {
                !(parameter.variadic && procedure.convention == jai_types::CallingConvention::C)
            })
            .zip(&source_parameters)
            .map(|(parameter, expected)| {
                parameter
                    .default
                    .as_ref()
                    .map(|expression| {
                        if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                            return Ok(crate::ParameterDefault::Discarded);
                        }
                        super::super::defaults::Defaults::with_evaluator(
                            self.graph,
                            self.types,
                            self.nominals,
                            self.evaluate,
                        )
                        .with_specializations(self.records)
                        .with_substitution(substitution.cloned())
                        .expression(file, expression, *expected)
                        .map(crate::ParameterDefault::Constant)
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, LocatedDiagnostic>>()
            .map_err(TypeFailure::from)?;
        self.nominals
            .remember_procedure_annotation(
                crate::procedure_values::source_annotations::AnnotationPublication {
                    file,
                    source: procedure,
                    substitution,
                    parameters: source_parameters,
                    defaults,
                    ty,
                    span,
                },
                self.types,
            )
            .map_err(|error| failure(self.graph, file, error))?;
        Ok(ty)
    }
    pub(super) fn pending_lookup(
        &self,
        file: FileInstanceId,
        path: &syntax::NamePath,
        span: Span,
    ) -> TypeResult<()> {
        if let Err(error) = self.graph.lookup(file, path)
            && let Ok(pending) = PendingType::from_lookup(
                error,
                jai_source::SourceSpan {
                    source: self.graph.file(file).expect("lookup file exists").source(),
                    span,
                },
            )
        {
            return Err(TypeFailure::Pending(pending));
        }
        Ok(())
    }

    pub(super) fn template_declaration(
        &self,
        file: FileInstanceId,
        path: &syntax::NamePath,
        span: Span,
    ) -> TypeResult<DeclarationId> {
        self.pending_lookup(file, path, span)?;
        let mut id = declaration_id(self.graph, file, path, span)
            .map_err(|error| failure(self.graph, file, error))?;
        let mut visited = HashSet::new();
        loop {
            if !visited.insert(id) {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(span, "cyclic record template alias"),
                ));
            }
            let declaration = self
                .graph
                .declaration(id)
                .expect("resolved declaration exists");
            let alias = match &declaration.syntax().kind {
                syntax::FileDeclarationKind::Record(_) => return Ok(id),
                syntax::FileDeclarationKind::TypeAlias(alias) => Some(alias.ty.clone()),
                syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                    type_expression(&constant.initializer)
                }
                _ => None,
            };
            let Some(syntax::TypeSyntax::Named(target)) = alias else {
                return Err(failure(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "type application base does not denote a record template",
                    ),
                ));
            };
            self.pending_lookup(declaration.file(), &target, declaration.location().span)?;
            id = declaration_id(self.graph, declaration.file(), &target, span)
                .map_err(|error| failure(self.graph, declaration.file(), error))?;
        }
    }
}
