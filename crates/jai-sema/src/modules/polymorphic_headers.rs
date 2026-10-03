//! Register bodyless generic headers without inventing a source procedure body.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn register_prototype(
    graph: &ModuleGraph,
    declaration: DeclarationId,
    file: FileInstanceId,
    source: &syntax::ProcedurePrototype,
    types: &mut TypeRegistry,
    declarations: &ScopedDeclarations<'_>,
    constants: &mut Constants<'_>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<bool, LocatedDiagnostic> {
    if !crate::polymorphism::is_polymorphic_prototype(source) {
        return Ok(false);
    }
    if let syntax::PrototypeBinding::Intrinsic {
        tag,
    } = &source.binding
    {
        jai_ir::RuntimeIntrinsic::validate_name(
            tag.as_deref()
                .unwrap_or_else(|| graph.symbols().name(source.name)),
        )
        .map_err(|error| located(graph, file, Diagnostic::new(source.span, error.to_string())))?;
    }
    let mut defaults = HashMap::new();
    for (expression, expected) in polymorphic_defaults::sources(&source.parameters, &source.results)
    {
        let info = polymorphic_defaults::prepare(
            graph,
            file,
            expression,
            expected.as_ref(),
            types,
            declarations,
            constants,
            meta,
        )?;
        defaults.insert((expression.span.start, expression.span.end), info);
    }
    let applications = aggregates::parameterized::prototype_patterns(
        graph,
        file,
        source,
        types,
        &declarations.nominals,
        &mut meta.record_specializations,
        &mut |file, expression| constants.evaluate_lazy(file, expression),
    )?;
    let scope = FileScope {
        declarations,
        file,
        substitution: None,
    };
    let template = crate::polymorphism::from_prototype_with_patterns(
        declaration,
        source,
        |syntax, span| {
            scope.annotation_with_specializations(
                syntax,
                types,
                &mut meta.record_specializations,
                span,
            )
        },
        |expression| Ok(defaults[&(expression.span.start, expression.span.end)].clone()),
        |expression| {
            let value = constants
                .evaluate(file, expression)
                .map_err(|error| Diagnostic::at_source(error.location, error.message))?;
            let integer = match value {
                ConstantValue::Literal(value) => value,
                ConstantValue::Int(value) => value.value(),
                _ => {
                    return Err(Diagnostic::new(
                        expression.span,
                        "array count requires an integer constant",
                    ));
                }
            };
            u64::try_from(integer)
                .map_err(|_| Diagnostic::new(expression.span, "array count is out of range"))
        },
        &applications,
    )
    .map_err(|error| located(graph, file, error))?;
    declarations.generics.borrow_mut().register_prototype(
        crate::polymorphism::integration::TemplateDefinition {
            template,
            file,
            span: source.span,
            return_abi: source.return_abi,
            convention: source.convention,
            context: source.context,
        },
    );
    Ok(true)
}
