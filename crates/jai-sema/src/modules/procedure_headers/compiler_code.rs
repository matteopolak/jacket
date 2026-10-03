//! Identify a closed compiler-only result before native
//! parameter defaults or procedure ABI construction can execute.
use super::*;

pub(super) fn template<'a>(
    graph: &'a ModuleGraph,
    declaration: &jai_modules::Declaration,
    declarations: &ScopedDeclarations<'a>,
    types: &mut TypeRegistry,
    constants: &mut Constants<'a>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<Option<crate::compiler_code::CompilerCodeTemplate>, LocatedDiagnostic> {
    let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
        return Ok(None);
    };
    if procedure.expands || crate::polymorphism::is_polymorphic(procedure) {
        return Ok(None);
    }
    let [result] = procedure.results.as_slice() else {
        return Ok(None);
    };
    let syntax::ResultBinding::Typed {
        ty,
        default,
    } = &result.binding
    else {
        return Ok(None);
    };
    let ty = declarations.nominals.resolve_type_with_specializations(
        graph,
        aggregates::parameterized::TypeRequest::new(declaration.file(), ty, result.span),
        types,
        &mut meta.record_specializations,
        &mut |file, expression| constants.evaluate_lazy(file, expression),
    )?;
    let kind = types.kind(ty).map_err(|error| {
        located(
            graph,
            declaration.file(),
            Diagnostic::new(result.span, error.to_string()),
        )
    })?;
    if !matches!(kind, TypeKind::Code) {
        return Ok(None);
    }
    if !procedure.parameters.is_empty() || default.is_some() {
        return Err(located(
            graph,
            declaration.file(),
            Diagnostic::new(
                procedure.span,
                "compiler Code procedures require the checked compiler-domain parameter and result-default binding phase",
            ),
        ));
    }
    if procedure.convention != CallingConvention::Jai
        || procedure.compiler.is_some()
        || procedure.modify.is_some()
    {
        return Err(located(
            graph,
            declaration.file(),
            Diagnostic::new(
                procedure.span,
                "compiler Code requires a checked compiler source header without native ABI or modifier execution",
            ),
        ));
    }
    meta.compiler_code
        .source_retention()
        .borrow_mut()
        .admit_procedure(procedure, procedure.span)
        .map_err(|error| located(graph, declaration.file(), error))?;
    Ok(Some(crate::compiler_code::CompilerCodeTemplate {
        declaration: declaration.id(),
        file: declaration.file(),
        location: declaration.location(),
        source: std::sync::Arc::new(procedure.clone()),
    }))
}
