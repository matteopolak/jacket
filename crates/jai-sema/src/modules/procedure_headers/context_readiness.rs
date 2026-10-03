//! Keep the original inferred formal until the full source Context is checked.
use super::*;
use std::sync::Arc;

#[derive(Clone)]
pub(in crate::modules) struct ContextHeaderPrerequisite {
    pub(in crate::modules) declaration: DeclarationId,
    file: FileInstanceId,
    parameter: usize,
    location: SourceSpan,
    source: Arc<str>,
}

pub(in crate::modules) enum HeaderPreparation {
    Ready,
    Pending(ContextHeaderPrerequisite),
}

pub(in crate::modules) fn prerequisite(
    graph: &ModuleGraph,
    declaration: &jai_modules::Declaration,
    declarations: &ScopedDeclarations<'_>,
) -> Result<Option<ContextHeaderPrerequisite>, LocatedDiagnostic> {
    let parameters = match &declaration.syntax().kind {
        FileDeclarationKind::Procedure(source) => &source.parameters,
        FileDeclarationKind::ProcedurePrototype(source) => &source.parameters,
        _ => return Ok(None),
    };
    for (parameter, formal) in parameters.iter().enumerate() {
        if formal.evaluation == syntax::ParameterEvaluation::Discard {
            continue;
        }
        let expression = match &formal.binding {
            syntax::ParameterBinding::Defaulted {
                ty: None,
                expression,
            }
            | syntax::ParameterBinding::DefaultedType {
                ty: None,
                expression,
            } => expression,
            _ => continue,
        };
        if !runtime_defaults::needs_context_schema(
            graph,
            declaration.file(),
            expression,
            declarations,
        )? {
            continue;
        }
        return Ok(Some(ContextHeaderPrerequisite {
            declaration: declaration.id(),
            file: declaration.file(),
            parameter,
            location: SourceSpan {
                source: declaration.location().source,
                span: expression.span,
            },
            source: graph
                .sources()
                .get(declaration.location().source)
                .expect("original source header")
                .shared_text(),
        }));
    }
    Ok(None)
}

pub(in crate::modules) fn complete<'graph>(
    graph: &'graph ModuleGraph,
    requests: &[ContextHeaderPrerequisite],
    types: &mut TypeRegistry,
    declarations: &mut ScopedDeclarations<'graph>,
    constants: &mut Constants<'graph>,
    meta: &mut crate::reflection::MetaContext,
) -> Result<(), LocatedDiagnostic> {
    for request in requests {
        request.validate(graph)?;
    }
    // Ordered source registration also retries headers whose inferred types depend
    // on one of these original procedures. No unchecked parameter type is issued.
    register(
        graph,
        types,
        declarations,
        constants,
        meta,
        HeaderPhase::TypesOnly,
    )
}

impl ContextHeaderPrerequisite {
    fn validate(&self, graph: &ModuleGraph) -> Result<(), LocatedDiagnostic> {
        let fail = || LocatedDiagnostic {
            location: self.location,
            message: "retained Context header source authority changed".into(),
        };
        let declaration = graph.declaration(self.declaration).ok_or_else(fail)?;
        let source = graph.sources().get(self.location.source).ok_or_else(fail)?;
        let parameters = match &declaration.syntax().kind {
            FileDeclarationKind::Procedure(source) => &source.parameters,
            FileDeclarationKind::ProcedurePrototype(source) => &source.parameters,
            _ => return Err(fail()),
        };
        let expression = parameters
            .get(self.parameter)
            .and_then(|formal| match &formal.binding {
                syntax::ParameterBinding::Defaulted {
                    ty: None,
                    expression,
                }
                | syntax::ParameterBinding::DefaultedType {
                    ty: None,
                    expression,
                } => Some(expression),
                _ => None,
            })
            .ok_or_else(fail)?;
        if declaration.file() != self.file
            || declaration.location().source != self.location.source
            || expression.span != self.location.span
            || !Arc::ptr_eq(&source.shared_text(), &self.source)
        {
            return Err(fail());
        }
        Ok(())
    }
}
