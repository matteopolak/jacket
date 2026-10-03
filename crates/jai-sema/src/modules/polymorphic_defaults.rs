//! Definition-site defaults reuse the common immutable constant evaluator.
use super::*;
use crate::overloads::ArgumentInfo;
use crate::polymorphism::BakedValue;

pub(super) fn sources<'a>(
    parameters: &'a [syntax::Parameter],
    results: &'a [syntax::ProcedureResult],
) -> Vec<(&'a syntax::Expression, Option<syntax::TypeSyntax>)> {
    let mut defaults = Vec::new();
    for parameter in parameters {
        match &parameter.binding {
            syntax::ParameterBinding::Defaulted { expression, ty } => defaults.push((
                expression,
                ty.map(|ty| syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty))),
            )),
            syntax::ParameterBinding::DefaultedType { expression, ty } => {
                defaults.push((expression, ty.clone()))
            }
            _ => {}
        }
    }
    for result in results {
        match &result.binding {
            syntax::ResultBinding::Typed {
                default: Some(expression),
                ty,
            } => defaults.push((expression, Some(ty.clone()))),
            syntax::ResultBinding::InferredDefault(expression) => defaults.push((expression, None)),
            _ => {}
        }
    }
    defaults
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    expected: Option<&syntax::TypeSyntax>,
    types: &mut TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    constants: &mut Constants<'_>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<ArgumentInfo, LocatedDiagnostic> {
    let scope = FileScope {
        declarations,
        file,
        substitution: None,
    };
    let span = expression.span;
    if let Some(read) = super::runtime_defaults::prepare(
        graph,
        file,
        expression,
        None,
        types,
        declarations,
        constants,
        meta,
    )? {
        return Ok(ArgumentInfo::runtime_read(read));
    }
    let (procedure_source, cast_annotation) = match &expression.kind {
        syntax::ExpressionKind::TypeCast { ty, value, .. } => (value.as_ref(), Some(ty)),
        _ => (expression, None),
    };
    let source_procedure = match &procedure_source.kind {
        syntax::ExpressionKind::Name(name) => scope.signature(&path(*name), span).ok(),
        syntax::ExpressionKind::QualifiedName(path) => scope.signature(path, span).ok(),
        _ => None,
    };
    if let Some(signature) = source_procedure {
        let ty = match cast_annotation.or(expected) {
            Some(expected) => scope
                .annotation_with_specializations(
                    expected,
                    types,
                    &mut meta.record_specializations,
                    span,
                )
                .map_err(|error| located(graph, file, error))?,
            None => signature.ty,
        };
        if ty != signature.ty {
            return Err(located(
                graph,
                file,
                Diagnostic::new(
                    span,
                    "procedure default requires the same canonical signature",
                ),
            ));
        }
        return Ok(ArgumentInfo::constant(
            BakedValue::Value(jai_ir::ConstantValue {
                ty,
                kind: jai_ir::ConstantKind::Procedure(signature.id),
            }),
            ty,
        ));
    }
    let resolve =
        |syntax: &syntax::TypeSyntax,
         types: &mut TypeRegistry,
         records: &mut aggregates::parameterized::RecordSpecializations| {
            scope
                .annotation_with_specializations(syntax, types, records, span)
                .map_err(|error| located(graph, file, error))
        };
    let target =
        match &expression.kind {
            syntax::ExpressionKind::SourceFile
            | syntax::ExpressionKind::SourceFilepath
            | syntax::ExpressionKind::SourceLine
            | syntax::ExpressionKind::SourceLocation => Some(match expected {
                Some(expected) => resolve(expected, types, &mut meta.record_specializations)?,
                None => match expression.kind {
                    syntax::ExpressionKind::SourceFile | syntax::ExpressionKind::SourceFilepath => {
                        types.string()
                    }
                    syntax::ExpressionKind::SourceLine => {
                        types.scalar(ScalarType::Int(IntegerType::S64))
                    }
                    syntax::ExpressionKind::SourceLocation => scope
                        .caller_location_type(types, span)
                        .map_err(|error| located(graph, file, error))?,
                    _ => unreachable!(),
                },
            }),
            syntax::ExpressionKind::StructLiteral(literal) => match &literal.ty {
                Some(path) => Some(resolve(
                    &syntax::TypeSyntax::Named(path.clone()),
                    types,
                    &mut meta.record_specializations,
                )?),
                None => Some(resolve(
                    expected.ok_or_else(|| {
                        located(
                            graph,
                            file,
                            Diagnostic::new(span, "record default requires a declared type"),
                        )
                    })?,
                    types,
                    &mut meta.record_specializations,
                )?),
            },
            syntax::ExpressionKind::ArrayLiteral(literal) => {
                if let Some(expected) = expected {
                    Some(resolve(expected, types, &mut meta.record_specializations)?)
                } else if let Some(element) = &literal.element_type {
                    let element = resolve(element, types, &mut meta.record_specializations)?;
                    Some(
                        types
                            .fixed_array(element, literal.elements.len() as u64)
                            .map_err(|error| {
                                located(graph, file, Diagnostic::new(span, error.to_string()))
                            })?,
                    )
                } else {
                    return Err(located(
                        graph,
                        file,
                        Diagnostic::new(span, "array default requires a declared element type"),
                    ));
                }
            }
            syntax::ExpressionKind::InferredMember(_) => Some(resolve(
                expected.ok_or_else(|| {
                    located(
                        graph,
                        file,
                        Diagnostic::new(span, "leading-dot default requires a declared enum type"),
                    )
                })?,
                types,
                &mut meta.record_specializations,
            )?),
            _ => None,
        };
    if let Some(target) = target {
        let mut evaluator =
            aggregates::Defaults::new(graph, types, &declarations.nominals, constants)
                .with_specializations(&meta.record_specializations)
                .with_context(declarations.context.as_ref());
        evaluator.fields = declarations.defaults.clone();
        hydrate_constants(&mut evaluator, declarations, meta);
        let value = evaluator.expression(file, expression, target)?;
        let value = BakedValue::runtime(value, types)
            .map_err(|error| located(graph, file, Diagnostic::new(span, error.to_string())))?;
        return Ok(ArgumentInfo::constant(value, target));
    }
    Ok(match &expression.kind {
        syntax::ExpressionKind::Code(syntax::CodeBody::Null) => {
            ArgumentInfo::code_null(types.code_type())
        }
        syntax::ExpressionKind::CallerLocation => ArgumentInfo::caller_location(
            scope
                .caller_location_type(types, span)
                .map_err(|error| located(graph, file, error))?,
        ),
        syntax::ExpressionKind::Null => ArgumentInfo::null(),
        syntax::ExpressionKind::String(bytes) => ArgumentInfo::constant(
            BakedValue::String(bytes.clone().into_boxed_slice()),
            types.string(),
        ),
        syntax::ExpressionKind::HereString(value) => ArgumentInfo::constant(
            BakedValue::String(value.bytes.clone().into_boxed_slice()),
            types.string(),
        ),
        syntax::ExpressionKind::Type(ty) => ArgumentInfo::constant(
            BakedValue::Type(resolve(ty, types, &mut meta.record_specializations)?),
            types.meta_type(),
        ),
        syntax::ExpressionKind::Name(name) => match scope.value(
            &NamePath {
                root: *name,
                members: Vec::new(),
            },
            span,
        ) {
            Ok(Binding::Enum(value)) => ArgumentInfo::constant(
                BakedValue::Value(jai_ir::ConstantValue {
                    ty: value.ty,
                    kind: jai_ir::ConstantKind::Enum(value.value),
                }),
                value.ty,
            ),
            Ok(Binding::TypedConstant(id)) => {
                let value = meta.constant(id).ok_or_else(|| {
                    located(
                        graph,
                        file,
                        Diagnostic::new(span, "typed default constant is unavailable"),
                    )
                })?;
                ArgumentInfo::constant(BakedValue::Value(value.clone()), value.ty)
            }
            _ => ArgumentInfo::scalar_constant(constants.evaluate(file, expression)?, types),
        },
        syntax::ExpressionKind::QualifiedName(path) => match scope.value(path, span) {
            Ok(Binding::Enum(value)) => ArgumentInfo::constant(
                BakedValue::Value(jai_ir::ConstantValue {
                    ty: value.ty,
                    kind: jai_ir::ConstantKind::Enum(value.value),
                }),
                value.ty,
            ),
            Ok(Binding::TypedConstant(id)) => {
                let value = meta.constant(id).ok_or_else(|| {
                    located(
                        graph,
                        file,
                        Diagnostic::new(span, "typed default constant is unavailable"),
                    )
                })?;
                ArgumentInfo::constant(BakedValue::Value(value.clone()), value.ty)
            }
            _ => ArgumentInfo::scalar_constant(constants.evaluate(file, expression)?, types),
        },
        _ => ArgumentInfo::scalar_constant(constants.evaluate(file, expression)?, types),
    })
}
