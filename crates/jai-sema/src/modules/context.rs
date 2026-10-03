//! Resolve context extensions in their defining module scopes, then publish one schema.
use super::*;
use crate::context::{ContextConstant, Field, Schema};

pub(super) fn build(
    graph: &ModuleGraph,
    types: &mut TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    constants: &mut Constants<'_>,
    registrations: &[super::context_registration::Registration],
    meta: &mut crate::reflection::MetaContext,
) -> Result<Schema, LocatedDiagnostic> {
    let mut fields = Vec::new();
    let mut constant_members = HashMap::new();
    let mut names = std::collections::HashSet::new();
    let root_file = graph.module(graph.root()).unwrap().entry();
    let additions = registrations
        .iter()
        .map(|field| (field.file, &field.syntax, field.location.span))
        .chain(
            graph
                .context_fields()
                .iter()
                .map(|field| (field.file(), field.syntax(), field.location().span)),
        );
    for (file, declaration, span) in additions {
        let name = match declaration {
            syntax::ContextFieldDeclaration::Field(field) => field.name,
            syntax::ContextFieldDeclaration::Variable(global) => global.declaration.name(),
            syntax::ContextFieldDeclaration::Constant(constant) => constant.name,
        };
        if !names.insert(name) {
            return Err(located(
                graph,
                file,
                Diagnostic::new(span, "conflicting context member name"),
            ));
        }
        match declaration {
            declaration @ (syntax::ContextFieldDeclaration::Variable(_)
            | syntax::ContextFieldDeclaration::Field(_)) => {
                let syntax = match declaration {
                    syntax::ContextFieldDeclaration::Field(field) => field.clone(),
                    syntax::ContextFieldDeclaration::Variable(global) => {
                        source_field(&global.declaration, span)
                            .map_err(|error| located(graph, file, error))?
                    }
                    _ => unreachable!(),
                };
                let (ty, expression) = match &syntax.binding {
                    syntax::FieldBinding::Inferred(initializer) => (
                        infer_constant_type(
                            graph,
                            file,
                            initializer,
                            types,
                            &declarations.nominals,
                            constants,
                        )?,
                        Some(initializer),
                    ),
                    syntax::FieldBinding::Explicit { ty, initializer } => (
                        declarations.nominals.resolve_type(
                            graph,
                            file,
                            ty,
                            types,
                            span,
                            &mut |file, expression| constants.evaluate_lazy(file, expression),
                        )?,
                        initializer.as_ref(),
                    ),
                };
                if syntax.using && !matches!(types.kind(ty), Ok(jai_types::TypeKind::Record(_))) {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(span, "using context field requires a record value"),
                    ));
                }
                let mut alignment = None;
                for attribute in &syntax.attributes {
                    let syntax::FieldAttribute::Alignment(expression) = attribute;
                    let value = constants.evaluate_lazy(file, expression)?;
                    let value = match value {
                        ConstantValue::Int(value) => u32::try_from(value.value()).ok(),
                        ConstantValue::Literal(value) => u32::try_from(value).ok(),
                        _ => None,
                    }.filter(|value| value.is_power_of_two()).ok_or_else(|| located(graph,file,
                        Diagnostic::new(expression.span,"alignment requires a nonzero power-of-two integer constant representable as u32")))?;
                    alignment = Some(value);
                }
                let mut defaults = defaults(graph, types, declarations, constants, meta);
                let value = match expression {
                    Some(expression) => defaults.expression(file, expression, ty)?,
                    None => defaults.default_value(file, ty, span)?,
                };
                let source = graph
                    .sources()
                    .get(graph.file(file).unwrap().source())
                    .unwrap()
                    .text();
                let notes = syntax
                    .notes
                    .iter()
                    .map(|note| {
                        note.span
                            .text(source)
                            .strip_prefix('@')
                            .unwrap_or_else(|| note.span.text(source))
                            .as_bytes()
                            .into()
                    })
                    .collect();
                fields.push(Field {
                    file,
                    notes,
                    name: syntax.name,
                    value,
                    span,
                    syntax,
                    alignment,
                });
            }
            syntax::ContextFieldDeclaration::Constant(constant) => {
                if constant_members.contains_key(&constant.name) {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(span, "conflicting context member name"),
                    ));
                }
                let path = match &constant.initializer.kind {
                    syntax::ExpressionKind::Name(name) => Some(path(*name)),
                    syntax::ExpressionKind::QualifiedName(path) => Some(path.clone()),
                    _ => None,
                };
                let signature = path
                    .and_then(|path| declaration_id(graph, file, &path, span).ok())
                    .and_then(|id| declarations.signatures.get(&id));
                let value = if let Some(signature) = signature {
                    if let Some(annotation) = &constant.ty {
                        let expected = FileScope {
                            declarations,
                            file,
                            substitution: None,
                        }
                        .annotation_with_specializations(
                            annotation,
                            types,
                            &mut meta.record_specializations,
                            constant.span,
                        )
                        .map_err(|error| located(graph, file, error))?;
                        if expected != signature.ty {
                            return Err(located(
                                graph,
                                file,
                                Diagnostic::new(
                                    span,
                                    "context procedure constant annotation requires the same canonical signature",
                                ),
                            ));
                        }
                    }
                    ContextConstant::Procedure {
                        procedure: signature.id,
                        ty: signature.ty,
                    }
                } else {
                    let ty = match &constant.ty {
                        Some(annotation) => FileScope {
                            declarations,
                            file,
                            substitution: None,
                        }
                        .annotation_with_specializations(
                            annotation,
                            types,
                            &mut meta.record_specializations,
                            constant.span,
                        )
                        .map_err(|error| located(graph, file, error))?,
                        None => infer_constant_type(
                            graph,
                            file,
                            &constant.initializer,
                            types,
                            &declarations.nominals,
                            constants,
                        )?,
                    };
                    let mut defaults = defaults(graph, types, declarations, constants, meta);
                    ContextConstant::Value(defaults.expression(file, &constant.initializer, ty)?)
                };
                constant_members.insert(constant.name, value);
            }
        }
    }
    let record_type = declarations.nominals.context_type(types);
    Schema::new(types, record_type, fields, constant_members)
        .map_err(|error| located(graph, root_file, error))
}

fn defaults<'a, 'b>(
    graph: &'a ModuleGraph,
    types: &'b TypeRegistry,
    declarations: &'b ScopedDeclarations<'a>,
    constants: &'b Constants<'a>,
    meta: &'b crate::reflection::MetaContext,
) -> aggregates::Defaults<'a, 'b> {
    let mut defaults = aggregates::Defaults::new(graph, types, &declarations.nominals, constants)
        .with_specializations(&meta.record_specializations);
    defaults.fields = declarations.defaults.clone();
    enum_constants::seed_defaults(&mut defaults, &declarations.values, meta);
    hydrate_constants(&mut defaults, declarations, meta);
    defaults
}

fn source_field(
    declaration: &syntax::Declaration,
    span: Span,
) -> Result<syntax::FieldDeclaration, Diagnostic> {
    let binding = match declaration {
        syntax::Declaration::External { .. } => {
            return Err(Diagnostic::new(
                span,
                "external data cannot supply an owned context field",
            ));
        }
        syntax::Declaration::Inferred { initializer, .. } => {
            syntax::FieldBinding::Inferred(initializer.clone())
        }
        syntax::Declaration::Explicit {
            ty, initializer, ..
        } => syntax::FieldBinding::Explicit {
            ty: syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(*ty)),
            initializer: initializer.clone(),
        },
        syntax::Declaration::UnresolvedExplicit {
            ty, initializer, ..
        } => syntax::FieldBinding::Explicit {
            ty: ty.clone(),
            initializer: initializer.clone(),
        },
    };
    Ok(syntax::FieldDeclaration {
        name: declaration.name(),
        binding,
        using: false,
        conversion: syntax::FieldConversion::None,
        span,
        attributes: declaration
            .attributes()
            .iter()
            .map(|attribute| match attribute {
                syntax::DeclarationAttribute::Alignment(expression) => {
                    syntax::FieldAttribute::Alignment(expression.clone())
                }
            })
            .collect(),
        notes: vec![],
    })
}
