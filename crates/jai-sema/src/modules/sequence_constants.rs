//! Typed literal declarations use the semantic constant arena rather than scalar bindings.
use super::*;
use std::collections::HashSet;

pub(super) fn is_sequence_constant(
    graph: &ModuleGraph,
    declaration: &jai_modules::Declaration,
) -> bool {
    sequence_declaration(graph, declaration.id(), &mut HashSet::new())
}

fn sequence_declaration(
    graph: &ModuleGraph,
    mut id: DeclarationId,
    visiting: &mut HashSet<DeclarationId>,
) -> bool {
    loop {
        if !visiting.insert(id) {
            return false;
        }
        let declaration = graph.declaration(id).expect("declaration identity exists");
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            return false;
        };
        let name = match &constant.initializer.kind {
            syntax::ExpressionKind::String(_)
            | syntax::ExpressionKind::SourceFile
            | syntax::ExpressionKind::SourceFilepath
            | syntax::ExpressionKind::SourceLine
            | syntax::ExpressionKind::SourceLocation
            | syntax::ExpressionKind::HereString(_)
            | syntax::ExpressionKind::ArrayLiteral(_)
            | syntax::ExpressionKind::StructLiteral(_)
            | syntax::ExpressionKind::PositionalStructLiteral(_) => return true,
            syntax::ExpressionKind::TypeCast { ty, .. }
                if !matches!(ty, syntax::TypeSyntax::Builtin(_)) =>
            {
                return true;
            }
            syntax::ExpressionKind::Name(name) => path(*name),
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return false,
        };
        if let Ok(jai_modules::Binding::Parameter(parameter)) =
            graph.lookup(declaration.file(), &name)
        {
            return matches!(
                graph.parameter(parameter).map(|parameter| &parameter.value),
                Some(jai_modules::ParameterValue::String(_))
            );
        }
        let Ok(target) = declaration_id(graph, declaration.file(), &name, constant.span) else {
            return false;
        };
        id = target;
    }
}

pub(super) fn infer<'a>(
    graph: &'a ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    types: &mut TypeRegistry,
    nominals: &Nominals<'a>,
    constants: &Constants<'a>,
) -> Result<Option<TypeId>, LocatedDiagnostic> {
    LiteralInference {
        graph,
        nominals,
        constants,
        visiting: HashSet::new(),
        depth: 0,
    }
    .expression(file, expression, types)
}

struct LiteralInference<'graph, 'metadata> {
    graph: &'graph ModuleGraph,
    nominals: &'metadata Nominals<'graph>,
    constants: &'metadata Constants<'graph>,
    visiting: HashSet<DeclarationId>,
    depth: usize,
}
impl LiteralInference<'_, '_> {
    fn expression(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        types: &mut TypeRegistry,
    ) -> Result<Option<TypeId>, LocatedDiagnostic> {
        if self.depth >= crate::constant_limits::MAX_CONSTANT_DEPTH {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "constant exceeds compiler constant depth budget",
                ),
            ));
        }
        self.depth += 1;
        let result = self.expression_inner(file, expression, types);
        self.depth -= 1;
        result
    }
    fn expression_inner(
        &mut self,
        file: FileInstanceId,
        expression: &syntax::Expression,
        types: &mut TypeRegistry,
    ) -> Result<Option<TypeId>, LocatedDiagnostic> {
        let ty = match &expression.kind {
            syntax::ExpressionKind::String(_)
            | syntax::ExpressionKind::HereString(_)
            | syntax::ExpressionKind::SourceFile
            | syntax::ExpressionKind::SourceFilepath => types.string(),
            syntax::ExpressionKind::SourceLine => types.scalar(ScalarType::Int(IntegerType::S64)),
            syntax::ExpressionKind::SourceLocation => crate::caller_locations::infer_target(
                self.graph,
                file,
                self.nominals,
                types,
                expression.span,
            )
            .map_err(|error| located(self.graph, file, error))?,
            syntax::ExpressionKind::TypeCast { ty, .. } => self.nominals.resolve_type(
                self.graph,
                file,
                ty,
                types,
                expression.span,
                &mut |file, expression| self.constants.evaluate(file, expression),
            )?,
            syntax::ExpressionKind::ArrayLiteral(literal) => {
                let element = if let Some(annotation) = &literal.element_type {
                    self.nominals.resolve_type(
                        self.graph,
                        file,
                        annotation,
                        types,
                        expression.span,
                        &mut |file, expression| self.constants.evaluate(file, expression),
                    )?
                } else {
                    let first = literal.elements.first().ok_or_else(|| {
                        located(
                            self.graph,
                            file,
                            Diagnostic::new(
                                expression.span,
                                "empty array literal requires an element type",
                            ),
                        )
                    })?;
                    match self.expression(file, first, types)? {
                        Some(ty) => ty,
                        None => self.constants.evaluate(file, first)?.type_id(types),
                    }
                };
                let count = u64::try_from(literal.elements.len()).map_err(|_| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(expression.span, "array literal has too many elements"),
                    )
                })?;
                types.fixed_array(element, count).map_err(|error| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(expression.span, error.to_string()),
                    )
                })?
            }
            syntax::ExpressionKind::StructLiteral(syntax::StructLiteral { ty, .. })
            | syntax::ExpressionKind::PositionalStructLiteral(syntax::PositionalStructLiteral {
                ty,
                ..
            }) => {
                let path = ty.as_ref().ok_or_else(|| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "record literal requires a contextual type",
                        ),
                    )
                })?;
                let id = declaration_id(self.graph, file, path, expression.span)
                    .map_err(|error| located(self.graph, file, error))?;
                *self.nominals.declarations.get(&id).ok_or_else(|| {
                    located(
                        self.graph,
                        file,
                        Diagnostic::new(
                            expression.span,
                            "record literal declaration does not denote a type",
                        ),
                    )
                })?
            }
            syntax::ExpressionKind::Name(name) => {
                return self.name(file, &path(*name), expression.span, types);
            }
            syntax::ExpressionKind::QualifiedName(path) => {
                return self.name(file, path, expression.span, types);
            }
            _ => return Ok(None),
        };
        Ok(Some(ty))
    }
    fn name(
        &mut self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
        types: &mut TypeRegistry,
    ) -> Result<Option<TypeId>, LocatedDiagnostic> {
        if let Some(ty) = self.nominals.value_type(self.graph, file, path) {
            return Ok(Some(ty));
        }
        if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, path)
            && matches!(
                self.graph.parameter(id).map(|parameter| &parameter.value),
                Some(jai_modules::ParameterValue::String(_))
            )
        {
            return Ok(Some(types.string()));
        }
        let Ok(id) = declaration_id(self.graph, file, path, span) else {
            return Ok(None);
        };
        if !sequence_declaration(self.graph, id, &mut HashSet::new()) {
            return Ok(None);
        }
        if !self.visiting.insert(id) {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(span, "cyclic typed constant dependencies"),
            ));
        }
        let declaration = self.graph.declaration(id).expect("declaration exists");
        let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
            return Ok(None);
        };
        let result = self.expression(declaration.file(), &constant.initializer, types);
        self.visiting.remove(&id);
        result
    }
}

pub(super) fn bind<'a>(
    declarations: &mut ScopedDeclarations<'a>,
    types: &mut TypeRegistry,
    constants: &Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<(), LocatedDiagnostic> {
    let deferred = deferred_constants::classify(declarations.graph);
    let ids = declarations
        .graph
        .declarations()
        .iter()
        .filter(|declaration| {
            !deferred.contains(&declaration.id())
                && (is_sequence_constant(declarations.graph, declaration)
                    || constants.has_typed_annotation(declaration.id()))
        })
        .map(|declaration| declaration.id())
        .collect::<Vec<_>>();
    let mut visiting = HashSet::new();
    for id in ids {
        bind_one(id, declarations, types, constants, meta, &mut visiting)?;
    }
    Ok(())
}

fn bind_one<'a>(
    id: DeclarationId,
    declarations: &mut ScopedDeclarations<'a>,
    types: &mut TypeRegistry,
    constants: &Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
    visiting: &mut HashSet<DeclarationId>,
) -> Result<(), LocatedDiagnostic> {
    if declarations.values.contains_key(&id) {
        return Ok(());
    }
    let declaration = declarations
        .graph
        .declaration(id)
        .expect("declaration exists");
    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
        return Ok(());
    };
    if visiting.len() >= crate::constant_limits::MAX_CONSTANT_DEPTH {
        return Err(located(
            declarations.graph,
            declaration.file(),
            Diagnostic::new(
                constant.span,
                "constant exceeds compiler constant depth budget",
            ),
        ));
    }
    if !visiting.insert(id) {
        return Err(located(
            declarations.graph,
            declaration.file(),
            Diagnostic::new(constant.span, "cyclic typed constant dependencies"),
        ));
    }
    let mut pending = vec![&constant.initializer];
    let mut dependencies = Vec::new();
    while let Some(expression) = pending.pop() {
        let name = match &expression.kind {
            syntax::ExpressionKind::Name(name) => Some(path(*name)),
            syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
            syntax::ExpressionKind::ArrayLiteral(literal) => {
                pending.extend(&literal.elements);
                None
            }
            syntax::ExpressionKind::StructLiteral(literal) => {
                pending.extend(literal.fields.iter().map(|field| &field.value));
                None
            }
            syntax::ExpressionKind::PositionalStructLiteral(literal) => {
                pending.extend(&literal.values);
                None
            }
            syntax::ExpressionKind::TypeCast { value, .. } => {
                pending.push(value);
                None
            }
            _ => None,
        };
        if let Some(path) = name
            && let Ok(id) = declaration_id(
                declarations.graph,
                declaration.file(),
                &path,
                expression.span,
            )
            && (sequence_declaration(declarations.graph, id, &mut HashSet::new())
                || constants.has_typed_annotation(id))
        {
            dependencies.push(id);
        }
    }
    for dependency in dependencies {
        bind_one(dependency, declarations, types, constants, meta, visiting)?;
    }
    let ty = match constants.annotation(id) {
        Some(ty) => ty,
        None => infer(
            declarations.graph,
            declaration.file(),
            &constant.initializer,
            types,
            &declarations.nominals,
            constants,
        )?
        .expect("typed literal has a sequence or record type"),
    };
    let mut evaluator =
        aggregates::Defaults::new(declarations.graph, types, &declarations.nominals, constants)
            .with_specializations(&meta.record_specializations)
            .with_context(declarations.context.as_ref());
    evaluator.fields = declarations.defaults.clone();
    hydrate_constants(&mut evaluator, declarations, meta);
    let value = evaluator.expression(declaration.file(), &constant.initializer, ty)?;
    let handle = meta.intern_constant(value);
    declarations
        .values
        .insert(id, Binding::TypedConstant(handle));
    visiting.remove(&id);
    Ok(())
}
