//! Partial aliases capture typed bindings in their actual defining file.
use super::partial_arguments::{
    CompletedRecordArgument, PartialRecordApplication, PartialRecordArgument, RecordFormalId,
};
use super::*;

#[derive(Default)]
pub(super) struct PartialRecordAliases {
    bindings: HashMap<DeclarationId, PartialRecordApplication>,
    active: HashSet<DeclarationId>,
}

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn partial_template_application(
        &mut self,
        file: FileInstanceId,
        path: &syntax::NamePath,
        span: Span,
    ) -> TypeResult<PartialRecordApplication> {
        self.pending_lookup(file, path, span)?;
        let id = declaration_id(self.graph, file, path, span)
            .map_err(|error| failure(self.graph, file, error))?;
        self.partial_template_declaration(id, span)
    }

    pub(super) fn partial_template_declaration(
        &mut self,
        id: DeclarationId,
        span: Span,
    ) -> TypeResult<PartialRecordApplication> {
        if let Some(application) = self.records.partial_aliases.bindings.get(&id) {
            return Ok(application.clone());
        }
        let declaration = self
            .graph
            .declaration(id)
            .expect("partial template belongs to the graph");
        if let syntax::FileDeclarationKind::Record(_) = &declaration.syntax().kind {
            return Ok(PartialRecordApplication {
                declaration: id,
                file: declaration.file(),
                location: declaration.location(),
                arguments: vec![],
            });
        }
        if !self.records.partial_aliases.active.insert(id) {
            return Err(failure(
                self.graph,
                declaration.file(),
                Diagnostic::new(span, "cyclic partial record alias"),
            ));
        }
        let result = self.capture_partial_alias(id, span);
        self.records.partial_aliases.active.remove(&id);
        if let Ok(application) = &result {
            self.records
                .partial_aliases
                .bindings
                .insert(id, application.clone());
        }
        result
    }

    fn capture_partial_alias(
        &mut self,
        id: DeclarationId,
        span: Span,
    ) -> TypeResult<PartialRecordApplication> {
        let declaration = self
            .graph
            .declaration(id)
            .expect("alias belongs to the graph");
        let (path, supplied) = match &declaration.syntax().kind {
            syntax::FileDeclarationKind::TypeAlias(alias) => {
                let syntax::TypeSyntax::Named(path) = &alias.ty else {
                    return Err(failure(
                        self.graph,
                        declaration.file(),
                        Diagnostic::new(
                            span,
                            "partial application base is not a record template alias",
                        ),
                    ));
                };
                (path.clone(), vec![])
            }
            syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                match &constant.initializer.kind {
                    syntax::ExpressionKind::BakeArguments(baked) => {
                        let path = expression_path(&baked.callee).ok_or_else(|| {
                            failure(
                                self.graph,
                                declaration.file(),
                                Diagnostic::new(
                                    baked.callee.span,
                                    "partial record alias requires its named originating template",
                                ),
                            )
                        })?;
                        (path, baked.arguments.clone())
                    }
                    _ => {
                        let path = expression_path(&constant.initializer).ok_or_else(|| {
                            failure(
                                self.graph,
                                declaration.file(),
                                Diagnostic::new(
                                    span,
                                    "partial application base is not a record template alias",
                                ),
                            )
                        })?;
                        (path, vec![])
                    }
                }
            }
            _ => {
                return Err(failure(
                    self.graph,
                    declaration.file(),
                    Diagnostic::new(
                        span,
                        "partial application base is not a record template alias",
                    ),
                ));
            }
        };
        let mut application = self.partial_template_application(
            declaration.file(),
            &path,
            declaration.location().span,
        )?;
        let original = self
            .graph
            .declaration(application.declaration)
            .expect("partial template origin exists");
        let syntax::FileDeclarationKind::Record(record) = &original.syntax().kind else {
            unreachable!("partial template lookup retained the original record")
        };
        let mut slots = vec![None; record.parameters.len()];
        for argument in &application.arguments {
            slots[argument.formal.ordinal] = Some(argument.value.clone());
        }
        let mut additions = vec![None; record.parameters.len()];
        for argument in &supplied {
            let name = argument.name.ok_or_else(|| {
                failure(
                    self.graph,
                    declaration.file(),
                    Diagnostic::new(
                        argument.value.span,
                        "#bake_arguments requires named record arguments",
                    ),
                )
            })?;
            if argument.spread {
                return Err(failure(
                    self.graph,
                    declaration.file(),
                    Diagnostic::new(
                        argument.value.span,
                        "partial record arguments cannot spread a runtime pack",
                    ),
                ));
            }
            let formal = record
                .parameters
                .iter()
                .position(|parameter| parameter.name == name)
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        declaration.file(),
                        Diagnostic::new(argument.value.span, "unknown partial record argument"),
                    )
                })?;
            if slots[formal].is_some() || additions[formal].replace(&argument.value).is_some() {
                return Err(failure(
                    self.graph,
                    declaration.file(),
                    Diagnostic::new(
                        argument.value.span,
                        "record formal supplied more than once across partial application",
                    ),
                ));
            }
        }
        let mut preceding = Substitution::default();
        for (ordinal, parameter) in record.parameters.iter().enumerate() {
            let value = match (&slots[ordinal], additions[ordinal]) {
                (Some(value), _) => Some(value.clone()),
                (None, Some(expression)) => {
                    let expected = self.parameter_type(original.file(), parameter, &preceding)?;
                    let value = self.in_lexical_scope(false, |resolver| {
                        resolver.baked(declaration.file(), expression, expected, None)
                    })?;
                    application.arguments.push(PartialRecordArgument {
                        formal: RecordFormalId {
                            declaration: application.declaration,
                            ordinal,
                        },
                        value: value.clone(),
                        location: jai_source::SourceSpan {
                            source: declaration.location().source,
                            span: expression.span,
                        },
                    });
                    Some(value)
                }
                _ => None,
            };
            if let Some(value) = value {
                preceding.bind_constant(parameter.name, value);
            }
        }
        application
            .arguments
            .sort_by_key(|argument| argument.formal.ordinal);
        Ok(application)
    }

    pub(super) fn instantiate_partial_application(
        &mut self,
        partial: &PartialRecordApplication,
        arguments: &[syntax::CallArgument],
        caller_file: FileInstanceId,
        caller_substitution: Option<&Substitution>,
        span: Span,
    ) -> TypeResult<TypeId> {
        let declaration = self
            .graph
            .declaration(partial.declaration)
            .expect("retained template exists");
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            unreachable!("partial carrier has an actual original record")
        };
        let location = jai_source::SourceSpan {
            source: self.graph.file(caller_file).unwrap().source(),
            span,
        };
        let bound = partial
            .complete_arguments(&record.parameters, arguments, location)
            .map_err(|error| failure(self.graph, caller_file, error))?;
        let caller_lexical = self.lexical_active;
        let mut substitution = Substitution::default();
        for (formal, argument) in bound {
            let parameter = &record.parameters[formal.ordinal];
            let expected = self.parameter_type(declaration.file(), parameter, &substitution)?;
            let value = match argument {
                CompletedRecordArgument::Baked(value) => match value {
                    BakedValue::Type(ty) if expected == self.types.meta_type() => {
                        self.types.kind(*ty).map_err(|error| {
                            failure(
                                self.graph,
                                declaration.file(),
                                Diagnostic::new(parameter.span, error.to_string()),
                            )
                        })?;
                        value.clone()
                    }
                    _ => {
                        let constant =
                            value
                                .clone()
                                .into_runtime(expected, self.types)
                                .map_err(|error| {
                                    failure(
                                        self.graph,
                                        declaration.file(),
                                        Diagnostic::new(parameter.span, error.to_string()),
                                    )
                                })?;
                        BakedValue::runtime(constant, self.types).map_err(|error| {
                            failure(
                                self.graph,
                                declaration.file(),
                                Diagnostic::new(parameter.span, error.to_string()),
                            )
                        })?
                    }
                },
                CompletedRecordArgument::Source {
                    expression,
                    defaulted,
                } => {
                    let (file, overlay) = if defaulted {
                        (declaration.file(), Some(&substitution))
                    } else {
                        (caller_file, caller_substitution)
                    };
                    self.in_lexical_scope(caller_lexical && !defaulted, |resolver| {
                        resolver.baked(file, expression, expected, overlay)
                    })?
                }
            };
            substitution.bind_constant(parameter.name, value);
        }
        self.instantiate_at(partial.declaration, substitution, location)
    }
}
