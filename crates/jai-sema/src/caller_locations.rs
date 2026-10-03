//! Deferred source locations remain typed until a concrete call binds them.
use super::*;
use jai_modules::{FileInstanceId, ModuleGraph};
use jai_types::{RecordKind, TypeKind};

pub(crate) fn validate_target(
    graph: &ModuleGraph,
    nominals: &modules::aggregates::Nominals<'_>,
    types: &TypeRegistry,
    ty: TypeId,
    span: Span,
) -> Result<(), Diagnostic> {
    let record = nominals.records.get(&ty).ok_or_else(|| {
        Diagnostic::new(
            span,
            "#caller_location requires the source Source_Code_Location struct",
        )
    })?;
    let declaration = graph
        .declaration(record.declaration)
        .expect("source nominal declaration");
    if graph.symbols().name(declaration.name()) != "Source_Code_Location"
        || record.kind != RecordKind::Struct
    {
        return Err(Diagnostic::new(
            span,
            "#caller_location requires the source Source_Code_Location struct",
        ));
    }
    let names = ["fully_pathed_filename", "line_number", "character_number"];
    if record.fields.len() != names.len()
        || record
            .fields
            .iter()
            .zip(names)
            .any(|(field, name)| graph.symbols().name(field.name) != name)
    {
        return Err(Diagnostic::new(
            span,
            "Source_Code_Location requires the source filename/line/character field order",
        ));
    }
    if record.fields[0].ty != types.string()
        || record.fields[1..].iter().any(|field| {
            !matches!(
                types.kind(field.ty),
                Ok(TypeKind::Integer(IntegerType::S64))
            )
        })
    {
        return Err(Diagnostic::new(
            span,
            "Source_Code_Location requires string, s64, s64 field types",
        ));
    }
    Ok(())
}

pub(crate) fn infer_target(
    graph: &ModuleGraph,
    file: FileInstanceId,
    nominals: &modules::aggregates::Nominals<'_>,
    types: &TypeRegistry,
    span: Span,
) -> Result<TypeId, Diagnostic> {
    let name = graph
        .symbols()
        .find("Source_Code_Location")
        .ok_or_else(|| {
            Diagnostic::new(
                span,
                "#caller_location requires a visible source Source_Code_Location type",
            )
        })?;
    let ty = selected_source_target(
        graph,
        file,
        &syntax::NamePath {
            root: name,
            members: vec![],
        },
        nominals,
        span,
        &mut std::collections::HashSet::new(),
    )?;
    // Callable type reservation runs before record fields are defined. Prove
    // the real nominal identity now; complete headers and calls check fields.
    if nominals.records.contains_key(&ty) {
        validate_target(graph, nominals, types, ty, span)?;
    } else if !nominals.declarations.iter().any(|(&id, &candidate)| {
        candidate == ty && graph.declaration(id).is_some_and(|declaration| {
            graph.symbols().name(declaration.name()) == "Source_Code_Location"
                && matches!(&declaration.syntax().kind, syntax::FileDeclarationKind::Record(record)
                    if record.kind == RecordKind::Struct)
        })
    }) {
        return Err(Diagnostic::new(
            span,
            "#caller_location requires the source Source_Code_Location struct",
        ));
    }
    Ok(ty)
}

/// Follow only the selected source aliases in their defining files. Reservation
/// owns each nominal ID; this lookup neither creates a type nor evaluates a value.
fn selected_source_target(
    graph: &ModuleGraph,
    file: FileInstanceId,
    path: &syntax::NamePath,
    nominals: &modules::aggregates::Nominals<'_>,
    span: Span,
    visiting: &mut std::collections::HashSet<jai_source::DeclarationId>,
) -> Result<TypeId, Diagnostic> {
    let binding = graph.lookup(file, path).map_err(|_| {
        Diagnostic::new(
            span,
            "#caller_location requires a visible source Source_Code_Location type",
        )
    })?;
    let id = match binding {
        jai_modules::Binding::Declaration(id) => id,
        jai_modules::Binding::Parameter(id) => return nominals
            .module_parameter_types
            .get(&id)
            .copied()
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "#caller_location source type parameter is pending canonical materialization",
                )
            }),
        _ => {
            return Err(Diagnostic::new(
                span,
                "#caller_location requires a source type declaration",
            ));
        }
    };
    if let Some(&ty) = nominals.declarations.get(&id) {
        return Ok(ty);
    }
    let declaration = graph.declaration(id).expect("selected source declaration");
    if visiting.len() >= 256 || !visiting.insert(id) {
        return Err(Diagnostic::at_source(
            declaration.location(),
            "#caller_location has cyclic or excessively deep source type aliases",
        ));
    }
    let alias = match &declaration.syntax().kind {
        syntax::FileDeclarationKind::TypeAlias(alias) => match &alias.ty {
            syntax::TypeSyntax::Named(path) => Some(path.clone()),
            _ => None,
        },
        syntax::FileDeclarationKind::Constant(constant) if constant.ty.is_none() => match &constant
            .initializer
            .kind
        {
            syntax::ExpressionKind::Name(root) => Some(syntax::NamePath {
                root: *root,
                members: vec![],
            }),
            syntax::ExpressionKind::QualifiedName(path)
            | syntax::ExpressionKind::Type(syntax::TypeSyntax::Named(path)) => Some(path.clone()),
            _ => None,
        },
        _ => None,
    };
    let result = match alias {
        Some(path) => {
            selected_source_target(graph, declaration.file(), &path, nominals, span, visiting)
        }
        None => Err(Diagnostic::at_source(
            declaration.location(),
            "#caller_location requires a selected source nominal or transparent source type alias",
        )),
    };
    visiting.remove(&id);
    result
}

impl Resolver<'_> {
    pub(crate) fn expanded_caller_location_parameter_type(
        &mut self,
        target: &metaprogram::ExpandedTarget,
        parameter: &syntax::Parameter,
    ) -> Result<Option<TypeId>, Diagnostic> {
        let (expression, annotation) = match &parameter.binding {
            syntax::ParameterBinding::Defaulted {
                expression,
                ty,
            } => (
                expression,
                ty.map(|ty| syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty))),
            ),
            syntax::ParameterBinding::DefaultedType {
                expression,
                ty,
            } => (expression, ty.clone()),
            _ => return Ok(None),
        };
        if !matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
            return Ok(None);
        }
        let definition = self
            .graph_scope
            .ok_or_else(|| {
                Diagnostic::new(
                    expression.span,
                    "#caller_location requires source module resolution",
                )
            })?
            .code_file(target.file);
        let definition_source = target.captured_source().unwrap_or(definition.source());
        let ty = match annotation {
            Some(annotation) => self.expanded_annotation(target, &annotation, parameter.span)?,
            None => definition
                .caller_location_type(self.types, expression.span)
                .map_err(|error| error.with_fallback_source(definition_source))?,
        };
        definition
            .validate_caller_location_type(ty, self.types, expression.span)
            .map_err(|error| error.with_fallback_source(definition_source))?;
        Ok(Some(ty))
    }

    pub(crate) fn expanded_caller_location_default(
        &mut self,
        target: &metaprogram::ExpandedTarget,
        parameter: &syntax::Parameter,
        initializers: &mut Vec<Statement>,
        span: Span,
    ) -> Result<Option<Binding>, Diagnostic> {
        let Some(ty) = self.expanded_caller_location_parameter_type(target, parameter)? else {
            return Ok(None);
        };
        let value =
            self.materialize_parameter_default(&ParameterDefault::CallerLocation, ty, span)?;
        if parameter.baking != syntax::ParameterBaking::None {
            let value = self.literal_constant(value, span)?;
            return Ok(Some(Binding::TypedConstant(
                self.meta.intern_constant(value),
            )));
        }
        let local = self.allocate_typed(ty)?;
        let storage = Storage::local(local, self.types);
        self.debug_prefix_local(
            local,
            parameter.name,
            parameter.span,
            target.captured_source().unwrap_or_else(|| {
                self.graph_scope
                    .expect("validated macro source graph")
                    .code_file(target.file)
                    .source()
            }),
            span,
            initializers.len(),
        )?;
        initializers.push(Statement::Store(local.place(), value));
        Ok(Some(Binding::Storage(storage)))
    }

    pub(crate) fn caller_location_type(&self, span: Span) -> Result<TypeId, Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "#caller_location requires source module resolution")
        })?;
        scope.caller_location_type(self.types, span)
    }

    pub(crate) fn parameter_default(
        &mut self,
        expression: &syntax::Expression,
        ty: TypeId,
    ) -> Result<ParameterDefault, Diagnostic> {
        if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
            self.validate_caller_location_type(ty, expression.span)?;
            Ok(ParameterDefault::CallerLocation)
        } else if matches!(
            expression.kind,
            syntax::ExpressionKind::Code(syntax::CodeBody::Null)
        ) {
            if ty != self.types.code_type() {
                return Err(Diagnostic::new(
                    expression.span,
                    "#code,null requires a compile-time Code parameter",
                ));
            }
            Ok(ParameterDefault::CodeNull {
                ty,
            })
        } else if let Some(read) = self.runtime_parameter_default(expression, ty)? {
            Ok(ParameterDefault::RuntimeRead(read))
        } else {
            self.local_typed_constant(expression, ty)
                .map(ParameterDefault::Constant)
        }
    }

    fn validate_caller_location_type(&self, ty: TypeId, span: Span) -> Result<(), Diagnostic> {
        let scope = self.graph_scope.ok_or_else(|| {
            Diagnostic::new(span, "#caller_location requires source module resolution")
        })?;
        scope.validate_caller_location_type(ty, self.types, span)
    }

    pub(crate) fn materialize_parameter_default(
        &mut self,
        default: &ParameterDefault,
        ty: TypeId,
        span: Span,
    ) -> Result<ValueExpr, Diagnostic> {
        match default {
            ParameterDefault::Source(source) => {
                let checked = source.require()?;
                self.materialize_parameter_default(checked, ty, span)
            }
            ParameterDefault::RuntimeRead(read) => self.materialize_runtime_default(read, ty, span),
            ParameterDefault::Discarded => Err(Diagnostic::new(
                span,
                "#discard default has no runtime value",
            )),
            ParameterDefault::CodeNull {
                ..
            } => Err(Diagnostic::new(
                span,
                "#code,null default requires a compile-time Code parameter binding",
            )),
            ParameterDefault::Constant(value) => {
                if value.ty != ty {
                    return Err(Diagnostic::new(
                        span,
                        "parameter default differs from its declared type",
                    ));
                }
                Ok(value.clone().into_expression())
            }
            ParameterDefault::CallerLocation => {
                self.validate_caller_location_type(ty, span)?;
                let location = match self.debug.caller_origin() {
                    Some(location) => location,
                    None => self.ast_source_location(span)?,
                };
                self.source_location_at(ty, location)
            }
        }
    }

    pub(crate) fn materialize_code_default(
        &mut self,
        default: &ParameterDefault,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        match default {
            ParameterDefault::Source(source) => {
                self.materialize_code_default(source.require()?, span)
            }
            ParameterDefault::CodeNull {
                ty,
            } if *ty == self.types.code_type() => Ok(Expr::Code(self.meta.codes.null())),
            _ => Err(Diagnostic::new(
                span,
                "expected a checked compile-time Code default",
            )),
        }
    }
}
