//! Declaration queries retain compiler Code and validate its graph transport.
use super::*;

pub(super) fn evaluate(
    request: &jai_modules::DeclarationInsertionRequest,
    context: &crate::compile_time::Context<'_>,
    declarations: &ScopedDeclarations<'_>,
    types: &mut TypeRegistry,
    places: &mut PlaceRegistry,
    meta: &mut crate::reflection::MetaContext,
    admission: &InsertionAdmissionCallback<'_>,
) -> Result<DiscoveryInsertionDecision, LocatedDiagnostic> {
    let expression = &request.directive.value;
    if !request.directive.replacements.is_empty() {
        return Err(LocatedDiagnostic {
            location: request.location,
            message: "file declaration insertion cannot replace procedural loop controls".into(),
        });
    }
    if !matches!(
        expression.kind,
        syntax::ExpressionKind::Code(_)
            | syntax::ExpressionKind::Name(_)
            | syntax::ExpressionKind::QualifiedName(_)
            | syntax::ExpressionKind::CompileTime(_)
    ) {
        return Err(LocatedDiagnostic {
            location: request.location,
            message: "declaration insertion producer requires a retained compiler Code query"
                .into(),
        });
    }
    discovery_conditions::with_source_resolver(
        discovery_conditions::SourceRequest {
            file: request.file,
            purpose: discovery_conditions::SourceRequestPurpose::Condition,
            expression,
            lexical: None,
            assertion: None,
            substitution: None,
        },
        context,
        declarations,
        types,
        places,
        meta,
        |resolver| {
            if let syntax::ExpressionKind::CompileTime(run) = &expression.kind {
                let publication = resolver.execute_compiler_code_run(
                    run,
                    expression.span,
                    crate::compile_time::CompilerDestination::Declarations {
                        request,
                        admission,
                    },
                )?;
                return match publication {
                    Some(crate::compile_time::CompilerRunResult::Declarations(decision)) => {
                        Ok(decision)
                    }
                    _ => Err(Diagnostic::at_source(
                        request.location,
                        "declaration insertion #run requires a compiler Code source result",
                    )),
                };
            }
            let crate::Expr::Code(id) = resolver.expr(expression)? else {
                return Err(Diagnostic::at_source(
                    request.location,
                    "declaration insertion requires a captured Code value",
                ));
            };
            let code = resolver.declaration_insertion_code(
                id,
                request.visibility,
                request.directive.scope,
                request.directive.span,
            )?;
            let admission = admission(request.id, &code)
                .map_err(|error| Diagnostic::at_source(request.location, error.to_string()))?;
            Ok(DiscoveryInsertionDecision {
                request: request.id,
                code,
                admission,
            })
        },
    )
}
