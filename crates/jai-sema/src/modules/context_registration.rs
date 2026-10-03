//! Register the designated Preload quotation before the Context schema is defined.
use super::*;

pub(super) struct Registration {
    pub file: FileInstanceId,
    pub syntax: syntax::ContextFieldDeclaration,
    pub location: SourceSpan,
}

pub(super) struct BootstrapRegistration {
    pub fields: Vec<Registration>,
    pub consumed: std::collections::HashSet<DeclarationId>,
}

pub(super) fn collect(
    graph: &ModuleGraph,
    constants: &mut Constants<'_>,
) -> Result<BootstrapRegistration, LocatedDiagnostic> {
    let mut result = BootstrapRegistration {
        fields: vec![],
        consumed: std::collections::HashSet::new(),
    };
    let (Some(prelude), Some(_)) = (graph.prelude(), graph.runtime_support()) else {
        return Ok(result);
    };
    let module = graph.module(prelude).expect("designated Preload module");
    let binding = graph.symbols().find("FIRST_ADD_CONTEXT")
        .and_then(|name| module.exports().get(&name))
        .ok_or_else(|| located(graph, module.entry(), Diagnostic::new(Span::new(0, 0),
            "designated Preload must export FIRST_ADD_CONTEXT before Runtime_Support can initialize Context")))?;
    let jai_modules::Binding::Declaration(id) = binding else {
        return Err(located(
            graph,
            module.entry(),
            Diagnostic::new(
                Span::new(0, 0),
                "Preload FIRST_ADD_CONTEXT must be a quoted context declaration",
            ),
        ));
    };
    let declaration = graph.declaration(*id).expect("exported declaration");
    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
        return Err(LocatedDiagnostic {
            location: declaration.location(),
            message: "Preload FIRST_ADD_CONTEXT must be a quoted context declaration".into(),
        });
    };
    if constant.ty.is_some() {
        return Err(LocatedDiagnostic {
            location: declaration.location(),
            message: "Preload FIRST_ADD_CONTEXT quotation cannot have a scalar annotation".into(),
        });
    }
    let syntax::ExpressionKind::Code(body) = &constant.initializer.kind else {
        return Err(LocatedDiagnostic {
            location: declaration.location(),
            message: "Preload FIRST_ADD_CONTEXT must contain structural #code".into(),
        });
    };
    let mut statements: Vec<&syntax::Statement> = match body {
        syntax::CodeBody::Statement(statement) => vec![statement.as_ref()],
        syntax::CodeBody::Block(statements) => statements.iter().rev().collect(),
        syntax::CodeBody::Expression(_) | syntax::CodeBody::Null => {
            return Err(LocatedDiagnostic {
                location: declaration.location(),
                message: "Preload FIRST_ADD_CONTEXT requires quoted #add_context declarations"
                    .into(),
            });
        }
    };
    while let Some(statement) = statements.pop() {
        match &statement.kind {
            syntax::StatementKind::CompileTimeIf { condition, then_body, else_body } => {
                let crate::ScalarConstant::Bool(selected) = constants.evaluate_lazy(declaration.file(), condition)? else {
                    return Err(LocatedDiagnostic {
                        location: SourceSpan { source: declaration.location().source, span: condition.span },
                        message: "bootstrap context #if condition must be an immutable boolean".into(),
                    });
                };
                statements.extend(if selected { then_body } else { else_body }.iter().rev());
            }
            syntax::StatementKind::ContextField(field) => {
                result.fields.push(Registration {
                    file: declaration.file(),
                    syntax: field.clone(),
                    location: SourceSpan { source: declaration.location().source, span: statement.span },
                });
            }
            syntax::StatementKind::Block(block) => statements.extend(block.iter().rev()),
            _ => return Err(LocatedDiagnostic {
                location: declaration.location(),
                message: "Preload FIRST_ADD_CONTEXT bootstrap accepts only structural #add_context declarations".into(),
            }),
        }
    }
    result.consumed.insert(*id);
    Ok(result)
}

#[cfg(test)]
mod tests;
