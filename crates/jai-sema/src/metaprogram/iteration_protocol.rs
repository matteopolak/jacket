//! Select collection protocols with the shared pure matcher and real origins.
use super::*;
use crate::{TypeId, polymorphism::Substitution};

pub(super) struct IterationProtocol {
    pub target: ExpandedTarget,
    pub annotations: Vec<syntax::TypeSyntax>,
    pub source_parameter_ty: TypeId,
    pub substitution: Substitution,
}

impl Resolver<'_> {
    pub(super) fn select_iteration_protocol(
        &mut self,
        path: &syntax::NamePath,
        source_ty: TypeId,
        span: Span,
    ) -> Result<IterationProtocol, Diagnostic> {
        let candidates = self.expanded_candidates(path, span)?;
        if candidates.is_empty() {
            return Err(Diagnostic::new(
                span,
                "custom iteration requires a source #expand procedure",
            ));
        }
        let mut matches = Vec::new();
        let mut viable = Vec::new();
        let mut rejections = Vec::new();
        for target in candidates {
            let annotations = match protocol_annotations(&target.procedure, span) {
                Ok(annotations) => annotations,
                Err(diagnostic) => {
                    rejections.push(crate::overloads::Rejection {
                        declaration: target.id,
                        diagnostic,
                    });
                    continue;
                }
            };
            match self.match_expanded_source_ranked(
                &target,
                &annotations[0],
                target.procedure.parameters[0].span,
                source_ty,
                span,
            ) {
                Ok(selected) => {
                    matches.push(selected.matched);
                    viable.push((target, annotations, selected.expected));
                }
                Err(ExpandedSourceError::Definition(error)) => return Err(error),
                Err(ExpandedSourceError::Mismatch(diagnostic)) => {
                    rejections.push(crate::overloads::Rejection {
                        declaration: target.id,
                        diagnostic,
                    });
                }
            }
        }
        if matches.is_empty() {
            return Err(crate::overloads::SelectionError::NoMatch(rejections).diagnostic(span));
        }
        let selected =
            crate::overloads::select_matches(matches).map_err(|error| error.diagnostic(span))?;
        let (target, annotations, expected) = viable
            .into_iter()
            .find(|(target, _, _)| target.id == selected.declaration)
            .expect("selected match retains a real source candidate");
        Ok(IterationProtocol {
            target,
            annotations,
            source_parameter_ty: expected,
            substitution: selected.substitution,
        })
    }
}

fn protocol_annotations(
    procedure: &syntax::Procedure,
    span: Span,
) -> Result<Vec<syntax::TypeSyntax>, Diagnostic> {
    if procedure.parameters.len() != 3 || !procedure.results.is_empty() {
        return Err(Diagnostic::new(
            span,
            "a for expansion requires three parameters (source, Code body, For_Flags) and no results",
        ));
    }
    if procedure.parameters.iter().any(|parameter| {
        parameter.baking != syntax::ParameterBaking::None || parameter.using || parameter.variadic
    }) {
        return Err(Diagnostic::new(
            span,
            "baked, using, and variadic custom iteration parameters are not implemented",
        ));
    }
    if procedure
        .parameters
        .iter()
        .any(|parameter| parameter.evaluation == syntax::ParameterEvaluation::Discard)
    {
        return Err(Diagnostic::new(
            span,
            "discarded custom iteration parameters are not implemented",
        ));
    }
    procedure
        .parameters
        .iter()
        .map(|parameter| match &parameter.binding {
            syntax::ParameterBinding::Required(ty) => Ok(syntax::TypeSyntax::Builtin(
                syntax::BuiltinType::Scalar(*ty),
            )),
            syntax::ParameterBinding::RequiredType(ty) => Ok(ty.clone()),
            _ => Err(Diagnostic::new(
                span,
                "custom iteration parameters require explicit types and cannot have defaults",
            )),
        })
        .collect()
}
