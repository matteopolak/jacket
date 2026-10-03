//! Source defaults use the same literal inference as globals and record defaults.
use super::*;

pub(super) fn infer_discarded_default<'a>(
    graph: &'a ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    types: &mut TypeRegistry,
    nominals: &Nominals<'a>,
    constants: &Constants<'a>,
) -> Result<TypeId, LocatedDiagnostic> {
    let path = match &expression.kind {
        syntax::ExpressionKind::Call(name, _) => Some(NamePath {
            root: *name,
            members: vec![],
        }),
        syntax::ExpressionKind::QualifiedCall(path, _) => Some(path.clone()),
        syntax::ExpressionKind::CallHint { call, .. }
        | syntax::ExpressionKind::CompileTime(syntax::CompileTimeRun {
            body: syntax::CompileTimeBody::Expression(call),
            ..
        }) => {
            return infer_discarded_default(graph, file, call, types, nominals, constants);
        }
        _ => None,
    };
    if let Some(path) = path
        && let Some(ty) = nominals.value_type(graph, file, &path)
        && let Ok(signature) = types.procedure_definition(ty)
    {
        return match signature.results.as_ref() {
            [ty] => Ok(*ty),
            _ => Err(located(
                graph,
                file,
                Diagnostic::new(
                    expression.span,
                    "discarded default inference requires one checked procedure result",
                ),
            )),
        };
    }
    infer_default(graph, file, expression, types, nominals, constants)
}

pub(super) fn infer_default<'a>(
    graph: &'a ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    types: &mut TypeRegistry,
    nominals: &Nominals<'a>,
    constants: &Constants<'a>,
) -> Result<TypeId, LocatedDiagnostic> {
    if matches!(
        expression.kind,
        syntax::ExpressionKind::Code(syntax::CodeBody::Null)
    ) {
        return Ok(types.code_type());
    }
    if matches!(expression.kind, syntax::ExpressionKind::CallerLocation) {
        return crate::caller_locations::infer_target(
            graph,
            file,
            nominals,
            types,
            expression.span,
        )
        .map_err(|error| located(graph, file, error));
    }
    match sequence_constants::infer(graph, file, expression, types, nominals, constants)? {
        Some(ty) => Ok(ty),
        None => infer_constant_type(graph, file, expression, types, nominals, constants),
    }
}
